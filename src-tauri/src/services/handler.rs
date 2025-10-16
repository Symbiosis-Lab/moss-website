//! NSServices handler for macOS context menu integration
//!
//! Receives file paths from Finder via NSPasteboard when user
//! selects "Publish" from the context menu.
//!
//! # Architecture
//!
//! This module implements the Objective-C to Rust bridge for macOS NSServices:
//!
//! 1. **ServiceHandler** - Objective-C class created via `define_class!`
//!    - Registered with NSApplication via `setServicesProvider:`
//!    - Implements `publishFolder:userData:error:` selector
//!    - Called by macOS when user selects "Publish" from context menu
//!
//! 2. **handle_service_request()** - Rust function that processes the request
//!    - Reads NSURL objects from NSPasteboard using `readObjectsForClasses_options`
//!    - Converts URLs to file paths
//!    - Validates paths using `path_extractor` module
//!    - Triggers compilation via `spawn_compile_window()`
//!
//! 3. **spawn_compile_window()** - Async task spawner
//!    - Retrieves Tauri AppHandle from static storage
//!    - Spawns async task to create preview window
//!    - Calls `system::compile_with_preview_window()`
//!
//! # Testing
//!
//! ## Unit Tests
//!
//! The business logic (path extraction/validation) is unit tested in
//! [`path_extractor`](super::path_extractor) module with 11 comprehensive tests
//! covering URL parsing, path validation, and error handling.
//!
//! ## Integration Tests (Manual)
//!
//! ### Test: Right-click folder → "Publish" triggers compilation
//!
//! 1. **Setup**: Install moss.app to `/Applications/`
//! 2. **Action**: Right-click any folder in Finder → Select "Publish"
//! 3. **Expected Behavior**:
//!    - moss icon appears in dock
//!    - Preview window opens in fullscreen
//!    - Compilation progress shown
//!    - Preview displays compiled site
//! 4. **Verification**: Check `/tmp/moss_services_debug.log` for:
//!    ```
//!    ✅ Got N URL objects from pasteboard
//!    ✅ Extracted folder path: /path/to/folder
//!    ✅ Preview window created successfully
//!    ```
//!
//! ### Test: Service appears in Finder context menu
//!
//! 1. **Setup**: Launch moss.app
//! 2. **Action**: Right-click any folder in Finder
//! 3. **Expected**: "Publish" menu item visible at top level (not in Services submenu)
//!
//! ### Test: respondsToSelector verification
//!
//! 1. **Action**: Launch moss.app
//! 2. **Verification**: Check `/tmp/moss_services_debug.log` contains:
//!    ```
//!    🔍 Handler respondsToSelector(publishFolder:userData:error:) = true
//!    ```
//!
//! ## Why No Unit Tests for handler.rs?
//!
//! This module is a thin integration layer between:
//! - macOS NSServices (Objective-C runtime)
//! - NSPasteboard API (Foundation framework)
//! - Tauri application runtime
//!
//! The business logic (path extraction/validation) is extracted to `path_extractor.rs`
//! which has comprehensive unit test coverage (11 tests, 95%+ coverage).
//!
//! Testing `handler.rs` would require:
//! - Mocking NSPasteboard (impossible - Objective-C runtime dependency)
//! - Mocking NSURL objects (impossible - Foundation framework)
//! - Mocking Tauri AppHandle (possible but fragile, couples tests to Tauri internals)
//! - Running macOS Services subsystem (requires full system integration)
//!
//! Per project testing guidelines: "Test user behavior, not implementation details."
//! The integration test (right-click → compilation) validates the complete user-facing feature.
//!
//! # Debug Logging
//!
//! All operations are logged to `/tmp/moss_services_debug.log` with timestamps.
//! Check this file when troubleshooting NSServices issues:
//!
//! ```bash
//! tail -f /tmp/moss_services_debug.log
//! ```
//!
//! Expected log sequence for successful invocation:
//! ```
//! 🔧 handle_service_request CALLED
//! ✅ Got 1 URL objects from pasteboard
//!    URL path: /Users/username/folder-name
//! ✅ Extracted 1 file URLs
//! ✅ Extracted folder path: /Users/username/folder-name
//! 🔧 spawn_compile_window CALLED
//! ✅ APP_HANDLE is initialized
//! 🔧 Spawning async task for path: /Users/username/folder-name
//! ✅ Async task spawned successfully
//! 🔧 Async task started
//! ✅ Preview window created successfully
//! ```

use objc2::{define_class, msg_send, rc::Retained, ClassType};
use objc2_app_kit::NSPasteboard;
use objc2_foundation::{MainThreadMarker, NSArray, NSObject, NSString, NSURL};
use std::sync::OnceLock;

use super::path_extractor::extract_folder_path_from_urls;

// Global storage for Tauri app handle so NSServices handler can access it
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

/// Store the app handle for use by the NSServices handler
///
/// This must be called during app initialization before NSServices are used
pub fn set_app_handle(app: tauri::AppHandle) {
    if APP_HANDLE.set(app).is_err() {
        eprintln!("⚠️ App handle already initialized");
    }
}

