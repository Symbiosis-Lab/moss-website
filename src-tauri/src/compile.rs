//! Moss compilation module - Static site generation from folders
//! 
//! This module provides the core functionality for transforming folders containing
//! documents (markdown, Pages, Word) into static websites. It orchestrates three
//! main phases: analysis, generation, and serving.
//!
//! ## Module Structure
//! - `analysis` - Folder scanning and project structure detection
//! - `generator` - Static site generation and HTML templating  
//! - `server` - Preview server for generated sites
//!
//! ## Core Workflow
//! 1. **Analysis**: Scan folder → detect structure → classify project type
//! 2. **Generation**: Process content → generate HTML → copy assets
//! 3. **Serving**: Start preview server → enable live preview

pub mod analysis;
pub mod generator; 
pub mod navigation;
pub mod server;

// Re-export public functions for backward compatibility
pub use analysis::scan_folder;
pub use generator::generate_static_site;
pub use server::{start_preview_server, stop_preview_server};

#[cfg(test)]
pub use analysis::{detect_homepage_file, detect_content_folders, detect_project_type_from_content};

use crate::types::*;
use std::path::Path;
use std::fs;
use tauri::{Manager, Emitter};

/// Recursively copies a directory and all its contents
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), String> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    if !src.exists() {
        return Err(format!("Source directory does not exist: {}", src.display()));
    }
    
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    
    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read source directory: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)
                .map_err(|e| format!("Failed to copy file {}: {}", path.display(), e))?;
        }
    }
    
    Ok(())
}


/// Synchronous compilation function for CLI usage.
/// 
/// This is a blocking compilation function for CLI usage without frontend communication.
/// 
/// # Arguments
/// * `folder_path` - Absolute path to the folder containing website content
/// * `auto_serve` - Whether to automatically start preview server after compilation (default: false)
/// 
/// # Returns
/// * `Ok(String)` - Success message with compilation summary (and server info if started)
/// * `Err(String)` - Error message describing what went wrong
pub fn compile_folder_sync(folder_path: String, auto_serve: Option<bool>) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // Replicate compile_folder logic but without channel support for CLI usage
        let auto_serve = auto_serve.unwrap_or(false);
        if folder_path.is_empty() {
            return Err("Empty folder path provided".to_string());
        }
        
        // Scan folder for content suitable for compilation
        let project_structure = scan_folder(&folder_path)?;
        
        // Basic validation - ensure we have some content to compile
        if project_structure.total_files == 0 {
            return Err("No files found in the specified folder".to_string());
        }
        
        // Generate a simple site identifier based on folder name
        let folder_name = Path::new(&folder_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unnamed-site");
        
        // Generate static site from scanned files
        let site_result = generate_static_site(&folder_path, &project_structure)?;
        
        // Create compilation strategy message based on detected type
        let strategy_message = match project_structure.project_type {
            ProjectType::HomepageWithCollections => "Homepage with organized content collections detected",
            ProjectType::SimpleFlatSite => "Simple site with all pages in navigation menu",
            ProjectType::BlogStyleFlatSite => "Blog-style site with essential pages in menu, others listed on homepage",
        };
        
        let homepage_info = if let Some(homepage) = &project_structure.homepage_file {
            format!(" (Homepage: {})", homepage)
        } else {
            String::new()
        };
        
        let base_message = format!(
            "📁 '{}': {} files scanned. {} {}. Content folders: {:?}. Site generated at {}",
            folder_name,
            project_structure.total_files,
            strategy_message,
            homepage_info,
            project_structure.content_folders,
            site_result.output_path
        );

        // Optionally start preview server
        if auto_serve {
            let port = start_preview_server(&site_result.output_path).await?;
            let preview_url = format!("http://localhost:{}", port);
            Ok(format!("{}\n🌐 Preview server ready! Access at {}", base_message, preview_url))
        } else {
            Ok(base_message)
        }
    })
}

