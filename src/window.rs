use dioxus::prelude::*;
use dioxus_desktop::tao::window::WindowId;
use dioxus_desktop::{window, Config, WeakDesktopContext, WindowBuilder};

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::assets::MAIN_STYLE;
use crate::components::app::{App, AppProps};
use crate::state::LAST_SELECTED_THEME;
use crate::theme::resolve_theme;

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

fn register_main_window(handle: WeakDesktopContext) {
    MAIN_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        windows.retain(|w| w.upgrade().is_some());
        if !windows.iter().any(|w| w.ptr_eq(&handle)) {
            windows.push(handle);
        }
    });
}

pub fn has_any_main_windows() -> bool {
    MAIN_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        windows.retain(|w| w.upgrade().is_some());
        !windows.is_empty()
    })
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

pub fn create_new_main_window(file: Option<PathBuf>, show_welcome: bool) {
    // This cause ERROR but it seems we can ignore it safely
    // https://github.com/DioxusLabs/dioxus/issues/3872
    let dom = VirtualDom::new_with_props(App, AppProps { file, show_welcome });
    // Set None for child window menu to avoid panic when closing windows.
    // The menu from the main window will be used instead.
    let config = Config::new()
        .with_menu(None) // To avoid child window taking over the main window's menu
        .with_window(WindowBuilder::new().with_title("Arto"))
        // Add main style in config. Otherwise the style takes time to load and
        // the window appears unstyled for a brief moment.
        .with_custom_head(indoc::formatdoc! {r#"<link rel="stylesheet" href="{MAIN_STYLE}">"#})
        // Use a custom index to set the initial theme correctly
        .with_custom_index(build_custom_index());

    let handle = window().new_window(dom, config);
    register_main_window(handle);
}

fn build_custom_index() -> String {
    let theme = resolve_theme(&LAST_SELECTED_THEME.lock().unwrap());
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
