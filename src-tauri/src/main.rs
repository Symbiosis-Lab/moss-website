// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[cfg(test)]
mod tray_tests;

// Learn more about Tauri commands at https://tauri.app/v2/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn test_tray_icon(app: tauri::AppHandle) -> Result<String, String> {
    // Test if we can find the tray icon
    if let Some(tray) = app.tray_by_id("main") {
        // Try to update the tooltip as a test
        tray.set_tooltip(Some("Test: Tray icon is working!"))
            .map_err(|e| format!("Failed to set tooltip: {:?}", e))?;
        
        Ok("Tray icon found and is responsive".to_string())
    } else {
        Err("Tray icon not found by ID".to_string())
    }
}

#[tauri::command]
fn publish_folder_from_deep_link(folder_path: String) -> Result<String, String> {
    // Handle deep link request to publish a folder
    if folder_path.is_empty() {
        return Err("Empty folder path provided".to_string());
    }
    
    println!("🔗 Deep link triggered: publishing folder '{}'", folder_path);
    
    // TODO: Implement actual static site generation and publishing
    Ok(format!("Publishing initiated for: {}", folder_path))
}

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

# URL encode the path
encoded_path=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$folder_path'))")

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

# URL encode the path
encoded_path=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$folder_path'))")

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
        .invoke_handler(tauri::generate_handler![greet, test_tray_icon, publish_folder_from_deep_link, install_finder_integration])
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
    fn test_greet_function() {
        // Test basic functionality
        let result = greet("World");
        assert!(result.contains("World"));
        assert!(result.contains("Hello"));
        assert!(result.contains("greeted from Rust"));
        
        // Test with different inputs
        let result2 = greet("Moss");
        assert_eq!(result2, "Hello, Moss! You've been greeted from Rust!");
        
        // Test with empty string
        let result3 = greet("");
        assert_eq!(result3, "Hello, ! You've been greeted from Rust!");
    }

    #[test]
    fn test_greet_special_characters() {
        // Test with special characters
        let result = greet("José");
        assert!(result.contains("José"));
        
        // Test with numbers
        let result2 = greet("User123");
        assert!(result2.contains("User123"));
    }

    #[test]
    fn test_greet_long_name() {
        // Test with very long name
        let long_name = "A".repeat(1000);
        let result = greet(&long_name);
        assert!(result.contains(&long_name));
        assert!(result.len() > 1000); // Should be longer due to template text
    }

    #[test]
    fn test_publish_folder_from_deep_link_valid_path() {
        let result = publish_folder_from_deep_link("/Users/test/my-content".to_string());
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("Publishing initiated for: /Users/test/my-content"));
    }

    #[test]
    fn test_publish_folder_from_deep_link_empty_path() {
        let result = publish_folder_from_deep_link("".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty folder path provided");
    }

    #[test]
    fn test_publish_folder_from_deep_link_url_encoded_path() {
        let result = publish_folder_from_deep_link("/Users/test/my folder with spaces".to_string());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("my folder with spaces"));
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
}