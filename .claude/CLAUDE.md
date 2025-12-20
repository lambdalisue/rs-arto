# Project-Specific Rules

> **📖 For detailed best practices and tips:** See [TIPS.md](.claude/TIPS.md)

## Quick Reference

- **Code Comments**: Must be in English
- **Test Code**: Use `indoc` crate for multi-line strings
- **Module System**: Use Rust 2018+ (no `mod.rs`)
- **Icon Management**: Use `add-icon` skill
- **UI/UX Design**: See `.claude/rules/ui-design.md`

## Content Source

**Always check existing content files before writing descriptions:**

- Welcome page content: `assets/welcome.md`
- README: Project description and philosophy
- Use actual project descriptions, not generic placeholders

## Architecture Patterns

### Window Management & Lifecycle

**Arto uses a multi-window architecture with a hidden background window:**

#### Window Types

1. **Background Entrypoint Window** (hidden)
   - Created at startup, never shown to user
   - Handles system events: file open, app reopen
   - Prevents multiple app instances
   - Required for macOS menu system
   - Parent of all main windows

2. **Main Windows** (user-visible)
   - Created on demand (File → New Window)
   - Each has independent tabs and state
   - Child windows properly close when parent closes

3. **Child Windows** (specialized)
   - Mermaid diagram viewer, etc.
   - Owned by a parent main window
   - Auto-close when parent closes

#### Window Creation Pattern

```rust
// Initial window (async startup)
let initial_dir = config::get_startup_directory().await;
let initial_theme = config::get_startup_theme().await;
window::open_window(initial_dir, Some(initial_theme)).await?;

// New window (sync, uses last focused state)
let new_dir = config::get_new_window_directory();
let new_theme = config::get_new_window_theme();
window::open_window_sync(new_dir, Some(new_theme));
```

**Key differences:**
- **Startup**: Uses `Session` from `state.json` (last closed window)
- **New Window**: Uses in-memory globals (last focused window)

#### Window Lifecycle Hooks

```rust
// In App component
use_drop(move || {
    // Save state on window close
    config::save_session_sync(
        Some(current_dir),
        Some(current_theme),
        Some(sidebar_visible),
        Some(sidebar_width),
        Some(show_all_files),
    );

    // Close child windows
    window::close_child_windows();
});
```

**IMPORTANT:** Use `persisted.save_sync()` in `use_drop()` context (synchronous, blocking).

### State Management Hierarchy

**Three-tier system (see TIPS.md and architecture-overview.md for details):**

1. **Global Statics** - Shared across windows (CONFIG, LAST_SELECTED_THEME, broadcast channels)
2. **Context (AppState)** - Per-window state (tabs, active tab, zoom)
3. **Local (use_signal)** - Component-only UI state

**Startup priority:**
1. `PersistedState` from `state.json` (last closed window)
2. Fallback to `Config` defaults

**New window priority:**
1. In-memory globals (last focused window)
2. Fallback to `Config` defaults

### Configuration System

**Dual-file system (see TIPS.md and architecture-overview.md for details):**

```
~/Library/Application Support/arto/
├── config.json   # User preferences (Config type)
└── state.json    # Last window state (PersistedState type)
```

**Hot reload:** File changes broadcast to all windows via `CONFIG_CHANGED_BROADCAST`.

### Async Patterns in Dioxus

**Key patterns (see TIPS.md for details):**

- `spawn()` - Event handlers, one-time async
- `use_effect()` - React to state changes
- `spawn_forever()` - Infinite loops (broadcast listeners)
- `use_drop()` - Cleanup (synchronous only!)

**Critical:** `spawn_forever()` never returns. `use_drop()` is synchronous - use `persisted.save_sync()` for blocking operations.

### Markdown Rendering Pipeline

**Markdown rendering follows a specific order to handle special syntax:**

