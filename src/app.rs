use std::collections::VecDeque;
use std::ops::Range;

use ratatui::widgets::TableState;
use rfd::FileDialog;

use anyhow::Result;

use crate::parser;
use crate::parser::LogEntryIndices;
use log::info;

use crate::tab::Tab;
use crate::tab::TabType;
use crate::tab::TableItems;

const DEFAULT_VIEW_BUFFER_SIZE: usize = 50;
const DEFAULT_SKIP_SIZE: usize = 5;
const COMBINED_TAB_INDEX: usize = 0;

pub enum SelectedInput {
    Filter,
    Search,
}
pub enum ViewMode {
    Table,
    SearchView,
    TableItem(usize /* index */),
}

pub struct App {
    pub running: bool,
    pub state: TableState,

    // we keep a history of view modes to be able to switch back
    pub tabs: Vec<Tab>,
    pub selected_tab_index: usize,
    pub view_mode: VecDeque<ViewMode>, // TODO; Merge selected_input & view_mode together
    selected_input: Option<SelectedInput>,
    filter_input_text: String,
    search_input_text: String,
    view_buffer_size: usize,
    tail_enabled: bool, // TODO: add tailing support
}

impl App {
    pub fn new(file_paths: Vec<String>) -> App {
        // The combined tab goes first
        let mut tabs = vec![Tab::new(
            "".to_owned(),
            TableItems {
                data: vec![],
                selected_item_index: 0,
            },
            TabType::Combined,
        )];

        tabs.append(
            &mut file_paths
                .iter()
                .map(|file_path| {
                    let table_items = TableItems {
                        data: parser::parse_log_by_path(&file_path).unwrap(),
                        selected_item_index: 0,
                    };
                    Tab::new(file_path.to_owned(), table_items, TabType::Normal)
                })
                .collect::<Vec<Tab>>(),
        );

        let mut app = App {
            running: true,
            state: TableState::default(),
            view_mode: vec![ViewMode::Table].into(),
            tabs,
            selected_tab_index: 0,
            selected_input: None,
            filter_input_text: "".to_string(),
            search_input_text: "".to_string(),
            view_buffer_size: DEFAULT_VIEW_BUFFER_SIZE,
            tail_enabled: false,
        };

        app.reload_combined_tab();

        app
    }

    pub fn search_input_text(&self) -> &String {
        return &self.search_input_text;
    }

    pub fn search_input_text_mut(&mut self) -> &mut String {
        return &mut self.search_input_text;
    }

    pub fn filter_input_text(&self) -> &String {
        return &self.filter_input_text;
    }

    pub fn filter_input_text_mut(&mut self) -> &mut String {
        return &mut self.filter_input_text;
    }

    pub fn selected_input(&self) -> &Option<SelectedInput> {
        return &self.selected_input;
    }

    pub fn selected_input_mut(&mut self) -> &mut Option<SelectedInput> {
        return &mut self.selected_input;
    }

    pub fn tail_enabled(&self) -> bool {
        return self.tail_enabled;
    }

    pub fn set_tail_enabled(&mut self, tail_enabled: bool) {
        self.tail_enabled = tail_enabled;
    }

    pub fn reload_combined_tab(&mut self) {
        let tabs = &mut self.tabs;

        let mut all_tab_items = vec![];
        for i in 0..tabs.len() {
            if let TabType::Combined = tabs[i].tab_type {
                continue;
            }

            let mut current_tab_items = tabs[i].filtered_view_items.data.clone();
            all_tab_items.append(&mut current_tab_items);
        }

        tabs[COMBINED_TAB_INDEX].filtered_view_items.data = all_tab_items;

        tabs[COMBINED_TAB_INDEX]
            .filtered_view_items
            .data
            .sort_by(|a, b| {
                return a[LogEntryIndices::Date as usize].cmp(&b[LogEntryIndices::Date as usize]);
            });

        tabs[COMBINED_TAB_INDEX]
            .filtered_view_items
            .selected_item_index = tabs[COMBINED_TAB_INDEX].filtered_view_items.data.len() - 1;

        let items = tabs[COMBINED_TAB_INDEX].filtered_view_items.clone();
        tabs[COMBINED_TAB_INDEX].set_items(items);
    }

    pub fn get_view_buffer_range(&self) -> Range<usize> {
        let items = &self.tabs[self.selected_tab_index].filtered_view_items;
        let num_items = self.tabs[self.selected_tab_index]
            .filtered_view_items
            .data
            .len();

        if self.tabs.is_empty() || items.data.is_empty() {
            return 0..0;
        }

        // gets the view range based on the view_buffer_size
        std::cmp::max(
            items
                .selected_item_index
                .saturating_sub(self.view_buffer_size / 2),
            0,
        )
            ..std::cmp::min(
                items
                    .selected_item_index
                    .saturating_add(3 * self.view_buffer_size / 2)
                    + 1,
                num_items,
            )
    }

    fn calculate_position_in_view_buffer(&self) -> usize {
        if self.tabs.is_empty() {
            return 0;
        }

        let pos = {
            self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index
                - self.get_view_buffer_range().start
        };

        info!("Screen position calculated to be {}", pos);

        pos
    }

