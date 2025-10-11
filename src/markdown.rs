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

/// Process GitHub alert format
fn process_github_alerts(markdown: &str) -> String {
    let alert_types = [
        ("NOTE", "note"),
        ("TIP", "tip"),
        ("IMPORTANT", "important"),
        ("WARNING", "warning"),
        ("CAUTION", "caution"),
    ];

    let lines: Vec<&str> = markdown.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check alert pattern
        let mut matched = false;
        for (alert_name, alert_class) in &alert_types {
            if let Some(rest) = line.strip_prefix(&format!("> [!{}]", alert_name)) {
                matched = true;

                // Start of alert with GitHub structure
                result.push(format!(
                    r#"<div class="markdown-alert markdown-alert-{}" dir="auto">"#,
                    alert_class
                ));

                // Title with icon placeholder (SVG injected by JavaScript)
                let icon_placeholder = get_alert_icon_placeholder(alert_class);
                result.push(format!(
                    r#"<p class="markdown-alert-title" dir="auto">{}{}</p>"#,
                    icon_placeholder, alert_name
                ));

                // Content of first line
                if !rest.trim().is_empty() {
                    result.push(rest.trim().to_string());
                }

                // Collect following quoted lines
                i += 1;
                while i < lines.len() && lines[i].starts_with('>') {
                    let content = lines[i].strip_prefix('>').unwrap_or(lines[i]).trim();
                    if !content.is_empty() {
                        result.push(content.to_string());
                    }
                    i += 1;
                }

                result.push("</div>".to_string());
                i -= 1; // Adjust because it's incremented at the end of loop
                break;
            }
        }

        if !matched {
            result.push(line.to_string());
        }

        i += 1;
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
                            r#"<span class="md-link" onclick="window.handleMarkdownLinkClick('{}');">"#,
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
