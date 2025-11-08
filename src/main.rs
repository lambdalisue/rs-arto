mod assets;
mod components;
mod history;
mod markdown;
mod menu;
mod state;
mod theme;
mod utils;
mod watcher;
mod window;

use dioxus::desktop::tao::event::{Event, WindowEvent};
use dioxus::desktop::{tao::dpi::PhysicalPosition, Config, LogicalSize, WindowBuilder};
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
    let silence_filter = tracing_subscriber::filter::filter_fn(|metadata| {
        // Filter out specific error from dioxus_core::properties:136
        // Known issue: https://github.com/DioxusLabs/dioxus/issues/3872
        metadata.target() != "dioxus_core::properties::__component_called_as_function"
    });

    let env_filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOGLEVEL));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .without_time()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_filter(silence_filter.clone());

    let registry = tracing_subscriber::registry()
        .with(env_filter_layer)
        .with(fmt_layer);

    // On macOS, log to Console.app via oslog
    #[cfg(target_os = "macos")]
    let registry = registry.with(
        tracing_oslog::OsLogger::new("com.lambdalisue.Arto", "defaut").with_filter(silence_filter),
    );

    registry.init();
}

#[cfg(target_os = "macos")]
fn create_config() -> Config {
    let (tx, rx) = channel::<state::OpenEvent>(10);
    state::OPEN_EVENT_RECEIVER
        .lock()
        .expect("Failed to lock OPEN_EVENT_RECEIVER")
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
        .with_custom_event_handler(move |event, _target| match event {
            Event::Opened { urls, .. } => {
                for url in urls {
                    if let Ok(path) = url.to_file_path() {
                        let open_event = if path.is_dir() {
                            state::OpenEvent::Directory(path)
                        } else if path.is_file() {
                            state::OpenEvent::File(path)
                        } else {
                            // Skip invalid paths
                            continue;
                        };
                        tx.try_send(open_event).expect("Failed to send open event");
                    }
                }
            }
            Event::Reopen { .. } => {
                // Send reopen event through channel to handle it safely in component context
                tx.try_send(state::OpenEvent::Reopen).ok();
            }
            Event::WindowEvent {
                event: WindowEvent::Focused(true),
                window_id,
                ..
            } => {
                window::update_last_focused_window(*window_id);
            }
            _ => {}
        })
        .with_menu(menu)
        .with_window(window)
}

#[cfg(not(target_os = "macos"))]
fn create_config() -> Config {
    let (_tx, rx) = channel::<state::OpenEvent>(10);
    state::OPEN_EVENT_RECEIVER
        .lock()
        .expect("Failed to lock OPEN_EVENT_RECEIVER")
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
        // Listen to window focus events
        .with_custom_event_handler(move |event, _target| {
            if let Event::WindowEvent {
                event: WindowEvent::Focused(true),
                window_id,
                ..
            } = event
            {
                window::update_last_focused_window(*window_id);
            }
        })
        .with_menu(menu)
        .with_window(window)
        // This is important to avoid panic when closing child windows
        .with_close_behaviour(WindowCloseBehaviour::LastWindowHides)
}
