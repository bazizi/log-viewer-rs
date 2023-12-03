use crate::parser::parse_log_by_path;
use crate::parser::LogEntry;

use anyhow::Result;

#[derive(Clone)]
pub struct TableItems {
    pub data: Vec<LogEntry>,
    pub selected_item_index: usize,
}

pub struct Tab {
    pub name: String,
    pub file_path: String,
    items: TableItems,
    pub filtered_view_items: TableItems,
    pub last_file_size: usize,
}

impl Tab {
    pub fn new(file_path: String, table_items: TableItems) -> Self {
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
        }
    }

    pub fn reset_filtered_view_items(&mut self) {
        self.filtered_view_items = self.items.clone();
    }

    pub fn reload(&mut self) -> Result<()> {
        let log_lines = parse_log_by_path(&self.file_path)?;
        self.items.selected_item_index = 0;
        self.items.data = log_lines;
        Ok(())
    }
}
