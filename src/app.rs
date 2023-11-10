use ratatui::widgets::TableState;
use rfd::FileDialog;

use crate::parser;
use crate::parser::log_parser::LogEntry;
// use crate::parser::log_parser::{parse_log_by_path, LogEntry};

pub struct App {
    pub should_quit: bool,
    pub state: TableState,
    pub view_mode: ViewMode,
    pub tabs: Vec<Tab>,
    pub tab_index: usize,
}

impl App {
    pub fn new(file_path: Option<&String>) -> App {
        if let Some(file_path) = file_path {
            App {
                should_quit: false,
                state: TableState::default(),
                view_mode: ViewMode::Table,
                tabs: vec![Tab {
                    file_path: file_path.clone(),
                    items: parser::log_parser::parse_log_by_path(&file_path, 0).unwrap(),
                    selected_item: 0,
                }],
                tab_index: 0,
            }
        } else {
            App {
                should_quit: false,
                // TODO: Add a help page on startup
                state: TableState::default(),
                view_mode: ViewMode::Table,
                tabs: vec![Tab {
                    file_path: "Help".to_owned(),
                    items: vec![],
                    selected_item: 0,
                }],
                tab_index: 0,
            }
        }
    }
    pub fn next(&mut self) {
        self.tabs[self.tab_index].selected_item = match self.state.selected() {
            Some(i) => {
                if i >= self.tabs[self.tab_index].items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn previous(&mut self) {
        self.tabs[self.tab_index].selected_item = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tabs[self.tab_index].items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn start(&mut self) {
        self.tabs[self.tab_index].selected_item = 0;
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn end(&mut self) {
        self.tabs[self.tab_index].selected_item = self.tabs[self.tab_index].items.len() - 1;
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn page_down(&mut self) {
        self.tabs[self.tab_index].selected_item = match self.state.selected() {
            Some(i) => {
                if i >= self.tabs[self.tab_index].items.len() - 21 {
                    self.tabs[self.tab_index].items.len() - 1
                } else {
                    i + 20
                }
            }
            None => 0,
        };
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn page_up(&mut self) {
        self.tabs[self.tab_index].selected_item = match self.state.selected() {
            Some(i) => {
                if i <= 20 {
                    0
                } else {
                    i - 20
                }
            }
            None => 0,
        };
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
    }

    pub fn switch_to_item_view(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i,
            None => 0,
        };

        match self.view_mode {
            ViewMode::Table => self.view_mode = ViewMode::TableItem(i),
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
            items: parser::log_parser::parse_log_by_path(&file_path, 0).unwrap(),
            file_path: file_path,
            selected_item: 0,
        });
        self.tab_index = self.tabs.len() - 1;
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tabs.len();
        self.state
            .select(Some(self.tabs[self.tab_index].selected_item));
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
