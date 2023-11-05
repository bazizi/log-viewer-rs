use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Wrap},
    Frame, Terminal,
};
use std::{env, error::Error, io};

mod parser;
use parser::log_parser::LogEntry;

enum ViewMode {
    Table,
    TableItem(usize /* index */),
}

struct App {
    state: TableState,
    view_mode: ViewMode,
    items: Vec<LogEntry>,
    file_path: String,
}

impl App {
    fn new(file_path: &String) -> App {
        App {
            state: TableState::default(),
            view_mode: ViewMode::Table,
            items: parser::log_parser::parse_log_by_path(&file_path, 0).unwrap(),
            file_path: file_path.clone(),
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn start(&mut self) {
        self.state.select(Some(0));
    }

    pub fn end(&mut self) {
        self.state.select(Some(self.items.len() - 1));
    }

    pub fn page_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 21 {
                    self.items.len() - 1
                } else {
                    i + 20
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn page_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= 20 {
                    0
                } else {
                    i - 20
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
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
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let args = env::args();
    let args = args.into_iter().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("\n\nUsage: {} <path/to/file>", args[0]);
        return Ok(());
    }

    // create app and run it
    let app = App::new(&args[1]);
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
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(5)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_cells = ["index", "date", "pid", "tid", "level", "log"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Cyan))
        .height(1)
        .bottom_margin(0);
    let rows = app.items.iter().map(|item| {
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

    match app.view_mode {
        ViewMode::TableItem(item) => {
            let t = ratatui::widgets::Paragraph::new(&*app.items[item][5])
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
                        .title(app.file_path.clone()),
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
            f.render_stateful_widget(t, rects[0], &mut app.state);
        }
    }
}
