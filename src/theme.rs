use dioxus_sdk_window::theme::get_theme;

pub use dioxus_sdk_window::theme::Theme;

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    Auto,
    Light,
    Dark,
}

impl From<&str> for ThemePreference {
    fn from(s: &str) -> Self {
        match s {
            "light" => ThemePreference::Light,
            "dark" => ThemePreference::Dark,
            _ => ThemePreference::Auto,
        }
    }
}

pub fn resolve_theme(theme: &ThemePreference) -> Theme {
    match theme {
        ThemePreference::Auto => get_theme().unwrap_or(Theme::Light),
        ThemePreference::Light => Theme::Light,
        ThemePreference::Dark => Theme::Dark,
    }
}
