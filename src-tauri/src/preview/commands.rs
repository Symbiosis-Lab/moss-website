//! Tauri commands for preview window functionality
//!
//! Provides the backend API for preview window operations including
//! window creation, publishing, editing, and syndication.

use crate::preview::{PreviewState, PreviewWindowManager, create_preview_window, close_preview_window};
use crate::preview::git::{create_github_repo_and_remote, sanitize_repo_name};
use crate::preview::github::deploy_to_github_pages;
use crate::preview::state::PublishButtonState;
use std::path::PathBuf;
use tauri::{AppHandle, State, Manager, Emitter};

/// Global state for preview window management
pub type PreviewManagerState<'a> = State<'a, PreviewWindowManager>;

/// Validate and prepare a publish request from preview window
pub fn validate_publish_request(state: &PreviewState) -> Result<(), String> {
    if state.is_published {
        return Err("Content has already been published to this platform. Use a different platform or edit and rebuild to publish changes.".to_string());
    }
    
    if !state.folder_path.exists() {
        return Err("Source folder no longer exists. Cannot publish.".to_string());
    }
    
    // Check if the site has been built (should exist since preview window opened after build)
    let site_path = state.folder_path.join(".moss/site");
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



/// Tauri command: Open preview window for a folder
#[tauri::command]
#[specta::specta]
pub async fn open_preview_window(
    app: AppHandle,
    manager: PreviewManagerState<'_>,
    folder_path: String,
) -> Result<String, String> {
    let path = PathBuf::from(&folder_path);
    
    if !path.exists() {
        return Err("Folder does not exist".to_string());
    }
    
    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    // Create preview state (server URL will be set later when server starts)
    let state = PreviewState::new(path);
    let preview_id = state.id.clone();
    
    // Create the window
    let _window_id = create_preview_window(&app, state.clone(), None)?;
    
    // Store state in manager
    manager.add_window(state);
    
    Ok(preview_id)
}

/// Tauri command: Publish content from preview window to hosting platform
///
/// This handles the "Publish" step (upload to host), not the "Build" step.
/// The site should already be built and preview server running when this is called.
#[tauri::command]
#[specta::specta]
pub async fn publish_from_preview(
    manager: PreviewManagerState<'_>,
    preview_id: String,
    platform: Option<String>,
) -> Result<String, String> {
    let mut state = manager.get_window(&preview_id)
        .ok_or("Preview window not found")?;
    
    // Validate publish request
    validate_publish_request(&state)?;
    
    // Check current publish button state to determine action
    match &state.publish_button_state {
        PublishButtonState::SetupGit => {
            return Err("Git repository not found. Please set up git first.".to_string());
        },
        PublishButtonState::ConnectToGitHub => {
            return Err("No GitHub remote configured. Please connect to GitHub first.".to_string());
        },
        PublishButtonState::Published(url) => {
            return Err(format!("Already published to: {}", url));
        },
        PublishButtonState::Publish => {
            // Proceed with publishing
        }
    }
    
    // Get GitHub remote information
    let git_remote = state.git_remote.as_ref()
        .ok_or("No git remote configured")?;
    
    if !git_remote.is_github {
        return Err("Only GitHub repositories are currently supported".to_string());
    }
    
    // Deploy to GitHub Pages
    match deploy_to_github_pages(&state.folder_path, git_remote).await {
        Ok(pages_url) => {
            // Update state to mark as published
            state.mark_published("GitHub Pages");
            state.publish_button_state = PublishButtonState::Published(pages_url.clone());
            
            // Update stored state
            manager.update_window(&preview_id, state)?;
            
            Ok(format!("Published to GitHub Pages: {}", pages_url))
        },
        Err(error) => Err(format!("Failed to publish to GitHub Pages: {}", error))
    }
}

/// Tauri command: Setup GitHub repository and configure remote
///
/// Creates a new GitHub repository and configures it as the origin remote
/// for the project. This handles the "Connect to GitHub" button action.
#[tauri::command]
#[specta::specta]
pub async fn setup_github_repository(
    manager: PreviewManagerState<'_>,
    preview_id: String,
    repo_name: String,
    is_public: bool,
    github_token: String,
) -> Result<String, String> {
    let mut state = manager.get_window(&preview_id)
        .ok_or("Preview window not found")?;
    
    // Validate inputs
    if repo_name.trim().is_empty() {
        return Err("Repository name cannot be empty".to_string());
    }
    
    let sanitized_name = sanitize_repo_name(&repo_name);
    if sanitized_name != repo_name {
        return Err(format!("Repository name '{}' contains invalid characters. Suggested name: '{}'", repo_name, sanitized_name));
    }
    
    // Create GitHub repository and set up remote
    match create_github_repo_and_remote(&state.folder_path, &repo_name, is_public, &github_token).await {
        Ok(git_remote) => {
            // Update state with new git remote info
            state.git_remote = Some(git_remote.clone());
            state.refresh_publish_state();
            
            // Store updated state
            manager.update_window(&preview_id, state)?;
            
            let visibility = if is_public { "public" } else { "private" };
            Ok(format!("Created {} repository: {}", visibility, git_remote.url))
        },
        Err(error) => Err(format!("Failed to create GitHub repository: {}", error))
    }
}

/// Tauri command: Refresh publish button state
///
/// Re-checks git configuration and updates the publish button state.
/// Useful after external git operations or manual git setup.
#[tauri::command]
#[specta::specta]
pub async fn refresh_publish_state(
    manager: PreviewManagerState<'_>,
    preview_id: String,
) -> Result<String, String> {
    let mut state = manager.get_window(&preview_id)
        .ok_or("Preview window not found")?;
    
    state.refresh_publish_state();
    manager.update_window(&preview_id, state.clone())?;
    
    let state_description = match state.publish_button_state {
        crate::preview::state::PublishButtonState::SetupGit => "No git repository found",
        crate::preview::state::PublishButtonState::ConnectToGitHub => "Git repository found, no GitHub remote",
        crate::preview::state::PublishButtonState::Publish => "Ready to publish to GitHub Pages",
        crate::preview::state::PublishButtonState::Published(ref url) => {
            return Ok(format!("Already published: {}", url));
        }
    };
    
    Ok(state_description.to_string())
}

/// Tauri command: Open folder in system editor
#[tauri::command]
#[specta::specta]
pub async fn open_editor_from_preview(
    manager: PreviewManagerState<'_>,
    preview_id: String,
) -> Result<String, String> {
    let state = manager.get_window(&preview_id)
        .ok_or("Preview window not found")?;
    
    let folder_path = state.get_edit_path();
    
    if !folder_path.exists() {
        return Err("Source folder no longer exists".to_string());
    }
    
    // Open folder in default file manager
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(format!("Opened folder: {}", folder_path.display()))
}

/// Tauri command: Add syndication target to preview
#[tauri::command]
#[specta::specta]
pub async fn add_syndication_target(
    manager: PreviewManagerState<'_>,
    preview_id: String,
    target: String,
) -> Result<String, String> {
    let mut state = manager.get_window(&preview_id)
        .ok_or("Preview window not found")?;
    
    state.add_syndication_target(target.clone())?;
    manager.update_window(&preview_id, state)?;
    
    Ok(format!("Added syndication target: {}", target))
}

/// Tauri command: Remove syndication target from preview
#[tauri::command]
#[specta::specta]
pub async fn remove_syndication_target(
    manager: PreviewManagerState<'_>,
    preview_id: String,
    target: String,
) -> Result<String, String> {
    let mut state = manager.get_window(&preview_id)
        .ok_or("Preview window not found")?;
    
    if state.remove_syndication_target(&target) {
        manager.update_window(&preview_id, state)?;
        Ok(format!("Removed syndication target: {}", target))
    } else {
        Err("Syndication target not found".to_string())
    }
}

/// Tauri command: Get preview window state
#[tauri::command]
#[specta::specta]
pub async fn get_preview_state(
    manager: PreviewManagerState<'_>,
    preview_id: String,
) -> Result<PreviewState, String> {
    manager.get_window(&preview_id)
        .ok_or("Preview window not found".to_string())
}

/// Tauri command: Close preview window
#[tauri::command]
#[specta::specta]
pub async fn close_preview_window_cmd(
    app: AppHandle,
    manager: PreviewManagerState<'_>,
    preview_id: String,
) -> Result<String, String> {
    // Close the window
    close_preview_window(&app, &preview_id)?;
    
    // Remove from manager
    if manager.remove_window(&preview_id) {
        Ok("Preview window closed".to_string())
    } else {
        Err("Preview window not found".to_string())
    }
}

/// Update the iframe source in the main window instead of creating separate preview windows
#[tauri::command]
#[specta::specta]
pub async fn update_main_window_preview(
    app: AppHandle,
    preview_url: String,
    folder_path: String,
) -> Result<String, String> {
    // Get the main window
    let main_window = app.get_webview_window("main")
        .ok_or("Main window not found")?;
    
    // Emit an event to the frontend with the new preview URL
    main_window.emit("preview-url-updated", serde_json::json!({
        "url": preview_url,
        "folder_path": folder_path
    })).map_err(|e| format!("Failed to emit preview update event: {}", e))?;
    
    Ok("Preview updated in main window".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_publish_prevents_double_publish() {
        // Create a temporary directory structure for testing
        let temp_dir = std::env::temp_dir().join("moss_test_validate_publish");
        let site_dir = temp_dir.join(".moss/site");
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