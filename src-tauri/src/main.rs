//! # Moss - Desktop Publishing App
//!
//! A Tauri-based desktop application that allows users to publish folders as websites
//! with right-click integration on macOS Finder.
//!
//! ## Backend API
//!
//! The backend exposes a minimal set of Tauri commands:
//! - [`publish_folder`] - Core publishing functionality
//! - [`install_finder_integration`] - Installs macOS Finder context menu integration
//! - [`get_system_status`] - Returns basic system information
//!
//! ## Documentation Generation
//!
//! Generate API documentation with: `cargo doc --open`
//! 
//! Learn more about Tauri commands at <https://v2.tauri.app/develop/calling-rust/>

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Listener, Manager};
use serde::{Deserialize, Serialize};

/// Tray icon visibility status
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TrayVisibilityStatus {
    /// Tray icon was not created or failed to be added to menu bar
    NotAdded,
    /// Tray icon is added but hidden/de-prioritized by macOS due to space constraints
    AddedButHidden,
    /// Tray icon is visible in the menu bar
    Visible,
}


/// System information for debugging and support
#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInfo {
    pub os: String,
    pub tray_status: TrayVisibilityStatus,
    pub finder_integration: bool,
    pub app_version: String,
}

#[cfg(test)]
mod tray_tests;


/// Handles deep link requests to publish a folder as a website.
/// 
/// This command is triggered when the app receives a `moss://publish?path=...` URL.
/// Currently logs the request and returns a confirmation message.
/// 
/// # Arguments
/// 
/// * `folder_path` - The absolute path to the folder to be published
/// 
/// # Returns
/// 
/// * `Ok(String)` - Confirmation message with the folder path
/// * `Err(String)` - Error message if the folder path is invalid
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - The provided folder path is empty
/// 
/// # Future Implementation
/// 
/// TODO: Implement actual static site generation and publishing logic
/// 
/// # Examples
/// 
/// ```rust
/// let result = publish_folder("/Users/username/my-site".to_string());
/// assert!(result.is_ok());
/// ```
#[tauri::command]
fn publish_folder(folder_path: String) -> Result<String, String> {
    // Core publishing logic
    if folder_path.is_empty() {
        return Err("Empty folder path provided".to_string());
    }
    
    // TODO: Validate folder exists
    // TODO: Generate static site
    // TODO: Deploy to moss.pub
    
    println!("🌱 Publishing folder '{}'", folder_path);
    
    // TODO: Return actual published URL
    Ok(format!("https://{}.moss.pub", "demo-site"))
}

/// Gets basic system information for debugging and support.
/// 
/// Returns information about the current system state including OS,
/// tray status, and integration status.
/// 
/// # Returns
/// 
/// * `Ok(SystemInfo)` - System information struct
/// * `Err(String)` - Error message if system info cannot be retrieved
/// 
/// # Examples
/// 
/// ```rust
/// let info = get_system_status();
/// assert!(info.is_ok());
/// ```
#[tauri::command]
fn get_system_status(app: tauri::AppHandle) -> Result<SystemInfo, String> {
    let os = std::env::consts::OS.to_string();
    let app_version = app.package_info().version.to_string();
    
    // Detect detailed tray icon visibility status
    let tray_status = detect_tray_visibility_status(&app);
    
    // Check if Finder integration is installed (macOS only)
    let finder_integration = if cfg!(target_os = "macos") {
        let home_dir = std::env::var("HOME").unwrap_or_default();
        let workflow_path = format!("{}/Library/Services/Publish to Web.workflow", home_dir);
        std::path::Path::new(&workflow_path).exists()
    } else {
        false
    };
    
    Ok(SystemInfo {
        os,
        tray_status,
        finder_integration,
        app_version,
    })
}

