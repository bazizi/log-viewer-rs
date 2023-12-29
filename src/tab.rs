use crate::parser::LogEntry;

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
                name: " [Combined] ".to_owned(),
                items: table_items.clone(),
                filtered_view_items: table_items,
                last_file_size: 0,
                file_path: "".to_owned(),
                tab_type,
            };
        }

        Tab {
            name: format!(
                " [{}] ",
                std::path::Path::new(file_path.clone().as_str())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            ),
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

    pub fn items_mut(&mut self) -> &mut TableItems {
        &mut self.items
    }

    pub fn items(&self) -> &TableItems {
        &self.items
    }
}
