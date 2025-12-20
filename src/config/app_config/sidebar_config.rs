use super::behavior::{NewWindowBehavior, StartupBehavior};
use serde::{Deserialize, Serialize};

/// Default sidebar width in pixels
pub const DEFAULT_SIDEBAR_WIDTH: f64 = 280.0;

fn default_sidebar_width() -> f64 {
    DEFAULT_SIDEBAR_WIDTH
}

/// Configuration for sidebar-related settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarConfig {
    /// Whether sidebar is open by default
    pub default_open: bool,
    /// Default sidebar width in pixels
    #[serde(default = "default_sidebar_width")]
    pub default_width: f64,
    /// Whether to show all files (including non-markdown) by default
    pub default_show_all_files: bool,
    /// Behavior on app startup: "default" or "last_closed"
    pub on_startup: StartupBehavior,
    /// Behavior when opening a new window: "default" or "last_focused"
    pub on_new_window: NewWindowBehavior,
}

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            default_open: false,
            default_width: default_sidebar_width(),
            default_show_all_files: false,
            on_startup: StartupBehavior::Default,
            on_new_window: NewWindowBehavior::Default,
        }
    }
}
