use crate::state::OPENED_FILES_RECEIVER;
use dioxus::desktop::{window, Config};
use dioxus::prelude::*;

use super::app::{App, AppProps};

#[component]
pub fn Entrypoint() -> Element {
    let mut rx = OPENED_FILES_RECEIVER
        .lock()
        .expect("Failed to lock OPENED_FILES_RECEIVER")
        .take()
        .expect("OPENED_FILES_RECEIVER is not set");
    // Use main thread to receive the first file if exists
    let first_file = if let Ok(file) = rx.try_recv() {
        tracing::info!("Opening first file: {:?}", &file);
        Some(file)
    } else {
        tracing::info!("No initial file to open");
        get_sampe_file_on_debug_build()
    };
    // Spawn a task to receive files forever to open new windows
    spawn_forever(async move {
        while let Some(file) = rx.recv().await {
            tracing::info!("Opening new file: {:?}", file);
            let dom = VirtualDom::new_with_props(App, AppProps { file: Some(file) });
            let config = create_config();
            window().new_window(dom, config);
        }
    });
    rsx! { App { file: first_file } }
}

fn create_config() -> Config {
    use dioxus::desktop::WindowBuilder;
    Config::new().with_window(
        WindowBuilder::new()
            .with_title("Octoscope")
            .with_focused(!cfg!(debug_assertions)), // Avoid stealing focus in debug mode
    )
}

#[cfg(debug_assertions)]
fn get_sampe_file_on_debug_build() -> Option<std::path::PathBuf> {
    use std::path::Path;
    let sample_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("example")
        .join("sample1.md");
    if sample_file.exists() {
        Some(sample_file)
    } else {
        tracing::warn!("Sample file does not exist at {:?}", &sample_file);
        None
    }
}

#[cfg(not(debug_assertions))]
fn get_sampe_file_on_debug_build() -> Option<std::path::PathBuf> {
    None
}
