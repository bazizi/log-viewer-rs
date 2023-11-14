use crate::{app::SelectedInput, parser::LogEntryIndices, App, ViewMode};

use ratatui::{
    layout::{Constraint, Layout},
    prelude::Direction,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (tabs_area, table_area) = (areas[1], areas[2]);

    let input_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(areas[0]);

    let (filter_area, search_area) = (input_area[0], input_area[1]);

    let mut filter_text = None;
    let mut search_text = None;
    if let Some(SelectedInput::Filter(_filter_text)) = &app.selected_input {
        filter_text = Some(_filter_text.clone());
        f.set_cursor(
            filter_area.x + (_filter_text.len() as u16) + 1,
            filter_area.y + 1,
        );
    } else if let Some(SelectedInput::Search(_search_text)) = &app.selected_input {
        search_text = Some(_search_text.clone());
        f.set_cursor(
            search_area.x + (_search_text.len() as u16) + 1,
            search_area.y + 1,
        );
    }

    let filter = Paragraph::new(filter_text.clone().unwrap_or("".to_owned()))
        .block(Block::default().borders(Borders::ALL).title("[F]ilter"));
    f.render_widget(filter, filter_area);

    let search = Paragraph::new(search_text.unwrap_or("".to_owned()))
        .block(Block::default().borders(Borders::ALL).title("[S]earch"));
    f.render_widget(search, search_area);

    let header_cells = ["index", "date", "pid", "tid", "level", "log"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Cyan))
        .height(1)
        .bottom_margin(0);

    let mut items = &app.tabs[app.selected_tab_index].items;

    if let Some(ViewMode::FilteredView) = app.view_mode.back() {
        // we're in filtered view mode so show filtered items instead of all items
        items = &app.tabs[app.selected_tab_index].filtered_view_items;
    } else if let Some(ViewMode::TableItem(_)) = app.view_mode.back() {
        if app.view_mode.len() >= 2 {
            if let Some(ViewMode::FilteredView) = app.view_mode.get(app.view_mode.len() - 2) {
                // We're viewing a filtered view item
                items = &app.tabs[app.selected_tab_index].filtered_view_items;
            }
        }
    }

    match app.view_mode.back() {
        Some(ViewMode::TableItem(item)) => {
            let t = ratatui::widgets::Paragraph::new(&*items[*item][LogEntryIndices::LOG as usize])
                .block(Block::default().title("Log entry").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .wrap(Wrap { trim: false });
            f.render_widget(t, table_area);
        }
        _ => {
            let tabs = ratatui::widgets::Tabs::new(
                app.tabs
                    .iter()
                    .map(|tab| tab.file_path.clone())
                    .collect::<Vec<String>>(),
            )
            .block(Block::default().title("Tabs").borders(Borders::ALL))
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .divider(ratatui::symbols::bar::FULL)
            .select(app.selected_tab_index);

            f.render_widget(tabs, tabs_area);

            let rows = items[app.get_view_buffer_range()].iter().map(|item| {
                let height = item
                    .iter()
                    .map(|content| content.chars().filter(|c| *c == '\n').count())
                    .max()
                    .unwrap_or(0)
                    + 1;
                let cells = item.iter().map(|c| Cell::from(&**c));
                let row = Row::new(cells).height(height as u16);
                let color = match item[LogEntryIndices::LEVEL as usize].as_str() {
                    "ERROR" => (Color::Red, Color::White),
                    "WARN" => (Color::LightYellow, Color::Black),
                    _ => (Color::Black, Color::White),
                };

                row.style(Style::default().bg(color.0).fg(color.1))
            });

            let t = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(app.tabs[app.selected_tab_index].file_path.clone()),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Percentage(0),
                    Constraint::Length(24),
                    Constraint::Length(6),
                    Constraint::Length(6),
                    Constraint::Length(6),
                    Constraint::Percentage(100),
                ]);
            f.render_stateful_widget(t, table_area, &mut app.state);
        }
    }
}
