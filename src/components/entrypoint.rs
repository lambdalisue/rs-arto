use crate::state::OPENED_FILES_RECEIVER;
use crate::window as window_manager;
use dioxus::desktop::window;
use dioxus::prelude::*;
use dioxus_desktop::use_muda_event_handler;

#[component]
pub fn Entrypoint() -> Element {
    // Handle global menu events
    use_muda_event_handler(move |event| {
        crate::menu::handle_menu_event_global(event);
    });

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
        get_sample_file_on_debug_build()
    };
    tracing::info!("Creating first child window");
    window_manager::create_new_window(first_file.clone());

    // Broadcast received files to all windows
    spawn_forever(async move {
        while let Some(file) = rx.recv().await {
            tracing::info!("Broadcasting file open request: {:?}", file);

            // If no windows exist, create a new window with the file
            if !window_manager::has_any_child_windows() {
                tracing::info!("No windows open, creating new window with file: {:?}", file);
                window_manager::create_new_window(Some(file));
            } else {
                // Otherwise broadcast to existing windows
                let _ = crate::state::FILE_OPEN_BROADCAST.send(file);
            }
        }
    });

    // Hide the background window and create the first child window
    use_hook(move || {
        tracing::info!("Hiding background window");
        window().set_visible(false);
    });

    rsx! {
        div {
            h1 { "Arto Background Process" }
        }
    }
}

#[cfg(debug_assertions)]
fn get_sample_file_on_debug_build() -> Option<std::path::PathBuf> {
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
fn get_sample_file_on_debug_build() -> Option<std::path::PathBuf> {
    None
}
