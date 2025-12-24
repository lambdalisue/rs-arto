use dioxus::desktop::tao::dpi::{LogicalPosition, LogicalSize};
use dioxus::prelude::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::state::AppState;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<LogicalPosition<i32>> for Position {
    fn from(from: LogicalPosition<i32>) -> Self {
        Self {
            x: from.x,
            y: from.y,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl From<LogicalSize<u32>> for Size {
    fn from(from: LogicalSize<u32>) -> Self {
        Self {
            width: from.width,
            height: from.height,
        }
    }
}

/// Persisted state from the last closed window
///
/// This is a subset of AppState that gets saved to session.json
/// when a window closes and loaded on app startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PersistedState {
    pub directory: Option<PathBuf>,
    pub theme: Theme,
    pub sidebar_open: bool,
    pub sidebar_width: f64,
    pub sidebar_show_all_files: bool,
    pub window_position: Position,
    pub window_size: Size,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            directory: None,
            theme: Theme::default(),
            sidebar_open: false,
            sidebar_width: 280.0,
            sidebar_show_all_files: false,
            window_position: Position::default(),
            window_size: Size::default(),
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
            window_position: (*state.position.read()).into(),
            window_size: (*state.size.read()).into(),
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
