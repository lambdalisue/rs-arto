use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::assets::MAIN_SCRIPT;
use crate::markdown::render_to_html;
use crate::state::{AppState, TabContent};
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
pub fn MarkdownViewer(content: TabContent) -> Element {
    let state = use_context::<AppState>();
    let html = use_signal(String::new);
    let reload_trigger = use_signal(|| 0usize);

    // Setup component hooks - always call in the same order
    use_main_script_loader();

    // Extract file path if available
    let file_opt = match &content {
        TabContent::File(file) => Some(file.clone()),
        _ => None,
    };

    // Extract inline content if available
    let inline_opt = match &content {
        TabContent::Inline(markdown) => Some(markdown.clone()),
        _ => None,
    };

    // Always call hooks, but conditionally execute based on Option
    use_markdown_file_loader_conditional(file_opt.clone(), html, reload_trigger);
    use_file_watcher_conditional(file_opt.clone(), reload_trigger);
    use_link_click_handler_conditional(file_opt, state.clone());
    use_inline_markdown_loader_conditional(inline_opt, html);

    // Warn if None content
    if matches!(content, TabContent::None) {
        tracing::warn!("MarkdownViewer called with TabContent::None");
    }

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

/// Hook to load and render markdown file content (conditional version)
fn use_markdown_file_loader_conditional(
    file_opt: Option<PathBuf>,
    html: Signal<String>,
    reload_trigger: Signal<usize>,
) {
    use_effect(use_reactive!(|file_opt, reload_trigger| {
        let mut html = html;
        let _ = reload_trigger();

        if let Some(file) = file_opt {
            spawn(async move {
                tracing::info!("Loading and rendering file: {:?}", &file);

                match tokio::fs::read_to_string(file.as_path()).await {
                    Ok(content) => {
                        let rendered = render_to_html(&content, &file).unwrap_or_else(|e| {
                            tracing::error!("Failed to render markdown: {}", e);
                            format!(r#"<p class="error">Error rendering markdown: {:?}</p>"#, e)
                        });
                        html.set(rendered);
                        tracing::trace!("Rendered HTML: {}", html);
                    }
                    Err(e) => {
                        tracing::error!("Failed to read file {:?}: {}", file, e);
                        html.set(format!(
                            r#"<p class="error">Error reading file: {:?}</p>"#,
                            e
                        ));
                    }
                }
            });
        }
    }));
}

/// Hook to render inline markdown content (conditional version)
fn use_inline_markdown_loader_conditional(markdown_opt: Option<String>, html: Signal<String>) {
    use_effect(use_reactive!(|markdown_opt| {
        let mut html = html;
        if let Some(markdown) = markdown_opt {
            spawn(async move {
                // Render inline markdown (use a dummy path since images are already embedded)
                let rendered = render_to_html(&markdown, Path::new(".")).unwrap_or_else(|e| {
                    tracing::error!("Failed to render inline markdown: {}", e);
                    format!(r#"<p class="error">Error rendering markdown: {}</p>"#, e)
                });
                html.set(rendered);
            });
        }
    }));
}

/// Hook to watch file for changes and trigger reload (conditional version)
fn use_file_watcher_conditional(file_opt: Option<PathBuf>, reload_trigger: Signal<usize>) {
    use_effect(use_reactive!(|file_opt| {
        let mut reload_trigger = reload_trigger;

        if let Some(file) = file_opt {
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
        }
    }));
}

/// Hook to setup JavaScript handler for markdown link clicks (conditional version)
fn use_link_click_handler_conditional(file_opt: Option<PathBuf>, state: AppState) {
    use_effect(move || {
        let file_opt = file_opt.clone();
        if let Some(file) = file_opt {
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
        }
    });
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
