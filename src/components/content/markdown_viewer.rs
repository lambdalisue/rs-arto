use dioxus::prelude::*;
use std::path::PathBuf;

use crate::markdown::render_to_html;

#[component]
pub fn MarkdownViewer(file: ReadOnlySignal<PathBuf>) -> Element {
    let html = use_signal(String::new);

    // Read the file and render markdown to HTML when the component is mounted or when the file changes
    use_effect(use_reactive!(|file| {
        let mut html = html;
        spawn(async move {
            tracing::info!("Loading and rendering file: {:?}", &file);
            // Read the file content
            let file = file();
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
