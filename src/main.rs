mod components;
mod markdown;
mod state;
mod theme;

use dioxus::desktop::Config;
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
    use dioxus::desktop::WindowBuilder;

    let (tx, rx) = channel::<PathBuf>(10);
    state::OPENED_FILES_RECEIVER
        .lock()
        .expect("Failed to lock OPENED_FILES_RECEIVER")
        .replace(rx);
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
        .with_window(
            WindowBuilder::new()
                .with_title("Octoscope")
                .with_focused(!cfg!(debug_assertions)), // Avoid focus stealing in debug mode
        )
}

#[cfg(not(target_os = "macos"))]
fn create_config() -> Config {
    use dioxus::desktop::WindowBuilder;

    let (_tx, rx) = channel::<PathBuf>(10);
    state::OPENED_FILES_RECEIVER
        .lock()
        .expect("Failed to lock OPENED_FILES_RECEIVER")
        .replace(rx);
    Config::new().with_window(
        WindowBuilder::new()
            .with_title("Octoscope")
            .with_focused(!cfg!(debug_assertions)), // Avoid focus stealing in debug mode
    )
}
