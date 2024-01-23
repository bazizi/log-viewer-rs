use std::collections::VecDeque;
use std::io::Read;
use std::ops::Range;
use std::vec;

use ratatui::widgets::TableState;
use rfd::FileDialog;

use crate::parser;
use crate::parser::LogEntryIndices;
use log::info;

use crate::input_element::InputTextElement;
use crate::tab::Tab;
use crate::tab::TabType;
use crate::tab::TableItems;

use serde_json::{json, Value};
use std::io::prelude::Write;

const DEFAULT_VIEW_BUFFER_SIZE: usize = 50;
const DEFAULT_SKIP_SIZE: usize = 5;
const COMBINED_TAB_INDEX: usize = 0;
const CONFIG_FILE_NAME: &str = "log-viewer-rs-config.json";

pub enum SelectedInput {
    Filter,
    Search,
}
pub enum ViewMode {
    Table,
    SearchView,
    TableItem(usize /* index */),
}

pub struct TableViewState {
    state: TableState,
    position: Option<(u16, u16)>,
}

impl TableViewState {
    pub fn state(&self) -> &TableState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    pub fn position(&self) -> &Option<(u16, u16)> {
        &self.position
    }

    pub fn position_mut(&mut self) -> &mut Option<(u16, u16)> {
        &mut self.position
    }
}

pub struct App {
    running: bool,
    table_view_state: TableViewState,

