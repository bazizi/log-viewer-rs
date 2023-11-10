use crate::{App, ViewMode};

use ratatui::{
    layout::{Constraint, Layout},
    prelude::Direction,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_cells = ["index", "date", "pid", "tid", "level", "log"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Cyan))
        .height(1)
        .bottom_margin(0);
    let rows = app.tabs[app.tab_index].items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(&**c));
        let row = Row::new(cells).height(height as u16);
        if app.state.selected() == Some(usize::from_str_radix(&item[0], 10).unwrap()) {
            row
        } else {
            let color = match item[4].as_str() {
                "ERROR" => (Color::Red, Color::White),
                "WARN" => (Color::LightYellow, Color::Black),
                _ => (Color::Black, Color::White),
            };

            row.style(Style::default().bg(color.0).fg(color.1))
        }
    });

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
    .select(app.tab_index);

    f.render_widget(tabs, rects[0]);

    match app.view_mode {
        ViewMode::TableItem(item) => {
            let t = ratatui::widgets::Paragraph::new(&*app.tabs[app.tab_index].items[item][5])
                .block(Block::default().title("Log entry").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .wrap(Wrap { trim: true });
            f.render_widget(t, rects[0]);
        }
        _ => {
            let t = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(app.tabs[app.tab_index].file_path.clone()),
                )
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Length(6),
                    Constraint::Length(24),
                    Constraint::Length(6),
                    Constraint::Length(6),
                    Constraint::Length(6),
                    Constraint::Percentage(100),
                ]);
            f.render_stateful_widget(t, rects[1], &mut app.state);
        }
    }
}
