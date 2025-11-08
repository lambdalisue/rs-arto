use dioxus::html::HasFileData;
use dioxus::prelude::*;
use dioxus_core::use_drop;
use dioxus_desktop::{use_muda_event_handler, window};
use std::path::PathBuf;

use super::content::Content;
use super::header::Header;
use super::icon::{Icon, IconName};
use super::sidebar::Sidebar;
use super::tab_bar::TabBar;
use crate::menu;
use crate::state::{AppState, Tab, DIRECTORY_OPEN_BROADCAST, FILE_OPEN_BROADCAST};

#[component]
pub fn App(file: Option<PathBuf>, show_welcome: bool) -> Element {
    // Initialize application state with optional initial file or welcome screen
    let state = use_context_provider(|| {
        let mut app_state = AppState::default();
        if let Some(path) = file {
            // Set sidebar root directory to file's parent directory on window initialization
            if let Some(parent) = path.parent() {
                app_state.sidebar.write().root_directory = Some(parent.to_path_buf());
            }
            app_state.tabs.write()[0] = Tab::new(Some(path));
        } else if show_welcome {
            // Show welcome screen with embedded markdown content
            let welcome_content = crate::assets::get_default_markdown_content();
            app_state.tabs.write()[0] = Tab::with_inline_content(welcome_content);
        }
        app_state
    });

    // Track drag-and-drop hover state
    let mut is_dragging = use_signal(|| false);

    // Clone state for event handlers
    let mut state_for_menu = state.clone();
    let state_for_drop = state.clone();

    // Handle menu events (only state-dependent events, not global ones)
    use_muda_event_handler(move |event| {
        // Only handle state-dependent events
        menu::handle_menu_event_with_state(event, &mut state_for_menu);
    });

    // Listen for file open broadcasts from background process
    setup_file_open_listener(state.clone());

    // Listen for directory open broadcasts from background process
    setup_directory_open_listener(state.clone());

    // Close child windows when this window closes
    use_drop(move || {
        crate::window::close_child_windows_for_parent(window().id());
    });

    rsx! {
        div {
            class: "app-container",
            class: if is_dragging() { "drag-over" },
            ondragover: move |evt| {
                // Only accept file/directory drops
                let files = evt.files();
                if !files.is_empty() {
                    evt.prevent_default();
                    is_dragging.set(true);
                    return;
                }
                is_dragging.set(false);
            },
            ondragleave: move |_| {
                is_dragging.set(false);
            },
            ondrop: move |evt| {
                evt.prevent_default();
                is_dragging.set(false);

                let state = state_for_drop.clone();
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
        let path = PathBuf::from(&file_data.name());

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
            if !state.sidebar.read().is_visible {
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
fn setup_file_open_listener(state: AppState) {
    use_effect(move || {
        let mut state_clone = state.clone();
        let mut rx = FILE_OPEN_BROADCAST.subscribe();

        spawn(async move {
            while let Ok(file) = rx.recv().await {
                // Only handle in the focused window
                if window().is_focused() {
                    tracing::info!("Opening file from broadcast: {:?}", file);
                    state_clone.open_file(file);
                }
            }
        });
    });
}

/// Setup listener for directory open broadcasts from the background process
fn setup_directory_open_listener(state: AppState) {
    use_effect(move || {
        let mut state_clone = state.clone();
        let mut rx = DIRECTORY_OPEN_BROADCAST.subscribe();

        spawn(async move {
            while let Ok(dir) = rx.recv().await {
                // Only handle in the focused window
                if window().is_focused() {
                    tracing::info!("Opening directory from broadcast: {:?}", dir);
                    state_clone.set_root_directory(dir.clone());
                    // Optionally show the sidebar if it's hidden
                    if !state_clone.sidebar.read().is_visible {
                        state_clone.toggle_sidebar();
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
                    "Drop markdown file to open"
                }
            }
        }
    }
}
