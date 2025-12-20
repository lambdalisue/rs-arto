use dioxus::prelude::*;
use std::sync::{LazyLock, Mutex};

use crate::components::icon::{Icon, IconName};
use crate::state::AppState;

// Shared state for tracking the currently dragged tab
static DRAGGED_TAB_INDEX: LazyLock<Mutex<Option<usize>>> =
    LazyLock::new(|| Mutex::new(None));

// Track whether a drop was successful (within the window)
static DROP_SUCCESSFUL: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

/// Extract display name from a tab's content
fn get_tab_display_name(tab: &crate::state::Tab) -> String {
    use crate::state::TabContent;
    match &tab.content {
        TabContent::File(path) | TabContent::FileError(path, _) => path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unnamed file".to_string()),
        TabContent::Inline(_) => "Welcome".to_string(),
        TabContent::Preferences => "Preferences".to_string(),
        TabContent::None => "No file".to_string(),
    }
}

#[component]
pub fn TabBar() -> Element {
    let state = use_context::<AppState>();
    let tabs = state.tabs.read().clone();
    let active_tab_index = *state.active_tab.read();

    rsx! {
        div {
            class: "tab-bar",

            // Render existing tabs
            for (index, tab) in tabs.iter().enumerate() {
                TabItem {
                    key: "{index}",
                    index,
                    tab: tab.clone(),
                    is_active: index == active_tab_index,
                }
            }

            // New tab button
            NewTabButton {}
        }
    }
}

#[component]
fn TabItem(index: usize, tab: crate::state::Tab, is_active: bool) -> Element {
    let state = use_context::<AppState>();
    let tab_name = get_tab_display_name(&tab);

    // Track drag state for visual feedback
    let mut is_dragging = use_signal(|| false);
    let mut drag_over_index = use_signal(|| None::<usize>);

    // Clone state for event handlers
    let mut state_for_switch = state;
    let mut state_for_close = state;
    let mut state_for_reorder = state;
    let mut state_for_detach = state;

    rsx! {
        div {
            class: "tab",
            class: if is_active { "active" },
            class: if is_dragging() { "dragging" },
            class: if drag_over_index() == Some(index) { "drag-over" },
            draggable: true,
            onclick: move |_| {
                state_for_switch.switch_to_tab(index);
            },
            ondragstart: move |_evt| {
                is_dragging.set(true);
                // Store the dragged tab index in shared state
                *DRAGGED_TAB_INDEX.lock().unwrap() = Some(index);
                // Reset drop successful flag
                *DROP_SUCCESSFUL.lock().unwrap() = false;
            },
            ondragend: move |_| {
                is_dragging.set(false);
                drag_over_index.set(None);

                // Check if the drop was successful
                let was_successful = *DROP_SUCCESSFUL.lock().unwrap();

                // If not successful, the tab was dropped outside the window
                if !was_successful {
                    if let Some(dragged_index) = *DRAGGED_TAB_INDEX.lock().unwrap() {
                        // Get cursor position to detect target window
                        use dioxus_desktop::window;
                        let cursor_pos = window().cursor_position();

                        if let Ok(pos) = cursor_pos {
                            let screen_x = pos.x as i32;
                            let screen_y = pos.y as i32;

                            // Check if there's a window at the cursor position
                            let target_window = crate::window::get_window_at_position(screen_x, screen_y);
                            let current_window = window().id();

                            // Get the tab data before closing it
                            let tabs = state_for_detach.tabs.read();
                            if let Some(tab) = tabs.get(dragged_index).cloned() {
                                let directory = state_for_detach.directory.read().clone();
                                drop(tabs); // Release the read lock

                                match target_window {
                                    Some(target_id) if target_id != current_window => {
                                        // Transfer to existing window
                                        tracing::info!(
                                            ?target_id,
                                            index = dragged_index,
                                            "Transferring tab to existing window"
                                        );
                                        let _ = crate::window::TAB_TRANSFER_BROADCAST.send(
                                            crate::window::TabTransfer {
                                                target_window: target_id,
                                                tab,
                                            }
                                        );
                                    }
                                    _ => {
                                        // Create new window
                                        tracing::info!(index = dragged_index, "Detaching tab to new window");
                                        crate::window::detach_tab_to_new_window(tab, directory);
                                    }
                                }

                                // Close the tab in the source window
                                state_for_detach.close_tab(dragged_index);
                            }
                        }
                    }
                }

                // Clear the dragged tab index
                *DRAGGED_TAB_INDEX.lock().unwrap() = None;
                *DROP_SUCCESSFUL.lock().unwrap() = false;
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drag_over_index.set(Some(index));
            },
            ondragleave: move |_| {
                drag_over_index.set(None);
            },
            ondrop: move |evt| {
                evt.prevent_default();
                drag_over_index.set(None);

                // Mark drop as successful
                *DROP_SUCCESSFUL.lock().unwrap() = true;

                // Get the source tab index from shared state
                if let Some(source_index) = *DRAGGED_TAB_INDEX.lock().unwrap() {
                    // Reorder the tab
                    state_for_reorder.reorder_tab(source_index, index);
                }
            },

            span {
                class: "tab-name",
                "{tab_name}"
            }

            button {
                class: "tab-close",
                onclick: move |evt| {
                    evt.stop_propagation();
                    state_for_close.close_tab(index);
                },
                Icon { name: IconName::Close, size: 14 }
            }
        }
    }
}

#[component]
fn NewTabButton() -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        button {
            class: "tab-new",
            onclick: move |_| {
                state.add_tab(None, true);
            },
            Icon { name: IconName::Add, size: 16 }
        }
    }
}
