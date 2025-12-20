# Configuration Module Patterns

Design patterns and best practices for structuring configuration modules in Rust/Dioxus applications.

## Module Organization

**Organize configuration and state into focused modules:**

```
src/
├── config/
│   ├── app_config.rs        # Module entry point (re-exports only)
│   ├── app_config/          # Config type definitions and enums
│   │   ├── behavior.rs
│   │   ├── directory_config.rs
│   │   ├── sidebar_config.rs
│   │   └── theme_config.rs
│   └── persistence.rs       # File I/O and global CONFIG instance
├── state/
│   ├── app_state.rs         # Module entry point (re-exports only)
│   ├── app_state/           # Per-window state types
│   │   ├── sidebar.rs
│   │   └── tabs.rs
│   └── persistence.rs       # PersistedState (saved on window close)
└── window/
    └── helpers.rs           # Startup/new window value resolution
```

### Module Entry Point Pattern

**Entry point files should only declare modules and re-export public APIs:**

```rust
// config/app_config.rs
mod behavior;
mod directory_config;
mod sidebar_config;
mod theme_config;

pub use behavior::{NewWindowBehavior, StartupBehavior};
pub use directory_config::DirectoryConfig;
pub use sidebar_config::SidebarConfig;
pub use theme_config::ThemeConfig;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub directory: DirectoryConfig,
    pub theme: ThemeConfig,
    pub sidebar: SidebarConfig,
}
```

**Do NOT implement complex logic in entry point modules.**

## Configuration vs State Separation

**Separate user configuration from application state:**

- **config.json** - User preferences (manually edited or via UI)
  - Default values
  - Behavior settings (startup, new window)
  - User-controlled configuration

- **state.json** - Session state (auto-saved on window close)
  - Last used directory
  - Last used theme
  - Last window settings
  - Runtime state

### File Locations

```rust
// Platform-specific config directory
#[cfg(target_os = "macos")]
{
    if let Some(mut path) = dirs::config_local_dir() {
        path.push("app-name");
        path.push("config.json");
        return path;
    }
}
```

## Startup vs New Window Pattern

**Use value resolution helpers in window/helpers.rs:**

```rust
// window/helpers.rs provides unified value resolution
pub fn get_theme_value(is_first_window: bool) -> ThemeValue {
    let cfg = CONFIG.read();
    let theme = if is_first_window {
        match cfg.theme.on_startup {
            StartupBehavior::Default => cfg.theme.default_theme,
            StartupBehavior::LastClosed => LAST_FOCUSED_STATE.read().theme,
        }
    } else {
        match cfg.theme.on_new_window {
            NewWindowBehavior::Default => cfg.theme.default_theme,
            NewWindowBehavior::LastFocused => LAST_FOCUSED_STATE.read().theme,
        }
    };
    ThemeValue { theme }
}
```

**Usage in window creation:**

```rust
// First window (startup)
let theme = get_theme_value(true);
let dir = get_directory_value(true);
let sidebar = get_sidebar_value(true);

// Subsequent windows
let theme = get_theme_value(false);
let dir = get_directory_value(false);
let sidebar = get_sidebar_value(false);
```

**Key differences:**
- **Startup** (`is_first_window: true`): Uses `LAST_FOCUSED_STATE` (saved from last closed window)
- **New Window** (`is_first_window: false`): Uses `LAST_FOCUSED_STATE` (updated by last focused window)

## Avoid Duplicate Enums

**Bad - Multiple enums for same concept:**

```rust
pub enum DirectoryStartupBehavior {
    Default,
    LastClosed,
}

pub enum ThemeStartupBehavior {
    Default,
    LastClosed,
}

pub enum SidebarStartupBehavior {
    Default,
    LastClosed,
}
```

**Good - Unified enums:**

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupBehavior {
    #[default]
    Default,
    LastClosed,  // Auto-converted to "last_closed"
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewWindowBehavior {
    #[default]
    Default,
    LastFocused,  // Auto-converted to "last_focused"
}
```

**Use the same enum across all config structs:**

```rust
pub struct DirectoryConfig {
    pub on_startup: StartupBehavior,
    pub on_new_window: NewWindowBehavior,
}

pub struct ThemeConfig {
    pub on_startup: StartupBehavior,      // ✓ Same enum
    pub on_new_window: NewWindowBehavior, // ✓ Same enum
}
```

## Enum vs String

**Use enums for fixed sets of values:**

```rust
// Bad - String allows typos ("ligt", "autoo", etc.)
pub struct ThemeConfig {
    pub default_theme: String,
}

// Good - Type-safe enum
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    Auto,   // → "auto"
    Light,  // → "light"
    Dark,   // → "dark"
}

pub struct ThemeConfig {
    pub default_theme: ThemePreference,
}
```

**Benefits:**
- Type safety (prevents typos)
- Better IDE support
- Self-documenting code
- Easy to refactor
