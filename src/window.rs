use dioxus::desktop::tao::dpi::LogicalPosition;
use dioxus::desktop::tao::window::WindowId;
use dioxus::desktop::{window, Config, WeakDesktopContext, WindowBuilder};
use dioxus::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::assets::MAIN_STYLE;
use crate::components::app::{App, AppProps};
use crate::components::mermaid_window::{generate_diagram_id, MermaidWindow, MermaidWindowProps};
use crate::config::{WindowPositionOffset, CONFIG};
use crate::state::LAST_FOCUSED_STATE;
use crate::theme::{resolve_theme, ThemePreference};

pub mod helpers;
mod types;
pub use helpers::*;

struct ChildWindowEntry {
    handle: WeakDesktopContext,
    window_id: WindowId,
    parent_id: WindowId,
}

impl ChildWindowEntry {
    fn is_alive(&self) -> bool {
        self.handle.upgrade().is_some()
    }

    fn focus(&self) -> bool {
        self.handle.upgrade().is_some_and(|ctx| {
            ctx.window.set_focus();
            true
        })
    }

    fn close(&self) {
        if let Some(ctx) = self.handle.upgrade() {
            ctx.close();
        }
    }

    fn is_window(&self, window_id: WindowId) -> bool {
        self.window_id == window_id
    }

    fn is_child_of(&self, parent_id: WindowId) -> bool {
        self.parent_id == parent_id
    }
}

thread_local! {
    static MAIN_WINDOWS: RefCell<Vec<WeakDesktopContext>> = const { RefCell::new(Vec::new()) };
    static CHILD_WINDOWS: RefCell<HashMap<String, ChildWindowEntry>> = RefCell::new(HashMap::new());
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
                .and_then(|ctx| ctx.window.is_visible().then_some(true))
                .unwrap_or(false)
        })
    })
}