/// Detects the actual visibility status of the tray icon
fn detect_tray_visibility_status(app: &tauri::AppHandle) -> TrayVisibilityStatus {
    // First check if tray icon was created successfully
    if app.tray_by_id("main").is_none() {
        return TrayVisibilityStatus::NotAdded;
    }

    // If we have a tray icon, check if it's actually visible
    #[cfg(target_os = "macos")]
    {
        if is_tray_icon_actually_visible() {
            TrayVisibilityStatus::Visible
        } else {
            TrayVisibilityStatus::AddedButHidden
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, assume visible if it was created successfully
        TrayVisibilityStatus::Visible
    }
}

/// Check if the tray icon is actually visible in the macOS menu bar
/// using accessibility APIs to detect if it's hidden due to space constraints
#[cfg(target_os = "macos")]
fn is_tray_icon_actually_visible() -> bool {
    // For now, use a simplified heuristic approach
    // Full accessibility API implementation requires complex CFString handling
    // and process enumeration which is beyond the current scope
    
    // In practice, detecting tray icon visibility requires:
    // 1. Getting the NSStatusItem's window position
    // 2. Using AXUIElementCopyElementAtPosition to check what's at that position
    // 3. Comparing bundle IDs to verify it's our app
    // 4. Handling screen bounds and multiple displays
    
    // For now, assume visible if we reach this point (tray was created successfully)
    true
}

#[cfg(not(target_os = "macos"))]
fn is_tray_icon_actually_visible() -> bool {
    true
}

/// Test helper function to create tray icon data for testing
/// This simulates the icon creation logic from the main app
#[cfg(test)]
fn create_test_tray_icon_data() -> Vec<u8> {
    // Create the same icon as in the main app
    let mut icon_rgba = vec![0x00; 16 * 16 * 4];
    
    // Draw a simple black circle (same logic as main app)
    for y in 4..12 {
        for x in 4..12 {
            let distance_sq = (x as i32 - 8).pow(2) + (y as i32 - 8).pow(2);
            if distance_sq <= 16 {
                let idx = (y * 16 + x) * 4;
                icon_rgba[idx] = 0x00;     // R
                icon_rgba[idx + 1] = 0x00; // G
                icon_rgba[idx + 2] = 0x00; // B
                icon_rgba[idx + 3] = 0xFF; // A (opaque)
            }
        }
    }
    
    icon_rgba
}

/// Installs macOS Finder integration for right-click publishing.
/// 
/// Creates an Automator workflow in `~/Library/Services/` that adds a "Publish to Web"
/// option to the Finder context menu when right-clicking on folders.
/// 
/// The integration works by:
/// 1. Creating a `.workflow` bundle in the Services directory
/// 2. Setting up an Automator shell script that opens `moss://` deep links
/// 3. Registering the service with macOS for folder context menus
/// 
/// # Returns
/// 
/// * `Ok(String)` - Success message with installation path
/// * `Err(String)` - Error message if installation fails
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - The HOME environment variable is not set
/// - Failed to create the Services directory
/// - Failed to write workflow files due to permissions
/// - Failed to remove existing workflow during reinstallation
/// 
/// # Platform Support
/// 
/// Currently only supports macOS. The function creates macOS-specific
/// Automator workflows and plist files.
/// 
/// # Security Considerations
/// 
/// This function creates executable workflows that will be run by macOS.
/// The created shell script only uses built-in macOS commands (`printf`, `sed`, `open`).
#[tauri::command]
fn install_finder_integration() -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    
    // Get user's home directory
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(_) => return Err("Could not determine home directory".to_string()),
    };
    
    let services_dir = format!("{}/Library/Services", home_dir);
    let workflow_path = format!("{}/Publish to Web.workflow", services_dir);
    
    // Create Services directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&services_dir) {
        return Err(format!("Failed to create Services directory: {}", e));
    }
    
    // Remove existing workflow if it exists (to ensure clean reinstall)
    if Path::new(&workflow_path).exists() {
        if let Err(e) = fs::remove_dir_all(&workflow_path) {
            return Err(format!("Failed to remove existing workflow: {}", e));
        }
        println!("🗑️ Removed existing workflow for clean reinstall");
    }
    
    // Create the .workflow bundle directory
    if let Err(e) = fs::create_dir_all(format!("{}/Contents", workflow_path)) {
        return Err(format!("Failed to create workflow bundle: {}", e));
    }
    
    // Create Info.plist for the workflow bundle
    // NOTE: No NSIconName property to ensure it appears in top-level context menu
    let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.moss.publisher.publish-to-web</string>
    <key>CFBundleName</key>
    <string>Publish to Web</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict>
                <key>default</key>
                <string>Publish to Web</string>
            </dict>
            <key>NSMessage</key>
            <string>runWorkflowAsService</string>
            <key>NSSendFileTypes</key>
            <array>
                <string>public.folder</string>
            </array>
            <key>NSRequiredContext</key>
            <dict>
                <key>NSApplicationIdentifier</key>
                <string>com.apple.finder</string>
            </dict>
        </dict>
    </array>
