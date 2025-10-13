use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use std::path::{Path, PathBuf};

/// Render Markdown to HTML
pub fn render_to_html(markdown: &str, base_path: &Path) -> Result<String> {
    // Enable GitHub Flavored Markdown options
    let options = Options::all();

    // Get base directory for resolving relative paths
    let base_dir = base_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    // Process GitHub alerts
    let processed_markdown = process_github_alerts(markdown);

    // Parse Markdown and process blocks
    let parser = Parser::new_ext(&processed_markdown, options);
    let parser = process_mermaid_blocks(parser);
    let parser = process_image_paths(parser, base_dir.as_path());
    let parser = process_anchor_markdown_files(parser);

    // Convert to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}

/// Get SVG icon placeholder for alert type (actual SVG injected by JavaScript)
fn get_alert_icon_placeholder(alert_type: &str) -> String {
    format!(
        r#"<span class="alert-icon" data-alert-type="{}"></span>"#,
        alert_type
    )
}

/// Check if a line starts a GitHub alert and return alert info
fn parse_alert_start(line: &str) -> Option<(&'static str, &'static str, &str)> {
    const ALERT_TYPES: [(&str, &str); 5] = [
        ("NOTE", "note"),
        ("TIP", "tip"),
        ("IMPORTANT", "important"),
        ("WARNING", "warning"),
        ("CAUTION", "caution"),
    ];

    for &(alert_name, alert_class) in &ALERT_TYPES {
        if let Some(rest) = line.strip_prefix(&format!("> [!{}]", alert_name)) {
            return Some((alert_name, alert_class, rest));
        }
    }
    None
}

/// Process a single alert block and return HTML lines and next index
fn process_alert_block(
    lines: &[&str],
    start_index: usize,
    alert_name: &str,
    alert_class: &str,
    first_line_content: &str,
) -> (Vec<String>, usize) {
    let mut html_lines = Vec::new();

    // Alert opening tag
    html_lines.push(format!(
        r#"<div class="markdown-alert markdown-alert-{}" dir="auto">"#,
        alert_class
    ));

    // Alert title with icon
    let icon_placeholder = get_alert_icon_placeholder(alert_class);
    html_lines.push(format!(
        r#"<p class="markdown-alert-title" dir="auto">{}{}</p>"#,
        icon_placeholder, alert_name
    ));

    // First line content
    if !first_line_content.trim().is_empty() {
        html_lines.push(first_line_content.trim().to_string());
    }

    // Collect following quoted lines
    let mut i = start_index + 1;
    while i < lines.len() && lines[i].starts_with('>') {
        if let Some(content) = lines[i].strip_prefix('>') {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                html_lines.push(trimmed.to_string());
            }
        }
        i += 1;
    }

    html_lines.push("</div>".to_string());

    (html_lines, i)
}

/// Process GitHub alert format
fn process_github_alerts(markdown: &str) -> String {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some((alert_name, alert_class, rest)) = parse_alert_start(line) {
            let (alert_html, next_index) =
                process_alert_block(&lines, i, alert_name, alert_class, rest);
            result.extend(alert_html);
            i = next_index;
        } else {
            result.push(line.to_string());
            i += 1;
        }
    }

    result.join("\n")
}

/// Convert image paths from relative paths to Data URLs
fn process_image_paths<'a>(
    parser: impl Iterator<Item = Event<'a>>,
    base_dir: &'a Path,
) -> impl Iterator<Item = Event<'a>> {
    parser.map(move |event| {
        match event {
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                // Check if URL is a relative path (doesn't start with http:// or https://)
                let url_str = dest_url.as_ref();

                if !url_str.starts_with("http://")
                    && !url_str.starts_with("https://")
                    && !url_str.starts_with("data:")
                {
                    // Convert relative path to absolute path
                    let absolute_path = base_dir.join(url_str);

                    if let Ok(canonical_path) = absolute_path.canonicalize() {
                        // Read file and encode to Base64
                        if let Ok(image_data) = std::fs::read(&canonical_path) {
                            let mime_type = get_mime_type(&canonical_path);
                            let base64_data = general_purpose::STANDARD.encode(&image_data);
                            let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                            return Event::Start(Tag::Image {
                                link_type,
                                dest_url: CowStr::from(data_url),
                                title,
                                id,
                            });
                        }
                    }
                }
                // Return original event if conversion is not needed
                Event::Start(Tag::Image {
                    link_type,
                    dest_url,
                    title,
                    id,
                })
            }
            other => other,
        }
    })
}

