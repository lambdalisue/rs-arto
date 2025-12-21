use super::super::form_controls::{OptionCardItem, OptionCards, SliderInput};
use crate::config::{Config, NewWindowBehavior, StartupBehavior};
use dioxus::prelude::*;

#[component]
pub fn SidebarTab(
    config: Signal<Config>,
    has_changes: Signal<bool>,
    current_sidebar_width: f64,
) -> Element {
    // Extract values upfront to avoid holding read guard across closures
    let sidebar = config.read().sidebar.clone();

    rsx! {
        div {
            class: "preferences-pane",

            h3 { class: "preference-section-title", "Default Settings" }

            div {
                class: "preference-item",
                div {
                    class: "preference-item-header",
                    label { "Open by Default" }
                    p { class: "preference-description", "Whether the sidebar is open when starting." }
                }
                OptionCards {
                    name: "sidebar-default-open".to_string(),
                    options: vec![
                        OptionCardItem {
                            icon: None,
                            value: false,
                            title: "Closed".to_string(),
                            description: Some("Sidebar closed by default".to_string()),
                        },
                        OptionCardItem {
                            icon: None,
                            value: true,
                            title: "Open".to_string(),
                            description: Some("Sidebar open by default".to_string()),
                        },
                    ],
                    selected: sidebar.default_open,
                    on_change: move |new_state| {
                        config.write().sidebar.default_open = new_state;
                        has_changes.set(true);
                    },
                }
            }

            div {
                class: "preference-item",
                div {
                    class: "preference-item-header",
                    label { "Default Width" }
                    p { class: "preference-description", "The default sidebar width in pixels." }
                }
                SliderInput {
                    value: sidebar.default_width,
                    min: 200.0,
                    max: 600.0,
                    step: 10.0,
                    unit: "px".to_string(),
                    on_change: move |new_width| {
                        config.write().sidebar.default_width = new_width;
                        has_changes.set(true);
                    },
                    current_value: Some(current_sidebar_width),
                }
            }

            div {
                class: "preference-item",
                div {
                    class: "preference-item-header",
                    label { "Show All Files" }
                    p { class: "preference-description", "Whether to show non-markdown files in the file explorer." }
                }
                OptionCards {
                    name: "sidebar-show-all-files".to_string(),
                    options: vec![
                        OptionCardItem {
                            icon: None,
                            value: false,
                            title: "Markdown Only".to_string(),
                            description: Some("Show only markdown files".to_string()),
                        },
                        OptionCardItem {
                            icon: None,
                            value: true,
                            title: "All Files".to_string(),
                            description: Some("Show all file types".to_string()),
                        },
                    ],
                    selected: sidebar.default_show_all_files,
                    on_change: move |new_state| {
                        config.write().sidebar.default_show_all_files = new_state;
                        has_changes.set(true);
                    },
                }
            }

            h3 { class: "preference-section-title", "Behavior" }

            div {
                class: "preference-item",
                div {
                    class: "preference-item-header",
                    label { "On Startup" }
                    p { class: "preference-description", "Sidebar state when the application starts." }
                }
                OptionCards {
                    name: "sidebar-startup".to_string(),
                    options: vec![
                        OptionCardItem {
                            icon: None,
                            value: StartupBehavior::Default,
                            title: "Default".to_string(),
                            description: Some("Use default settings".to_string()),
                        },
                        OptionCardItem {
                            icon: None,
                            value: StartupBehavior::LastClosed,
                            title: "Last Closed".to_string(),
                            description: Some("Resume from last closed window".to_string()),
                        },
                    ],
                    selected: sidebar.on_startup,
                    on_change: move |new_behavior| {
                        config.write().sidebar.on_startup = new_behavior;
                        has_changes.set(true);
                    },
                }
            }

            div {
                class: "preference-item",
                div {
                    class: "preference-item-header",
                    label { "On New Window" }
                    p { class: "preference-description", "Sidebar state in new windows." }
                }
                OptionCards {
                    name: "sidebar-new-window".to_string(),
                    options: vec![
                        OptionCardItem {
                            icon: None,
                            value: NewWindowBehavior::Default,
                            title: "Default".to_string(),
                            description: Some("Use default settings".to_string()),
                        },
                        OptionCardItem {
                            icon: None,
                            value: NewWindowBehavior::LastFocused,
                            title: "Last Focused".to_string(),
                            description: Some("Same as current window".to_string()),
                        },
                    ],
                    selected: sidebar.on_new_window,
                    on_change: move |new_behavior| {
                        config.write().sidebar.on_new_window = new_behavior;
                        has_changes.set(true);
                    },
                }
            }
        }
    }
}
