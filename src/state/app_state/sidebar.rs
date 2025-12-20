use super::super::persistence::LAST_FOCUSED_STATE;
use super::AppState;
use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

/// Represents the state of the sidebar file explorer
#[derive(Debug, Clone, PartialEq)]
pub struct Sidebar {
    pub open: bool,
    pub expanded_dirs: HashSet<PathBuf>,
    pub width: f64,
    pub show_all_files: bool,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            open: false,
            expanded_dirs: HashSet::new(),
            width: 280.0,
            show_all_files: false,
        }
    }
}

impl Sidebar {
    /// Toggle directory expansion state
    pub fn toggle_expansion(&mut self, path: PathBuf) {
        if self.expanded_dirs.contains(&path) {
            self.expanded_dirs.remove(&path);
        } else {
            self.expanded_dirs.insert(path);
        }
    }
}

impl AppState {
    /// Toggle sidebar visibility
    pub fn toggle_sidebar(&mut self) {
        let mut sidebar = self.sidebar.write();
        sidebar.open = !sidebar.open;
        LAST_FOCUSED_STATE.write().sidebar_open = sidebar.open;
    }

    /// Set the root directory for the sidebar file explorer
    /// Note: The directory is persisted to state file when window closes
    pub fn set_root_directory(&mut self, path: PathBuf) {
        *self.directory.write() = Some(path.clone());
        self.sidebar.write().expanded_dirs.clear();
        LAST_FOCUSED_STATE.write().directory = Some(path);
    }

    /// Toggle directory expansion state
    pub fn toggle_directory_expansion(&mut self, path: PathBuf) {
        let mut sidebar = self.sidebar.write();
        sidebar.toggle_expansion(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_default() {
        let sidebar = Sidebar::default();

        assert!(!sidebar.open);
        assert_eq!(sidebar.width, 280.0);
        assert!(!sidebar.show_all_files);
        assert!(sidebar.expanded_dirs.is_empty());
    }

    #[test]
    fn test_sidebar_toggle_expansion() {
        let mut sidebar = Sidebar::default();
        let path = PathBuf::from("/test/dir");

        // Initially empty
        assert!(!sidebar.expanded_dirs.contains(&path));

        // First toggle - expands
        sidebar.toggle_expansion(path.clone());
        assert!(sidebar.expanded_dirs.contains(&path));

        // Second toggle - collapses
        sidebar.toggle_expansion(path.clone());
        assert!(!sidebar.expanded_dirs.contains(&path));
    }

    #[test]
    fn test_sidebar_toggle_multiple_paths() {
        let mut sidebar = Sidebar::default();
        let path1 = PathBuf::from("/test/dir1");
        let path2 = PathBuf::from("/test/dir2");

        sidebar.toggle_expansion(path1.clone());
        sidebar.toggle_expansion(path2.clone());

        assert!(sidebar.expanded_dirs.contains(&path1));
        assert!(sidebar.expanded_dirs.contains(&path2));

        sidebar.toggle_expansion(path1.clone());

        assert!(!sidebar.expanded_dirs.contains(&path1));
        assert!(sidebar.expanded_dirs.contains(&path2));
    }
}
