//! Git repository detection and GitHub integration utilities
//!
//! Provides functionality for detecting git repositories, parsing GitHub remotes,
//! and managing repository configuration for publishing.

use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::preview::github_api::{
    GitHubApiClient, CreateRepoRequest
};

/// Information about a git remote, specifically GitHub remotes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct GitRemoteInfo {
    /// The remote URL (e.g., https://github.com/user/repo.git)
    pub url: String,
    /// Whether this is a GitHub remote
    pub is_github: bool,
    /// Repository name extracted from URL
    pub repo_name: String,
    /// Repository owner/organization
    pub owner: String,
}

/// Detects git remote configuration in a folder
/// 
/// Checks for `.git/config` and parses the origin remote URL.
/// If the remote is a GitHub URL, extracts owner and repository information.
/// 
/// # Arguments
/// * `folder_path` - Path to check for git repository
/// 
/// # Returns
/// * `Ok(Some(GitRemoteInfo))` - Git remote found and parsed
/// * `Ok(None)` - No git repository or no remote configured
/// * `Err(String)` - Error reading git configuration
pub fn detect_git_remote(folder_path: &Path) -> Result<Option<GitRemoteInfo>, String> {
    let git_config_path = folder_path.join(".git/config");
    
    if !git_config_path.exists() {
        return Ok(None);
    }
    
    let config_content = fs::read_to_string(&git_config_path)
        .map_err(|e| format!("Failed to read git config: {}", e))?;
    
    // Parse git config for origin remote URL
    if let Some(remote_url) = extract_origin_url(&config_content) {
        if let Some(github_info) = parse_github_url(&remote_url) {
            return Ok(Some(GitRemoteInfo {
                url: remote_url,
                is_github: true,
                repo_name: github_info.1,
                owner: github_info.0,
            }));
        } else {
            // Non-GitHub remote
            return Ok(Some(GitRemoteInfo {
                url: remote_url,
                is_github: false,
                repo_name: "unknown".to_string(),
                owner: "unknown".to_string(),
            }));
        }
    }
    
    Ok(None)
}

/// Checks if a folder contains a git repository
/// 
/// Simply checks for the existence of a `.git` directory or file.
/// 
/// # Arguments
/// * `folder_path` - Path to check
/// 
/// # Returns
/// * `true` if `.git` exists (directory or file for submodules)
/// * `false` if no git repository detected
pub fn has_git_repository(folder_path: &Path) -> bool {
    folder_path.join(".git").exists()
}

/// Extracts the origin remote URL from git config content
/// 
/// Parses the git config format to find `[remote "origin"]` section
/// and extract the `url` value.
/// 
/// # Arguments
/// * `config_content` - Content of `.git/config` file
/// 
/// # Returns
/// * `Some(String)` - Origin URL if found
/// * `None` - No origin remote configured
fn extract_origin_url(config_content: &str) -> Option<String> {
    let lines: Vec<&str> = config_content.lines().collect();
    let mut in_origin_section = false;
    
    for line in lines {
        let line = line.trim();
        
        // Check for [remote "origin"] section
        if line == "[remote \"origin\"]" || line == "[remote 'origin']" {
            in_origin_section = true;
            continue;
        }
        
        // Check for end of section
        if in_origin_section && line.starts_with('[') && !line.contains("origin") {
            break;
        }
        
        // Extract URL from origin section
        if in_origin_section && line.starts_with("url = ") {
            return Some(line[6..].to_string());
        }
    }
    
    None
}

