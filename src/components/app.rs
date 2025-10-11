use crate::state::OPENED_FILES_RECEIVER;
use dioxus::desktop::{window, Config};
use dioxus::prelude::*;
use std::path::PathBuf;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/dist/main.css");
const MAIN_SCRIPT: Asset = asset!("/assets/dist/main.js");

#[component]
pub fn App() -> Element {
    let mut rx = OPENED_FILES_RECEIVER
        .lock()
        .expect("Failed to lock OPENED_FILES_RECEIVER")
        .take()
        .expect("OPENED_FILES_RECEIVER is not set");
    // Use main thread to receive the first file if exists
    let first_file = rx.try_recv().ok();
    if let Some(ref file) = first_file {
        tracing::info!("Opening first file: {:?}", file);
    } else {
        tracing::info!("No initial file to open");
    }
    // Spawn a task to receive files forever to open new windows
    spawn_forever(async move {
        while let Some(file) = rx.recv().await {
            tracing::info!("Opening new file: {:?}", file);
            let dom = VirtualDom::new_with_props(AppWindow, AppWindowProps { file: Some(file) });
            window().new_window(dom, Config::new());
        }
    });
    rsx! { AppWindow { file: first_file } }
}

#[component]
fn AppWindow(file: Option<PathBuf>) -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Script { r#type: "module", src: MAIN_SCRIPT }
        h2 { "Opened File: {file:?}" }
    }
}
