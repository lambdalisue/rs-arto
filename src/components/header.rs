mod theme_selector;

use dioxus::prelude::*;

use crate::components::icon::{Icon, IconName};
use crate::state::AppState;
use theme_selector::ThemeSelector;

#[component]
pub fn Header() -> Element {
    let mut state = use_context::<AppState>();
    let file = state
        .file
        .read()
        .as_deref()
        .map(|f| {
            f.file_name()
                .unwrap_or(f.as_os_str())
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|| "No file opened".to_string());

    let can_go_back = state.history.read().can_go_back();
    let can_go_forward = state.history.read().can_go_forward();

    let on_back = move |_| {
        if let Some(path) = state.history.write().go_back() {
            state.file.set(Some(path));
        }
    };

    let on_forward = move |_| {
        if let Some(path) = state.history.write().go_forward() {
            state.file.set(Some(path));
        }
    };

    rsx! {
        div {
            class: "header",

            // File name display (left side) with navigation buttons
            div {
                class: "header-left",

                // Back button
                button {
                    class: "nav-button",
                    disabled: !can_go_back,
                    onclick: on_back,
                    Icon { name: IconName::ChevronLeft }
                }

                // Forward button
                button {
                    class: "nav-button",
                    disabled: !can_go_forward,
                    onclick: on_forward,
                    Icon { name: IconName::ChevronRight }
                }

                // File name
                span {
                    class: "file-name",
                    "{file}"
                }
            }

            // Theme selector (right side)
            ThemeSelector {}
        }
    }
}
