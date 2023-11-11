use crate::{app::SelectedInput, App, ViewMode};

use crossterm::event::{self, Event, KeyCode};

use std::io;

pub fn update(app: &mut App) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {
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
                    app.should_quit = true;
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

            KeyCode::Char('f') => app.selected_input = Some(SelectedInput::Filter("".to_owned())),
            KeyCode::Char('s') => app.selected_input = Some(SelectedInput::Search("".to_owned())),
            _ => {}
        }
    }

    if let Event::Mouse(mouse_event) = event::read()? {
        if mouse_event.kind == crossterm::event::MouseEventKind::ScrollUp {
            app.previous();
        } else if mouse_event.kind == crossterm::event::MouseEventKind::ScrollDown {
            app.next();
        }
    }

    Ok(())
}
