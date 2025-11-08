use dioxus::prelude::*;
use dioxus_sdk_window::theme::use_system_theme;

use crate::components::icon::{Icon, IconName};
use crate::state::LAST_SELECTED_THEME;
use crate::theme::{Theme, ThemePreference};

#[component]
pub fn ThemeSelector(current_theme: Signal<ThemePreference>) -> Element {
    let system_theme = use_system_theme();
    let resolved_theme = use_memo(move || match current_theme() {
        ThemePreference::Auto => system_theme().unwrap_or(Theme::Light),
        ThemePreference::Light => Theme::Light,
        ThemePreference::Dark => Theme::Dark,
    });

    // Dispatch custom event when resolved theme changes
    use_effect(use_reactive!(|resolved_theme| {
        let theme = resolved_theme();
        let theme_str = match theme {
            Theme::Light => "light",
            Theme::Dark => "dark",
        };
        spawn(async move {
            let _ = document::eval(&format!(
                "document.dispatchEvent(new CustomEvent('arto:theme-changed', {{ detail: '{}' }}))",
                theme_str
            ))
            .await;
        });
    }));

    // Save last selected theme
    use_effect(use_reactive!(|current_theme| {
        let theme = current_theme();
        spawn(async move {
            *LAST_SELECTED_THEME.lock().unwrap() = theme;
        });
    }));

    let (light_class, dark_class, auto_class) = match current_theme() {
        ThemePreference::Light => (
            "theme-button theme-button--light theme-button--active",
            "theme-button theme-button--dark",
            "theme-button theme-button--auto",
        ),
        ThemePreference::Dark => (
            "theme-button theme-button--light",
            "theme-button theme-button--dark theme-button--active",
            "theme-button theme-button--auto",
        ),
        ThemePreference::Auto => (
            "theme-button theme-button--light",
            "theme-button theme-button--dark",
            "theme-button theme-button--auto theme-button--active",
        ),
    };

    rsx! {
        div {
            class: "theme-selector",

            button {
                class: light_class,
                "aria-pressed": if current_theme() == ThemePreference::Light { "true" } else { "false" },
                onclick: move |_| {
                    let mut current_theme = current_theme;
                    current_theme.set(ThemePreference::Light);
                },
                title: "Light theme",
                Icon { name: IconName::Sun, size: 18 }
            }

            button {
                class: dark_class,
                "aria-pressed": if current_theme() == ThemePreference::Dark { "true" } else { "false" },
                onclick: move |_| {
                    let mut current_theme = current_theme;
                    current_theme.set(ThemePreference::Dark);
                },
                title: "Dark theme",
                Icon { name: IconName::Moon, size: 18 }
            }

            button {
                class: auto_class,
                "aria-pressed": if current_theme() == ThemePreference::Auto { "true" } else { "false" },
                onclick: move |_| {
                    let mut current_theme = current_theme;
                    current_theme.set(ThemePreference::Auto);
                },
                title: "Auto theme (follows system)",
                Icon { name: IconName::Contrast2, size: 18 }
            }
        }
    }
}
