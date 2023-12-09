use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
};

use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io, sync::Arc};

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

mod tab;

mod file_monitor;
use file_monitor::FileMonitor;

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

    let args = env::args();
    let args = args.into_iter().collect::<Vec<String>>();
    execute!(
        io::stdout(),
        SetTitle(
            args.iter()
                .map(|e| e.clone())
                .reduce(|acc, e| { acc.to_string() + &e })
                .unwrap()
        )
    )?;

    // create app and run it
    let app = std::sync::Arc::new(std::sync::Mutex::new(App::new(
        args.iter()
            .skip(1)
            .map(|item| item.clone())
            .collect::<Vec<String>>(),
    )));

    let _file_monitor_thread = FileMonitor::new(Arc::clone(&app));
    let events = EventHandler::new(250);

    while *app.lock().unwrap().running() {
        terminal.draw(|f| render(f, &mut app.lock().unwrap()))?;
        update(&events, &mut app.lock().unwrap())?;
    }

    Ok(())
}
