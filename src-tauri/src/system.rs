//! System integration and GUI utilities for moss
//!
//! This module handles system-level functionality including:
//! - System diagnostics and status reporting
//! - macOS Finder integration setup
//! - User preferences and directory picker
//! - GUI dialog workflows
//! - File system utilities

use crate::types::*;
use crate::compile::server::stop_preview_server;
use std::path::Path;
use std::fs;
use std::sync::Mutex;
use tauri::Manager;

/// Delay in milliseconds required for macOS activation policy changes to propagate
///
/// When switching between Accessory and Regular activation policies, macOS needs time to:
/// - Update application state in the system
/// - Register/unregister dock icon
/// - Prepare window server for proper window display
/// - Update focus management
///
/// This value is empirically determined from `compile_with_directory_picker` working behavior.
/// Without this delay, windows may appear in a limbo state without proper dock icon or focus.
///
/// See: https://developer.apple.com/documentation/appkit/nsapplication/activationpolicy
pub const ACTIVATION_POLICY_DELAY_MS: u64 = 100;

/// Extract the last two directory levels from a path for display purposes
///
/// Examples:
/// - "/Users/username/Documents/my-blog" -> "Documents/my-blog"
/// - "/home/user/projects" -> "user/projects"
/// - "/single" -> "single"
pub fn get_short_path(path: &str) -> String {
    let path_buf = std::path::Path::new(path);
    let components: Vec<_> = path_buf.components().collect();

    match components.len() {
        0 => "Unknown".to_string(),
        1 => path_buf.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string(),
        _ => {
            let last_two: Vec<String> = components
                .iter()
                .rev()
                .take(2)
                .rev()
                .filter_map(|component| {
                    match component {
                        std::path::Component::Normal(name) => name.to_str().map(|s| s.to_string()),
                        _ => None,
                    }
                })
                .collect();

            if last_two.is_empty() {
                "Unknown".to_string()
            } else {
                last_two.join("/")
            }
        }
    }
}

/// Validates that a folder path exists and is a directory
///
/// Pure business logic function for path validation.
/// Used by compilation workflow to fail fast on invalid inputs.
///
/// # Arguments
/// * `path` - String path to validate
///
/// # Returns
/// * `Ok(())` - Path exists and is a directory
/// * `Err(String)` - Path is invalid with descriptive error message
///
/// # Examples
/// ```
/// let result = validate_folder_path("/tmp");
/// assert!(result.is_ok());
///
/// let result = validate_folder_path("");
/// assert!(result.is_err());
/// ```
pub fn validate_folder_path(path: &str) -> Result<(), String> {
    // Check for empty or whitespace-only path
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Empty folder path provided".to_string());
    }

    // Check if path exists
    let path_buf = Path::new(trimmed);
    if !path_buf.exists() {
        return Err(format!("Folder path does not exist: {}", trimmed));
    }

    // Check if path is a directory (not a file)
    if !path_buf.is_dir() {
        return Err(format!("Path is a file, not a directory: {}", trimmed));
    }

    Ok(())
}

/// Generates preview window title from folder path
///
/// Pure business logic function that creates user-friendly window titles
/// by extracting the last two path components.
///
/// # Arguments
/// * `folder_path` - Absolute path to folder being compiled
///
/// # Returns
/// * Window title string in format "website preview of {path}"
///
/// # Examples
/// ```
/// let title = generate_preview_window_title("/Users/test/blog");
/// assert_eq!(title, "website preview of test/blog");
/// ```
pub fn generate_preview_window_title(folder_path: &str) -> String {
    format!("website preview of {}", get_short_path(folder_path))
}

