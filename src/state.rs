use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;

use crate::history::HistoryManager;
use crate::theme::ThemePreference;

/// A global receiver to receive opened files from the main thread
pub static OPENED_FILES_RECEIVER: Mutex<Option<Receiver<PathBuf>>> = Mutex::new(None);

/// Global broadcast sender for opening files in tabs
pub static FILE_OPEN_BROADCAST: LazyLock<broadcast::Sender<PathBuf>> =
    LazyLock::new(|| broadcast::channel(100).0);

pub static LAST_SELECTED_THEME: LazyLock<Mutex<ThemePreference>> =
    LazyLock::new(|| Mutex::new(ThemePreference::default()));

/// Represents a single tab with its file and navigation history
#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub file: Option<PathBuf>,
    pub history: HistoryManager,
}

impl Tab {
    pub fn new(file: Option<PathBuf>) -> Self {
        let mut history = HistoryManager::new();
        if let Some(ref path) = file {
            history.push(path.clone());
        }
        Self { file, history }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub tabs: Signal<Vec<Tab>>,
    pub active_tab: Signal<usize>,
    pub current_theme: Signal<ThemePreference>,
    pub zoom_level: Signal<f64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tabs: Signal::new(vec![Tab::new(None)]),
            active_tab: Signal::new(0),
            current_theme: Signal::new(*LAST_SELECTED_THEME.lock().unwrap()),
            zoom_level: Signal::new(1.0),
        }
    }
}

impl AppState {
    /// Get a read-only copy of the current active tab
    pub fn current_tab(&self) -> Option<Tab> {
        let tabs = self.tabs.read();
        let active_index = *self.active_tab.read();
        tabs.get(active_index).cloned()
    }

    /// Update the current active tab using a closure
    pub fn update_current_tab<F>(&mut self, update_fn: F)
    where
        F: FnOnce(&mut Tab),
    {
        let active_index = *self.active_tab.read();
        let mut tabs = self.tabs.write();

        if let Some(tab) = tabs.get_mut(active_index) {
            update_fn(tab);
        }
    }

    /// Add a new tab and optionally switch to it
    pub fn add_tab(&mut self, file: Option<PathBuf>, switch_to: bool) {
        let mut tabs = self.tabs.write();
        tabs.push(Tab::new(file));

        if switch_to {
            let new_tab_index = tabs.len() - 1;
            self.active_tab.set(new_tab_index);
        }
    }

    /// Close a tab by index, handling edge cases
    pub fn close_tab(&mut self, index: usize) {
        let mut tabs = self.tabs.write();

        // Keep at least one tab (clear it instead of removing)
        if tabs.len() <= 1 {
            if let Some(tab) = tabs.first_mut() {
                *tab = Tab::new(None);
            }
            return;
        }

        tabs.remove(index);

        // Update active tab index if necessary
        let current_active = *self.active_tab.read();
        let new_active = match current_active.cmp(&index) {
            std::cmp::Ordering::Greater => current_active - 1,
            std::cmp::Ordering::Equal if current_active >= tabs.len() => tabs.len() - 1,
            _ => current_active,
        };

        if new_active != current_active {
            self.active_tab.set(new_active);
        }
    }

    /// Switch to a specific tab by index
    pub fn switch_to_tab(&mut self, index: usize) {
        let tabs = self.tabs.read();
        if index < tabs.len() {
            self.active_tab.set(index);
        }
    }

    /// Check if the current active tab has no file (NoFile tab)
    pub fn is_current_tab_no_file(&self) -> bool {
        self.current_tab()
            .map(|tab| tab.file.is_none())
            .unwrap_or(false)
    }

    /// Find the index of a tab that has the specified file open
    pub fn find_tab_with_file(&self, file: &PathBuf) -> Option<usize> {
        let tabs = self.tabs.read();
        tabs.iter()
            .position(|tab| tab.file.as_ref().map(|f| f == file).unwrap_or(false))
    }

    /// Open a file, reusing NoFile tab or existing tab with the same file if possible
    pub fn open_file(&mut self, file: PathBuf) {
        // Check if the file is already open in another tab
        if let Some(tab_index) = self.find_tab_with_file(&file) {
            // Switch to the existing tab instead of creating a new one
            self.switch_to_tab(tab_index);
        } else if self.is_current_tab_no_file() {
            // If current tab is NoFile, open the file in it
            self.update_current_tab(|tab| {
                tab.history.push(file.clone());
                tab.file = Some(file);
            });
        } else {
            // Otherwise, create a new tab
            self.add_tab(Some(file), true);
        }
    }
}
