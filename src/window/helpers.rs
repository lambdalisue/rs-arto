use dioxus::desktop::tao::dpi::{LogicalPosition, LogicalSize};
use dioxus::desktop::tao::window::Window;
use mouse_position::mouse_position::Mouse;
use std::path::{Path, PathBuf};

use super::types::{ResolvedWindowValue, WindowMetrics};
use crate::config::{
    NewWindowBehavior, StartupBehavior, WindowDimension, WindowDimensionUnit, WindowPosition,
    WindowPositionMode, WindowSize, CONFIG,
};
use crate::state::{Position, Size, LAST_FOCUSED_STATE};
use crate::theme::ThemePreference;
use crate::utils::screen::{get_current_display_size, get_cursor_display, get_primary_display};

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

fn choose_by_behavior<T>(
    is_first_window: bool,
    on_startup: StartupBehavior,
    on_new_window: NewWindowBehavior,
    default: impl FnOnce() -> T,
    last: impl FnOnce() -> T,
) -> T {
    if is_first_window {
        match on_startup {
            StartupBehavior::Default => default(),
            StartupBehavior::LastClosed => last(),
        }
    } else {
        match on_new_window {
            NewWindowBehavior::Default => default(),
            NewWindowBehavior::LastFocused => last(),
        }
    }
}

fn resolve_window_size(config: WindowSize, max_size: LogicalSize<f64>) -> LogicalSize<f64> {
    let size = config.to_logical_size(&max_size);
    LogicalSize::new(size.width.max(100.0), size.height.max(100.0))
}

fn resolve_window_position(
    config: WindowPosition,
    screen_size: LogicalSize<f64>,
    window_size: LogicalSize<f64>,
) -> LogicalPosition<f64> {
    let available_width = (screen_size.width - window_size.width).max(0.0);
    let available_height = (screen_size.height - window_size.height).max(0.0);
    let available_size = LogicalSize::new(available_width as i32, available_height as i32);
    let position = config.to_logical_position(available_size);
    LogicalPosition::new(position.x as f64, position.y as f64)
}

fn resolve_window_position_from_cursor(
    window_size: LogicalSize<f64>,
) -> Option<LogicalPosition<f64>> {
    let (x, y) = match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => (x as f64, y as f64),
        Mouse::Error => return None,
    };
    let display = get_cursor_display().or_else(get_primary_display)?;
    let scale = display.scale_factor as f64;
    let display_x_physical = display.x as f64;
    let display_y_physical = display.y as f64;
    let display_width_physical = display.width as f64;
    let display_height_physical = display.height as f64;
    let display_x = display_x_physical / scale;
    let display_y = display_y_physical / scale;
    let display_width = display_width_physical / scale;
    let display_height = display_height_physical / scale;
    let (cursor_x, cursor_y) = if x >= display_x_physical
        && x < display_x_physical + display_width_physical
        && y >= display_y_physical
        && y < display_y_physical + display_height_physical
    {
        (x / scale, y / scale)
    } else if x >= display_x
        && x < display_x + display_width
        && y >= display_y
        && y < display_y + display_height
    {
        (x, y)
    } else {
        (x / scale, y / scale)
    };
    let max_x = (display_x + display_width - window_size.width).max(display_x);
    let max_y = (display_y + display_height - window_size.height).max(display_y);
    let clamped_x = cursor_x.clamp(display_x, max_x);
    let clamped_y = cursor_y.clamp(display_y, max_y);
    Some(LogicalPosition::new(clamped_x, clamped_y))
}

fn window_size_from_state(size: Size) -> WindowSize {
    WindowSize {
        width: WindowDimension {
            value: size.width as f64,
            unit: WindowDimensionUnit::Pixels,
        },
        height: WindowDimension {
            value: size.height as f64,
            unit: WindowDimensionUnit::Pixels,
        },
    }
}

fn window_position_from_state(position: Position) -> WindowPosition {
    WindowPosition {
        x: WindowDimension {
            value: position.x as f64,
            unit: WindowDimensionUnit::Pixels,
        },
        y: WindowDimension {
            value: position.y as f64,
            unit: WindowDimensionUnit::Pixels,
        },
    }
}

