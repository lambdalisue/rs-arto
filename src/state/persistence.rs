use crate::state::AppState;
use crate::theme::ThemePreference;
use dioxus::prelude::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Persisted state from the last closed window
///
/// This is a subset of AppState that gets saved to session.json
/// when a window closes and loaded on app startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedState {
    pub directory: Option<PathBuf>,
    pub theme: ThemePreference,
    pub sidebar_open: bool,
    pub sidebar_width: f64,
    pub sidebar_show_all_files: bool,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            directory: None,
            theme: ThemePreference::default(),
            sidebar_open: false,
            sidebar_width: 280.0,
            sidebar_show_all_files: false,
        }
    }
}

impl From<&AppState> for PersistedState {
    fn from(state: &AppState) -> Self {
        let sidebar = state.sidebar.read();
        Self {
            directory: state.directory.read().clone(),
            theme: *state.current_theme.read(),
            sidebar_open: sidebar.open,
            sidebar_width: sidebar.width,
            sidebar_show_all_files: sidebar.show_all_files,
        }
    }
}

impl PersistedState {
    /// Get the state file path (state.json in local data directory)
    pub fn path() -> PathBuf {
        const FILENAME: &str = "state.json";
        if let Some(mut path) = dirs::data_local_dir() {
            path.push("arto");
            path.push(FILENAME);
            return path;
        }

        // Fallback to home directory
        if let Some(mut path) = dirs::home_dir() {
            path.push(".arto");
            path.push(FILENAME);
            return path;
        }

        PathBuf::from(FILENAME)
    }

    /// Load persisted state from file or return default
    pub fn load() -> Self {
        let path = Self::path();

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save persisted state to file
    ///
    /// This function should be called when a window is closing to persist its state.
    pub fn save(&self) {
        let path = Self::path();

        tracing::debug!(
            path = %path.display(),
            theme = ?self.theme,
            sidebar_open = self.sidebar_open,
            sidebar_width = self.sidebar_width,
            sidebar_show_all_files = self.sidebar_show_all_files,
            "Saving persisted state"
        );

        // Save to file synchronously
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::error!(?e, "Failed to create session directory");
                return;
            }
        }

        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    tracing::error!(?e, "Failed to save persisted state");
                }
            }
            Err(e) => {
                tracing::error!(?e, "Failed to serialize persisted state");
            }
        }
    }
}

/// Last focused window state (used for "last_focused" behavior when opening new windows)
pub static LAST_FOCUSED_STATE: LazyLock<RwLock<PersistedState>> =
    LazyLock::new(|| RwLock::new(PersistedState::load()));
