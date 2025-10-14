use crate::state::OPEN_EVENT_RECEIVER;
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

    let mut rx = OPEN_EVENT_RECEIVER
        .lock()
        .expect("Failed to lock OPEN_EVENT_RECEIVER")
        .take()
        .expect("OPEN_EVENT_RECEIVER is not set");

    // Open the first file or show welcome screen
    let first_file = if let Ok(Some(file)) = rx.try_recv() {
        tracing::info!("Opening first file: {:?}", &file);
        Some(file)
    } else {
        tracing::info!("No initial file to open, showing welcome screen");
        None
    };
    tracing::info!("Creating first child window");
    window_manager::create_new_main_window(first_file.clone(), true);

    // Handle open events (file opened or app icon clicked)
    spawn_forever(async move {
        while let Some(event) = rx.recv().await {
            match event {
                Some(file) => {
                    // Event::Opened
                    if !window_manager::has_any_main_windows() {
                        window_manager::create_new_main_window(Some(file), false);
                    } else {
                        let _ = crate::state::FILE_OPEN_BROADCAST.send(file);
                    }
                }
                None => {
                    // Event::Reopen
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
