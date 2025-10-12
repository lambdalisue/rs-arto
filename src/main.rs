mod components;
mod history;
mod markdown;
mod menu;
mod state;
mod theme;
mod window;

use dioxus::desktop::{tao::dpi::PhysicalPosition, Config, LogicalSize, WindowBuilder};
use std::path::PathBuf;
use tokio::sync::mpsc::channel;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;

const DEFAULT_LOGLEVEL: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "info"
};

fn main() {
    // Load environment variables from .env file
    if let Ok(dotenv) = dotenvy::dotenv() {
        println!("Loaded .env file from: {}", dotenv.display());
    }
    init_tracing();

    let config = create_config();
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(components::entrypoint::Entrypoint);
}

fn init_tracing() {
    let registry = tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOGLEVEL)),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .without_time()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true),
        );

    // On macOS, log to Console.app via oslog
    #[cfg(target_os = "macos")]
    let registry = registry.with(tracing_oslog::OsLogger::new(
        "com.lambdalisue.Octoscope",
        "defaut",
    ));

    registry.init();
}

#[cfg(target_os = "macos")]
fn create_config() -> Config {
    use dioxus::desktop::tao::event::Event;

    let (tx, rx) = channel::<PathBuf>(10);
    state::OPENED_FILES_RECEIVER
        .lock()
        .expect("Failed to lock OPENED_FILES_RECEIVER")
        .replace(rx);

    let menu = menu::build_menu();

    // Create a hidden background window for menu handling
    // This window should not be visible to users
    let window = WindowBuilder::new()
        .with_focused(false)
        // Must be visible at start, otherwise child windows won't be shown
        // We will hide it immediately after creation in the component
        .with_visible(true)
        // Make the window as small and unobtrusive as possible
        .with_decorations(false)
        .with_position(PhysicalPosition::new(0, 0))
        // Must be at least 1x1, otherwise it will panic
        .with_inner_size(LogicalSize::new(1, 1));

    Config::new()
        // Listen to macOS open file events. This custom event handler must be specified before
        // the window is created. Otherwise, the Opened event will be lost for first launch.
        .with_custom_event_handler(move |event, _target| {
            if let Event::Opened { urls, .. } = event {
                tracing::debug!(target: "open", "Opened {:?}", urls);
                for url in urls {
                    if let Ok(path) = url.to_file_path() {
                        tx.try_send(path).expect("Failed to send opened file");
                    }
                }
            }
        })
        .with_menu(menu)
        .with_window(window)
}

#[cfg(not(target_os = "macos"))]
fn create_config() -> Config {
    let (_tx, rx) = channel::<PathBuf>(10);
    state::OPENED_FILES_RECEIVER
        .lock()
        .expect("Failed to lock OPENED_FILES_RECEIVER")
        .replace(rx);

    let menu = menu::build_menu();

    // Create a hidden background window for menu handling
    // This window should not be visible to users
    let window = WindowBuilder::new()
        .with_focused(false)
        // Must be visible at start, otherwise child windows won't be shown
        // We will hide it immediately after creation in the component
        .with_visible(true)
        // Make the window as small and unobtrusive as possible
        .with_decorations(false)
        .with_position(PhysicalPosition::new(0, 0))
        // Must be at least 1x1, otherwise it will panic
        .with_inner_size(LogicalSize::new(1, 1));

    Config::new()
        .with_menu(menu)
        .with_window(window)
        // This is important to avoid panic when closing child windows
        .with_close_behaviour(WindowCloseBehaviour::LastWindowHides)
}
