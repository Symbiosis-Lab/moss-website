//! Path extraction and validation from NSPasteboard file URLs
//!
//! This module handles the conversion of file:// URLs from NSPasteboard
//! into filesystem paths, with validation to ensure they point to folders.

use std::path::Path;

/// Errors that can occur during path extraction
#[derive(Debug, PartialEq)]
pub enum PathError {
    /// No file paths provided in the list
    EmptyPathList,
    /// Invalid file:// URL format
    InvalidUrl(String),
    /// Path points to a file, not a folder
    NotAFolder(String),
    /// Path contains non-UTF-8 characters
    NonUtf8Path,
    /// Path does not exist on filesystem
    PathDoesNotExist(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::EmptyPathList => write!(f, "No folder selected"),
            PathError::InvalidUrl(url) => write!(f, "Invalid file URL: {}", url),
            PathError::NotAFolder(path) => write!(f, "Not a folder: {}", path),
            PathError::NonUtf8Path => write!(f, "Path contains invalid characters"),
            PathError::PathDoesNotExist(path) => write!(f, "Path does not exist: {}", path),
        }
    }
}

impl std::error::Error for PathError {}

/// Extract folder path from NSPasteboard file URL array
///
/// Takes a list of file:// URLs (as provided by macOS NSPasteboard) and
/// returns the first valid folder path.
///
/// # Arguments
/// * `file_urls` - Array of file:// URL strings from NSPasteboard
///
/// # Returns
/// * `Ok(String)` - Valid folder path
/// * `Err(PathError)` - Path extraction or validation failed
///
/// # Examples
/// ```
/// use moss::services::extract_folder_path_from_urls;
///
/// let urls = vec!["file:///Users/test/Desktop".to_string()];
/// let path = extract_folder_path_from_urls(urls).unwrap();
/// assert_eq!(path, "/Users/test/Desktop");
/// ```
pub fn extract_folder_path_from_urls(file_urls: Vec<String>) -> Result<String, PathError> {
    // Check for empty list
    if file_urls.is_empty() {
        return Err(PathError::EmptyPathList);
    }

    // Take the first URL
    let file_url = &file_urls[0];

    // Convert file:// URL to path
    let path = file_url_to_path(file_url)?;

    // Validate it's a folder
    validate_folder_path(&path)?;

    Ok(path)
}

/// Convert file:// URL to filesystem path
///
/// Handles URL decoding of percent-encoded characters (e.g., %20 for spaces).
///
/// # Arguments
/// * `file_url` - file:// URL string
///
/// # Returns
/// * `Ok(String)` - Decoded filesystem path
/// * `Err(PathError)` - Invalid URL format
///
/// # Examples
/// ```
/// use moss::services::file_url_to_path;
///
/// let path = file_url_to_path("file:///Users/test/My%20Documents").unwrap();
/// assert_eq!(path, "/Users/test/My Documents");
/// ```
pub fn file_url_to_path(file_url: &str) -> Result<String, PathError> {
    // Check for file:// prefix
    if !file_url.starts_with("file://") {
        return Err(PathError::InvalidUrl(file_url.to_string()));
    }

    // Strip "file://" prefix and decode percent-encoded characters
    let path = &file_url[7..];

    // Use urlencoding to decode percent-encoded characters like %20
    match urlencoding::decode(path) {
        Ok(decoded) => Ok(decoded.to_string()),
        Err(_) => Err(PathError::InvalidUrl(file_url.to_string())),
    }
}

/// Validate that path is a folder (not a file)
///
/// Checks that the path exists and points to a directory.
///
/// # Arguments
/// * `path` - Filesystem path to validate
///
/// # Returns
/// * `Ok(())` - Path is a valid folder
/// * `Err(PathError)` - Path is invalid or not a folder
///
/// # Examples
/// ```
/// use moss::services::validate_folder_path;
///
/// let result = validate_folder_path("/tmp");
/// assert!(result.is_ok());
/// ```
pub fn validate_folder_path(path: &str) -> Result<(), PathError> {
    let path_buf = Path::new(path);

    // Check if path exists
    if !path_buf.exists() {
        return Err(PathError::PathDoesNotExist(path.to_string()));
    }

    // Check if it's a directory
    if !path_buf.is_dir() {
        return Err(PathError::NotAFolder(path.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_folder_path_from_valid_urls() {
        // Use a directory that actually exists on the system
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.to_str().unwrap();
        let file_url = format!("file://{}", temp_path);

        let urls = vec![file_url];
        let result = extract_folder_path_from_urls(urls);
        assert_eq!(result, Ok(temp_path.to_string()));
    }

    #[test]
    fn test_extract_folder_path_from_empty_list() {
        let urls = vec![];
        let result = extract_folder_path_from_urls(urls);
        assert_eq!(result, Err(PathError::EmptyPathList));
    }

    #[test]
    fn test_file_url_to_path_conversion() {
        assert_eq!(
            file_url_to_path("file:///Users/test/folder"),
            Ok("/Users/test/folder".to_string())
        );
    }

    #[test]
    fn test_file_url_to_path_with_spaces() {
        assert_eq!(
            file_url_to_path("file:///Users/test/My%20Documents"),
            Ok("/Users/test/My Documents".to_string())
        );
    }

    #[test]
    fn test_validate_folder_path_success() {
        // Create temp dir for test
        let temp_dir = std::env::temp_dir();
        let result = validate_folder_path(temp_dir.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_folder_path_not_a_folder() {
        // Use a file path
        let result = validate_folder_path("/etc/hosts");
        assert_eq!(result, Err(PathError::NotAFolder("/etc/hosts".to_string())));
    }

    #[test]
    fn test_validate_folder_path_does_not_exist() {
        let result = validate_folder_path("/nonexistent/folder");
        assert!(matches!(result, Err(PathError::PathDoesNotExist(_))));
    }
}
