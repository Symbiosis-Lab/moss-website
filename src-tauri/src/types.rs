//! Data types and structures for moss publishing system

use serde::{Deserialize, Serialize};
use specta::Type;

/// Real-time progress update for compilation process.
/// 
/// Provides structured progress information for the frontend
/// to display during website compilation and server startup.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ProgressUpdate {
    /// Current step being executed
    pub step: String,
    /// Detailed message about current operation
    pub message: String,
    /// Progress completion percentage (0-100)
    pub percentage: u8,
    /// Whether this step is completed
    pub completed: bool,
    /// Preview server port when server is ready (optional)
    pub port: Option<u16>,
}

/// System diagnostic information for debugging and user support.
/// 
/// Contains runtime information about the application's integration
/// with the operating system and current operational status.
/// Used by support commands and debugging workflows.
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct SystemInfo {
    /// Operating system identifier (e.g., "macos", "windows", "linux")
    pub os: String,
    /// Whether Finder integration is installed (macOS only)
    pub finder_integration: bool,
    /// Application version string from Cargo.toml
    pub app_version: String,
}

/// Metadata for a single file discovered during folder scanning.
/// 
/// Contains essential information needed for static site generation,
/// including file type classification and modification timestamps.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
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
#[derive(Serialize, Deserialize, Debug, Type)]
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
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
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
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
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

/// Result of static site generation process.
/// 
/// Contains summary information about the generated site
/// including page count and output location.
#[derive(Serialize, Deserialize, Debug, Type)]
pub struct SiteResult {
    /// Number of HTML pages generated
    pub page_count: usize,
    /// Path to the generated site directory
    pub output_path: String,
    /// Site metadata extracted from content
    pub site_title: String,
}

/// Parsed markdown document with frontmatter and content.
/// 
/// Represents a processed markdown file ready for HTML generation.
/// Enhanced data model following Jekyll/Hugo patterns for consistent site generation.
/// References: 
/// - https://jekyllrb.com/docs/variables/
/// - https://gohugo.io/variables/page/
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ParsedDocument {
    /// Document title from frontmatter or filename
    pub title: String,
    /// Raw markdown content without frontmatter
    #[allow(dead_code)]
    pub content: String,
    /// HTML content generated from markdown
    pub html_content: String,
    /// Relative URL path for the generated page
    pub url_path: String,
    /// Publication date if specified in frontmatter
    pub date: Option<String>,
    /// Topics/categories from frontmatter
    pub topics: Vec<String>,
    /// Estimated reading time in minutes
    pub reading_time: u32,
    /// Excerpt or summary of the content
    #[allow(dead_code)]
    pub excerpt: String,
    /// URL-safe slug identifier (auto-generated from title/filename)
    /// Following Hugo slug conventions
    pub slug: String,
    /// Complete URL with depth-aware path generation
    /// Following Jekyll permalink patterns
    pub permalink: String,
    /// Preferred display title (H1 > frontmatter.title > filename)
    /// Following Eleventy computed data patterns
    pub display_title: String,
}

/// State management for tracking active preview servers.
///
/// Used to track which servers are running for which folders to enable server reuse
/// and avoid creating duplicate servers for the same content.
#[derive(Default, Debug)]
pub struct ServerState {
    /// Map of folder paths to their running server ports
    /// Enables server reuse when compiling the same folder multiple times
    pub active_servers: std::collections::HashMap<String, u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_state_registration() {
        let mut state = ServerState::default();
        state.active_servers.insert("/path1".to_string(), 3000);

        assert_eq!(state.active_servers.get("/path1"), Some(&3000));
        assert_eq!(state.active_servers.get("/nonexistent"), None);
    }

    #[test]
    fn test_server_state_removal() {
        let mut state = ServerState::default();
        state.active_servers.insert("/path1".to_string(), 3000);

        state.active_servers.remove("/path1");
        assert_eq!(state.active_servers.get("/path1"), None);
    }

    #[test]
    fn test_server_state_duplicate_paths() {
        let mut state = ServerState::default();
        state.active_servers.insert("/path1".to_string(), 3000);
        state.active_servers.insert("/path1".to_string(), 4000);

        // Should overwrite with new port
        assert_eq!(state.active_servers.get("/path1"), Some(&4000));
    }

    #[test]
    fn test_server_state_multiple_servers() {
        let mut state = ServerState::default();
        state.active_servers.insert("/path1".to_string(), 3000);
        state.active_servers.insert("/path2".to_string(), 4000);
        state.active_servers.insert("/path3".to_string(), 5000);

        assert_eq!(state.active_servers.len(), 3);
        assert_eq!(state.active_servers.get("/path2"), Some(&4000));
    }

    #[test]
    fn test_progress_update_creation() {
        let progress = ProgressUpdate {
            step: "scanning".to_string(),
            message: "Scanning files...".to_string(),
            percentage: 25,
            completed: false,
            port: None,
        };

        assert_eq!(progress.step, "scanning");
        assert_eq!(progress.message, "Scanning files...");
        assert_eq!(progress.percentage, 25);
        assert!(!progress.completed);
        assert_eq!(progress.port, None);
    }

    #[test]
    fn test_progress_update_with_port() {
        let progress = ProgressUpdate {
            step: "serving".to_string(),
            message: "Server started".to_string(),
            percentage: 100,
            completed: true,
            port: Some(3000),
        };

        assert_eq!(progress.step, "serving");
        assert!(progress.completed);
        assert_eq!(progress.port, Some(3000));
    }

    #[test]
    fn test_progress_update_boundary_values() {
        // Test minimum percentage
        let progress_min = ProgressUpdate {
            step: "start".to_string(),
            message: "Starting...".to_string(),
            percentage: 0,
            completed: false,
            port: None,
        };
        assert_eq!(progress_min.percentage, 0);

        // Test maximum percentage
        let progress_max = ProgressUpdate {
            step: "complete".to_string(),
            message: "Complete!".to_string(),
            percentage: 100,
            completed: true,
            port: Some(8080),
        };
        assert_eq!(progress_max.percentage, 100);
        assert!(progress_max.completed);
    }

    #[test]
    fn test_project_type_variants() {
        // Test that all ProjectType variants can be created
        let homepage_with_collections = ProjectType::HomepageWithCollections;
        let simple_flat = ProjectType::SimpleFlatSite;
        let blog_style = ProjectType::BlogStyleFlatSite;

        // Test equality
        assert_eq!(homepage_with_collections, ProjectType::HomepageWithCollections);
        assert_ne!(simple_flat, ProjectType::HomepageWithCollections);
        assert_ne!(blog_style, simple_flat);
    }
}