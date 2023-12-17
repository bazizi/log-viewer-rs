use crate::{app::SelectedInput, event::EventHandler, tab::TabType, App, ViewMode};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::event::Event;
use anyhow::Result;

use log::info;

pub fn update(events: &EventHandler, app: &mut App) -> Result<()> {
    *app.copying_to_clipboard_mut() = false;

    match events.next()? {
        Event::Tick => {}
        Event::Key(key) => {
            handle_key_press(key, app);
        }
        Event::Mouse(mouse_event) => {
            if mouse_event.kind == crossterm::event::MouseEventKind::ScrollUp {
                app.previous(None);
            } else if mouse_event.kind == crossterm::event::MouseEventKind::ScrollDown {
                app.next(None);
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

    handle_normal_mode(key.code, app);
}

fn handle_filtered_mode(key_code: KeyCode, key_modifiers: KeyModifiers, app: &mut App) {
    if let Some(SelectedInput::Filter) = &mut app.selected_input() {
        match key_code {
            KeyCode::Char(c) => {
                app.filter_input_text_mut().push(c);
                app.filter_by_current_input(app.filter_input_text().clone());
            }
            KeyCode::Backspace => {
                if key_modifiers == KeyModifiers::CONTROL {
                    app.filter_input_text_mut().clear();
                } else {
                    app.filter_input_text_mut().pop();
                }
                app.filter_by_current_input(app.filter_input_text().clone());
            }
            KeyCode::Enter => app.switch_to_item_view(),

            // Arrow keys to select filtered items
            // We can't support Vim style bindings in this mode because the users might actually be typing j, k, etc.
            KeyCode::Down => app.next(None),
            KeyCode::Up => app.previous(None),
            KeyCode::Left => app.prev_tab(),
            KeyCode::Right => app.next_tab(),
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
                app.filter_by_current_input(app.filter_input_text().clone());
            }
        }
    }
}

fn handle_search_mode(key_code: KeyCode, key_modifiers: KeyModifiers, app: &mut App) {
    if let Some(SelectedInput::Search) = &mut app.selected_input() {
        match key_code {
            KeyCode::Char(c) => {
                app.search_input_text_mut().push(c);
            }
            KeyCode::Backspace => {
                if key_modifiers == KeyModifiers::CONTROL {
                    app.search_input_text_mut().clear();
                } else {
                    app.search_input_text_mut().pop();
                }
            }
            // Arrow keys to select filtered items
            // We can't support Vim style bindings in this mode because the users might actually be typing j, k, etc.
            KeyCode::Down => app.next(Some(app.search_input_text().clone())),
            KeyCode::Up => app.previous(Some(app.search_input_text().clone())),
            KeyCode::Left => app.prev_tab(),
            KeyCode::Right => app.next_tab(),
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

fn handle_normal_mode(key_code: KeyCode, app: &mut App) {
    match key_code {
        KeyCode::Char('q') => {
            *app.running_mut() = false;
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
        KeyCode::Right => app.next_tab(),
        KeyCode::Left => app.prev_tab(),
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
            // Currently only windows supported
            if !cfg!(windows) {
                return;
            }

            // temporary hack: remove pipe operators as escaping them doesn't work in CMD
            let log_str = app
                .selected_log_entry_in_text()
                .replace('>', "")
                .replace('<', "")
                .replace('|', "");
            std::process::Command::new("cmd")
                .args(["/C", format!("echo {log_str} | clip.exe").as_str()])
                .output()
                .unwrap();
            *app.copying_to_clipboard_mut() = true;
            return;
        }
        _ => {}
    }
}
