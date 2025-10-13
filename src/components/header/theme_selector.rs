use dioxus::prelude::*;

use crate::assets::MAIN_SCRIPT;
use crate::components::icon::{Icon, IconName};
use crate::state::AppState;
use crate::theme::ThemePreference;

#[component]
pub fn ThemeSelector() -> Element {
    let state = use_context::<AppState>();
    let current_theme = state.current_theme;
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

    // Set the current theme in JavaScript whenever it changes
    use_effect(use_reactive!(|current_theme| {
        let resolved = match current_theme() {
            ThemePreference::Light => "light".to_string(),
            ThemePreference::Dark => "dark".to_string(),
            ThemePreference::Auto => "auto".to_string(),
        };
        spawn(async move {
            let eval = document::eval(&indoc::formatdoc! {r#"
                const {{ setCurrentTheme }} = await import('{MAIN_SCRIPT}');
                setCurrentTheme('{resolved}');
            "#});
            if let Err(err) = eval.await {
                tracing::error!("Failed to set theme in JS: {err}");
            }
        });
    }));

    // Get the current theme from JavaScript on initialization
    use_effect(move || {
        let mut current_theme = current_theme;
        spawn(async move {
            let eval = document::eval(&indoc::formatdoc! {r#"
                const {{ getCurrentTheme }} = await import('{MAIN_SCRIPT}');
                return getCurrentTheme();
            "#});
            let theme = match eval.await {
                Ok(value) => match value.as_str() {
                    Some(theme_str) => match theme_str {
                        "light" => ThemePreference::Light,
                        "dark" => ThemePreference::Dark,
                        _ => ThemePreference::Auto,
                    },
                    _ => {
                        tracing::error!("Unexpected value from JS");
                        ThemePreference::Auto
                    }
                },
                Err(err) => {
                    tracing::error!("Failed to get current theme from JS: {err}");
                    ThemePreference::Auto
                }
            };
            current_theme.set(theme);
        });
    });

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
