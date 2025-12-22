use crate::events::{DIRECTORY_OPEN_BROADCAST, FILE_OPEN_BROADCAST};
use crate::window as window_manager;
use crate::window::metrics::update_outer_to_inner_metrics;
use crate::window::settings;
use dioxus::core::spawn_forever;
use dioxus::desktop::use_muda_event_handler;
use dioxus::desktop::{window, WindowCloseBehaviour};
use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;

// ============================================================================
// OpenEvent definition
// ============================================================================

/// Open event types for distinguishing files, directories, and reopen events
/// Used to communicate between OS event handler (main.rs) and MainApp component
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
/// This is set once by main.rs and consumed once by this MainApp component.
pub static OPEN_EVENT_RECEIVER: Mutex<Option<Receiver<OpenEvent>>> = Mutex::new(None);

// ============================================================================
// System event handling
// ============================================================================

#[tracing::instrument]
fn handle_open_event(event: OpenEvent) {
    tracing::debug!(?event, "Handling system open event");

    match event {
        OpenEvent::File(file) => {
            if window_manager::has_any_main_windows() {
                let _ = FILE_OPEN_BROADCAST.send(file);
            } else {
                window_manager::create_new_main_window(Some(file), None, false);
            }
        }
        OpenEvent::Directory(dir) => {
            if window_manager::has_any_main_windows() {
                let _ = DIRECTORY_OPEN_BROADCAST.send(dir);
            } else {
                window_manager::create_new_main_window(None, Some(dir), false);
            }
        }
        OpenEvent::Reopen => {
            if !window_manager::focus_last_focused_main_window() {
                window_manager::create_new_main_window(None, None, false);
            }
        }
    }
}

// ============================================================================
// MainApp component
// ============================================================================

/// MainApp - Component dedicated to the first window
/// Configures system event handling and WindowHides behavior
///
/// NOTE: This component should only be used for the first window launched from main.rs.
/// Additional windows should use the App component directly.
#[component]
pub fn MainApp() -> Element {
    // Configure WindowCloseBehaviour::WindowHides for first window
    use_hook(|| {
        tracing::debug!("Configuring main window with WindowHides behavior");
        window().set_close_behavior(WindowCloseBehaviour::WindowHides);

        // Register the first window in MAIN_WINDOWS list
        // This is critical for has_any_main_windows() to work correctly
        let weak_handle = std::rc::Rc::downgrade(&window());
        window_manager::register_main_window(weak_handle);

        // Dioxus inner_size doesn't update after resize; cache outer->inner deltas once.
        update_outer_to_inner_metrics(&window().window);
    });

    // Set up global menu event handling
    use_muda_event_handler(move |event| {
        crate::menu::handle_menu_event_global(event);
    });

    // Get receiver and consume initial event
    let mut rx = OPEN_EVENT_RECEIVER
        .lock()
        .expect("Failed to lock OPEN_EVENT_RECEIVER")
        .take()
        .expect("OPEN_EVENT_RECEIVER not initialized");

    // Handle initial event (file, directory, or none)
    let first_event = if let Ok(event) = rx.try_recv() {
        tracing::debug!(?event, "Received initial open event");
        Some(event)
    } else {
        tracing::debug!("No initial event, will show welcome screen");
        None
    };

    // Extract initial file/directory if present
    let file = match &first_event {
        Some(OpenEvent::File(path)) => Some(path.clone()),
        _ => None,
    };
    let directory = match &first_event {
        Some(OpenEvent::Directory(path)) => Some(path.clone()),
        _ => None,
    };

    // Get initial configuration values (using existing sync functions)
    let is_first_window = true;
    let directory_value = settings::get_directory_value(is_first_window, file.as_ref(), directory);
    let sidebar_value = settings::get_sidebar_value(is_first_window);

    // Set up system event handler (for subsequent events)
    use_hook(|| {
        spawn_forever(async move {
            while let Some(event) = rx.recv().await {
                handle_open_event(event);
            }
        });
    });

    // Render App component with initial state
    rsx! {
        crate::components::app::App {
            file: file,
            directory: directory_value.directory,
            sidebar_open: sidebar_value.open,
            sidebar_width: sidebar_value.width,
            sidebar_show_all_files: sidebar_value.show_all_files,
            show_welcome: first_event.is_none(),
        }
    }
}
