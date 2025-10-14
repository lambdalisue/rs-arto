use dioxus::prelude::{Readable, Writable};
use dioxus_desktop::muda::accelerator::{Accelerator, Code, Modifiers};
use dioxus_desktop::muda::{
    AboutMetadataBuilder, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use dioxus_desktop::window;
use std::path::PathBuf;

use crate::state::AppState;
use crate::window;

/// Menu identifier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuId {
    NewWindow,
    NewTab,
    Open,
    OpenDirectory,
    CloseTab,
    CloseAllTabs,
    CloseWindow,
    CloseAllChildWindows,
    CloseAllWindows,
    ToggleSidebar,
    ActualSize,
    ZoomIn,
    ZoomOut,
    GoBack,
    GoForward,
    GoToHomepage,
}

impl MenuId {
    /// Convert menu ID string to enum variant
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "file.new_window" => Some(Self::NewWindow),
            "file.new_tab" => Some(Self::NewTab),
            "file.open" => Some(Self::Open),
            "file.open_directory" => Some(Self::OpenDirectory),
            "file.close_tab" => Some(Self::CloseTab),
            "file.close_all_tabs" => Some(Self::CloseAllTabs),
            "file.close_window" => Some(Self::CloseWindow),
            "window.close_all_child_windows" => Some(Self::CloseAllChildWindows),
            "window.close_all_windows" => Some(Self::CloseAllWindows),
            "view.toggle_sidebar" => Some(Self::ToggleSidebar),
            "view.actual_size" => Some(Self::ActualSize),
            "view.zoom_in" => Some(Self::ZoomIn),
            "view.zoom_out" => Some(Self::ZoomOut),
            "history.back" => Some(Self::GoBack),
            "history.forward" => Some(Self::GoForward),
            "help.homepage" => Some(Self::GoToHomepage),
            _ => None,
        }
    }

    /// Get the string ID for this menu item
    fn as_str(self) -> &'static str {
        match self {
            Self::NewWindow => "file.new_window",
            Self::NewTab => "file.new_tab",
            Self::Open => "file.open",
            Self::OpenDirectory => "file.open_directory",
            Self::CloseTab => "file.close_tab",
            Self::CloseAllTabs => "file.close_all_tabs",
            Self::CloseWindow => "file.close_window",
            Self::CloseAllChildWindows => "window.close_all_child_windows",
            Self::CloseAllWindows => "window.close_all_windows",
            Self::ToggleSidebar => "view.toggle_sidebar",
            Self::ActualSize => "view.actual_size",
            Self::ZoomIn => "view.zoom_in",
            Self::ZoomOut => "view.zoom_out",
            Self::GoBack => "history.back",
            Self::GoForward => "history.forward",
            Self::GoToHomepage => "help.homepage",
        }
    }
}

/// Helper to create a menu item with optional keyboard shortcut
fn create_menu_item(
    id: MenuId,
    label: &str,
    code: Option<Code>,
    additional_modifiers: Option<Modifiers>,
) -> MenuItem {
    let accelerator = code.map(|c| get_cmd_or_ctrl(c, additional_modifiers));
    MenuItem::with_id(id.as_str(), label, true, accelerator)
}

/// Build the application menu bar
pub fn build_menu() -> Menu {
    #[cfg(target_os = "macos")]
    disable_automatic_window_tabbing();

    let menu = Menu::new();

    #[cfg(target_os = "macos")]
    add_app_menu(&menu);

    add_file_menu(&menu);
    add_view_menu(&menu);
    add_history_menu(&menu);
    add_window_menu(&menu);
    add_help_menu(&menu);

    menu
}

#[cfg(target_os = "macos")]
fn add_app_menu(menu: &Menu) {
    let arto_menu = Submenu::new("Arto", true);
    let about_metadata = AboutMetadataBuilder::new()
        .name(Some("Arto".to_string()))
        .version(Some(env!("CARGO_PKG_VERSION")))
        .authors(Some(
            vec!["lambdalisue <lambdalisue@gmail.com>".to_string()],
        ))
        .website(Some("https://github.com/lambdalisue/rs-arto".to_string()))
        .website_label(Some("GitHub".to_string()))
        .copyright(Some("Copyright 2025 lambdalisue".to_string()))
        .build();

    arto_menu
        .append_items(&[
            &PredefinedMenuItem::about(Some("Arto"), Some(about_metadata)),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(Some("Quit")),
        ])
        .unwrap();

    menu.append(&arto_menu).unwrap();
}

