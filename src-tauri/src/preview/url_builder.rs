//! URL building utilities for preview functionality
//!
//! Handles construction and sanitization of preview URLs with proper encoding
//! and security considerations.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Parameters for preview URL construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewParams {
    pub refresh_token: Option<String>,
    pub theme: Option<String>,
    pub debug: bool,
}

impl Default for PreviewParams {
    fn default() -> Self {
        Self {
            refresh_token: None,
            theme: None,
            debug: false,
        }
    }
}

/// Build a preview URL from base URL and folder path
pub fn build_preview_url(base: &str, path: &Path) -> String {
    let path_str = sanitize_preview_path(path)
        .to_string_lossy()
        .to_string();
    
    // URL encode the path
    let encoded_path = urlencoding::encode(&path_str);
    
    format!("{}?source={}", base.trim_end_matches('/'), encoded_path)
}


/// Sanitize preview path for security
pub fn sanitize_preview_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    
    // Remove dangerous patterns
    let sanitized = path_str
        .replace("../", "")  // Prevent directory traversal
        .replace("..\\", "") // Windows directory traversal
        .replace("~", "")    // Home directory expansion
        .replace("$", "");   // Variable expansion
    
    PathBuf::from(sanitized)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_preview_url_encoding() {
        let base = "http://localhost:8080";
        let path = PathBuf::from("/Users/test/My Documents/blog");
        
        let result = build_preview_url(base, &path);
        
        assert!(result.contains("source="));
        assert!(result.contains("%20")); // Space should be encoded
        assert!(!result.contains(" ")); // No raw spaces
    }

    #[test]
    fn test_build_preview_url_trailing_slash() {
        let base = "http://localhost:8080/";
        let path = PathBuf::from("/simple/path");
        
        let result = build_preview_url(base, &path);
        
        assert_eq!(result, "http://localhost:8080?source=%2Fsimple%2Fpath");
    }




    #[test]
    fn test_sanitize_preview_path_security() {
        let dangerous_path = Path::new("../../../etc/passwd");
        let result = sanitize_preview_path(dangerous_path);
        
        assert!(!result.to_string_lossy().contains("../"));
        assert_eq!(result.to_string_lossy(), "etc/passwd");
    }

    #[test]
    fn test_sanitize_preview_path_windows() {
        let dangerous_path = Path::new("..\\..\\Windows\\System32");
        let result = sanitize_preview_path(dangerous_path);
        
        assert!(!result.to_string_lossy().contains("..\\"));
        assert_eq!(result.to_string_lossy(), "Windows\\System32");
    }

    #[test]
    fn test_sanitize_preview_path_variables() {
        let dangerous_path = Path::new("~/docs/$USER/file.txt");
        let result = sanitize_preview_path(dangerous_path);
        
        assert!(!result.to_string_lossy().contains("~"));
        assert!(!result.to_string_lossy().contains("$"));
        assert_eq!(result.to_string_lossy(), "/docs/USER/file.txt");
    }



}