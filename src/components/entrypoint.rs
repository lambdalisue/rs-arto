use crate::state::OPENED_FILES_RECEIVER;
use crate::window as window_manager;
use dioxus::desktop::window;
use dioxus::prelude::*;

#[component]
pub fn Entrypoint() -> Element {
    let mut rx = OPENED_FILES_RECEIVER
        .lock()
        .expect("Failed to lock OPENED_FILES_RECEIVER")
        .take()
        .expect("OPENED_FILES_RECEIVER is not set");

    // Open the first file or empty window
    let first_file = if let Ok(file) = rx.try_recv() {
        tracing::info!("Opening first file: {:?}", &file);
        Some(file)
    } else {
        tracing::info!("No initial file to open");
        get_sampe_file_on_debug_build()
    };
    tracing::info!("Creating first child window");
    window_manager::create_new_window(first_file.clone());

    // Spawn a task to receive files forever to open new windows
    spawn_forever(async move {
        while let Some(file) = rx.recv().await {
            tracing::info!("Opening new file: {:?}", file);
            window_manager::create_new_window(Some(file));
        }
    });

    // Hide the background window and create the first child window
    use_hook(move || {
        tracing::info!("Hiding background window");
        window().set_visible(false);
    });

    rsx! {
        div {
            h1 { "Octoscope Background Process" }
        }
    }
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
