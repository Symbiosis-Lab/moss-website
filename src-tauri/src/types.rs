//! Data types and structures for Moss publishing system

use serde::{Deserialize, Serialize};

/// System tray icon visibility status on different platforms.
/// 
/// Provides detailed information about whether the tray icon is actually
/// visible to the user, accounting for platform-specific behaviors like
/// macOS hiding icons when the menu bar is too crowded.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TrayVisibilityStatus {
    /// Tray icon creation failed or was not attempted
    NotAdded,
    /// Tray icon exists but is hidden by the system (e.g., macOS space constraints)
    AddedButHidden,
    /// Tray icon is visible and accessible to the user
    Visible,
}

/// System diagnostic information for debugging and user support.
/// 
/// Contains runtime information about the application's integration
/// with the operating system and current operational status.
/// Used by support commands and debugging workflows.
#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInfo {
    /// Operating system identifier (e.g., "macos", "windows", "linux")
    pub os: String,
    /// Current tray icon visibility status
    pub tray_status: TrayVisibilityStatus,
    /// Whether Finder integration is installed (macOS only)
    pub finder_integration: bool,
    /// Application version string from Cargo.toml
    pub app_version: String,
}

/// Metadata for a single file discovered during folder scanning.
/// 
/// Contains essential information needed for static site generation,
/// including file type classification and modification timestamps.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    /// Relative path from the scanned root directory
    pub path: String,
    /// File extension in lowercase (e.g., "md", "jpg", "html")
    pub file_type: String,
    /// File size in bytes
    pub size: u64,
    /// Unix timestamp as string, if available
    pub modified: Option<String>,
}

/// Website structure classification based on content organization patterns.
/// 
/// Automatically detected from folder structure to determine the most
/// appropriate static site generation strategy.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProjectType {
    /// Site with homepage and organized content in subdirectories
    /// (e.g., `/posts/`, `/projects/`, `/docs/`)
    HomepageWithCollections,
    /// Small site with ≤5 document files, all suitable for main navigation
    SimpleFlatSite,
    /// Large flat site with >5 files, requiring selective navigation menu
    BlogStyleFlatSite,
}

/// Complete analysis of a folder's contents for static site generation.
/// 
/// Contains categorized file listings and inferred project characteristics
/// used to determine the optimal site generation strategy.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectStructure {
    /// Absolute path to the scanned directory
    pub root_path: String,
    /// Markdown and text files (.md, .markdown, etc.)
    pub markdown_files: Vec<FileInfo>,
    /// HTML files (.html, .htm)
    pub html_files: Vec<FileInfo>,
    /// Image files (.jpg, .png, .gif, .svg, etc.)
    pub image_files: Vec<FileInfo>,
    /// All other files including documents (.docx, .pages, etc.)
    pub other_files: Vec<FileInfo>,
    /// Total count of all discovered files
    pub total_files: usize,
    /// Inferred site structure type
    pub project_type: ProjectType,
    /// Detected homepage file path, if any
    pub homepage_file: Option<String>,
    /// Subdirectories containing document files
    pub content_folders: Vec<String>,
}

/// Static site deployment configuration.
/// 
/// Specifies where and how the generated site should be published,
/// supporting multiple hosting providers and custom domains.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeploymentConfig {
    /// Hosting provider identifier ("moss.pub", "github", "netlify", etc.)
    pub provider: String,
    /// Custom domain name for the published site
    pub custom_domain: Option<String>,
    /// Whether to automatically republish when content changes
    pub auto_publish: bool,
}

/// User configuration for site generation and publishing.
/// 
/// Contains site metadata, theme settings, and deployment preferences.
/// Can be stored in `.moss/config.toml` within project directories.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MossConfig {
    /// Display name for the generated website
    pub site_name: Option<String>,
    /// Author name for metadata and attribution
    pub author: Option<String>,
    /// Theme identifier for styling the generated site
    pub theme: Option<String>,
    /// Local directory for generated site files
    pub output_dir: Option<String>,
    /// Base URL for absolute links (e.g., "<https://example.com>")
    pub base_url: Option<String>,
    /// Deployment configuration settings
    pub deployment: DeploymentConfig,
}