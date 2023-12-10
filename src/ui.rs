use crate::tab::TabType;
use crate::{app::SelectedInput, parser::LogEntryIndices, App, ViewMode};

use ratatui::layout::Margin;
use ratatui::widgets::Scrollbar;
use ratatui::widgets::ScrollbarOrientation;
use ratatui::widgets::ScrollbarState;

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
                Constraint::Length(3),      // top menu area
                Constraint::Length(3),      // search/filter
                Constraint::Length(3),      // Tabs
                Constraint::Percentage(10), // preview
                Constraint::Percentage(90), // table
            ]
            .as_ref(),
        )
        .split(f.size());

    {
        let menu_area = areas[0];
        let menu = [
            "[o]pen",
            &("[t]ail ".to_owned()
                + if app.tail_enabled() {
                    "(enabled)"
                } else {
                    "(disabled)"
                }),
            "[s]earch",
            "[f]ilter",
            "[c]opy",
            "move [Arrow keys]",
            "select [enter]",
            "[b]ack [Esc]",
            "close tab [x]",
            "[q]uit",
        ];

        let menu_item_constraints = menu
            .iter()
            .map(|_| Constraint::Percentage(100 / menu.len() as u16))
            .collect::<Vec<Constraint>>();

        let menu_item_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(menu_item_constraints)
            .split(menu_area);

        for i in 0..menu.len() {
            let borders = if i == menu.len() - 1 {
                Borders::ALL
            } else {
                Borders::LEFT | Borders::TOP | Borders::BOTTOM
            };

            let menu_item = Paragraph::new(menu[i])
                .block(Block::default().borders(borders))
                .alignment(ratatui::layout::Alignment::Center)
                .on_blue();
            f.render_widget(menu_item, menu_item_area[i]);
        }

        if app.tabs().is_empty() {
            return;
        }
    }

    let (tabs_area, preview_area, table_area) = (areas[2], areas[3], areas[4]);

    let text = if app.tabs()[app.selected_tab_index()]
        .filtered_view_items
        .data
        .len()
        == 0
    {
        "".to_owned()
    } else {
        app.tabs()[app.selected_tab_index()]
            .filtered_view_items
            .data[app.tabs()[app.selected_tab_index()]
            .filtered_view_items
            .selected_item_index][LogEntryIndices::Log as usize]
            .clone()
    };
    let preview = Paragraph::new(text).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .title(" [Preview] ")
            .title_alignment(ratatui::layout::Alignment::Center),
    );
    f.render_widget(preview, preview_area);

    let input_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(areas[1]);

    let (filter_area, search_area) = (input_area[0], input_area[1]);

    if let Some(SelectedInput::Filter) = &app.selected_input() {
        f.set_cursor(
            filter_area.x + (app.filter_input_text().len() as u16) + 1,
            filter_area.y + 1,
        );
    } else if let Some(SelectedInput::Search) = &app.selected_input() {
        f.set_cursor(
            search_area.x + (app.search_input_text().len() as u16) + 1,
            search_area.y + 1,
        );
    }

    let filter = Paragraph::new(app.filter_input_text().clone())
        .block(Block::default().borders(Borders::ALL).title("[F]ilter"));
    f.render_widget(filter, filter_area);

    let search = Paragraph::new(app.search_input_text().clone())
        .block(Block::default().borders(Borders::ALL).title("[S]earch"));
    f.render_widget(search, search_area);

    // Show the file name only in the combined tab
    let column_names = if let TabType::Combined = app.tabs()[app.selected_tab_index()].tab_type {
        ["source", "date", "tid", "level", "log"].to_vec()
    } else {
        ["date", "tid", "level", "log"].to_vec()
    };

    let header_cells = column_names
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Cyan))
        .height(1)
        .bottom_margin(0);

    let (is_in_table_item_mode, item) = match app.view_mode().back() {
        Some(ViewMode::TableItem(item)) => (true, Some(*item)),
        _ => (false, None),
    };

    if is_in_table_item_mode {
        *app.selected_input_mut() = None;
        let items = &app.tabs()[app.selected_tab_index()]
            .filtered_view_items
            .data;
        if items.is_empty() {
            return;
        }

        let t =
            ratatui::widgets::Paragraph::new(&*items[item.unwrap()][LogEntryIndices::Log as usize])
                .block(
                    Block::default()
                        .title("Log entry")
                        .borders(Borders::TOP | Borders::BOTTOM),
                )
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .wrap(Wrap { trim: false });
        f.render_widget(t, table_area);
        return;
    }

    let tabs = ratatui::widgets::Tabs::new(
        app.tabs()
            .iter()
            .map(|tab| tab.name.clone())
            .collect::<Vec<String>>(),
    )
    .block(Block::default().title("Tabs").borders(Borders::ALL))
    .style(Style::default().white())
    .highlight_style(Style::default().yellow())
    .divider(ratatui::symbols::bar::FULL)
    .highlight_style(Style::default().on_cyan())
    .select(app.selected_tab_index());

    f.render_widget(tabs, tabs_area);

    let selected_tab_index = app.selected_tab_index();
    let (rows, num_items) = {
        let tabs = &app.tabs();
        let items = &tabs[selected_tab_index].filtered_view_items.data;
        if items.is_empty() {
            return;
        }

        let rows = items[app.get_view_buffer_range()].iter().map(|item| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;

            // Show the file name only in the combined tab
            let starting_cell =
                if let TabType::Combined = app.tabs()[app.selected_tab_index()].tab_type {
                    LogEntryIndices::FileName as usize
                } else {
                    LogEntryIndices::Date as usize
                };
            let cells = item[starting_cell..item.len()]
                .iter()
                .map(|c| Cell::from(&**c));
            let row = Row::new(cells).height(height as u16);
            let color = match item[LogEntryIndices::Level as usize].as_str() {
                "ERROR" => (Color::Red, Color::White),
                "WARN" => (Color::LightYellow, Color::Black),
                _ => (Color::Black, Color::White),
            };

            row.style(Style::default().bg(color.0).fg(color.1))
        });

        let num_rows = rows.len();
        (rows, num_rows)
    };

    let column_widts = if let TabType::Combined = app.tabs()[app.selected_tab_index()].tab_type {
        [
            // Show the file name only in the combined tab
            Constraint::Length(13),
            Constraint::Length(24),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Percentage(100),
        ]
        .to_vec()
    } else {
        [
            Constraint::Length(24),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Percentage(100),
        ]
        .to_vec()
    };

    let t = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.tabs()[app.selected_tab_index()].name.clone()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>")
        .widths(&column_widts);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state = ScrollbarState::new(num_items).position(
        app.tabs()[app.selected_tab_index()]
            .filtered_view_items
            .selected_item_index,
    );

    let mut state = app.state().clone();
    f.render_stateful_widget(t, table_area, &mut state);
    f.render_stateful_widget(
        scrollbar,
        table_area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    *app.state_mut() = state;
}
