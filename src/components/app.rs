use dioxus::html::HasFileData;
use dioxus::prelude::*;
use dioxus_desktop::{use_muda_event_handler, window};
use std::path::PathBuf;

use super::content::Content;
use super::header::Header;
use super::icon::{Icon, IconName};
use super::tab_bar::TabBar;
use crate::menu;
use crate::state::{AppState, Tab, FILE_OPEN_BROADCAST};
use crate::utils::file::is_markdown_file;

#[component]
pub fn App(file: Option<PathBuf>, show_welcome: bool) -> Element {
    // Initialize application state with optional initial file or welcome screen
    let state = use_context_provider(|| {
        let mut app_state = AppState::default();
        if let Some(path) = file {
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

    rsx! {
        div {
            class: "app-container",
            class: if is_dragging() { "drag-over" },
            ondragover: move |evt| {
                evt.prevent_default();
                is_dragging.set(true);
            },
            ondragleave: move |_| {
                is_dragging.set(false);
            },
            ondrop: move |evt| {
                let state = state_for_drop.clone();
                async move {
                    evt.prevent_default();
                    is_dragging.set(false);

                    handle_dropped_files(evt, state).await;
                }
            },

            Header {},
            TabBar {},
            Content {},

            // Drag and drop overlay
            if is_dragging() {
                DragDropOverlay {}
            }
        }
    }
}

/// Handle dropped markdown files - opens each file appropriately
async fn handle_dropped_files(evt: Event<DragData>, mut state: AppState) {
    let Some(file_engine) = evt.files() else {
        return;
    };

    for file_name in &file_engine.files() {
        let path = PathBuf::from(file_name);

        if is_markdown_file(&path) {
            tracing::info!("Opening dropped file: {:?}", path);
            state.open_file(path);
        } else {
            tracing::warn!("Ignored non-markdown file: {:?}", path);
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
