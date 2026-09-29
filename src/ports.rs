//! Looks up the processes that use a local port and kills them.

use anyhow::{bail, Context, Result};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// One socket on the searched port and the process that owns it.
pub struct Entry {
    pub pid: u32,
    pub name: String,
    pub protocol: &'static str,
    pub local: String,
    pub state: String,
}

/// Returns every TCP/UDP socket whose local port is `port`, one entry per owning process.
pub fn find(port: u16) -> Result<Vec<Entry>> {
    let sockets = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP,
    )
    .context("could not read the socket table")?;

    let mut entries = Vec::new();
    for socket in sockets.into_iter().filter(|s| s.local_port() == port) {
        let local = SocketAddr::new(socket.local_addr(), port).to_string();
        let (protocol, state) = match &socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => ("TCP", tcp.state.to_string()),
            ProtocolSocketInfo::Udp(_) => ("UDP", String::new()),
        };
        // PID 0 marks sockets without an owner, e.g. TIME_WAIT on Windows.
        for pid in socket.associated_pids.into_iter().filter(|&p| p != 0) {
            entries.push(Entry {
                pid,
                name: String::new(),
                protocol,
                local: local.clone(),
                state: state.clone(),
            });
        }
    }

    // A full refresh also names processes we may not open, e.g. services on Windows.
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    for e in &mut entries {
        e.name = match sys.process(Pid::from_u32(e.pid)) {
            Some(p) => p.name().to_string_lossy().into_owned(),
            None => "unknown".into(),
        };
    }

    entries.sort_by(|a, b| (a.pid, a.protocol, &a.local).cmp(&(b.pid, b.protocol, &b.local)));
    entries.dedup_by(|a, b| {
        (a.pid, a.protocol, &a.local, &a.state) == (b.pid, b.protocol, &b.local, &b.state)
    });
    Ok(entries)
}

/// Force-kills `pid` and waits up to a second for it to disappear.
pub fn kill(pid: u32) -> Result<()> {
    let pid = Pid::from_u32(pid);
    let mut sys = System::new();
    let refresh = |sys: &mut System| {
        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing(),
        )
    };

    refresh(&mut sys);
    let Some(process) = sys.process(pid) else {
        return Ok(()); // already gone
    };
    if !process.kill() {
        if cfg!(windows) {
            bail!("access denied, try running as administrator");
        }
        bail!("permission denied, try running with sudo");
    }

    // Wait so the refreshed list no longer shows the process.
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        refresh(&mut sys);
        if sys.process(pid).is_none() {
            break;
        }
        sleep(Duration::from_millis(50));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn finds_own_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let entries = find(port).unwrap();
        let own = entries.iter().find(|e| e.pid == std::process::id());
        let own = own.expect("own process not found on its listening port");
        assert_eq!(own.protocol, "TCP");
        assert_eq!(own.local, format!("127.0.0.1:{port}"));
        assert!(!own.name.is_empty());
    }
}
