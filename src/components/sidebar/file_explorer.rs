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

    rsx! {
        div {
            class: "file-explorer",

            if let Some(root) = root_directory {
                ParentNavigation { current_dir: root.clone() }
                DirectoryTree { path: root }
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
fn ParentNavigation(current_dir: PathBuf) -> Element {
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

    rsx! {
        div {
            class: "parent-nav-container",

            // Parent directory navigation (if exists)
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
            }

            // File visibility toggle button
            button {
                class: "file-visibility-toggle",
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
    let mut state_for_dblclick = state.clone();
    let path_for_dblclick = path.clone();

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
                ondoubleclick: move |_| {
                    let path_clone = path_for_dblclick.clone();
                    if is_dir {
                        state_for_dblclick.set_root_directory(path_clone);
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
