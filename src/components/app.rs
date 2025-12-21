use dioxus::desktop::tao::dpi::{LogicalPosition, LogicalSize};
use dioxus::desktop::tao::event::{Event as TaoEvent, WindowEvent};
use dioxus::desktop::{use_muda_event_handler, use_wry_event_handler, window};
use dioxus::document;
use dioxus::html::HasFileData;
use dioxus::prelude::*;
use dioxus_core::use_drop;
use std::path::PathBuf;

use super::content::Content;
use super::header::Header;
use super::icon::{Icon, IconName};
use super::sidebar::Sidebar;
use super::tab_bar::TabBar;
use crate::assets::MAIN_SCRIPT;
use crate::events::{DIRECTORY_OPEN_BROADCAST, FILE_OPEN_BROADCAST};
use crate::menu;
use crate::state::{AppState, PersistedState, Tab, LAST_FOCUSED_STATE};

#[component]
pub fn App(
    file: Option<PathBuf>,
    directory: PathBuf,
    sidebar_open: bool,
    sidebar_width: f64,
    sidebar_show_all_files: bool,
    show_welcome: bool,
) -> Element {
    // Initialize application state with optional initial file or welcome screen
    let mut state = use_context_provider(|| {
        let mut app_state = AppState::default();
        if let Some(path) = file.clone() {
            app_state.tabs.write()[0] = Tab::new(path);
        } else if show_welcome {
            // Show welcome screen with embedded markdown content
            let welcome_content = crate::assets::get_default_markdown_content();
            app_state.tabs.write()[0] = Tab::with_inline_content(welcome_content);
        }
        // Apply initial directory from config (for startup/new window behavior)
        *app_state.directory.write() = Some(directory.clone());
        // Update last focused state for "Last Focused" behavior
        LAST_FOCUSED_STATE.write().directory = Some(directory);
        // Apply initial sidebar settings from config
        {
            let mut sidebar = app_state.sidebar.write();
            sidebar.open = sidebar_open;
            sidebar.width = sidebar_width;
            sidebar.show_all_files = sidebar_show_all_files;
            // Update last focused state for "Last Focused" behavior
            let mut state = LAST_FOCUSED_STATE.write();
            state.sidebar_open = sidebar_open;
            state.sidebar_width = sidebar_width;
            state.sidebar_show_all_files = sidebar_show_all_files;
        }
        let scale = window().scale_factor();
        *app_state.position.write() = window()
            .outer_position()
            .expect("failed to get outer position")
            .to_logical(scale);
        *app_state.size.write() = window().outer_size().to_logical(scale);
        app_state
    });
    // Track drag-and-drop hover state
    let mut is_dragging = use_signal(|| false);

    // Initialize JavaScript main module (theme listeners, etc.)
    use_hook(|| {
        spawn(async move {
            let _ = document::eval(&format!(
                r#"
                const {{ init }} = await import("{MAIN_SCRIPT}");
                init();
                "#
            ))
            .await;
        });
    });

    // Handle menu events (only state-dependent events, not global ones)
    use_muda_event_handler(move |event| {
        // Only handle state-dependent events
        menu::handle_menu_event_with_state(event, &mut state);
    });

    let sync_window_metrics =
        |position: Option<LogicalPosition<i32>>, size: Option<LogicalSize<u32>>| {
            if let Some(position) = position {
                *state.position.write() = position;
            }
            if let Some(size) = size {
                *state.size.write() = size;
            }
            if position.is_some() || size.is_some() {
                let mut last_focused = LAST_FOCUSED_STATE.write();
                if let Some(position) = position {
                    last_focused.window_position = position.into();
                }
                if let Some(size) = size {
                    last_focused.window_size = size.into();
                }
            }
        };

    // Handle window events
    use_wry_event_handler(move |event, _| match event {
        TaoEvent::WindowEvent {
            event: WindowEvent::Resized(size),
            window_id,
            ..
        } => {
            let window = window();
            if window_id == &window.id() {
                sync_window_metrics(
                    None,
                    Some(size.to_logical::<u32>(window.scale_factor())),
                );
            }
        }
        TaoEvent::WindowEvent {
            event: WindowEvent::Moved(position),
            window_id,
            ..
        } => {
            let window = window();
            if window_id == &window.id() {
                sync_window_metrics(
                    Some(position.to_logical::<i32>(window.scale_factor())),
                    None,
                );
            }
        }
        TaoEvent::WindowEvent {
            event: WindowEvent::Focused(true),
            window_id,
            ..
        } => {
            let window = window();
            if window_id == &window.id() {
                let scale = window.scale_factor();
                let position = window
                    .outer_position()
                    .ok()
                    .map(|pos| pos.to_logical::<i32>(scale));
                let size = Some(window.outer_size().to_logical::<u32>(scale));
                sync_window_metrics(position, size);
            }
        }
        _ => {}
    });

    // Listen for file open broadcasts from background process
    setup_file_open_listener(state);

    // Listen for directory open broadcasts from background process
    setup_directory_open_listener(state);

    // Save state and close child windows when this window closes
    use_drop(move || {
        // Save last used state from this window
        // Read directly from state signals instead of global statics
        // to ensure we get the current window's values, not the global last-modified values
        let mut persisted = PersistedState::from(&state);
        let window_metrics = crate::window::helpers::capture_window_metrics(&window().window);
        persisted.window_position = window_metrics.position;
        persisted.window_size = window_metrics.size;
        {
            let mut last_focused = LAST_FOCUSED_STATE.write();
            last_focused.window_position = window_metrics.position;
            last_focused.window_size = window_metrics.size;
        }
        persisted.save();

        // Close child windows
        crate::window::close_child_windows_for_parent(window().id());
    });

    rsx! {
        div {
            class: "app-container",
            class: if is_dragging() { "drag-over" },
            ondragover: move |evt| {
                evt.prevent_default();
                is_dragging.set(true);
            },
            ondragleave: move |evt| {
                evt.prevent_default();
                is_dragging.set(false);
            },
            ondrop: move |evt| {
                evt.prevent_default();
                is_dragging.set(false);

                spawn(async move {
                    handle_dropped_files(evt, state).await;
                });
            },

            Sidebar {},

            div {
                class: "main-area",
                Header {},
                TabBar {},
                Content {},
            }

            // Drag and drop overlay
            if is_dragging() {
                DragDropOverlay {}
            }
        }
    }
}

