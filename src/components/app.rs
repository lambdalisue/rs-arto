use dioxus::prelude::*;
use std::path::PathBuf;

use super::content::Content;
use crate::state::AppState;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/dist/main.css");
const MAIN_SCRIPT: Asset = asset!("/assets/dist/main.js");

#[component]
pub fn App(file: Option<PathBuf>) -> Element {
    let file = Signal::new(file);
    let _state = use_context_provider(|| AppState {
        file,
        ..Default::default()
    });
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Script { r#type: "module", src: MAIN_SCRIPT }
        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh;",

            Content {},
        }
    }
}