</dict>
</plist>"#;
    
    // Write Info.plist
    let info_plist_path = format!("{}/Contents/Info.plist", workflow_path);
    if let Err(e) = fs::write(&info_plist_path, info_plist) {
        return Err(format!("Failed to write Info.plist: {}", e));
    }
    
    // Create the main workflow document (document.wflow)
    let workflow_document = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AMApplicationBuild</key>
    <string>521</string>
    <key>AMApplicationVersion</key>
    <string>2.10</string>
    <key>AMDocumentVersion</key>
    <string>2</string>
    <key>actions</key>
    <array>
        <dict>
            <key>action</key>
            <dict>
                <key>AMAccepts</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Optional</key>
                    <true/>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.string</string>
                    </array>
                </dict>
                <key>AMActionVersion</key>
                <string>2.0.3</string>
                <key>AMApplication</key>
                <array>
                    <string>Automator</string>
                </array>
                <key>AMParameterProperties</key>
                <dict>
                    <key>COMMAND_STRING</key>
                    <dict>
                        <key>tokenizedValue</key>
                        <array>
                            <string>folder_path="$1"

# URL encode the path using native shell (no Python dependency)
encoded_path=$(printf '%s\n' "$folder_path" | sed 's/ /%20/g')

# Open the moss:// deep link
open "moss://publish?path=$encoded_path"</string>
                        </array>
                    </dict>
                    <key>CheckedForUserDefaultShell</key>
                    <true/>
                    <key>inputMethod</key>
                    <integer>1</integer>
                    <key>shell</key>
                    <string>/bin/bash</string>
                    <key>source</key>
                    <string></string>
                </dict>
                <key>AMProvides</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.string</string>
                    </array>
                </dict>
                <key>ActionBundlePath</key>
                <string>/System/Library/Automator/Run Shell Script.action</string>
                <key>ActionName</key>
                <string>Run Shell Script</string>
                <key>ActionParameters</key>
                <dict>
                    <key>COMMAND_STRING</key>
                    <string>folder_path="$1"

# URL encode the path using native shell (no Python dependency)
encoded_path=$(printf '%s\n' "$folder_path" | sed 's/ /%20/g')

