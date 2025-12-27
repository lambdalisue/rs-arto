use crate::theme::{resolve_theme, Theme};

pub fn build_custom_index(theme: Theme) -> String {
    let resolved = resolve_theme(theme);
    indoc::formatdoc! {r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Arto</title>
            <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
            <!-- CUSTOM HEAD -->
        </head>
        <body data-theme="{resolved}">
            <div id="main"></div>
            <!-- MODULE LOADER -->
        </body>
    </html>
    "#}
}

pub(crate) fn build_mermaid_window_index(theme: Theme) -> String {
    let resolved = resolve_theme(theme);
    indoc::formatdoc! {r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Mermaid Viewer - Arto</title>
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <!-- CUSTOM HEAD -->
        </head>
        <body data-theme="{resolved}" class="mermaid-window-body">
            <div id="main"></div>
            <!-- MODULE LOADER -->
        </body>
    </html>
    "#}
}
