use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;

use crate::history::HistoryManager;
use crate::theme::ThemePreference;

/// Open event types for distinguishing files, directories, and reopen events
#[derive(Debug, Clone)]
pub enum OpenEvent {
    /// File opened from Finder/CLI
    File(PathBuf),
    /// Directory opened from Finder/CLI (should set sidebar root)
    Directory(PathBuf),
    /// App icon clicked (reopen event)
    Reopen,
}

/// A global receiver to receive open events from the main thread
pub static OPEN_EVENT_RECEIVER: Mutex<Option<Receiver<OpenEvent>>> = Mutex::new(None);

/// Global broadcast sender for opening files in tabs
pub static FILE_OPEN_BROADCAST: LazyLock<broadcast::Sender<PathBuf>> =
    LazyLock::new(|| broadcast::channel(100).0);

/// Global broadcast sender for opening directories in sidebar
pub static DIRECTORY_OPEN_BROADCAST: LazyLock<broadcast::Sender<PathBuf>> =
    LazyLock::new(|| broadcast::channel(100).0);

pub static LAST_SELECTED_THEME: LazyLock<Mutex<ThemePreference>> =
    LazyLock::new(|| Mutex::new(ThemePreference::default()));

/// Content source for a tab
#[derive(Debug, Clone, PartialEq)]
pub enum TabContent {
    /// No content (shows NoFile component)
    None,
    /// File from filesystem
    File(PathBuf),
    /// Inline markdown content (for welcome screen)
    Inline(String),
    /// File that cannot be opened (binary or error)
    FileError(PathBuf, String),
}

/// Represents a single tab with its content and navigation history
#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub content: TabContent,
    pub history: HistoryManager,
}

impl Tab {
    pub fn new(file: Option<PathBuf>) -> Self {
        let mut history = HistoryManager::new();
        let content = match file {
            Some(path) => {
                history.push(path.clone());
                TabContent::File(path)
            }
            None => TabContent::None,
        };
        Self { content, history }
    }

    pub fn with_inline_content(content: String) -> Self {
        Self {
            content: TabContent::Inline(content),
            history: HistoryManager::new(),
        }
    }

    /// Get the file path if this tab has a file
    pub fn file(&self) -> Option<&PathBuf> {
        match &self.content {
            TabContent::File(path) | TabContent::FileError(path, _) => Some(path),
            _ => None,
        }
    }
}

/// Represents the state of the sidebar file explorer
#[derive(Debug, Clone, PartialEq)]
pub struct SidebarState {
    pub is_visible: bool,
    pub root_directory: Option<PathBuf>,
    pub expanded_dirs: HashSet<PathBuf>,
    pub width: f64,              // Width in pixels
    pub hide_non_markdown: bool, // Hide non-markdown files
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub tabs: Signal<Vec<Tab>>,
    pub active_tab: Signal<usize>,
    pub current_theme: Signal<ThemePreference>,
    pub zoom_level: Signal<f64>,
    pub sidebar: Signal<SidebarState>,
}

impl Default for AppState {
    fn default() -> Self {
        // Set current working directory as default root
        let current_dir = std::env::current_dir().ok();

        Self {
            tabs: Signal::new(vec![Tab::new(None)]),
            active_tab: Signal::new(0),
            current_theme: Signal::new(*LAST_SELECTED_THEME.lock().unwrap()),
            zoom_level: Signal::new(1.0),
            sidebar: Signal::new(SidebarState {
                is_visible: false,
                root_directory: current_dir,
                expanded_dirs: HashSet::new(),
                width: 280.0,            // Default width
                hide_non_markdown: true, // Hide non-markdown files by default
            }),
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

    /// Check if the current active tab has no file (NoFile tab, Inline content, or FileError)
    /// None, Inline content, and FileError can be replaced when opening a file
    pub fn is_current_tab_no_file(&self) -> bool {
        self.current_tab()
            .map(|tab| {
                matches!(
                    tab.content,
                    TabContent::None | TabContent::Inline(_) | TabContent::FileError(_, _)
                )
            })
            .unwrap_or(false)
    }

    /// Find the index of a tab that has the specified file open
    pub fn find_tab_with_file(&self, file: &PathBuf) -> Option<usize> {
        let tabs = self.tabs.read();
        tabs.iter()
            .position(|tab| tab.file().map(|f| f == file).unwrap_or(false))
    }

    /// Open a file, reusing NoFile tab or existing tab with the same file if possible
    /// Used when opening from sidebar or external sources
    pub fn open_file(&mut self, file: PathBuf) {
        // Check if the file is already open in another tab
        if let Some(tab_index) = self.find_tab_with_file(&file) {
            // Switch to the existing tab instead of creating a new one
            self.switch_to_tab(tab_index);
        } else if self.is_current_tab_no_file() {
            // If current tab is NoFile, open the file in it
            self.update_current_tab(|tab| {
                tab.history.push(file.clone());
                tab.content = TabContent::File(file);
            });
        } else {
            // Otherwise, create a new tab
            self.add_tab(Some(file), true);
        }
    }

    /// Navigate to a file in the current tab (for in-tab navigation like markdown links)
    /// Always opens in current tab regardless of whether file is open elsewhere
    pub fn navigate_to_file(&mut self, file: PathBuf) {
        self.update_current_tab(|tab| {
            tab.history.push(file.clone());
            tab.content = TabContent::File(file);
        });
    }

    /// Toggle sidebar visibility
    pub fn toggle_sidebar(&mut self) {
        let mut sidebar = self.sidebar.write();
        sidebar.is_visible = !sidebar.is_visible;
    }

    /// Set the root directory for the sidebar file explorer
    pub fn set_root_directory(&mut self, path: PathBuf) {
        let mut sidebar = self.sidebar.write();
        sidebar.root_directory = Some(path);
    }

    /// Toggle directory expansion state
    pub fn toggle_directory_expansion(&mut self, path: PathBuf) {
        let mut sidebar = self.sidebar.write();
        if sidebar.expanded_dirs.contains(&path) {
            sidebar.expanded_dirs.remove(&path);
        } else {
            sidebar.expanded_dirs.insert(path);
        }
    }
}
