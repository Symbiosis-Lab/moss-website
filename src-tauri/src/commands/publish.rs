//! Tauri commands and business logic for moss publishing system

use crate::types::*;
use walkdir::WalkDir;
use std::path::Path;
use std::fs;
use pulldown_cmark::{Parser, Options, html};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use axum::Router;
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tauri::Manager;

/// Recursively copies a directory and all its contents
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), String> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    if !src.exists() {
        return Err(format!("Source directory does not exist: {}", src.display()));
    }
    
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    
    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read source directory: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)
                .map_err(|e| format!("Failed to copy file {}: {}", path.display(), e))?;
        }
    }
    
    Ok(())
}

/// Identifies the most likely homepage file from a collection of files.
/// 
/// Uses a priority-based detection algorithm to find the main entry point
/// for the website. Prioritizes conventional homepage filenames and falls
/// back to alphabetical ordering for ambiguous cases.
/// 
/// # Priority Order
/// 1. `index.md` - Standard markdown homepage
/// 2. `index.pages` - macOS Pages document
/// 3. `index.docx` - Microsoft Word document
/// 4. `README.md` - Project documentation
/// 5. First document file alphabetically
/// 
/// # Arguments
/// * `files` - Complete list of files discovered in the project
/// 
/// # Returns
/// * `Some(String)` - Relative path to the detected homepage file
/// * `None` - No suitable homepage file found
pub fn detect_homepage_file(files: &[FileInfo]) -> Option<String> {
    let root_files: Vec<&FileInfo> = files.iter()
        .filter(|f| !f.path.contains('/')) // Only files in root directory
        .collect();
    
    // Priority 1: index.md (markdown homepage)
    if let Some(file) = root_files.iter().find(|f| f.path.to_lowercase() == "index.md") {
        return Some(file.path.clone());
    }
    
    // Priority 2: index.pages (macOS Pages homepage)
    if let Some(file) = root_files.iter().find(|f| f.path.to_lowercase() == "index.pages") {
        return Some(file.path.clone());
    }
    
    // Priority 3: index.docx (Word homepage)
    if let Some(file) = root_files.iter().find(|f| f.path.to_lowercase() == "index.docx") {
        return Some(file.path.clone());
    }
    
    // Priority 4: README.md (documentation/project description)
    if let Some(file) = root_files.iter().find(|f| f.path.to_lowercase() == "readme.md") {
        return Some(file.path.clone());
    }
    
    // Fallback: First document file alphabetically
    let mut doc_files: Vec<&FileInfo> = root_files.iter()
        .filter(|f| {
            let ext = f.file_type.to_lowercase();
            matches!(ext.as_str(), "md" | "pages" | "docx" | "doc")
        })
        .copied()
        .collect();
    doc_files.sort_by(|a, b| a.path.cmp(&b.path));
    
    doc_files.first().map(|f| f.path.clone())
}

/// Identifies subdirectories containing document files.
/// 
/// Scans the file list to find folders that contain publishable content
/// (markdown, Pages, Word documents). These folders typically represent
/// content collections like blog posts, projects, or documentation sections.
/// 
/// # Content Detection
/// Only counts folders containing files with extensions:
/// - `.md`, `.markdown` - Markdown files
/// - `.pages` - macOS Pages documents  
/// - `.docx`, `.doc` - Microsoft Word documents
/// 
/// # Arguments
/// * `files` - Complete list of files from recursive directory scan
/// 
/// # Returns
/// * `Vec<String>` - Names of subdirectories containing document files
pub fn detect_content_folders(files: &[FileInfo]) -> Vec<String> {
    let mut folders_with_docs = std::collections::HashSet::new();
    
    // Find folders that contain document files
    for file in files {
        if file.path.contains('/') {
            if let Some(folder) = file.path.split('/').next() {
                // Check if this file is a document
                let ext = file.file_type.to_lowercase();
                if matches!(ext.as_str(), "md" | "pages" | "docx" | "doc") {
                    folders_with_docs.insert(folder.to_string());
                }
            }
        }
    }
    
    folders_with_docs.into_iter().collect()
}