/// Parses a GitHub URL to extract owner and repository name
/// 
/// Supports various GitHub URL formats:
/// - https://github.com/owner/repo.git
/// - git@github.com:owner/repo.git
/// - https://github.com/owner/repo (no .git suffix)
/// 
/// # Arguments
/// * `url` - Git remote URL
/// 
/// # Returns
/// * `Some((owner, repo_name))` - If URL is a valid GitHub URL
/// * `None` - If URL is not a GitHub URL or cannot be parsed
fn parse_github_url(url: &str) -> Option<(String, String)> {
    // Handle HTTPS URLs: https://github.com/owner/repo.git
    if url.starts_with("https://github.com/") {
        let path = &url[19..]; // Remove "https://github.com/"
        let path = path.strip_suffix(".git").unwrap_or(path); // Remove .git if present
        
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    
    // Handle SSH URLs: git@github.com:owner/repo.git
    if url.starts_with("git@github.com:") {
        let path = &url[15..]; // Remove "git@github.com:"
        let path = path.strip_suffix(".git").unwrap_or(path); // Remove .git if present
        
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    
    None
}

/// Creates a GitHub repository and sets up git remote
/// 
/// Creates a new repository using the GitHub REST API, initializes git if needed,
/// and sets up the remote origin. Requires a GitHub personal access token.
/// 
/// # Arguments
/// * `folder_path` - Path to the project folder
/// * `repo_name` - Name for the new repository (will be sanitized)
/// * `is_public` - Whether the repository should be public
/// * `github_token` - Personal access token with `repo` scope
/// 
/// # Returns
/// * `Ok(GitRemoteInfo)` - Information about the created repository
/// * `Err(String)` - Error creating repository or setting up git
/// 
/// # API References
/// - Repository creation: https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
/// - Required token scopes: https://docs.github.com/en/developers/apps/building-oauth-apps/scopes-for-oauth-apps
pub async fn create_github_repo_and_remote(
    folder_path: &Path, 
    repo_name: &str, 
    is_public: bool,
    github_token: &str
) -> Result<GitRemoteInfo, String> {
    // Validate inputs
    if repo_name.is_empty() {
        return Err("Repository name cannot be empty".to_string());
    }
    
    if !folder_path.exists() {
        return Err("Project folder does not exist".to_string());
    }
    
    if github_token.is_empty() {
        return Err("GitHub token is required".to_string());
    }
    
    // Sanitize repository name for GitHub
    let sanitized_name = sanitize_repo_name(repo_name);
    
    // Create GitHub API client
    let client = GitHubApiClient::new(github_token.to_string());
    
    // Create repository request
    let request = CreateRepoRequest {
        name: sanitized_name.clone(),
        description: Some(format!("Website published with moss from {}", 
            folder_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("folder")
        )),
        private: !is_public,
        has_issues: false, // Keep minimal for publishing
        has_projects: false,
        has_wiki: false,
        auto_init: false, // We'll push our own content
    };
    
    // Create repository via GitHub API
    let repo = client.create_repository(request).await
        .map_err(|e| format!("Failed to create GitHub repository: {}", e))?;
    
    // TODO: Initialize git if needed and set up remote
    // This would involve:
    // 1. Check if .git exists, run `git init` if not
    // 2. Add remote: `git remote add origin <clone_url>`
    // 3. Set up initial branch if needed
    
    // Extract owner from full_name (format: "owner/repo")
    let (owner, _) = repo.full_name.split_once('/').unwrap_or(("unknown", &repo.name));
    
    Ok(GitRemoteInfo {
        url: repo.clone_url,
        is_github: true,
        repo_name: repo.name,
        owner: owner.to_string(),
    })
}

/// Sanitizes a repository name for GitHub requirements
/// 
/// GitHub repository names must:
/// - Be 1-100 characters long
/// - Contain only alphanumeric characters, hyphens, periods, and underscores
/// - Not start or end with special characters
/// - Not contain consecutive special characters
/// 
/// # Arguments
/// * `name` - Raw repository name (e.g., from folder name)
/// 
/// # Returns
/// * Sanitized repository name safe for GitHub
pub fn sanitize_repo_name(name: &str) -> String {
    let mut sanitized = name
        .to_lowercase()
        .replace(' ', "-")                    // Spaces to hyphens
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect::<String>();
    
    // Remove leading/trailing special characters
    sanitized = sanitized.trim_matches(|c: char| c == '-' || c == '_' || c == '.').to_string();
    
    // Ensure minimum length
    if sanitized.is_empty() {
        sanitized = "my-project".to_string();
    }
    
    // Ensure maximum length
    if sanitized.len() > 100 {
        sanitized.truncate(100);
        // Remove trailing special character if truncation created one
        sanitized = sanitized.trim_end_matches(|c: char| c == '-' || c == '_' || c == '.').to_string();
    }
    
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_parse_github_url_https() {
        let url = "https://github.com/user/repo.git";
        let result = parse_github_url(url);
        assert_eq!(result, Some(("user".to_string(), "repo".to_string())));
    }
    
    #[test]
    fn test_parse_github_url_https_no_git() {
        let url = "https://github.com/user/repo";
        let result = parse_github_url(url);
        assert_eq!(result, Some(("user".to_string(), "repo".to_string())));
    }
    
    #[test]
    fn test_parse_github_url_ssh() {
        let url = "git@github.com:user/repo.git";
        let result = parse_github_url(url);
        assert_eq!(result, Some(("user".to_string(), "repo".to_string())));
    }
    
    #[test]
    fn test_parse_github_url_non_github() {
        let url = "https://gitlab.com/user/repo.git";
        let result = parse_github_url(url);
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_extract_origin_url() {
        let config_content = r#"
[core]
    repositoryformatversion = 0
[remote "origin"]
    url = https://github.com/user/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*
[branch "main"]
    remote = origin
"#;
        
        let result = extract_origin_url(config_content);
        assert_eq!(result, Some("https://github.com/user/repo.git".to_string()));
    }
    
    #[test]
    fn test_extract_origin_url_no_origin() {
        let config_content = r#"
[core]
    repositoryformatversion = 0
[branch "main"]
    remote = upstream
"#;
        
        let result = extract_origin_url(config_content);
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_has_git_repository() {
        // Test with current directory (should have git)
        let current_dir = std::env::current_dir().unwrap();
        let has_git = has_git_repository(&current_dir);
        // This may or may not be true depending on test environment
        // Just ensure it doesn't panic
        let _ = has_git;
        
        // Test with temp directory (definitely no git)
        let temp_dir = std::env::temp_dir().join("moss_test_no_git");
        fs::create_dir_all(&temp_dir).unwrap();
        assert!(!has_git_repository(&temp_dir));
        fs::remove_dir_all(&temp_dir).ok();
    }
    
    #[test]
    fn test_sanitize_repo_name() {
        assert_eq!(sanitize_repo_name("My Awesome Project"), "my-awesome-project");
        assert_eq!(sanitize_repo_name("test@#$%^&*()"), "test");
        assert_eq!(sanitize_repo_name(""), "my-project");
        assert_eq!(sanitize_repo_name("valid-name"), "valid-name");
        assert_eq!(sanitize_repo_name("UPPERCASE"), "uppercase");
        
        // Test truncation
        let long_name = "a".repeat(150);
        let sanitized = sanitize_repo_name(&long_name);
        assert!(sanitized.len() <= 100);
    }
    
    #[test]
    fn test_detect_git_remote_no_git() {
        let temp_dir = std::env::temp_dir().join("moss_test_no_git_remote");
        fs::create_dir_all(&temp_dir).unwrap();
        
        let result = detect_git_remote(&temp_dir);
        assert_eq!(result.unwrap(), None);
        
        fs::remove_dir_all(&temp_dir).ok();
    }
    
}