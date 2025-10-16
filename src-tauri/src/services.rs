//! macOS NSServices integration for context menu "Publish" functionality
//!
//! This module implements native macOS NSServices to provide right-click
//! "Publish" integration in Finder, appearing at the top level of the context menu.
//!
//! ## Architecture
//!
//! - Pure business logic modules (fully unit tested):
//!   - `path_extractor` - Extract and validate folder paths from NSPasteboard
//!   - `deep_link` - Generate and parse moss:// deep link URLs
//!
//! - System integration modules (integration tested):
//!   - `handler` - objc2 service handler (thin glue code)
//!   - `registration` - Service registration with NSApplication
//!
//! ## Usage
//!
//! ```rust
//! #[cfg(target_os = "macos")]
//! use services;
//!
//! // In main app setup:
//! services::init(app)?;
//! ```

// Public API re-exports
pub use self::path_extractor::{extract_folder_path_from_urls, validate_folder_path, PathError};
pub use self::deep_link::{generate_publish_deep_link, parse_deep_link, DeepLinkError};

#[cfg(target_os = "macos")]
pub use self::registration::register_services;

// Internal modules
#[cfg(target_os = "macos")]
mod handler;
mod path_extractor;
mod deep_link;
#[cfg(target_os = "macos")]
mod registration;

/// Initialize NSServices integration
///
/// Registers the service handler with NSApplication and refreshes the services cache.
/// This function should be called once during app startup.
///
/// # Arguments
/// * `app` - Tauri app handle
///
/// # Returns
/// * `Ok(())` - Service successfully registered
/// * `Err(String)` - Registration failed with error message
#[cfg(target_os = "macos")]
pub fn init(app: &tauri::AppHandle) -> Result<(), String> {
    registration::register_services(app)
}