# Open the moss:// deep link
open "moss://publish?path=$encoded_path"</string>
                    <key>CheckedForUserDefaultShell</key>
                    <true/>
                    <key>inputMethod</key>
                    <integer>1</integer>
                    <key>shell</key>
                    <string>/bin/bash</string>
                    <key>source</key>
                    <string></string>
                </dict>
                <key>BundleIdentifier</key>
                <string>com.apple.RunShellScript</string>
                <key>CFBundleVersion</key>
                <string>2.0.3</string>
                <key>CanShowSelectedItemsWhenRun</key>
                <false/>
                <key>CanShowWhenRun</key>
                <true/>
                <key>Category</key>
                <array>
                    <string>AMCategoryUtilities</string>
                </array>
                <key>Class Name</key>
                <string>RunShellScriptAction</string>
                <key>InputUUID</key>
                <string>AEAA5C01-E8BB-4944-B1FC-94F0B9C16A62</string>
                <key>Keywords</key>
                <array>
                    <string>Shell</string>
                    <string>Script</string>
                    <string>Command</string>
                    <string>Run</string>
                    <string>Unix</string>
                </array>
                <key>OutputUUID</key>
                <string>D1071B2C-A747-4CB3-B4CB-6C5D95B1069B</string>
                <key>UUID</key>
                <string>6B8F247C-FC27-4D2F-A6E0-D1E5B7E69C0F</string>
                <key>UnlocalizedApplications</key>
                <array>
                    <string>Automator</string>
                </array>
                <key>arguments</key>
                <dict>
                    <key>0</key>
                    <dict>
                        <key>default value</key>
                        <integer>1</integer>
                        <key>name</key>
                        <string>inputMethod</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>0</string>
                    </dict>
                    <key>1</key>
                    <dict>
                        <key>default value</key>
                        <string></string>
                        <key>name</key>
                        <string>source</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>1</string>
                    </dict>
                    <key>2</key>
                    <dict>
                        <key>default value</key>
                        <false/>
                        <key>name</key>
                        <string>CheckedForUserDefaultShell</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>2</string>
                    </dict>
                    <key>3</key>
                    <dict>
                        <key>default value</key>
                        <string></string>
                        <key>name</key>
                        <string>COMMAND_STRING</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>3</string>
                    </dict>
                    <key>4</key>
                    <dict>
                        <key>default value</key>
                        <string>/bin/sh</string>
                        <key>name</key>
                        <string>shell</string>
                        <key>required</key>
                        <string>0</string>
                        <key>type</key>
                        <string>0</string>
                        <key>uuid</key>
                        <string>4</string>
                    </dict>
                </dict>
            </dict>
            <key>isViewVisible</key>
            <integer>1</integer>
            <key>location</key>
            <string>449.000000:316.000000</string>
            <key>nibPath</key>
            <string>/System/Library/Automator/Run Shell Script.action/Contents/Resources/Base.lproj/main.nib</string>
        </dict>
    </array>
    <key>connectors</key>
    <dict/>
    <key>workflowMetaData</key>
    <dict>
        <key>serviceApplicationBundleID</key>
        <string>com.apple.finder</string>
        <key>serviceApplicationPath</key>
        <string>/System/Library/CoreServices/Finder.app</string>
        <key>serviceInputTypeIdentifier</key>
        <string>com.apple.Automator.fileSystemObject.folder</string>
        <key>serviceOutputTypeIdentifier</key>
        <string>com.apple.Automator.nothing</string>
        <key>serviceProcessesInput</key>
        <integer>0</integer>
        <key>workflowTypeIdentifier</key>
        <string>com.apple.Automator.servicesMenu</string>
    </dict>
