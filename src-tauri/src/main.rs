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

// Publishing will be triggered from OS context menu, not from tray
// TODO: Implement handler for OS-level right-click context menu integration

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
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
        .invoke_handler(tauri::generate_handler![greet, test_tray_icon])
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
}