fn add_file_menu(menu: &Menu) {
    let file_menu = Submenu::new("File", true);

    file_menu
        .append_items(&[
            &create_menu_item(MenuId::NewWindow, "New Window", Some(Code::KeyN), None),
            &create_menu_item(MenuId::NewTab, "New Tab", Some(Code::KeyT), None),
            &PredefinedMenuItem::separator(),
            &create_menu_item(MenuId::Open, "Open File...", Some(Code::KeyO), None),
            &create_menu_item(
                MenuId::OpenDirectory,
                "Open Directory...",
                Some(Code::KeyO),
                Some(Modifiers::SHIFT),
            ),
            &PredefinedMenuItem::separator(),
            &create_menu_item(MenuId::CloseTab, "Close Tab", Some(Code::KeyW), None),
            &create_menu_item(MenuId::CloseAllTabs, "Close All Tabs", None, None),
            &create_menu_item(
                MenuId::CloseWindow,
                "Close Window",
                Some(Code::KeyW),
                Some(Modifiers::SHIFT),
            ),
        ])
        .unwrap();

    menu.append(&file_menu).unwrap();
}

fn add_view_menu(menu: &Menu) {
    let view_menu = Submenu::new("View", true);

    view_menu
        .append_items(&[
            &create_menu_item(
                MenuId::ToggleSidebar,
                "Toggle Sidebar",
                Some(Code::KeyB),
                None,
            ),
            &PredefinedMenuItem::separator(),
            &create_menu_item(MenuId::ActualSize, "Actual Size", Some(Code::Digit0), None),
            &create_menu_item(MenuId::ZoomIn, "Zoom In", Some(Code::Equal), None),
            &create_menu_item(MenuId::ZoomOut, "Zoom Out", Some(Code::Minus), None),
        ])
        .unwrap();

    menu.append(&view_menu).unwrap();
}

fn add_history_menu(menu: &Menu) {
    let history_menu = Submenu::new("History", true);

    history_menu
        .append_items(&[
            &create_menu_item(MenuId::GoBack, "Go Back", Some(Code::BracketLeft), None),
            &create_menu_item(
                MenuId::GoForward,
                "Go Forward",
                Some(Code::BracketRight),
                None,
            ),
        ])
        .unwrap();

    menu.append(&history_menu).unwrap();
}

fn add_window_menu(menu: &Menu) {
    let window_menu = Submenu::new("Window", true);

    window_menu
        .append_items(&[
            &create_menu_item(
                MenuId::CloseAllChildWindows,
                "Close All Child Windows",
                None,
                None,
            ),
            &create_menu_item(MenuId::CloseAllWindows, "Close All Windows", None, None),
        ])
        .unwrap();

    menu.append(&window_menu).unwrap();
}

fn add_help_menu(menu: &Menu) {
    let help_menu = Submenu::new("Help", true);

    help_menu
        .append(&create_menu_item(
            MenuId::GoToHomepage,
            "Go to Homepage",
            None,
            None,
        ))
        .unwrap();

    menu.append(&help_menu).unwrap();
}

/// Get Cmd (macOS) or Ctrl (others) modifier with optional additional modifiers
fn get_cmd_or_ctrl(code: Code, additional: Option<Modifiers>) -> Accelerator {
    #[cfg(target_os = "macos")]
    let base_modifier = Modifiers::SUPER;
    #[cfg(not(target_os = "macos"))]
    let base_modifier = Modifiers::CONTROL;

    let modifiers = if let Some(additional_mods) = additional {
        base_modifier | additional_mods
    } else {
        base_modifier
    };

    Accelerator::new(Some(modifiers), code)
}

