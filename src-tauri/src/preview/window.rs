//! Preview window lifecycle management
//!
//! Handles creation, configuration, and lifecycle of preview windows
//! with proper security settings and IPC setup.

use tauri::{AppHandle, Manager, TitleBarStyle, WebviewWindowBuilder, WebviewUrl};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::preview::PreviewState;

/// Global store of active preview windows
pub struct PreviewWindowManager {
    windows: Mutex<HashMap<String, PreviewState>>,
}

impl PreviewWindowManager {
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_window(&self, state: PreviewState) {
        let mut windows = self.windows.lock().unwrap();
        windows.insert(state.id.clone(), state);
    }

    pub fn get_window(&self, id: &str) -> Option<PreviewState> {
        let windows = self.windows.lock().unwrap();
        windows.get(id).cloned()
    }

    pub fn update_window(&self, id: &str, state: PreviewState) -> Result<(), String> {
        let mut windows = self.windows.lock().unwrap();
        if windows.contains_key(id) {
            windows.insert(id.to_string(), state);
            Ok(())
        } else {
            Err(format!("Preview window {} not found", id))
        }
    }

    pub fn remove_window(&self, id: &str) -> bool {
        let mut windows = self.windows.lock().unwrap();
        windows.remove(id).is_some()
    }
}

/// Configuration for preview window creation
pub struct PreviewWindowConfig {
    pub width: f64,
    pub height: f64,
    pub resizable: bool,
    pub always_on_top: bool,
}

impl Default for PreviewWindowConfig {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            resizable: true,
            always_on_top: false,
        }
    }
}

/// Create a new preview window
pub fn create_preview_window(
    app: &AppHandle,
    state: PreviewState,
    config: Option<PreviewWindowConfig>,
) -> Result<String, String> {
    let config = config.unwrap_or_default();
    let window_id = format!("preview_{}", state.id);
    
    // Load the preview URL directly
    let window_url = WebviewUrl::External(state.url.parse()
        .map_err(|e| format!("Invalid preview URL: {}", e))?);

    let mut window_builder = WebviewWindowBuilder::new(app, &window_id, window_url)
        .decorations(true)
        .title("")
        .inner_size(config.width, config.height)
        .resizable(config.resizable)
        .always_on_top(config.always_on_top);
    
    #[cfg(target_os = "macos")]
    {
        window_builder = window_builder.title_bar_style(TitleBarStyle::Visible);
    }
    
    let _window = window_builder
        .build()
        .map_err(|e| format!("Failed to create preview window: {}", e))?;

    Ok(window_id)
}

/// Close a preview window
pub fn close_preview_window(app: &AppHandle, preview_id: &str) -> Result<(), String> {
    let window_id = format!("preview_{}", preview_id);
    
    if let Some(window) = app.get_webview_window(&window_id) {
        window.close().map_err(|e| format!("Failed to close window: {}", e))?;
        Ok(())
    } else {
        Err(format!("Preview window {} not found", preview_id))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_preview_window_manager_add_get() {
        let manager = PreviewWindowManager::new();
        let state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        let state_id = state.id.clone();
        
        manager.add_window(state);
        
        let retrieved = manager.get_window(&state_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, state_id);
    }

    #[test]
    fn test_preview_window_manager_update() {
        let manager = PreviewWindowManager::new();
        let mut state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        let state_id = state.id.clone();
        
        manager.add_window(state.clone());
        
        // Update the state
        state.mark_published("moss.pub");
        let result = manager.update_window(&state_id, state);
        assert!(result.is_ok());
        
        // Verify update
        let retrieved = manager.get_window(&state_id).unwrap();
        assert!(retrieved.is_published);
    }

    #[test]
    fn test_preview_window_manager_update_nonexistent() {
        let manager = PreviewWindowManager::new();
        let state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        
        let result = manager.update_window("nonexistent", state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_preview_window_manager_remove() {
        let manager = PreviewWindowManager::new();
        let state = PreviewState::new(
            PathBuf::from("/test"),
            "http://localhost:8080".to_string()
        );
        let state_id = state.id.clone();
        
        manager.add_window(state);
        
        // Remove window
        let removed = manager.remove_window(&state_id);
        assert!(removed);
        
        // Verify removal
        let retrieved = manager.get_window(&state_id);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_preview_window_config_default() {
        let config = PreviewWindowConfig::default();
        
        assert_eq!(config.width, 1200.0);
        assert_eq!(config.height, 800.0);
        assert!(config.resizable);
        assert!(!config.always_on_top);
    }
}