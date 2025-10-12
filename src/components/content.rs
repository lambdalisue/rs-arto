mod markdown_viewer;

use dioxus::prelude::*;

use crate::state::AppState;
use markdown_viewer::MarkdownViewer;

#[component]
pub fn Content() -> Element {
    let state = use_context::<AppState>();
    let file = state.file;
    rsx! {
        div {
            class: "content",

            if let Some(file) = file().as_ref() {
                MarkdownViewer { file }
            } else {
                div {
                    class: "no-file",
                    "No file opened. Please open a markdown file to view its content."
                }
            }
        }
    }
}
