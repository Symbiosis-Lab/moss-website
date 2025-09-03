//! Preview window state management
//!
//! Manages the state of preview windows including publication status,
//! syndication targets, and folder paths.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Represents the state of a preview window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewState {
    /// The preview URL being displayed
    pub url: String,
    /// Path to the source folder being previewed
    pub folder_path: PathBuf,
    /// Whether this content has been published
    pub is_published: bool,
    /// List of syndication targets (platforms where content is shared)
    pub syndication_targets: Vec<String>,
    /// Unique identifier for this preview session
    pub id: String,
}

/// Metadata extracted from the preview content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub created_date: Option<String>,
}

impl PreviewState {
    /// Create a new preview state
    pub fn new(folder_path: PathBuf, url: String) -> Self {
        let id = format!("preview_{}", chrono::Utc::now().timestamp());
        
        Self {
            url,
            folder_path,
            is_published: false,
            syndication_targets: Vec::new(),
            id,
        }
    }

    /// Mark this preview as published to a platform
    pub fn mark_published(&mut self, platform: &str) {
        self.is_published = true;
        if !self.syndication_targets.contains(&platform.to_string()) {
            self.syndication_targets.push(platform.to_string());
        }
    }

    /// Check if content can be published
    pub fn can_publish(&self) -> bool {
        // Can publish if not already published or if folder has been modified
        !self.is_published && self.folder_path.exists()
    }

    /// Get the path for editing (returns the source folder)
    pub fn get_edit_path(&self) -> PathBuf {
        self.folder_path.clone()
    }

    /// Check if folder has been modified since last publish
    pub fn is_modified_since_publish(&self) -> Result<bool, std::io::Error> {
        if !self.is_published {
            return Ok(true);
        }

        // For now, return true - in future could check file timestamps
        Ok(true)
    }

    /// Add a syndication target
    pub fn add_syndication_target(&mut self, target: String) -> Result<(), String> {
        if target.is_empty() {
            return Err("Syndication target cannot be empty".to_string());
        }

        if self.syndication_targets.contains(&target) {
            return Err("Syndication target already exists".to_string());
        }

        self.syndication_targets.push(target);
        Ok(())
    }

    /// Remove a syndication target
    pub fn remove_syndication_target(&mut self, target: &str) -> bool {
        if let Some(pos) = self.syndication_targets.iter().position(|x| x == target) {
            self.syndication_targets.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_preview_state_initialization() {
        let folder_path = PathBuf::from("/test/folder");
        let url = "http://localhost:8080".to_string();
        
        let state = PreviewState::new(folder_path.clone(), url.clone());
        
        assert_eq!(state.url, url);
        assert_eq!(state.folder_path, folder_path);
        assert!(!state.is_published);
        assert!(state.syndication_targets.is_empty());
        assert!(state.id.starts_with("preview_"));
    }

    #[test]
    fn test_mark_published_updates_state() {
        let mut state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        assert!(!state.is_published);
        assert!(state.syndication_targets.is_empty());
        
        state.mark_published("moss.pub");
        
        assert!(state.is_published);
        assert_eq!(state.syndication_targets.len(), 1);
        assert!(state.syndication_targets.contains(&"moss.pub".to_string()));
    }

    #[test]
    fn test_mark_published_prevents_duplicates() {
        let mut state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        state.mark_published("moss.pub");
        state.mark_published("moss.pub"); // Duplicate
        
        assert_eq!(state.syndication_targets.len(), 1);
    }

    #[test]
    fn test_can_publish_logic() {
        // Use current directory which should always exist
        let folder_path = std::env::current_dir().unwrap();
        let mut state = PreviewState::new(
            folder_path,
            "http://localhost:8080".to_string()
        );
        
        // Can publish initially
        assert!(state.can_publish());
        
        // Cannot publish after marking as published
        state.mark_published("moss.pub");
        assert!(!state.can_publish());
    }

    #[test]
    fn test_can_publish_nonexistent_folder() {
        let folder_path = PathBuf::from("/nonexistent/folder");
        let state = PreviewState::new(
            folder_path,
            "http://localhost:8080".to_string()
        );
        
        // Cannot publish if folder doesn't exist
        assert!(!state.can_publish());
    }

    #[test]
    fn test_get_edit_path_resolution() {
        let folder_path = PathBuf::from("/test/folder");
        let state = PreviewState::new(
            folder_path.clone(),
            "http://localhost:8080".to_string()
        );
        
        assert_eq!(state.get_edit_path(), folder_path);
    }

    #[test]
    fn test_add_syndication_target_success() {
        let mut state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        let result = state.add_syndication_target("twitter".to_string());
        assert!(result.is_ok());
        assert!(state.syndication_targets.contains(&"twitter".to_string()));
    }

    #[test]
    fn test_add_syndication_target_empty() {
        let mut state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        let result = state.add_syndication_target("".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Syndication target cannot be empty");
    }

    #[test]
    fn test_add_syndication_target_duplicate() {
        let mut state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        state.add_syndication_target("twitter".to_string()).unwrap();
        let result = state.add_syndication_target("twitter".to_string());
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Syndication target already exists");
    }

    #[test]
    fn test_remove_syndication_target() {
        let mut state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        state.add_syndication_target("twitter".to_string()).unwrap();
        assert!(state.syndication_targets.contains(&"twitter".to_string()));
        
        let removed = state.remove_syndication_target("twitter");
        assert!(removed);
        assert!(!state.syndication_targets.contains(&"twitter".to_string()));
        
        // Try to remove again
        let removed_again = state.remove_syndication_target("twitter");
        assert!(!removed_again);
    }
}