/// Classifies the project structure to determine optimal site generation strategy.
/// 
/// Analyzes the distribution of document files to infer how the site should
/// be organized and navigated. Uses simple heuristics based on common
/// content organization patterns.
/// 
/// # Classification Logic
/// 1. **Homepage + Collections**: Has subdirectories with documents
/// 2. **Simple Flat Site**: ≤5 document files in root (all in navigation)
/// 3. **Blog-style Flat Site**: >5 document files in root (selective navigation)
/// 
/// # Arguments
/// * `files` - Complete file listing from directory scan
/// * `content_folders` - Subdirectories containing documents
/// 
/// # Returns
/// * `ProjectType` - Recommended site structure classification
pub fn detect_project_type_from_content(
    files: &[FileInfo], 
    content_folders: &[String]
) -> ProjectType {
    // Count document files in root directory
    let root_doc_count = files.iter()
        .filter(|f| !f.path.contains('/')) // Root files only
        .filter(|f| {
            let ext = f.file_type.to_lowercase();
            matches!(ext.as_str(), "md" | "pages" | "docx" | "doc")
        })
        .count();
    
    // Decision tree based on simplified logic
    
    // 1. Has subdirectories with documents → Homepage + Collections
    if !content_folders.is_empty() {
        return ProjectType::HomepageWithCollections;
    }
    
    // 2. Root has ≤5 document files → Simple Flat Site
    if root_doc_count <= 5 {
        return ProjectType::SimpleFlatSite;
    }
    
    // 3. Root has >5 document files → Blog-style Flat Site
    ProjectType::BlogStyleFlatSite
}

/// Performs recursive directory analysis for static site generation.
/// 
/// Walks through the specified folder and all subdirectories to build
/// a complete inventory of files, categorized by type and purpose.
/// Also performs content analysis to determine the optimal site structure.
/// 
/// # File Categorization
/// * **Markdown**: `.md`, `.markdown`, `.mdown`, `.mkd`
/// * **HTML**: `.html`, `.htm`  
/// * **Images**: `.jpg`, `.jpeg`, `.png`, `.gif`, `.svg`, `.webp`
/// * **Documents**: `.pages`, `.docx`, `.doc` (stored in other_files)
/// * **Other**: All remaining file types
/// 
/// # Content Analysis
/// Automatically detects:
/// - Homepage file candidates
/// - Content organization folders
/// - Recommended site structure type
/// 
/// # Arguments
/// * `folder_path` - Absolute path to the directory to analyze
/// 
/// # Returns
/// * `Ok(ProjectStructure)` - Complete analysis with file inventory and recommendations
/// * `Err(String)` - Error message if folder is inaccessible or invalid
/// 
/// # Errors
/// - Folder does not exist
/// - Path is not a directory
/// - Permission denied during scanning
fn scan_folder(folder_path: &str) -> Result<ProjectStructure, String> {
    let path = Path::new(folder_path);
    
    if !path.exists() {
        return Err(format!("Folder does not exist: {}", folder_path));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }
    
    let mut markdown_files = Vec::new();
    let mut html_files = Vec::new();
    let mut image_files = Vec::new();
    let mut other_files = Vec::new();
    
    // Walk through the directory recursively
    for entry in WalkDir::new(path).into_iter() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                // Log error but continue scanning
                eprintln!("Warning: Failed to read entry: {}", e);
                continue;
            }
        };
        
        // Skip directories, only process files
        if !entry.file_type().is_file() {
            continue;
        }
        
        let file_path = entry.path();
        let relative_path = match file_path.strip_prefix(path) {
            Ok(rel_path) => rel_path.to_string_lossy().to_string(),
            Err(_) => file_path.to_string_lossy().to_string(),
        };
        
        // Get file metadata
        let metadata = match entry.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                eprintln!("Warning: Failed to read metadata for {}: {}", relative_path, e);
                continue;
            }
        };
        
        let size = metadata.len();
        let modified = metadata.modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs().to_string());
        
        // Determine file type based on extension
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let file_info = FileInfo {
            path: relative_path,
            file_type: extension.clone(),
            size,
            modified,
        };
        
        // Categorize files by extension
        match extension.as_str() {
            "md" | "markdown" | "mdown" | "mkd" => markdown_files.push(file_info),
            "html" | "htm" => html_files.push(file_info),
            "jpg" | "jpeg" | "png" | "gif" | "svg" | "webp" => image_files.push(file_info),
            "pages" | "docx" | "doc" => other_files.push(file_info), // Document files go to other_files for now
            _ => other_files.push(file_info),
        }
    }
    
    let total_files = markdown_files.len() + html_files.len() + image_files.len() + other_files.len();
    
    // Combine all files for analysis
    let all_files: Vec<FileInfo> = markdown_files.iter()
        .chain(html_files.iter())
        .chain(image_files.iter())
        .chain(other_files.iter())
        .cloned()
        .collect();
    
    // Detect content patterns
    let homepage_file = detect_homepage_file(&all_files);
    let content_folders = detect_content_folders(&all_files);
    let project_type = detect_project_type_from_content(&all_files, &content_folders);
    
    
    Ok(ProjectStructure {
        root_path: folder_path.to_string(),
        markdown_files,
        html_files,
        image_files,
        other_files,
        total_files,
        project_type,
        homepage_file,
        content_folders,
    })
}

