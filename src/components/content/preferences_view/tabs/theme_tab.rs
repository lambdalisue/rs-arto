use super::super::form_controls::{OptionCardItem, OptionCards};
use crate::components::icon::IconName;
use crate::config::{Config, NewWindowBehavior, StartupBehavior};
use crate::theme::ThemePreference;
use dioxus::prelude::*;

#[component]
pub fn ThemeTab(config: Signal<Config>, has_changes: Signal<bool>) -> Element {
    // Extract values upfront to avoid holding read guard across closures
    let (default_theme, on_startup, on_new_window) = {
        let cfg = config.read();
        (
            cfg.theme.default_theme,
            cfg.theme.on_startup,
            cfg.theme.on_new_window,
        )
    };

    rsx! {
        div {
            class: "preferences-pane",

            h3 { class: "preference-section-title", "Default Settings" }

            div {
                class: "preference-item",
                div {
                    class: "preference-item-header",
                    label { "Default Theme" }
                    p { class: "preference-description", "The color theme used by default." }
                }
                OptionCards {
                    name: "theme-default".to_string(),
                    options: vec![
                        OptionCardItem {
                            value: ThemePreference::Auto,
                            icon: Some(IconName::Contrast2),
                            title: "Auto".to_string(),
                            description: None,
                        },
                        OptionCardItem {
                            value: ThemePreference::Light,
                            icon: Some(IconName::Sun),
                            title: "Light".to_string(),
                            description: None,
                        },
                        OptionCardItem {
                            value: ThemePreference::Dark,
                            icon: Some(IconName::Moon),
                            title: "Dark".to_string(),
                            description: None,
                        },
                    ],
                    selected: default_theme,
                    on_change: move |new_theme| {
                        config.write().theme.default_theme = new_theme;
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
                    p { class: "preference-description", "Which theme to apply when the application starts." }
                }
                OptionCards {
                    name: "theme-startup".to_string(),
                    options: vec![
                        OptionCardItem {
                            value: StartupBehavior::Default,
                            icon: None,
                            title: "Default".to_string(),
                            description: Some("Use default theme".to_string()),
                        },
                        OptionCardItem {
                            value: StartupBehavior::LastClosed,
                            icon: None,
                            title: "Last Closed".to_string(),
                            description: Some("Resume from last closed window".to_string()),
                        },
                    ],
                    selected: on_startup,
                    on_change: move |new_behavior| {
                        config.write().theme.on_startup = new_behavior;
                        has_changes.set(true);
                    },
                }
            }

            div {
                class: "preference-item",
                div {
                    class: "preference-item-header",
                    label { "On New Window" }
                    p { class: "preference-description", "Which theme to apply in new windows." }
                }
                OptionCards {
                    name: "theme-new-window".to_string(),
                    options: vec![
                        OptionCardItem {
                            value: NewWindowBehavior::Default,
                            icon: None,
                            title: "Default".to_string(),
                            description: Some("Use default theme".to_string()),
                        },
                        OptionCardItem {
                            value: NewWindowBehavior::LastFocused,
                            icon: None,
                            title: "Last Focused".to_string(),
                            description: Some("Same as current window".to_string()),
                        },
                    ],
                    selected: on_new_window,
                    on_change: move |new_behavior| {
                        config.write().theme.on_new_window = new_behavior;
                        has_changes.set(true);
                    },
                }
            }
        }
    }
}
