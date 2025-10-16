//! Deep link URL generation and parsing
//!
//! Handles moss:// URL scheme generation and parsing.

#[derive(Debug, PartialEq)]
pub enum DeepLinkError {
    InvalidScheme,
    MissingPath,
    InvalidEncoding,
}

impl std::fmt::Display for DeepLinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeepLinkError::InvalidScheme => write!(f, "Invalid URL scheme (expected moss://)"),
            DeepLinkError::MissingPath => write!(f, "Missing path parameter"),
            DeepLinkError::InvalidEncoding => write!(f, "Invalid URL encoding"),
        }
    }
}

impl std::error::Error for DeepLinkError {}

/// Generate a moss:// deep link URL for publishing a folder
///
/// # Arguments
/// * `folder_path` - Absolute filesystem path to the folder
///
/// # Returns
/// * Deep link URL in format: `moss://publish?path=/encoded/path`
///
/// # Examples
/// ```
/// use moss::services::generate_publish_deep_link;
///
/// let url = generate_publish_deep_link("/Users/test/My Documents");
/// assert_eq!(url, "moss://publish?path=/Users/test/My%20Documents");
/// ```
pub fn generate_publish_deep_link(folder_path: &str) -> String {
    // Use url::form_urlencoded to properly encode query parameters
    // This preserves forward slashes while encoding special characters
    use url::form_urlencoded;

    let encoded = form_urlencoded::Serializer::new(String::new())
        .append_pair("path", folder_path)
        .finish();

    format!("moss://publish?{}", encoded)
}

/// Parse a moss:// deep link URL to extract the folder path
///
/// # Arguments
/// * `url` - Deep link URL to parse
///
/// # Returns
/// * `Ok(String)` - Decoded folder path
/// * `Err(DeepLinkError)` - Invalid URL format or missing parameters
///
/// # Examples
/// ```
/// use moss::services::parse_deep_link;
///
/// let path = parse_deep_link("moss://publish?path=/Users/test/Desktop").unwrap();
/// assert_eq!(path, "/Users/test/Desktop");
/// ```
pub fn parse_deep_link(url: &str) -> Result<String, DeepLinkError> {
    // Check scheme
    if !url.starts_with("moss://") {
        return Err(DeepLinkError::InvalidScheme);
    }

    // Parse URL
    let parsed = url::Url::parse(url).map_err(|_| DeepLinkError::InvalidEncoding)?;

    // Extract path query parameter
    let path = parsed
        .query_pairs()
        .find(|(key, _)| key == "path")
        .map(|(_, value)| value.to_string())
        .ok_or(DeepLinkError::MissingPath)?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_publish_deep_link_simple_path() {
        let url = generate_publish_deep_link("/Users/test/Desktop");
        // Verify it's a valid moss:// URL and can be parsed back
        assert!(url.starts_with("moss://publish?path="));
        let parsed = parse_deep_link(&url).unwrap();
        assert_eq!(parsed, "/Users/test/Desktop");
    }

    #[test]
    fn test_generate_publish_deep_link_with_spaces() {
        let url = generate_publish_deep_link("/Users/test/My Documents");
        // Verify it's a valid moss:// URL and can be parsed back
        assert!(url.starts_with("moss://publish?path="));
        let parsed = parse_deep_link(&url).unwrap();
        assert_eq!(parsed, "/Users/test/My Documents");
    }

    #[test]
    fn test_generate_publish_deep_link_with_special_chars() {
        let url = generate_publish_deep_link("/Users/test/docs & files");
        // Verify it's a valid moss:// URL and can be parsed back
        assert!(url.starts_with("moss://publish?path="));
        let parsed = parse_deep_link(&url).unwrap();
        assert_eq!(parsed, "/Users/test/docs & files");
    }

    #[test]
    fn test_parse_deep_link_simple_path() {
        let path = parse_deep_link("moss://publish?path=/Users/test/Desktop").unwrap();
        assert_eq!(path, "/Users/test/Desktop");
    }

    #[test]
    fn test_parse_deep_link_with_encoded_spaces() {
        let path = parse_deep_link("moss://publish?path=/Users/test/My%20Documents").unwrap();
        assert_eq!(path, "/Users/test/My Documents");
    }

    #[test]
    fn test_parse_deep_link_invalid_scheme() {
        let result = parse_deep_link("https://example.com/publish?path=/test");
        assert_eq!(result, Err(DeepLinkError::InvalidScheme));
    }

    #[test]
    fn test_parse_deep_link_missing_path() {
        let result = parse_deep_link("moss://publish");
        assert_eq!(result, Err(DeepLinkError::MissingPath));
    }

    #[test]
    fn test_roundtrip_simple_path() {
        let original = "/Users/test/Desktop";
        let url = generate_publish_deep_link(original);
        let parsed = parse_deep_link(&url).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_roundtrip_complex_path() {
        let original = "/Users/test/My Documents & Files";
        let url = generate_publish_deep_link(original);
        let parsed = parse_deep_link(&url).unwrap();
        assert_eq!(parsed, original);
    }
}