/// Tauri command for folder compilation with real-time progress channel.
/// 
/// This function handles GUI compilation with real-time progress updates sent
/// through a Tauri channel. It performs folder scanning, site generation, and
/// preview server startup with detailed progress information.
/// 
/// # Arguments
/// * `app` - Tauri app handle for preview URL communication
/// * `folder_path` - Absolute path to the folder containing website content
/// * `auto_serve` - Whether to automatically start preview server after compilation (default: false)
/// * `on_progress` - Channel for real-time progress updates
/// 
/// # Returns
/// * `Ok(String)` - Success message with compilation summary (and server info if started)
/// * `Err(String)` - Error message describing what went wrong
#[tauri::command]
#[specta::specta]
pub async fn compile_folder(
    app: tauri::AppHandle, 
    folder_path: String, 
    auto_serve: Option<bool>,
    on_progress: tauri::ipc::Channel<crate::types::ProgressUpdate>
) -> Result<String, String> {
    use crate::types::ProgressUpdate;
    
    let auto_serve = auto_serve.unwrap_or(false);
    if folder_path.is_empty() {
        return Err("Empty folder path provided".to_string());
    }
    
    // Helper function to send progress updates
    let send_progress = |step: &str, message: &str, percentage: u8, completed: bool| {
        let update = ProgressUpdate {
            step: step.to_string(),
            message: message.to_string(),
            percentage,
            completed,
            url: None,
            port: None,
        };
        let _ = on_progress.send(update);
    };

    // Helper function to send progress updates with URL and port
    let send_progress_with_server = |step: &str, message: &str, percentage: u8, completed: bool, url: Option<String>, port: Option<u16>| {
        let update = ProgressUpdate {
            step: step.to_string(),
            message: message.to_string(),
            percentage,
            completed,
            url,
            port,
        };
        let _ = on_progress.send(update);
    };

    // Step 1: Start scanning
    send_progress("scanning", "Scanning folder structure...", 10, false);
    
    // Scan folder for content suitable for compilation
    let project_structure = scan_folder(&folder_path)?;
    
    // Basic validation - ensure we have some content to compile
    if project_structure.total_files == 0 {
        return Err("No files found in the specified folder".to_string());
    }
    
    // Step 2: Analysis complete
    let folder_name = Path::new(&folder_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unnamed-site");
    
    send_progress(
        "analyzing", 
        &format!("Analyzed {} files in '{}'", project_structure.total_files, folder_name), 
        30, 
        true
    );
    
    // Step 3: Start generating site
    send_progress("generating", "Generating static site...", 40, false);
    
    // Generate static site from scanned files
    let site_result = generate_static_site(&folder_path, &project_structure)?;
    
    send_progress("generating", "Static site generated successfully", 70, true);
    
    // Create compilation strategy message based on detected type
    let strategy_message = match project_structure.project_type {
        ProjectType::HomepageWithCollections => "Homepage with organized content collections detected",
        ProjectType::SimpleFlatSite => "Simple site with all pages in navigation menu",
        ProjectType::BlogStyleFlatSite => "Blog-style site with essential pages in menu, others listed on homepage",
    };
    
    let homepage_info = if let Some(homepage) = &project_structure.homepage_file {
        format!(" (Homepage: {})", homepage)
    } else {
        String::new()
    };
    
    let base_message = format!(
        "📁 '{}': {} files scanned. {} {}. Content folders: {:?}. Site generated at {}",
        folder_name,
        project_structure.total_files,
        strategy_message,
        homepage_info,
        project_structure.content_folders,
        site_result.output_path
    );

    // Optionally start preview server and send preview URL to frontend
    if auto_serve {
        // Step 4: Start server
        send_progress("serving", "Starting preview server...", 80, false);
        
        let port = start_preview_server(&site_result.output_path).await?;
        let preview_url = format!("http://localhost:{}", port);
        
        send_progress("serving", &format!("Preview server started on port {}", port), 90, true);
        
        // Step 5: Send preview URL to frontend
        send_progress("ready", "Preparing preview...", 95, false);
        
        use crate::preview::commands::update_main_window_preview;
        
        let path_buf = std::path::PathBuf::from(&folder_path);
        let folder_path_str = path_buf.to_string_lossy().to_string();
        
        // Send the preview URL update to frontend
        if let Err(error) = update_main_window_preview(
            app, 
            preview_url.clone(), 
            folder_path_str
        ).await {
            eprintln!("⚠️ Failed to update preview: {}", error);
            // Continue anyway - server is running
        }
        
        // Step 6: Complete
        send_progress_with_server("ready", "Ready! Site compiled successfully.", 100, true, Some(preview_url.clone()), Some(port));
        
        Ok(format!("{}\n🌐 Preview server ready! Access at {}", base_message, preview_url))
    } else {
        send_progress("complete", "Compilation completed", 100, true);
        Ok(base_message)
    }
}

/// Compile folder specifically for preview window and update the window when complete
///
/// This function is used when no main window exists and a separate preview window is created.
/// It compiles the folder and then updates the specific preview window with the final preview URL.
pub async fn compile_folder_async_for_preview_window(
    app: tauri::AppHandle,
    folder_path: String,
    preview_id: String,
) -> Result<String, String> {
    use std::path::Path;

    if folder_path.is_empty() {
        return Err("Empty folder path provided".to_string());
    }

    // Scan folder for content suitable for compilation
    let project_structure = scan_folder(&folder_path)?;

    // Basic validation - ensure we have some content to compile
    if project_structure.total_files == 0 {
        return Err("No files found in the specified folder".to_string());
    }

    // Generate a simple site identifier based on folder name
    let folder_name = Path::new(&folder_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unnamed-site");

    // Generate static site from scanned files
    let site_result = generate_static_site(&folder_path, &project_structure)?;

    // Create compilation strategy message based on detected type
    let strategy_message = match project_structure.project_type {
        ProjectType::HomepageWithCollections => "Homepage with organized content collections detected",
        ProjectType::SimpleFlatSite => "Simple site with all pages in navigation menu",
        ProjectType::BlogStyleFlatSite => "Blog-style site with essential pages in menu, others listed on homepage",
    };

    let homepage_info = if let Some(homepage) = &project_structure.homepage_file {
        format!(" (Homepage: {})", homepage)
    } else {
        String::new()
    };

    let base_message = format!(
        "✅ Site compiled for {}: {} unique pages generated. Strategy: {}{}",
        folder_name,
        site_result.page_count,
        strategy_message,
        homepage_info
    );

    // Start local server with the built site output path
    let port = start_preview_server(&site_result.output_path).await?;
    let preview_url = format!("http://localhost:{}", port);
    println!("🌐 Preview server started at: {}", preview_url);

    // Update the specific preview window with the final URL
    if let Some(preview_window) = app.get_webview_window(&format!("preview_{}", preview_id)) {
        if let Err(e) = preview_window.eval(&format!(
            "window.location.href = '{}';",
            preview_url
        )) {
            eprintln!("⚠️ Failed to update preview window URL: {}", e);
        } else {
            println!("✅ Preview window updated with URL: {}", preview_url);
        }
    } else {
        eprintln!("⚠️ Preview window not found for ID: {}", preview_id);
    }

    Ok(format!("{}\n🌐 Preview server ready! Access at {}", base_message, preview_url))
}

/// Tauri command to retrieve system diagnostic information.
/// 
/// Collects runtime information about the application's integration
/// with the operating system for debugging, support, and user feedback.
/// 
/// # Collected Information
/// * Operating system identification
/// * Tray icon visibility status  
/// * Finder integration installation status (macOS)
/// * Application version from package metadata
/// 
/// # Arguments
/// * `app` - Tauri application handle for system access
/// 
/// # Returns
/// * `Ok(SystemInfo)` - Diagnostic information structure
/// * `Err(String)` - Error collecting system information
/// 
/// # macOS Integration
/// Checks for Automator workflow at:
/// `~/Library/Services/Publish to Web.workflow`
/// 
/// This indicates whether right-click folder compilation is available.
#[tauri::command]
#[specta::specta]
pub fn get_system_status(app: tauri::AppHandle) -> Result<SystemInfo, String> {
    let os = std::env::consts::OS.to_string();
    let app_version = app.package_info().version.to_string();
    
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
        finder_integration,
        app_version,
    })
}

