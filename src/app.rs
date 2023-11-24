use std::collections::VecDeque;
use std::ops::Range;

use ratatui::widgets::TableState;
use rfd::FileDialog;

use crate::parser;
use crate::parser::{LogEntry, LogEntryIndices};
use log::info;

const DEFAULT_VIEW_BUFFER_SIZE: usize = 150;
const DEFAULT_SKIP_SIZE: usize = 5;

pub enum SelectedInput {
    Filter(String),
    Search(String),
}

pub struct App {
    pub running: bool,
    pub state: TableState,

    // we keep a history of view modes to be able to switch back
    pub tabs: Vec<Tab>,
    pub selected_tab_index: usize,
    pub view_mode: VecDeque<ViewMode>, // TODO; Merge selected_input & view_mode together
    pub selected_input: Option<SelectedInput>,
    pub view_buffer_size: usize,
}

impl App {
    pub fn new(file_path: Option<&String>) -> App {
        let mut view_mode = VecDeque::new();
        view_mode.push_back(ViewMode::Table);

        if let Some(file_path) = file_path {
            App {
                running: true,
                state: TableState::default(),
                view_mode: view_mode,
                tabs: vec![Tab {
                    file_path: file_path.clone(),
                    items: parser::parse_log_by_path(&file_path, 0).unwrap(),
                    selected_item_index: 0,
                    filtered_view_items: vec![],
                    selected_filtered_view_item_index: 0,
                }],
                selected_tab_index: 0,
                selected_input: None,
                view_buffer_size: DEFAULT_VIEW_BUFFER_SIZE,
            }
        } else {
            App {
                running: true,
                // TODO: Add a help page on startup
                state: TableState::default(),
                view_mode: view_mode,
                tabs: vec![Tab {
                    file_path: "Help".to_owned(),
                    items: vec![],
                    selected_item_index: 0,
                    filtered_view_items: vec![],
                    selected_filtered_view_item_index: 0,
                }],
                selected_tab_index: 0,
                selected_input: None,
                view_buffer_size: DEFAULT_VIEW_BUFFER_SIZE,
            }
        }
    }

    pub fn get_view_buffer_range(&self) -> Range<usize> {
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index
        let mut index = self.tabs[self.selected_tab_index].selected_item_index;
        let mut num_items = self.tabs[self.selected_tab_index].items.len();

        if let Some(ViewMode::FilteredView) = self.view_mode.back() {
            index = self.tabs[self.selected_tab_index].selected_filtered_view_item_index;
            num_items = self.tabs[self.selected_tab_index].filtered_view_items.len();
        }

        // gets the view range based on the view_buffer_size
        std::cmp::max(
            index
                // It's better to have less items before the selected index than after it
                // to avoid the selected item showing up at the bottom of the screen
                .saturating_sub(self.view_buffer_size / 4),
            0,
        )
            ..std::cmp::min(
                index.saturating_add(self.view_buffer_size / 2) + 1,
                num_items,
            )
    }

    fn calculate_position_in_view_buffer(&self) -> usize {
        // Location on screen is relative to the start of the view buffer
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        let pos = if let Some(ViewMode::FilteredView) = self.view_mode.back() {
            self.tabs[self.selected_tab_index].selected_filtered_view_item_index
                - self.get_view_buffer_range().start
        } else {
            self.tabs[self.selected_tab_index].selected_item_index
                - self.get_view_buffer_range().start
        };

        info!("Screen position calculated to be {}", pos);

        pos
    }

