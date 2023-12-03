use std::collections::VecDeque;
use std::ops::Range;

use ratatui::widgets::TableState;
use rfd::FileDialog;

use anyhow::Result;

use crate::parser;
use crate::parser::LogEntryIndices;
use log::info;

use crate::tab::Tab;
use crate::tab::TableItems;

const DEFAULT_VIEW_BUFFER_SIZE: usize = 50;
const DEFAULT_SKIP_SIZE: usize = 5;

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
    pub selected_input: Option<SelectedInput>,
    pub filter_input_text: String,
    pub search_input_text: String,
    pub view_buffer_size: usize,
    pub tail_enabled: bool, // TODO: add tailing support
}

impl App {
    pub fn new(file_paths: Vec<String>) -> App {
        App {
            running: true,
            state: TableState::default(),
            view_mode: vec![ViewMode::Table].into(),
            tabs: file_paths
                .iter()
                .map(|file_path| {
                    let table_items = TableItems {
                        data: parser::parse_log_by_path(&file_path).unwrap(),
                        selected_item_index: 0,
                    };
                    Tab {
                        name: std::path::Path::new(file_path.clone().as_str())
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                        items: table_items.clone(),
                        filtered_view_items: table_items,
                        last_file_size: if let Ok(meta) = std::fs::metadata(file_path) {
                            meta.len().try_into().unwrap_or(0)
                        } else {
                            0
                        },
                        file_path: file_path.to_string(),
                    }
                })
                .collect::<Vec<Tab>>(),
            selected_tab_index: 0,
            selected_input: None,
            filter_input_text: "".to_string(),
            search_input_text: "".to_string(),
            view_buffer_size: DEFAULT_VIEW_BUFFER_SIZE,
            tail_enabled: false,
        }
    }

    pub fn get_view_buffer_range(&self) -> Range<usize> {
        if self.tabs.is_empty() {
            return 0..0;
        }

        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index
        let mut num_items = self.tabs[self.selected_tab_index].items.data.len();
        let mut items = &self.tabs[self.selected_tab_index].items;

        if !self.filter_input_text.is_empty() {
            items = &self.tabs[self.selected_tab_index].filtered_view_items;
            num_items = self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .len();
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

        // Location on screen is relative to the start of the view buffer
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        let pos = if !self.filter_input_text.is_empty() {
            self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index
                - self.get_view_buffer_range().start
        } else {
            self.tabs[self.selected_tab_index].items.selected_item_index
                - self.get_view_buffer_range().start
        };

        info!("Screen position calculated to be {}", pos);

        pos
    }

    pub fn next(&mut self, search: Option<String>) {
        if self.tabs.is_empty() {
            return;
        }

        let (mut index, items) = if !self.filter_input_text.is_empty() {
            (
                self.tabs[self.selected_tab_index]
                    .filtered_view_items
                    .selected_item_index,
                &mut self.tabs[self.selected_tab_index].filtered_view_items,
            )
        } else {
            (
                self.tabs[self.selected_tab_index].items.selected_item_index,
                &mut self.tabs[self.selected_tab_index].items,
            )
        };

        let new_index = if search.is_none() || search.as_ref().unwrap().is_empty() {
            std::cmp::min(index.saturating_add(1), items.data.len() - 1)
        } else {
            // search mode
            let mut final_index = index;
            loop {
                index = std::cmp::min(index.saturating_add(1), items.data.len() - 1);

                if items.data[index][LogEntryIndices::LOG as usize]
                    .to_lowercase()
                    .contains(&search.as_ref().unwrap().to_lowercase())
                {
                    final_index = index;
                    break;
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

        let (mut index, items) = if !self.filter_input_text.is_empty() {
            (
                self.tabs[self.selected_tab_index]
                    .filtered_view_items
                    .selected_item_index,
                &mut self.tabs[self.selected_tab_index].filtered_view_items,
            )
        } else {
            (
                self.tabs[self.selected_tab_index].items.selected_item_index,
                &mut self.tabs[self.selected_tab_index].items,
            )
        };

        let new_index = if search.is_none() || search.as_ref().unwrap().is_empty() {
            std::cmp::max(index.saturating_sub(1), 0)
        } else {
            // search mode
            let mut final_index = index;
            loop {
                index = std::cmp::max(index.saturating_sub(1), 0);

                if items.data[index][LogEntryIndices::LOG as usize]
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

        items.selected_item_index = new_index;

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn skipping_next(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        if !self.filter_input_text.is_empty() {
            let num_items = self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .len();
            let index = &mut self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index;
            *index = std::cmp::min(index.saturating_add(DEFAULT_SKIP_SIZE), num_items - 1);
        } else {
            let num_items = self.tabs[self.selected_tab_index].items.data.len();
            let index = &mut self.tabs[self.selected_tab_index].items.selected_item_index;
            *index = std::cmp::min(index.saturating_add(DEFAULT_SKIP_SIZE), num_items - 1);
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn skipping_prev(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        if !self.filter_input_text.is_empty() {
            let index = &mut self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index;
            *index = std::cmp::max(index.saturating_sub(DEFAULT_SKIP_SIZE), 0);
        } else {
            let index = &mut self.tabs[self.selected_tab_index].items.selected_item_index;
            *index = std::cmp::max(index.saturating_sub(DEFAULT_SKIP_SIZE), 0);
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn start(&mut self) {
        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        if !self.filter_input_text.is_empty() {
            self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index = 0
        } else {
            self.tabs[self.selected_tab_index].items.selected_item_index = 0
        }

        self.state
            .select(Some(self.calculate_position_in_view_buffer()));
    }

    pub fn end(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        // If we're in filtered view, we should use the filtered view index
        // If not, we use the normal tab index

        if !self.filter_input_text.is_empty() {
            self.tabs[self.selected_tab_index]
                .filtered_view_items
                .selected_item_index = self.tabs[self.selected_tab_index]
                .filtered_view_items
                .data
                .len()
                - 1;
        } else {
            self.tabs[self.selected_tab_index].items.selected_item_index =
                self.tabs[self.selected_tab_index].items.data.len() - 1;
        }

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
                    self.tabs[self.selected_tab_index].items.selected_item_index,
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
                self.tabs.push(Tab {
                    items: table_items.clone(),
                    name: std::path::Path::new(file_path.clone().as_str())
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                    filtered_view_items: table_items,
                    file_path: file_path.to_string(),
                    last_file_size: if let Ok(meta) = std::fs::metadata(file_path) {
                        meta.len().try_into().unwrap_or(0)
                    } else {
                        0
                    },
                });
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
        if !self.tail_enabled {
            return Ok(());
        }

        for tab in &mut self.tabs {
            let metadata = std::fs::metadata(&tab.file_path)?;
            let current_file_size = metadata.len().try_into().unwrap_or(0);
            if tab.last_file_size != current_file_size {
                tab.reload()?;
                tab.last_file_size = current_file_size;
            }
        }

        Ok(())
    }
}
