use std::thread;

use std::sync::{Arc, Mutex};

use crate::app::App;
use crate::parser::parse_log_by_path;
use crate::tab::TableItems;

pub struct FileMonitor {
    _handler: thread::JoinHandle<()>,
}

impl FileMonitor {
    pub fn new(app: Arc<Mutex<App>>) -> Self {
        let handler = thread::spawn(move || loop {
            // this code attempts to minimize the duration the App mutex is locked

            if !app.lock().unwrap().tail_enabled() {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }

            let file_paths_and_sizes = app
                .lock()
                .unwrap()
                .tabs()
                .iter()
                .map(|tab| (tab.file_path.clone(), tab.last_file_size))
                .collect::<Vec<(String, usize)>>();

            let mut file_path_to_log_entries = std::collections::HashMap::new();

            for (file_path, last_file_size) in file_paths_and_sizes {
                let file_meta = std::fs::metadata(&file_path);
                if file_meta.is_err() {
                    continue;
                }
                let current_file_size = file_meta.unwrap().len();

                if current_file_size == last_file_size.try_into().unwrap() {
                    continue;
                }

                if let Ok(log_entries) = parse_log_by_path(&file_path) {
                    file_path_to_log_entries
                        .insert(file_path.clone(), (log_entries, current_file_size));
                }
            }

            let mut any_tabs_updated = false;
            for tab in app.lock().unwrap().tabs_mut() {
                if let crate::tab::TabType::Combined = tab.tab_type {
                    continue;
                }

                if !file_path_to_log_entries.contains_key(&tab.file_path) {
                    continue;
                }

                let (data, file_size) = file_path_to_log_entries
                    .remove(&tab.file_path)
                    .take()
                    .unwrap();

                *tab.items_mut() = TableItems {
                    selected_item_index: data.len() - 1,
                    data,
                };

                tab.last_file_size = file_size as usize;

                any_tabs_updated = true;
            }

            if any_tabs_updated {
                let filter_text = app.lock().unwrap().filter_input_text().clone();
                let mut app = app.lock().unwrap();
                app.filter_by_current_input(filter_text);
                app.reload_combined_tab();
            }
        });

        FileMonitor { _handler: handler }
    }
}