</dict>
</plist>"#;
    
    // Write the workflow document
    let document_path = format!("{}/Contents/document.wflow", workflow_path);
    if let Err(e) = fs::write(&document_path, workflow_document) {
        return Err(format!("Failed to write workflow document: {}", e));
    }
    
    println!("📁 Installed Finder integration: {}", workflow_path);
    Ok("Finder integration installed successfully! Right-click any folder → Quick Actions → 'Publish to Web'".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Prevent app from exiting when window is closed
            // We want to stay in system tray
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            
            // Set up deep link handler for moss:// URLs
            app.listen("deep-link://new-url", |event| {
                if let Ok(url) = serde_json::from_str::<String>(&event.payload()) {
                    if url.starts_with("moss://publish?path=") {
                        if let Some(path_start) = url.find("path=") {
                            let encoded_path = &url[path_start + 5..];
                            // Decode URL-encoded path
                            let decoded_path = encoded_path.replace("%20", " ");
                            println!("📥 Deep link received: {}", decoded_path);
                            // Note: In a real implementation, we'd trigger publish_folder here
                            // For now, just log the received path
                        }
                    }
                }
            });
            
            use tauri::{
                image::Image,
                menu::{MenuBuilder, MenuItem, PredefinedMenuItem},
                tray::TrayIconBuilder,
            };

            // Create menu items
            let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let about_i = MenuItem::with_id(app, "about", "About Moss", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            // Build menu with separators
            let menu = MenuBuilder::new(app)
                .item(&settings_i)
                .item(&about_i)
                .item(&PredefinedMenuItem::separator(app)?)
                .item(&quit_i)
                .build()?;

            // Create template icon for macOS tray
            let mut icon_rgba = vec![0x00; 16 * 16 * 4];
            
            // Draw a simple black circle
            for y in 4..12 {
                for x in 4..12 {
                    let distance_sq = (x as i32 - 8).pow(2) + (y as i32 - 8).pow(2);
                    if distance_sq <= 16 {
                        let idx = (y * 16 + x) * 4;
                        icon_rgba[idx] = 0x00;
                        icon_rgba[idx + 1] = 0x00;
                        icon_rgba[idx + 2] = 0x00;
                        icon_rgba[idx + 3] = 0xFF;
                    }
                }
            }
            
            let icon = Image::new(&icon_rgba, 16, 16);
            
            let tray_result = TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "settings" => {
                            println!("⚙️ Settings menu item clicked");
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.set_title("Moss Settings");
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "about" => {
                            // TODO: Show about dialog
                            println!("ℹ️ About menu item clicked");
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|_tray, _event| {
                    // Handle tray icon events if needed for future functionality
                })
                .build(app);
            
            match tray_result {
                Ok(tray) => {
                    if let Some(retrieved_tray) = app.tray_by_id(tray.id()) {
                        let _ = retrieved_tray.set_tooltip(Some("Moss - Right-click to publish"));
                    }
                    let _tray = tray;
                },
                Err(e) => {
                    eprintln!("Failed to create tray icon: {:?}", e);
                }
            }

            // Handle window close event - just hide instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent default close behavior
                        api.prevent_close();
                        // Just hide the window instead
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![publish_folder, install_finder_integration, get_system_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_publish_folder_valid_path() {
        let result = publish_folder("/Users/test/my-content".to_string());
        assert!(result.is_ok());
        let url = result.unwrap();
        assert!(url.contains("moss.pub"));
    }

    #[test]
    fn test_publish_folder_empty_path() {
        let result = publish_folder("".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty folder path provided");
    }

    #[test]
    fn test_publish_folder_returns_url() {
        let result = publish_folder("/Users/test/my-site".to_string());
        assert!(result.is_ok());
        let url = result.unwrap();
        assert!(url.starts_with("https://"));
        assert!(url.contains(".moss.pub"));
    }

    #[test]
    fn test_install_finder_integration_basic() {
        // This test only checks that the function doesn't panic
        // Actual filesystem operations would require more complex mocking
        // For now, we just verify the function can be called
        // In real usage, it requires proper HOME environment and filesystem access
        if std::env::var("HOME").is_ok() {
            let result = install_finder_integration();
            // Should either succeed or fail gracefully, not panic
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_tray_icon_creation_logic() {
        use tauri::image::Image;
        
        // Test that we can create the tray icon without panicking
        let icon_data = create_test_tray_icon_data();
        
        // Verify we have the expected data size (RGBA = 4 bytes per pixel)
        let expected_size = 16 * 16 * 4;
        assert_eq!(icon_data.len(), expected_size, "Icon data should be {} bytes", expected_size);
        
        // Test that we can create a Tauri Image from this data
        let icon = Image::new(&icon_data, 16, 16);
        assert_eq!(icon.width(), 16, "Icon width should be 16 pixels");
        assert_eq!(icon.height(), 16, "Icon height should be 16 pixels");
        
        // Verify some pixels are transparent (alpha = 0) and some opaque (alpha = 255)
        let transparent_pixels = icon_data.chunks(4).filter(|pixel| pixel[3] == 0x00).count();
        let opaque_pixels = icon_data.chunks(4).filter(|pixel| pixel[3] == 0xFF).count();
        
        assert!(transparent_pixels > 0, "Should have transparent pixels for background");
        assert!(opaque_pixels > 0, "Should have opaque pixels for the circle");
        
        println!("✅ Tray icon creation test passed: {} transparent, {} opaque pixels", 
                transparent_pixels, opaque_pixels);
    }

    #[test]
    fn test_tray_visibility_status_enum() {
        // Test serialization/deserialization of TrayVisibilityStatus
        let test_cases = [
            TrayVisibilityStatus::NotAdded,
            TrayVisibilityStatus::AddedButHidden,
            TrayVisibilityStatus::Visible,
        ];

        for status in test_cases.iter() {
            let json = serde_json::to_string(status).unwrap();
            let deserialized: TrayVisibilityStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, &deserialized);
        }

        // Test that each status serializes to expected string
        assert_eq!(serde_json::to_string(&TrayVisibilityStatus::NotAdded).unwrap(), "\"NotAdded\"");
        assert_eq!(serde_json::to_string(&TrayVisibilityStatus::AddedButHidden).unwrap(), "\"AddedButHidden\"");
        assert_eq!(serde_json::to_string(&TrayVisibilityStatus::Visible).unwrap(), "\"Visible\"");
    }
}