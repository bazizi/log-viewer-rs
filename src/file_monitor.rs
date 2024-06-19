use std::thread;

use std::sync::{mpsc, Arc, Mutex};

use crate::app::App;
use crate::event::Event;
use crate::parser::parse_log_by_path;
use crate::tab::{TabType, TableItems};

pub struct FileMonitor {
    handler: thread::JoinHandle<()>,
    running: Arc<Mutex<bool>>,
}

impl FileMonitor {
    pub fn new(app: Arc<Mutex<App>>, sender: mpsc::Sender<Event>) -> Self {
        let running = Arc::new(Mutex::new(true));
        let running2 = running.clone();
        let handler = thread::spawn(move || loop {
            if !*running2.lock().unwrap() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_secs(1));

            // this code attempts to minimize the duration the App mutex is locked
            if !app.lock().unwrap().tail_enabled() {
                continue;
            }

            let file_paths_and_sizes = {
                let app = app.lock().unwrap();
                let current_tab = &app.tabs()[app.selected_tab_index()];
                let file_paths_and_sizes = if let TabType::Combined = current_tab.tab_type {
                    // we're in the combined tab so return all tabs info
                    app.tabs()
                        .iter()
                        .map(|tab| (tab.file_path.clone(), tab.last_file_size))
                        .collect::<Vec<(String, usize)>>()
                } else {
                    // only return the current tab info
                    vec![(current_tab.file_path.clone(), current_tab.last_file_size)]
                };

                file_paths_and_sizes
            };

            let mut file_path_to_log_entries = std::collections::HashMap::new();

            for (file_path, last_file_size) in file_paths_and_sizes {
                let file_meta = std::fs::metadata(&file_path);
                if file_meta.is_err() {
                    continue;
                }
                let current_file_size = file_meta.unwrap().len();

                if current_file_size == TryInto::<u64>::try_into(last_file_size).unwrap() {
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
                let filter_text = app.lock().unwrap().filter_input_text().to_string().clone();
                let mut app = app.lock().unwrap();
                app.filter_by_current_input(filter_text);
                sender.send(Event::Tick).unwrap();
            }
        });

        FileMonitor { handler, running }
    }

    pub fn join(self) {
        *self.running.lock().unwrap() = false;
        self.handler.join().unwrap();
    }
}
