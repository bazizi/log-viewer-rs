use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    prelude::Direction,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Wrap},
    Frame, Terminal,
};
use std::{env, error::Error, io};

mod parser;
use parser::log_parser::LogEntry;

use rfd::FileDialog;

enum ViewMode {
    Table,
    TableItem(usize /* index */),
}

struct Tab {
    file_path: String,
    items: Vec<LogEntry>,
    selected_item: usize,
}

struct App {
    state: TableState,
    view_mode: ViewMode,

    // TODO: persist state info
    tabs: Vec<Tab>,
    tab_index: usize,
}

impl App {
    fn new(file_path: Option<&String>) -> App {
        if let Some(file_path) = file_path {
            App {
                state: TableState::default(),
                view_mode: ViewMode::Table,
                tabs: vec![Tab {
                    file_path: file_path.clone(),
                    items: parser::log_parser::parse_log_by_path(&file_path, 0).unwrap(),
                    selected_item: 0,
                }],
                tab_index: 0,
            }
        } else {
            App {
                // TODO: Add a help page on startup
                state: TableState::default(),
                view_mode: ViewMode::Table,
                tabs: vec![Tab {
                    file_path: "Help".to_owned(),
                    items: vec![],
                    selected_item: 0,
                }],
                tab_index: 0,
            }
        }
    }
    pub fn next(&mut self) {
        self.tabs[self.tab_index].selected_item = match self.state.selected() {
            Some(i) => {
                if i >= self.tabs[self.tab_index].items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn previous(&mut self) {
        self.tabs[self.tab_index].selected_item = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tabs[self.tab_index].items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn start(&mut self) {
        self.tabs[self.tab_index].selected_item = 0;
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn end(&mut self) {
        self.tabs[self.tab_index].selected_item = self.tabs[self.tab_index].items.len() - 1;
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn page_down(&mut self) {
        self.tabs[self.tab_index].selected_item = match self.state.selected() {
            Some(i) => {
                if i >= self.tabs[self.tab_index].items.len() - 21 {
                    self.tabs[self.tab_index].items.len() - 1
                } else {
                    i + 20
                }
            }
            None => 0,
        };
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn page_up(&mut self) {
        self.tabs[self.tab_index].selected_item = match self.state.selected() {
            Some(i) => {
                if i <= 20 {
                    0
                } else {
                    i - 20
                }
            }
            None => 0,
        };
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn switch_to_item_view(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i,
            None => 0,
        };

        match self.view_mode {
            ViewMode::Table => self.view_mode = ViewMode::TableItem(i),
            _ => {}
        }
    }

    pub fn load_files(&mut self) {
        let file = FileDialog::new()
            .add_filter("text", &["txt", "log", "bak"])
            .pick_file()
            .unwrap();
        let file_path = file.to_str().unwrap().to_string();
        self.tabs.push(Tab {
            items: parser::log_parser::parse_log_by_path(&file_path, 0).unwrap(),
            file_path: file_path,
            selected_item: 0,
        });
        self.tab_index = self.tabs.len() - 1;
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tabs.len();
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let args = env::args();
    let args = args.into_iter().collect::<Vec<String>>();

    // create app and run it
    let app = App::new(if args.len() == 2 {
        Some(&args[1])
    } else {
        None
    });
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::<B>(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') => match app.view_mode {
                    ViewMode::Table => return Ok(()),
                    _ => {
                        app.view_mode = ViewMode::Table;
                    }
                },
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Home => app.start(),
                KeyCode::End => app.end(),
                KeyCode::PageDown => app.page_down(),
                KeyCode::PageUp => app.page_up(),
                KeyCode::Enter => app.switch_to_item_view(),
                KeyCode::Char('o') => app.load_files(),
                KeyCode::Right => app.next_tab(),

                // VIm style bindings
                KeyCode::Char('j') => app.next(),
                KeyCode::Char('k') => app.previous(),
                _ => {}
            }
        }

        if let Event::Mouse(mouse_event) = event::read()? {
            if mouse_event.kind == crossterm::event::MouseEventKind::ScrollUp {
                app.previous()
            } else if mouse_event.kind == crossterm::event::MouseEventKind::ScrollDown {
                app.next()
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_cells = ["index", "date", "pid", "tid", "level", "log"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Cyan))
        .height(1)
        .bottom_margin(0);
    let rows = app.tabs[app.tab_index].items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(&**c));
        let row = Row::new(cells).height(height as u16);
        if app.state.selected() == Some(usize::from_str_radix(&item[0], 10).unwrap()) {
            row
        } else {
            let color = match item[4].as_str() {
                "ERROR" => (Color::Red, Color::White),
                "WARN" => (Color::LightYellow, Color::Black),
                _ => (Color::Black, Color::White),
            };

            row.style(Style::default().bg(color.0).fg(color.1))
        }
    });

    let tabs = ratatui::widgets::Tabs::new(
        app.tabs
            .iter()
            .map(|tab| tab.file_path.clone())
            .collect::<Vec<String>>(),
    )
    .block(Block::default().title("Tabs").borders(Borders::ALL))
    .style(Style::default().white())
    .highlight_style(Style::default().yellow())
    .divider(ratatui::symbols::bar::FULL)
    .select(app.tab_index);

    f.render_widget(tabs, rects[0]);

    match app.view_mode {
        ViewMode::TableItem(item) => {
            let t = ratatui::widgets::Paragraph::new(&*app.tabs[app.tab_index].items[item][5])
                .block(Block::default().title("Log entry").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .wrap(Wrap { trim: true });
            f.render_widget(t, rects[0]);
        }
        _ => {
            let t = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(app.tabs[app.tab_index].file_path.clone()),
                )
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Length(6),
                    Constraint::Length(24),
                    Constraint::Length(6),
                    Constraint::Length(6),
                    Constraint::Length(6),
                    Constraint::Percentage(100),
                ]);
            f.render_stateful_widget(t, rects[1], &mut app.state);
        }
    }
}
