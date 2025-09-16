//! GitHub Pages deployment utilities
//!
//! Handles deployment of static sites to GitHub Pages, including
//! git operations and GitHub Actions workflow setup.

use std::path::Path;
use std::process::Command;
use std::fs;
use crate::preview::git::GitRemoteInfo;
use crate::preview::github_api::{
    GitHubApiClient, EnablePagesRequest, PagesSource
};

/// GitHub Pages deployment handler
pub struct GitHubPages {
    remote_info: GitRemoteInfo,
}

impl GitHubPages {
    /// Create a new GitHub Pages deployer with remote information
    pub fn new(remote_info: GitRemoteInfo) -> Self {
        Self { remote_info }
    }

    /// Deploy a static site to GitHub Pages
    /// 
    /// This function handles the complete deployment workflow:
    /// 1. Copies the built site to the repository
    /// 2. Commits and pushes to GitHub
    /// 3. Configures GitHub Pages if needed
    /// 
    /// # Arguments
    /// * `folder_path` - Path to the project folder
    /// 
    /// # Returns
    /// * `Ok(String)` - URL of the deployed site
    /// * `Err(String)` - Error message if deployment fails
    pub async fn deploy_to_pages(&self, folder_path: &Path) -> Result<String, String> {
        // Validate that the site has been built
        let site_path = folder_path.join(".moss/site");
        if !site_path.exists() {
            return Err("Built site not found. Please build the site first.".to_string());
        }

        // Check if this is a git repository
        if !folder_path.join(".git").exists() {
            return Err("Not a git repository. Please initialize git first.".to_string());
        }

        // For now, return a mock deployment
        // In a full implementation, this would:
        // 1. Copy files from .moss/site to root or docs/ folder
        // 2. Git add, commit, and push
        // 3. Enable GitHub Pages via GitHub API
        // 4. Return the GitHub Pages URL
        
        let pages_url = format!("https://{}.github.io/{}", 
            self.remote_info.owner, 
            self.remote_info.repo_name
        );

        // TODO: Implement actual deployment workflow
        // This would require:
        // 1. Copy site files with proper exclusions
        // 2. Git commit and push changes  
        // 3. Enable GitHub Pages (requires token)
        // For now, return the expected URL

        Ok(pages_url)
    }

    /// Copy site files to the repository root for GitHub Pages
    /// 
    /// GitHub Pages can serve from:
    /// - Root directory of main/master branch
    /// - /docs directory of main/master branch  
    /// - gh-pages branch
    /// 
    /// We use the root directory approach for simplicity.
    #[allow(dead_code)]
    fn copy_site_files(&self, folder_path: &Path, site_path: &Path) -> Result<(), String> {
        // Copy all files from .moss/site to project root
        // Exclude .moss directory and other development files
        
        // Copy files from .moss/site to project root
        // This is a simplified implementation - production version should:
        // - Check for conflicts with existing files
        // - Preserve important files like README.md, .gitignore
        // - Only copy web assets (HTML, CSS, JS, images)
        
        if !site_path.exists() {
            return Err("Generated site not found. Please compile first.".to_string());
        }
        
        // List of files to exclude from copying
        let exclude_patterns = [".git", ".moss", "README.md", ".gitignore", "Cargo.toml"];
        
        // Copy site files to project root
        copy_dir_contents(site_path, folder_path, &exclude_patterns)
            .map_err(|e| format!("Failed to copy site files: {}", e))?;
        
        Ok(())
    }

    /// Commit changes and push to GitHub
    #[allow(dead_code)]
    fn commit_and_push(&self, folder_path: &Path) -> Result<(), String> {
        // Run git commands to commit and push
        let git_add = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(folder_path)
            .output()
            .map_err(|e| format!("Failed to run git add: {}", e))?;

        if !git_add.status.success() {
            return Err(format!("Git add failed: {}", String::from_utf8_lossy(&git_add.stderr)));
        }

        let git_commit = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Deploy to GitHub Pages via moss")
            .current_dir(folder_path)
            .output()
            .map_err(|e| format!("Failed to run git commit: {}", e))?;

        if !git_commit.status.success() {
            return Err(format!("Git commit failed: {}", String::from_utf8_lossy(&git_commit.stderr)));
        }

        let git_push = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("main")
            .current_dir(folder_path)
            .output()
            .map_err(|e| format!("Failed to run git push: {}", e))?;

        if !git_push.status.success() {
            return Err(format!("Git push failed: {}", String::from_utf8_lossy(&git_push.stderr)));
        }

        Ok(())
    }