    pub fn next(&mut self, search: Option<String>) {
        if self.tabs.is_empty() {
            return;
        }

        let (mut index, items) = (
            self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index,
            &mut self.tabs[self.selected_tab_index].filtered_view_items,
        );

        let new_index = if search.is_none() || search.as_ref().unwrap().is_empty() {
            std::cmp::min(index.saturating_add(1), items.data.len() - 1)
        } else {
            // search mode
            let keywords = search
                .as_ref()
                .unwrap()
                .split(',')
                .map(|keyword| keyword.to_owned())
                .collect::<Vec<String>>();

            let mut final_index = index;
            'outer: loop {
                index = std::cmp::min(index.saturating_add(1), items.data.len() - 1);

                for search_keyword in &keywords {
                    if items.data[index][LogEntryIndices::Log as usize]
                        .to_lowercase()
                        .contains(&search_keyword.to_lowercase())
                    {
                        final_index = index;
                        break 'outer;
                    }
                }

                if index == items.data.len() - 1 {
                    // reached the beginning
                    break;
                }
            }

            final_index
        };

        items.selected_item_index = new_index;

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn previous(&mut self, search: Option<String>) {
        if self.tabs.is_empty() {
            return;
        }

        let (mut index, items) = (
            self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index,
            &mut self.tabs[self.selected_tab_index].filtered_view_items,
        );

        let new_index = if search.is_none() || search.as_ref().unwrap().is_empty() {
            std::cmp::max(index.saturating_sub(1), 0)
        } else {
            let keywords = search
                .as_ref()
                .unwrap()
                .split(',')
                .map(|keyword| keyword.to_owned())
                .collect::<Vec<String>>();
            // search mode
            let mut final_index = index;
            'outer: loop {
                index = std::cmp::max(index.saturating_sub(1), 0);

                for search_keyword in &keywords {
                    if items.data[index][LogEntryIndices::Log as usize]
                        .to_lowercase()
                        .contains(&search_keyword.to_lowercase())
                    {
                        final_index = index;
                        break 'outer;
                    }
                }

                if index == 0 {
                    // reached the beginning
                    break;
                }
            }

            final_index
        };

        items.selected_item_index = new_index;

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn skipping_next(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        {
            let num_items = self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .len();
            let index = &mut self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index;
            *index = std::cmp::min(index.saturating_add(DEFAULT_SKIP_SIZE), num_items - 1);
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn skipping_prev(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        {
            let index = &mut self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index;
            *index = std::cmp::max(index.saturating_sub(DEFAULT_SKIP_SIZE), 0);
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn start(&mut self) {
        self.tabs[self.selected_tab_index]
            .filtered_view_items
            .selected_item_index = 0;

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn end(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        self.tabs[self.selected_tab_index]
            .filtered_view_items
            .selected_item_index = self.tabs[self.selected_tab_index]
            .filtered_view_items
            .data
            .len()
            - 1;

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn switch_to_item_view(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        if !self.filter_input_text.is_empty() {
            self.view_mode.push_back(ViewMode::TableItem(
                self.tabs[self.selected_tab_index]
                    .filtered_view_items
                    .selected_item_index,
            ));
            return;
        }

        match self.view_mode.back() {
            Some(ViewMode::Table) => {
                self.view_mode.push_back(ViewMode::TableItem(
                    self.tabs[self.selected_tab_index]
                        .filtered_view_items
                        .selected_item_index,
                ));
            }
            _ => {}
        }
    }

    pub fn load_files(&mut self) {
        let files = FileDialog::new()
            .add_filter("text", &["txt", "log", "bak"])
            .pick_files();

        if let Some(files) = files {
            for file in files {
                let file_path = file.to_str().unwrap().to_string();
                let table_items = TableItems {
                    data: parser::parse_log_by_path(&file_path).unwrap(),
                    selected_item_index: 0,
                };
                self.tabs
                    .push(Tab::new(file_path, table_items, TabType::Normal));
                self.selected_tab_index = self.tabs.len() - 1;
            }
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        self.selected_tab_index = std::cmp::min(
            self.selected_tab_index.saturating_add(1),
            self.tabs.len() - 1,
        );
        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        self.selected_tab_index = self.selected_tab_index.saturating_sub(1);
        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn update_stale_tabs(&mut self) -> Result<()> {
        if !self.tail_enabled() {
            return Ok(());
        }

        self.filter_by_current_input(self.filter_input_text.clone());

        let mut any_tabs_updated = false;
        for tab in &mut self.tabs {
            if let TabType::Combined = tab.tab_type {
                continue;
            }

            let metadata = std::fs::metadata(&tab.file_path)?;
            let current_file_size = metadata.len().try_into().unwrap_or(0);
            if tab.last_file_size != current_file_size {
                tab.reload()?;
                tab.last_file_size = current_file_size;
                tab.filtered_view_items.selected_item_index =
                    tab.filtered_view_items.data.len() - 1;
                any_tabs_updated = true;
            }
        }

        if any_tabs_updated {
            self.reload_combined_tab();
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));

        Ok(())
    }

    pub fn filter_by_current_input(&mut self, filter: String) {
        for tab in &mut self.tabs {
            tab.reset_filtered_view_items();

            tab.filtered_view_items.data = tab
                .filtered_view_items
                .data
                .iter()
                .filter(|item| {
                    if filter.trim().is_empty() {
                        return true;
                    }
                    let keywords = filter
                        .split(',')
                        .map(|keyword| keyword.to_owned())
                        .collect::<Vec<String>>();

                    let mut include_item = false;
                    for filter_keyword in &keywords {
                        include_item = include_item
                            || item[LogEntryIndices::Log as usize]
                                .to_lowercase()
                                .contains(filter_keyword.to_lowercase().as_str());

                        if include_item {
                            break;
                        }
                    }

                    include_item
                })
                .map(|item| item.clone())
                .collect::<Vec<Vec<String>>>();

            tab.filtered_view_items.selected_item_index = if self.tail_enabled {
                tab.filtered_view_items.data.len() - 1
            } else {
                0
            };
        }
    }
}