/// Tauri command to install macOS Finder integration for one-click compilation.
/// 
/// Creates an Automator workflow that adds "Publish to Web" to the Finder
/// context menu when right-clicking folders. This provides seamless integration
/// with the user's file management workflow.
/// 
/// # Installation Process
/// 1. Creates `~/Library/Services/Publish to Web.workflow` bundle
/// 2. Writes Info.plist with proper NSServices configuration
/// 3. Embeds shell script that triggers `moss://` deep links
/// 4. Registers with macOS Services for folder context menus
/// 
/// # Workflow Behavior
/// When users right-click a folder in Finder:
/// - "Publish to Web" appears in the context menu
/// - Clicking it URL-encodes the folder path
/// - Opens `moss://publish?path=<encoded_path>`
/// - Triggers the compilation workflow in this app
/// 
/// # Security
/// The generated shell script uses only built-in macOS commands:
/// - `printf` and `sed` for URL encoding
/// - `open` for deep link activation
/// - No external dependencies or network access
/// 
/// # Returns
/// * `Ok(String)` - Success message with installation location
/// * `Err(String)` - Error message with failure details
/// 
/// # Errors
/// * HOME environment variable not found
/// * Permission denied creating Services directory
/// * File write failures during installation
/// * Cannot remove existing workflow for reinstallation
/// 
/// # Platform Support
/// Currently macOS only. Uses Automator and NSServices APIs.
/// 
/// # Example
/// ```rust
/// let result = install_finder_integration()?;
/// // Returns: "Finder integration installed successfully! Right-click..."
/// ```
#[tauri::command]
#[specta::specta]
pub fn install_finder_integration() -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    
    // Get user's home directory
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(_) => return Err("Could not determine home directory".to_string()),
    };
    
    let services_dir = format!("{}/Library/Services", home_dir);
    let workflow_path = format!("{}/Publish.workflow", services_dir);
    
    // Create Services directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&services_dir) {
        return Err(format!("Failed to create Services directory: {}", e));
    }
    
    // Remove existing workflow if it exists (to ensure clean reinstall)
    if Path::new(&workflow_path).exists() {
        if let Err(e) = fs::remove_dir_all(&workflow_path) {
            return Err(format!("Failed to remove existing workflow: {}", e));
        }
    }
    
    // Copy workflow from bundled resources
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?
        .parent()
        .ok_or_else(|| "Failed to get executable directory".to_string())?
        .to_path_buf();
        
    // Try Tauri bundle structure first, then fall back to development structure
    let resource_path = exe_dir
        .join("../Resources/_up_/resources/services/Publish.workflow")
        .canonicalize()
        .unwrap_or_else(|_| {
            exe_dir
                .join("../Resources/services/Publish.workflow")
                .canonicalize()
                .unwrap_or_else(|_| exe_dir.join("resources/services/Publish.workflow"))
        });
        
    if !resource_path.exists() {
        return Err("Bundled Publish.workflow not found in resources".to_string());
    }
    
    // Copy the entire workflow directory from resources to Services
    copy_dir_all(&resource_path, &workflow_path)?;
    
    Ok("Finder integration installed successfully! Right-click any folder → 'Publish'".to_string())
}

