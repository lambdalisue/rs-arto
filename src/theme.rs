#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ThemePreference {
    #[default]
    Auto,
    Light,
    Dark,
}

impl ThemePreference {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemePreference::Auto => "auto",
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        }
    }
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
