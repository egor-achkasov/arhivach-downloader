use arhivarch_downloader::config::Config;
use arhivarch_downloader::event::Event;
use arhivarch_downloader::export::{html::HtmlExporter, ExporterKind};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{
        self, DisableMouseCapture, EnableMouseCapture, Event as CEvent,
        KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
    },
    crossterm::execute,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use std::{io::stdout, path::PathBuf, sync::mpsc};

// ── App state ────────────────────────────────────────────────────────────────

enum AppState {
    Input,
    Running,
    Done,
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Url,
    Dir,
    Thumb,
    Files,
    Resume,
    Retries,
}

const FIELDS: &[Field] = &[
    Field::Url,
    Field::Dir,
    Field::Thumb,
    Field::Files,
    Field::Resume,
    Field::Retries,
];

struct App {
    state: AppState,
    url: String,
    dir: String,
    thumb: bool,
    files: bool,
    resume: bool,
    retries: String,
    selected: usize,
    log: Vec<String>,
    rx: Option<mpsc::Receiver<Event>>,
    field_areas: Vec<Rect>,
}

impl App {
    fn new() -> Self {
        App {
            state: AppState::Input,
            url: String::new(),
            dir: String::from("."),
            thumb: false,
            files: false,
            resume: false,
            retries: String::from("3"),
            selected: 0,
            log: Vec::new(),
            rx: None,
            field_areas: vec![Rect::default(); FIELDS.len()],
        }
    }

    fn field(&self) -> Field {
        FIELDS[self.selected]
    }

    fn start(&mut self) {
        let config = Config {
            url: self.url.clone(),
            dir: PathBuf::from(&self.dir),
            exporter: ExporterKind::Html(HtmlExporter),
            thumb: self.thumb,
            files: self.files,
            resume: self.resume,
            download_retries: self.retries.parse().unwrap_or(3),
        };
        let (tx, rx) = mpsc::channel::<Event>();
        self.rx = Some(rx);
        self.state = AppState::Running;
        self.log.clear();
        std::thread::spawn(move || {
            let _ = arhivarch_downloader::run(&config, tx);
        });
    }

    fn poll(&mut self) {
        let Some(rx) = &self.rx else { return };
        loop {
            match rx.try_recv() {
                Ok(ev) => self.log.push(event_label(&ev)),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.state = AppState::Done;
                    break;
                }
            }
        }
    }
}

fn event_label(ev: &Event) -> String {
    match ev {
        Event::GetStarted => "Fetching thread...".into(),
        Event::GetDone => "Thread fetched.".into(),
        Event::GetFailed { error } => format!("ERROR: {error}"),
        Event::DownloadAllStarted => "Downloading assets...".into(),
        Event::DownloadAllDone => "All downloads complete.".into(),
        Event::DownloadAllFailed { error } => format!("Download error: {error}"),
        Event::DownloadStarted { index, max_index } => format!("  [{index}/{max_index}] Downloading..."),
        Event::DownloadDone { index, max_index } => format!("  [{index}/{max_index}] Done."),
        Event::DownloadSkipped { index, max_index } => format!("  [{index}/{max_index}] Skipped."),
        Event::DownloadFailed { url, error } => format!("  Failed {url}: {error}"),
        Event::DownloadFilesStarted => "Downloading files...".into(),
        Event::DownloadFilesDone => "Files downloaded.".into(),
        Event::DownloadThumbStarted => "Downloading thumbnails...".into(),
        Event::DownloadThumbDone => "Thumbnails downloaded.".into(),
        Event::ExportStarted => "Exporting...".into(),
        Event::ExportDone => "Export done.".into(),
        Event::ExportFailed { error } => format!("Export error: {error}"),
    }
}

// ── Drawing ──────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, app: &mut App) {
    match app.state {
        AppState::Input => draw_input(frame, app),
        AppState::Running | AppState::Done => draw_log(frame, app),
    }
}

