mod markdown_viewer;
mod no_file_view;

use dioxus::prelude::*;

use crate::state::AppState;
use markdown_viewer::MarkdownViewer;
use no_file_view::NoFileView;

#[component]
pub fn Content() -> Element {
    let state = use_context::<AppState>();
    let file = state.file;
    let zoom_level = state.zoom_level;

    // Use CSS zoom property for vector-based scaling (not transform: scale)
    // This ensures fonts and images remain sharp at any zoom level
    let zoom_style = format!("zoom: {};", zoom_level());

    rsx! {
        div {
            class: "content",
            style: "{zoom_style}",

            if let Some(file) = file().clone() {
                MarkdownViewer { file }
            } else {
                NoFileView {}
            }
        }
    }
}
