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
    Open,
    OpenInCurrentWindow,
    CloseWindow,
    CloseAllWindows,
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
            "file.open" => Some(Self::Open),
            "file.open_in_current_window" => Some(Self::OpenInCurrentWindow),
            "file.close_window" => Some(Self::CloseWindow),
            "file.close_all" => Some(Self::CloseAllWindows),
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
            Self::Open => "file.open",
            Self::OpenInCurrentWindow => "file.open_in_current_window",
            Self::CloseWindow => "file.close_window",
            Self::CloseAllWindows => "file.close_all",
            Self::ActualSize => "view.actual_size",
            Self::ZoomIn => "view.zoom_in",
            Self::ZoomOut => "view.zoom_out",
            Self::GoBack => "history.back",
            Self::GoForward => "history.forward",
            Self::GoToHomepage => "help.homepage",
        }
    }
}

/// Build the application menu bar
pub fn build_menu() -> Menu {
    #[cfg(target_os = "macos")]
    disable_automatic_window_tabbing();

    let menu = Menu::new();

    // macOS: Add Arto menu (app menu)
    #[cfg(target_os = "macos")]
    {
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

    // File menu
    let file_menu = Submenu::new("File", true);

    let new_window = MenuItem::with_id(
        MenuId::NewWindow.as_str(),
        "New Window",
        true,
        Some(get_cmd_or_ctrl(Code::KeyN, None)),
    );

    let open = MenuItem::with_id(
        MenuId::Open.as_str(),
        "Open...",
        true,
        Some(get_cmd_or_ctrl(Code::KeyO, None)),
    );

    let open_in_current_window = MenuItem::with_id(
        MenuId::OpenInCurrentWindow.as_str(),
        "Open in Current Window",
        true,
        Some(get_cmd_or_ctrl(Code::KeyO, Some(Modifiers::SHIFT))),
    );

    let close_window = MenuItem::with_id(
        MenuId::CloseWindow.as_str(),
        "Close Window",
        true,
        Some(get_cmd_or_ctrl(Code::KeyW, None)),
    );

    file_menu
        .append_items(&[
            &new_window,
            &open,
            &open_in_current_window,
            &PredefinedMenuItem::separator(),
            &close_window,
        ])
        .unwrap();

    #[cfg(target_os = "macos")]
    {
        let close_all_windows = MenuItem::with_id(
            MenuId::CloseAllWindows.as_str(),
            "Close All Windows",
            true,
            Some(get_cmd_or_ctrl(Code::KeyW, Some(Modifiers::ALT))),
        );
        file_menu.append(&close_all_windows).unwrap();
    }

    menu.append(&file_menu).unwrap();

    // View menu
    let view_menu = Submenu::new("View", true);

    let actual_size = MenuItem::with_id(
        MenuId::ActualSize.as_str(),
        "Actual Size",
        true,
        Some(get_cmd_or_ctrl(Code::Digit0, None)),
    );

    let zoom_in = MenuItem::with_id(
        MenuId::ZoomIn.as_str(),
        "Zoom In",
        true,
        Some(get_cmd_or_ctrl(Code::Equal, None)),
    );

    let zoom_out = MenuItem::with_id(
        MenuId::ZoomOut.as_str(),
        "Zoom Out",
        true,
        Some(get_cmd_or_ctrl(Code::Minus, None)),
    );

    view_menu
        .append_items(&[&actual_size, &zoom_in, &zoom_out])
        .unwrap();

    menu.append(&view_menu).unwrap();

    // History menu
    let history_menu = Submenu::new("History", true);

    let go_back = MenuItem::with_id(
        MenuId::GoBack.as_str(),
        "Go Back",
        true,
        Some(get_cmd_or_ctrl(Code::BracketLeft, None)),
    );

    let go_forward = MenuItem::with_id(
        MenuId::GoForward.as_str(),
        "Go Forward",
        true,
        Some(get_cmd_or_ctrl(Code::BracketRight, None)),
    );

    history_menu.append_items(&[&go_back, &go_forward]).unwrap();

    menu.append(&history_menu).unwrap();

    // Help menu
    let help_menu = Submenu::new("Help", true);

    let go_to_homepage =
        MenuItem::with_id(MenuId::GoToHomepage.as_str(), "Go to Homepage", true, None);

    help_menu.append(&go_to_homepage).unwrap();

    menu.append(&help_menu).unwrap();

    menu
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
            window::create_new_window(None);
        }
        MenuId::Open => {
            tracing::info!("Opening file picker for new window...");
            if let Some(file) = pick_markdown_file() {
                tracing::info!("File selected: {:?}", file);
                window::create_new_window(Some(file));
            }
        }
        MenuId::CloseAllWindows => {
            tracing::info!("Closing all child windows");
            window::close_all_child_windows();
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
        MenuId::CloseWindow => {
            window().close();
        }
        MenuId::OpenInCurrentWindow => {
            if let Some(file) = pick_markdown_file() {
                state.history.write().push(file.clone());
                state.file.set(Some(file));
            }
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
            if let Some(path) = state.history.write().go_back() {
                state.file.set(Some(path));
            }
        }
        MenuId::GoForward => {
            if let Some(path) = state.history.write().go_forward() {
                state.file.set(Some(path));
            }
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

#[cfg(target_os = "macos")]
fn disable_automatic_window_tabbing() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSWindow;
    let marker = MainThreadMarker::new().expect("Failed to get main thread marker");
    NSWindow::setAllowsAutomaticWindowTabbing(false, marker);
}
