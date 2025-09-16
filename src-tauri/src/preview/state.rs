//! Preview window state management
//!
//! Manages the state of preview windows including publication status,
//! syndication targets, and folder paths.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::preview::git::{GitRemoteInfo, detect_git_remote, has_git_repository};

/// Represents the state of a preview window
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PreviewState {
    /// Port number of the running preview server
    pub server_port: Option<u16>,
    /// Path to the source folder being previewed
    pub folder_path: PathBuf,
    /// Whether this content has been published
    pub is_published: bool,
    /// List of syndication targets (platforms where content is shared)
    pub syndication_targets: Vec<String>,
    /// Unique identifier for this preview session
    pub id: String,
    /// Git remote information if available
    pub git_remote: Option<GitRemoteInfo>,
    /// Current publish button state
    pub publish_button_state: PublishButtonState,
}

/// Represents the different states of the publish button
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum PublishButtonState {
    /// No git repository found - show "Setup Git" 
    SetupGit,
    /// Git repository exists but no GitHub remote - show "Connect to GitHub"
    ConnectToGitHub,
    /// GitHub remote exists and ready to publish - show "Publish"
    Publish,
    /// Content has been published - show "Published" with URL
    Published(String),
}

/// Metadata extracted from the preview content
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PreviewMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub created_date: Option<String>,
}

impl PreviewState {
    /// Create a new preview state with git detection
    pub fn new(folder_path: PathBuf) -> Self {
        let id = format!("preview_{}", chrono::Utc::now().timestamp());

        // Detect git configuration
        let git_remote = detect_git_remote(&folder_path).unwrap_or(None);
        let publish_button_state = determine_publish_button_state(&folder_path, &git_remote);

        Self {
            server_port: None,
            folder_path,
            is_published: false,
            syndication_targets: Vec::new(),
            id,
            git_remote,
            publish_button_state,
        }
    }

    /// Set the server port for this preview state
    pub fn set_server_port(&mut self, port: u16) {
        self.server_port = Some(port);
    }

    /// Get the preview URL from the server port
    pub fn get_preview_url(&self) -> Option<String> {
        self.server_port.map(|port| format!("http://localhost:{}", port))
    }

    /// Mark this preview as published to a platform
    pub fn mark_published(&mut self, platform: &str) {
        self.is_published = true;
        if !self.syndication_targets.contains(&platform.to_string()) {
            self.syndication_targets.push(platform.to_string());
        }
    }

    /// Check if content can be published
    #[allow(dead_code)]
    pub fn can_publish(&self) -> bool {
        // Can publish if not already published or if folder has been modified
        !self.is_published && self.folder_path.exists()
    }

    /// Get the path for editing (returns the source folder)
    pub fn get_edit_path(&self) -> PathBuf {
        self.folder_path.clone()
    }

    /// Check if folder has been modified since last publish
    #[allow(dead_code)]
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
    
    /// Refresh the publish button state based on current git configuration
    pub fn refresh_publish_state(&mut self) {
        self.git_remote = detect_git_remote(&self.folder_path).unwrap_or(None);
        self.publish_button_state = determine_publish_button_state(&self.folder_path, &self.git_remote);
    }
}

