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






#[cfg(test)]
mod tests {










}