/// Creates preview window with compilation configuration
///
/// Core shared logic for creating preview windows from both menu bar and deep link paths.
/// Handles activation policy management, window creation, and configuration injection.
///
/// # Workflow
/// 1. Validates folder path exists and is a directory
/// 2. Switches to Regular activation policy for proper window display
/// 3. Waits for policy change to propagate (ACTIVATION_POLICY_DELAY_MS)
/// 4. Closes any existing "main" window
/// 5. Creates new window with compile configuration injected
/// 6. Shows and focuses the window
///
/// # Arguments
/// * `app` - Tauri application handle for window management
/// * `folder_path` - Absolute path to folder to compile
///
/// # Returns
/// * `Ok(WebviewWindow)` - Successfully created and displayed window
/// * `Err(String)` - Error with descriptive message (validation, policy, or window creation failure)
///
/// # Side Effects
/// - Changes activation policy to Regular (adds dock icon)
/// - Closes existing "main" window if present
/// - On error, attempts to restore Accessory mode
///
/// # Integration Test Contract
/// This function requires Tauri runtime and cannot be easily unit tested.
/// Manual testing should verify:
/// - ✅ Window appears with correct title
/// - ✅ Dock icon appears (Regular activation policy active)
/// - ✅ Window is visible and focused
/// - ✅ Compilation starts automatically (frontend detects `__MOSS_COMPILE_CONFIG__`)
/// - ✅ Invalid path returns Err without creating window
///
/// # Examples
/// ```ignore
/// // Called from menu bar publish:
/// let window = compile_with_preview_window(&app, "/Users/test/blog").await?;
///
/// // Called from deep link:
/// let window = compile_with_preview_window(&app, "/Users/test/blog").await?;
/// ```
pub async fn compile_with_preview_window(
    app: &tauri::AppHandle,
    folder_path: &str,
) -> Result<tauri::WebviewWindow, String> {
    // Step 1: Validate folder path (fail fast on invalid input)
    validate_folder_path(folder_path)?;

    // Step 2: Switch to Regular activation policy for proper window display
    // (Accessory mode windows don't show in Mission Control or cmd+tab)
    println!("🔧 Switching to Regular activation policy for preview window");
    if let Err(e) = app.set_activation_policy(tauri::ActivationPolicy::Regular) {
        eprintln!("⚠️ Failed to set Regular activation policy: {}", e);
        // Continue anyway - window might still work, but may have issues
    }

    // Step 3: CRITICAL - Wait for activation policy change to propagate
    // Without this delay, the window appears in limbo state without dock icon
    // This delay matches the working behavior in compile_with_directory_picker
    tokio::time::sleep(std::time::Duration::from_millis(ACTIVATION_POLICY_DELAY_MS)).await;

    // Step 4: Close any existing main window to ensure clean state
    if let Some(existing_window) = app.get_webview_window("main") {
        println!("🔧 Closing existing preview window");
        let _ = existing_window.close();
    }

    // Step 5: Generate initialization script with folder path
    // This injects window.__MOSS_COMPILE_CONFIG__ for frontend to detect
    let init_script = crate::window_manager::generate_compile_init_script(folder_path);

    // Step 6: Generate user-friendly window title
    let title = generate_preview_window_title(folder_path);

    // Step 7: Create window with compile configuration injected
    println!("🔧 Creating preview window: {}", title);
    let window_result = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(&title)
    .fullscreen(true)
    .resizable(true)
    .center()
    .initialization_script(&init_script)
    .build();

    // Step 8: Handle window creation result
    match window_result {
        Ok(window) => {
            // Show and focus the window
            let _ = window.show();
            let _ = window.set_focus();
            println!("✅ Preview window opened for: {}", folder_path);
            Ok(window)
        }
        Err(e) => {
            eprintln!("❌ Failed to create preview window: {}", e);
            // Restore Accessory mode on error (remove dock icon)
            println!("🔧 Restoring Accessory activation policy after window creation failure");
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Err(format!("Failed to create preview window: {}", e))
        }
    }
}