fn draw_input(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let rows: &[(&str, String, Field)] = &[
        ("URL", app.url.clone(), Field::Url),
        ("Output dir", app.dir.clone(), Field::Dir),
        ("Thumbnails", checkbox_label(app.thumb), Field::Thumb),
        ("Files", checkbox_label(app.files), Field::Files),
        ("Resume", checkbox_label(app.resume), Field::Resume),
        ("Retries", app.retries.clone(), Field::Retries),
    ];

    for (i, (label, value, field)) in rows.iter().enumerate() {
        let active = app.field() == *field;
        let border_style = if active {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(*label)
            .border_style(border_style);
        app.field_areas[i] = chunks[i];
        frame.render_widget(Paragraph::new(value.as_str()).block(block), chunks[i]);
    }

    let hint = "Tab/↑↓: navigate  Space/click: toggle  Enter: start  Ctrl+C: quit";
    frame.render_widget(
        Paragraph::new(hint).block(Block::default().borders(Borders::ALL).title("Help")),
        chunks[6],
    );
}

fn draw_log(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let title = match app.state {
        AppState::Running => "Running",
        AppState::Done => "Done",
        AppState::Input => unreachable!(),
    };

    let items: Vec<ListItem> = app.log.iter().map(|s| ListItem::new(s.as_str())).collect();
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(title)),
        chunks[0],
    );

    let hint = match app.state {
        AppState::Done => "q / Enter: quit",
        _ => "Ctrl+C: quit",
    };
    frame.render_widget(
        Paragraph::new(hint).block(Block::default().borders(Borders::ALL)),
        chunks[1],
    );
}

fn checkbox_label(b: bool) -> String {
    if b { "[x]".into() } else { "[ ]".into() }
}

fn is_bool_field(field: Field) -> bool {
    matches!(field, Field::Thumb | Field::Files | Field::Resume)
}

fn toggle_field(app: &mut App, field: Field) {
    match field {
        Field::Thumb => app.thumb = !app.thumb,
        Field::Files => app.files = !app.files,
        Field::Resume => app.resume = !app.resume,
        _ => {}
    }
}

// ── Event handling ────────────────────────────────────────────────────────────

fn handle_input_key(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Tab | KeyCode::Down => {
            app.selected = (app.selected + 1) % FIELDS.len();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.selected = app.selected.checked_sub(1).unwrap_or(FIELDS.len() - 1);
        }
        KeyCode::Enter => {
            if !app.url.is_empty() {
                app.start();
            }
        }
        KeyCode::Char(' ') => {
            let f = app.field();
            if is_bool_field(f) {
                toggle_field(app, f);
            }
        }
        KeyCode::Char(c) => match app.field() {
            Field::Url => app.url.push(c),
            Field::Dir => app.dir.push(c),
            Field::Retries if c.is_ascii_digit() => app.retries.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app.field() {
            Field::Url => { app.url.pop(); }
            Field::Dir => { app.dir.pop(); }
            Field::Retries => { app.retries.pop(); }
            _ => {}
        },
        _ => {}
    }
}

fn handle_mouse_click(app: &mut App, col: u16, row: u16) {
    for (i, area) in app.field_areas.iter().enumerate() {
        if col >= area.x && col < area.x + area.width
            && row >= area.y && row < area.y + area.height
        {
            if app.selected == i && is_bool_field(FIELDS[i]) {
                toggle_field(app, FIELDS[i]);
            } else {
                app.selected = i;
            }
            break;
        }
    }
}

// ── Main loop ─────────────────────────────────────────────────────────────────

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    execute!(stdout(), EnableMouseCapture)?;
    let result = run_loop(terminal, app);
    execute!(stdout(), DisableMouseCapture)?;
    result
}

fn run_loop(terminal: &mut DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    loop {
        app.poll();
        terminal.draw(|f| draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                CEvent::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        return Ok(());
                    }
                    match app.state {
                        AppState::Input => handle_input_key(app, key),
                        AppState::Running => {}
                        AppState::Done => {
                            if matches!(key.code, KeyCode::Char('q') | KeyCode::Enter) {
                                return Ok(());
                            }
                        }
                    }
                }
                CEvent::Mouse(mouse) => {
                    if matches!(app.state, AppState::Input)
                        && matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
                    {
                        handle_mouse_click(app, mouse.column, mouse.row);
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, &mut App::new());
    ratatui::restore();
    result
}
