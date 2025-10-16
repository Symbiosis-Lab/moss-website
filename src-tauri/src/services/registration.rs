//! Service registration with NSApplication
//!
//! Handles registering the NSServices handler with the macOS system.

use objc2::msg_send;
use objc2::rc::Retained;
use objc2_app_kit::NSApplication;
use objc2_foundation::MainThreadMarker;

use super::handler::ServiceHandler;

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

/// Register NSServices handler on app startup
///
/// This function:
/// 1. Stores the app handle for the handler to use
/// 2. Gets the NSApplication instance
/// 3. Creates a ServiceHandler instance
/// 4. Sets it as the services provider
/// 5. Attempts to refresh the services cache
///
/// # Arguments
/// * `app` - Tauri application handle
///
/// # Returns
/// * `Ok(())` - Service registered successfully
/// * `Err(String)` - Registration failed
pub fn register_services(app: &tauri::AppHandle) -> Result<(), String> {
    debug_log("🔧 register_services CALLED");

    // Store app handle for handler to use when creating windows
    super::handler::set_app_handle(app.clone());
    debug_log("✅ APP_HANDLE stored successfully");

    unsafe {
        // Get main thread marker (required for NSApplication APIs)
        let mtm = MainThreadMarker::new()
            .ok_or_else(|| "NSServices must be registered on main thread".to_string())?;

        // Get shared NSApplication instance
        let ns_app = NSApplication::sharedApplication(mtm);

        // Create service handler
        let handler: Retained<ServiceHandler> = ServiceHandler::new(mtm);

        // Register as services provider
        // Using msg_send since setServicesProvider might not be in objc2-app-kit
        let _: () = msg_send![&ns_app, setServicesProvider: &*handler];

        debug_log("✅ NSServices handler registered with NSApplication");
        println!("✅ NSServices handler registered");

        // Verify handler responds to our service selector
        use objc2::sel;
        let selector = sel!(publishFolder:userData:error:);
        let responds: bool = msg_send![&*handler, respondsToSelector: selector];
        debug_log(&format!("🔍 Handler respondsToSelector(publishFolder:userData:error:) = {}", responds));
        if !responds {
            debug_log("❌ WARNING: Handler does NOT respond to selector!");
        }

        // MEMORY MANAGEMENT: Intentionally leak the handler to keep it alive forever
        //
        // Why this is necessary:
        // 1. NSApplication's setServicesProvider: stores only a WEAK reference to the service provider
        // 2. This is by design - Apple allows service providers to be changed at runtime
        // 3. NSApplication does NOT assume ownership or retain the provider
        // 4. If we let `handler` go out of scope, Rust will drop it (deallocate the memory)
        // 5. NSApplication would then have a dangling pointer → crash when service is invoked
        //
        // The solution:
        // - Use std::mem::forget() to prevent Rust from running the destructor
        // - This "leaks" the memory in Rust's view, but that's intentional
        // - The handler must live for the entire application lifetime anyway
        // - This pattern is common in FFI when interfacing with C/Objective-C APIs
        //   that expect objects to remain valid indefinitely
        //
        // Alternative approaches considered:
        // - Static storage with OnceCell: More explicit, but requires additional dependency
        // - Tauri managed state: Cleaner, but handler is !Send which may conflict
        // - Current approach: Simple, explicit about intent, standard FFI pattern
        std::mem::forget(handler);

        // Try to refresh services cache (may not work until app restart)
        refresh_services_cache().ok();

        Ok(())
    }
}

/// Refresh services cache by killing pbs daemon
///
/// This function uses the approach of killing the pasteboard services daemon
/// to force macOS to re-scan Info.plist and update the Services menu.
pub fn refresh_services_cache() -> Result<(), String> {
    use std::process::Command;

    let user = std::env::var("USER").unwrap_or_default();

    let output = Command::new("killall")
        .args(&["-u", &user, "pbs"])
        .output()
        .map_err(|e| format!("Failed to refresh services cache: {}", e))?;

    if !output.status.success() {
        // pbs might not be running, that's okay
        eprintln!("⚠️  Could not refresh services cache automatically (pbs not running)");
    } else {
        println!("✅ Services cache refreshed");
    }

    Ok(())
}