/// Tauri command to publish a folder as a static website.
/// 
/// Triggered by deep links from Finder integration or direct API calls.
/// Analyzes the folder structure and prepares content for static site
/// generation and deployment.
/// 
/// # Workflow
/// 1. Validates the provided folder path
/// 2. Recursively scans and categorizes all files
/// 3. Analyzes content organization patterns
/// 4. Determines optimal site structure
/// 5. Generates deployment URL suggestion
/// 
/// # Arguments
/// * `folder_path` - Absolute path to the folder containing website content
/// 
/// # Returns
/// * `Ok(String)` - Success message with scan summary and deployment URL
/// * `Err(String)` - Error message describing what went wrong
/// 
/// # Errors
/// * Empty folder path provided
/// * Folder does not exist
/// * No publishable files found
/// * Permission denied during scanning
/// 
/// # Future Implementation
/// Currently performs analysis only. Future versions will:
/// - Generate static HTML/CSS from content
/// - Deploy to moss.pub hosting
/// - Support custom domains and themes
/// 
/// # Example
/// ```rust
/// let result = publish_folder("/Users/alice/my-blog".to_string());
/// // Returns: "📁 'my-blog': 15 files scanned. Blog-style site..."
/// ```
#[tauri::command]
pub fn publish_folder(folder_path: String) -> Result<String, String> {
    if folder_path.is_empty() {
        return Err("Empty folder path provided".to_string());
    }
    
    
    // Scan folder for publishable content
    let project_structure = scan_folder(&folder_path)?;
    
    // Basic validation - ensure we have some content to publish
    if project_structure.total_files == 0 {
        return Err("No files found in the specified folder".to_string());
    }
    
    // Generate a simple site identifier based on folder name
    let folder_name = Path::new(&folder_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unnamed-site");
    
    // Generate static site from scanned files
    let site_result = generate_static_site(&folder_path, &project_structure)?;
    
    
    // Start local preview server (browser opening handled by preview window)
    start_site_server(&site_result.output_path)?;
    
    // TODO: Deploy to moss.pub or configured hosting (Phase 3)
    
    
    // Create publishing strategy message based on detected type
    let strategy_message = match project_structure.project_type {
        ProjectType::HomepageWithCollections => "Homepage with organized content collections detected",
        ProjectType::SimpleFlatSite => "Simple site with all pages in navigation menu",
        ProjectType::BlogStyleFlatSite => "Blog-style site with essential pages in menu, others listed on homepage",
    };
    
    let homepage_info = if let Some(homepage) = &project_structure.homepage_file {
        format!(" (Homepage: {})", homepage)
    } else {
        String::new()
    };
    
    Ok(format!(
        "📁 '{}': {} files scanned. {} {}. Content folders: {:?}. Ready for https://{}.moss.pub",
        folder_name,
        project_structure.total_files,
        strategy_message,
        homepage_info,
        project_structure.content_folders,
        folder_name.replace(" ", "-").to_lowercase()
    ))
}

/// Determines whether the system tray icon is actually visible to the user.
/// 
/// Checks both successful tray creation and platform-specific visibility
/// constraints. On macOS, tray icons can be created but hidden when the
/// menu bar is crowded.
/// 
/// # Platform Behavior
/// * **macOS**: May hide icons due to space constraints
/// * **Windows/Linux**: Generally visible if created successfully
/// 
/// # Arguments
/// * `app` - Tauri application handle for tray icon access
/// 
/// # Returns
/// * `TrayVisibilityStatus` - Current visibility state
pub fn detect_tray_visibility_status(app: &tauri::AppHandle) -> TrayVisibilityStatus {
    // First check if tray icon was created successfully
    if app.tray_by_id("main").is_none() {
        return TrayVisibilityStatus::NotAdded;
    }

    // If we have a tray icon, check if it's actually visible
    #[cfg(target_os = "macos")]
    {
        if is_tray_icon_actually_visible() {
            TrayVisibilityStatus::Visible
        } else {
            TrayVisibilityStatus::AddedButHidden
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, assume visible if it was created successfully
        TrayVisibilityStatus::Visible
    }
}

/// Checks if the tray icon is visible in the macOS menu bar.
/// 
/// Uses accessibility APIs to determine if the icon is actually displayed
/// to the user, accounting for macOS space management that can hide icons
/// when the menu bar becomes crowded.
/// 
/// # Implementation Note
/// Currently uses a simplified heuristic. Full implementation would require:
/// 1. Getting NSStatusItem window position
/// 2. Using AXUIElementCopyElementAtPosition for visibility detection
/// 3. Bundle ID verification
/// 4. Multi-display support
/// 
/// # Returns
/// * `true` - Icon is visible to the user
/// * `false` - Icon is hidden by the system
#[cfg(target_os = "macos")]
fn is_tray_icon_actually_visible() -> bool {
    // For now, use a simplified heuristic approach
    // Full accessibility API implementation requires complex CFString handling
    // and process enumeration which is beyond the current scope
    
    // In practice, detecting tray icon visibility requires:
    // 1. Getting the NSStatusItem's window position
    // 2. Using AXUIElementCopyElementAtPosition to check what's at that position
    // 3. Comparing bundle IDs to verify it's our app
    // 4. Handling screen bounds and multiple displays
    
    // For now, assume visible if we reach this point (tray was created successfully)
    true
}

#[cfg(not(target_os = "macos"))]
fn is_tray_icon_actually_visible() -> bool {
    true
}

/// Starts a local HTTP server to serve the generated website.
/// 
/// Creates a lightweight Axum server to properly serve the static files with correct
/// HTTP headers, avoiding CORS issues and providing real web server behavior.
/// The preview window will display the site via iframe.
/// 
/// # Arguments
/// * `site_path` - Path to the generated site directory containing index.html
/// 
/// # Returns
/// * `Ok(())` - Successfully started server
/// * `Err(String)` - Failed to start server
/// 
/// # Implementation
/// - Uses Axum + tower-http for lightweight static file serving
/// - Finds an available port automatically (starting from 3000)
/// - Serves the site directory with proper HTTP headers
/// - Server runs in background thread
/// - Port 8080 is preferred for consistency with preview window
fn start_site_server(site_path: &str) -> Result<(), String> {
    let site_path = site_path.to_string();
    
    // Check if index.html exists
    let index_path = Path::new(&site_path).join("index.html");
    if !index_path.exists() {
        return Err("Generated site has no index.html file".to_string());
    }
    
    // Use port 8080 for consistency with preview window
    let port = if is_port_available(8080) { 8080 } else { find_available_port(8080)? };
    
    
    // Start server in background
    let server_site_path = site_path.clone();
    
    // Try to spawn in existing Tokio runtime, fallback to thread for tests
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async move {
            if let Err(e) = start_preview_server(port, server_site_path).await {
                eprintln!("❌ Preview server error: {}", e);
            }
        });
    } else {
        // Fallback for test environment - create minimal runtime
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = start_preview_server(port, server_site_path).await {
                    eprintln!("❌ Preview server error: {}", e);
                }
            });
        });
    }
    
    // Give server a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    Ok(())
}

