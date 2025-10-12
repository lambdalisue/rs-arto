use dioxus::prelude::*;
use dioxus_sdk::theme::{use_system_theme, SystemTheme};

use crate::state::AppState;
use crate::theme::ThemePreference;

const FALLBACK_THEME_STR: &str = "light";

#[component]
pub fn ThemeSelector() -> Element {
    let state = use_context::<AppState>();
    let current_theme = state.current_theme;
    let system_theme = use_system_theme();

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

    // Get the current theme from JavaScript on initialization
    use_effect({
        let mut current_theme = current_theme;
        move || {
            spawn(async move {
                let result = document::eval("window.getCurrentMarkdownTheme()").await;

                if let Ok(value) = result {
                    if let Some(theme_str) = value.as_str() {
                        let theme = match theme_str {
                            "light" => ThemePreference::Light,
                            "dark" => ThemePreference::Dark,
                            _ => ThemePreference::Auto,
                        };
                        current_theme.set(theme);
                    }
                }
            });
        }
    });

    use_effect(use_reactive!(|current_theme, system_theme| {
        let preference = current_theme();
        let pref_str = preference.as_str().to_string();
        let resolved = match preference {
            ThemePreference::Light => "light".to_string(),
            ThemePreference::Dark => "dark".to_string(),
            ThemePreference::Auto => match system_theme() {
                Ok(SystemTheme::Dark) => "dark".to_string(),
                Ok(SystemTheme::Light) => "light".to_string(),
                Err(_) => FALLBACK_THEME_STR.to_string(),
            },
        };

        spawn(async move {
            let script = format!(
                "window.setMarkdownTheme('{pref}'); window.applyMarkdownResolvedTheme('{resolved}');",
                pref = pref_str,
                resolved = resolved,
            );
            let _ = document::eval(&script).await;
        });
    }));

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
                "☀️ Light"
            }

            button {
                class: dark_class,
                "aria-pressed": if current_theme() == ThemePreference::Dark { "true" } else { "false" },
                onclick: move |_| {
                    let mut current_theme = current_theme;
                    current_theme.set(ThemePreference::Dark);
                },
                title: "Dark theme",
                "🌙 Dark"
            }

            button {
                class: auto_class,
                "aria-pressed": if current_theme() == ThemePreference::Auto { "true" } else { "false" },
                onclick: move |_| {
                    let mut current_theme = current_theme;
                    current_theme.set(ThemePreference::Auto);
                },
                title: "Auto theme (follows system)",
                "🔄 Auto"
            }
        }
    }
}