/// Infer MIME type from file extension
fn get_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        _ => "image/png", // Default
    }
}

/// Process anchor tags pointing to Markdown files
/// Convert <a href="*.md"> to <span class="md-link"> with onclick handler
fn process_anchor_markdown_files<'a>(
    parser: impl Iterator<Item = Event<'a>>,
) -> impl Iterator<Item = Event<'a>> {
    let mut in_md_link = false;
    let mut link_url = String::new();

    parser.flat_map(move |event| match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let url_str = dest_url.as_ref();

            // Check if this is a Markdown file link (not http/https)
            if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
                if let Some(ext) = std::path::Path::new(url_str)
                    .extension()
                    .and_then(|e| e.to_str())
                {
                    if ext == "md" || ext == "markdown" {
                        // This is a Markdown file link, convert to span
                        in_md_link = true;
                        link_url = url_str.to_string();
                        let html = format!(
                            r#"<span class="md-link" onmousedown="if (event.button === 0 || event.button === 1) {{ event.preventDefault(); window.handleMarkdownLinkClick('{}', event.button); }}">"#,
                            url_str.replace('\'', "\\'")
                        );
                        return vec![Event::Html(html.into())];
                    }
                }
            }

            // Keep original link for non-Markdown files
            vec![Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            })]
        }
        Event::End(TagEnd::Link) if in_md_link => {
            in_md_link = false;
            link_url.clear();
            vec![Event::Html("</span>".into())]
        }
        _ => vec![event],
    })
}

