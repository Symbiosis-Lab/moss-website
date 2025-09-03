//! # moss - Desktop Publishing App
//!
//! A Tauri-based desktop application that allows users to compile folders into websites
//! with right-click integration on macOS Finder.
//!
//! ## Backend API
//!
//! The backend exposes a minimal set of Tauri commands:
//! - [`compile_and_serve`] - Core compilation functionality
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
mod preview;

use commands::*;
use preview::PreviewWindowManager;
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
/// Handles first launch setup including automatic Finder integration installation
fn setup_first_launch(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Create app config directory if it doesn't exist  
    let app_config_dir = app.path().app_config_dir()
        .map_err(|e| format!("Failed to get app config directory: {}", e))?;
    
    std::fs::create_dir_all(&app_config_dir)?;
    
    // Check if first launch setup is complete
    let first_launch_marker = app_config_dir.join("finder_integration_installed");
    
    if !first_launch_marker.exists() {
        // First launch - attempt to install Finder integration automatically
        match install_finder_integration() {
            Ok(message) => {
                // Mark installation as complete
                std::fs::write(&first_launch_marker, "installed")?;
                println!("✅ First launch: {}", message);
            },
            Err(e) => {
                // Log error but don't fail app startup
                eprintln!("⚠️ First launch: Failed to install Finder integration: {}", e);
                eprintln!("💡 Users can install manually via Settings");
            }
        }
    }
    
    Ok(())
}

/// Extracts and decodes the folder path from a moss:// deep link URL
/// 
/// # Arguments
/// * `url` - Deep link URL in format "moss://publish?path=<encoded_path>"
/// 
/// # Returns
/// * `Some(String)` - Decoded folder path if URL is valid
/// * `None` - If URL format is invalid
pub fn extract_path_from_deep_link(url: &str) -> Option<String> {
    if url.starts_with("moss://publish?path=") {
        if let Some(path_start) = url.find("path=") {
            let encoded_path = &url[path_start + 5..];
            // Decode URL-encoded path (basic space handling)
            let decoded_path = encoded_path.replace("%20", " ");
            return Some(decoded_path);
        }
    }
    None
}

/// Handles deep link URLs by building the site and then opening a preview window
/// 
/// Workflow: Build → Preview → (user can then Publish/Syndicate)
/// Processes URLs in the format: `moss://publish?path=<encoded_path>`
fn handle_deep_link_url(app: &tauri::AppHandle, url: &str) {
    if let Some(folder_path) = extract_path_from_deep_link(url) {
        let path = std::path::PathBuf::from(&folder_path);
        
        // Step 1: Build the site (compile files, start local server)
        match commands::compile_and_serve(folder_path.clone()) {
            Ok(result) => {
                println!("✅ Build completed: {}", result);
                
                // Step 2: Open preview window pointing to local server
                let preview_url = preview::build_preview_url("http://localhost:8080", &path);
                let state = preview::PreviewState::new(path, preview_url);
                
                if let Err(error) = preview::create_preview_window(app, state.clone(), None) {
                    eprintln!("❌ Failed to create preview window: {}", error);
                    return;
                }
                
                // Add to manager if available
                if let Some(manager) = app.try_state::<PreviewWindowManager>() {
                    manager.add_window(state);
                    println!("✅ Opened preview window after successful build");
                } else {
                    eprintln!("❌ Preview window manager not available");
                }
            },
            Err(error) => {
                eprintln!("❌ Build failed, cannot open preview: {}", error);
            }
        }
    }
}

