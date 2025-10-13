use dioxus::html::HasFileData;
use dioxus::prelude::*;
use dioxus_desktop::use_muda_event_handler;
use std::path::PathBuf;

use super::content::Content;
use super::header::Header;
use super::icon::{Icon, IconName};
use crate::menu;
use crate::state::AppState;
use crate::window;

#[component]
pub fn App(file: Option<PathBuf>) -> Element {
    let file_signal = Signal::new(file.clone());
    let mut state = use_context_provider(|| AppState {
        file: file_signal,
        ..Default::default()
    });

    // Add initial file to history if provided
    if let Some(path) = file {
        state.history.write().push(path);
    }

    // Track drag-over state for visual feedback
    let mut hovered = use_signal(|| false);

    // Clone state references for different closures
    let mut state_for_menu = state.clone();
    let mut state_for_drop = state;

    // Handle menu events (only state-dependent events, not global ones)
    use_muda_event_handler(move |event| {
        // Only handle state-dependent events
        menu::handle_menu_event_with_state(event, &mut state_for_menu);
    });

    rsx! {
        div {
            class: "app-container",
            class: if hovered() { "drag-over" },
            ondragover: move |evt| {
                evt.prevent_default();
                hovered.set(true);
            },
            ondragleave: move |_| {
                hovered.set(false);
            },
            ondrop: move |evt| async move {
                evt.prevent_default();
                hovered.set(false);

                if let Some(file_engine) = evt.files() {
                    let file_names = file_engine.files();
                    let mut first_file = true;

                    for file_name in &file_names {
                        let path = PathBuf::from(&file_name);
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");

                        if ext == "md" || ext == "markdown" {
                            if first_file {
                                // Open first file in current window
                                tracing::info!("Opening file in current window: {:?}", path);
                                state_for_drop.history.write().push(path.clone());
                                state_for_drop.file.set(Some(path));
                                first_file = false;
                            } else {
                                // Open subsequent files in new windows
                                tracing::info!("Opening file in new window: {:?}", path);
                                window::create_new_window(Some(path));
                            }
                        } else {
                            tracing::warn!("Dropped file is not a markdown file: {:?}", path);
                        }
                    }
                }
            },

            Header {},
            Content {},

            // Drag and drop overlay
            if hovered() {
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
    }
}