/// Handle dropped files/directories - opens markdown files or sets directory as root
async fn handle_dropped_files(evt: Event<DragData>, mut state: AppState) {
    let files = evt.files();
    if files.is_empty() {
        return;
    }

    for file_data in files {
        let path = file_data.path();

        // Resolve symlinks and canonicalize the path to handle Finder sidebar items
        let resolved_path = match std::fs::canonicalize(&path) {
            Ok(p) => {
                tracing::info!("Resolved path: {:?} -> {:?}", path, p);
                p
            }
            Err(e) => {
                tracing::warn!("Failed to canonicalize path {:?}: {}", path, e);
                path.clone()
            }
        };

        tracing::info!(
            "Processing dropped path: {:?}, is_dir: {}",
            resolved_path,
            resolved_path.is_dir()
        );

        if resolved_path.is_dir() {
            // If it's a directory, set it as root and show the sidebar
            tracing::info!("Setting dropped directory as root: {:?}", resolved_path);
            state.set_root_directory(resolved_path);
            // Show the sidebar if it's hidden so users can see the directory tree
            if !state.sidebar.read().open {
                state.toggle_sidebar();
            }
        } else {
            // Open any file (not just markdown)
            tracing::info!("Opening dropped file: {:?}", resolved_path);
            state.open_file(resolved_path);
        }
    }
}

/// Setup listener for file open broadcasts from the background process
fn setup_file_open_listener(mut state: AppState) {
    use_effect(move || {
        let mut rx = FILE_OPEN_BROADCAST.subscribe();

        spawn(async move {
            while let Ok(file) = rx.recv().await {
                // Only handle in the focused window
                if window().is_focused() {
                    tracing::info!("Opening file from broadcast: {:?}", file);
                    state.open_file(file);
                }
            }
        });
    });
}

/// Setup listener for directory open broadcasts from the background process
fn setup_directory_open_listener(mut state: AppState) {
    use_effect(move || {
        let mut rx = DIRECTORY_OPEN_BROADCAST.subscribe();

        spawn(async move {
            while let Ok(dir) = rx.recv().await {
                // Only handle in the focused window
                if window().is_focused() {
                    tracing::info!("Opening directory from broadcast: {:?}", dir);
                    state.set_root_directory(dir.clone());
                    // Optionally show the sidebar if it's hidden
                    if !state.sidebar.read().open {
                        state.toggle_sidebar();
                    }
                }
            }
        });
    });
}

#[component]
fn DragDropOverlay() -> Element {
    rsx! {
        div {
            class: "drag-drop-overlay",
            div {
                class: "drag-drop-content",
                div {
                    class: "drag-drop-icon",
                    Icon { name: IconName::FileUpload, size: 64 }
                }
                div {
                    class: "drag-drop-text",
                    "Drop Markdown file or directory to open"
                }
            }
        }
    }
}
