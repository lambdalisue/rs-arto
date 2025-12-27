use std::path::{Path, PathBuf};

/// Manages navigation history for markdown files
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryManager {
    history: Vec<PathBuf>,
    current_index: usize,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryManager {
    /// Create a new empty history manager
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_index: 0,
        }
    }

    /// Push a new file to the history
    /// Clears forward history if not at the end
    pub fn push(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        // Don't add duplicate if it's the same as current
        if let Some(current) = self.current() {
            if current == path {
                return;
            }
        }

        if self.history.is_empty() {
            // First item
            self.history.push(path);
            self.current_index = 0;
        } else {
            // Remove all items after current index (forward history)
            self.history.truncate(self.current_index + 1);
            // Add new path
            self.history.push(path);
            self.current_index += 1;
        }
    }

    /// Check if we can go back
    pub fn can_go_back(&self) -> bool {
        self.current_index > 0
    }

    /// Check if we can go forward
    pub fn can_go_forward(&self) -> bool {
        !self.history.is_empty() && self.current_index < self.history.len().saturating_sub(1)
    }

    /// Go back in history, returns the previous path
    pub fn go_back(&mut self) -> Option<&Path> {
        if !self.history.is_empty() && self.current_index > 0 {
            self.current_index -= 1;
            return self.current();
        }
        None
    }

    /// Go forward in history, returns the next path
    pub fn go_forward(&mut self) -> Option<&Path> {
        if !self.history.is_empty() && self.current_index < self.history.len().saturating_sub(1) {
            self.current_index += 1;
            return self.current();
        }
        None
    }

    /// Get the current file path
    pub fn current(&self) -> Option<&Path> {
        if !self.history.is_empty() {
            return self.history.get(self.current_index).map(|v| v.as_path());
        }
        None
    }

    /// Get the history length
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if history is empty
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_manager() {
        let manager = HistoryManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.current(), None);
        assert!(!manager.can_go_back());
        assert!(!manager.can_go_forward());
    }

    #[test]
    fn test_push_first_item() {
        let mut manager = HistoryManager::new();
        let path = Path::new("/test/file1.md");
        manager.push(path);

        assert_eq!(manager.len(), 1);
        assert_eq!(manager.current(), Some(path));
        assert!(!manager.can_go_back());
        assert!(!manager.can_go_forward());
    }

    #[test]
    fn test_push_multiple_items() {
        let mut manager = HistoryManager::new();
        let path1 = Path::new("/test/file1.md");
        let path2 = Path::new("/test/file2.md");
        let path3 = Path::new("/test/file3.md");

        manager.push(path1);
        manager.push(path2);
        manager.push(path3);

        assert_eq!(manager.len(), 3);
        assert_eq!(manager.current(), Some(path3));
        assert!(manager.can_go_back());
        assert!(!manager.can_go_forward());
    }

    #[test]
    fn test_go_back() {
        let mut manager = HistoryManager::new();
        let path1 = Path::new("/test/file1.md");
        let path2 = Path::new("/test/file2.md");

        manager.push(path1);
        manager.push(path2);

        let back = manager.go_back();
        assert_eq!(back, Some(path1));
        assert_eq!(manager.current(), Some(path1));
        assert!(!manager.can_go_back());
        assert!(manager.can_go_forward());
    }

    #[test]
    fn test_go_forward() {
        let mut manager = HistoryManager::new();
        let path1 = Path::new("/test/file1.md");
        let path2 = Path::new("/test/file2.md");

        manager.push(path1);
        manager.push(path2);
        manager.go_back();

        let forward = manager.go_forward();
        assert_eq!(forward, Some(path2));
        assert_eq!(manager.current(), Some(path2));
        assert!(manager.can_go_back());
        assert!(!manager.can_go_forward());
    }

    #[test]
    fn test_push_clears_forward_history() {
        let mut manager = HistoryManager::new();
        let path1 = Path::new("/test/file1.md");
        let path2 = Path::new("/test/file2.md");
        let path3 = Path::new("/test/file3.md");

        manager.push(path1);
        manager.push(path2);
        manager.go_back();

        // Now push a new path, should clear file2 from history
        manager.push(path3);

        assert_eq!(manager.len(), 2);
        assert_eq!(manager.current(), Some(path3));
        assert!(manager.can_go_back());
        assert!(!manager.can_go_forward());
    }

    #[test]
    fn test_push_duplicate_does_nothing() {
        let mut manager = HistoryManager::new();
        let path = PathBuf::from("/test/file1.md");

        manager.push(path.clone());
        manager.push(path.clone());

        assert_eq!(manager.len(), 1);
    }
}
