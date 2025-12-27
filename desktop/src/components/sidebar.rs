pub mod file_explorer;

use dioxus::document;
use dioxus::prelude::*;

use crate::state::{AppState, LAST_FOCUSED_STATE};

#[component]
pub fn Sidebar() -> Element {
    let mut state = use_context::<AppState>();
    let sidebar_state = state.sidebar.read();
    let is_visible = sidebar_state.open;
    let width = sidebar_state.width;

    let mut is_resizing = use_signal(|| false);

    // Clamp initial width to window size on mount
    use_effect(move || {
        spawn(async move {
            // Get the max width based on current window size
            let result = document::eval(r#"window.innerWidth * 0.7"#)
                .await
                .ok()
                .and_then(|v| v.as_f64());

            if let Some(max_width) = result {
                let current_width = state.sidebar.read().width;
                if current_width > max_width {
                    let clamped = current_width.clamp(200.0, max_width);
                    state.sidebar.write().width = clamped;
                    LAST_FOCUSED_STATE.write().sidebar_width = clamped;
                }
            }
        });
    });

    let style = if is_visible {
        format!("width: {}px;", width)
    } else {
        "width: 0;".to_string()
    };

    rsx! {
        div {
            class: "sidebar",
            class: if is_visible { "visible" },
            class: if is_resizing() { "resizing" },
            style: "{style}",

            // File explorer content (always mounted for animation)
            file_explorer::FileExplorer {}

            // Resize handle (only when visible)
            if is_visible {
                div {
                    class: "sidebar-resize-handle",
                    class: if is_resizing() { "resizing" },
                    onmousedown: move |evt| {
                        evt.prevent_default();
                        is_resizing.set(true);
                        let start_x = evt.page_coordinates().x;
                        let start_width = state.sidebar.read().width;

                        spawn(async move {
                            #[derive(serde::Deserialize)]
                            struct DragMessage {
                                r#type: String,
                                x: Option<f64>,
                                #[serde(rename = "maxWidth")]
                                max_width: Option<f64>,
                            }

                            let mut eval = document::eval(r#"
                                new Promise((resolve) => {
                                    const handleMouseMove = (e) => {
                                        const maxWidth = window.innerWidth * 0.7;
                                        dioxus.send({ type: 'move', x: e.pageX, maxWidth });
                                    };
                                    const handleMouseUp = () => {
                                        document.removeEventListener('mousemove', handleMouseMove);
                                        document.removeEventListener('mouseup', handleMouseUp);
                                        dioxus.send({ type: 'end' });
                                        resolve();
                                    };
                                    document.addEventListener('mousemove', handleMouseMove);
                                    document.addEventListener('mouseup', handleMouseUp);
                                })
                            "#);

                            while let Ok(msg) = eval.recv::<DragMessage>().await {
                                match msg.r#type.as_str() {
                                    "move" => {
                                        if let Some(x) = msg.x {
                                            let delta = x - start_x;
                                            let max_width = msg.max_width.unwrap_or(600.0);
                                            let new_width = (start_width + delta).clamp(200.0, max_width);
                                            state.sidebar.write().width = new_width;
                                        }
                                    }
                                    "end" => {
                                        // Update last focused sidebar width when resize ends
                                        let final_width = state.sidebar.read().width;
                                        LAST_FOCUSED_STATE.write().sidebar_width = final_width;
                                        is_resizing.set(false);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                }
            }
        }
    }
}
