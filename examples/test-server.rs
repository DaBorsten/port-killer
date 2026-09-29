//! Minimal HTTP server for trying out port-killer.
//!
//!   cargo run --example test-server -- 3000 Server-1

use std::io::{Read, Write};
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let port: u16 = args.next().and_then(|p| p.parse().ok()).unwrap_or(3000);
    let name = args.next().unwrap_or_else(|| "Server".into());

    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!(
        "{name} listening on http://127.0.0.1:{port} (PID {})",
        std::process::id()
    );

    for mut stream in listener.incoming().flatten() {
        let _ = stream.read(&mut [0; 1024]);
        let body = format!("{name} running on port {port}");
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
    }
    Ok(())
}
