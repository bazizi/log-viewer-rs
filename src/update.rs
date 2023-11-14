use crate::{app::SelectedInput, event::EventHandler, parser::LogEntryIndices, App, ViewMode};

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

            let mut filter_by_current_input = |current_input: &String| {
                // User is typing in filter mode so reset the cursors and update filtered items
                app.tabs[app.selected_tab_index].selected_filtered_view_item_index = 0;
                app.tabs[app.selected_tab_index].filtered_view_items = app.tabs
                    [app.selected_tab_index]
                    .items
                    .iter()
                    .filter(|item| {
                        current_input.trim().is_empty()
                            || item[LogEntryIndices::LOG as usize]
                                .to_lowercase()
                                .contains(current_input.to_lowercase().as_str())
                    })
                    .map(|item| item.clone())
                    .collect::<Vec<Vec<String>>>();
            };

            if let Some(ViewMode::FilteredView) = app.view_mode.back() {
                if let Some(SelectedInput::Filter(current_input)) = &mut app.selected_input {
                    match key.code {
                        KeyCode::Char(c) => {
                            current_input.push(c);
                            filter_by_current_input(&current_input);
                        }
                        KeyCode::Backspace => {
                            current_input.pop();
                            filter_by_current_input(&current_input);
                        }
                        KeyCode::Enter => app.switch_to_item_view(),

                        // Arrow keys to select filtered items
                        // We can't support Vim style bindings in this mode because the users might actually be typing j, k, etc.
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::PageDown => app.page_down(),
                        KeyCode::PageUp => app.page_up(),
                        KeyCode::Home => app.start(),
                        KeyCode::End => app.end(),

                        KeyCode::Esc => {
                            if let Some(ViewMode::TableItem(_)) = app.view_mode.back() {
                                app.view_mode.pop_back();
                            } else if let Some(ViewMode::FilteredView) = app.view_mode.back() {
                                app.view_mode.pop_back();
                                app.selected_input = None;
                            }
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                return Ok(());
            } else if let Some(ViewMode::FilteredView) = app.view_mode.back() {
                if let Some(SelectedInput::Search(current_input)) = &mut app.selected_input {
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
                }
                return Ok(());
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => match app.view_mode.back() {
                    Some(ViewMode::Table) => {
                        app.running = false;
                        return Ok(());
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
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::PageDown => app.page_down(),
                KeyCode::PageUp => app.page_up(),

                KeyCode::Char('f') => {
                    app.selected_input = Some(SelectedInput::Filter("".to_owned()));
                    app.view_mode.push_back(ViewMode::FilteredView);
                }
                KeyCode::Char('s') => {
                    app.selected_input = Some(SelectedInput::Search("".to_owned()));
                    app.view_mode.push_back(ViewMode::SearchView);
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
