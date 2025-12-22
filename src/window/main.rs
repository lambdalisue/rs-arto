use dioxus::desktop::tao::dpi::{LogicalPosition, LogicalSize};
use dioxus::desktop::tao::window::WindowId;
use dioxus::desktop::{window, Config, WeakDesktopContext, WindowBuilder};
use dioxus::prelude::*;

use std::cell::RefCell;
use std::path::PathBuf;

use crate::assets::MAIN_STYLE;
use crate::components::app::{App, AppProps};
use crate::config::{WindowPositionOffset, CONFIG};
use crate::state::LAST_FOCUSED_STATE;
use crate::utils::screen::get_current_display_bounds;

use super::child;
use super::index::build_custom_index;
use super::metrics::capture_window_metrics;
use super::settings;
use super::types::WindowMetrics;

const MAX_POSITION_SHIFT_ATTEMPTS: usize = 20;

thread_local! {
    static MAIN_WINDOWS: RefCell<Vec<WeakDesktopContext>> = const { RefCell::new(Vec::new()) };
    static LAST_FOCUSED_WINDOW: RefCell<Option<WindowId>> = const { RefCell::new(None) };
}

pub fn register_main_window(handle: WeakDesktopContext) {
    MAIN_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        windows.retain(|w| w.upgrade().is_some());
        if !windows.iter().any(|w| w.ptr_eq(&handle)) {
            windows.push(handle);
        }
    });
}

/// Checks if there are any visible main windows.
///
/// Note: With WindowCloseBehaviour::WindowHides, closed windows remain in memory
/// with valid weak references but are not visible. We must check visibility to
/// avoid sending events (e.g., FILE_OPEN_BROADCAST) to hidden windows, which would
/// be invisible to users.
pub fn has_any_main_windows() -> bool {
    MAIN_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        // Remove destroyed windows
        windows.retain(|w| w.upgrade().is_some());
        // Check if any remaining windows are actually visible
        windows.iter().any(|w| {
            w.upgrade()
                .map(|ctx| ctx.window.is_visible())
                .unwrap_or(false)
        })
    })
}

pub fn focus_last_focused_main_window() -> bool {
    if let Some(window_id) = get_last_focused_window() {
        // Resolve to parent window if the last focused was a child window
        let main_window_id = child::resolve_to_parent_window(window_id);

        MAIN_WINDOWS.with(|windows| {
            windows
                .borrow()
                .iter()
                .filter_map(|w| w.upgrade())
                .find(|ctx| ctx.window.id() == main_window_id)
                .map(|ctx| {
                    ctx.window.set_focus();
                    true
                })
                .unwrap_or(false)
        })
    } else {
        false
    }
}

pub fn close_all_main_windows() {
    let windows = MAIN_WINDOWS.with(|w| {
        w.borrow()
            .iter()
            .filter_map(|w| w.upgrade())
            .collect::<Vec<_>>()
    });

    windows.iter().for_each(|w| w.close());
    MAIN_WINDOWS.with(|w| w.borrow_mut().clear());
}

pub fn create_new_main_window(
    file: Option<PathBuf>,
    directory: Option<PathBuf>,
    show_welcome: bool,
) {
    dioxus_core::spawn(async move {
        create_new_main_window_async(file, directory, show_welcome).await;
    });
}