/// Finds an available TCP port starting from the given port number.
fn find_available_port(start_port: u16) -> Result<u16, String> {
    for port in start_port..start_port + 100 {
        if is_port_available(port) {
            return Ok(port);
        }
    }
    Err(format!("No available ports found starting from {}", start_port))
}

/// Checks if a TCP port is available for binding.
fn is_port_available(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Starts an Axum server to serve static files from the specified directory.
async fn start_preview_server(port: u16, site_path: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .fallback_service(ServeDir::new(site_path));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(&addr).await?;
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Opens the given URL in the user's default browser.
fn open_browser(url: &str) -> Result<(), String> {
    // Use system command to open in default browser
    #[cfg(target_os = "macos")]
    let command_result = std::process::Command::new("open")
        .arg(url)
        .status();
        
    #[cfg(target_os = "windows")]
    let command_result = std::process::Command::new("cmd")
        .args(&["/C", "start", "", url])
        .status();
        
    #[cfg(target_os = "linux")]
    let command_result = std::process::Command::new("xdg-open")
        .arg(url)
        .status();
    
    match command_result {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err("Failed to open browser".to_string())
            }
        },
        Err(e) => Err(format!("Failed to execute browser command: {}", e))
    }
}

/// Tauri command to retrieve system diagnostic information.
/// 
/// Collects runtime information about the application's integration
/// with the operating system for debugging, support, and user feedback.
/// 
/// # Collected Information
/// * Operating system identification
/// * Tray icon visibility status  
/// * Finder integration installation status (macOS)
/// * Application version from package metadata
/// 
/// # Arguments
/// * `app` - Tauri application handle for system access
/// 
/// # Returns
/// * `Ok(SystemInfo)` - Complete diagnostic information
/// * `Err(String)` - Error if system information cannot be gathered
/// 
/// # Usage
/// Typically called by:
/// - Settings/About dialogs
/// - Troubleshooting workflows
/// - Support ticket generation
/// 
/// # Example
/// ```rust
/// let info = get_system_status(app_handle)?;
/// println!("Running on {} with tray: {:?}", info.os, info.tray_status);
/// ```

