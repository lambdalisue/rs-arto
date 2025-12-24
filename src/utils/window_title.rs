use crate::state::TabContent;

/// Generate window title based on active tab content
pub fn generate_window_title(tab_content: &TabContent) -> String {
    match tab_content {
        TabContent::File(path) => {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            format!("Arto - {}", filename)
        }
        TabContent::Inline(_) => "Arto - Welcome".to_string(),
        TabContent::Preferences => "Arto - Preferences".to_string(),
        TabContent::FileError(path, _) => {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            format!("Arto - {} (Error)", filename)
        }
        TabContent::None => "Arto".to_string(),
    }
}