/// Determines the appropriate publish button state based on git configuration
/// 
/// Logic:
/// - If no .git directory: SetupGit
/// - If git but no remote: ConnectToGitHub  
/// - If git with GitHub remote: Publish
/// - Special handling for published state
fn determine_publish_button_state(folder_path: &PathBuf, git_remote: &Option<GitRemoteInfo>) -> PublishButtonState {
    // Check if git repository exists
    if !has_git_repository(folder_path) {
        return PublishButtonState::SetupGit;
    }
    
    // Check if we have a remote configured
    match git_remote {
        Some(remote) if remote.is_github => PublishButtonState::Publish,
        Some(_) => PublishButtonState::ConnectToGitHub, // Non-GitHub remote
        None => PublishButtonState::ConnectToGitHub,    // No remote
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_preview_state_initialization() {
        let folder_path = PathBuf::from("/test/folder");

        let state = PreviewState::new(folder_path.clone());

        assert_eq!(state.server_port, None);
        assert_eq!(state.folder_path, folder_path);
        assert!(!state.is_published);
        assert!(state.syndication_targets.is_empty());
        assert!(state.id.starts_with("preview_"));
        assert_eq!(state.git_remote, None);
        assert_eq!(state.publish_button_state, PublishButtonState::SetupGit);
    }

    #[test]
    fn test_mark_published_updates_state() {
        let mut state = PreviewState::new(PathBuf::from("/test"));

        assert!(!state.is_published);
        assert!(state.syndication_targets.is_empty());

        state.mark_published("moss.pub");

        assert!(state.is_published);
        assert_eq!(state.syndication_targets.len(), 1);
        assert!(state.syndication_targets.contains(&"moss.pub".to_string()));
    }

    #[test]
    fn test_mark_published_prevents_duplicates() {
        let mut state = PreviewState::new(PathBuf::from("/test"));

        state.mark_published("moss.pub");
        state.mark_published("moss.pub"); // Duplicate

        assert_eq!(state.syndication_targets.len(), 1);
    }

    #[test]
    fn test_can_publish_logic() {
        // Use current directory which should always exist
        let folder_path = std::env::current_dir().unwrap();
        let mut state = PreviewState::new(folder_path);

        // Can publish initially
        assert!(state.can_publish());

        // Cannot publish after marking as published
        state.mark_published("moss.pub");
        assert!(!state.can_publish());
    }

    #[test]
    fn test_can_publish_nonexistent_folder() {
        let folder_path = PathBuf::from("/nonexistent/folder");
        let state = PreviewState::new(folder_path);

        // Cannot publish if folder doesn't exist
        assert!(!state.can_publish());
    }

    #[test]
    fn test_get_edit_path_resolution() {
        let folder_path = PathBuf::from("/test/folder");
        let state = PreviewState::new(folder_path.clone());

        assert_eq!(state.get_edit_path(), folder_path);
    }

    #[test]
    fn test_add_syndication_target_success() {
        let mut state = PreviewState::new(PathBuf::from("/test"));

        let result = state.add_syndication_target("twitter".to_string());
        assert!(result.is_ok());
        assert!(state.syndication_targets.contains(&"twitter".to_string()));
    }

    #[test]
    fn test_add_syndication_target_empty() {
        let mut state = PreviewState::new(PathBuf::from("/test"));

        let result = state.add_syndication_target("".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Syndication target cannot be empty");
    }

    #[test]
    fn test_add_syndication_target_duplicate() {
        let mut state = PreviewState::new(PathBuf::from("/test"));

        state.add_syndication_target("twitter".to_string()).unwrap();
        let result = state.add_syndication_target("twitter".to_string());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Syndication target already exists");
    }

    #[test]
    fn test_remove_syndication_target() {
        let mut state = PreviewState::new(PathBuf::from("/test"));

        state.add_syndication_target("twitter".to_string()).unwrap();
        assert!(state.syndication_targets.contains(&"twitter".to_string()));

        let removed = state.remove_syndication_target("twitter");
        assert!(removed);
        assert!(!state.syndication_targets.contains(&"twitter".to_string()));

        // Try to remove again
        let removed_again = state.remove_syndication_target("twitter");
        assert!(!removed_again);
    }

    #[test]
    fn test_server_port_and_url_methods() {
        let mut state = PreviewState::new(PathBuf::from("/test"));

        // Initially no server port
        assert_eq!(state.server_port, None);
        assert_eq!(state.get_preview_url(), None);

        // Set server port
        state.set_server_port(8080);
        assert_eq!(state.server_port, Some(8080));
        assert_eq!(state.get_preview_url(), Some("http://localhost:8080".to_string()));
    }
}