pub async fn create_new_main_window_async(
    file: Option<PathBuf>,
    directory: Option<PathBuf>,
    show_welcome: bool,
) {
    // Check if this is the first window (0 -> 1 transition)
    // Use "On Startup" (Last Closed) for first window, "On New Window" (Last Focused) for additional
    let is_first_window = !has_any_main_windows();

    // Get theme from config and state
    let theme_value = settings::get_theme_value(is_first_window);

    // Get directory from config and state
    let directory_value = settings::get_directory_value(is_first_window, file.as_ref(), directory);

    // Get sidebar settings from config and state
    let sidebar_value = settings::get_sidebar_value(is_first_window);

    // Get window settings from config and state
    let resolved = settings::get_window_value(is_first_window);
    let position_offset = CONFIG.read().window_position.position_offset;
    let (screen_origin, screen_size) = get_current_display_bounds()
        .unwrap_or_else(|| (LogicalPosition::new(0, 0), LogicalSize::new(1000, 800)));
    let occupied = existing_main_window_positions();
    let shifted_position = shift_position_if_needed(
        resolved.position,
        resolved.size,
        position_offset,
        screen_origin,
        screen_size,
        &occupied,
    );
    tracing::debug!(
        screen_size=?screen_size,
        position_offset=?position_offset,
        resolved_position=?resolved.position,
        shifted_position=?shifted_position,
        "Shifted position is calculated"
    );

    // This cause ERROR but it seems we can ignore it safely
    // https://github.com/DioxusLabs/dioxus/issues/3872
    let dom = VirtualDom::new_with_props(
        App,
        AppProps {
            file,
            directory: directory_value.directory,
            sidebar_open: sidebar_value.open,
            sidebar_width: sidebar_value.width,
            sidebar_show_all_files: sidebar_value.show_all_files,
            show_welcome,
        },
    );
    let config = Config::new()
        .with_menu(None) // To avoid child window taking over the main window's menu
        .with_window(
            WindowBuilder::new()
                .with_title("Arto")
                .with_position(shifted_position)
                .with_inner_size(resolved.size),
        )
        // Add main style in config. Otherwise the style takes time to load and
        // the window appears unstyled for a brief moment.
        .with_custom_head(indoc::formatdoc! {r#"<link rel="stylesheet" href="{MAIN_STYLE}">"#})
        // Use a custom index to set the initial theme correctly
        .with_custom_index(build_custom_index(theme_value.theme));

    let pending = window().new_window(dom, config);
    let handle = pending.await;
    register_main_window(std::rc::Rc::downgrade(&handle));
}

pub fn update_last_focused_window(window_id: WindowId) {
    LAST_FOCUSED_WINDOW.with(|last| *last.borrow_mut() = Some(window_id));
    if let Some(metrics) = find_window_metrics(window_id) {
        let mut last_focused = LAST_FOCUSED_STATE.write();
        last_focused.window_position = metrics.position;
        last_focused.window_size = metrics.size;
    }
}

pub(crate) fn get_last_focused_window() -> Option<WindowId> {
    LAST_FOCUSED_WINDOW.with(|last| *last.borrow())
}

fn find_window_metrics(window_id: WindowId) -> Option<WindowMetrics> {
    MAIN_WINDOWS.with(|windows| {
        windows
            .borrow()
            .iter()
            .filter_map(|w| w.upgrade())
            .find(|ctx| ctx.window.id() == window_id)
            .map(|ctx| capture_window_metrics(&ctx.window))
    })
}

fn existing_main_window_positions() -> Vec<LogicalPosition<i32>> {
    MAIN_WINDOWS.with(|windows| {
        windows
            .borrow()
            .iter()
            .filter_map(|w| w.upgrade())
            .map(|ctx| {
                let metrics = capture_window_metrics(&ctx.window);
                LogicalPosition::new(metrics.position.x, metrics.position.y)
            })
            .collect()
    })
}

fn shift_position_if_needed(
    base: LogicalPosition<i32>,
    window_size: LogicalSize<u32>,
    offset: WindowPositionOffset,
    screen_origin: LogicalPosition<i32>,
    screen_size: LogicalSize<u32>,
    occupied: &[LogicalPosition<i32>],
) -> LogicalPosition<i32> {
    if offset.x == 0 && offset.y == 0 {
        return base;
    }
    let min_x = screen_origin.x;
    let min_y = screen_origin.y;
    let max_x = (screen_origin.x + screen_size.width as i32 - window_size.width as i32).max(min_x);
    let max_y =
        (screen_origin.y + screen_size.height as i32 - window_size.height as i32).max(min_y);
    let mut position = LogicalPosition::new(base.x.clamp(min_x, max_x), base.y.clamp(min_y, max_y));
    let mut offset_x = offset.x;
    let mut offset_y = offset.y;
    for _ in 0..MAX_POSITION_SHIFT_ATTEMPTS {
        // Heuristic: avoid identical/nearby top-left positions rather than full rect overlap.
        let x_half = offset_x.abs().max(1) / 2;
        let y_half = offset_y.abs().max(1) / 2;
        let x_min = position.x - x_half;
        let x_max = position.x + x_half;
        let y_min = position.y - y_half;
        let y_max = position.y + y_half;
        if !occupied.iter().any(|existing| {
            existing.x >= x_min && existing.x <= x_max && existing.y >= y_min && existing.y <= y_max
        }) {
            break;
        }
        let mut next_x = position.x + offset_x;
        let mut next_y = position.y + offset_y;
        if next_x < min_x || next_x > max_x {
            offset_x = -offset_x;
            next_x = position.x + offset_x;
        }
        if next_y < min_y || next_y > max_y {
            offset_y = -offset_y;
            next_y = position.y + offset_y;
        }
        position = LogicalPosition::new(next_x.clamp(min_x, max_x), next_y.clamp(min_y, max_y));
    }
    position
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::desktop::tao::dpi::{LogicalPosition, LogicalSize};

    #[test]
    fn test_shift_position_if_needed_no_offset() {
        let base = LogicalPosition::new(10, 10);
        let result = shift_position_if_needed(
            base,
            LogicalSize::new(100, 100),
            WindowPositionOffset { x: 0, y: 0 },
            LogicalPosition::new(0, 0),
            LogicalSize::new(500, 500),
            &[],
        );
        assert_eq!(result, base);
    }

    #[test]
    fn test_shift_position_if_needed_shifts_when_occupied() {
        let base = LogicalPosition::new(0, 0);
        let result = shift_position_if_needed(
            base,
            LogicalSize::new(50, 50),
            WindowPositionOffset { x: 20, y: 20 },
            LogicalPosition::new(0, 0),
            LogicalSize::new(200, 200),
            &[base],
        );
        assert_eq!(result, LogicalPosition::new(20, 20));
    }

    #[test]
    fn test_shift_position_if_needed_bounces_on_bounds() {
        let base = LogicalPosition::new(50, 50);
        let result = shift_position_if_needed(
            base,
            LogicalSize::new(50, 50),
            WindowPositionOffset { x: 20, y: 20 },
            LogicalPosition::new(0, 0),
            LogicalSize::new(100, 100),
            &[base],
        );
        assert_eq!(result, LogicalPosition::new(30, 30));
    }

    #[test]
    fn test_shift_position_if_needed_with_oversized_window_width() {
        let base = LogicalPosition::new(10, 10);
        let result = shift_position_if_needed(
            base,
            LogicalSize::new(500, 50),
            WindowPositionOffset { x: 20, y: 20 },
            LogicalPosition::new(0, 0),
            LogicalSize::new(100, 100),
            &[base],
        );
        assert_eq!(result, LogicalPosition::new(0, 30));
    }

    #[test]
    fn test_shift_position_if_needed_with_oversized_window() {
        let base = LogicalPosition::new(10, 10);
        let result = shift_position_if_needed(
            base,
            LogicalSize::new(500, 500),
            WindowPositionOffset { x: 20, y: 20 },
            LogicalPosition::new(0, 0),
            LogicalSize::new(100, 100),
            &[base],
        );
        assert_eq!(result, LogicalPosition::new(0, 0));
    }

    #[test]
    fn test_shift_position_if_needed_with_negative_origin() {
        let base = LogicalPosition::new(-240, 20);
        let result = shift_position_if_needed(
            base,
            LogicalSize::new(100, 100),
            WindowPositionOffset { x: 20, y: 20 },
            LogicalPosition::new(-300, -200),
            LogicalSize::new(200, 200),
            &[base],
        );
        assert_eq!(result, LogicalPosition::new(-240, -100));
    }
}
