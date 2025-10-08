//! Tauri commands for main window functionality
//!
//! Provides the backend API for main window operations including
//! publishing, editing, and syndication.

use crate::preview::PreviewState;

/// Validate and prepare a publish request from preview window
pub fn validate_publish_request(state: &PreviewState) -> Result<(), String> {
    if state.is_published {
        return Err("Content has already been published to this platform. Use a different platform or edit and rebuild to publish changes.".to_string());
    }
    
    if !state.folder_path.exists() {
        return Err("Source folder no longer exists. Cannot publish.".to_string());
    }
    
    // Check if the site has been built (should exist since preview window opened after build)
    let site_path = state.folder_path.join(".moss/docs");
    if !site_path.exists() {
        return Err("Built site not found. Please rebuild the site first.".to_string());
    }
    
    // Check if built site contains files
    let has_built_files = std::fs::read_dir(&site_path)
        .map_err(|e| format!("Cannot read built site directory: {}", e))?
        .any(|entry| {
            if let Ok(entry) = entry {
                entry.path().is_file()
            } else {
                false
            }
        });
    
    if !has_built_files {
        return Err("Built site directory is empty. Please rebuild the site first.".to_string());
    }
    
    Ok(())
}




/// Tauri command: Publish content from main window to hosting platform
///
/// This handles the "Publish" step (upload to host), not the "Build" step.
/// The site should already be built and preview server running when this is called.
/// Note: This command is currently not used by the folder picker flow but kept for potential future use.
#[tauri::command]
#[specta::specta]
pub async fn publish_from_preview(
    _platform: Option<String>,
) -> Result<String, String> {
    // TODO: Implement publishing for main window flow when needed
    Err("Publishing not yet implemented for main window flow".to_string())
}

/// Tauri command: Setup GitHub repository and configure remote
///
/// Creates a new GitHub repository and configures it as the origin remote
/// for the project. This handles the "Connect to GitHub" button action.
/// Note: This command is currently not used by the folder picker flow but kept for potential future use.
#[tauri::command]
#[specta::specta]
pub async fn setup_github_repository(
    _repo_name: String,
    _is_public: bool,
    _github_token: String,
) -> Result<String, String> {
    // TODO: Implement GitHub setup for main window flow when needed
    Err("GitHub setup not yet implemented for main window flow".to_string())
}

/// Tauri command: Refresh publish button state
///
/// Re-checks git configuration and updates the publish button state.
/// Useful after external git operations or manual git setup.
/// Note: This command is currently not used by the folder picker flow but kept for potential future use.
#[tauri::command]
#[specta::specta]
pub async fn refresh_publish_state() -> Result<String, String> {
    // TODO: Implement state refresh for main window flow when needed
    Err("State refresh not yet implemented for main window flow".to_string())
}

/// Tauri command: Open folder in system editor
/// Note: This command is currently not used by the folder picker flow but kept for potential future use.
#[tauri::command]
#[specta::specta]
pub async fn open_editor_from_preview() -> Result<String, String> {
    // TODO: Implement folder opening for main window flow when needed
    Err("Folder opening not yet implemented for main window flow".to_string())
}

/// Tauri command: Add syndication target to preview
/// Note: This command is currently not used by the folder picker flow but kept for potential future use.
#[tauri::command]
#[specta::specta]
pub async fn add_syndication_target(
    _target: String,
) -> Result<String, String> {
    // TODO: Implement syndication for main window flow when needed
    Err("Syndication not yet implemented for main window flow".to_string())
}

/// Tauri command: Remove syndication target from preview
/// Note: This command is currently not used by the folder picker flow but kept for potential future use.
#[tauri::command]
#[specta::specta]
pub async fn remove_syndication_target(
    _target: String,
) -> Result<String, String> {
    // TODO: Implement syndication removal for main window flow when needed
    Err("Syndication removal not yet implemented for main window flow".to_string())
}

/// Tauri command: Get preview window state
/// Note: This command is currently not used by the folder picker flow but kept for potential future use.
#[tauri::command]
#[specta::specta]
pub async fn get_preview_state() -> Result<PreviewState, String> {
    // TODO: Implement state retrieval for main window flow when needed
    Err("State retrieval not yet implemented for main window flow".to_string())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_publish_prevents_double_publish() {
        // Create a temporary directory structure for testing
        let temp_dir = std::env::temp_dir().join("moss_test_validate_publish");
        let site_dir = temp_dir.join(".moss/docs");
        std::fs::create_dir_all(&site_dir).unwrap();
        std::fs::write(site_dir.join("index.html"), "test content").unwrap();
        
        let mut state = PreviewState::new(temp_dir.clone());
        
        // First publish should be allowed (site exists and not yet published)
        assert!(validate_publish_request(&state).is_ok());
        
        // Mark as published
        state.mark_published("moss.pub");
        
        // Second publish should be prevented
        let result = validate_publish_request(&state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already been published"));
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_validate_publish_checks_built_site_exists() {
        // Test with folder that exists but has no built site
        let temp_dir = std::env::temp_dir().join("moss_test_no_site");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let state = PreviewState::new(temp_dir.clone());
        
        let result = validate_publish_request(&state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Built site not found"));
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }



}