# Architecture Overview: Config, Session, State

Understanding the relationship between Config, Session, and State modules.

## Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ User Configuration Layer (config/)                          │
│ - File: config.json                                         │
│ - Edited by: User (manual or via Preferences UI)           │
│ - Contains: Default values, behavior settings               │
│ - Example: "default_theme": "auto"                          │
│           "on_startup": "last_closed"                       │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ State Persistence Layer (state/persistence.rs)              │
│ - File: state.json                                          │
│ - Edited by: App (auto-saved on window close)              │
│ - Contains: Last closed window's state (PersistedState)     │
│ - Example: "last_theme": "dark"                             │
│           "last_directory": "/path/to/project"              │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Runtime State Layer (state.rs)                              │
│ - File: None (memory only)                                  │
│ - Scope: Per-window                                         │
│ - Contains: Current UI state (tabs, zoom, sidebar, etc.)   │
│ - Example: tabs: [Tab1, Tab2], active_tab: 0               │
│           sidebar.is_visible: true                          │
└─────────────────────────────────────────────────────────────┘
```

## Module Responsibilities

### 1. Config Module (`src/config/`)

**Purpose:** User preferences and default values

**Files:**
- `config.json` - Stored in `~/Library/Application Support/arto/` (macOS)

**Example content:**
```json
{
  "theme": {
    "defaultTheme": "auto",
    "onStartup": "last_closed",
    "onNewWindow": "last_focused"
  },
  "directory": {
    "defaultDirectory": "/Users/alice/Documents",
    "onStartup": "default",
    "onNewWindow": "last_focused"
  },
  "sidebar": {
    "defaultOpen": true,
    "defaultWidth": 280.0,
    "defaultShowAllFiles": false
  }
}
```

**Key type:** `Config`

**When used:**
- On app startup (to determine default behavior)
- When user opens Preferences and saves changes
- As fallback when state.json doesn't exist

### 2. State Persistence (`src/state/persistence.rs`)

**Purpose:** Remember the last closed window's state for restoration

**Files:**
- `state.json` - Stored in `~/Library/Application Support/arto/` (macOS)

**Example content:**
```json
{
  "lastDirectory": "/Users/alice/project/docs",
  "lastTheme": "dark",
  "lastSidebarVisible": true,
  "lastSidebarWidth": 320.0,
  "lastShowAllFiles": false
}
```

**Key type:** `PersistedState`

**When used:**
- Saved: When any window closes (`use_drop()` in App component)
- Loaded: On app startup (if user configured `on_startup: "last_closed"`)

### 3. State Module (`src/state.rs`)

**Purpose:** Current UI state for each window instance

**Storage:** Memory only (never saved to disk)

**Key type:** `AppState`

**Contents:**
```rust
pub struct AppState {
    pub tabs: Signal<Vec<Tab>>,              // Open tabs
    pub active_tab: Signal<usize>,           // Which tab is active
    pub current_theme: Signal<ThemePreference>, // Current theme
    pub zoom_level: Signal<f64>,             // Zoom level
    pub sidebar: Signal<SidebarState>,       // Sidebar state
}
```

**Lifecycle:**
- Created: When window opens
- Updated: During user interaction (opening files, changing theme, etc.)
- Destroyed: When window closes (after saving to state.json)

## Data Flow

### Startup Flow (First Window)

```
1. Load config.json
   ├─> Config { theme.on_startup: "last_closed" }
   └─> Config { directory.on_startup: "default" }

2. Load state.json
   ├─> PersistedState { last_theme: "dark" }
   └─> PersistedState { last_directory: "/path/to/project" }

3. Apply startup behavior
   ├─> Theme: "last_closed" → Use persisted.last_theme ("dark")
   └─> Directory: "default" → Use config.default_directory

4. Create AppState with computed values
   └─> AppState { current_theme: "dark", sidebar.root_directory: "/Users/alice/Documents" }
```

### New Window Flow (Second+ Window)

```
1. Load config.json
   ├─> Config { theme.on_new_window: "last_focused" }
   └─> Config { directory.on_new_window: "last_focused" }

2. Read in-memory globals
   ├─> LAST_SELECTED_THEME (from last focused window)
   └─> LAST_FOCUSED_DIRECTORY (from last focused window)

3. Apply new window behavior
   ├─> Theme: "last_focused" → Use LAST_SELECTED_THEME
   └─> Directory: "last_focused" → Use LAST_FOCUSED_DIRECTORY

