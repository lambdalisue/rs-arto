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
    let file = Signal::new(file);
    let _state = use_context_provider(|| AppState {
        file,
        ..Default::default()
    });
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
