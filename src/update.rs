use crate::{app::SelectedInput, event::EventHandler, parser::LogEntryIndices, App, ViewMode};

use crossterm::event::{KeyCode, KeyEvent};

use crate::event::Event;
use anyhow::Result;

use log::info;

pub fn update(events: &EventHandler, app: &mut App) -> Result<()> {
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

    if let Some(ViewMode::FilteredView) = app.view_mode.back() {
        if app.selected_input.is_some() {
            handle_filtered_mode(key.code, app);
        }
        return;
    } else if let Some(ViewMode::SearchView) = app.view_mode.back() {
        if app.selected_input.is_some() {
            handle_search_mode(key.code, app);
        }
        return;
    }

    handle_normal_mode(key.code, app);
}

fn filter_by_current_input(current_input: String, app: &mut App, search_all: bool) {
    // User is typing in filter mode so reset the cursors and update filtered items
    app.tabs[app.selected_tab_index].selected_filtered_view_item_index = 0;

    let items = if search_all {
        &app.tabs[app.selected_tab_index].items
    } else {
        &app.tabs[app.selected_tab_index].filtered_view_items
    };

    app.tabs[app.selected_tab_index].filtered_view_items = items
        .iter()
        .filter(|item| {
            current_input.trim().is_empty()
                || item[LogEntryIndices::LOG as usize]
                    .to_lowercase()
                    .contains(current_input.to_lowercase().as_str())
        })
        .map(|item| item.clone())
        .collect::<Vec<Vec<String>>>();
}

fn handle_filtered_mode(key_code: KeyCode, app: &mut App) {
    if let Some(SelectedInput::Filter(current_input)) = &mut app.selected_input {
        match key_code {
            KeyCode::Char(c) => {
                current_input.push(c);
                filter_by_current_input(current_input.clone(), app, false);
            }
            KeyCode::Backspace => {
                current_input.pop();
                filter_by_current_input(current_input.clone(), app, true);
            }
            KeyCode::Enter => app.switch_to_item_view(),

            // Arrow keys to select filtered items
            // We can't support Vim style bindings in this mode because the users might actually be typing j, k, etc.
            KeyCode::Down => app.next(None),
            KeyCode::Up => app.previous(None),
            KeyCode::PageDown => app.skipping_next(),
            KeyCode::PageUp => app.skipping_prev(),
            KeyCode::Home => app.start(),
            KeyCode::End => app.end(),

            KeyCode::Esc => {
                if let Some(ViewMode::TableItem(_)) | Some(ViewMode::FilteredView) =
                    app.view_mode.back()
                {
                    app.view_mode.pop_back();
                    app.selected_input = None;
                    app.tabs[app.selected_tab_index].filtered_view_items.clear();
                }
            }
            _ => {}
        }
    }
}

fn handle_search_mode(key_code: KeyCode, app: &mut App) {
    if let Some(SelectedInput::Search(current_input)) = &mut app.selected_input {
        let current_input_copy = current_input.clone();

        match key_code {
            KeyCode::Char(c) => {
                current_input.push(c);
            }
            KeyCode::Backspace => {
                current_input.pop();
            }
            // Arrow keys to select filtered items
            // We can't support Vim style bindings in this mode because the users might actually be typing j, k, etc.
            KeyCode::Down => app.next(Some(current_input_copy)),
            KeyCode::Up => app.previous(Some(current_input_copy)),
            KeyCode::PageDown => app.skipping_next(),
            KeyCode::PageUp => app.skipping_prev(),
            KeyCode::Home => app.start(),
            KeyCode::End => app.end(),

            KeyCode::Esc => {
                app.selected_input = None;
                app.view_mode.pop_back();
            }
            _ => {}
        }
    }
}

fn handle_normal_mode(key_code: KeyCode, app: &mut App) {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => match app.view_mode.back() {
            Some(ViewMode::Table) => {
                app.running = false;
                return;
            }
            _ => {
                app.view_mode.pop_back();
                match app.view_mode.back() {
                    Some(ViewMode::Table) => {
                        app.selected_input = None;
                    }
                    _ => {}
                }
            }
        },
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
            if app.tabs.is_empty() {
                return;
            }

            app.selected_input = Some(SelectedInput::Filter("".to_owned()));
            app.tabs[app.selected_tab_index].filtered_view_items =
                app.tabs[app.selected_tab_index].items.clone();
            app.view_mode.push_back(ViewMode::FilteredView);
        }
        KeyCode::Char('s') => {
            if app.tabs.is_empty() {
                return;
            }

            app.selected_input = Some(SelectedInput::Search("".to_owned()));
            app.view_mode.push_back(ViewMode::SearchView);
        }
        _ => {}
    }
}
