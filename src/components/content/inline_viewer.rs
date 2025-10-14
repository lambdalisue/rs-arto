use dioxus::prelude::*;
use std::path::Path;

use crate::assets::MAIN_SCRIPT;
use crate::markdown::render_to_html;

#[component]
pub fn InlineViewer(markdown: String) -> Element {
    let html = use_signal(String::new);

    // Setup component hooks
    use_main_script_loader();
    use_inline_markdown_loader(markdown, html);

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

/// Hook to render inline markdown content
fn use_inline_markdown_loader(markdown: String, html: Signal<String>) {
    use_effect(move || {
        let mut html = html;
        let markdown = markdown.clone();

        spawn(async move {
            // Render inline markdown (use a dummy path since images are already embedded)
            let rendered = render_to_html(&markdown, Path::new(".")).unwrap_or_else(|e| {
                tracing::error!("Failed to render inline markdown: {}", e);
                format!(r#"<p class="error">Error rendering markdown: {}</p>"#, e)
            });
            html.set(rendered);
        });
    });
}
