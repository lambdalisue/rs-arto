use crate::theme::{resolve_theme, ThemePreference};

pub fn build_custom_index(theme_preference: ThemePreference) -> String {
    let theme = resolve_theme(theme_preference);
    indoc::formatdoc! {r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Arto</title>
            <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
            <!-- CUSTOM HEAD -->
        </head>
        <body data-theme="{theme}">
            <div id="main"></div>
            <!-- MODULE LOADER -->
        </body>
    </html>
    "#}
}

pub(crate) fn build_mermaid_window_index(theme: ThemePreference) -> String {
    let resolved_theme = resolve_theme(theme);
    indoc::formatdoc! {r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Mermaid Viewer - Arto</title>
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <!-- CUSTOM HEAD -->
        </head>
        <body data-theme="{resolved_theme}" class="mermaid-window-body">
            <div id="main"></div>
            <!-- MODULE LOADER -->
        </body>
    </html>
    "#}
}
