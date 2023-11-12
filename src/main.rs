use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io};

use anyhow::Result;
use std::io::stdout;

mod parser;

/// Application.
mod app;
use crate::app::App;
use crate::app::ViewMode;

/// Widget renderer.
mod ui;
use crate::ui::render;

/// Application updater.
mod update;
use crate::update::update;

mod event;
use crate::event::EventHandler;

fn main() -> Result<()> {
    startup()?;
    let status = run();
    shutdown()?;
    status?;
    Ok(())
}

fn startup() -> Result<()> {
    env_logger::init();

    // setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}

fn run() -> Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let events = EventHandler::new(1);

    let args = env::args();
    let args = args.into_iter().collect::<Vec<String>>();

    // create app and run it
    let mut app = App::new(if args.len() == 2 {
        Some(&args[1])
    } else {
        None
    });

    while app.running {
        terminal.draw(|f| render(f, &mut app))?;
        update(&events, &mut app)?;
    }

    Ok(())
}