/// Gets the last selected directory from app configuration.
/// 
/// Retrieves the user's previously selected directory from persistent storage
/// for the directory picker dialog. Used to provide a better user experience
/// by remembering the last location.
/// 
/// # Arguments
/// * `app` - Tauri application handle for configuration access
/// 
/// # Returns
/// * `Some(String)` - Previously selected directory path
/// * `None` - No previous selection or configuration error
fn get_last_selected_directory(app: &tauri::AppHandle) -> Option<String> {
    app.path().app_config_dir()
        .ok()
        .and_then(|config_dir| {
            let pref_file = config_dir.join("directory_preference.txt");
            fs::read_to_string(pref_file).ok()
        })
        .and_then(|content| {
            let path = content.trim();
            if Path::new(path).exists() {
                Some(path.to_string())
            } else {
                None
            }
        })
}

/// Saves the selected directory to app configuration.
/// 
/// Persists the user's directory selection for future use by the directory
/// picker dialog. Creates the configuration file if it doesn't exist.
/// 
/// # Arguments
/// * `app` - Tauri application handle for configuration access
/// * `directory` - Directory path to save
/// 
/// # Returns
/// * `Ok(())` - Successfully saved
/// * `Err(String)` - Failed to save configuration
fn save_last_selected_directory(app: &tauri::AppHandle, directory: &str) -> Result<(), String> {
    let config_dir = app.path().app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    let pref_file = config_dir.join("directory_preference.txt");
    fs::write(pref_file, directory)
        .map_err(|e| format!("Failed to save directory preference: {}", e))?;
    
    Ok(())
}