/// Handle menu events that don't require app state
pub fn handle_menu_event_global(event: &MenuEvent) -> bool {
    let menu_id = event.id().0.as_ref();
    tracing::info!("Global menu event: {}", menu_id);

    let id = match MenuId::from_str(menu_id) {
        Some(id) => id,
        None => return false,
    };

    match id {
        MenuId::NewWindow => {
            window::create_new_main_window(None, false);
        }
        MenuId::NewTab => {
            if !window::has_any_main_windows() {
                window::create_new_main_window(None, false);
                return true;
            }
            return false;
        }
        MenuId::CloseAllChildWindows => {
            window::close_child_windows_for_last_focused();
        }
        MenuId::CloseAllWindows => {
            window::close_all_main_windows();
        }
        MenuId::GoToHomepage => {
            let _ = open::that("https://github.com/lambdalisue/rs-arto");
        }
        _ => return false,
    }

    true
}

/// Handle menu events that require app state (must be called from component context)
/// Only handles the event if the current window is focused
pub fn handle_menu_event_with_state(event: &MenuEvent, state: &mut AppState) -> bool {
    // Check if current window is focused
    if !window().is_focused() {
        return false;
    }

    let menu_id = event.id().0.as_ref();
    tracing::debug!("State menu event (focused window): {}", menu_id);

    let id = match MenuId::from_str(menu_id) {
        Some(id) => id,
        None => return false,
    };

    match id {
        MenuId::NewTab => {
            state.add_tab(None, true);
        }
        MenuId::Open => {
            if let Some(file) = pick_markdown_file() {
                state.open_file(file);
            }
        }
        MenuId::OpenDirectory => {
            if let Some(dir) = pick_directory() {
                state.set_root_directory(dir);
            }
        }
        MenuId::CloseTab => {
            let active_tab = *state.active_tab.read();
            state.close_tab(active_tab);
        }
        MenuId::CloseAllTabs => {
            // Close all tabs except one, then clear it
            let mut tabs = state.tabs.write();
            tabs.clear();
            tabs.push(crate::state::Tab::new(None));
            state.active_tab.set(0);
        }
        MenuId::CloseWindow => {
            window().close();
        }
        MenuId::ToggleSidebar => {
            state.toggle_sidebar();
        }
        MenuId::ActualSize => {
            state.zoom_level.set(1.0);
        }
        MenuId::ZoomIn => {
            let current = *state.zoom_level.read();
            // Max zoom: 10.0
            state.zoom_level.set((current + 0.1).min(10.0));
        }
        MenuId::ZoomOut => {
            let current = *state.zoom_level.read();
            // Min zoom: 0.1
            state.zoom_level.set((current - 0.1).max(0.1));
        }
        MenuId::GoBack => {
            state.update_current_tab(|tab| {
                if let Some(path) = tab.history.go_back() {
                    tab.content = crate::state::TabContent::File(path);
                }
            });
        }
        MenuId::GoForward => {
            state.update_current_tab(|tab| {
                if let Some(path) = tab.history.go_forward() {
                    tab.content = crate::state::TabContent::File(path);
                }
            });
        }
        _ => return false,
    }

    true
}

/// Show file picker dialog and return selected file
fn pick_markdown_file() -> Option<PathBuf> {
    use rfd::FileDialog;

    tracing::debug!("Opening file picker dialog...");
    let start = std::time::Instant::now();

    let file = FileDialog::new()
        .add_filter("Markdown", &["md", "markdown"])
        .set_directory(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
        .pick_file();

    tracing::debug!("File picker completed in {:?}", start.elapsed());

    file
}

/// Show directory picker dialog and return selected directory
fn pick_directory() -> Option<PathBuf> {
    use rfd::FileDialog;

    tracing::debug!("Opening directory picker dialog...");
    let start = std::time::Instant::now();

    let dir = FileDialog::new()
        .set_directory(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
        .pick_folder();

    tracing::debug!("Directory picker completed in {:?}", start.elapsed());

    dir
}

#[cfg(target_os = "macos")]
fn disable_automatic_window_tabbing() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSWindow;
    let marker = MainThreadMarker::new().expect("Failed to get main thread marker");
    NSWindow::setAllowsAutomaticWindowTabbing(false, marker);
}
