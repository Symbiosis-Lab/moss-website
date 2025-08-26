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

mod types;
mod commands;

use commands::*;
use tauri::Manager;

/// Main application entry point for Tauri desktop and mobile platforms.
/// 
/// Initializes the Tauri application with all required plugins and sets up:
/// - System tray icon with context menu
/// - Deep link handler for moss:// URLs
/// - Window management (hide on close, stay in tray)
/// - macOS-specific behaviors (Accessory activation policy)
/// 
/// # Plugins
/// - `tauri_plugin_dialog` - File dialogs and system dialogs
/// - `tauri_plugin_fs` - File system access
/// - `tauri_plugin_shell` - Shell command execution
/// - `tauri_plugin_deep_link` - Custom URL protocol handling
/// 
/// # Platform-specific behavior
/// - **macOS**: Sets Accessory activation policy to prevent dock icon
/// - **All platforms**: Creates system tray with menu items
/// 
/// # Deep link handling
/// Listens for `moss://publish?path=<encoded_path>` URLs and triggers
/// the publishing workflow for the specified folder.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            println!("🌿 Moss app starting up...");
            // Configure macOS-specific behavior: prevent dock icon, stay in tray only
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            
            // Register deep links at runtime for development mode (Linux/Windows only)
            #[cfg(any(target_os = "linux", all(debug_assertions, target_os = "windows")))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Err(e) = app.deep_link().register_all() {
                    eprintln!("❌ Failed to register deep links: {}", e);
                } else {
                    println!("✅ Deep links registered for development mode");
                }
            }
            
            // Register deep link handler for moss:// protocol URLs
            // Processes publish requests from Finder integration
            println!("🚀 Setting up deep link handler...");
            let app_handle = app.handle().clone();
            
            // Use proper Tauri v2 deep link API
            use tauri_plugin_deep_link::DeepLinkExt;
            app.deep_link().on_open_url(move |event| {
                let urls = event.urls();
                println!("🔗 Deep link event received: {:?}", urls);
                eprintln!("🔗 STDERR: Deep link event received: {:?}", urls);
                
                for url in &urls {
                    println!("📥 Processing deep link URL: {}", url);
                    let url_str = url.as_str();
                    if url_str.starts_with("moss://publish?path=") {
                        if let Some(path_start) = url_str.find("path=") {
                            let encoded_path = &url_str[path_start + 5..];
                            // Decode URL-encoded path (basic space handling)
                            let decoded_path = encoded_path.replace("%20", " ");
                            println!("📁 Decoded folder path: {}", decoded_path);
                            
                            // Directly call the publish command
                            match publish_folder(decoded_path) {
                                Ok(result) => {
                                    println!("✅ Deep link publish success: {}", result);
                                },
                                Err(error) => {
                                    eprintln!("❌ Deep link publish failed: {}", error);
                                }
                            }
                        }
                    }
                }
            });
            
            use tauri::{
                image::Image,
                menu::{MenuBuilder, MenuItem, PredefinedMenuItem},
                tray::TrayIconBuilder,
            };

            // Build system tray menu with standard items
            let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let about_i = MenuItem::with_id(app, "about", "About Moss", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            // Assemble menu with proper separators for platform consistency
            let menu = MenuBuilder::new(app)
                .item(&settings_i)
                .item(&about_i)
                .item(&PredefinedMenuItem::separator(app)?)
                .item(&quit_i)
                .build()?;

            // Generate programmatic tray icon (16x16 RGBA)
            // Uses template format for automatic dark/light mode adaptation on macOS
            let mut icon_rgba = vec![0x00; 16 * 16 * 4];
            
            // Draw simple circular icon using distance calculation
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
                            // Show main window as settings dialog
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.set_title("Moss Settings");
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "about" => {
                            // TODO: Implement about dialog with app info
                            println!("ℹ️ About menu item clicked");
                        }
                        "quit" => {
                            // Clean exit from tray menu
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|_tray, _event| {
                    // Reserved for future tray icon click/double-click handling
                })
                .build(app);
            
            // Handle tray icon creation result
            match tray_result {
                Ok(tray) => {
                    // Set helpful tooltip for user guidance
                    if let Some(retrieved_tray) = app.tray_by_id(tray.id()) {
                        let _ = retrieved_tray.set_tooltip(Some("Moss - Right-click to publish"));
                    }
                    let _tray = tray; // Keep tray alive
                },
                Err(e) => {
                    eprintln!("Failed to create tray icon: {:?}", e);
                }
            }

            // Configure window behavior: hide instead of quit on close
            // This keeps the app running in the system tray
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Override default close behavior
                        api.prevent_close();
                        // Hide window but keep app running in tray
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![publish_folder, install_finder_integration, get_system_status, test_publish_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Application entry point.
/// 
/// Delegates to [`run()`] function for Tauri application lifecycle management.
fn main() {
    run();
}


#[cfg(test)]
mod tests {

    /// Feature 1: Menu bar icon with dropdown menu
    /// Tests the core tray functionality that users interact with
    
    #[test]
    fn test_content_analysis_homepage_detection() {
        // Behavior: App should correctly identify homepage files by priority
        use crate::types::FileInfo;
        use crate::commands::*;
        
        // Test priority order: index.md > index.pages > index.docx > README.md
        let files = vec![
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "README.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 150, modified: None },
        ];
        
        let result = detect_homepage_file(&files);
        assert_eq!(result, Some("index.md".to_string()), "Should prioritize index.md over README.md");
        
        // Test fallback to README.md when no index files exist
        let files_no_index = vec![
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "README.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
        ];
        
        let result = detect_homepage_file(&files_no_index);
        assert_eq!(result, Some("README.md".to_string()), "Should fallback to README.md");
    }
    
    #[test] 
    fn test_content_analysis_folder_detection() {
        // Behavior: App should identify folders containing publishable content
        use crate::types::FileInfo;
        use crate::commands::*;
        
        let files = vec![
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "posts/first-post.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
            FileInfo { path: "posts/second-post.md".to_string(), file_type: "md".to_string(), size: 150, modified: None },
            FileInfo { path: "projects/app.docx".to_string(), file_type: "docx".to_string(), size: 300, modified: None },
            FileInfo { path: "images/photo.jpg".to_string(), file_type: "jpg".to_string(), size: 5000, modified: None },
        ];
        
        let result = detect_content_folders(&files);
        assert_eq!(result.len(), 2, "Should detect 2 content folders");
        assert!(result.contains(&"posts".to_string()), "Should detect posts folder");
        assert!(result.contains(&"projects".to_string()), "Should detect projects folder");
        assert!(!result.contains(&"images".to_string()), "Should ignore image-only folders");
    }
    
    #[test]
    fn test_content_analysis_project_classification() {
        // Behavior: App should classify project structure for optimal site generation
        use crate::types::{FileInfo, ProjectType};
        use crate::commands::*;
        
        // Test 1: Homepage with collections (has content folders)
        let files_with_collections = vec![
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "posts/post1.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
        ];
        let content_folders = vec!["posts".to_string()];
        
        let result = detect_project_type_from_content(&files_with_collections, &content_folders);
        assert_eq!(result, ProjectType::HomepageWithCollections, "Should classify as homepage with collections");
        
        // Test 2: Simple flat site (≤5 root documents, no collections)
        let files_simple = vec![
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "contact.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
        ];
        let no_folders: Vec<String> = vec![];
        
        let result = detect_project_type_from_content(&files_simple, &no_folders);
        assert_eq!(result, ProjectType::SimpleFlatSite, "Should classify as simple flat site");
        
        // Test 3: Blog-style flat site (>5 root documents, no collections)
        let files_blog: Vec<FileInfo> = (1..=7).map(|i| FileInfo { 
            path: format!("post{}.md", i), 
            file_type: "md".to_string(), 
            size: 100,
            modified: None,
        }).collect();
        
        let result = detect_project_type_from_content(&files_blog, &no_folders);
        assert_eq!(result, ProjectType::BlogStyleFlatSite, "Should classify as blog-style flat site");
    }


}