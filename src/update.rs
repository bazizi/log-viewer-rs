use crate::{App, ViewMode};

use crossterm::event::{self, Event, KeyCode};

use std::io;

pub fn update(app: &mut App) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != crossterm::event::KeyEventKind::Press {
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
