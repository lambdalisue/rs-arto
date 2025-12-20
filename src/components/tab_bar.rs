use dioxus::prelude::*;

use crate::components::icon::{Icon, IconName};
use crate::state::AppState;

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

    // Clone state for event handlers
    let mut state_for_switch = state;
    let mut state_for_close = state;

    rsx! {
        div {
            class: "tab",
            class: if is_active { "active" },
            onclick: move |_| {
                state_for_switch.switch_to_tab(index);
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