pub fn focus_last_focused_main_window() -> bool {
    if let Some(window_id) = get_last_focused_window() {
        // Resolve to parent window if the last focused was a child window
        let main_window_id = resolve_to_parent_window(window_id);

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
    let theme_value = get_theme_value(is_first_window);

    // Get directory from config and state
    let directory_value = get_directory_value(is_first_window, file.as_ref(), directory);

    // Get sidebar settings from config and state
    let sidebar_value = get_sidebar_value(is_first_window);

    // Get window settings from config and state
    let resolved = get_window_value(is_first_window);
    let position_offset = CONFIG.read().window_position.position_offset;
    let occupied = existing_main_window_bounds();
    let resolved_position =
        shift_position_if_needed(resolved.position, resolved.size, position_offset, &occupied);

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
                .with_inner_size(resolved.size)
                .with_position(resolved_position),
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

pub fn build_custom_index(theme_preference: ThemePreference) -> String {
    let theme = resolve_theme(theme_preference);
    indoc::formatdoc! {r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Arto</title>
            <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
            <!-- CUSTOM HEAD -->
        </head>
        <body data-theme="{theme}">
            <div id="main"></div>
            <!-- MODULE LOADER -->
        </body>
    </html>
    "#}
}

pub fn update_last_focused_window(window_id: WindowId) {
    LAST_FOCUSED_WINDOW.with(|last| *last.borrow_mut() = Some(window_id));
    if let Some(metrics) = find_window_metrics(window_id) {
        let mut last_focused = LAST_FOCUSED_STATE.write();
        last_focused.window_position = metrics.position;
        last_focused.window_size = metrics.size;
    }
}

fn find_window_metrics(window_id: WindowId) -> Option<types::WindowMetrics> {
    MAIN_WINDOWS.with(|windows| {
        windows
            .borrow()
            .iter()
            .filter_map(|w| w.upgrade())
            .find(|ctx| ctx.window.id() == window_id)
            .map(|ctx| capture_window_metrics(&ctx.window))
    })
}

struct WindowBounds {
    position: LogicalPosition<f64>,
    size: dioxus::desktop::tao::dpi::LogicalSize<f64>,
}

fn existing_main_window_bounds() -> Vec<WindowBounds> {
    MAIN_WINDOWS.with(|windows| {
        windows
            .borrow()
            .iter()
            .filter_map(|w| w.upgrade())
            .filter_map(|ctx| {
                let scale = ctx.window.scale_factor();
                let position = ctx
                    .window
                    .outer_position()
                    .ok()
                    .map(|pos| pos.to_logical::<f64>(scale))?;
                let size = ctx.window.outer_size().to_logical::<f64>(scale);
                Some(WindowBounds { position, size })
            })
            .collect()
    })
}

fn bounds_overlap(
    a_pos: LogicalPosition<f64>,
    a_size: dioxus::desktop::tao::dpi::LogicalSize<f64>,
    b_pos: LogicalPosition<f64>,
    b_size: dioxus::desktop::tao::dpi::LogicalSize<f64>,
) -> bool {
    let a_right = a_pos.x + a_size.width;
    let a_bottom = a_pos.y + a_size.height;
    let b_right = b_pos.x + b_size.width;
    let b_bottom = b_pos.y + b_size.height;
    a_pos.x < b_right && a_right > b_pos.x && a_pos.y < b_bottom && a_bottom > b_pos.y
}

fn shift_position_if_needed(
    base: LogicalPosition<f64>,
    window_size: dioxus::desktop::tao::dpi::LogicalSize<f64>,
    offset: WindowPositionOffset,
    occupied: &[WindowBounds],
) -> LogicalPosition<f64> {
    if offset.x == 0 && offset.y == 0 {
        return base;
    }
    let mut position = base;
    for _ in 0..20 {
        if !occupied
            .iter()
            .any(|existing| bounds_overlap(position, window_size, existing.position, existing.size))
        {
            break;
        }
        let shifted = offset.apply(LogicalPosition::new(
            position.x.round() as i32,
            position.y.round() as i32,
        ));
        position = LogicalPosition::new(shifted.x as f64, shifted.y as f64);
    }
    position
}

fn get_last_focused_window() -> Option<WindowId> {
    LAST_FOCUSED_WINDOW.with(|last| *last.borrow())
}

fn resolve_to_parent_window(window_id: WindowId) -> WindowId {
    CHILD_WINDOWS.with(|windows| {
        windows
            .borrow()
            .values()
            .find(|e| e.is_window(window_id))
            .map(|e| e.parent_id)
            .unwrap_or(window_id)
    })
}

pub fn close_child_windows_for_parent(parent_id: WindowId) {
    CHILD_WINDOWS.with(|windows| {
        windows.borrow_mut().retain(|_, e| {
            if e.is_child_of(parent_id) {
                e.close();
                false
            } else {
                e.is_alive()
            }
        });
    });
}

pub fn close_child_windows_for_last_focused() {
    if let Some(a) = get_last_focused_window().map(resolve_to_parent_window) {
        close_child_windows_for_parent(a)
    }
}

pub fn open_or_focus_mermaid_window(source: String, theme: ThemePreference) {
    let diagram_id = generate_diagram_id(&source);
    let parent_id = window().id();

    // Check if window already exists and can be focused
    let needs_creation = CHILD_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        windows.retain(|_, e| e.is_alive());

        !windows.get(&diagram_id).is_some_and(|e| e.focus())
    });

    if needs_creation {
        dioxus_core::spawn(create_and_register_mermaid_window(
            source, diagram_id, theme, parent_id,
        ));
    }
}

async fn create_and_register_mermaid_window(
    source: String,
    diagram_id: String,
    theme: ThemePreference,
    parent_id: WindowId,
) {
    let dom = VirtualDom::new_with_props(
        MermaidWindow,
        MermaidWindowProps {
            source,
            diagram_id: diagram_id.clone(),
            theme,
        },
    );

    let config = Config::new()
        .with_menu(None)
        .with_window(WindowBuilder::new().with_title("Mermaid Viewer"))
        .with_custom_head(indoc::formatdoc! {r#"<link rel="stylesheet" href="{MAIN_STYLE}">"#})
        .with_custom_index(build_mermaid_window_index(theme));

    let pending = window().new_window(dom, config);
    let ctx = pending.await;
    let weak_handle = std::rc::Rc::downgrade(&ctx);
    let window_id = ctx.window.id();

    CHILD_WINDOWS.with(|windows| {
        windows.borrow_mut().insert(
            diagram_id,
            ChildWindowEntry {
                handle: weak_handle,
                window_id,
                parent_id,
            },
        );
    });
}

fn build_mermaid_window_index(theme: ThemePreference) -> String {
    let resolved_theme = resolve_theme(theme);
    indoc::formatdoc! {r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Mermaid Viewer - Arto</title>
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <!-- CUSTOM HEAD -->
        </head>
        <body data-theme="{resolved_theme}" class="mermaid-window-body">
            <div id="main"></div>
            <!-- MODULE LOADER -->
        </body>
    </html>
    "#}
}
