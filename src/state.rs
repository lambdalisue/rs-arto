use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;

use crate::history::HistoryManager;
use crate::theme::ThemePreference;

/// A global receiver to receive opened files from the main thread
pub static OPENED_FILES_RECEIVER: Mutex<Option<Receiver<PathBuf>>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub file: Signal<Option<PathBuf>>,
    pub current_theme: Signal<ThemePreference>,
    pub history: Signal<HistoryManager>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            file: Signal::new(None),
            current_theme: Signal::new(ThemePreference::default()),
            history: Signal::new(HistoryManager::new()),
        }
    }
}