```
Input Markdown
    ↓
1. Pre-process GitHub Alerts
   (Convert blockquote-based alerts to custom HTML)
    ↓
2. Parse with pulldown-cmark
   (GitHub Flavored Markdown options)
    ↓
3. Process Special Code Blocks
   - Mermaid diagrams → custom renderer
   - Math expressions → KaTeX
    ↓
4. Render to HTML
    ↓
5. Post-process with lol_html
   - Convert relative image paths to data URLs
   - Convert local .md links to clickable spans
   - Preserve HTTP/HTTPS URLs
    ↓
Output HTML
```

#### Key Implementation Details

**1. GitHub Alerts** (`markdown.rs`):
```rust
// Convert blockquote alerts BEFORE parsing
fn preprocess_github_alerts(markdown: &str) -> String {
    // [!NOTE] → <div class="markdown-alert markdown-alert-note">
}
```

**2. Special Code Blocks** (during HTML generation):
```rust
Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
    match lang.as_ref() {
        "mermaid" => {
            // Generate mermaid diagram container
        }
        "math" => {
            // Generate KaTeX container
        }
        _ => {
            // Regular syntax highlighting
        }
    }
}
```

**3. Post-processing** (`lol_html` element handler):
```rust
// Convert relative images to data URLs (offline support)
element!("img[src]", |el| {
    if let Some(src) = el.get_attribute("src") {
        if !src.starts_with("http") && !src.starts_with("data:") {
            let data_url = image_to_data_url(&base_path.join(&src))?;
            el.set_attribute("src", &data_url)?;
        }
    }
});

// Convert local .md links to custom click handlers
element!("a[href]", |el| {
    if let Some(href) = el.get_attribute("href") {
        if href.ends_with(".md") && !href.starts_with("http") {
            el.remove_attribute("href");
            el.set_attribute("class", "markdown-link")?;
            el.set_attribute("data-path", &href)?;
        }
    }
});
```

**IMPORTANT:** Always follow this order. Pre-processing must happen before parsing to avoid conflicts.

### File Operations

**Key patterns (see TIPS.md for details):**

- Always canonicalize paths (macOS symlinks)
- Extract directory root: use parent for files
- File watcher is thread-local (avoid Send/Sync issues)

### Menu & Event Handling

**Menu system follows platform-specific patterns with type-safe IDs:**

#### Menu ID Pattern

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuId {
    // File menu
    NewWindow,
    OpenFile,
    OpenDirectory,
    CloseTab,

    // Edit menu
    Copy,
    SelectAll,

    // View menu
    ZoomIn,
    ZoomOut,
    ZoomReset,

    // Custom items (replace predefined)
    About,
    Preferences,
}

impl From<MenuId> for MenuItemId {
    fn from(id: MenuId) -> Self {
        MenuItemId::new(format!("{:?}", id))
    }
}
```

**Why enum over strings:** Type safety, autocomplete, refactoring support.

#### Split Handler Pattern

Menu events are handled in two places:

**1. Global Handler** (no state access):
```rust
// In entrypoint.rs
menu_event.listen(move |event: MenuEvent| {
    match event.id.as_ref().parse::<MenuId>() {
        MenuId::NewWindow => {
            window::open_window_sync(None, None);
        }
        // Other global actions...
        _ => {}
    }
});
```

**2. State-Dependent Handler** (in App component):
```rust
// In app.rs
use_effect(move || {
    spawn_forever(async move {
        while let Ok(event) = rx.recv().await {
            match event.id.as_ref().parse::<MenuId>() {
                MenuId::CloseTab => {
                    state.close_current_tab();
                }
                MenuId::Preferences => {
                    state.open_preferences();
                }
                // Other state actions...
                _ => {}
            }
        }
    });
});
```

**Why split:** Some actions don't need state (new window), others do (close tab, preferences).

#### Platform-Specific Menus

```rust
#[cfg(target_os = "macos")]
fn build_menu() -> Menu {
    Menu::new()
        .add_submenu("Arto", true, app_menu())
        .add_submenu("File", false, file_menu())
        // macOS-specific menu structure
}

#[cfg(not(target_os = "macos"))]
fn build_menu() -> Menu {
    Menu::new()
        .add_submenu("File", false, file_menu())
        // Standard menu structure
}
```

**IMPORTANT:** Replace `PredefinedMenuItem::about()` with custom `MenuId::About` to control navigation.
