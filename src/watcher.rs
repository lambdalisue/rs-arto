use notify_debouncer_full::{
    new_debouncer, notify::RecursiveMode, DebounceEventResult, Debouncer, RecommendedCache,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, LazyLock, Mutex,
};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("Failed to send watcher command")]
    CommandFailed,
}

type WatcherResult<T> = Result<T, WatcherError>;

/// Unique identifier for each file watcher
type WatcherId = u64;

/// Type alias for the watcher map to reduce complexity
type WatcherMap = HashMap<PathBuf, HashMap<WatcherId, Sender<()>>>;

/// RAII guard for file watching
/// Automatically unwatches the file when dropped
pub struct FileWatchGuard {
    path: PathBuf,
    watcher_id: WatcherId,
    command_tx: Sender<FileWatcherCommand>,
}

impl Drop for FileWatchGuard {
    fn drop(&mut self) {
        tracing::debug!(
            "Dropping FileWatchGuard for {:?} (watcher_id: {})",
            self.path,
            self.watcher_id
        );
        // Send unwatch command when guard is dropped
        let _ = self.command_tx.blocking_send(FileWatcherCommand::Unwatch(
            self.path.clone(),
            self.watcher_id,
        ));
    }
}

/// Global file watcher that manages file change notifications
pub struct FileWatcher {
    command_tx: Sender<FileWatcherCommand>,
    next_watcher_id: Arc<AtomicU64>,
}

enum FileWatcherCommand {
    Watch(PathBuf, WatcherId, Sender<()>),
    Unwatch(PathBuf, WatcherId),
}

impl FileWatcher {
    fn new() -> Self {
        let (command_tx, mut command_rx) = mpsc::channel::<FileWatcherCommand>(100);

        // Spawn a dedicated thread for the file watcher
        std::thread::spawn(move || {
            // Map of file paths to their notification channels
            // Inner HashMap maps WatcherId to Sender for precise unwatch operations
            let watchers: Arc<Mutex<WatcherMap>> = Arc::new(Mutex::new(HashMap::new()));
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
                        let mut watchers = watchers_clone.lock().unwrap();
                        for path in changed_paths {
                            if let Some(senders_map) = watchers.get_mut(&path) {
                                tracing::debug!("File changed: {:?}", path);

                                // Collect closed channels to remove them
                                let mut closed_ids = Vec::new();

                                for (watcher_id, sender) in senders_map.iter() {
                                    if sender.blocking_send(()).is_err() {
                                        // Channel is closed, mark for removal
                                        closed_ids.push(*watcher_id);
                                    }
                                }

                                // Remove closed channels
                                for id in closed_ids {
                                    senders_map.remove(&id);
                                    tracing::debug!("Removed closed watcher {} for {:?}", id, path);
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
                    Some(FileWatcherCommand::Watch(path, watcher_id, tx)) => {
                        let mut watchers = watchers.lock().unwrap();
                        let is_first = !watchers.contains_key(&path);

                        watchers
                            .entry(path.clone())
                            .or_default()
                            .insert(watcher_id, tx);

                        // Only start watching if this is the first watcher for this file
                        if is_first {
                            if let Err(e) = debouncer.watch(&path, RecursiveMode::NonRecursive) {
                                tracing::error!("Failed to watch file {:?}: {:?}", path, e);
                            } else {
                                tracing::info!("Started watching file: {:?}", path);
                            }
                        }
                        tracing::debug!(
                            "Registered watcher {} for {:?}",
                            watcher_id,
                            path
                        );
                    }
                    Some(FileWatcherCommand::Unwatch(path, watcher_id)) => {
                        let mut watchers = watchers.lock().unwrap();
                        if let Some(senders_map) = watchers.get_mut(&path) {
                            senders_map.remove(&watcher_id);
                            tracing::debug!(
                                "Unregistered watcher {} for {:?}",
                                watcher_id,
                                path
                            );

                            // If no more watchers for this file, stop watching
                            if senders_map.is_empty() {
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

        Self {
            command_tx,
            next_watcher_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Watch a file and receive notifications when it changes
    /// Returns a RAII guard and a receiver channel
    /// The file will be automatically unwatched when the guard is dropped
    pub async fn watch(&self, path: PathBuf) -> WatcherResult<(FileWatchGuard, Receiver<()>)> {
        // Generate unique watcher ID
        let watcher_id = self.next_watcher_id.fetch_add(1, Ordering::SeqCst);

        // Create channel for file change notifications
        let (tx, rx) = mpsc::channel(10);

        // Send watch command
        self.command_tx
            .send(FileWatcherCommand::Watch(path.clone(), watcher_id, tx))
            .await
            .map_err(|_| WatcherError::CommandFailed)?;

        // Create RAII guard
        let guard = FileWatchGuard {
            path,
            watcher_id,
            command_tx: self.command_tx.clone(),
        };

        Ok((guard, rx))
    }
}

pub static FILE_WATCHER: LazyLock<FileWatcher> = LazyLock::new(FileWatcher::new);