    // we keep a history of view modes to be able to switch back
    tabs: Vec<Tab>,
    selected_tab_index: usize,
    view_mode: VecDeque<ViewMode>,
    selected_input: Option<SelectedInput>,
    filter_input_text: InputTextElement,
    search_input_text: InputTextElement,
    view_buffer_size: usize,
    tail_enabled: bool,
    copying_to_clipboard: bool,
    mouse_position: (u16, u16),
    last_key_input: Option<char>,
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
                        data: parser::parse_log_by_path(file_path).unwrap_or_default(),
                        selected_item_index: 0,
                    };
                    Tab::new(file_path.to_owned(), table_items, TabType::Normal)
                })
                .collect::<Vec<Tab>>(),
        );

        // Load config file saved the last session before exit
        if let Ok(mut config_file) = std::fs::File::open(CONFIG_FILE_NAME) {
            let mut str_config_file = String::new();
            if config_file.read_to_string(&mut str_config_file).is_ok() {
                let mut json_config_file: Value = serde_json::from_str(&str_config_file).unwrap();
                tabs.append(
                    &mut json_config_file["tabs"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .filter(|file_path| !file_path.to_string().is_empty())
                        .map(|file_path| {
                            let mut file_path = file_path.to_string();

                            // sometimes command line quotes are included so here we strip the out
                            if file_path.starts_with('"') && file_path.len() > 1 {
                                file_path = file_path[1..].to_string();
                            }

                            if file_path.ends_with('"') && file_path.len() > 1 {
                                file_path = file_path[..file_path.len() - 1].to_string();
                            }

                            let table_items = TableItems {
                                data: parser::parse_log_by_path(&file_path).unwrap_or_default(),
                                selected_item_index: 0,
                            };
                            Tab::new(file_path, table_items, TabType::Normal)
                        })
                        .collect::<Vec<Tab>>(),
                );
            }
        }

        let mut app = App {
            running: true,
            table_view_state: TableViewState {
                state: TableState::default(),
                position: None,
            },
            view_mode: vec![ViewMode::Table].into(),
            tabs,
            selected_tab_index: 0,
            selected_input: None,
            filter_input_text: InputTextElement::new("".to_string()),
            search_input_text: InputTextElement::new("".to_string()),
            view_buffer_size: DEFAULT_VIEW_BUFFER_SIZE,
            tail_enabled: false,
            copying_to_clipboard: false,
            mouse_position: (0, 0),
            last_key_input: None,
        };

        app.reload_combined_tab();

        app
    }

    pub fn last_key_input(&self) -> Option<char> {
        self.last_key_input
    }

    pub fn last_key_input_mut(&mut self) -> &mut Option<char> {
        &mut self.last_key_input
    }

    pub fn copying_to_clipboard(&mut self) -> bool {
        self.copying_to_clipboard
    }

    pub fn copying_to_clipboard_mut(&mut self) -> &mut bool {
        &mut self.copying_to_clipboard
    }

    pub fn view_buffer_size(&self) -> usize {
        self.view_buffer_size
    }

    pub fn tabs(&self) -> &Vec<Tab> {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut Vec<Tab> {
        &mut self.tabs
    }

    pub fn running(&self) -> &bool {
        &self.running
    }

    pub fn running_mut(&mut self) -> &mut bool {
        &mut self.running
    }

    pub fn selected_tab_index(&self) -> usize {
        self.selected_tab_index
    }

    pub fn selected_tab_index_mut(&mut self) -> &mut usize {
        &mut self.selected_tab_index
    }

    pub fn selected_input(&self) -> &Option<SelectedInput> {
        &self.selected_input
    }

    pub fn selected_input_mut(&mut self) -> &mut Option<SelectedInput> {
        &mut self.selected_input
    }

    pub fn view_mode(&self) -> &VecDeque<ViewMode> {
        &self.view_mode
    }

    pub fn view_mode_mut(&mut self) -> &mut VecDeque<ViewMode> {
        &mut self.view_mode
    }

    pub fn tail_enabled(&self) -> bool {
        self.tail_enabled
    }

    pub fn set_tail_enabled(&mut self, tail_enabled: bool) {
        self.tail_enabled = tail_enabled;
    }

    pub fn reload_combined_tab(&mut self) {
        let tabs = &mut self.tabs;

        let mut all_tab_items = vec![];
        for tab in tabs.iter() {
            if matches!(tab.tab_type, TabType::Combined) {
                continue;
            }

            let mut current_tab_items = tab.filtered_view_items.data.clone();
            all_tab_items.append(&mut current_tab_items);
        }

        all_tab_items.sort_by(|a, b| {
            a[LogEntryIndices::Date as usize].cmp(&b[LogEntryIndices::Date as usize])
        });

        tabs[COMBINED_TAB_INDEX].filtered_view_items.data = all_tab_items;

        tabs[COMBINED_TAB_INDEX]
            .filtered_view_items
            .selected_item_index = tabs[COMBINED_TAB_INDEX].filtered_view_items.data.len() - 1;

        let items = tabs[COMBINED_TAB_INDEX].filtered_view_items.clone();
        *tabs[COMBINED_TAB_INDEX].items_mut() = items;
    }

    pub fn get_view_buffer_range(&self) -> Range<usize> {
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
            return 0..0;
        }

        let items = &self.tabs[self.selected_tab_index].filtered_view_items;
        let num_items = self.tabs[self.selected_tab_index]
            .filtered_view_items
            .data
            .len();

        // gets the view range based on the view_buffer_size
        let chunk_multiplier = items.selected_item_index / self.view_buffer_size;

        chunk_multiplier * self.view_buffer_size
            ..std::cmp::min(num_items, (chunk_multiplier + 1) * self.view_buffer_size)
    }

    fn calculate_position_in_view_buffer(&self) -> usize {
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
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
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
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

        self.table_view_state
            .state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn previous(&mut self, search: Option<String>) {
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
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

        self.table_view_state
            .state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn skipping_next(&mut self) {
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
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

        self.table_view_state
            .state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn skipping_prev(&mut self) {
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
            return;
        }

        {
            let index = &mut self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index;
            *index = std::cmp::max(index.saturating_sub(DEFAULT_SKIP_SIZE), 0);
        }

        self.table_view_state
            .state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn start(&mut self) {
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
            return;
        }

        self.tabs[self.selected_tab_index]
            .filtered_view_items
            .selected_item_index = 0;

        self.table_view_state
            .state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn end(&mut self) {
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
            return;
        }

        self.tabs[self.selected_tab_index]
            .filtered_view_items
            .selected_item_index = self.tabs[self.selected_tab_index]
            .filtered_view_items
            .data
            .len()
            - 1;

        self.table_view_state
            .state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn switch_to_item_view(&mut self) {
        if self.tabs.is_empty()
            || self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .is_empty()
        {
            return;
        }

        if !self.filter_input_text.text().is_empty() {
            self.view_mode.push_back(ViewMode::TableItem(
                self.tabs[self.selected_tab_index]
                    .filtered_view_items
                    .selected_item_index,
            ));
            return;
        }

        if matches!(self.view_mode.back(), Some(ViewMode::Table)) {
            self.view_mode.push_back(ViewMode::TableItem(
                self.tabs[self.selected_tab_index]
                    .filtered_view_items
                    .selected_item_index,
            ));
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
                    data: parser::parse_log_by_path(&file_path).unwrap_or_default(),
                    selected_item_index: 0,
                };
                self.tabs
                    .push(Tab::new(file_path, table_items, TabType::Normal));
                self.selected_tab_index = self.tabs.len() - 1;
            }
            self.reload_combined_tab();
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
        self.table_view_state
            .state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        self.selected_tab_index = self.selected_tab_index.saturating_sub(1);
        self.table_view_state
            .state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn filter_by_current_input(&mut self, filter: String) {
        for tab in &mut self.tabs {
            tab.filtered_view_items.data = tab
                .items()
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
                .cloned()
                .collect::<Vec<Vec<String>>>();

            tab.filtered_view_items.selected_item_index = if self.tail_enabled {
                tab.filtered_view_items.data.len() - 1
            } else {
                0
            };
        }

        self.reload_combined_tab();
    }

    pub fn selected_log_entry_in_text(&self) -> String {
        let items = &self.tabs()[self.selected_tab_index()].filtered_view_items;

        let date = &items.data[items.selected_item_index][LogEntryIndices::Date as usize];
        let level = &items.data[items.selected_item_index][LogEntryIndices::Level as usize];
        let text = &items.data[items.selected_item_index][LogEntryIndices::Log as usize];
        let log_entry = format!("{:<25}{:<8}{}", date, level, text);

        log_entry
    }

    pub fn filter_input_text(&self) -> &InputTextElement {
        &self.filter_input_text
    }

    pub fn filter_input_text_mut(&mut self) -> &mut InputTextElement {
        &mut self.filter_input_text
    }

    pub fn search_input_text(&self) -> &InputTextElement {
        &self.search_input_text
    }

    pub fn search_input_text_mut(&mut self) -> &mut InputTextElement {
        &mut self.search_input_text
    }

    pub fn mouse_position_mut(&mut self) -> &mut (u16, u16) {
        &mut self.mouse_position
    }

    pub fn mouse_position(&self) -> (u16, u16) {
        self.mouse_position
    }

    pub fn table_view_state_mut(&mut self) -> &mut TableViewState {
        &mut self.table_view_state
    }

    pub fn table_view_state(&self) -> &TableViewState {
        &self.table_view_state
    }

    pub fn handle_table_mouse_click(&mut self) {
        if let Some((_, table_position_top)) = *self.table_view_state().position() {
            let mouse_position_top = self.mouse_position().1;
            let table_position_first_row = table_position_top + 2;

            if mouse_position_top < table_position_first_row {
                // mouse click is outside the table
                return;
            }

            let offset_of_first_row_in_view = self.table_view_state().state().offset() as u16;
            self.table_view_state.state.select(Some(
                (mouse_position_top - table_position_first_row + offset_of_first_row_in_view)
                    .into(),
            ));

            let view_buffer_start = self.get_view_buffer_range().start;
            let selected_tab_index = self.selected_tab_index;
            let table_state = &self.table_view_state;
            let mouse_position_top = self.mouse_position.1;

            let item_to_select = view_buffer_start
                + table_state.state().offset()
                + ((mouse_position_top - table_position_top - 2) as usize);

            if item_to_select
                == self.tabs_mut()[selected_tab_index]
                    .filtered_view_items
                    .selected_item_index
            {
                self.switch_to_item_view();
            } else if item_to_select
                < self.tabs()[selected_tab_index]
                    .filtered_view_items
                    .data
                    .len()
            {
                self.tabs_mut()[selected_tab_index]
                    .filtered_view_items
                    .selected_item_index = item_to_select;
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let serialized = json!({"tabs": self.tabs().iter()
        .filter(|tab| !tab.file_path.is_empty())
        .map(|tab|{
        tab.file_path.clone()
        }).collect::<Vec<String>>()});
        let mut config_file = std::fs::File::create(CONFIG_FILE_NAME).unwrap();

        println!("Serializing config ..");
        config_file
            .write_all(serialized.to_string().as_bytes())
            .unwrap();
        println!("DONE");
    }
}
