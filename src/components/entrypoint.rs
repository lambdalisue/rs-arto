use crate::events::{DIRECTORY_OPEN_BROADCAST, FILE_OPEN_BROADCAST};
use crate::window as window_manager;
use dioxus::desktop::window;
use dioxus::prelude::*;
use dioxus_core::spawn_forever;
use dioxus_desktop::use_muda_event_handler;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;

/// Open event types for distinguishing files, directories, and reopen events
/// Used to communicate between OS event handler (main.rs) and Entrypoint component
#[derive(Debug, Clone)]
pub enum OpenEvent {
    /// File opened from Finder/CLI
    File(PathBuf),
    /// Directory opened from Finder/CLI (should set sidebar root)
    Directory(PathBuf),
    /// App icon clicked (reopen event)
    Reopen,
}

/// A global receiver to receive open events from the main thread (OS → Dioxus context)
/// This is set once by main.rs and consumed once by this Entrypoint component.
pub static OPEN_EVENT_RECEIVER: Mutex<Option<Receiver<OpenEvent>>> = Mutex::new(None);

#[component]
pub fn Entrypoint() -> Element {
    // Handle global menu events
    use_muda_event_handler(move |event| {
        crate::menu::handle_menu_event_global(event);
    });

    let mut rx = OPEN_EVENT_RECEIVER
        .lock()
        .expect("Failed to lock OPEN_EVENT_RECEIVER")
        .take()
        .expect("OPEN_EVENT_RECEIVER is not set");

    // Handle initial event (file, directory, or none)
    let first_event = if let Ok(event) = rx.try_recv() {
        tracing::info!("Received initial open event: {:?}", &event);
        Some(event)
    } else {
        tracing::info!("No initial event, showing welcome screen");
        None
    };

    // Extract initial file if present (directories handled separately via broadcast)
    let first_file = match &first_event {
        Some(OpenEvent::File(path)) => Some(path.clone()),
        Some(OpenEvent::Directory(_)) => None,
        _ => None,
    };

    // Clone first_event for use inside spawn
    let first_event_for_spawn = first_event.clone();

    // Create first window
    spawn(async move {
        // Create first window (theme/directory settings applied in window.rs)
        tracing::info!("Creating first child window");
        window_manager::create_new_main_window(first_file.clone(), true);

        // Handle explicit directory event (overrides config settings)
        if let Some(OpenEvent::Directory(dir)) = first_event_for_spawn {
            tracing::info!("Setting initial directory from event: {:?}", &dir);
            let _ = DIRECTORY_OPEN_BROADCAST.send(dir);
        }
    });

    // Handle open events (file opened, directory opened, or app icon clicked)
    spawn_forever(async move {
        while let Some(event) = rx.recv().await {
            match event {
                OpenEvent::File(file) => {
                    if !window_manager::has_any_main_windows() {
                        window_manager::create_new_main_window(Some(file), false);
                    } else {
                        let _ = FILE_OPEN_BROADCAST.send(file);
                    }
                }
                OpenEvent::Directory(dir) => {
                    if !window_manager::has_any_main_windows() {
                        window_manager::create_new_main_window(None, false);
                    }
                    // Broadcast directory change to all windows
                    let _ = DIRECTORY_OPEN_BROADCAST.send(dir);
                }
                OpenEvent::Reopen => {
                    if !window_manager::focus_last_focused_main_window() {
                        window_manager::create_new_main_window(None, false);
                    }
                }
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
