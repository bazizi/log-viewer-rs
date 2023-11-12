use crate::{app::SelectedInput, event::EventHandler, App, ViewMode};

use crossterm::event::KeyCode;

use crate::event::Event;
use anyhow::Result;

use log::info;

pub fn update(events: &EventHandler, app: &mut App) -> Result<()> {
    match events.next()? {
        Event::Tick => {}
        Event::Key(key) => {
            info!("Received key event ...");
            if key.kind != crossterm::event::KeyEventKind::Press {
                return Ok(());
            }

            if let Some(SelectedInput::Filter(current_input)) = &mut app.selected_input {
                match key.code {
                    KeyCode::Char(c) => {
                        current_input.push(c);
                    }
                    KeyCode::Backspace => {
                        current_input.pop();
                    }
                    KeyCode::Esc => app.selected_input = None,
                    _ => {}
                }
                return Ok(());
            } else if let Some(SelectedInput::Search(current_input)) = &mut app.selected_input {
                match key.code {
                    KeyCode::Char(c) => {
                        current_input.push(c);
                    }
                    KeyCode::Backspace => {
                        current_input.pop();
                    }
                    KeyCode::Esc => app.selected_input = None,
                    _ => {}
                }
                return Ok(());
            }

            match key.code {
                KeyCode::Char('q') => match app.view_mode {
                    ViewMode::Table => {
                        app.running = false;
                        return Ok(());
                    }
                    _ => {
                        app.view_mode = ViewMode::Table;
                    }
                },
                KeyCode::Home => app.start(),
                KeyCode::End => app.end(),
                KeyCode::PageDown => app.page_down(),
                KeyCode::PageUp => app.page_up(),
                KeyCode::Enter => app.switch_to_item_view(),
                KeyCode::Char('o') => app.load_files(),
                KeyCode::Right => app.next_tab(),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),

                KeyCode::Char('f') => {
                    app.selected_input = Some(SelectedInput::Filter("".to_owned()))
                }
                KeyCode::Char('s') => {
                    app.selected_input = Some(SelectedInput::Search("".to_owned()))
                }
                _ => {}
            }
        }
        Event::Mouse(mouse_event) => {
            if mouse_event.kind == crossterm::event::MouseEventKind::ScrollUp {
                app.previous();
            } else if mouse_event.kind == crossterm::event::MouseEventKind::ScrollDown {
                app.next();
            }
        }
        Event::Resize(_, _) => {}
    }

    Ok(())
}