/// # Deep link handling
/// Uses command-line argument processing with single instance plugin for reliable
/// deep link handling across platforms, especially macOS.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            
            // Process deep link URLs from command line arguments
            for arg in args {
                if arg.starts_with("moss://") {
                    handle_deep_link_url(app, &arg);
                }
            }
        }))
        .setup(|app| {
        // First launch setup - install Finder integration automatically
        setup_first_launch(&app.handle())?;
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
                }
            }
            
            // Process deep links from startup arguments
            let args: Vec<String> = std::env::args().collect();
            for arg in &args[1..] {  // Skip binary name
                if arg.starts_with("moss://") {
                    handle_deep_link_url(&app.handle(), arg);
                }
            }
            
            // Also set up event-based deep link handler for comprehensive coverage
            use tauri_plugin_deep_link::DeepLinkExt;
            let app_handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                let urls = event.urls();
                
                for url in &urls {
                    let url_str = url.as_str();
                    handle_deep_link_url(&app_handle, url_str);
                }
            });
            
            use tauri::{
                image::Image,
                menu::{MenuBuilder, MenuItem, PredefinedMenuItem},
                tray::TrayIconBuilder,
            };

            // Build system tray menu with standard items
            let publish_i = MenuItem::with_id(app, "publish", "Compile...", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let about_i = MenuItem::with_id(app, "about", "About moss", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            // Assemble menu with proper separators for platform consistency
            let menu = MenuBuilder::new(app)
                .item(&publish_i)
                .item(&PredefinedMenuItem::separator(app)?)
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
                        "publish" => {
                            // Trigger directory picker and compile workflow
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                match commands::compile_with_directory_picker(app_handle).await {
                                    Ok(message) => {
                                        println!("✅ {}", message);
                                    },
                                    Err(error) => {
                                        eprintln!("❌ Compile failed: {}", error);
                                    }
                                }
                            });
                        }
                        "settings" => {
                            // Show main window as settings dialog
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.set_title("moss Settings");
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "about" => {
                            // Future: Implement about dialog with app info
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
                        let _ = retrieved_tray.set_tooltip(Some("moss - Right-click to compile"));
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
        .manage(PreviewWindowManager::new())
        .invoke_handler(tauri::generate_handler![
            compile_and_serve,
            install_finder_integration, 
            get_system_status,
            open_preview_window,
            publish_from_preview,
            open_editor_from_preview,
            add_syndication_target,
            remove_syndication_target,
            get_preview_state,
            close_preview_window_cmd,
            compile_with_directory_picker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Application entry point.
/// 
/// Handles both GUI mode (default) and CLI mode based on command line arguments.
/// 
/// # CLI Usage
/// - `moss compile <folder_path>` - Compile site from folder
/// - `moss compile <folder_path> --serve` - Compile and serve site
/// - `moss compile <folder_path> --watch` - Compile and watch for changes  
/// - `moss compile <folder_path> --serve --watch` - Compile, serve, and watch
/// - `moss --help` - Show help message
/// 
/// # GUI Mode
/// - No arguments: Launch Tauri GUI application
fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // Check for CLI mode arguments
    if args.len() > 1 {
        match args[1].as_str() {
            "compile" => {
                if args.len() < 3 {
                    eprintln!("Usage: moss compile <folder_path> [--serve] [--watch]");
                    std::process::exit(1);
                }
                let folder_path = &args[2];
                let serve_flag = args.contains(&"--serve".to_string());
                let watch_flag = args.contains(&"--watch".to_string());
                run_cli_compile(folder_path, serve_flag, watch_flag);
            }
            "--help" | "-h" => {
                print_help();
            }
            _ => {
                // Check if it's a deep link URL, otherwise show help
                if args[1].starts_with("moss://") {
                    // Let GUI mode handle deep links
                    run();
                } else {
                    eprintln!("Unknown command: {}", args[1]);
                    print_help();
                    std::process::exit(1);
                }
            }
        }
    } else {
        // No arguments - launch GUI mode
        run();
    }
}

/// Prints CLI help information
fn print_help() {
    println!("moss - Desktop Publishing App

USAGE:
    moss [COMMAND] [OPTIONS]

COMMANDS:
    compile <folder>               Compile a website from the specified folder
    compile <folder> --serve       Compile and serve the website  
    compile <folder> --watch       Compile and watch for file changes
    compile <folder> --serve --watch  Compile, serve, and watch for changes
    
OPTIONS:
    -h, --help                     Show this help message

EXAMPLES:
    moss compile docs/public/      # Compile docs/public folder (build only)
    moss compile ~/blog/ --serve   # Compile and serve ~/blog folder  
    moss compile ~/blog/ --watch   # Compile and watch ~/blog for changes
    moss                          # Launch GUI application

For GUI usage, simply run moss without arguments or use system tray integration.");
}

/// CLI mode: Compile website with optional serve and watch modes
fn run_cli_compile(folder_path: &str, serve: bool, watch: bool) {
    use std::path::Path;
    
    // Validate folder path
    let path = Path::new(folder_path);
    if !path.exists() {
        eprintln!("❌ Error: Folder does not exist: {}", folder_path);
        std::process::exit(1);
    }
    
    if !path.is_dir() {
        eprintln!("❌ Error: Path is not a directory: {}", folder_path);
        std::process::exit(1);
    }
    
    println!("🏗️  Compiling website from: {}", folder_path);
    
    // Build the site using the compile_folder function
    match commands::compile_folder(folder_path.to_string()) {
        Ok(result) => {
            println!("✅ {}", result);
            
            if serve || watch {
                if serve {
                    // Extract the output path from result to start server
                    let site_path = Path::new(folder_path).join(".moss/site");
                    if site_path.exists() {
                        // Start the server (this function exists in commands module)
                        if let Err(e) = commands::start_site_server_cli(&site_path.to_string_lossy().to_string()) {
                            eprintln!("❌ Failed to start server: {}", e);
                            std::process::exit(1);
                        }
                        println!("🚀 Site is now serving at http://localhost:8080");
                    } else {
                        eprintln!("❌ Site directory not found at: {}", site_path.display());
                        std::process::exit(1);
                    }
                }
                if watch {
                    println!("👀 Watching for file changes...");
                }
                println!("📝 Press Ctrl+C to stop");
                
                // Keep running until Ctrl+C if serve or watch is enabled
                let (tx, rx) = std::sync::mpsc::channel();
                ctrlc::set_handler(move || {
                    println!("\n🛑 Stopping...");
                    tx.send(()).expect("Could not send signal on channel.");
                }).expect("Error setting Ctrl+C handler");
                
                // Wait for Ctrl+C
                rx.recv().expect("Could not receive from channel.");
                println!("✅ Stopped");
            } else {
                println!("✅ Compilation complete! Generated files are in .moss/site/");
                println!("💡 Use --serve to start a local server, or --watch to watch for changes");
            }
        },
        Err(error) => {
            eprintln!("❌ Build failed: {}", error);
            std::process::exit(1);
        }
    }
}


#[cfg(test)]
mod tests {

    /// Feature 1: Menu bar icon with dropdown menu
    /// Tests the core tray functionality that users interact with
    
    #[test]
    fn test_deep_link_url_parsing() {
        // Behavior: App should correctly decode deep link URLs for compilation
        use crate::extract_path_from_deep_link;
        
        // Test valid URLs with different path formats
        assert_eq!(
            extract_path_from_deep_link("moss://publish?path=/simple/path"),
            Some("/simple/path".to_string()),
            "Should parse simple paths"
        );
        
        assert_eq!(
            extract_path_from_deep_link("moss://publish?path=/path%20with%20spaces"),
            Some("/path with spaces".to_string()),
            "Should decode URL-encoded spaces"
        );
        
        assert_eq!(
            extract_path_from_deep_link("moss://publish?path=/Users/test/My%20Documents"),
            Some("/Users/test/My Documents".to_string()),
            "Should handle complex paths with spaces"
        );
        
        // Test invalid URLs
        assert_eq!(
            extract_path_from_deep_link("https://example.com"),
            None,
            "Should reject non-moss URLs"
        );
        
        assert_eq!(
            extract_path_from_deep_link("moss://invalid"),
            None,
            "Should reject moss URLs without path parameter"
        );
        
        assert_eq!(
            extract_path_from_deep_link("moss://publish"),
            None,
            "Should reject URLs missing path parameter"
        );
    }

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
        // Behavior: App should identify folders containing content suitable for compilation
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