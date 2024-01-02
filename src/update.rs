use std::sync::{Arc, Mutex};

use crate::{app::SelectedInput, event::EventHandler, tab::TabType, App, ViewMode};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};

use crate::event::Event;
use anyhow::Result;

use log::info;

pub fn update(events: &EventHandler, app: Arc<Mutex<App>>) -> Result<()> {
    *app.lock().unwrap().copying_to_clipboard_mut() = false;

    match events.next()? {
        // To avoid deadlock, the app mutex must be locked after the event is read
        // This is so that we can process events from the file monitor (which also locks the mutex)
        Event::Tick => {}
        Event::Key(key) => {
            let mut app = app.lock().unwrap();
            handle_key_press(key, &mut app);
        }
        Event::Mouse(mouse_event) => {
            let mut app = app.lock().unwrap();
            if mouse_event.kind == MouseEventKind::ScrollUp {
                app.previous(None);
            } else if mouse_event.kind == MouseEventKind::ScrollDown {
                app.next(None);
            } else if let MouseEventKind::Down(mouse_button) = mouse_event.kind {
                match mouse_button {
                    MouseButton::Left => {
                        *app.mouse_position_mut() = (mouse_event.column, mouse_event.row);
                        app.handle_table_mouse_click();
                    }
                    MouseButton::Right => {
                        if app.view_mode().len() > 1 {
                            app.view_mode_mut().pop_back();
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::Resize(_, _) => {}
    }

    Ok(())
}

fn handle_key_press(key: KeyEvent, app: &mut App) {
    info!("Received key event ...");
    if key.kind != crossterm::event::KeyEventKind::Press {
        return;
    }

    if let Some(SelectedInput::Filter) = app.selected_input() {
        handle_filtered_mode(key.code, key.modifiers, app);
        return;
    } else if let Some(ViewMode::SearchView) = app.view_mode().back() {
        handle_search_mode(key.code, key.modifiers, app);
        return;
    }

    handle_normal_mode(key.code, app, key.modifiers);
}

fn handle_filtered_mode(key_code: KeyCode, key_modifiers: KeyModifiers, app: &mut App) {
    if let Some(SelectedInput::Filter) = &mut app.selected_input() {
        match key_code {
            KeyCode::Char(c) => {
                if c.to_lowercase().eq('c'.to_lowercase())
                    && key_modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL
                {
                    // Exiting out of filter mode using Ctrl-C
                    if let Some(ViewMode::TableItem(_)) = app.view_mode().back() {
                        app.view_mode_mut().pop_back();
                    } else {
                        *app.selected_input_mut() = None;
                    }
                    return;
                }
                app.filter_input_text_mut().add_char(c);
                app.filter_by_current_input(app.filter_input_text().text().clone());
            }
            KeyCode::Backspace => {
                if key_modifiers == KeyModifiers::CONTROL {
                    app.filter_input_text_mut().clear();
                } else {
                    app.filter_input_text_mut().remove_char();
                }
                app.filter_by_current_input(app.filter_input_text().text().clone());
            }
            KeyCode::Delete => {
                app.filter_input_text_mut().delete_char();
            }
            KeyCode::Enter => app.switch_to_item_view(),

            // Arrow keys to select filtered items
            // We can't support Vim style bindings in this mode because the users might actually be typing j, k, etc.
            KeyCode::Down => app.next(None),
            KeyCode::Up => app.previous(None),
            KeyCode::Left => app.filter_input_text_mut().cursor_left(),
            KeyCode::Right => app.filter_input_text_mut().cursor_right(),
            KeyCode::PageDown => app.skipping_next(),
            KeyCode::PageUp => app.skipping_prev(),
            KeyCode::Home => app.start(),
            KeyCode::End => app.end(),

            KeyCode::Esc => {
                if let Some(ViewMode::TableItem(_)) = app.view_mode().back() {
                    app.view_mode_mut().pop_back();
                } else {
                    *app.selected_input_mut() = None;
                }
            }
            _ => {
                app.filter_by_current_input(app.filter_input_text().text().clone());
            }
        }
    }
}

fn handle_search_mode(key_code: KeyCode, key_modifiers: KeyModifiers, app: &mut App) {
    if let Some(SelectedInput::Search) = &mut app.selected_input() {
        match key_code {
            KeyCode::Char(c) => {
                if c.to_lowercase().eq('c'.to_lowercase())
                    && key_modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL
                {
                    // Exiting out of search mode using Ctrl-C
                    *app.selected_input_mut() = None;
                    app.view_mode_mut().pop_back();
                    return;
                }
                app.search_input_text_mut().add_char(c);
            }
            KeyCode::Backspace => {
                if key_modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
                    app.search_input_text_mut().clear();
                } else {
                    app.search_input_text_mut().remove_char();
                }
            }
            KeyCode::Delete => {
                app.search_input_text_mut().delete_char();
            }
            // Arrow keys to select filtered items
            // We can't support Vim style bindings in this mode because the users might actually be typing j, k, etc.
            KeyCode::Down => app.next(Some(app.search_input_text().text().clone())),
            KeyCode::Up => app.previous(Some(app.search_input_text().text().clone())),
            KeyCode::Enter => {
                if key_modifiers & KeyModifiers::SHIFT == KeyModifiers::SHIFT {
                    app.previous(Some(app.search_input_text().text().clone()));
                } else if key_modifiers == KeyModifiers::NONE {
                    app.next(Some(app.search_input_text().text().clone()));
                }
            }
            KeyCode::Left => app.search_input_text_mut().cursor_left(),
            KeyCode::Right => app.search_input_text_mut().cursor_right(),
            KeyCode::PageDown => app.skipping_next(),
            KeyCode::PageUp => app.skipping_prev(),
            KeyCode::Home => app.start(),
            KeyCode::End => app.end(),
            KeyCode::Esc => {
                *app.selected_input_mut() = None;
                app.view_mode_mut().pop_back();
            }
            _ => {}
        }
    }
}

fn handle_normal_mode(key_code: KeyCode, app: &mut App, key_modifiers: KeyModifiers)
{
    match key_code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            *app.running_mut() = false;
        }
        KeyCode::Char('G') | KeyCode::Char('g') => {
            if let KeyModifiers::SHIFT = key_modifiers {
                // Vim style binding to go to the end of log
                app.end();
            }
        }
        KeyCode::Char('x') => {
            if app.tabs().is_empty() {
                return;
            } else if let TabType::Combined = app.tabs()[app.selected_tab_index()].tab_type {
                return;
            }

            let index_to_remove = app.selected_tab_index();
            if app.selected_tab_index() == app.tabs().len() - 1 {
                *app.selected_tab_index_mut() = app.selected_tab_index().saturating_sub(1);
            }

            app.tabs_mut().remove(index_to_remove);
            app.reload_combined_tab();
        }
        KeyCode::Char('b') | KeyCode::Esc => {
            if app.view_mode().len() > 1 {
                app.view_mode_mut().pop_back();
            }
        }
        KeyCode::Home => app.start(),
        KeyCode::End => app.end(),
        KeyCode::Enter => app.switch_to_item_view(),
        KeyCode::Char('o') => app.load_files(),
        KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
        KeyCode::Left | KeyCode::Char('h') => app.prev_tab(),
        KeyCode::Down | KeyCode::Char('j') => app.next(None),
        KeyCode::Up | KeyCode::Char('k') => app.previous(None),
        KeyCode::PageDown => app.skipping_next(),
        KeyCode::PageUp => app.skipping_prev(),

        KeyCode::Char('f') => {
            if app.tabs().is_empty() {
                return;
            }

            *app.selected_input_mut() = Some(SelectedInput::Filter);
        }
        KeyCode::Char('s') => {
            if app.tabs().is_empty() {
                return;
            }

            *app.selected_input_mut() = Some(SelectedInput::Search);
            app.view_mode_mut().push_back(ViewMode::SearchView);
        }
        KeyCode::Char('t') => {
            app.set_tail_enabled(!app.tail_enabled());
        }
        KeyCode::Char('c') => {
            copy_to_clipboard(&app
                              .selected_log_entry_in_text()
                              .replace(['>', '<', '|'], ""));
                *app.copying_to_clipboard_mut() = true;

        }
        KeyCode::Char('{') => {
            if let KeyModifiers::SHIFT = key_modifiers {
                app.skipping_prev()
            }
        }
        KeyCode::Char('}') => {
            if let KeyModifiers::SHIFT = key_modifiers {
                app.skipping_next()
            }
        }
        _ => {}
    }
}

#[cfg(target_os = "linux")]
fn copy_to_clipboard(_log_str: &str)
{
    unimplemented!();
}

#[cfg(target_os = "windows")]
fn copy_to_clipboard(log_str: &str)
{
    // temporary hack: remove pipe operators as escaping them doesn't work in CMD
    std::process::Command::new("cmd")
        .args(["/C", format!("echo {log_str} | clip.exe").as_str()])
        .output()
        .unwrap();
}
