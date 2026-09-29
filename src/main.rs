//! port-killer – find the processes that use a port and kill them from a terminal UI.
//!
//! Reads the socket table via `netstat2`, shows the owning processes in a
//! ratatui table and force-kills the selected one via `sysinfo`.

mod ports;

use anyhow::Result;
use ports::Entry;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Row, Table, TableState};
use ratatui::{DefaultTerminal, Frame};
use std::process::exit;

const USAGE: &str = "\
Find and kill processes by port in an interactive terminal UI.

Usage: port-killer [OPTIONS] [PORT]

Lists every process with a TCP or UDP socket on PORT and lets you kill
the selected one. Without PORT you are asked for one.

Options:
  -h, --help     Print help
  -V, --version  Print version";

enum Mode {
    Input,
    List,
}

/// Work that runs after the next draw, so its status message is visible meanwhile.
enum Action {
    Search,
    Kill(usize),
}

enum Tone {
    Info,
    Ok,
    Err,
}

struct App {
    mode: Mode,
    port: String,
    entries: Vec<Entry>,
    table: TableState,
    status: (String, Tone),
    pending: Option<Action>,
    quit: bool,
}

impl App {
    fn new(port: Option<u16>) -> Self {
        let mut app = App {
            mode: Mode::Input,
            port: String::new(),
            entries: Vec::new(),
            table: TableState::default(),
            status: (String::new(), Tone::Info),
            pending: None,
            quit: false,
        };
        match port {
            Some(p) => {
                app.port = p.to_string();
                app.start_search();
            }
            None => app.reset(),
        }
        app
    }

    fn reset(&mut self) {
        self.mode = Mode::Input;
        self.port.clear();
        self.entries.clear();
        self.table.select(None);
        self.status = ("Enter a port and press Enter.".into(), Tone::Info);
    }

    fn start_search(&mut self) {
        match self.port.parse::<u16>() {
            Ok(p) if p > 0 => {
                self.mode = Mode::List;
                self.status = (format!("Searching port {p}..."), Tone::Info);
                self.pending = Some(Action::Search);
            }
            _ => self.status = ("Enter a port between 1 and 65535.".into(), Tone::Err),
        }
    }

    fn perform(&mut self, action: Action) {
        match action {
            Action::Search => self.search(),
            Action::Kill(i) => {
                let e = &self.entries[i];
                let label = format!("{} (PID {})", e.name, e.pid);
                match ports::kill(e.pid) {
                    Ok(()) => {
                        self.search();
                        self.status = (format!("Killed {label}."), Tone::Ok);
                    }
                    Err(err) => self.status = (format!("Could not kill {label}: {err}"), Tone::Err),
                }
            }
        }
    }

