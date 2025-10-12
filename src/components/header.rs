mod theme_selector;

use dioxus::prelude::*;

use crate::state::AppState;
use theme_selector::ThemeSelector;

#[component]
pub fn Header() -> Element {
    let state = use_context::<AppState>();
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
    rsx! {
        div {
            class: "header",

            // File name display (left side)
            div {
                class: "header-left",
                span { "{file}" }
            }

            // Theme selector (right side)
            ThemeSelector {}
        }
    }
}