/// Process Mermaid code blocks
fn process_mermaid_blocks<'a>(parser: Parser<'a>) -> impl Iterator<Item = Event<'a>> {
    let mut in_mermaid = false;
    let mut mermaid_content = String::new();

    parser.flat_map(move |event| match event {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) if lang.as_ref() == "mermaid" => {
            in_mermaid = true;
            mermaid_content.clear();
            vec![]
        }
        Event::End(TagEnd::CodeBlock) if in_mermaid => {
            in_mermaid = false;
            // Don't HTML-escape Mermaid content - it needs raw text for parsing
            // Store original content in data attribute for theme switching
            let html = format!(
                r#"<pre class="mermaid" data-mermaid-src="{}">{}</pre>"#,
                html_escape::encode_text(&mermaid_content),
                mermaid_content
            );
            vec![Event::Html(html.into())]
        }
        Event::Text(text) if in_mermaid => {
            mermaid_content.push_str(&text);
            vec![]
        }
        _ => vec![event],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type(Path::new("test.png")), "image/png");
        assert_eq!(get_mime_type(Path::new("test.jpg")), "image/jpeg");
        assert_eq!(get_mime_type(Path::new("test.jpeg")), "image/jpeg");
        assert_eq!(get_mime_type(Path::new("test.gif")), "image/gif");
        assert_eq!(get_mime_type(Path::new("test.svg")), "image/svg+xml");
        assert_eq!(get_mime_type(Path::new("test.webp")), "image/webp");
        assert_eq!(get_mime_type(Path::new("test.bmp")), "image/bmp");
        assert_eq!(get_mime_type(Path::new("test.ico")), "image/x-icon");
        assert_eq!(get_mime_type(Path::new("test.unknown")), "image/png");
    }

    #[test]
    fn test_get_alert_icon_placeholder() {
        let result = get_alert_icon_placeholder("note");
        assert_eq!(
            result,
            r#"<span class="alert-icon" data-alert-type="note"></span>"#
        );

        let result = get_alert_icon_placeholder("warning");
        assert_eq!(
            result,
            r#"<span class="alert-icon" data-alert-type="warning"></span>"#
        );
    }

    #[test]
    fn test_process_github_alerts_note() {
        let input = indoc! {"
            > [!NOTE]
            > This is a note
        "};
        let result = process_github_alerts(input);

        assert!(result.contains(r#"<div class="markdown-alert markdown-alert-note""#));
        assert!(result.contains(r#"<p class="markdown-alert-title""#));
        assert!(result.contains("NOTE"));
        assert!(result.contains("This is a note"));
        assert!(result.contains("</div>"));
    }

    #[test]
    fn test_process_github_alerts_warning() {
        let input = indoc! {"
            > [!WARNING]
            > Be careful!
        "};
        let result = process_github_alerts(input);

        assert!(result.contains(r#"markdown-alert-warning"#));
        assert!(result.contains("WARNING"));
        assert!(result.contains("Be careful!"));
    }

    #[test]
    fn test_process_github_alerts_with_multiline() {
        let input = indoc! {"
            > [!IMPORTANT]
            > First line
            > Second line
            > Third line
        "};
        let result = process_github_alerts(input);

        assert!(result.contains(r#"markdown-alert-important"#));
        assert!(result.contains("First line"));
        assert!(result.contains("Second line"));
        assert!(result.contains("Third line"));
    }

    #[test]
    fn test_process_github_alerts_all_types() {
        let alert_types = vec![
            ("NOTE", "note"),
            ("TIP", "tip"),
            ("IMPORTANT", "important"),
            ("WARNING", "warning"),
            ("CAUTION", "caution"),
        ];

        for (alert_name, alert_class) in alert_types {
            let input = format!("> [!{}]\n> Test content", alert_name);
            let result = process_github_alerts(&input);

            assert!(
                result.contains(&format!(r#"markdown-alert-{}"#, alert_class)),
                "Should contain alert class for {}",
                alert_name
            );
            assert!(
                result.contains(alert_name),
                "Should contain alert name {}",
                alert_name
            );
        }
    }

    #[test]
    fn test_process_github_alerts_no_match() {
        let input = "Regular paragraph\n> Regular quote";
        let result = process_github_alerts(input);

        assert_eq!(result, input);
        assert!(!result.contains("markdown-alert"));
    }

    #[test]
    fn test_process_mermaid_blocks() {
        let markdown = indoc! {"
            ```mermaid
            graph TD
                A-->B
            ```
        "};

        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);
        let events: Vec<Event> = process_mermaid_blocks(parser).collect();

        // Verify that Mermaid block is converted to a single HTML event
        let html_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Html(html) = e {
                    Some(html.as_ref())
                } else {
                    None
                }
            })
            .collect();

        assert!(!html_events.is_empty(), "Should contain HTML event");
        let html = html_events[0];
        assert!(html.contains(r#"<pre class="mermaid""#));
        assert!(html.contains("graph TD"));
        assert!(html.contains("A-->B"));
        assert!(html.contains(r#"data-mermaid-src="#));
    }

    #[test]
    fn test_process_image_paths_http_url() {
        let markdown = "![test](https://example.com/image.png)";
        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);

        let events: Vec<Event> = process_image_paths(parser, Path::new(".")).collect();

        // Verify that HTTPS URLs are not modified
        let image_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Start(Tag::Image { dest_url: url, .. }) = e {
                    Some(url.as_ref())
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(image_events.len(), 1);
        assert_eq!(image_events[0], "https://example.com/image.png");
    }

    #[test]
    fn test_process_image_paths_data_url() {
        let markdown = "![test](data:image/png;base64,abc123)";
        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);

        let events: Vec<Event> = process_image_paths(parser, Path::new(".")).collect();

        // Verify that Data URLs are not modified
        let image_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Start(Tag::Image { dest_url: url, .. }) = e {
                    Some(url.as_ref())
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(image_events.len(), 1);
        assert!(image_events[0].starts_with("data:image/png"));
    }

    #[test]
    fn test_process_image_paths_relative_path() {
        // Create temporary image file for testing
        let temp_dir = TempDir::new().unwrap();
        let image_path = temp_dir.path().join("test.png");

        // Simple PNG header (valid binary)
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR length
        ];
        let mut file = fs::File::create(&image_path).unwrap();
        file.write_all(&png_data).unwrap();

        let markdown = "![test](test.png)";
        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);

        let events: Vec<Event> = process_image_paths(parser, temp_dir.path()).collect();

        // Verify that relative path is converted to data URL
        let image_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Start(Tag::Image { dest_url: url, .. }) = e {
                    Some(url.as_ref())
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(image_events.len(), 1);
        assert!(
            image_events[0].starts_with("data:image/png;base64,"),
            "Should convert to data URL"
        );
    }

    #[test]
    fn test_process_anchor_markdown_files_md_link() {
        let markdown = "[Link to doc](./docs/README.md)";
        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);

        let events: Vec<Event> = process_anchor_markdown_files(parser).collect();

        // Verify that MD links are converted to span elements
        let html_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Html(html) = e {
                    Some(html.as_ref())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            html_events
                .iter()
                .any(|h| h.contains(r#"<span class="md-link""#)),
            "Should contain md-link span"
        );
        assert!(
            html_events
                .iter()
                .any(|h| h.contains("handleMarkdownLinkClick")),
            "Should contain click handler"
        );
        assert!(
            html_events.iter().any(|h| h.contains("./docs/README.md")),
            "Should contain the link URL"
        );
    }

    #[test]
    fn test_process_anchor_markdown_files_markdown_extension() {
        let markdown = "[Link](./file.markdown)";
        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);

        let events: Vec<Event> = process_anchor_markdown_files(parser).collect();

        let html_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Html(html) = e {
                    Some(html.as_ref())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            html_events
                .iter()
                .any(|h| h.contains(r#"<span class="md-link""#)),
            "Should handle .markdown extension"
        );
    }

    #[test]
    fn test_process_anchor_markdown_files_http_link() {
        let markdown = "[Link](https://example.com/page.md)";
        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);

        let events: Vec<Event> = process_anchor_markdown_files(parser).collect();

        // Verify that HTTPS links are kept as regular anchor tags
        let link_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, Event::Start(Tag::Link { .. })))
            .collect();

        assert!(!link_events.is_empty(), "Should keep regular link tag");

        let html_events: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Html(html) = e {
                    Some(html.as_ref())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            !html_events
                .iter()
                .any(|h| h.contains(r#"<span class="md-link""#)),
            "Should NOT convert HTTP links to span"
        );
    }

    #[test]
    fn test_process_anchor_markdown_files_non_md_link() {
        let markdown = "[Link](./file.txt)";
        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);

        let events: Vec<Event> = process_anchor_markdown_files(parser).collect();

        // Verify that .txt links are kept as regular anchor tags
        let link_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, Event::Start(Tag::Link { .. })))
            .collect();

        assert!(!link_events.is_empty(), "Should keep regular link tag");
    }

    #[test]
    fn test_render_to_html_basic() {
        let markdown = "# Hello\n\nThis is a test.";
        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("test.md");

        let result = render_to_html(markdown, &md_path).unwrap();

        assert!(result.contains("<h1>"));
        assert!(result.contains("Hello"));
        assert!(result.contains("<p>"));
        assert!(result.contains("This is a test."));
    }

    #[test]
    fn test_code_block_language_classes() {
        let markdown = indoc! {"
            # Code Blocks Test

            ```rust
            fn main() {
                println!(\"Hello\");
            }
            ```

            ```python
            def hello():
                print(\"world\")
            ```

            ```
            no language specified
            ```
        "};

        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("test.md");

        let result = render_to_html(markdown, &md_path).unwrap();

        // Print the output to inspect
        println!("\n=== HTML OUTPUT ===\n{}\n===================\n", result);

        // Check if language classes are present
        let has_rust = result.contains("language-rust") || result.contains("class=\"rust\"");
        let has_python = result.contains("language-python") || result.contains("class=\"python\"");

        println!("Has rust class: {}", has_rust);
        println!("Has python class: {}", has_python);
    }

    #[test]
    fn test_render_to_html_with_alert() {
        let markdown = indoc! {"
            # Title

            > [!NOTE]
            > This is important
        "};

        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("test.md");

        let result = render_to_html(markdown, &md_path).unwrap();

        assert!(result.contains("markdown-alert-note"));
        assert!(result.contains("This is important"));
    }

    #[test]
    fn test_render_to_html_with_mermaid() {
        let markdown = indoc! {"
            ```mermaid
            graph LR
                A-->B
            ```
        "};

        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("test.md");

        let result = render_to_html(markdown, &md_path).unwrap();

        assert!(result.contains(r#"<pre class="mermaid""#));
        assert!(result.contains("graph LR"));
    }

    #[test]
    fn test_render_to_html_integrated() {
        // Integration test: combining multiple features
        let temp_dir = TempDir::new().unwrap();

        // Create test image
        let image_path = temp_dir.path().join("image.png");
        let png_data = vec![0x89, 0x50, 0x4E, 0x47];
        fs::write(&image_path, png_data).unwrap();

        let markdown = indoc! {"
            # Test Document

            > [!WARNING]
            > Be careful

            ![Test Image](image.png)

            [Link to other doc](other.md)

            ```mermaid
            graph TD
                A-->B
            ```
        "};

        let md_path = temp_dir.path().join("test.md");

        let result = render_to_html(markdown, &md_path).unwrap();

        // Verify that all features are correctly integrated
        assert!(result.contains("<h1>"), "Should render heading");
        assert!(
            result.contains("markdown-alert-warning"),
            "Should render alert"
        );
        assert!(
            result.contains("data:image/png"),
            "Should convert image to data URL"
        );
        assert!(
            result.contains(r#"class="md-link""#),
            "Should convert md link"
        );
        assert!(
            result.contains(r#"<pre class="mermaid""#),
            "Should render mermaid"
        );
    }
}
