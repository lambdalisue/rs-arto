use crate::state::OPEN_EVENT_RECEIVER;
use crate::window as window_manager;
use dioxus::desktop::window;
use dioxus::prelude::*;
use dioxus_core::spawn_forever;
use dioxus_desktop::use_muda_event_handler;

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

    // Extract initial file if present
    let first_file = match &first_event {
        Some(crate::state::OpenEvent::File(path)) => Some(path.clone()),
        _ => None,
    };

    tracing::info!("Creating first child window");
    window_manager::create_new_main_window(first_file.clone(), true);

    // Handle directory event after window creation
    if let Some(crate::state::OpenEvent::Directory(dir)) = first_event {
        tracing::info!("Setting initial directory: {:?}", &dir);
        let _ = crate::state::DIRECTORY_OPEN_BROADCAST.send(dir);
    }

    // Handle open events (file opened, directory opened, or app icon clicked)
    spawn_forever(async move {
        while let Some(event) = rx.recv().await {
            match event {
                crate::state::OpenEvent::File(file) => {
                    if !window_manager::has_any_main_windows() {
                        window_manager::create_new_main_window(Some(file), false);
                    } else {
                        let _ = crate::state::FILE_OPEN_BROADCAST.send(file);
                    }
                }
                crate::state::OpenEvent::Directory(dir) => {
                    if !window_manager::has_any_main_windows() {
                        window_manager::create_new_main_window(None, false);
                    }
                    // Broadcast directory change to all windows
                    let _ = crate::state::DIRECTORY_OPEN_BROADCAST.send(dir);
                }
                crate::state::OpenEvent::Reopen => {
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
