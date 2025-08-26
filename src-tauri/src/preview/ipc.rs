//! IPC message validation and handling for preview windows
//!
//! Provides secure communication between preview windows and backend,
//! with validation to prevent injection and ensure data integrity.

use serde::{Deserialize, Serialize};

/// Commands that can be sent from preview window to backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PreviewCommand {
    Publish {
        preview_id: String,
        platform: String,
    },
    OpenEditor {
        preview_id: String,
    },
    AddSyndication {
        preview_id: String,
        target: String,
    },
    RemoveSyndication {
        preview_id: String,
        target: String,
    },
    RefreshPreview {
        preview_id: String,
    },
    ClosePreview {
        preview_id: String,
    },
}

/// Responses sent from backend to preview window
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PreviewResponse {
    Success {
        preview_id: String,
        message: String,
    },
    Error {
        preview_id: String,
        error: String,
    },
    StateUpdate {
        preview_id: String,
        is_published: bool,
        syndication_targets: Vec<String>,
    },
}

/// Validation errors for preview commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    MissingField(String),
    InvalidFormat(String),
    SecurityViolation(String),
    UnknownCommand,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ValidationError::InvalidFormat(field) => write!(f, "Invalid format for field: {}", field),
            ValidationError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            ValidationError::UnknownCommand => write!(f, "Unknown command type"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a preview command for security and completeness
pub fn validate_preview_command(cmd: &PreviewCommand) -> Result<(), ValidationError> {
    match cmd {
        PreviewCommand::Publish { preview_id, platform } => {
            validate_preview_id(preview_id)?;
            validate_platform_name(platform)?;
        }
        PreviewCommand::OpenEditor { preview_id } => {
            validate_preview_id(preview_id)?;
        }
        PreviewCommand::AddSyndication { preview_id, target } => {
            validate_preview_id(preview_id)?;
            validate_syndication_target(target)?;
        }
        PreviewCommand::RemoveSyndication { preview_id, target } => {
            validate_preview_id(preview_id)?;
            validate_syndication_target(target)?;
        }
        PreviewCommand::RefreshPreview { preview_id } => {
            validate_preview_id(preview_id)?;
        }
        PreviewCommand::ClosePreview { preview_id } => {
            validate_preview_id(preview_id)?;
        }
    }
    
    Ok(())
}

/// Sanitize preview response to remove sensitive information
pub fn sanitize_preview_response(response: PreviewResponse) -> PreviewResponse {
    match response {
        PreviewResponse::Error { preview_id, error } => {
            // Remove potentially sensitive path information from error messages
            let sanitized_error = error
                .replace(&std::env::current_dir().unwrap_or_default().to_string_lossy().to_string(), "[PROJECT_ROOT]")
                .replace(&dirs::home_dir().unwrap_or_default().to_string_lossy().to_string(), "[HOME]");
            
            PreviewResponse::Error {
                preview_id,
                error: sanitized_error,
            }
        }
        other => other, // Success and StateUpdate responses are already safe
    }
}

/// Validate preview ID format
fn validate_preview_id(id: &str) -> Result<(), ValidationError> {
    if id.is_empty() {
        return Err(ValidationError::MissingField("preview_id".to_string()));
    }
    
    // Preview IDs should follow pattern: preview_<timestamp>
    if !id.starts_with("preview_") {
        return Err(ValidationError::InvalidFormat("preview_id must start with 'preview_'".to_string()));
    }
    
    // Check for potentially dangerous characters
    if id.contains("..") || id.contains("/") || id.contains("\\") {
        return Err(ValidationError::SecurityViolation("preview_id contains dangerous characters".to_string()));
    }
    
    Ok(())
}

/// Validate platform name
fn validate_platform_name(platform: &str) -> Result<(), ValidationError> {
    if platform.is_empty() {
        return Err(ValidationError::MissingField("platform".to_string()));
    }
    
    // Only allow alphanumeric characters, dots, and hyphens for platform names
    if !platform.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
        return Err(ValidationError::InvalidFormat("platform name can only contain alphanumeric characters, dots, and hyphens".to_string()));
    }
    
    Ok(())
}

