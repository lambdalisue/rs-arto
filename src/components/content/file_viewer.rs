use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::assets::MAIN_SCRIPT;
use crate::markdown::render_to_html;
use crate::state::{AppState, TabContent};
use crate::utils::file::is_markdown_file;
use crate::watcher::FILE_WATCHER;

/// Data structure for markdown link clicks from JavaScript
#[derive(Serialize, Deserialize)]
struct LinkClickData {
    path: String,
    button: u32,
}

/// Mouse button constants
const LEFT_CLICK: u32 = 0;
const MIDDLE_CLICK: u32 = 1;

#[component]
pub fn FileViewer(file: PathBuf) -> Element {
    let state = use_context::<AppState>();
    let html = use_signal(String::new);
    let reload_trigger = use_signal(|| 0usize);

    // Setup component hooks
    use_main_script_loader();
    use_file_loader(file.clone(), html, reload_trigger, state.clone());
    use_file_watcher(file.clone(), reload_trigger);
    use_link_click_handler(file, state.clone());

    rsx! {
        div {
            class: "markdown-viewer",
            article {
                class: "markdown-body",
                dangerous_inner_html: "{html}"
            }
        }
    }
}

/// Hook to load the main JavaScript bundle once on mount
fn use_main_script_loader() {
    use_effect(|| {
        spawn(async move {
            let eval = document::eval(&indoc::formatdoc! {r#"
                const {{ init }} = await import("{MAIN_SCRIPT}");
                if (document.readyState === "loading") {{
                    document.addEventListener("DOMContentLoaded", init);
                }} else {{
                    init();
                }}
            "#});
            if let Err(e) = eval.await {
                tracing::error!("Failed to load main script: {}", e);
            }
        });
    });
}

/// Hook to load and render file content
fn use_file_loader(
    file: PathBuf,
    html: Signal<String>,
    reload_trigger: Signal<usize>,
    state: AppState,
) {
    use_effect(use_reactive!(|file, reload_trigger| {
        let mut html = html;
        let _ = reload_trigger();
        let file = file.clone();
        let mut state_for_error = state.clone();

        spawn(async move {
            tracing::info!("Loading and rendering file: {:?}", &file);

            // Try to read as string (UTF-8 text file)
            match tokio::fs::read_to_string(file.as_path()).await {
                Ok(content) => {
                    // Check if file has markdown extension
                    if is_markdown_file(&file) {
                        // Render as markdown
                        match render_to_html(&content, &file) {
                            Ok(rendered) => {
                                html.set(rendered);
                                tracing::trace!("Rendered as Markdown: {:?}", &file);
                            }
                            Err(e) => {
                                // Markdown parsing failed, render as plain text
                                tracing::warn!(
                                    "Markdown parsing failed for {:?}, rendering as plain text: {}",
                                    &file,
                                    e
                                );
                                let escaped_content = html_escape::encode_text(&content);
                                let plain_html = format!(
                                    r#"<pre class="plain-text-viewer">{}</pre>"#,
                                    escaped_content
                                );
                                html.set(plain_html);
                            }
                        }
                    } else {
                        // Non-markdown file, render as plain text directly
                        tracing::info!("Rendering non-markdown file as plain text: {:?}", &file);
                        let escaped_content = html_escape::encode_text(&content);
                        let plain_html = format!(
                            r#"<pre class="plain-text-viewer">{}</pre>"#,
                            escaped_content
                        );
                        html.set(plain_html);
                    }
                }
                Err(e) => {
                    // Failed to read as UTF-8 text (likely binary file)
                    tracing::error!("Failed to read file {:?} as text: {}", file, e);
                    let error_msg = format!("{:?}", e);

                    // Update tab content to FileError
                    let file_clone = file.clone();
                    state_for_error.update_current_tab(move |tab| {
                        tab.content = TabContent::FileError(file_clone, error_msg);
                    });
                    html.set(String::new());
                }
            }
        });
    }));
}

/// Hook to watch file for changes and trigger reload
fn use_file_watcher(file: PathBuf, reload_trigger: Signal<usize>) {
    use_effect(use_reactive!(|file| {
        let mut reload_trigger = reload_trigger;
        let file = file.clone();

        spawn(async move {
            let file_path = file.clone();
            let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(10);

            if let Err(e) = FILE_WATCHER.watch(file_path.clone(), tx).await {
                tracing::error!(
                    "Failed to register file watcher for {:?}: {:?}",
                    file_path,
                    e
                );
                return;
            }

            while rx.recv().await.is_some() {
                tracing::info!("File change detected, reloading: {:?}", file_path);
                reload_trigger.set(reload_trigger() + 1);
            }

            if let Err(e) = FILE_WATCHER.unwatch(file_path.clone()).await {
                tracing::error!(
                    "Failed to unregister file watcher for {:?}: {:?}",
                    file_path,
                    e
                );
            }
        });
    }));
}

/// Hook to setup JavaScript handler for markdown link clicks
fn use_link_click_handler(file: PathBuf, state: AppState) {
    use_effect(use_reactive!(|file| {
        let file = file.clone();
        let mut eval_provider = document::eval(indoc::indoc! {r#"
            window.handleMarkdownLinkClick = (path, button) => {
                dioxus.send({ path, button });
            };
        "#});

        let base_dir = file
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        let mut state_clone = state.clone();

        spawn(async move {
            while let Ok(click_data) = eval_provider.recv::<LinkClickData>().await {
                handle_link_click(click_data, &base_dir, &mut state_clone);
            }
        });
    }));
}

/// Handle a markdown link click event
fn handle_link_click(click_data: LinkClickData, base_dir: &Path, state: &mut AppState) {
    let LinkClickData { path, button } = click_data;

    tracing::info!("Markdown link clicked: {} (button: {})", path, button);

    // Resolve and normalize the path
    let target_path = base_dir.join(&path);
    let Ok(canonical_path) = target_path.canonicalize() else {
        tracing::error!("Failed to resolve path: {:?}", target_path);
        return;
    };

    tracing::info!("Opening file: {:?}", canonical_path);

    match button {
        MIDDLE_CLICK => {
            // Open in new tab (always create a new tab for middle-click)
            state.add_tab(Some(canonical_path), true);
        }
        LEFT_CLICK => {
            // Navigate in current tab (in-tab navigation, no existing tab check)
            state.navigate_to_file(canonical_path);
        }
        _ => {
            tracing::debug!("Ignoring click with button: {}", button);
        }
    }
}
