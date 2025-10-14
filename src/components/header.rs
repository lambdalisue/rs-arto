mod theme_selector;

use dioxus::prelude::*;

use crate::components::icon::{Icon, IconName};
use crate::state::AppState;
use theme_selector::ThemeSelector;

#[component]
pub fn Header() -> Element {
    let state = use_context::<AppState>();

    let current_tab = state.current_tab();
    let file = current_tab
        .as_ref()
        .and_then(|tab| tab.file())
        .map(|f| {
            f.file_name()
                .unwrap_or(f.as_os_str())
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|| "No file opened".to_string());

    let can_go_back = current_tab
        .as_ref()
        .is_some_and(|tab| tab.history.can_go_back());
    let can_go_forward = current_tab
        .as_ref()
        .is_some_and(|tab| tab.history.can_go_forward());

    let is_sidebar_visible = state.sidebar.read().is_visible;

    let mut state_for_back = state.clone();
    let mut state_for_forward = state.clone();
    let mut state_for_sidebar = state.clone();

    let on_back = move |_| {
        state_for_back.update_current_tab(|tab| {
            if let Some(path) = tab.history.go_back() {
                tab.content = crate::state::TabContent::File(path);
            }
        });
    };

    let on_forward = move |_| {
        state_for_forward.update_current_tab(|tab| {
            if let Some(path) = tab.history.go_forward() {
                tab.content = crate::state::TabContent::File(path);
            }
        });
    };

    rsx! {
        div {
            class: "header",

            // File name display (left side) with navigation buttons
            div {
                class: "header-left",

                // Sidebar toggle button
                button {
                    class: "sidebar-toggle-button",
                    class: if is_sidebar_visible { "active" },
                    onclick: move |_| {
                        state_for_sidebar.toggle_sidebar();
                    },
                    Icon {
                        name: IconName::Sidebar,
                        size: 20,
                    }
                }

                // Back button
                button {
                    class: "nav-button",
                    disabled: !can_go_back,
                    onclick: on_back,
                    Icon { name: IconName::ChevronLeft }
                }

                // Forward button
                button {
                    class: "nav-button",
                    disabled: !can_go_forward,
                    onclick: on_forward,
                    Icon { name: IconName::ChevronRight }
                }

                // File name
                span {
                    class: "file-name",
                    "{file}"
                }
            }

            // Theme selector (right side)
            ThemeSelector {}
        }
    }
}
