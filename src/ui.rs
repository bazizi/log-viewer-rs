use crate::tab::TabType;
use crate::{app::SelectedInput, parser::LogEntryIndices, App, ViewMode};
use ratatui::layout::Margin;
use ratatui::style::Stylize;
use ratatui::widgets::block::Position;
use ratatui::widgets::Scrollbar;
use ratatui::widgets::ScrollbarOrientation;
use ratatui::widgets::ScrollbarState;
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Direction,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::utils::{beatify_enclosed_json, highlight_keywords_in_text};

const DEFAULT_BG_COLOR: Color = Color::DarkGray;
const DEFAULT_HIGHLIGHT_COLOR: Color = Color::LightMagenta;

pub fn render(f: &mut Frame, app: &mut App) {
    let is_in_table_item_mode = matches!(app.view_mode().back(), Some(ViewMode::TableItem(_)));

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if is_in_table_item_mode {
            [Constraint::Percentage(100), Constraint::Length(1)].as_ref()
        } else {
            [
                Constraint::Length(3),      // search/filter
                Constraint::Length(3),      // Tabs
                Constraint::Percentage(10), // preview
                Constraint::Percentage(90), // table
                Constraint::Length(1),      // bottom menu area
            ]
            .as_ref()
        })
        .split(f.size());

    {
        let menu_area = areas[areas.len() - 1];
        const TAIL_PREFIX: &str = "[t]ail ";
        const FILTER_PREFIX: &str = "[f]ilter";
        const SEARCH_PREFIX: &str = "[s]earch";
        const COPY_PREFIX: &str = "[c]opy";
        let menu = [
            "[o]pen",
            &(TAIL_PREFIX.to_owned()
                + if app.tail_enabled() {
                    "(enabled)"
                } else {
                    "(disabled)"
                }),
            SEARCH_PREFIX,
            FILTER_PREFIX,
            COPY_PREFIX,
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
            let mut menu_item = Paragraph::new(menu[i])
                .block(Block::default().borders(Borders::NONE))
                .alignment(ratatui::layout::Alignment::Center)
                .bg(DEFAULT_BG_COLOR);

            let search_focused = matches!(app.selected_input(), Some(SelectedInput::Search));

            if (menu[i].starts_with(TAIL_PREFIX) && (app.tail_enabled()))
                || (menu[i].starts_with(FILTER_PREFIX)
                    && (!app.filter_input_text().to_string().is_empty()))
                || (menu[i].starts_with(SEARCH_PREFIX) && search_focused)
                || (menu[i].starts_with(COPY_PREFIX) && app.copying_to_clipboard())
            {
                menu_item = menu_item.bg(DEFAULT_HIGHLIGHT_COLOR);
            }

            f.render_widget(menu_item, menu_item_area[i]);
        }

        if app.tabs().is_empty() {
            return;
        }
    }

    if is_in_table_item_mode {
        *app.selected_input_mut() = None;
        let items = &app.tabs()[app.selected_tab_index()]
            .filtered_view_items
            .data;
        if items.is_empty() {
            return;
        }

        let item_view_area = areas[0];

        let mut log_text = app.selected_log_entry_in_text();

        if let Some(json_beautified) = beatify_enclosed_json(&log_text) {
            log_text = json_beautified;
        }

        let input_str = app.search_input_text().to_string();
        let t = ratatui::widgets::Paragraph::new(highlight_keywords_in_text(&log_text, input_str))
            .block(
                Block::default()
                    .title(" [Log entry] ")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .borders(Borders::TOP | Borders::BOTTOM),
            )
            .style(Style::default().fg(Color::White).bg(DEFAULT_BG_COLOR))
            .wrap(Wrap { trim: false });
        f.render_widget(t, item_view_area);
        return;
    }

    let (tabs_area, preview_area, table_area) = (areas[1], areas[2], areas[3]);
    *app.table_view_state_mut().position_mut() = Some((table_area.left(), table_area.top()));

    let text = if app.tabs()[app.selected_tab_index()]
        .filtered_view_items
        .data
        .is_empty()
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

    let input_str = app.search_input_text().to_string();
    let text = highlight_keywords_in_text(&text, input_str);

    let preview = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .title(" [Preview] ")
                .title_alignment(ratatui::layout::Alignment::Center),
        )
        .bg(DEFAULT_BG_COLOR);
    f.render_widget(preview, preview_area);

    let input_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(areas[0]);

    let (filter_area, search_area) = (input_area[0], input_area[1]);

    let mut filter = Paragraph::new(app.filter_input_text().to_string())
        .block(Block::default().borders(Borders::ALL).title("[F]ilter"))
        .bg(DEFAULT_BG_COLOR);

    let mut search = Paragraph::new(app.search_input_text().to_string())
        .block(Block::default().borders(Borders::ALL).title("[S]earch"))
        .bg(DEFAULT_BG_COLOR);

    if let Some(SelectedInput::Filter) = &app.selected_input() {
        f.set_cursor(
            filter_area.x + (app.filter_input_text().cursor() as u16) + 1,
            filter_area.y + 1,
        );
        filter = filter.bg(DEFAULT_HIGHLIGHT_COLOR);
    } else if let Some(SelectedInput::Search) = &app.selected_input() {
        f.set_cursor(
            search_area.x + (app.search_input_text().cursor() as u16) + 1,
            search_area.y + 1,
        );
        search = search.bg(DEFAULT_HIGHLIGHT_COLOR);
    }

    f.render_widget(filter, filter_area);
    f.render_widget(search, search_area);

    // Show the file name only in the combined tab
    let column_names = if let TabType::Combined = app.tabs()[app.selected_tab_index()].tab_type {
        ["source", "date", "level", "log"].to_vec()
    } else {
        ["date", "level", "log"].to_vec()
    };

    let header_cells = column_names
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(DEFAULT_BG_COLOR))
        .height(1)
        .bottom_margin(0);

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
    .highlight_style(Style::default().bg(DEFAULT_HIGHLIGHT_COLOR))
    .select(app.selected_tab_index())
    .bg(DEFAULT_BG_COLOR);

    f.render_widget(tabs, tabs_area);

    let selected_tab_index = app.selected_tab_index();
    let rows = {
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

            // Show the file name column only in the combined tab
            let starting_cell =
                if let TabType::Combined = app.tabs()[app.selected_tab_index()].tab_type {
                    LogEntryIndices::FileName as usize
                } else {
                    LogEntryIndices::Date as usize
                };
            let cells = item[starting_cell..item.len()].iter().map(|c: &String| {
                let input_str = app.search_input_text().to_string();
                Cell::from(highlight_keywords_in_text(c, input_str))
            });
            let row = Row::new(cells).height(height as u16);
            let color = match item[LogEntryIndices::Level as usize].as_str() {
                "ERROR" => (Color::Red, Color::White),
                "WARN" => (Color::LightYellow, Color::Black),
                _ => (DEFAULT_BG_COLOR, Color::White),
            };

            row.style(Style::default().bg(color.0).fg(color.1))
        });

        rows
    };

    let column_widts = if let TabType::Combined = app.tabs()[app.selected_tab_index()].tab_type {
        [
            // Show the file name only in the combined tab
            Constraint::Length(13),
            Constraint::Length(24),
            Constraint::Length(6),
            Constraint::Percentage(100),
        ]
        .to_vec()
    } else {
        [
            Constraint::Length(24),
            Constraint::Length(6),
            Constraint::Percentage(100),
        ]
        .to_vec()
    };

    let t = Table::new(rows, column_widts.clone())
        .header(header)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(app.tabs()[app.selected_tab_index()].name.clone()),
        )
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .title(format!(
                    " Page [{}] ",
                    app.tabs()[app.selected_tab_index()]
                        .filtered_view_items
                        .selected_item_index
                        / app.view_buffer_size()
                        + 1
                ))
                .title_position(Position::Bottom),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>")
        .widths(&column_widts);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .style(Style::default().fg(Color::White));

    let filtered_view_items = &app.tabs()[app.selected_tab_index()].filtered_view_items;
    let mut scrollbar_state = ScrollbarState::new(filtered_view_items.data.len())
        .position(filtered_view_items.selected_item_index);

    let mut state = app.table_view_state().state().clone();
    f.render_stateful_widget(t, table_area, &mut state);
    f.render_stateful_widget(
        scrollbar,
        table_area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    *app.table_view_state_mut().state_mut() = state;
}
