use crate::parser::parse_log_by_path;
use crate::parser::LogEntry;

use anyhow::Result;

#[derive(Clone)]
pub struct TableItems {
    pub data: Vec<LogEntry>,
    pub selected_item_index: usize,
}

impl TableItems {
    pub fn clear(&mut self) {
        self.data.clear();
        self.selected_item_index = 0;
    }
}

pub struct Tab {
    pub name: String,
    pub file_path: String,
    pub items: TableItems,
    pub filtered_view_items: TableItems,
    pub last_file_size: usize,
}

impl Tab {
    pub fn reload(&mut self) -> Result<()> {
        let log_lines = parse_log_by_path(&self.file_path)?;
        self.items.selected_item_index = log_lines.len() - 1;
        self.items.data = log_lines;
        self.filtered_view_items.clear();

        Ok(())
    }
}
