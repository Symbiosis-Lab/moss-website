//! Window management utilities for preview windows
//!
//! Provides shared logic for creating and configuring preview windows
//! with compilation configuration injection.

use tauri::Manager;

/// Escape path string for safe embedding in JavaScript string literals
///
/// Handles special characters that need escaping in JavaScript strings:
/// - Backslashes: \ → \\
/// - Double quotes: " → \"
/// - Newlines: \n → \\n
///
/// # Arguments
/// * `path` - Filesystem path to escape
///
/// # Returns
/// * Escaped string safe for embedding in JavaScript
///
/// # Examples
/// ```
/// let escaped = escape_path_for_js(r#"C:\Users\test"#);
/// assert_eq!(escaped, r#"C:\\Users\\test"#);
/// ```
fn escape_path_for_js(path: &str) -> String {
    path.replace("\\", "\\\\").replace("\"", "\\\"")
}

/// Generate initialization script for preview window
///
/// Creates JavaScript that injects compilation configuration into the window.
/// The frontend detects `window.__MOSS_COMPILE_CONFIG__` and starts compilation.
///
/// # Arguments
/// * `folder_path` - Absolute path to folder to compile
///
/// # Returns
/// * JavaScript initialization script as string
///
/// # Examples
/// ```
/// let script = generate_compile_init_script("/Users/test/blog");
/// assert!(script.contains("window.__MOSS_COMPILE_CONFIG__"));
/// assert!(script.contains("/Users/test/blog"));
/// ```
pub fn generate_compile_init_script(folder_path: &str) -> String {
    format!(
        r#"
          window.__MOSS_COMPILE_CONFIG__ = {{
            folder_path: "{}",
            auto_serve: true
          }};
        "#,
        escape_path_for_js(folder_path)
    )
}

/// Create and show preview window for folder compilation
///
/// This function:
/// 1. Switches to Regular activation policy (makes window visible)
/// 2. Closes any existing main window
/// 3. Creates new main window with compile config injected
/// 4. Shows and focuses the window
/// 5. On error, restores Accessory mode
///
/// # Arguments
/// * `app` - Tauri application handle
/// * `folder_path` - Absolute path to folder to compile
///
/// # Returns
/// * `Ok(WebviewWindow)` - Successfully created window
/// * `Err(String)` - Error message if creation failed
///
/// # Side Effects
/// - Changes activation policy to Regular
/// - Closes existing "main" window
/// - Creates new window
/// - On error, restores Accessory mode
pub fn create_preview_window(
    app: &tauri::AppHandle,
    folder_path: &str,
) -> Result<tauri::WebviewWindow, String> {
    // Switch to Regular activation policy so window appears properly
    // (Accessory mode windows don't show in Mission Control or cmd+tab)
    if let Err(e) = app.set_activation_policy(tauri::ActivationPolicy::Regular) {
        eprintln!("⚠️ Failed to set Regular activation policy: {}", e);
        // Continue anyway - window might still work
    }

    // Close any existing main window to ensure clean state
    if let Some(existing_window) = app.get_webview_window("main") {
        let _ = existing_window.close();
    }

    // Generate initialization script with folder path
    let init_script = generate_compile_init_script(folder_path);

    // Get short path for window title
    let title = format!(
        "website preview of {}",
        crate::system::get_short_path(folder_path)
    );

    // Create window with compile configuration injected
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
            // Restore Accessory mode on error
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Err(format!("Failed to create main window: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_path_basic() {
        let path = "/Users/test/blog";
        let escaped = escape_path_for_js(path);
        assert_eq!(escaped, "/Users/test/blog");
    }

    #[test]
    fn test_escape_path_windows_backslashes() {
        let path = r"C:\Users\test\blog";
        let escaped = escape_path_for_js(path);
        assert_eq!(escaped, r"C:\\Users\\test\\blog");
    }

    #[test]
    fn test_escape_path_quotes() {
        let path = r#"/Users/test/"my blog""#;
        let escaped = escape_path_for_js(path);
        assert_eq!(escaped, r#"/Users/test/\"my blog\""#);
    }

    #[test]
    fn test_escape_path_combined_special_chars() {
        let path = r#"C:\Users\"test"\blog"#;
        let escaped = escape_path_for_js(path);
        assert_eq!(escaped, r#"C:\\Users\\\"test\"\\blog"#);
    }

    #[test]
    fn test_generate_init_script_basic_path() {
        let script = generate_compile_init_script("/Users/test/blog");

        // Verify structure
        assert!(script.contains("window.__MOSS_COMPILE_CONFIG__"));
        assert!(script.contains("folder_path"));
        assert!(script.contains("auto_serve"));

        // Verify path is included
        assert!(script.contains("/Users/test/blog"));

        // Verify auto_serve is true
        assert!(script.contains("auto_serve: true"));
    }

    #[test]
    fn test_generate_init_script_with_spaces() {
        let script = generate_compile_init_script("/Users/test/My Documents");

        // Path with spaces should be preserved
        assert!(script.contains("/Users/test/My Documents"));
    }

    #[test]
    fn test_generate_init_script_windows_path() {
        let script = generate_compile_init_script(r"C:\Users\test\blog");

        // Backslashes should be escaped
        assert!(script.contains(r"C:\\Users\\test\\blog"));

        // Should not contain unescaped backslashes in the path
        // (except in the format string structure itself)
        let path_section = script
            .split("folder_path:")
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap();
        assert!(path_section.contains(r"\\"));
    }
}
