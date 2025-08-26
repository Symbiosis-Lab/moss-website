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

/// Add preview parameters to existing URL
pub fn add_preview_params(url: String, params: PreviewParams) -> String {
    let mut result = url;
    let separator = if result.contains('?') { "&" } else { "?" };
    
    let mut query_parts = Vec::new();
    
    if let Some(token) = params.refresh_token {
        query_parts.push(format!("token={}", urlencoding::encode(&token)));
    }
    
    if let Some(theme) = params.theme {
        query_parts.push(format!("theme={}", urlencoding::encode(&theme)));
    }
    
    if params.debug {
        query_parts.push("debug=true".to_string());
    }
    
    if !query_parts.is_empty() {
        result.push_str(separator);
        result.push_str(&query_parts.join("&"));
    }
    
    result
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

/// Extract folder path from preview URL
pub fn extract_folder_from_preview_url(url: &str) -> Option<PathBuf> {
    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(source) = parsed_url.query_pairs()
            .find(|(key, _)| key == "source")
            .map(|(_, value)| value.to_string()) 
        {
            // URL decode the path
            if let Ok(decoded) = urlencoding::decode(&source) {
                return Some(PathBuf::from(decoded.to_string()));
            }
        }
    }
    None
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
    fn test_add_preview_params_query_string() {
        let url = "http://localhost:8080?source=test".to_string();
        let params = PreviewParams {
            refresh_token: Some("abc123".to_string()),
            theme: Some("dark".to_string()),
            debug: true,
        };
        
        let result = add_preview_params(url, params);
        
        assert!(result.contains("&token=abc123"));
        assert!(result.contains("&theme=dark"));
        assert!(result.contains("&debug=true"));
    }

    #[test]
    fn test_add_preview_params_no_existing_query() {
        let url = "http://localhost:8080".to_string();
        let params = PreviewParams {
            refresh_token: Some("token123".to_string()),
            ..Default::default()
        };
        
        let result = add_preview_params(url, params);
        
        assert!(result.contains("?token=token123"));
    }

    #[test]
    fn test_add_preview_params_empty() {
        let url = "http://localhost:8080".to_string();
        let params = PreviewParams::default();
        
        let result = add_preview_params(url, params);
        
        assert_eq!(result, "http://localhost:8080");
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

    #[test]
    fn test_extract_folder_from_preview_url() {
        let url = "http://localhost:8080?source=%2FUsers%2Ftest%2FMy%20Documents";
        let result = extract_folder_from_preview_url(url);
        
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("/Users/test/My Documents"));
    }

    #[test]
    fn test_extract_folder_from_preview_url_invalid() {
        let url = "invalid-url";
        let result = extract_folder_from_preview_url(url);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_folder_no_source_param() {
        let url = "http://localhost:8080?theme=dark";
        let result = extract_folder_from_preview_url(url);
        
        assert!(result.is_none());
    }
}