/// Recursively copies a directory and all its contents
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), String> {
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
pub fn get_last_selected_directory(app: &tauri::AppHandle) -> Option<String> {
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
pub fn save_last_selected_directory(app: &tauri::AppHandle, directory: &str) -> Result<(), String> {
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

    // Step 3: Open directory picker with explicit parent
    let (sender, receiver) = tokio::sync::oneshot::channel();

    app
        .dialog()
        .file()
        .set_parent(&dialog_anchor)  
        // Use persistent dialog anchor window
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

            // Step 6: Create preview window with compile configuration
            // Use shared compile_with_preview_window function
            // Note: This handles validation, activation policy delay, and window creation
            compile_with_preview_window(&app, &path_str).await?;

            // Step 7: Window created with compile configuration injected
            Ok(format!("Directory selected: {}", path_buf.file_name().unwrap_or_default().to_string_lossy()))
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

/// Tauri command: Stop all active preview servers
///
/// Manually stops all preview servers currently tracked in the application state.
/// Useful for debugging and manual cleanup when servers are not automatically stopped.
#[tauri::command]
#[specta::specta]
pub async fn stop_all_preview_servers(app: tauri::AppHandle) -> Result<String, String> {
    if let Some(server_state) = app.try_state::<Mutex<ServerState>>() {
        if let Ok(mut state) = server_state.lock() {
            let servers_to_stop: Vec<(String, u16)> = state.active_servers.iter()
                .map(|(path, port)| (path.clone(), *port))
                .collect();

            if servers_to_stop.is_empty() {
                return Ok("No active preview servers to stop".to_string());
            }

            let mut stopped_count = 0;
            let mut failed_count = 0;

            for (folder_path, port) in servers_to_stop {
                match stop_preview_server(port) {
                    Ok(_) => {
                        println!("🔧 Manually stopped preview server on port {} for {}", port, folder_path);
                        state.active_servers.remove(&folder_path);
                        stopped_count += 1;
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to stop server on port {} for {}: {}", port, folder_path, e);
                        // Remove from state anyway to prevent accumulation
                        state.active_servers.remove(&folder_path);
                        failed_count += 1;
                    }
                }
            }

            let _total = stopped_count + failed_count;
            if failed_count == 0 {
                Ok(format!("Successfully stopped {} preview server(s)", stopped_count))
            } else {
                Ok(format!("Stopped {} preview server(s), {} failed", stopped_count, failed_count))
            }
        } else {
            Err("Failed to acquire server state lock".to_string())
        }
    } else {
        Err("Server state not initialized".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test suite for validate_folder_path
    /// Tests path validation business logic
    #[test]
    fn test_validate_folder_path_valid_directory() {
        // Behavior: Valid directory should return Ok(())
        // Use a known system directory that exists
        let result = validate_folder_path("/tmp");
        assert!(result.is_ok(), "Valid directory should be accepted");
    }

    #[test]
    fn test_validate_folder_path_nonexistent() {
        // Behavior: Non-existent path should return descriptive error
        let result = validate_folder_path("/this/path/does/not/exist/hopefully");
        assert!(result.is_err(), "Non-existent path should be rejected");

        let error = result.unwrap_err();
        assert!(
            error.contains("does not exist") || error.contains("not found"),
            "Error should indicate path doesn't exist, got: {}", error
        );
    }

    #[test]
    fn test_validate_folder_path_file_not_directory() {
        // Behavior: File path should be rejected (not a directory)
        // Create a temporary file for testing
        let temp_file = std::env::temp_dir().join("moss_test_file.txt");
        std::fs::write(&temp_file, "test").unwrap();

        let result = validate_folder_path(temp_file.to_str().unwrap());

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);

        assert!(result.is_err(), "File path should be rejected");
        let error = result.unwrap_err();
        assert!(
            error.contains("not a directory") || error.contains("is a file"),
            "Error should indicate path is not a directory, got: {}", error
        );
    }

    #[test]
    fn test_validate_folder_path_empty_string() {
        // Behavior: Empty string should be rejected
        let result = validate_folder_path("");
        assert!(result.is_err(), "Empty string should be rejected");

        let error = result.unwrap_err();
        assert!(
            error.contains("empty") || error.contains("Empty"),
            "Error should mention empty path, got: {}", error
        );
    }

    #[test]
    fn test_validate_folder_path_whitespace_only() {
        // Behavior: Whitespace-only string should be rejected
        let result = validate_folder_path("   ");
        assert!(result.is_err(), "Whitespace-only string should be rejected");

        let error = result.unwrap_err();
        assert!(
            error.contains("empty") || error.contains("Empty") || error.contains("invalid"),
            "Error should indicate invalid path, got: {}", error
        );
    }

    /// Test suite for generate_preview_window_title
    /// Tests window title generation business logic
    #[test]
    fn test_generate_preview_window_title_multi_component_path() {
        // Behavior: Should extract last two path components
        let title = generate_preview_window_title("/Users/test/blog");
        assert_eq!(title, "website preview of test/blog");
    }

    #[test]
    fn test_generate_preview_window_title_single_component() {
        // Behavior: Single component should show just that component
        let title = generate_preview_window_title("/single");
        assert_eq!(title, "website preview of single");
    }

    #[test]
    fn test_generate_preview_window_title_deep_path() {
        // Behavior: Should show only last two components even for deep paths
        let title = generate_preview_window_title("/a/b/c/d/e");
        assert_eq!(title, "website preview of d/e");
    }

    #[test]
    fn test_generate_preview_window_title_empty_path() {
        // Behavior: Empty path should show "Unknown"
        let title = generate_preview_window_title("");
        assert_eq!(title, "website preview of Unknown");
    }

    #[test]
    fn test_generate_preview_window_title_root_path() {
        // Behavior: Root path should have reasonable fallback
        let title = generate_preview_window_title("/");
        assert!(
            title.contains("Unknown") || title == "website preview of /",
            "Root path should have reasonable title, got: {}", title
        );
    }

    #[test]
    fn test_generate_preview_window_title_with_spaces() {
        // Behavior: Should handle paths with spaces
        let title = generate_preview_window_title("/Users/test/My Documents");
        assert_eq!(title, "website preview of test/My Documents");
    }
}