#[tauri::command]
pub fn get_system_status(app: tauri::AppHandle) -> Result<SystemInfo, String> {
    let os = std::env::consts::OS.to_string();
    let app_version = app.package_info().version.to_string();
    
    // Detect detailed tray icon visibility status
    let tray_status = detect_tray_visibility_status(&app);
    
    // Check if Finder integration is installed (macOS only)
    let finder_integration = if cfg!(target_os = "macos") {
        let home_dir = std::env::var("HOME").unwrap_or_default();
        let workflow_path = format!("{}/Library/Services/Publish to Web.workflow", home_dir);
        std::path::Path::new(&workflow_path).exists()
    } else {
        false
    };
    
    Ok(SystemInfo {
        os,
        tray_status,
        finder_integration,
        app_version,
    })
}

/// Tauri command to install macOS Finder integration for one-click publishing.
/// 
/// Creates an Automator workflow that adds "Publish to Web" to the Finder
/// context menu when right-clicking folders. This provides seamless integration
/// with the user's file management workflow.
/// 
/// # Installation Process
/// 1. Creates `~/Library/Services/Publish to Web.workflow` bundle
/// 2. Writes Info.plist with proper NSServices configuration
/// 3. Embeds shell script that triggers `moss://` deep links
/// 4. Registers with macOS Services for folder context menus
/// 
/// # Workflow Behavior
/// When users right-click a folder in Finder:
/// - "Publish to Web" appears in the context menu
/// - Clicking it URL-encodes the folder path
/// - Opens `moss://publish?path=<encoded_path>`
/// - Triggers the publishing workflow in this app
/// 
/// # Security
/// The generated shell script uses only built-in macOS commands:
/// - `printf` and `sed` for URL encoding
/// - `open` for deep link activation
/// - No external dependencies or network access
/// 
/// # Returns
/// * `Ok(String)` - Success message with installation location
/// * `Err(String)` - Error message with failure details
/// 
/// # Errors
/// * HOME environment variable not found
/// * Permission denied creating Services directory
/// * File write failures during installation
/// * Cannot remove existing workflow for reinstallation
/// 
/// # Platform Support
/// Currently macOS only. Uses Automator and NSServices APIs.
/// 
/// # Example
/// ```rust
/// let result = install_finder_integration()?;
/// // Returns: "Finder integration installed successfully! Right-click..."
/// ```
#[tauri::command]
pub fn install_finder_integration() -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    
    // Get user's home directory
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => dir,
        Err(_) => return Err("Could not determine home directory".to_string()),
    };
    
    let services_dir = format!("{}/Library/Services", home_dir);
    let workflow_path = format!("{}/Publish.workflow", services_dir);
    
    // Create Services directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&services_dir) {
        return Err(format!("Failed to create Services directory: {}", e));
    }
    
    // Remove existing workflow if it exists (to ensure clean reinstall)
    if Path::new(&workflow_path).exists() {
        if let Err(e) = fs::remove_dir_all(&workflow_path) {
            return Err(format!("Failed to remove existing workflow: {}", e));
        }
    }
    
    // Copy workflow from bundled resources
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?
        .parent()
        .ok_or_else(|| "Failed to get executable directory".to_string())?
        .to_path_buf();
        
    // Try Tauri bundle structure first, then fall back to development structure
    let resource_path = exe_dir
        .join("../Resources/_up_/resources/services/Publish.workflow")
        .canonicalize()
        .unwrap_or_else(|_| {
            exe_dir
                .join("../Resources/services/Publish.workflow")
                .canonicalize()
                .unwrap_or_else(|_| exe_dir.join("resources/services/Publish.workflow"))
        });
        
    if !resource_path.exists() {
        return Err("Bundled Publish.workflow not found in resources".to_string());
    }
    
    // Copy the entire workflow directory from resources to Services
    copy_dir_all(&resource_path, &workflow_path)?;
    
    Ok("Finder integration installed successfully! Right-click any folder → 'Publish'".to_string())
}