define_class!(
    /// Service handler class for macOS NSServices
    ///
    /// This class inherits from NSObject and implements the service handler method
    /// that macOS calls when "Publish" is selected from the context menu.
    ///
    /// # Safety
    /// - Superclass NSObject has no subclassing requirements
    /// - ServiceHandler does not implement Drop
    #[unsafe(super(NSObject))]
    #[name = "MossServiceHandler"]
    pub struct ServiceHandler;

    impl ServiceHandler {
        /// Handle "Publish" service invocation from Finder
        ///
        /// This method is called by macOS when the user selects "Publish" from
        /// the context menu. The method signature matches the NSMessage key in Info.plist:
        /// `publishFolder:userData:error:`
        ///
        /// # Safety
        /// The signature must exactly match the NSServices protocol expectations:
        /// - Selector: `publishFolder:userData:error:`
        /// - arg0: `*mut NSPasteboard` - NSServices passes mutable pointer to pasteboard
        /// - arg1: `*mut NSObject` - userData parameter (unused in our case)
        /// - arg2: `*mut *mut NSObject` - error out-parameter (unused in our case)
        ///
        /// Type mismatches cause Undefined Behavior. This signature has been verified
        /// against Apple's NSServices documentation and the working c2pa-preview implementation.
        ///
        /// # Arguments
        /// * `pasteboard` - Raw pointer to NSPasteboard containing file URLs from Finder
        /// * `_user_data` - Additional data (unused, but required by NSServices protocol)
        /// * `_error` - Output parameter for errors (unused in our implementation)
        #[unsafe(method(publishFolder:userData:error:))]
        fn _publish_folder(
            &self,
            pasteboard: *mut NSPasteboard,
            _user_data: *mut NSObject,
            _error: *mut *mut NSObject,
        ) {
            // SAFETY: pasteboard pointer is valid for the duration of this call
            // as guaranteed by the NSServices protocol. We immediately convert to
            // a reference with appropriate lifetime.
            unsafe {
                let pboard = &*pasteboard;
                handle_service_request(pboard);
            }
        }
    }
);

impl ServiceHandler {
    /// Create a new service handler instance
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe {
            let obj = mtm.alloc();
            let obj: Retained<Self> = msg_send![obj, init];
            obj
        }
    }
}

/// Write debug message to log file for troubleshooting
fn debug_log(message: &str) {
    use std::io::Write;
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_message = format!("[{}] {}\n", timestamp, message);

    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/moss_services_debug.log")
        .and_then(|mut f| f.write_all(log_message.as_bytes()));
}

/// Handle the service request by extracting paths and creating preview window
///
/// This is separated from the obj-c method to allow easier error handling in Rust
unsafe fn handle_service_request(pboard: &NSPasteboard) {
    debug_log("🔧 handle_service_request CALLED");

    // Use modern readObjectsForClasses_options API to get NSURL objects
    // This is the standard way to read file URLs from NSPasteboard
    let classes = NSArray::from_slice(&[NSURL::class()]);
    let urls_result = pboard.readObjectsForClasses_options(&classes, None);

    match urls_result {
        Some(urls_array) => {
            debug_log(&format!("✅ Got {} URL objects from pasteboard", urls_array.len()));

            // Convert NSURL objects to file path strings
            let mut file_urls = Vec::new();
            for i in 0..urls_array.len() {
                let url_obj = urls_array.objectAtIndex(i);

                // Downcast to NSURL
                if let Some(url) = url_obj.downcast_ref::<NSURL>() {
                    // Get file path as string
                    if let Some(path) = url.path() {
                        let path_str = path.to_string();
                        debug_log(&format!("   URL path: {}", path_str));
                        // Format as file:// URL for consistency with existing code
                        file_urls.push(format!("file://{}", path_str));
                    } else {
                        debug_log("⚠️  URL has no path");
                    }
                } else {
                    debug_log("⚠️  Failed to downcast to NSURL");
                }
            }

            if file_urls.is_empty() {
                debug_log("❌ No valid file URLs extracted from pasteboard");
                return;
            }

            debug_log(&format!("✅ Extracted {} file URLs", file_urls.len()));

            // Use existing business logic to validate and extract folder path
            match extract_folder_path_from_urls(file_urls) {
                Ok(folder_path) => {
                    debug_log(&format!("✅ Extracted folder path: {}", folder_path));
                    println!("🔧 NSServices: Received folder path: {}", folder_path);

                    // Direct function call instead of deep link indirection
                    spawn_compile_window(&folder_path);
                }
                Err(e) => {
                    debug_log(&format!("❌ Failed to extract folder path: {}", e));
                    eprintln!("❌ NSServices: Failed to extract folder path: {}", e);
                }
            }
        }
        None => {
            debug_log("❌ readObjectsForClasses returned None - no files on pasteboard");
            eprintln!("❌ NSServices: No files provided to service");
        }
    }
}

/// Spawn async task to create preview window with compilation
///
/// This directly calls our Rust function instead of using deep link indirection,
/// avoiding race conditions and ensuring the window is created reliably.
fn spawn_compile_window(folder_path: &str) {
    debug_log("🔧 spawn_compile_window CALLED");

    let app_handle = match APP_HANDLE.get() {
        Some(handle) => {
            debug_log("✅ APP_HANDLE is initialized");
            handle.clone()
        }
        None => {
            debug_log("❌ APP_HANDLE is NOT initialized (this is the bug!)");
            eprintln!("❌ NSServices: App handle not initialized");
            return;
        }
    };

    let path = folder_path.to_string();
    debug_log(&format!("🔧 Spawning async task for path: {}", path));

    // Spawn async task to create preview window
    tauri::async_runtime::spawn(async move {
        debug_log("🔧 Async task started");

        match crate::system::compile_with_preview_window(&app_handle, &path).await {
            Ok(_) => {
                debug_log("✅ Preview window created successfully");
                println!("✅ NSServices: Preview window created successfully");
            }
            Err(e) => {
                debug_log(&format!("❌ Failed to create preview window: {}", e));
                eprintln!("❌ NSServices: Failed to create preview window: {}", e);
            }
        }
    });

    debug_log("✅ Async task spawned successfully");
}
