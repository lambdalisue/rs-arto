use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tokio::sync::mpsc::Receiver;

use crate::history::HistoryManager;
use crate::theme::ThemePreference;

/// A global receiver to receive opened files from the main thread
pub static OPENED_FILES_RECEIVER: Mutex<Option<Receiver<PathBuf>>> = Mutex::new(None);

pub static LAST_SELECTED_THEME: LazyLock<Mutex<ThemePreference>> =
    LazyLock::new(|| Mutex::new(ThemePreference::default()));

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub file: Signal<Option<PathBuf>>,
    pub current_theme: Signal<ThemePreference>,
    pub history: Signal<HistoryManager>,
    pub zoom_level: Signal<f64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            file: Signal::new(None),
            current_theme: Signal::new(*LAST_SELECTED_THEME.lock().unwrap()),
            history: Signal::new(HistoryManager::new()),
            zoom_level: Signal::new(1.0),
        }
    }
}