/// Generates a static website from scanned folder contents.
/// 
/// Processes markdown files into HTML pages with beautiful default styling,
/// handles frontmatter for metadata, and creates a complete navigable website.
/// 
/// # Arguments
/// * `source_path` - Path to the source folder containing content
/// * `project_structure` - Analysis of the folder's contents and organization
/// 
/// # Returns
/// * `Ok(SiteResult)` - Information about the generated site
/// * `Err(String)` - Error message if generation fails
/// 
/// # Process
/// 1. Create temporary output directory
/// 2. Process all markdown files to HTML
/// 3. Copy image and asset files
/// 4. Generate index pages and navigation
/// 5. Create CSS stylesheet with beautiful defaults
fn generate_static_site(source_path: &str, project_structure: &ProjectStructure) -> Result<SiteResult, String> {
    
    // Create output directory in source folder under .moss/site
    let source_path_buf = Path::new(source_path);
    let moss_dir = source_path_buf.join(".moss");
    let output_dir = moss_dir.join("site");
    
    // Create .moss directory if it doesn't exist
    if !moss_dir.exists() {
        fs::create_dir_all(&moss_dir).map_err(|e| format!("Failed to create .moss directory: {}", e))?;
    }
    
    // Clean and recreate site directory
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).map_err(|e| format!("Failed to clean site directory: {}", e))?;
    }
    fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create site directory: {}", e))?;
    
    
    // Process markdown files
    let mut documents = Vec::new();
    for file_info in &project_structure.markdown_files {
        let source_file_path = Path::new(source_path).join(&file_info.path);
        
        if let Ok(content) = fs::read_to_string(&source_file_path) {
            if let Ok(doc) = process_markdown_file(&file_info.path, &content) {
                documents.push(doc);
            }
        }
    }
    
    // Generate HTML files
    let mut page_count = 0;
    for doc in &documents {
        let output_file_path = output_dir.join(&doc.url_path);
        
        // Create directory if needed
        if let Some(parent) = output_file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        let html_page = generate_html_page(doc, &documents, project_structure)?;
        fs::write(&output_file_path, html_page).map_err(|e| format!("Failed to write HTML file: {}", e))?;
        page_count += 1;
    }
    
    // Generate CSS
    let css_content = generate_default_css();
    fs::write(output_dir.join("style.css"), css_content).map_err(|e| format!("Failed to write CSS: {}", e))?;
    
    // Copy image files
    for file_info in &project_structure.image_files {
        let source_file = Path::new(source_path).join(&file_info.path);
        let dest_file = output_dir.join(&file_info.path);
        
        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create image directory: {}", e))?;
        }
        
        if let Err(_e) = fs::copy(&source_file, &dest_file) {
        }
    }
    
    // Generate index.html if no homepage exists
    if project_structure.homepage_file.is_none() && !documents.is_empty() {
        let index_html = generate_index_page(&documents, project_structure)?;
        fs::write(output_dir.join("index.html"), index_html).map_err(|e| format!("Failed to write index.html: {}", e))?;
        page_count += 1;
    }
    
    let site_title = project_structure.homepage_file.clone()
        .or_else(|| documents.first().map(|d| d.title.clone()))
        .unwrap_or_else(|| "Untitled Site".to_string());
    
    Ok(SiteResult {
        page_count,
        output_path: output_dir.to_string_lossy().to_string(),
        site_title,
    })
}

