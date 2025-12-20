use crate::config::{NewWindowBehavior, StartupBehavior, CONFIG};
use crate::state::LAST_FOCUSED_STATE;
use crate::theme::ThemePreference;
use std::path::{Path, PathBuf};

// ============================================================================
// Value Types
// ============================================================================

pub struct ThemeValue {
    pub theme: ThemePreference,
}

pub struct DirectoryValue {
    pub directory: PathBuf,
}

pub struct SidebarValue {
    pub open: bool,
    pub width: f64,
    pub show_all_files: bool,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Resolve directory with fallback: provided path -> home directory -> root
fn resolve_directory(dir: Option<PathBuf>) -> PathBuf {
    dir.or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("/"))
}

// ============================================================================
// Public API
// ============================================================================

pub fn get_theme_value(is_first_window: bool) -> ThemeValue {
    let cfg = CONFIG.read();
    let theme = if is_first_window {
        match cfg.theme.on_startup {
            StartupBehavior::Default => cfg.theme.default_theme,
            StartupBehavior::LastClosed => LAST_FOCUSED_STATE.read().theme,
        }
    } else {
        match cfg.theme.on_new_window {
            NewWindowBehavior::Default => cfg.theme.default_theme,
            NewWindowBehavior::LastFocused => LAST_FOCUSED_STATE.read().theme,
        }
    };
    ThemeValue { theme }
}

pub fn get_directory_value(
    is_first_window: bool,
    file: Option<impl AsRef<Path>>,
    directory: Option<impl AsRef<Path>>,
) -> DirectoryValue {
    let directory = {
        if let Some(directory) = directory.map(|v| v.as_ref().to_owned()) {
            // Use the specified directory
            directory
        } else if let Some(directory) =
            file.and_then(|v| v.as_ref().parent().map(ToOwned::to_owned))
        {
            // Use parent directory of the specified file
            directory
        } else {
            // Use default or last directory
            let cfg = CONFIG.read();
            let directory: Option<PathBuf> = if is_first_window {
                match cfg.directory.on_startup {
                    StartupBehavior::Default => cfg.directory.default_directory.clone(),
                    StartupBehavior::LastClosed => LAST_FOCUSED_STATE
                        .read()
                        .directory
                        .clone()
                        .or_else(|| cfg.directory.default_directory.clone()),
                }
            } else {
                match cfg.directory.on_new_window {
                    NewWindowBehavior::Default => cfg.directory.default_directory.clone(),
                    NewWindowBehavior::LastFocused => LAST_FOCUSED_STATE
                        .read()
                        .directory
                        .clone()
                        .or_else(|| cfg.directory.default_directory.clone()),
                }
            };
            resolve_directory(directory)
        }
    };
    DirectoryValue { directory }
}

pub fn get_sidebar_value(is_first_window: bool) -> SidebarValue {
    let cfg = CONFIG.read();
    if is_first_window {
        match cfg.sidebar.on_startup {
            StartupBehavior::Default => SidebarValue {
                open: cfg.sidebar.default_open,
                width: cfg.sidebar.default_width,
                show_all_files: cfg.sidebar.default_show_all_files,
            },
            StartupBehavior::LastClosed => {
                let state = LAST_FOCUSED_STATE.read();
                SidebarValue {
                    open: state.sidebar_open,
                    width: state.sidebar_width,
                    show_all_files: state.sidebar_show_all_files,
                }
            }
        }
    } else {
        match cfg.sidebar.on_new_window {
            NewWindowBehavior::Default => SidebarValue {
                open: cfg.sidebar.default_open,
                width: cfg.sidebar.default_width,
                show_all_files: cfg.sidebar.default_show_all_files,
            },
            NewWindowBehavior::LastFocused => {
                let state = LAST_FOCUSED_STATE.read();
                SidebarValue {
                    open: state.sidebar_open,
                    width: state.sidebar_width,
                    show_all_files: state.sidebar_show_all_files,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_directory_with_some() {
        let path = PathBuf::from("/custom/path");
        let result = resolve_directory(Some(path.clone()));
        assert_eq!(result, path);
    }

    #[test]
    fn test_resolve_directory_with_none() {
        let result = resolve_directory(None);
        // Should return home directory or root
        assert!(
            result == dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
                || result.as_path() == std::path::Path::new("/")
        );
    }

    #[test]
    fn test_get_theme_value_first_window() {
        let result = get_theme_value(true);
        // Should return a ThemeValue
        assert!(matches!(
            result.theme,
            ThemePreference::Auto | ThemePreference::Light | ThemePreference::Dark
        ));
    }

    #[test]
    fn test_get_theme_value_new_window() {
        let result = get_theme_value(false);
        // Should return a ThemeValue
        assert!(matches!(
            result.theme,
            ThemePreference::Auto | ThemePreference::Light | ThemePreference::Dark
        ));
    }

    #[test]
    fn test_get_directory_value_first_window() {
        let result = get_directory_value(true, None::<PathBuf>, None::<PathBuf>);
        // Should return a DirectoryValue with a non-empty path
        assert!(!result.directory.as_os_str().is_empty());
    }

    #[test]
    fn test_get_directory_value_new_window() {
        let result = get_directory_value(false, None::<PathBuf>, None::<PathBuf>);
        // Should return a DirectoryValue with a non-empty path
        assert!(!result.directory.as_os_str().is_empty());
    }

    #[test]
    fn test_get_sidebar_value_first_window() {
        let result = get_sidebar_value(true);
        // Should return a SidebarValue
        assert!(result.width > 0.0);
    }

    #[test]
    fn test_get_sidebar_value_new_window() {
        let result = get_sidebar_value(false);
        // Should return a SidebarValue
        assert!(result.width > 0.0);
    }
}
