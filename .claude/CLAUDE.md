# Project-Specific Rules

## Code Comments

**IMPORTANT: All code comments MUST be written in English.**

- Complex logic should include explanatory comments
- TODO comments should be specific and actionable
- Use English for all comments to maintain consistency with the codebase
- Follow Rust documentation conventions (///, //!)

## Test Code

**Use `indoc` crate for multi-line string literals in tests.**

- Improves readability by preserving proper indentation
- Maintains clean test structure without escaping issues
- Example:
  ```rust
  let input = indoc! {"
      Line 1
      Line 2
      Line 3
  "};
  ```

## Module Organization

**Use modern Rust module system (Rust 2018+) instead of `mod.rs`.**

- Use `module_name.rs` for single-file modules
- Use `module_name/` directory with a file named after the module for multi-file modules
- Declare submodules within the parent module file, not in `mod.rs`

### Examples:

**Single-file module:**
```
src/
  main.rs
  utils.rs        // Declare as `mod utils;` in main.rs
```

**Multi-file module (OLD style - DO NOT USE):**
```
src/
  main.rs
  utils/
    mod.rs        // ❌ Avoid this pattern
    helper.rs
```

**Multi-file module (NEW style - USE THIS):**
```
src/
  main.rs
  utils.rs        // Contains `pub mod helper;`
  utils/
    helper.rs     // ✅ Preferred pattern
```

Or alternatively:
```
src/
  main.rs
  utils/
    helper.rs
    another.rs
  utils.rs        // Contains `pub mod helper; pub mod another;`
```

This modern approach provides:
- Clearer module hierarchy
- Better editor/IDE navigation
- Consistency with Rust 2018+ conventions

## Icon Management

**Adding icons:** Use the `add-icon` skill when you need to add new Tabler icons to the UI.

## UI/UX Design

**Design guidelines:** Follow the patterns in `.claude/rules/ui-design.md` (auto-loaded when working with CSS or component files).

## Dioxus Patterns

### Asset Loading

**Use `asset!()` macro for static resources:**

```rust
const ICON: Asset = asset!("/extras/app-icon.png");

rsx! {
    img { src: "{ICON}", alt: "App" }
}
```

Assets are bundled at build time and paths are resolved automatically.

### Dynamic Text in RSX

**For dynamic values in RSX, use variable binding:**

```rust
// ✗ Wrong - doesn't work in Dioxus
span { "Version {}", env!("CARGO_PKG_VERSION") }

// ✓ Correct - bind to a variable first
{
    let version = format!("Version {}", env!("CARGO_PKG_VERSION"));
    rsx! {
        span { "{version}" }
    }
}
```

### Content Source

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
- **Startup**: Uses `AppPersistentState` from `state.json` (last closed window)
- **New Window**: Uses in-memory globals (last focused window)

#### Window Lifecycle Hooks

```rust
// In App component
use_drop(move || {
    // Save state on window close
    config::save_last_used_state_sync(
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

**IMPORTANT:** Use `save_last_used_state_sync()` in `use_drop()` context (synchronous, blocking).

### State Management Hierarchy

**Arto uses a three-tier state management system:**

#### 1. Global Statics (`LazyLock<Mutex<T>>`)

For app-wide state shared across all windows:

```rust
// Config and persistent state
pub static CONFIG: LazyLock<Mutex<Config>> = ...;
pub static APP_STATE: LazyLock<Mutex<AppPersistentState>> = ...;

// Last focused state (in-memory only)
pub static LAST_SELECTED_THEME: LazyLock<Mutex<ThemePreference>> = ...;
pub static LAST_FOCUSED_DIRECTORY: LazyLock<Mutex<Option<PathBuf>>> = ...;
pub static LAST_FOCUSED_SIDEBAR_VISIBLE: LazyLock<Mutex<Option<bool>>> = ...;

// Event channels
pub static FILE_OPEN_BROADCAST: LazyLock<broadcast::Sender<PathBuf>> = ...;
pub static DIRECTORY_OPEN_BROADCAST: LazyLock<broadcast::Sender<PathBuf>> = ...;

// File watcher (thread-local)
thread_local! {
    pub static FILE_WATCHER: RefCell<Option<FileWatcher>> = ...;
}
```

**When to use:**
- State shared across windows (theme, directory)
- Event communication (broadcasts)
- Singletons (file watcher)

#### 2. Context-Provided State (`AppState`)

For per-window state:

```rust
#[component]
fn App() -> Element {
    // Provide context at root
    let state = use_context_provider(|| AppState::new());

    rsx! {
        // Children can access via use_context::<AppState>()
    }
}

// In child components
let state = use_context::<AppState>();
let tabs = state.tabs.read();
```

**When to use:**
- Window-specific state (tabs, active tab, zoom)
- State that needs to be accessed by multiple components
- State that should NOT be shared across windows

#### 3. Local Component State (`use_signal`)

For UI-only state:

```rust
let mut expanded = use_signal(|| false);
let mut input_value = use_signal(String::new);
```

**When to use:**
- Component-specific UI state
- Temporary state (form inputs, dropdowns)
- State that doesn't need to be accessed by other components

#### Priority Order for Startup/New Window

**On Startup (first window):**
1. `AppPersistentState` from `state.json` (last closed window)
2. Fallback to `Config` defaults from `config.json`

**On New Window:**
1. In-memory globals (last focused window)
2. Fallback to `Config` defaults from `config.json`

**Example:**
```rust
// Startup
pub async fn get_startup_theme() -> String {
    let config = CONFIG.lock().await;
    let state = APP_STATE.lock().await;

    match config.theme.on_startup {
        ThemeStartupBehavior::Default => config.theme.default_theme.clone(),
        ThemeStartupBehavior::LastClosed => state.last_theme.clone()
            .unwrap_or_else(|| config.theme.default_theme.clone()),
    }
}

// New Window
pub fn get_new_window_theme() -> String {
    let config = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(CONFIG.lock())
    });

    match config.theme.on_new_window {
        ThemeNewWindowBehavior::Default => config.theme.default_theme.clone(),
        ThemeNewWindowBehavior::LastFocused => {
            let theme = *LAST_SELECTED_THEME.lock().unwrap();
            // Convert enum to string...
        }
    }
}
```

### Configuration System

**Arto uses a dual-file configuration system:**

#### File Structure

```
~/Library/Application Support/arto/  (macOS)
├── config.json    # User preferences (manually edited or via UI)
└── state.json     # Session state (auto-saved on window close)
```

**config.json** - User configuration:
- Theme defaults and behavior
- Sidebar defaults (visibility, width, filters)
- Directory defaults and behavior
- Behavior on startup/new window (default vs. last used)

**state.json** - Persistent state:
- Last used directory
- Last used theme
- Last sidebar settings
- Automatically saved on window close

#### Hot Reload Pattern

Configuration changes are detected and propagated to all windows:

```rust
// In config.rs
pub static CONFIG_CHANGED_BROADCAST: LazyLock<broadcast::Sender<Config>> = ...;

