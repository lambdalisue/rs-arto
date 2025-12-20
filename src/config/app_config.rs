use serde::{Deserialize, Serialize};

mod behavior;
mod directory_config;
mod sidebar_config;
mod theme_config;

pub use behavior::{NewWindowBehavior, StartupBehavior};
pub use directory_config::DirectoryConfig;
pub use sidebar_config::SidebarConfig;
pub use theme_config::ThemeConfig;

/// Global application configuration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub directory: DirectoryConfig,
    pub theme: ThemeConfig,
    pub sidebar: SidebarConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemePreference;
    use std::path::PathBuf;

    #[test]
    fn test_config_default() {
        let config = Config::default();

        // Theme defaults
        assert_eq!(config.theme.default_theme, ThemePreference::Auto);
        assert_eq!(config.theme.on_startup, StartupBehavior::Default);
        assert_eq!(config.theme.on_new_window, NewWindowBehavior::Default);

        // Directory defaults
        assert_eq!(config.directory.default_directory, None);
        assert_eq!(config.directory.on_startup, StartupBehavior::Default);
        assert_eq!(config.directory.on_new_window, NewWindowBehavior::Default);

        // Sidebar defaults
        assert!(!config.sidebar.default_open); // Default is false
        assert_eq!(config.sidebar.default_width, 280.0);
        assert!(!config.sidebar.default_show_all_files);
        assert_eq!(config.sidebar.on_startup, StartupBehavior::Default);
        assert_eq!(config.sidebar.on_new_window, NewWindowBehavior::Default);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config {
            theme: ThemeConfig {
                default_theme: ThemePreference::Dark,
                on_startup: StartupBehavior::LastClosed,
                on_new_window: NewWindowBehavior::LastFocused,
            },
            directory: DirectoryConfig {
                default_directory: Some(PathBuf::from("/home/user")),
                on_startup: StartupBehavior::Default,
                on_new_window: NewWindowBehavior::Default,
            },
            sidebar: SidebarConfig {
                default_open: false,
                default_width: 320.0,
                default_show_all_files: true,
                on_startup: StartupBehavior::LastClosed,
                on_new_window: NewWindowBehavior::LastFocused,
            },
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.theme.default_theme, ThemePreference::Dark);
        assert_eq!(parsed.theme.on_startup, StartupBehavior::LastClosed);
        assert_eq!(
            parsed.directory.default_directory,
            Some(PathBuf::from("/home/user"))
        );
        assert!(!parsed.sidebar.default_open);
        assert_eq!(parsed.sidebar.default_width, 320.0);
    }
}
