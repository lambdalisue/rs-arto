use dioxus::prelude::*;
use sha2::{Digest, Sha256};

use crate::assets::MAIN_SCRIPT;
use crate::components::theme_selector::ThemeSelector;
use crate::theme::Theme;

/// Props for MermaidWindow component
#[derive(Props, Clone, PartialEq)]
pub struct MermaidWindowProps {
    /// Mermaid source code
    pub source: String,
    /// Unique diagram identifier (hash)
    pub diagram_id: String,
    /// Initial theme
    pub theme: Theme,
}

/// Generate unique ID from Mermaid source
pub fn generate_diagram_id(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let result = hasher.finalize();
    // Use first 16 characters of hex hash
    format!("{:x}", result)[..16].to_string()
}

/// Mermaid Window Component
#[component]
pub fn MermaidWindow(props: MermaidWindowProps) -> Element {
    let current_theme = use_signal(|| props.theme);
    let zoom_level = use_signal(|| 100);

    // Load viewer script on mount
    use_viewer_script_loader(props.source.clone(), props.diagram_id.clone());

    // Setup zoom update handler
    use_zoom_update_handler(zoom_level);

    rsx! {
        div {
            class: "mermaid-window-container",

            // Header with controls
            div {
                class: "mermaid-window-header",

                // Empty spacer on left
                div {
                    class: "mermaid-window-title",
                }

                div {
                    class: "mermaid-window-controls",
                    ThemeSelector { current_theme }
                }
            }

            // Canvas container for diagram
            div {
                id: "mermaid-window-canvas",
                class: "mermaid-window-canvas",

                // Wrapper for positioning (translate)
                div {
                    id: "mermaid-diagram-wrapper",
                    class: "mermaid-diagram-wrapper",

                    // Inner container for zoom
                    div {
                        id: "mermaid-diagram-container",
                        class: "mermaid-diagram-container",
                        // Placeholder for Mermaid SVG
                    }
                }
            }

            // Status bar
            div {
                class: "mermaid-window-status",
                "Zoom: {zoom_level}% | Scroll to zoom, drag to pan, double-click to fit"
            }
        }
    }
}

/// Hook to load viewer script and initialize
fn use_viewer_script_loader(source: String, diagram_id: String) {
    use_effect(move || {
        let source = source.clone();
        let diagram_id = diagram_id.clone();

        spawn(async move {
            // Escape source for JavaScript (handle backticks, backslashes, quotes)
            let escaped_source = source
                .replace('\\', "\\\\")
                .replace('`', "\\`")
                .replace('$', "\\$");

            let eval_result = document::eval(&indoc::formatdoc! {r#"
                (async () => {{
                    try {{
                        const {{ initMermaidWindow }} = await import("{MAIN_SCRIPT}");
                        await initMermaidWindow(`{escaped_source}`, '{diagram_id}');
                    }} catch (error) {{
                        console.error("Failed to load mermaid window module:", error);
                    }}
                }})();
            "#});

            if let Err(e) = eval_result.await {
                tracing::error!("Failed to initialize mermaid window: {}", e);
            }
        });
    });
}

/// Hook to listen for zoom updates from JavaScript
fn use_zoom_update_handler(zoom_level: Signal<i32>) {
    use_effect(move || {
        let mut zoom_level = zoom_level;

        spawn(async move {
            let mut eval_provider = document::eval(indoc::indoc! {r#"
                window.updateZoomLevel = (zoom) => {
                    dioxus.send({ zoom: Math.round(zoom) });
                };
            "#});

            while let Ok(data) = eval_provider.recv::<serde_json::Value>().await {
                if let Some(zoom) = data.get("zoom").and_then(|v| v.as_i64()) {
                    zoom_level.set(zoom as i32);
                }
            }
        });
    });
}
