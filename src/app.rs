use std::ops::Range;

use ratatui::widgets::TableState;
use rfd::FileDialog;

use crate::parser;
use crate::parser::LogEntry;
use log::info;

pub enum SelectedInput {
    Filter(String),
    Search(String),
}

pub struct App {
    pub running: bool,
    pub state: TableState,
    pub view_mode: ViewMode,
    pub tabs: Vec<Tab>,
    pub tab_index: usize,
    pub selected_input: Option<SelectedInput>,
    pub view_buffer_size: usize,
}

impl App {
    pub fn new(file_path: Option<&String>) -> App {
        if let Some(file_path) = file_path {
            App {
                running: true,
                state: TableState::default(),
                view_mode: ViewMode::Table,
                tabs: vec![Tab {
                    file_path: file_path.clone(),
                    items: parser::parse_log_by_path(&file_path, 0).unwrap(),
                    selected_item: 0,
                }],
                tab_index: 0,
                selected_input: None,
                view_buffer_size: 10,
            }
        } else {
            App {
                running: true,
                // TODO: Add a help page on startup
                state: TableState::default(),
                view_mode: ViewMode::Table,
                tabs: vec![Tab {
                    file_path: "Help".to_owned(),
                    items: vec![],
                    selected_item: 0,
                }],
                tab_index: 0,
                selected_input: None,
                view_buffer_size: 10,
            }
        }
    }

    pub fn get_view_buffer_range(&self) -> Range<usize> {
        // gets the view range based on the view_buffer_size
        std::cmp::max(
            self.tabs[self.tab_index]
                .selected_item
                .saturating_sub(self.view_buffer_size / 2),
            0,
        )
            ..std::cmp::min(
                self.tabs[self.tab_index]
                    .selected_item
                    .saturating_add(self.view_buffer_size / 2)
                    + 1,
                self.tabs[self.tab_index].items.len(),
            )
    }

    fn calculate_position_on_screen(&self) -> usize {
        // The selected item is normally at the center of the current view unless
        // it's positioned before the item at location  (view_buffer_size / 2) or
        // after the item at location (lastItem -  view_buffer_size / 2)

        let pos = if self.tabs[self.tab_index].selected_item < self.view_buffer_size / 2 {
            self.tabs[self.tab_index].selected_item
        } else if self.tabs[self.tab_index].selected_item
            > self.tabs[self.tab_index]
                .items
                .len()
                .saturating_sub(self.view_buffer_size / 2)
        {
            self.tabs[self.tab_index].selected_item.saturating_sub(
                self.tabs[self.tab_index]
                    .items
                    .len()
                    .saturating_sub(self.view_buffer_size),
            )
        } else {
            self.view_buffer_size / 2
        };

        info!("Screen position calculated to be {}", pos);

        pos
    }

    pub fn next(&mut self) {
        self.tabs[self.tab_index].selected_item =
            self.tabs[self.tab_index].selected_item.saturating_add(1);

        self.state.select(Some(self.calculate_position_on_screen()));
    }

    pub fn previous(&mut self) {
        self.tabs[self.tab_index].selected_item =
            self.tabs[self.tab_index].selected_item.saturating_sub(1);
        self.state.select(Some(self.calculate_position_on_screen()));
    }

    pub fn start(&mut self) {
        self.tabs[self.tab_index].selected_item = 0;
        self.state.select(Some(self.calculate_position_on_screen()));
    }

    pub fn end(&mut self) {
        self.tabs[self.tab_index].selected_item = self.tabs[self.tab_index].items.len() - 1;
        self.state.select(Some(self.calculate_position_on_screen()));
    }

    pub fn page_down(&mut self) {
        self.tabs[self.tab_index].selected_item = self.tabs[self.tab_index]
            .selected_item
            .saturating_add(self.view_buffer_size);
        self.state.select(Some(self.calculate_position_on_screen()));
    }

    pub fn page_up(&mut self) {
        self.tabs[self.tab_index].selected_item = self.tabs[self.tab_index]
            .selected_item
            .saturating_sub(self.view_buffer_size);
        self.state.select(Some(self.calculate_position_on_screen()));
    }

    pub fn switch_to_item_view(&mut self) {
        match self.view_mode {
            ViewMode::Table => {
                self.view_mode = ViewMode::TableItem(self.tabs[self.tab_index].selected_item)
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
            selected_item: 0,
        });
        self.tab_index = self.tabs.len() - 1;
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tabs.len();
        self.state.select(Some(self.calculate_position_on_screen()));
    }
}

pub enum ViewMode {
    Table,
    TableItem(usize /* index */),
}

pub struct Tab {
    pub file_path: String,
    pub items: Vec<LogEntry>,
    pub selected_item: usize,
}