    fn search(&mut self) {
        let port = self.port.parse().unwrap_or_default();
        match ports::find(port) {
            Ok(entries) => {
                self.entries = entries;
                self.status = match self.entries.len() {
                    0 => (format!("Port {port} is not in use."), Tone::Info),
                    n => (format!("{n} socket(s) on port {port}."), Tone::Info),
                };
            }
            Err(err) => {
                self.entries.clear();
                self.status = (format!("{err:#}"), Tone::Err);
            }
        }
        // Keep the cursor in place after a kill, clamped to the new length.
        let selected = self.table.selected().unwrap_or(0);
        self.table.select(match self.entries.len() {
            0 => None,
            n => Some(selected.min(n - 1)),
        });
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        match self.mode {
            Mode::Input => match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() && self.port.len() < 5 => self.port.push(c),
                KeyCode::Backspace => {
                    self.port.pop();
                }
                KeyCode::Enter if !self.port.is_empty() => self.start_search(),
                KeyCode::Esc | KeyCode::Char('q') => self.quit = true,
                _ => {}
            },
            Mode::List => match key.code {
                KeyCode::Up | KeyCode::Char('k') => self.table.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => self.table.select_next(),
                KeyCode::Home => self.table.select_first(),
                KeyCode::End => self.table.select_last(),
                KeyCode::Enter => {
                    if let Some(i) = self.table.selected().filter(|&i| i < self.entries.len()) {
                        let e = &self.entries[i];
                        self.status =
                            (format!("Killing {} (PID {})...", e.name, e.pid), Tone::Info);
                        self.pending = Some(Action::Kill(i));
                    }
                }
                KeyCode::Char('r') => self.start_search(),
                KeyCode::Esc => self.reset(),
                KeyCode::Char('q') => self.quit = true,
                _ => {}
            },
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [header, body, status, help] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        let title = Line::from(vec![
            " PORT KILLER".bold().cyan(),
            " – find and kill processes by port".gray(),
        ]);
        frame.render_widget(
            Paragraph::new(title).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Cyan),
            ),
            header,
        );

        match self.mode {
            Mode::Input => self.draw_input(frame, body),
            Mode::List => self.draw_list(frame, body),
        }

        let (text, tone) = &self.status;
        let color = match tone {
            Tone::Info => Color::Gray,
            Tone::Ok => Color::Green,
            Tone::Err => Color::Red,
        };
        frame.render_widget(Line::from(format!(" {text}")).fg(color), status);

        let keys: &[(&str, &str, Color)] = match self.mode {
            Mode::Input => &[
                ("0-9", "port", Color::Cyan),
                ("Enter", "search", Color::Green),
                ("Esc/q", "quit", Color::Red),
            ],
            Mode::List => &[
                ("↑↓", "select", Color::Cyan),
                ("Enter", "kill", Color::Green),
                ("r", "refresh", Color::Cyan),
                ("Esc", "new search", Color::Yellow),
                ("q", "quit", Color::Red),
            ],
        };
        let spans = keys.iter().flat_map(|&(key, what, color)| {
            [
                Span::from(format!(" {key}")).fg(color),
                Span::from(format!(" {what} ")).dim(),
            ]
        });
        frame.render_widget(Line::from_iter(spans), help);
    }

    fn draw_input(&self, frame: &mut Frame, area: Rect) {
        let [input, _] = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);
        let text = Line::from(vec![" Port: ".into(), self.port.as_str().bold().yellow()]);
        frame.render_widget(
            Paragraph::new(text).block(
                Block::bordered()
                    .title(" Port input ".bold())
                    .border_style(Color::Yellow),
            ),
            input,
        );
        // 1 for the left border + 7 for " Port: "
        let x = input.x + 1 + 7 + self.port.len() as u16;
        frame.set_cursor_position(Position::new(x, input.y + 1));
    }

    fn draw_list(&mut self, frame: &mut Frame, area: Rect) {
        let count = self.entries.len();
        let block = Block::bordered()
            .title(Line::from(vec![
                " Port ".into(),
                self.port.as_str().bold().green(),
                format!(" · {count} socket{} ", if count == 1 { "" } else { "s" }).gray(),
            ]))
            .border_style(Color::Green);

        if count == 0 {
            let empty = if matches!(self.pending, Some(Action::Search)) {
                "Searching network connections..."
            } else {
                "No process uses this port."
            };
            frame.render_widget(
                Paragraph::new(format!(" {empty}")).gray().block(block),
                area,
            );
            return;
        }

        let header = Row::new(["PROCESS", "PID", "PROTO", "ADDRESS", "STATE"])
            .style(Style::new().bold().cyan())
            .bottom_margin(1);
        let rows = self.entries.iter().map(|e| {
            Row::new([
                e.name.clone().white(),
                e.pid.to_string().yellow(),
                e.protocol.green(),
                e.local.clone().cyan(),
                e.state.clone().magenta(),
            ])
        });
        let table = Table::new(
            rows,
            [
                Constraint::Min(20),
                Constraint::Length(8),
                Constraint::Length(6),
                Constraint::Min(22),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::new()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");
        frame.render_stateful_widget(table, area, &mut self.table);
    }
}

fn run(terminal: &mut DefaultTerminal, mut app: App) -> Result<()> {
    while !app.quit {
        terminal.draw(|f| app.draw(f))?;
        if let Some(action) = app.pending.take() {
            app.perform(action);
            continue;
        }
        // Windows also reports key releases; only react to presses.
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.on_key(key);
            }
        }
    }
    Ok(())
}

fn main() {
    let mut port = None;
    if let Some(arg) = std::env::args().nth(1) {
        match arg.as_str() {
            "-h" | "--help" => return println!("{USAGE}"),
            "-V" | "--version" => return println!("port-killer {}", env!("CARGO_PKG_VERSION")),
            _ => match arg.parse::<u16>() {
                Ok(p) if p > 0 => port = Some(p),
                _ => {
                    eprintln!("error: invalid port '{arg}'\n\n{USAGE}");
                    exit(2);
                }
            },
        }
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, App::new(port));
    ratatui::restore();
    if let Err(e) = result {
        eprintln!("error: {e:#}");
        exit(1);
    }
}