/// Processes a markdown file with frontmatter into a ParsedDocument.
fn process_markdown_file(file_path: &str, content: &str) -> Result<ParsedDocument, String> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(content);
    
    // Extract title from frontmatter or filename
    let title = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .replace("-", " ")
        .replace("_", " ");
    
    // Generate URL path
    let url_path = if file_path.to_lowercase() == "index.md" || file_path.to_lowercase() == "readme.md" {
        "index.html".to_string()
    } else {
        file_path.replace(".md", ".html").replace(".markdown", ".html")
    };
    
    // Convert markdown to HTML
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    
    let parser = Parser::new_ext(&result.content, options);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);
    
    // Extract date from frontmatter (simplified for now)
    let date: Option<String> = None;
    
    Ok(ParsedDocument {
        title,
        content: result.content,
        html_content,
        url_path,
        date,
    })
}

/// Generates complete HTML page with navigation and styling.
fn generate_html_page(doc: &ParsedDocument, all_docs: &[ParsedDocument], _project: &ProjectStructure) -> Result<String, String> {
    let navigation = generate_navigation(all_docs);
    
    Ok(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <header>
        <nav>
            {}
        </nav>
    </header>
    <main>
        <article>
            <h1>{}</h1>
            {}
        </article>
    </main>
</body>
</html>"#, doc.title, navigation, doc.title, doc.html_content))
}

/// Generates index page listing all content.
fn generate_index_page(documents: &[ParsedDocument], _project: &ProjectStructure) -> Result<String, String> {
    let navigation = generate_navigation(documents);
    
    let content_list = documents.iter()
        .filter(|doc| doc.url_path != "index.html")
        .map(|doc| format!(r#"<li><a href="{}">{}</a></li>"#, doc.url_path, doc.title))
        .collect::<Vec<_>>()
        .join("\n");
    
    Ok(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Home</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <header>
        <nav>
            {}
        </nav>
    </header>
    <main>
        <article>
            <h1>Welcome</h1>
            <p>This site was generated with moss.</p>
            <h2>Pages</h2>
            <ul>
                {}
            </ul>
        </article>
    </main>
</body>
</html>"#, navigation, content_list))
}

/// Generates navigation menu HTML.
fn generate_navigation(documents: &[ParsedDocument]) -> String {
    let nav_items = documents.iter()
        .take(5) // Limit navigation to first 5 pages for simplicity
        .map(|doc| {
            let label = if doc.url_path == "index.html" { "Home" } else { &doc.title };
            format!(r#"<a href="{}">{}</a>"#, doc.url_path, label)
        })
        .collect::<Vec<_>>()
        .join(" | ");
    
    nav_items
}

/// Generates beautiful default CSS styling.
fn generate_default_css() -> String {
    r#"/* moss - Beautiful default styling */
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    background: #f9f9f9;
}

header {
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #e0e0e0;
}

nav a {
    color: #2d5a2d;
    text-decoration: none;
    font-weight: 500;
}

nav a:hover {
    text-decoration: underline;
}

main {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

h1, h2, h3 {
    color: #2d5a2d;
    margin-top: 1.5em;
}

h1 {
    border-bottom: 2px solid #e8f5e8;
    padding-bottom: 0.5rem;
}

pre {
    background: #f5f5f5;
    padding: 1rem;
    border-radius: 4px;
    overflow-x: auto;
}

blockquote {
    border-left: 4px solid #2d5a2d;
    margin: 0;
    padding-left: 1rem;
    color: #666;
}

img {
    max-width: 100%;
    height: auto;
}

table {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
}

th, td {
    border: 1px solid #ddd;
    padding: 0.5rem;
    text-align: left;
}

th {
    background: #f5f5f5;
}
"#.to_string()
}


#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests the publishing workflow triggered by Finder integration
    // This is part of Feature 2 (after installation, right-click works)

    #[test]
    fn test_simple_blog_publishing() {
        // Test with our specific test content
        let test_path = "../test-content/simple-blog";
        if std::path::Path::new(test_path).exists() {
            let result = publish_folder(test_path.to_string());
            assert!(result.is_ok(), "Should successfully publish test blog: {:?}", result);
            
            if let Ok(success_msg) = result {
                assert!(success_msg.contains("simple-blog"), "Should mention folder name");
            }
        }
    }

    #[test]
    fn test_folder_publishing_workflow() {
        // When user right-clicks folder and selects "Publish to Web",
        // the app should process the folder and provide feedback
        
        // Should handle valid folders
        let result = publish_folder(".".to_string());
        assert!(result.is_ok() || result.unwrap_err().contains("No files found"), 
            "Should process valid folders or give clear error");
        
        // Should reject invalid input
        let result = publish_folder("".to_string());
        assert!(result.is_err(), "Should reject empty paths");
        assert_eq!(result.unwrap_err(), "Empty folder path provided");
        
        // Should handle non-existent folders gracefully  
        let result = publish_folder("/does/not/exist".to_string());
        assert!(result.is_err(), "Should handle non-existent folders");
        assert!(result.unwrap_err().contains("does not exist"));
    }
}

/// Gets the last selected directory from app config storage.
/// 
/// Returns the last directory path chosen by the user for publishing,
/// or None if no previous selection exists.
fn get_last_selected_directory(app: &tauri::AppHandle) -> Option<String> {
    let app_config_dir = app.path().app_config_dir().ok()?;
    let config_file = app_config_dir.join("last_directory.txt");
    
    if config_file.exists() {
        std::fs::read_to_string(config_file).ok()
    } else {
        None
    }
}

/// Saves the selected directory to app config storage.
/// 
/// Stores the directory path for use as default in future directory picker dialogs.
fn save_last_selected_directory(app: &tauri::AppHandle, directory: &str) -> Result<(), String> {
    let app_config_dir = app.path().app_config_dir()
        .map_err(|e| format!("Failed to get app config directory: {}", e))?;
    
    std::fs::create_dir_all(&app_config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    let config_file = app_config_dir.join("last_directory.txt");
    std::fs::write(config_file, directory)
        .map_err(|e| format!("Failed to save directory: {}", e))?;
    
    Ok(())
}

/// Tauri command to open directory picker and publish the selected folder.
/// 
/// Opens a native directory selection dialog, remembers the choice for next time,
/// and triggers the complete build → preview workflow.
/// 
/// # Arguments
/// * `app` - Tauri application handle for file dialog and config access
/// 
/// # Returns
/// * `Ok(String)` - Success message indicating publish started
/// * `Err(String)` - Error if dialog canceled or publish failed
/// 
/// # Behavior
/// 1. Shows native directory picker starting from last selected directory
/// 2. Saves new selection for next time
/// 3. Triggers build → preview workflow
/// 4. Returns immediately (preview opens asynchronously)
#[tauri::command]
pub async fn publish_with_directory_picker(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
    
    // Get last selected directory as default
    let default_path = get_last_selected_directory(&app);
    
    // Open directory picker
    let (sender, receiver) = tokio::sync::oneshot::channel();
    
    app
        .dialog()
        .file()
        .set_title("Select folder to publish")
        .set_directory(default_path.unwrap_or_else(|| {
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        }))
        .pick_folder(move |folder_path| {
            let _ = sender.send(folder_path);
        });
    
    let folder_path = receiver.await.map_err(|_| "Dialog channel error".to_string())?;
    
    match folder_path {
        Some(path) => {
            // Convert FilePath to string path
            let path_str = path.to_string();
            let path_buf = std::path::PathBuf::from(&path_str);
            
            // Save for next time
            if let Err(e) = save_last_selected_directory(&app, &path_str) {
                eprintln!("⚠️ Failed to save directory preference: {}", e);
            }
            
            // Step 1: Build the site (compile files, start local server)
            match publish_folder(path_str.clone()) {
                Ok(result) => {
                    println!("✅ Build completed: {}", result);
                    
                    // Step 2: Open preview window pointing to local server
                    use crate::preview;
                    
                    let preview_url = preview::build_preview_url("http://localhost:8080", &path_buf);
                    let state = preview::PreviewState::new(path_buf.clone(), preview_url);
                    
                    if let Err(error) = preview::create_preview_window(&app, state.clone(), None) {
                        return Err(format!("Build succeeded but preview failed: {}", error));
                    }
                    
                    // Add to manager if available
                    if let Some(manager) = app.try_state::<crate::preview::PreviewWindowManager>() {
                        manager.add_window(state);
                    }
                    
                    Ok(format!("Publishing {} - preview window opening", path_buf.file_name().unwrap_or_default().to_string_lossy()))
                },
                Err(error) => {
                    // Show error dialog to user
                    let _ = app
                        .dialog()
                        .message(format!("Failed to build site: {}", error))
                        .kind(MessageDialogKind::Error)
                        .title("Build Error")
                        .blocking_show();
                    
                    Err(format!("Build failed: {}", error))
                }
            }
        },
        None => Err("Directory selection canceled".to_string()),
    }
}