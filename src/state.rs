use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;

/// A global receiver to receive opened files from the main thread
pub static OPENED_FILES_RECEIVER: Mutex<Option<Receiver<PathBuf>>> = Mutex::new(None);