fn resolve_window_settings(
    is_first_window: bool,
) -> (WindowPosition, WindowPositionMode, WindowSize) {
    let cfg = CONFIG.read();
    let position = choose_by_behavior(
        is_first_window,
        cfg.window_position.on_startup,
        cfg.window_position.on_new_window,
        || cfg.window_position.default_position,
        || window_position_from_state(LAST_FOCUSED_STATE.read().window_position),
    );
    let position_mode = choose_by_behavior(
        is_first_window,
        cfg.window_position.on_startup,
        cfg.window_position.on_new_window,
        || cfg.window_position.default_position_mode,
        || WindowPositionMode::Coordinates,
    );
    let size = choose_by_behavior(
        is_first_window,
        cfg.window_size.on_startup,
        cfg.window_size.on_new_window,
        || cfg.window_size.default_size,
        || window_size_from_state(LAST_FOCUSED_STATE.read().window_size),
    );

    (position, position_mode, size)
}

// ============================================================================
// Public API
// ============================================================================

pub fn get_theme_value(is_first_window: bool) -> ThemeValue {
    let cfg = CONFIG.read();
    let theme = choose_by_behavior(
        is_first_window,
        cfg.theme.on_startup,
        cfg.theme.on_new_window,
        || cfg.theme.default_theme,
        || LAST_FOCUSED_STATE.read().theme,
    );
    ThemeValue { theme }
}

pub fn get_directory_value(
    is_first_window: bool,
    file: Option<impl AsRef<Path>>,
    directory: Option<impl AsRef<Path>>,
) -> DirectoryValue {
    let directory = if let Some(directory) = directory.map(|v| v.as_ref().to_owned()) {
        // Use the specified directory
        directory
    } else if let Some(directory) = file.and_then(|v| v.as_ref().parent().map(ToOwned::to_owned)) {
        // Use parent directory of the specified file
        directory
    } else {
        // Use default or last directory
        let cfg = CONFIG.read();
        let directory: Option<PathBuf> = choose_by_behavior(
            is_first_window,
            cfg.directory.on_startup,
            cfg.directory.on_new_window,
            || cfg.directory.default_directory.clone(),
            || {
                LAST_FOCUSED_STATE
                    .read()
                    .directory
                    .clone()
                    .or_else(|| cfg.directory.default_directory.clone())
            },
        );
        resolve_directory(directory)
    };
    DirectoryValue { directory }
}

pub fn get_sidebar_value(is_first_window: bool) -> SidebarValue {
    let cfg = CONFIG.read();
    choose_by_behavior(
        is_first_window,
        cfg.sidebar.on_startup,
        cfg.sidebar.on_new_window,
        || SidebarValue {
            open: cfg.sidebar.default_open,
            width: cfg.sidebar.default_width,
            show_all_files: cfg.sidebar.default_show_all_files,
        },
        || {
            let state = LAST_FOCUSED_STATE.read();
            SidebarValue {
                open: state.sidebar_open,
                width: state.sidebar_width,
                show_all_files: state.sidebar_show_all_files,
            }
        },
    )
}

pub fn get_window_value(is_first_window: bool) -> ResolvedWindowValue {
    let (position, position_mode, size) = resolve_window_settings(is_first_window);
    let screen_size = get_current_display_size().unwrap_or_else(|| LogicalSize::new(1000.0, 800.0));
    let resolved_size = resolve_window_size(size, screen_size);
    let resolved_position = match position_mode {
        WindowPositionMode::Coordinates => {
            resolve_window_position(position, screen_size, resolved_size)
        }
        WindowPositionMode::Mouse => resolve_window_position_from_cursor(resolved_size)
            .unwrap_or_else(|| LogicalPosition::new(0.0, 0.0)),
    };
    ResolvedWindowValue {
        position: resolved_position,
        size: resolved_size,
    }
}

pub fn capture_window_metrics(window: &Window) -> WindowMetrics {
    let scale = window.scale_factor();
    let position = window
        .outer_position()
        .map(|pos| pos.to_logical::<i32>(scale))
        .unwrap_or_else(|_| LogicalPosition::new(0, 0));
    let size = window.outer_size().to_logical::<u32>(scale);
    WindowMetrics {
        position: Position {
            x: position.x,
            y: position.y,
        },
        size: Size {
            width: size.width,
            height: size.height,
        },
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

    #[test]
    fn test_get_window_value_first_window() {
        let result = get_window_value(true);
        assert!(result.size.width > 0.0);
        assert!(result.size.height > 0.0);
    }

    #[test]
    fn test_get_window_value_new_window() {
        let result = get_window_value(false);
        assert!(result.size.width > 0.0);
        assert!(result.size.height > 0.0);
    }
}
