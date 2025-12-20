//! Event propagation system for multi-window coordination.
//!
//! # Event Flow Architecture
//!
//! This module implements Layer 2 of a 3-layer event propagation system:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Layer 1: OS → Dioxus (Single Consumer)                             │
//! │ File: main.rs → components/entrypoint.rs                           │
//! │ Channel: mpsc (single producer, single consumer)                   │
//! │                                                                     │
//! │  OS Event (Finder/CLI)                                             │
//! │       │                                                             │
//! │       ├──→ Event::Opened { urls } ──→ OpenEvent::File(path)        │
//! │       ├──→ Event::Opened { urls } ──→ OpenEvent::Directory(path)   │
//! │       └──→ Event::Reopen          ──→ OpenEvent::Reopen            │
//! │                │                                                    │
//! │                v                                                    │
//! │       OPEN_EVENT_RECEIVER (mpsc::Receiver)                         │
//! │                │                                                    │
//! │                v                                                    │
//! │       Entrypoint component (consumes once)                         │
//! └─────────────────────────────────────────────────────────────────────┘
//!                          │
//!                          v
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Layer 2: Entrypoint → Apps (Multiple Consumers) ← THIS MODULE      │
//! │ File: events.rs                                                     │
//! │ Channel: broadcast (single producer, multiple consumers)           │
//! │                                                                     │
//! │  Entrypoint logic:                                                 │
//! │    - Check if windows exist                                        │
//! │    - Handle Reopen specially (focus or create)                     │
//! │    - Broadcast to all windows if needed                            │
//! │                │                                                    │
//! │                ├──→ FILE_OPEN_BROADCAST.send(path)                 │
//! │                └──→ DIRECTORY_OPEN_BROADCAST.send(path)            │
//! │                         │                                           │
//! │                         v                                           │
//! │            ┌────────────┴────────────┐                              │
//! │            │                         │                              │
//! │            v                         v                              │
//! │       Window 1 (App)            Window 2 (App)                     │
//! │       rx.subscribe()            rx.subscribe()                     │
//! └─────────────────────────────────────────────────────────────────────┘
//!                          │
//!                          v
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Layer 3: Focus-based Filtering (Per-Window)                        │
//! │ File: components/app.rs                                            │
//! │                                                                     │
//! │  Each App component:                                               │
//! │    while let Ok(path) = rx.recv().await {                          │
//! │        if window().is_focused() {  ← Only focused window handles   │
//! │            state.open_file(path);                                  │
//! │        }                                                            │
//! │    }                                                                │
//! │                                                                     │
//! │  Why focus check? Without it, ALL windows would open the file!     │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Why Broadcast Channels?
//!
//! We use `tokio::sync::broadcast` instead of `mpsc` because:
//! - **Multiple windows**: Each window needs to receive the same event
//! - **Dynamic subscribers**: Windows can be created/destroyed at runtime
//! - **Focus filtering**: All windows receive events, but only focused window acts
//!
//! # Why Not Direct OS → Apps?
//!
//! We cannot send directly from `main.rs` to `App` components because:
//! 1. **Synchronous initial event**: First event must be handled before any window exists
//! 2. **Window existence check**: Need to create window if none exist
//! 3. **Reopen special handling**: App icon click should focus existing window, not always open file
//!
//! The Entrypoint layer provides this coordination logic before broadcasting to App components.

use std::path::PathBuf;
use tokio::sync::broadcast;

/// Global broadcast sender for opening files in tabs.
///
/// Distributes file open events from Entrypoint to all App components.
/// Each window's App component subscribes via `FILE_OPEN_BROADCAST.subscribe()`.
/// Only the focused window should handle the event (checked via `window().is_focused()`).
pub static FILE_OPEN_BROADCAST: std::sync::LazyLock<broadcast::Sender<PathBuf>> =
    std::sync::LazyLock::new(|| broadcast::channel(100).0);

/// Global broadcast sender for opening directories in sidebar.
///
/// Distributes directory open events from Entrypoint to all App components.
/// Unlike file events, directory events are typically handled by ALL windows
/// (updating each window's sidebar root), but this can be changed to focused-only if needed.
pub static DIRECTORY_OPEN_BROADCAST: std::sync::LazyLock<broadcast::Sender<PathBuf>> =
    std::sync::LazyLock::new(|| broadcast::channel(100).0);
