use dioxus::document;
use dioxus::prelude::*;
use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

use crate::components::icon::{Icon, IconName};
use crate::state::AppState;
use crate::utils::file::is_markdown_file;

// Sort entries: directories first, then files, both alphabetically
fn sort_entries(items: &mut [PathBuf]) {
    items.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();

        match (a_is_dir, b_is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });
}

// Read and sort directory entries
fn read_sorted_entries(path: &PathBuf) -> Vec<PathBuf> {
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut items: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
            sort_entries(&mut items);
            items
        }
        Err(err) => {
            tracing::error!("Failed to read directory {:?}: {}", path, err);
            vec![]
        }
    }
}

#[component]
pub fn FileExplorer() -> Element {
    let state = use_context::<AppState>();
    let sidebar_state = state.sidebar.read();
    let root_directory = sidebar_state.root_directory.clone();

    // Refresh counter to force DirectoryTree re-render
    let refresh_counter = use_signal(|| 0u32);

    rsx! {
        div {
            class: "file-explorer",

            if let Some(root) = root_directory {
                ParentNavigation { current_dir: root.clone(), refresh_counter }
                // Use refresh_counter as key to force re-render when reloading
                DirectoryTree { key: "{refresh_counter}", path: root }
            } else {
                div {
                    class: "file-explorer-empty",
                    "No directory open"
                }
            }
        }
    }
}

#[component]
fn ParentNavigation(current_dir: PathBuf, mut refresh_counter: Signal<u32>) -> Element {
    let mut state = use_context::<AppState>();
    let hide_non_markdown = state.sidebar.read().hide_non_markdown;

    let has_parent = current_dir.parent().is_some();

    // Get current directory name
    let dir_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("..")
        .to_string();

    let mut state_for_toggle = state.clone();

    // Reload state for animation
    let is_reloading = use_signal(|| false);
    let mut is_reloading_write = is_reloading;

    let on_reload = move |_| {
        // Set reloading state for animation
        is_reloading_write.set(true);

        // Increment counter to force DirectoryTree re-render
        refresh_counter.set(refresh_counter() + 1);

        // Reset reloading state after animation
        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
            is_reloading_write.set(false);
        });
    };

    rsx! {
        div {
            class: "parent-nav-container",

            // Parent directory navigation or root indicator
            if has_parent {
                div {
                    class: "file-tree-node parent-nav",
                    onclick: move |_| {
                        if let Some(parent) = current_dir.parent() {
                            state.set_root_directory(parent.to_path_buf());
                        }
                    },

                    div {
                        class: "file-tree-node-content",
                        Icon {
                            name: IconName::ChevronLeft,
                            size: 16,
                            class: "file-tree-icon",
                        }
                        span {
                            class: "file-tree-label",
                            "{dir_name}"
                        }
                    }
                }
            } else {
                // Show root indicator when at filesystem root
                div {
                    class: "file-tree-node parent-nav root-indicator",

                    div {
                        class: "file-tree-node-content",
                        Icon {
                            name: IconName::Server,
                            size: 16,
                            class: "file-tree-icon",
                        }
                        span {
                            class: "file-tree-label",
                            "/"
                        }
                    }
                }
            }

            // Toolbar buttons container
            div {
                class: "file-explorer-toolbar",

                // Reload button
                button {
                    class: "file-explorer-toolbar-button",
                    class: if *is_reloading.read() { "reloading" },
                    title: "Reload file explorer",
                    onclick: on_reload,
                    Icon {
                        name: IconName::Refresh,
                        size: 20,
                    }
                }

                // File visibility toggle button
                button {
                    class: "file-explorer-toolbar-button",
                    title: if hide_non_markdown { "Show all files" } else { "Hide non-markdown files" },
                    onclick: move |_| {
                        state_for_toggle.sidebar.write().hide_non_markdown = !hide_non_markdown;
                    },
                    Icon {
                        name: if hide_non_markdown { IconName::EyeOff } else { IconName::Eye },
                        size: 20,
                    }
                }
            }
        }
    }
}

