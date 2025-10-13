use dioxus::prelude::*;
use dioxus_desktop::use_muda_event_handler;
use std::path::PathBuf;

use super::content::Content;
use super::header::Header;
use crate::assets::MAIN_SCRIPT;
use crate::menu;
use crate::state::AppState;

#[component]
pub fn App(file: Option<PathBuf>) -> Element {
    let file_signal = Signal::new(file.clone());
    let mut state = use_context_provider(|| AppState {
        file: file_signal,
        ..Default::default()
    });

    // Add initial file to history if provided
    if let Some(path) = file {
        state.history.write().push(path);
    }

    // Handle menu events (only state-dependent events, not global ones)
    use_muda_event_handler(move |event| {
        // Only handle state-dependent events
        menu::handle_menu_event_with_state(event, &mut state);
    });

    rsx! {
        document::Script { r#type: "module", r#"
            import {{init}} from '{MAIN_SCRIPT}';
            if (document.readyState === "loading") {{
                document.addEventListener("DOMContentLoaded", init);
            }} else {{
                init();
            }}
        "#}
        div {
            class: "app-container",

            Header {},
            Content {},
        }
    }
}
