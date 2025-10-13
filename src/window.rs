use dioxus::prelude::*;
use dioxus_desktop::{window, Config, WeakDesktopContext, WindowBuilder};

use std::cell::RefCell;
use std::path::PathBuf;

use crate::assets::MAIN_STYLE;
use crate::components::app::{App, AppProps};
use crate::state::LAST_SELECTED_THEME;
use crate::theme::resolve_theme;

thread_local! {
    /// Registry of all child windows (not including the background window)
    static CHILD_WINDOWS: RefCell<Vec<WeakDesktopContext>> = const { RefCell::new(Vec::new()) };
}

/// Register a child window
fn register_child_window(handle: WeakDesktopContext) {
    CHILD_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        // Clean up dead references
        windows.retain(|w| w.upgrade().is_some());
        // Add new window if not already present
        if !windows.iter().any(|w| w.ptr_eq(&handle)) {
            windows.push(handle);
        }
    });
}

/// Close all child windows
pub fn close_all_child_windows() {
    CHILD_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        // Collect all live windows
        let live_windows: Vec<_> = windows.iter().filter_map(|w| w.upgrade()).collect();

        // Close all windows
        for win in live_windows {
            win.close();
        }

        // Clear the registry
        windows.clear();
    });
}

/// Create a new window
pub fn create_new_window(file: Option<PathBuf>) {
    // This cause ERROR but it seems we can ignore it safely
    // https://github.com/DioxusLabs/dioxus/issues/3872
    let dom = VirtualDom::new_with_props(App, AppProps { file });
    // Set None for child window menu to avoid panic when closing windows.
    // The menu from the main window will be used instead.
    let config = Config::new()
        .with_menu(None) // To avoid child window taking over the main window's menu
        .with_window(WindowBuilder::new().with_title("Octoscope"))
        // Add main style in config. Otherwise the style takes time to load and
        // the window appears unstyled for a brief moment.
        .with_custom_head(indoc::formatdoc! {r#"<link rel="stylesheet" href="{MAIN_STYLE}">"#})
        // Use a custom index to set the initial theme correctly
        .with_custom_index(build_custom_index());

    let handle = window().new_window(dom, config);
    register_child_window(handle);
}

fn build_custom_index() -> String {
    let theme = resolve_theme(&LAST_SELECTED_THEME.lock().unwrap());
    indoc::formatdoc! {r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Octoscope</title>
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