pub async fn watch_config_file() -> Result<()> {
    let config_path = Config::path();
    let (tx, mut rx) = mpsc::channel::<()>(10);

    FILE_WATCHER.watch(config_path.clone(), tx).await?;

    tokio::spawn(async move {
        while let Some(()) = rx.recv().await {
            match Config::load() {
                Ok(new_config) => {
                    *CONFIG.lock().await = new_config.clone();
                    let _ = CONFIG_CHANGED_BROADCAST.send(new_config);
                }
                Err(e) => {
                    tracing::error!("Failed to reload configuration: {:?}", e);
                }
            }
        }
    });

    Ok(())
}

// In components
use_effect(move || {
    let mut rx = CONFIG_CHANGED_BROADCAST.subscribe();
    spawn_forever(async move {
        while let Ok(new_config) = rx.recv().await {
            // React to config changes
        }
    });
});
```

**IMPORTANT:** Use `spawn_forever()` for infinite event loops in effects.

### Async Patterns in Dioxus

**Dioxus provides specific async primitives for different use cases:**

#### `spawn()` - One-time async task

```rust
let handle_click = move |_| {
    spawn(async move {
        let result = fetch_data().await;
        data.set(result);
    });
};
```

**Use for:** Event handlers, one-time async operations

#### `use_effect()` - Side effects with dependencies

```rust
use_effect(move || {
    let current_file = current_file.read().clone();
    spawn(async move {
        let content = load_file(&current_file).await;
        file_content.set(content);
    });
});
```

**Use for:** React to state changes, set up listeners

#### `spawn_forever()` - Infinite event loop

```rust
use_effect(move || {
    let mut rx = SOME_BROADCAST.subscribe();
    spawn_forever(async move {
        while let Ok(event) = rx.recv().await {
            // Handle event
        }
    });
});
```

**Use for:** Event listeners, broadcast channel subscriptions

**IMPORTANT:** `spawn_forever()` never returns, so it won't block the effect.

#### `use_drop()` - Cleanup on component unmount

```rust
use_drop(move || {
    // Synchronous cleanup code
    save_state_sync();
    close_resources();
});
```

**Use for:** Cleanup, saving state on window close

**IMPORTANT:** `use_drop()` is synchronous. Use `save_last_used_state_sync()` for blocking operations.

#### Blocking async in sync context

```rust
// When you need to await in a non-async function (e.g., Dioxus context)
let config = tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(CONFIG.lock())
});
```

**Use sparingly:** Only when you must bridge sync/async boundaries.

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

**File handling patterns for robust path and symlink management:**

#### Path Canonicalization

```rust
// ALWAYS canonicalize paths to handle symlinks
let canonical_path = path.canonicalize()?;
```

**Why:** macOS Finder aliases and symlinks must be resolved to real paths.

#### Directory Root Extraction

```rust
// For files: use parent directory as sidebar root
let root = if path.is_file() {
    path.parent().unwrap_or(&path).to_path_buf()
} else {
    path.clone()
};
```

#### File Watching Pattern

```rust
// Use NonRecursive mode for single file watching
FILE_WATCHER.watch(file_path.clone(), tx).await?;

// Use debouncer to prevent rapid re-renders
// (Built into notify-debouncer-full, 500ms default)
```

**Thread-local watcher:**
```rust
thread_local! {
    pub static FILE_WATCHER: RefCell<Option<FileWatcher>> =
        RefCell::new(Some(FileWatcher::new()));
}

FILE_WATCHER.with(|watcher| {
    watcher.borrow_mut().as_mut()
        .unwrap()
        .watch(path, tx)
});
```

**IMPORTANT:** File watcher is thread-local to avoid Send/Sync issues with Dioxus.

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
