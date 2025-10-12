use dioxus::prelude::*;
use std::path::PathBuf;

use super::content::Content;
use super::header::Header;
use crate::state::AppState;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_SCRIPT: Asset = asset!("/assets/dist/main.js");
const MAIN_STYLE: Asset = asset!("/assets/dist/main.css");

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
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_STYLE }
        document::Script { r#type: "module", src: MAIN_SCRIPT }
        div {
            class: "app-container",

            Header {},
            Content {},
        }
    }
}
