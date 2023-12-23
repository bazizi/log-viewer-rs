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

mod input_element;
mod utils;

const FPS: u64 = 60;

#[macro_use]
extern crate lazy_static;

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
                .cloned()
                .reduce(|acc, e| { acc.to_string() + " " + &e })
                .unwrap()
        )
    )?;

    // create app and run it
    let app = std::sync::Arc::new(std::sync::Mutex::new(App::new(
        args.iter()
            .skip(1)
            .filter(|item| !item.is_empty())
            .cloned()
            .collect::<Vec<String>>(),
    )));

    let file_monitor_thread = FileMonitor::new(Arc::clone(&app));
    let events_thread = EventHandler::new();

    while *app.lock().unwrap().running() {
        update(&events_thread, &mut app.lock().unwrap())?;
        terminal.draw(|f| render(f, &mut app.lock().unwrap()))?;
        std::thread::sleep(std::time::Duration::from_millis(1000 / FPS));
    }

    events_thread.join();
    file_monitor_thread.join();

    Ok(())
}
