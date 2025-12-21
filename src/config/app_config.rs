use serde::{Deserialize, Serialize};

mod behavior;
mod directory_config;
mod sidebar_config;
mod theme_config;
mod window_dimension;
mod window_position_config;
mod window_size_config;

pub use behavior::{NewWindowBehavior, StartupBehavior};
pub use directory_config::DirectoryConfig;
pub use sidebar_config::SidebarConfig;
pub use theme_config::ThemeConfig;
pub use window_dimension::{WindowDimension, WindowDimensionUnit};
pub use window_position_config::{
    WindowPosition, WindowPositionConfig, WindowPositionMode, WindowPositionOffset,
};
pub use window_size_config::{WindowSize, WindowSizeConfig};

/// Global application configuration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub directory: DirectoryConfig,
    pub theme: ThemeConfig,
    pub sidebar: SidebarConfig,
    pub window_position: WindowPositionConfig,
    pub window_size: WindowSizeConfig,
}

#[cfg(test)]
mod tests {
    use super::window_position_config::WindowPositionOffset;
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

        // Window size defaults
        assert_eq!(config.window_size.default_size.width.value, 1000.0);
        assert_eq!(
            config.window_size.default_size.width.unit,
            WindowDimensionUnit::Pixels
        );
        assert_eq!(config.window_size.default_size.height.value, 800.0);
        assert_eq!(
            config.window_size.default_size.height.unit,
            WindowDimensionUnit::Pixels
        );
        assert_eq!(config.window_size.on_startup, StartupBehavior::Default);
        assert_eq!(config.window_size.on_new_window, NewWindowBehavior::Default);

        // Window position defaults
        assert_eq!(
            config.window_position.default_position_mode,
            WindowPositionMode::Coordinates
        );
        assert_eq!(config.window_position.position_offset.x, 10);
        assert_eq!(config.window_position.position_offset.y, 10);
        assert_eq!(config.window_position.default_position.x.value, 50.0);
        assert_eq!(
            config.window_position.default_position.x.unit,
            WindowDimensionUnit::Percent
        );
        assert_eq!(config.window_position.default_position.y.value, 50.0);
        assert_eq!(
            config.window_position.default_position.y.unit,
            WindowDimensionUnit::Percent
        );
        assert_eq!(config.window_position.on_startup, StartupBehavior::Default);
        assert_eq!(
            config.window_position.on_new_window,
            NewWindowBehavior::Default
        );
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
            window_position: WindowPositionConfig {
                default_position: WindowPosition {
                    x: WindowDimension {
                        value: 10.0,
                        unit: WindowDimensionUnit::Percent,
                    },
                    y: WindowDimension {
                        value: 15.0,
                        unit: WindowDimensionUnit::Percent,
                    },
                },
                default_position_mode: WindowPositionMode::Mouse,
                position_offset: WindowPositionOffset { x: 24, y: 12 },
                on_startup: StartupBehavior::LastClosed,
                on_new_window: NewWindowBehavior::LastFocused,
            },
            window_size: WindowSizeConfig {
                default_size: WindowSize {
                    width: WindowDimension {
                        value: 1200.0,
                        unit: WindowDimensionUnit::Pixels,
                    },
                    height: WindowDimension {
                        value: 85.0,
                        unit: WindowDimensionUnit::Percent,
                    },
                },
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
        assert_eq!(parsed.window_position.default_position.x.value, 10.0);
        assert_eq!(
            parsed.window_position.default_position.x.unit,
            WindowDimensionUnit::Percent
        );
        assert_eq!(
            parsed.window_position.default_position_mode,
            WindowPositionMode::Mouse
        );
        assert_eq!(parsed.window_position.position_offset.x, 24);
        assert_eq!(parsed.window_size.default_size.width.value, 1200.0);
        assert_eq!(
            parsed.window_size.default_size.width.unit,
            WindowDimensionUnit::Pixels
        );
    }
}
