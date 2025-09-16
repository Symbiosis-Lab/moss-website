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
    
    // Load the preview URL if available, otherwise load a loading page
    let window_url = if let Some(url) = state.get_preview_url() {
        WebviewUrl::External(url.parse()
            .map_err(|e| format!("Invalid preview URL: {}", e))?)
    } else {
        // Create a simple loading page if no server URL is available yet
        let loading_html = "data:text/html,<html><body style='display:flex;align-items:center;justify-content:center;height:100vh;font-family:sans-serif;'>🌿 Loading preview...</body></html>";
        WebviewUrl::External(loading_html.parse()
            .map_err(|e| format!("Invalid loading page URL: {}", e))?)
    };

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
    
    let window = window_builder
        .build()
        .map_err(|e| format!("Failed to create preview window: {}", e))?;

    // Add window close event handler to restore Accessory mode and stop server
    // When preview window closes, hide dock icon and return to menu bar only
    let app_handle = app.clone();
    let preview_state = state.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            println!("🔧 Preview window closed - cleaning up resources");

            // Stop the preview server if one is running
            if let Some(server_port) = preview_state.server_port {
                use crate::compile::stop_preview_server;
                if let Err(e) = stop_preview_server(server_port) {
                    eprintln!("⚠️ Failed to stop preview server on port {}: {}", server_port, e);
                } else {
                    println!("🔧 Preview server stopped on port {}", server_port);
                }
            }

            // Restore Accessory mode to hide dock icon
            #[cfg(target_os = "macos")]
            {
                if let Err(e) = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory) {
                    eprintln!("⚠️ Failed to restore activation policy: {}", e);
                } else {
                    println!("🔧 Dock icon hidden - moss returned to menu bar only");
                }
            }
        }
    });

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
        let state = PreviewState::new(PathBuf::from("/test"));
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
        let state = PreviewState::new(PathBuf::from("/test"));
        
        let result = manager.update_window("nonexistent", state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_preview_window_manager_remove() {
        let manager = PreviewWindowManager::new();
        let state = PreviewState::new(PathBuf::from("/test"));
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