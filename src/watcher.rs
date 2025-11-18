use notify_debouncer_full::{
    new_debouncer, notify::RecursiveMode, DebounceEventResult, Debouncer, RecommendedCache,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("Failed to send watcher command")]
    CommandFailed,
}

type WatcherResult<T> = Result<T, WatcherError>;

/// Global file watcher that manages file change notifications
pub struct FileWatcher {
    command_tx: Sender<FileWatcherCommand>,
}

enum FileWatcherCommand {
    Watch(PathBuf, Sender<()>),
    Unwatch(PathBuf),
}

impl FileWatcher {
    fn new() -> Self {
        let (command_tx, mut command_rx) = mpsc::channel::<FileWatcherCommand>(100);

        // Spawn a dedicated thread for the file watcher
        std::thread::spawn(move || {
            // Map of file paths to their notification channels
            let watchers: Arc<Mutex<HashMap<PathBuf, Vec<Sender<()>>>>> =
                Arc::new(Mutex::new(HashMap::new()));
            let watchers_clone = watchers.clone();

            // Create a debouncer with 500ms delay
            let mut debouncer: Debouncer<
                notify_debouncer_full::notify::RecommendedWatcher,
                RecommendedCache,
            > = match new_debouncer(
                Duration::from_millis(500),
                None,
                move |result: DebounceEventResult| match result {
                    Ok(events) => {
                        // Collect unique paths that changed
                        let mut changed_paths = std::collections::HashSet::new();
                        for event in events {
                            for path in &event.paths {
                                changed_paths.insert(path.clone());
                            }
                        }

                        // Notify all watchers for changed files
                        let watchers = watchers_clone.lock().unwrap();
                        for path in changed_paths {
                            if let Some(senders) = watchers.get(&path) {
                                tracing::debug!("File changed: {:?}", path);
                                for sender in senders {
                                    let _ = sender.blocking_send(());
                                }
                            }
                        }
                    }
                    Err(errors) => {
                        for error in errors {
                            tracing::error!("File watcher error: {:?}", error);
                        }
                    }
                },
            ) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!("Failed to create file watcher: {:?}", e);
                    return;
                }
            };

            tracing::info!("Global file watcher started");

            // Process commands
            loop {
                match command_rx.blocking_recv() {
                    Some(FileWatcherCommand::Watch(path, tx)) => {
                        let mut watchers = watchers.lock().unwrap();
                        let is_first = !watchers.contains_key(&path);

                        watchers.entry(path.clone()).or_default().push(tx);

                        // Only start watching if this is the first watcher for this file
                        if is_first {
                            if let Err(e) = debouncer.watch(&path, RecursiveMode::NonRecursive) {
                                tracing::error!("Failed to watch file {:?}: {:?}", path, e);
                            } else {
                                tracing::info!("Started watching file: {:?}", path);
                            }
                        }
                    }
                    Some(FileWatcherCommand::Unwatch(path)) => {
                        let mut watchers = watchers.lock().unwrap();
                        if let Some(senders) = watchers.get_mut(&path) {
                            senders.pop();
                            // If no more watchers for this file, stop watching
                            if senders.is_empty() {
                                watchers.remove(&path);
                                if let Err(e) = debouncer.unwatch(&path) {
                                    tracing::error!("Failed to unwatch file {:?}: {:?}", path, e);
                                } else {
                                    tracing::info!("Stopped watching file: {:?}", path);
                                }
                            }
                        }
                    }
                    None => {
                        tracing::info!("File watcher command channel closed");
                        break;
                    }
                }
            }
        });

        Self { command_tx }
    }

    /// Watch a file and receive notifications when it changes
    pub async fn watch(&self, path: PathBuf, tx: Sender<()>) -> WatcherResult<()> {
        self.command_tx
            .send(FileWatcherCommand::Watch(path, tx))
            .await
            .map_err(|_| WatcherError::CommandFailed)
    }

    /// Stop watching a file
    pub async fn unwatch(&self, path: PathBuf) -> WatcherResult<()> {
        self.command_tx
            .send(FileWatcherCommand::Unwatch(path))
            .await
            .map_err(|_| WatcherError::CommandFailed)
    }
}

pub static FILE_WATCHER: LazyLock<FileWatcher> = LazyLock::new(FileWatcher::new);
