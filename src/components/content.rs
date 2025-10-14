mod file_error_view;
mod file_viewer;
mod inline_viewer;
mod no_file_view;

use dioxus::prelude::*;

use crate::state::{AppState, TabContent};
use file_error_view::FileErrorView;
use file_viewer::FileViewer;
use inline_viewer::InlineViewer;
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
                Some(TabContent::File(file)) => {
                    rsx! { FileViewer { file } }
                },
                Some(TabContent::Inline(markdown)) => {
                    rsx! { InlineViewer { markdown } }
                },
                Some(TabContent::FileError(file, error)) => {
                    let filename = file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown file")
                        .to_string();
                    rsx! { FileErrorView { filename, error_message: error } }
                },
                _ => rsx! { NoFileView {} },
            }
        }
    }
}
