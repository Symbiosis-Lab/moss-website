//! Preview window lifecycle management
//!
//! Handles creation, configuration, and lifecycle of preview windows
//! with proper security settings and IPC setup.

use tauri::{AppHandle, Manager, WebviewWindowBuilder, WebviewUrl};
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
    pub title: String,
    pub resizable: bool,
    pub always_on_top: bool,
}

impl Default for PreviewWindowConfig {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            title: "moss Preview".to_string(),
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
    
    // Build the URL for the preview HTML page
    let preview_html_url = format!("preview.html?preview_id={}", state.id);
    
    let window_url = WebviewUrl::App(preview_html_url.into());

    let window = WebviewWindowBuilder::new(app, &window_id, window_url)
        .title(&config.title)
        .inner_size(config.width, config.height)
        .resizable(config.resizable)
        .always_on_top(config.always_on_top)
        .build()
        .map_err(|e| format!("Failed to create preview window: {}", e))?;

    // Configure window after creation
    window.eval(&format!(r#"
        // Store preview ID for IPC communication
        window.PREVIEW_ID = '{}';
        
        // Configure iframe security when DOM loads
        document.addEventListener('DOMContentLoaded', function() {{
            const iframe = document.getElementById('preview-iframe');
            if (iframe) {{
                // Set sandbox attributes for security
                iframe.setAttribute('sandbox', 'allow-same-origin allow-scripts allow-forms allow-popups');
                
                // Set additional security headers via JavaScript
                iframe.addEventListener('load', function() {{
                    try {{
                        // Note: We can't directly set CSP on iframe content from parent
                        // The local preview server should handle CSP headers
                        console.log('Preview iframe loaded successfully');
                    }} catch (e) {{
                        console.log('Preview iframe loaded (cross-origin restrictions apply)');
                    }}
                }});
            }}
        }});
    "#, state.id))
    .map_err(|e| format!("Failed to configure window: {}", e))?;

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

/// Update preview window with new URL
pub fn refresh_preview_window(
    app: &AppHandle,
    preview_id: &str,
    new_url: &str,
) -> Result<(), String> {
    let window_id = format!("preview_{}", preview_id);
    
    if let Some(window) = app.get_webview_window(&window_id) {
        // Update the iframe source via JavaScript
        let script = format!(r#"
            const iframe = document.getElementById('preview-iframe');
            if (iframe) {{
                iframe.src = '{}';
            }}
        "#, new_url);
        
        window.eval(&script)
            .map_err(|e| format!("Failed to refresh preview: {}", e))?;
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
        assert_eq!(config.title, "moss Preview");
        assert!(config.resizable);
        assert!(!config.always_on_top);
    }
}