4. Create AppState with computed values
   └─> AppState { current_theme: <from memory>, sidebar.root_directory: <from memory> }
```

### Window Close Flow

```
1. Read current AppState
   ├─> current_theme: "dark"
   ├─> sidebar.root_directory: "/path/to/project"
   ├─> sidebar.is_visible: true
   ├─> sidebar.width: 320.0
   └─> sidebar.hide_non_markdown: false

2. Construct Session
   └─> PersistedState {
         last_theme: Some("dark"),
         last_directory: Some("/path/to/project"),
         last_sidebar_visible: Some(true),
         last_sidebar_width: Some(320.0),
         last_show_all_files: Some(true),
       }

3. Save to state.json
   └─> ~/Library/Application Support/arto/state.json

4. Update in-memory SESSION global
   └─> For next startup (if app reopens immediately)
```

## Decision Matrix

**When adding new settings, decide:**

| Question | Config | Session | State |
|----------|--------|---------|-------|
| Should user be able to edit it? | ✓ | ✗ | ✗ |
| Should it persist between app launches? | ✓ | ✓ | ✗ |
| Is it different per window? | ✗ | ✗ | ✓ |
| Should it restore on startup? | ✓ (default) | ✓ (last used) | ✗ |

**Examples:**

- **Default theme** → Config (user sets preference, applies to all windows)
- **Last used theme** → Session (app remembers, restores on startup)
- **Current theme** → State (per-window, might differ during session)
- **Open tabs** → State only (never saved)
- **Sidebar width** → Config (default), Session (last used), State (current)

## Common Patterns

### Reading a value on startup

```rust
// In config/getters.rs
pub async fn get_startup_theme() -> ThemePreference {
    let config = CONFIG.lock().await;
    let session = SESSION.lock().await;

    match config.theme.on_startup {
        StartupBehavior::Default => config.theme.default_theme,
        StartupBehavior::LastClosed => session.last_theme.unwrap_or(config.theme.default_theme),
    }
}
```

**Priority:**
1. Check config behavior setting (`on_startup`)
2. If "default" → use `config.theme.default_theme`
3. If "last_closed" → use `session.last_theme` (fallback to config default)

### Updating a value during runtime

```rust
// In App component
let mut state = use_context::<AppState>();

// User changes theme
state.current_theme.set(ThemePreference::Dark);

// Also update global for "last_focused" behavior
*LAST_SELECTED_THEME.lock().unwrap() = ThemePreference::Dark;
```

**Two updates:**
1. Update `AppState` for current window
2. Update global static for next "new window"

### Saving on window close

```rust
// In App component use_drop()
let session = PersistedState {
    last_directory: sidebar.root_directory.clone(),
    last_theme: Some(*current_theme.read()),
    last_sidebar_visible: Some(sidebar.is_visible),
    last_sidebar_width: Some(sidebar.width),
    last_show_all_files: Some(!sidebar.hide_non_markdown),
};

save_sync(&session);
```

**What happens:**
1. Collect values from current `AppState`
2. Construct `Session` struct
3. Save to `state.json` and update `SESSION` global

## In-Memory Globals

**Additional layer for "last_focused" behavior:**

```rust
// In state.rs
pub static LAST_SELECTED_THEME: LazyLock<Mutex<ThemePreference>> = ...;
pub static LAST_FOCUSED_DIRECTORY: LazyLock<Mutex<Option<PathBuf>>> = ...;
pub static LAST_FOCUSED_SIDEBAR_VISIBLE: LazyLock<Mutex<Option<bool>>> = ...;
```

**Purpose:** Remember the last focused window's state for "new window" behavior

**Differences from Session:**
- Session → Last **closed** window (persisted to disk)
- In-memory globals → Last **focused** window (memory only)

**Why both?**
- Startup uses last closed window (most recent state before app quit)
- New window uses last focused window (current active window's state)

## Summary

| Aspect | Config | PersistedState | State | In-Memory Globals |
|--------|--------|----------------|-------|-------------------|
| **Storage** | config.json | state.json | Memory | Memory |
| **Scope** | App-wide | App-wide | Per-window | App-wide |
| **Lifetime** | Permanent | Permanent | Window | App session |
| **Edited by** | User | App | App | App |
| **Used for** | Defaults | Last closed | Current | Last focused |
| **Example** | default_theme | last_theme | current_theme | LAST_SELECTED_THEME |