    pub fn next(&mut self, search: Option<String>) {
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index
        if let Some(ViewMode::FilteredView) = self.view_mode.back() {
            let num_items = self.tabs[self.selected_tab_index].filtered_view_items.len() - 1;
            let index = &mut self.tabs[self.selected_tab_index].selected_filtered_view_item_index;
            *index = std::cmp::min(index.saturating_add(1), num_items);
        } else {
            let new_index = if search.is_none() || search.as_ref().unwrap().is_empty() {
                // normal mode
                let index = self.tabs[self.selected_tab_index].selected_item_index;
                std::cmp::min(
                    index.saturating_add(1),
                    self.tabs[self.selected_tab_index].items.len() - 1,
                )
            } else {
                // search mode
                let mut index = self.tabs[self.selected_tab_index].selected_item_index;
                let mut final_index = index;
                loop {
                    index = std::cmp::min(
                        index.saturating_add(1),
                        self.tabs[self.selected_tab_index].items.len() - 1,
                    );

                    if self.tabs[self.selected_tab_index].items[index]
                        [LogEntryIndices::LOG as usize]
                        .to_lowercase()
                        .contains(&search.as_ref().unwrap().to_lowercase())
                    {
                        final_index = index;
                        break;
                    }

                    if index == (self.tabs[self.selected_tab_index].items.len() - 1) {
                        // reached the end
                        break;
                    }
                }

                final_index
            };

            self.tabs[self.selected_tab_index].selected_item_index = new_index;
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn previous(&mut self, search: Option<String>) {
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index
        if let Some(ViewMode::FilteredView) = self.view_mode.back() {
            let index = &mut self.tabs[self.selected_tab_index].selected_filtered_view_item_index;
            *index = std::cmp::max(index.saturating_sub(1), 0);
        } else {
            let new_index = if search.is_none() || search.as_ref().unwrap().is_empty() {
                // normal mode
                let index = self.tabs[self.selected_tab_index].selected_item_index;
                std::cmp::max(index.saturating_sub(1), 0)
            } else {
                // search mode
                let mut index = self.tabs[self.selected_tab_index].selected_item_index;
                let mut final_index = index;
                loop {
                    index = std::cmp::max(index.saturating_sub(1), 0);

                    if self.tabs[self.selected_tab_index].items[index]
                        [LogEntryIndices::LOG as usize]
                        .to_lowercase()
                        .contains(&search.as_ref().unwrap().to_lowercase())
                    {
                        final_index = index;
                        break;
                    }

                    if index == 0 {
                        // reached the beginning
                        break;
                    }
                }

                final_index
            };

            self.tabs[self.selected_tab_index].selected_item_index = new_index;
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn skipping_next(&mut self) {
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        if let Some(ViewMode::FilteredView) = self.view_mode.back() {
            let num_items = self.tabs[self.selected_tab_index].filtered_view_items.len();
            let index = &mut self.tabs[self.selected_tab_index].selected_filtered_view_item_index;
            *index = std::cmp::min(index.saturating_add(DEFAULT_SKIP_SIZE), num_items - 1);
        } else {
            let num_items = self.tabs[self.selected_tab_index].items.len();
            let index = &mut self.tabs[self.selected_tab_index].selected_item_index;
            *index = std::cmp::min(index.saturating_add(DEFAULT_SKIP_SIZE), num_items - 1);
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn skipping_prev(&mut self) {
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        if let Some(ViewMode::FilteredView) = self.view_mode.back() {
            let index = &mut self.tabs[self.selected_tab_index].selected_filtered_view_item_index;
            *index = std::cmp::max(index.saturating_sub(DEFAULT_SKIP_SIZE), 0);
        } else {
            let index = &mut self.tabs[self.selected_tab_index].selected_item_index;
            *index = std::cmp::max(index.saturating_sub(DEFAULT_SKIP_SIZE), 0);
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn start(&mut self) {
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        if let Some(ViewMode::FilteredView) = self.view_mode.back() {
            self.tabs[self.selected_tab_index].selected_filtered_view_item_index = 0
        } else {
            self.tabs[self.selected_tab_index].selected_item_index = 0
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn end(&mut self) {
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        if let Some(ViewMode::FilteredView) = self.view_mode.back() {
            self.tabs[self.selected_tab_index].selected_filtered_view_item_index =
                self.tabs[self.selected_tab_index].filtered_view_items.len() - 1;
        } else {
            self.tabs[self.selected_tab_index].selected_item_index =
                self.tabs[self.selected_tab_index].items.len() - 1;
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn switch_to_item_view(&mut self) {
        match self.view_mode.back() {
            Some(ViewMode::Table) => {
                self.view_mode.push_back(ViewMode::TableItem(
                    self.tabs[self.selected_tab_index].selected_item_index,
                ));
            }
            Some(ViewMode::FilteredView) => {
                self.view_mode.push_back(ViewMode::TableItem(
                    self.tabs[self.selected_tab_index].selected_filtered_view_item_index,
                ));
            }
            _ => {}
        }
    }

    pub fn load_files(&mut self) {
        let file = FileDialog::new()
            .add_filter("text", &["txt", "log", "bak"])
            .pick_file()
            .unwrap();
        let file_path = file.to_str().unwrap().to_string();
        self.tabs.push(Tab {
            items: parser::parse_log_by_path(&file_path, 0).unwrap(),
            file_path: file_path,
            selected_item_index: 0,
            filtered_view_items: vec![],
            selected_filtered_view_item_index: 0,
        });
        self.selected_tab_index = self.tabs.len() - 1;
    }

    pub fn next_tab(&mut self) {
        self.selected_tab_index = self.selected_tab_index.saturating_add(1) % self.tabs.len();
        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn prev_tab(&mut self) {
        self.selected_tab_index = self.selected_tab_index.saturating_sub(1);
        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }
}

pub enum ViewMode {
    Table,
    FilteredView,
    SearchView,
    TableItem(usize /* index */),
}

pub struct Tab {
    pub file_path: String,
    pub items: Vec<LogEntry>,
    pub selected_item_index: usize,
    pub filtered_view_items: Vec<LogEntry>,
    pub selected_filtered_view_item_index: usize,
}