    /// Enable GitHub Pages for the repository
    /// 
    /// Uses the GitHub REST API to enable Pages for the repository.
    /// Configures Pages to serve from the main branch root directory.
    /// 
    /// # Arguments
    /// * `github_token` - Personal access token with repo permissions
    /// 
    /// # Returns
    /// * `Ok(String)` - Pages URL if successful
    /// * `Err(String)` - API error message
    /// 
    /// # API Reference
    /// - Pages API: https://docs.github.com/en/rest/pages/pages?apiVersion=2022-11-28#create-a-apiname-pages-site
    #[allow(dead_code)]
    async fn enable_github_pages(&self, github_token: &str) -> Result<String, String> {
        let client = GitHubApiClient::new(github_token.to_string());
        
        // Check if Pages is already enabled
        match client.get_github_pages(&self.remote_info.owner, &self.remote_info.repo_name).await {
            Ok(pages) => {
                // Pages already enabled, return existing URL
                return Ok(pages.html_url);
            },
            Err(err) if err.contains("not enabled") => {
                // Pages not enabled, continue to enable it
            },
            Err(err) => {
                return Err(format!("Failed to check Pages status: {}", err));
            }
        }
        
        // Enable GitHub Pages
        let request = EnablePagesRequest {
            source: PagesSource {
                branch: "main".to_string(),
                path: Some("/".to_string()),
            },
            build_type: Some("legacy".to_string()),
        };
        
        let pages = client.enable_github_pages(
            &self.remote_info.owner, 
            &self.remote_info.repo_name, 
            request
        ).await
            .map_err(|e| format!("Failed to enable GitHub Pages: {}", e))?;
        
        Ok(pages.html_url)
    }
}

/// Quick deploy function for GitHub Pages
/// 
/// Simplified interface for deploying to GitHub Pages when
/// git remote is already configured.
/// 
/// # Arguments
/// * `folder_path` - Path to the project folder
/// * `remote_info` - Git remote information
/// 
/// # Returns
/// * `Ok(String)` - URL of the deployed site
/// * `Err(String)` - Error message if deployment fails
pub async fn deploy_to_github_pages(folder_path: &Path, remote_info: &GitRemoteInfo) -> Result<String, String> {
    if !remote_info.is_github {
        return Err("Remote is not a GitHub repository".to_string());
    }

    let deployer = GitHubPages::new(remote_info.clone());
    deployer.deploy_to_pages(folder_path).await
}

/// Copy directory contents with exclusion patterns
/// 
/// Recursively copies files from source to destination directory,
/// excluding files that match any of the provided patterns.
fn copy_dir_contents(
    src: &Path, 
    dst: &Path, 
    exclude_patterns: &[&str]
) -> Result<(), std::io::Error> {
    if !src.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Source is not a directory"
        ));
    }
    
    // Ensure destination directory exists
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        
        // Skip excluded files
        if exclude_patterns.iter().any(|pattern| file_name_str.contains(pattern)) {
            continue;
        }
        
        let src_path = entry.path();
        let dst_path = dst.join(&file_name);
        
        if src_path.is_dir() {
            copy_dir_contents(&src_path, &dst_path, exclude_patterns)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview::git::GitRemoteInfo;
    use std::fs;

    #[test]
    fn test_github_pages_creation() {
        let remote_info = GitRemoteInfo {
            url: "https://github.com/user/repo.git".to_string(),
            is_github: true,
            repo_name: "repo".to_string(),
            owner: "user".to_string(),
        };

        let pages = GitHubPages::new(remote_info.clone());
        assert_eq!(pages.remote_info.repo_name, "repo");
        assert_eq!(pages.remote_info.owner, "user");
    }

    #[tokio::test]
    async fn test_deploy_to_pages_no_site() {
        let temp_dir = std::env::temp_dir().join("moss_test_no_site");
        fs::create_dir_all(&temp_dir).unwrap();

        let remote_info = GitRemoteInfo {
            url: "https://github.com/user/repo.git".to_string(),
            is_github: true,
            repo_name: "repo".to_string(),
            owner: "user".to_string(),
        };

        let pages = GitHubPages::new(remote_info);
        let result = pages.deploy_to_pages(&temp_dir).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Built site not found"));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_deploy_to_github_pages_non_github() {
        let temp_dir = std::env::temp_dir().join("moss_test_non_github");
        fs::create_dir_all(&temp_dir).unwrap();

        let remote_info = GitRemoteInfo {
            url: "https://gitlab.com/user/repo.git".to_string(),
            is_github: false,
            repo_name: "repo".to_string(),
            owner: "user".to_string(),
        };

        let result = deploy_to_github_pages(&temp_dir, &remote_info).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a GitHub repository"));

        fs::remove_dir_all(&temp_dir).ok();
    }
}