/// Tauri command for folder compilation with GUI directory picker.
/// 
/// Provides a comprehensive workflow for folder compilation through the GUI:
/// 1. Temporarily switches to Regular activation policy for proper dialog display
/// 2. Shows native directory picker with remembered last location  
/// 3. Compiles selected folder and starts preview server
/// 4. Opens preview window with publish/edit controls
/// 5. Manages activation policy for optimal UX
/// 
/// # Activation Policy Management
/// - Switches to Regular briefly for centered dialog display
/// - Stays in Regular while preview window is open  
/// - Restores Accessory when no preview window (background operation)
/// 
/// # Arguments
/// * `app` - Tauri application handle for dialog and window management
/// 
/// # Returns
/// * `Ok(String)` - Success message with preview window status
/// * `Err(String)` - Error message (user cancellation, compilation failure, etc.)
/// 
/// # Error Handling
/// - User cancellation: Returns success with "canceled" message
/// - Compilation errors: Shows error dialog + restores Accessory mode
/// - Dialog timeout: Helpful message about bringing app to front
/// 
/// # Dialog Positioning
/// Uses both activation policy switching AND persistent dialog anchor window
/// for reliable dialog centering across different macOS system states.
#[tauri::command]
#[specta::specta]
pub async fn compile_with_directory_picker(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    // Step 1: Switch to Regular activation policy for proper dialog display
    println!("🔧 Temporarily switching to Regular activation policy for directory picker");
    if let Err(e) = app.set_activation_policy(tauri::ActivationPolicy::Regular) {
        eprintln!("⚠️ Failed to set Regular activation policy: {}", e);
        // Continue anyway - dialog might still work
    }
    
    // Small delay to ensure policy change takes effect
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Step 2: Determine default directory (last selected or home)
    let default_path = get_last_selected_directory(&app);
    println!("🔧 Default directory for picker: {:?}", default_path);
    
    // Get the persistent dialog anchor window (created during app setup)
    let dialog_anchor = match app.get_webview_window("dialog_anchor") {
        Some(window) => window,
        None => {
            eprintln!("⚠️ Dialog anchor window not found, dialogs may not center properly");
            return Err("Internal error: Dialog anchor not available".to_string());
        }
    };
    
    // Step 3: Open directory picker with explicit parent (now with proper positioning)
    let (sender, receiver) = tokio::sync::oneshot::channel();
    
    app
        .dialog()
        .file()
        .set_parent(&dialog_anchor)  // Use persistent dialog anchor window
        // Note: We use both activation policy switching AND parent window for maximum reliability
        // The parent window ensures centering
        .set_title("Select folder to publish")
        .set_directory(default_path.unwrap_or_else(|| {
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        }))
        .pick_folder(move |folder_path| {
            println!("🔧 Directory picker result: {:?}", folder_path.as_ref().map(|p| p.to_string()));
            let _ = sender.send(folder_path);
        });
    
    // Helper function to restore Accessory mode
    async fn restore_accessory_mode(app: &tauri::AppHandle) {
        println!("🔧 Restoring Accessory activation policy (hiding dock icon)");
        // Small delay for smooth transition
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        if let Err(e) = app.set_activation_policy(tauri::ActivationPolicy::Accessory) {
            eprintln!("⚠️ Failed to restore activation policy: {}", e);
        }
    }

    // Step 4: Wait for folder picker result with timeout
    let folder_path = match tokio::time::timeout(
        std::time::Duration::from_secs(60), // 60 second timeout for dialog
        receiver
    ).await {
        Ok(Ok(path)) => path,
        Ok(Err(_)) => {
            return Err("Dialog communication error".to_string());
        },
        Err(_) => {
            println!("⚠️ Directory picker dialog timed out");
            return Err("Dialog timed out - it may have appeared behind other windows. Try again or bring moss to front first.".to_string());
        }
    };
    
    // Step 5: Handle dialog result
    match folder_path {
        Some(path) => {
            println!("🔧 User selected directory: {}", path.to_string());
            // Convert FilePath to string path
            let path_str = path.to_string();
            let path_buf = std::path::PathBuf::from(&path_str);
            
            // Save for next time
            if let Err(e) = save_last_selected_directory(&app, &path_str) {
                eprintln!("⚠️ Failed to save directory preference: {}", e);
            }
            
            // Step 6: Create main window before compilation (so preview URL can be sent to it)
            if app.get_webview_window("main").is_none() {
                // Window doesn't exist, create it (similar to Settings menu logic)
                let window_result = tauri::WebviewWindowBuilder::new(
                    &app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into())
                )
                .title("moss")
                .inner_size(1200.0, 800.0)
                .resizable(true)
                .center()
                .build();
                
                match window_result {
                    Ok(window) => {
                        let window_clone = window.clone();
                        // Set up window event handler for close behavior
                        window.on_window_event(move |event| {
                            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                                // Override default close behavior - hide instead of quit
                                api.prevent_close();
                                let _ = window_clone.hide();
                            }
                        });
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    Err(e) => {
                        return Err(format!("Failed to create main window: {}", e));
                    }
                }
            } else {
                // Window exists, just show it
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            
            // Step 7: Send "ready for compile" event to frontend
            if let Some(window) = app.get_webview_window("main") {
                let payload = serde_json::json!({
                    "folder_path": path_str,
                    "auto_serve": true
                });
                println!("🔧 Emitting ready-for-compile event: {:?}", payload);

                let emit_result = window.emit("ready-for-compile", payload);
                println!("🔧 Event emit result: {:?}", emit_result);

                Ok(format!("Directory selected: {}", path_buf.file_name().unwrap_or_default().to_string_lossy()))
            } else {
                Err("Main window not found".to_string())
            }
        },
        None => {
            println!("🔧 User canceled directory selection");
            // User cancellation is a normal action, not an error
            // Restore Accessory mode after cancellation
            restore_accessory_mode(&app).await;
            Ok("Directory selection canceled".to_string())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileInfo;

    /// Feature 1: Content Analysis - Homepage Detection
    /// Tests the core logic for identifying the main page of a website
    #[test]
    fn test_content_analysis_homepage_detection() {
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
    
    /// Feature 2: Content Analysis - Folder Detection  
    /// Tests identification of content organization patterns
    #[test] 
    fn test_content_analysis_folder_detection() {
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
    
    /// Feature 3: Content Analysis - Project Classification
    /// Tests the logic for determining optimal site structure
    #[test]
    fn test_content_analysis_project_classification() {
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