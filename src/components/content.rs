mod markdown_viewer;
mod no_file_view;

use dioxus::prelude::*;

use crate::state::{AppState, TabContent};
use markdown_viewer::MarkdownViewer;
use no_file_view::NoFileView;

#[component]
pub fn Content() -> Element {
    let state = use_context::<AppState>();
    let zoom_level = state.zoom_level;

    let current_tab = state.current_tab();
    let content = current_tab.map(|tab| tab.content);

    // Use CSS zoom property for vector-based scaling (not transform: scale)
    // This ensures fonts and images remain sharp at any zoom level
    let zoom_style = format!("zoom: {};", zoom_level());

    rsx! {
        div {
            class: "content",
            style: "{zoom_style}",

            match content {
                Some(TabContent::File(_)) | Some(TabContent::Inline(_)) => {
                    rsx! { MarkdownViewer { content: content.unwrap() } }
                },
                _ => rsx! { NoFileView {} },
            }
        }
    }
}
