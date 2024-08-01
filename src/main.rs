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
    env,
    io::{self, Write},
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use anyhow::Result;
use std::io::stdout;
use std::io::Read;

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

    info!("Reading port number from file...");
    if let Ok(port_num) = std::fs::read_to_string("log_viewer_port") {
        info!("Attempting connection to port {}", port_num);
        if let Ok(mut conn) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port_num)) {
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
    let connections_thread = {
        let app_clone = Arc::clone(&app);
        thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            std::fs::write(
                "log_viewer_port",
                listener.local_addr().unwrap().port().to_string(),
            )
            .unwrap();
            listener.set_nonblocking(true).unwrap();
            for stream in listener.incoming() {
                if !app_clone.lock().unwrap().running() {
                    break;
                }
                match stream {
                    Ok(mut stream) => {
                        info!(
                            "reading from stream: {}",
                            stream.peer_addr().unwrap().to_string()
                        );

                        // do something with the TcpStream
                        stream
                            .set_read_timeout(Some(Duration::from_millis(100)))
                            .unwrap();
                        let mut file_path = String::new();
                        stream.read_to_string(&mut file_path).unwrap();
                        let table_items = tab::TableItems {
                            data: parser::parse_log_by_path(&file_path).unwrap_or_default(),
                            selected_item_index: 0,
                        };
                        let mut app_lock = app_clone.lock().unwrap();
                        app_lock.tabs_mut().push(tab::Tab::new(
                            file_path.to_string(),
                            table_items,
                            tab::TabType::Normal,
                        ));
                        *app_lock.selected_tab_index_mut() = app_lock.tabs().len() - 1;
                        app_lock.reload_combined_tab();
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // wait until network socket is ready, typically implemented
                        // via platform-specific APIs such as epoll or IOCP
                        sleep(Duration::from_millis(300));
                        continue;
                    }
                    Err(e) => panic!("encountered IO error: {e}"),
                }
            }
        })
    };

    while *app.lock().unwrap().running() {
        update(&events_thread, app.clone())?;
        terminal.draw(|f| render(f, &mut app.lock().unwrap()))?;
        std::thread::sleep(std::time::Duration::from_millis(1000 / FPS));
    }

    connections_thread.join().unwrap();
    file_monitor_thread.join();
    events_thread.join();

    Ok(())
}
