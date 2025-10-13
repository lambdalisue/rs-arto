use dioxus::prelude::*;

use crate::assets::MAIN_SCRIPT;
use crate::components::icon::{Icon, IconName};
use crate::state::{AppState, LAST_SELECTED_THEME};
use crate::theme::use_resolved_theme;
use crate::theme::ThemePreference;

#[component]
pub fn ThemeSelector() -> Element {
    let state = use_context::<AppState>();
    let resolved_theme = use_resolved_theme();
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
    use_effect(use_reactive!(|resolved_theme| {
        let resolved = resolved_theme();
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

    use_effect(use_reactive!(|current_theme| {
        let theme = current_theme();
        spawn(async move {
            // Save the last selected theme to the global state
            let mut last_theme = LAST_SELECTED_THEME.lock().unwrap();
            *last_theme = theme;
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
