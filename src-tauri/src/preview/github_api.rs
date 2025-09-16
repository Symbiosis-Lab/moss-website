//! GitHub API client for repository and Pages management
//!
//! Provides authenticated HTTP client for GitHub's REST API v3.
//! Supports repository creation and GitHub Pages configuration.
//!
//! API Reference: https://docs.github.com/en/rest

use serde::{Deserialize, Serialize};

/// GitHub API client with authentication support
pub struct GitHubApiClient {
    client: reqwest::Client,
    base_url: String,
    token: String,
}

/// Repository creation request payload
/// 
/// API Reference: https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
#[derive(Debug, Serialize)]
pub struct CreateRepoRequest {
    /// Repository name (required)
    pub name: String,
    /// Repository description
    pub description: Option<String>,
    /// Whether repository is private (default: false for public)
    pub private: bool,
    /// Enable issues (default: true)
    pub has_issues: bool,
    /// Enable projects (default: true)  
    pub has_projects: bool,
    /// Enable wiki (default: true)
    pub has_wiki: bool,
    /// Auto-initialize with README (default: false)
    pub auto_init: bool,
}

/// Repository creation response
/// 
/// Contains essential repository information returned by GitHub API
#[derive(Debug, Deserialize)]
pub struct CreateRepoResponse {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
    pub private: bool,
}

/// GitHub Pages site configuration request
/// 
/// API Reference: https://docs.github.com/en/rest/pages/pages?apiVersion=2022-11-28#create-a-apiname-pages-site
#[derive(Debug, Serialize)]
pub struct EnablePagesRequest {
    /// Source configuration for Pages site
    pub source: PagesSource,
    /// Build type: "legacy" or "workflow" (default: "legacy")
    pub build_type: Option<String>,
}

/// Pages source configuration
#[derive(Debug, Serialize)]
pub struct PagesSource {
    /// Branch to serve from (e.g., "main", "gh-pages")
    pub branch: String,
    /// Path within branch (default: "/")
    pub path: Option<String>,
}

/// GitHub Pages site information response
#[derive(Debug, Deserialize)]
pub struct PagesResponse {
    pub url: String,
    pub status: String,
    pub source: PagesSourceResponse,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct PagesSourceResponse {
    pub branch: String,
    pub path: String,
}

/// GitHub API error response
#[derive(Debug, Deserialize)]
pub struct GitHubApiError {
    pub message: String,
    pub documentation_url: Option<String>,
}

impl GitHubApiClient {
    /// Creates new GitHub API client with personal access token
    /// 
    /// # Arguments
    /// * `token` - Personal access token with `repo` scope
    /// 
    /// # Token Requirements
    /// - `repo` scope for creating private repositories  
    /// - `public_repo` scope sufficient for public repositories
    /// - Repository admin permissions required for Pages API
    ///
    /// # References
    /// - Token scopes: https://docs.github.com/en/developers/apps/building-oauth-apps/scopes-for-oauth-apps
    /// - Personal access tokens: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token
    pub fn new(token: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("moss/0.1.0") // GitHub requires User-Agent header
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://api.github.com".to_string(),
            token,
        }
    }

    /// Creates a new repository for the authenticated user
    /// 
    /// # Arguments
    /// * `request` - Repository configuration
    /// 
    /// # Returns
    /// * `Ok(CreateRepoResponse)` - Repository successfully created
    /// * `Err(String)` - API error or network failure
    /// 
    /// # API Endpoint
    /// `POST /user/repos`
    /// 
    /// # References
    /// - Repository creation API: https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
    pub async fn create_repository(&self, request: CreateRepoRequest) -> Result<CreateRepoResponse, String> {
        let url = format!("{}/user/repos", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if response.status().is_success() {
            let repo: CreateRepoResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(repo)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            // Try to parse GitHub API error format
            if let Ok(api_error) = serde_json::from_str::<GitHubApiError>(&error_text) {
                Err(format!("GitHub API error ({}): {}", status, api_error.message))
            } else {
                Err(format!("GitHub API error ({}): {}", status, error_text))
            }
        }
    }

    /// Enables GitHub Pages for a repository
    /// 
    /// # Arguments
    /// * `owner` - Repository owner username
    /// * `repo` - Repository name
    /// * `request` - Pages configuration
    /// 
    /// # Returns
    /// * `Ok(PagesResponse)` - Pages successfully enabled
    /// * `Err(String)` - API error or already enabled
    /// 
    /// # API Endpoint
    /// `POST /repos/{owner}/{repo}/pages`
    /// 
    /// # Required Permissions
    /// - Repository admin, maintainer, or "manage GitHub Pages settings" permission
    /// - `repo` scope for private repositories
    /// 
    /// # References
    /// - Pages API: https://docs.github.com/en/rest/pages/pages?apiVersion=2022-11-28#create-a-apiname-pages-site
    pub async fn enable_github_pages(
        &self, 
        owner: &str, 
        repo: &str, 
        request: EnablePagesRequest
    ) -> Result<PagesResponse, String> {
        let url = format!("{}/repos/{}/{}/pages", self.base_url, owner, repo);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if response.status().is_success() {
            let pages: PagesResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(pages)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            if let Ok(api_error) = serde_json::from_str::<GitHubApiError>(&error_text) {
                Err(format!("GitHub Pages API error ({}): {}", status, api_error.message))
            } else {
                Err(format!("GitHub Pages API error ({}): {}", status, error_text))
            }
        }
    }

    /// Gets GitHub Pages site information
    /// 
    /// # API Endpoint
    /// `GET /repos/{owner}/{repo}/pages`
    /// 
    /// # References
    /// - Pages API: https://docs.github.com/en/rest/pages/pages?apiVersion=2022-11-28#get-a-apiname-pages-site
    pub async fn get_github_pages(&self, owner: &str, repo: &str) -> Result<PagesResponse, String> {
        let url = format!("{}/repos/{}/{}/pages", self.base_url, owner, repo);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if response.status().is_success() {
            let pages: PagesResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(pages)
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            Err("GitHub Pages not enabled for this repository".to_string())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("GitHub Pages API error ({}): {}", status, error_text))
        }
    }
}

impl Default for CreateRepoRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            private: false,
            has_issues: true,
            has_projects: true, 
            has_wiki: true,
            auto_init: false,
        }
    }
}

impl Default for EnablePagesRequest {
    fn default() -> Self {
        Self {
            source: PagesSource {
                branch: "main".to_string(),
                path: Some("/".to_string()),
            },
            build_type: Some("legacy".to_string()),
        }
    }
}

// Tests removed: These tests verified implementation details (default values, 
// struct initialization) rather than user-observable behavior.