use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
};

use log::info;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    env, fs,
    io::{self, Write},
    sync::Arc
};

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

mod thirdparty;
mod utils;

mod net;
use net::NetHandler;

const FPS: u64 = 60;

const CONFIGS_PATH: &str = "log-viewer-rs";
const PORT_FILE: &str = "log_viewer_port";
const LOCALHOST_IPV4: &str = "127.0.0.1";

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

    fs::create_dir_all(format!(
        "{}/{}",
        std::env::var("LOCALAPPDATA").unwrap(),
        CONFIGS_PATH,
    ))
    .unwrap();

    info!("Reading port number from file...");
    if let Ok(port_num) = std::fs::read_to_string(format!(
        "{}/{}/{}",
        std::env::var("LOCALAPPDATA").unwrap(),
        CONFIGS_PATH,
        PORT_FILE
    )) {
        info!("Attempting connection to port {}", port_num);
        if let Ok(mut conn) =
            std::net::TcpStream::connect(format!("{}:{}", LOCALHOST_IPV4, port_num))
        {
            info!("Successfully connected to port {}", port_num);
            let args = env::args();
            let args = args.into_iter().collect::<Vec<String>>();
            conn.write_all(args.get(1).unwrap().as_bytes()).unwrap();
            panic!("Redirected to running instance");
        } else {
            info!("Could not connect to port {}", port_num);
        }
    } else {
        info!("No port file found");
    }

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
    let events_thread = EventHandler::new();
    let file_monitor_thread = FileMonitor::new(Arc::clone(&app), events_thread.sender.clone());
    let connections_thread = NetHandler::new(Arc::clone(&app));

    while *app.lock().unwrap().running() {
        update(&events_thread, app.clone())?;
        terminal.draw(|f| render(f, &mut app.lock().unwrap()))?;
        std::thread::sleep(std::time::Duration::from_millis(1000 / FPS));
    }

    connections_thread.shutdown();
    file_monitor_thread.shutdown();
    events_thread.shutdown();

    Ok(())
}