#[component]
fn DirectoryTree(path: PathBuf) -> Element {
    let entries = read_sorted_entries(&path);

    rsx! {
        div {
            class: "directory-tree",
            for entry in entries {
                FileTreeNode { path: entry, depth: 0 }
            }
        }
    }
}

#[component]
fn FileTreeNode(path: PathBuf, depth: usize) -> Element {
    let state = use_context::<AppState>();

    let is_dir = path.is_dir();
    let is_expanded = state.sidebar.read().expanded_dirs.contains(&path);
    let hide_non_markdown = state.sidebar.read().hide_non_markdown;

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let is_markdown = !is_dir && is_markdown_file(&path);

    // Hide non-markdown files if the setting is enabled
    if hide_non_markdown && !is_dir && !is_markdown {
        return rsx! {};
    }

    let current_tab = state.current_tab();
    let is_active = current_tab
        .and_then(|tab| tab.file().map(|f| f == &path))
        .unwrap_or(false);

    let indent_style = format!("padding-left: {}px", depth * 20);

    let mut state_for_click = state.clone();
    let path_for_click = path.clone();
    let mut state_for_enter = state.clone();
    let path_for_enter = path.clone();
    let path_for_copy = path.clone();

    // Copy feedback state
    let mut is_copied = use_signal(|| false);

    rsx! {
        div {
            class: "file-tree-node",
            class: if is_active { "active" },

            div {
                class: "file-tree-node-content",
                style: "{indent_style}",
                onclick: move |_| {
                    let path_clone = path_for_click.clone();
                    if is_dir {
                        state_for_click.toggle_directory_expansion(path_clone);
                    } else {
                        // Open any file (not just markdown)
                        state_for_click.open_file(path_clone);
                    }
                },

                // Directory icons
                if is_dir {
                    Icon {
                        name: if is_expanded { IconName::ChevronDown } else { IconName::ChevronRight },
                        size: 16,
                        class: "file-tree-chevron",
                    }
                    Icon {
                        name: if is_expanded { IconName::FolderOpen } else { IconName::Folder },
                        size: 16,
                        class: "file-tree-icon",
                    }
                } else {
                    // File icon with spacer
                    span { class: "file-tree-spacer" }
                    Icon {
                        name: IconName::File,
                        size: 16,
                        class: "file-tree-icon",
                    }
                }

                // Label
                span {
                    class: "file-tree-label",
                    class: if !is_markdown && !is_dir { "disabled" },
                    "{name}"
                }

                // Enter directory button (only for directories)
                if is_dir {
                    button {
                        class: "file-tree-enter-button",
                        title: "Open as root directory",
                        onclick: move |evt| {
                            // Prevent triggering parent click handler
                            evt.stop_propagation();
                            let path_clone = path_for_enter.clone();
                            state_for_enter.set_root_directory(path_clone);
                        },
                        span { class: "file-tree-enter-label", "Enter" }
                        Icon {
                            name: IconName::Login,
                            size: 12,
                        }
                    }
                }

                // Copy path button
                button {
                    class: "file-tree-copy-button",
                    class: if *is_copied.read() { "copied" },
                    title: "Copy full path",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        let path_str = path_for_copy.to_string_lossy().to_string();
                        // Escape backticks and backslashes for JavaScript
                        let escaped = path_str.replace('\\', "\\\\").replace('`', "\\`");
                        spawn(async move {
                            let js = format!("navigator.clipboard.writeText(`{}`)", escaped);
                            let _ = document::eval(&js).await;
                            // Show success feedback
                            is_copied.set(true);
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            is_copied.set(false);
                        });
                    },
                    Icon {
                        name: if *is_copied.read() { IconName::Check } else { IconName::Copy },
                        size: 12,
                    }
                }
            }

            // Expanded directory children
            if is_dir && is_expanded {
                {
                    let children = read_sorted_entries(&path);
                    rsx! {
                        for child in children {
                            FileTreeNode { path: child, depth: depth + 1 }
                        }
                    }
                }
            }
        }
    }
}