/// Validate syndication target
pub fn validate_syndication_target(target: &str) -> Result<(), ValidationError> {
    if target.is_empty() {
        return Err(ValidationError::MissingField("target".to_string()));
    }
    
    // Similar validation to platform name
    if !target.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_') {
        return Err(ValidationError::InvalidFormat("syndication target can only contain alphanumeric characters, dots, hyphens, and underscores".to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_preview_command_publish_valid() {
        let cmd = PreviewCommand::Publish {
            preview_id: "preview_123456".to_string(),
            platform: "moss.pub".to_string(),
        };
        
        let result = validate_preview_command(&cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_preview_command_missing_preview_id() {
        let cmd = PreviewCommand::Publish {
            preview_id: "".to_string(),
            platform: "moss.pub".to_string(),
        };
        
        let result = validate_preview_command(&cmd);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::MissingField(_)));
    }

    #[test]
    fn test_validate_preview_command_invalid_preview_id_format() {
        let cmd = PreviewCommand::Publish {
            preview_id: "invalid_id_format".to_string(),
            platform: "moss.pub".to_string(),
        };
        
        let result = validate_preview_command(&cmd);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidFormat(_)));
    }

    #[test]
    fn test_validate_preview_command_security_violation() {
        let cmd = PreviewCommand::Publish {
            preview_id: "preview_../../../etc/passwd".to_string(),
            platform: "moss.pub".to_string(),
        };
        
        let result = validate_preview_command(&cmd);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::SecurityViolation(_)));
    }

    #[test]
    fn test_validate_platform_name_valid() {
        assert!(validate_platform_name("moss.pub").is_ok());
        assert!(validate_platform_name("twitter-api").is_ok());
        assert!(validate_platform_name("dev.to").is_ok());
    }

    #[test]
    fn test_validate_platform_name_invalid() {
        assert!(validate_platform_name("").is_err());
        assert!(validate_platform_name("platform/with/slashes").is_err());
        assert!(validate_platform_name("platform with spaces").is_err());
        assert!(validate_platform_name("platform@with@symbols").is_err());
    }

    #[test]
    fn test_validate_syndication_target_valid() {
        assert!(validate_syndication_target("twitter").is_ok());
        assert!(validate_syndication_target("dev-to").is_ok());
        assert!(validate_syndication_target("my_blog").is_ok());
    }

    #[test]
    fn test_validate_syndication_target_invalid() {
        assert!(validate_syndication_target("").is_err());
        assert!(validate_syndication_target("target with spaces").is_err());
        assert!(validate_syndication_target("target/with/slashes").is_err());
    }

    #[test]
    fn test_sanitize_preview_response_removes_sensitive() {
        let response = PreviewResponse::Error {
            preview_id: "preview_123".to_string(),
            error: format!("Failed to read file at {}/secret/file.txt", 
                dirs::home_dir().unwrap_or_default().to_string_lossy()),
        };
        
        let sanitized = sanitize_preview_response(response);
        
        if let PreviewResponse::Error { error, .. } = sanitized {
            assert!(error.contains("[HOME]"));
            assert!(!error.contains("/Users/"));
            assert!(!error.contains("/home/"));
        } else {
            panic!("Expected error response");
        }
    }

    #[test]
    fn test_sanitize_preview_response_success_unchanged() {
        let response = PreviewResponse::Success {
            preview_id: "preview_123".to_string(),
            message: "Published successfully".to_string(),
        };
        
        let sanitized = sanitize_preview_response(response.clone());
        
        // Success responses should remain unchanged
        if let (PreviewResponse::Success { message: orig, .. }, 
                PreviewResponse::Success { message: sanitized, .. }) = (&response, &sanitized) {
            assert_eq!(orig, sanitized);
        } else {
            panic!("Expected success responses");
        }
    }
}