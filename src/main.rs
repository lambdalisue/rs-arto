mod components;

use dioxus::prelude::Element;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;

fn main() {
    // Load environment variables from .env file
    if let Ok(dotenv) = dotenvy::dotenv() {
        println!("Loaded .env file from: {}", dotenv.display());
    }

    // Initialize tracing with pretty formatter and env filter
    // Can be configured via RUST_LOG environment variable
    // Examples: RUST_LOG=debug, RUST_LOG=octoscope=trace, RUST_LOG=octoscope::markdown=debug
    init_tracing();

    launch(components::app::App);
}

fn init_tracing() {
    let registry = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .without_time()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true),
        );

    #[cfg(target_os = "macos")]
    let registry = registry.with(tracing_oslog::OsLogger::new(
        "com.lambdalisue.Octoscope",
        "defaut",
    ));

    registry.init();
}

#[cfg(target_os = "macos")]
fn launch(app: fn() -> Element) {
    use dioxus::desktop::{Config, WindowBuilder};

    let config = Config::new().with_window(
        WindowBuilder::new()
            .with_title("Octoscope")
            .with_focused(!cfg!(debug_assertions)), // Avoid focus stealing in debug mode
    );
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app);
}

#[cfg(not(target_os = "macos"))]
fn launch(app: fn() -> Element) {
    dioxus::launch(app);
}
