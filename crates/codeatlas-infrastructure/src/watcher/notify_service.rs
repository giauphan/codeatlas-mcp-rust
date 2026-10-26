use notify_debouncer_full::{
    new_debouncer,
    notify::{RecursiveMode, Watcher},
    DebounceEventResult,
};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct NotifyWatcherService;

impl NotifyWatcherService {
    pub fn new() -> Self {
        Self
    }

    pub fn watch_directory<F>(&self, path: &Path, on_change: F) -> Result<(), String>
    where
        F: Fn(Vec<std::path::PathBuf>) + Send + Sync + 'static,
    {
        let (tx, rx) = channel();

        let mut debouncer = new_debouncer(Duration::from_millis(200), None, tx)
            .map_err(|e| format!("Failed to create debouncer: {:?}", e))?;

        debouncer
            .watcher()
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {:?}", e))?;

        std::thread::spawn(move || {
            // Keep debouncer alive in thread
            let _debouncer = debouncer;
            while let Ok(result) = rx.recv() {
                match result {
                    DebounceEventResult::Ok(events) => {
                        let paths: Vec<std::path::PathBuf> =
                            events.into_iter().flat_map(|e| e.paths.clone()).collect();
                        if !paths.is_empty() {
                            on_change(paths);
                        }
                    }
                    DebounceEventResult::Err(errors) => {
                        eprintln!("[Watcher] Errors encountered: {:?}", errors);
                    }
                }
            }
        });

        Ok(())
    }
}

impl Default for NotifyWatcherService {
    fn default() -> Self {
        Self::new()
    }
}
