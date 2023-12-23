use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent, MouseEventKind};
use std::sync::mpsc;
use std::thread;

use anyhow::Result;

use log::info;

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Terminal event handler.
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread.
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                loop {
                    if event::poll(std::time::Duration::from_secs(1)).expect("no events available")
                    {
                        match event::read().expect("unable to read event") {
                            CrosstermEvent::Key(e) => {
                                info!("Sending key event ...");
                                sender.send(Event::Key(e))
                            }
                            CrosstermEvent::Mouse(e) => {
                                if e.kind == MouseEventKind::Moved {
                                    // avoid sending mouse move events as it can get too spammy
                                    continue;
                                }

                                sender.send(Event::Mouse(e))
                            }
                            CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                            _ => sender.send(Event::Tick),
                        }
                        .expect("failed to send terminal event")
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}
