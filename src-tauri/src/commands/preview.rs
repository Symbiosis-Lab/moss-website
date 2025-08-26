//! Tauri commands for preview window functionality
//!
//! Provides the backend API for preview window operations including
//! window creation, publishing, editing, and syndication.

use crate::preview::*;
use crate::preview::ipc::validate_syndication_target;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};

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

/// Prepare syndication data for publishing
pub fn prepare_syndication_data(
    _state: &PreviewState, 
    targets: Vec<String>
) -> Result<Vec<String>, String> {
    // Validate syndication targets
    let valid_targets: Vec<String> = targets
        .into_iter()
        .filter(|target| validate_syndication_target(target).is_ok())
        .collect();
    
    if valid_targets.is_empty() {
        return Err("No valid syndication targets provided".to_string());
    }
    
    Ok(valid_targets)
}

/// Extract preview metadata from folder contents
pub fn format_preview_metadata(folder_path: &Path) -> Result<PreviewMetadata, String> {
    let mut metadata = PreviewMetadata {
        title: None,
        description: None,
        author: None,
        created_date: None,
    };
    
    // Look for common metadata files
    let metadata_files = ["index.md", "README.md", "about.md"];
    
    for filename in &metadata_files {
        let file_path = folder_path.join(filename);
        if file_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                // Simple metadata extraction from first few lines
                let lines: Vec<&str> = content.lines().take(10).collect();
                
                // Look for title in first heading
                for line in &lines {
                    if line.starts_with("# ") {
                        metadata.title = Some(line[2..].trim().to_string());
                        break;
                    }
                }
                
                break; // Use first found file
            }
        }
    }
    
    // Use folder name as fallback title if not found in any file
    if metadata.title.is_none() {
        metadata.title = folder_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string());
    }
    
    // Set creation date to now if not found
    metadata.created_date = Some(chrono::Utc::now().format("%Y-%m-%d").to_string());
    
    Ok(metadata)
}

/// Tauri command: Open preview window for a folder
#[tauri::command]
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
    
    // Build preview URL (assuming local server is running)
    let preview_url = build_preview_url("http://localhost:8080", &path);
    
    // Create preview state
    let state = PreviewState::new(path, preview_url);
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
pub async fn publish_from_preview(
    manager: PreviewManagerState<'_>,
    preview_id: String,
    platform: Option<String>,
) -> Result<String, String> {
    let mut state = manager.get_window(&preview_id)
        .ok_or("Preview window not found")?;
    
    // Validate publish request
    validate_publish_request(&state)?;
    
    // TODO: Implement actual deployment to hosting platforms
    // For now, just simulate the publish step
    let platform_name = platform.unwrap_or_else(|| "moss.pub".to_string());
    
    // Simulate deployment process
    // In Phase 1, this will actually deploy to moss.pub or other platforms
    let site_path = state.folder_path.join(".moss/site");
    
    if !site_path.exists() {
        return Err("Built site not found. Please rebuild the site first.".to_string());
    }
    
    // Mock deployment success
    let mock_url = format!("https://{}/sites/{}", platform_name, 
        state.folder_path.file_name()
            .unwrap_or_default()
            .to_string_lossy());
    
    // Update state to mark as published
    state.mark_published(&platform_name);
    
    // Update stored state
    manager.update_window(&preview_id, state)?;
    
    Ok(format!("Published to {}: {}", platform_name, mock_url))
}

/// Tauri command: Open folder in system editor
#[tauri::command]
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
pub async fn get_preview_state(
    manager: PreviewManagerState<'_>,
    preview_id: String,
) -> Result<PreviewState, String> {
    manager.get_window(&preview_id)
        .ok_or("Preview window not found".to_string())
}

/// Tauri command: Close preview window
#[tauri::command]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_publish_prevents_double_publish() {
        // Create a temporary directory structure for testing
        let temp_dir = std::env::temp_dir().join("moss_test_validate_publish");
        let site_dir = temp_dir.join(".moss/site");
        std::fs::create_dir_all(&site_dir).unwrap();
        std::fs::write(site_dir.join("index.html"), "test content").unwrap();
        
        let mut state = PreviewState::new(
            temp_dir.clone(),
            "http://localhost:8080".to_string()
        );
        
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
        
        let state = PreviewState::new(
            temp_dir.clone(),
            "http://localhost:8080".to_string()
        );
        
        let result = validate_publish_request(&state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Built site not found"));
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_prepare_syndication_filters_invalid_targets() {
        let state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        let targets = vec![
            "twitter".to_string(),        // Valid
            "platform with spaces".to_string(), // Invalid
            "dev.to".to_string(),         // Valid
            "platform/with/slashes".to_string(), // Invalid
        ];
        
        let result = prepare_syndication_data(&state, targets).unwrap();
        
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"twitter".to_string()));
        assert!(result.contains(&"dev.to".to_string()));
    }

    #[test]
    fn test_prepare_syndication_no_valid_targets() {
        let state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        let targets = vec![
            "invalid target".to_string(),
            "another/invalid".to_string(),
        ];
        
        let result = prepare_syndication_data(&state, targets);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No valid syndication targets"));
    }

    #[test]
    fn test_format_preview_metadata_fallback_title() {
        // Test with current directory (should exist)
        let current_dir = std::env::current_dir().unwrap();
        let metadata = format_preview_metadata(&current_dir).unwrap();
        
        // Title should be set either from file content or folder name
        assert!(metadata.title.is_some(), "Title should be set from folder name as fallback");
        assert!(metadata.created_date.is_some(), "Created date should always be set");
        
        // Should have some reasonable title (either from file or folder name)
        let title = metadata.title.unwrap();
        assert!(!title.is_empty(), "Title should not be empty");
    }
}