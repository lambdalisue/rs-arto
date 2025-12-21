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

pub fn resolve_theme(theme: ThemePreference) -> Theme {
    match theme {
        // NOTE:
        // We cannot use dioxus_sdk_window::theme::get_theme here because
        // it requires a Dioxus runtime and cannot be called from outside
        // of Dioxus context. That's why we use dark_light crate instead.
        ThemePreference::Auto => match dark_light::detect() {
            Ok(dark_light::Mode::Light) => Theme::Light,
            Ok(dark_light::Mode::Dark) => Theme::Dark,
            Ok(dark_light::Mode::Unspecified) | Err(_) => Theme::Light,
        },
        ThemePreference::Light => Theme::Light,
        ThemePreference::Dark => Theme::Dark,
    }
}
