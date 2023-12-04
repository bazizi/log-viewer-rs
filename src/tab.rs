use crate::parser::parse_log_by_path;
use crate::parser::LogEntry;

use anyhow::Result;

#[derive(Clone)]
pub struct TableItems {
    pub data: Vec<LogEntry>,
    pub selected_item_index: usize,
}

pub enum TabType {
    Normal,
    Combined, // the tab which combines data from all other tabs
}

pub struct Tab {
    pub name: String,
    pub file_path: String,
    items: TableItems,
    pub filtered_view_items: TableItems,
    pub last_file_size: usize,
    pub tab_type: TabType,
}

impl Tab {
    pub fn new(file_path: String, table_items: TableItems, tab_type: TabType) -> Self {
        if let TabType::Combined = tab_type {
            return Tab {
                name: "Combined".to_owned(), // TODO: Use Option<T>
                items: table_items.clone(),
                filtered_view_items: table_items,
                last_file_size: 0,        // TODO: Use Option<T>
                file_path: "".to_owned(), // TODO: Use Option<T>
                tab_type,
            };
        }

        Tab {
            name: std::path::Path::new(file_path.clone().as_str())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            items: table_items.clone(),
            filtered_view_items: table_items,
            last_file_size: if let Ok(meta) = std::fs::metadata(file_path.clone()) {
                meta.len().try_into().unwrap_or(0)
            } else {
                0
            },
            file_path: file_path.to_string(),
            tab_type,
        }
    }

    pub fn reset_filtered_view_items(&mut self) {
        self.filtered_view_items = self.items.clone();
    }

    pub fn set_items(&mut self, items: TableItems) {
        self.items = items;
    }

    pub fn reload(&mut self) -> Result<()> {
        if let TabType::Combined = self.tab_type {
            return Ok(());
        }

        let log_lines = parse_log_by_path(&self.file_path)?;
        self.items.selected_item_index = 0;
        self.items.data = log_lines;
        Ok(())
    }
}
