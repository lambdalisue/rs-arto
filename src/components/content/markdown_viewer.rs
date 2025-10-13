use dioxus::prelude::*;
use std::path::PathBuf;

use crate::assets::MAIN_SCRIPT;
use crate::markdown::render_to_html;
use crate::state::AppState;

#[component]
pub fn MarkdownViewer(file: PathBuf) -> Element {
    let mut state = use_context::<AppState>();
    let html = use_signal(String::new);

    // Load the main script once when the component is mounted
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

    // Read the file and render markdown to HTML when the component is mounted or when the file changes
    use_effect(use_reactive!(|file| {
        let mut html = html;
        spawn(async move {
            tracing::info!("Loading and rendering file: {:?}", &file);
            // Read the file content
            match tokio::fs::read_to_string(file.as_path()).await {
                Ok(content) => {
                    html.set(render_to_html(&content, &file).unwrap_or_else(|e| {
                        tracing::error!("Failed to render markdown: {}", e);
                        format!(r#"<p class="error">Error rendering markdown: {:?}</p>"#, e)
                    }));
                    // Update the state with the rendered HTML
                    // This is a placeholder, implement state management as needed
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
    }));

    // Listen for markdown-link-click events from JavaScript
    use_effect(move || {
        let mut eval_provider = document::eval(indoc::indoc! {r#"
            window.handleMarkdownLinkClick = (path) => {
                dioxus.send(path);
            };
        "#});
        // Get the current file's directory
        let base_dir = file
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        spawn(async move {
            loop {
                if let Ok(response) = eval_provider.recv::<String>().await {
                    tracing::info!("Markdown link clicked: {}", response);

                    // Resolve the relative path
                    let target_path = base_dir.join(&response);

                    // Normalize the path
                    match target_path.canonicalize() {
                        Ok(canonical_path) => {
                            tracing::info!("Opening file: {:?}", canonical_path);
                            // Update history and file state
                            state.history.write().push(canonical_path.clone());
                            state.file.set(Some(canonical_path));
                        }
                        Err(e) => {
                            tracing::error!("Failed to resolve path {:?}: {}", target_path, e);
                        }
                    }
                }
            }
        });
    });

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
