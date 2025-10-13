use std::path::Path;

/// Check if a file path has a markdown extension (.md or .markdown)
pub fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "md" || ext == "markdown")
        .unwrap_or(false)
}
