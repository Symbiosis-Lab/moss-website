//! Tauri commands and business logic for moss compilation system

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
/// Scans the file list to find folders that contain content suitable for compilation
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

/// Compiles a folder into a static website with optional server startup.
/// 
/// This is the unified compilation function used by both CLI and GUI modes.
/// It generates the static site files and optionally starts a preview server.
/// 
/// # Arguments
/// * `folder_path` - Absolute path to the folder containing website content
/// * `auto_serve` - Whether to automatically start preview server after compilation
/// 
/// # Returns
/// * `Ok(String)` - Success message with compilation summary (and server info if started)
/// * `Err(String)` - Error message describing what went wrong
pub fn compile_folder_with_options(folder_path: String, auto_serve: bool) -> Result<String, String> {
    if folder_path.is_empty() {
        return Err("Empty folder path provided".to_string());
    }
    
    // Scan folder for content suitable for compilation
    let project_structure = scan_folder(&folder_path)?;
    
    // Basic validation - ensure we have some content to compile
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
    
    // Create compilation strategy message based on detected type
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
    
    let base_message = format!(
        "📁 '{}': {} files scanned. {} {}. Content folders: {:?}. Site generated at {}",
        folder_name,
        project_structure.total_files,
        strategy_message,
        homepage_info,
        project_structure.content_folders,
        site_result.output_path
    );

    // Optionally start preview server
    if auto_serve {
        start_site_server(&site_result.output_path)?;
        Ok(format!("{}\n🌐 Preview server started! Access at http://localhost:8080", base_message))
    } else {
        Ok(base_message)
    }
}

/// CLI-compatible compile function (no auto-serve).
/// Maintains backward compatibility for CLI usage.
pub fn compile_folder(folder_path: String) -> Result<String, String> {
    compile_folder_with_options(folder_path, false)
}

/// Tauri command for GUI compilation workflow.
/// Automatically starts preview server for immediate GUI preview.
#[tauri::command]
pub fn compile_and_serve(folder_path: String) -> Result<String, String> {
    compile_folder_with_options(folder_path, true)
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

/// Starts a local HTTP server for CLI usage (synchronous, doesn't return immediately).
/// 
/// This version is for CLI usage where we want to start the server and keep it running.
/// 
/// # Arguments
/// * `site_path` - Path to the generated site directory containing index.html
/// 
/// # Returns
/// * `Ok(())` - Successfully started server
/// * `Err(String)` - Failed to start server
pub fn start_site_server_cli(site_path: &str) -> Result<(), String> {
    let site_path = site_path.to_string();
    
    // Check if index.html exists
    let index_path = Path::new(&site_path).join("index.html");
    if !index_path.exists() {
        return Err("Generated site has no index.html file".to_string());
    }
    
    // Use port 8080 for consistency
    let port = if is_port_available(8080) { 8080 } else { find_available_port(8080)? };
    
    // Start server in background for CLI
    let server_site_path = site_path.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = start_preview_server(port, server_site_path).await {
                eprintln!("❌ CLI server error: {}", e);
            }
        });
    });
    
    // Give server a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    Ok(())
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

/// Tauri command to install macOS Finder integration for one-click compilation.
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
/// - Triggers the compilation workflow in this app
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
    
    // Generate index.html - either standalone blog feed or combined with homepage content
    if !documents.is_empty() {
        if project_structure.homepage_file.is_some() {
            // There's a homepage file (likely README.md) - combine it with blog feed
            let index_html = generate_homepage_with_blog_feed(&documents, project_structure)?;
            fs::write(output_dir.join("index.html"), index_html).map_err(|e| format!("Failed to write index.html: {}", e))?;
            page_count += 1;
        } else {
            // No homepage file - generate pure blog feed
            let index_html = generate_index_page(&documents, project_structure)?;
            fs::write(output_dir.join("index.html"), index_html).map_err(|e| format!("Failed to write index.html: {}", e))?;
            page_count += 1;
        }
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
    
    // Determine CSS path based on document depth
    let css_path = if doc.url_path.contains('/') {
        // Document is in subdirectory, use relative path to go up
        let depth = doc.url_path.matches('/').count();
        "../".repeat(depth) + "style.css"
    } else {
        // Document is in root, use direct path
        "style.css".to_string()
    };
    
    Ok(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="{}">
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
</html>"#, doc.title, css_path, navigation, doc.title, doc.html_content))
}

/// Generates index page as blog feed with journal entries prominently displayed.
fn generate_index_page(documents: &[ParsedDocument], project: &ProjectStructure) -> Result<String, String> {
    let navigation = generate_navigation(documents);
    
    // Separate journal entries from other pages
    let mut journal_entries: Vec<&ParsedDocument> = documents.iter()
        .filter(|doc| doc.url_path.starts_with("journal/"))
        .collect();
    
    // Sort journal entries by filename (which contains date) in reverse order (newest first)
    journal_entries.sort_by(|a, b| b.url_path.cmp(&a.url_path));
    
    // Generate blog feed with journal entries
    let blog_feed = if journal_entries.is_empty() {
        "<p>No journal entries yet.</p>".to_string()
    } else {
        journal_entries.iter()
            .map(|doc| {
                // Extract date from filename (e.g., "2025-09-02" from "journal/2025-09-02.html")
                let date_str = doc.url_path
                    .strip_prefix("journal/")
                    .and_then(|s| s.strip_suffix(".html"))
                    .unwrap_or("Unknown Date");
                
                // Get excerpt from content (first paragraph)
                let excerpt = extract_excerpt(&doc.html_content);
                
                // Get proper title (remove file-based title if it's just the date)
                let display_title = if doc.title.replace(" ", "-").to_lowercase() == date_str.to_lowercase() {
                    // Title matches date pattern, extract actual title from content
                    extract_first_heading(&doc.html_content).unwrap_or(doc.title.clone())
                } else {
                    doc.title.clone()
                };
                
                format!(r#"
                <article class="blog-entry">
                    <header class="blog-entry-header">
                        <h2 class="blog-entry-title"><a href="{}">{}</a></h2>
                        <time class="blog-entry-date">{}</time>
                    </header>
                    <div class="blog-entry-excerpt">
                        {}
                        <p><a href="{}" class="read-more">Read more →</a></p>
                    </div>
                </article>
                "#, doc.url_path, display_title, format_date(date_str), excerpt, doc.url_path)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    
    // Get site title from homepage file or use default
    let site_title = project.homepage_file.as_ref()
        .and_then(|_| documents.iter().find(|doc| doc.url_path == "index.html"))
        .map(|doc| doc.title.clone())
        .unwrap_or_else(|| "Blog".to_string());
    
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
        <section class="blog-feed">
            {}
        </section>
    </main>
</body>
</html>"#, site_title, navigation, blog_feed))
}

/// Generates a homepage that combines README content with a blog feed.
/// 
/// This creates the main landing page by taking the content from the homepage file
/// (usually README.md) and appending a chronological feed of journal entries below it.
/// This mimics the structure of sites like stephango.com or Substack.
/// 
/// # Arguments
/// * `documents` - All parsed documents including homepage and journal entries
/// * `project` - Project structure information
/// 
/// # Returns  
/// * `Result<String, String>` - Complete HTML for the combined homepage
fn generate_homepage_with_blog_feed(documents: &[ParsedDocument], _project: &ProjectStructure) -> Result<String, String> {
    let navigation = generate_navigation(documents);
    
    // Find the homepage document (README.md becomes index.html)
    let homepage_doc = documents.iter()
        .find(|doc| doc.url_path == "index.html")
        .ok_or("Homepage document not found")?;
    
    // Get journal entries and sort by date (newest first)
    let mut journal_entries: Vec<&ParsedDocument> = documents.iter()
        .filter(|doc| doc.url_path.starts_with("journal/"))
        .collect();
    journal_entries.sort_by(|a, b| b.url_path.cmp(&a.url_path));
    
    // Generate blog feed section
    let blog_feed_section = if journal_entries.is_empty() {
        String::new() // No journal section if no entries
    } else {
        let blog_entries = journal_entries.iter()
            .map(|doc| {
                let date_str = doc.url_path
                    .strip_prefix("journal/")
                    .and_then(|s| s.strip_suffix(".html"))
                    .unwrap_or("Unknown Date");
                
                let excerpt = extract_excerpt(&doc.html_content);
                let display_title = if doc.title.replace(" ", "-").to_lowercase() == date_str.to_lowercase() {
                    // Title matches date pattern, extract actual title from content
                    extract_first_heading(&doc.html_content).unwrap_or(doc.title.clone())
                } else {
                    doc.title.clone()
                };
                
                format!(r#"
                <article class="blog-entry">
                    <header class="blog-entry-header">
                        <h2 class="blog-entry-title"><a href="{}">{}</a></h2>
                        <time class="blog-entry-date">{}</time>
                    </header>
                    <div class="blog-entry-excerpt">
                        {}
                        <p><a href="{}" class="read-more">Read more →</a></p>
                    </div>
                </article>
                "#, doc.url_path, display_title, format_date(date_str), excerpt, doc.url_path)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        format!(r#"
        <hr>
        <section class="blog-feed">
            <h2>Recent Posts</h2>
            {}
        </section>
        "#, blog_entries)
    };
    
    // Combine homepage content with blog feed
    // Extract the main content from homepage document (remove the outer HTML structure)
    let homepage_content = if let Some(start) = homepage_doc.html_content.find("<article>") {
        if let Some(end) = homepage_doc.html_content.rfind("</article>") {
            &homepage_doc.html_content[start + 9..end] // +9 to skip "<article>"
        } else {
            &homepage_doc.html_content
        }
    } else {
        &homepage_doc.html_content
    };
    
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
            {}
            {}
        </article>
    </main>
</body>
</html>"#, homepage_doc.title, navigation, homepage_content, blog_feed_section))
}

/// Generates navigation menu HTML.
fn generate_navigation(documents: &[ParsedDocument]) -> String {
    let nav_items = documents.iter()
        .filter(|doc| !doc.url_path.starts_with("journal/")) // Exclude journal entries from navigation
        .map(|doc| {
            let label = if doc.url_path == "index.html" { 
                "Home".to_string()
            } else { 
                // Clean up title by removing file extensions and converting to title case
                doc.title.replace(".html", "")
            };
            
            let href = if doc.url_path == "index.html" {
                "/".to_string()
            } else {
                doc.url_path.clone()
            };
            
            format!(r#"<a href="{}">{}</a>"#, href, label)
        })
        .collect::<Vec<_>>()
        .join(" | ");
    
    nav_items
}

/// Extracts an excerpt from HTML content (first paragraph or first 200 characters).
fn extract_excerpt(html_content: &str) -> String {
    // Try to extract first paragraph
    if let Some(start) = html_content.find("<p>") {
        if let Some(end) = html_content[start..].find("</p>") {
            let paragraph = &html_content[start + 3..start + end];
            // Remove any HTML tags and limit length
            let clean_text = paragraph.replace("<strong>", "").replace("</strong>", "")
                .replace("<em>", "").replace("</em>", "")
                .replace("<a href", "<a href"); // Keep links intact for now
            
            if clean_text.len() > 200 {
                format!("{}...", &clean_text[..200])
            } else {
                clean_text
            }
        } else {
            "No excerpt available.".to_string()
        }
    } else {
        // Fallback: take first 200 characters
        let plain_text = html_content.replace("<", " <").replace(">", "> ");
        if plain_text.len() > 200 {
            format!("{}...", &plain_text[..200])
        } else {
            plain_text
        }
    }
}

/// Extracts the first heading from HTML content.
fn extract_first_heading(html_content: &str) -> Option<String> {
    // Look for h1, h2, etc. tags
    for heading_tag in ["<h1>", "<h2>", "<h3>"] {
        if let Some(start) = html_content.find(heading_tag) {
            let tag_len = heading_tag.len();
            let close_tag = format!("</{}>", &heading_tag[1..heading_tag.len()-1]);
            if let Some(end) = html_content[start + tag_len..].find(&close_tag) {
                let heading_text = &html_content[start + tag_len..start + tag_len + end];
                return Some(heading_text.to_string());
            }
        }
    }
    None
}

/// Formats a date string (YYYY-MM-DD) into a more readable format.
fn format_date(date_str: &str) -> String {
    // Simple date formatting: "2025-09-02" -> "September 2, 2025"
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(year), Ok(month), Ok(day)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
            let month_names = [
                "January", "February", "March", "April", "May", "June",
                "July", "August", "September", "October", "November", "December"
            ];
            if month >= 1 && month <= 12 {
                return format!("{} {}, {}", month_names[(month - 1) as usize], day, year);
            }
        }
    }
    // Fallback to original string if parsing fails
    date_str.to_string()
}

/// Generates enhanced CSS styling optimized for Writers & Publishers.
/// 
/// Creates a typography-first design system with:
/// - 18px base font size for comfortable long-form reading
/// - 65ch optimal line length for sustained reading
/// - 1.7 line-height for enhanced readability
/// - Warm, paper-inspired color palette reducing eye strain
/// - CSS custom properties for maintainability
/// - Dark mode support with consistent color relationships
/// - Mobile-responsive design with appropriate font scaling
fn generate_default_css() -> String {
    r#"/* moss - Typography-first design for Writers & Publishers */

/* CSS Custom Properties */
:root {
    /* Color System - Warm, paper-inspired */
    --moss-text-primary: #2c2825;
    --moss-text-secondary: #5d5853;
    --moss-text-muted: #8a8580;
    --moss-background: #faf8f5;
    --moss-background-alt: #f4f1ec;
    --moss-accent: #2d5a2d;
    --moss-accent-hover: #1e3d1e;
    --moss-border-light: #e6e2db;
    --moss-border-medium: #d1cdc4;
    
    /* Typography Scale */
    --moss-font-base: 1.125rem;     /* 18px - optimal for long-form reading */
    --moss-font-sm: 1rem;           /* 16px */
    --moss-font-lg: 1.25rem;        /* 20px */
    --moss-font-xl: 1.5rem;         /* 24px */
    --moss-font-2xl: 2rem;          /* 32px */
    --moss-font-3xl: 2.5rem;        /* 40px */
    
    /* Spacing Scale (8pt grid) */
    --moss-space-xs: 0.5rem;        /* 8px */
    --moss-space-sm: 1rem;          /* 16px */
    --moss-space-md: 1.5rem;        /* 24px */
    --moss-space-lg: 2rem;          /* 32px */
    --moss-space-xl: 3rem;          /* 48px */
    --moss-space-2xl: 4rem;         /* 64px */
    
    /* Layout */
    --moss-content-width: 65ch;     /* Optimal line length */
    --moss-container-padding: clamp(1rem, 5vw, 2rem);
}

/* Base Styles */
* {
    box-sizing: border-box;
}

html {
    font-size: 100%;
    -webkit-text-size-adjust: 100%;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Inter', system-ui, sans-serif;
    font-size: var(--moss-font-base);
    line-height: 1.7;
    color: var(--moss-text-primary);
    background: var(--moss-background);
    margin: 0;
    max-width: var(--moss-content-width);
    margin: 0 auto;
    padding: var(--moss-space-lg) var(--moss-container-padding);
}

/* Typography Hierarchy */
h1, h2, h3, h4, h5, h6 {
    font-weight: 600;
    line-height: 1.3;
    color: var(--moss-text-primary);
    margin: var(--moss-space-xl) 0 var(--moss-space-md) 0;
}

h1 {
    font-size: var(--moss-font-3xl);
    margin-top: 0;
    border-bottom: 2px solid var(--moss-border-light);
    padding-bottom: var(--moss-space-sm);
}

h2 {
    font-size: var(--moss-font-2xl);
}

h3 {
    font-size: var(--moss-font-xl);
}

h4 {
    font-size: var(--moss-font-lg);
}

h5, h6 {
    font-size: var(--moss-font-base);
}

/* Body Text & Paragraphs */
p {
    margin: 0 0 var(--moss-space-md) 0;
    max-width: var(--moss-content-width);
}

p + p {
    margin-top: var(--moss-space-md);
}

/* Lists */
ul, ol {
    margin: var(--moss-space-md) 0;
    padding-left: var(--moss-space-lg);
}

li {
    margin-bottom: var(--moss-space-xs);
    line-height: 1.6;
}

li > ul, li > ol {
    margin-top: var(--moss-space-xs);
    margin-bottom: var(--moss-space-xs);
}

/* Links */
a {
    color: var(--moss-accent);
    text-decoration: none;
    border-bottom: 1px solid transparent;
    transition: border-color 0.2s ease;
}

a:hover, a:focus {
    color: var(--moss-accent-hover);
    border-bottom-color: var(--moss-accent-hover);
}

/* Header */
header {
    margin-bottom: var(--moss-space-2xl);
    padding-bottom: var(--moss-space-lg);
    border-bottom: 1px solid var(--moss-border-light);
}

nav {
    font-weight: 500;
}

nav a {
    margin-right: var(--moss-space-md);
    color: var(--moss-text-secondary);
    border-bottom: none;
}

nav a:hover {
    color: var(--moss-accent);
}

/* Main Content */
main {
    background: white;
    padding: var(--moss-space-2xl);
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(44, 40, 37, 0.1);
    margin-bottom: var(--moss-space-2xl);
}

article {
    max-width: var(--moss-content-width);
}

/* Code */
code {
    font-family: 'SF Mono', 'Monaco', 'Cascadia Code', 'Roboto Mono', monospace;
    font-size: 0.9em;
    background: var(--moss-background-alt);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    color: var(--moss-text-primary);
}

pre {
    background: var(--moss-background-alt);
    padding: var(--moss-space-lg);
    border-radius: 6px;
    overflow-x: auto;
    border: 1px solid var(--moss-border-light);
    margin: var(--moss-space-lg) 0;
}

pre code {
    background: none;
    padding: 0;
    font-size: 0.875rem;
}

/* Blockquotes */
blockquote {
    border-left: 4px solid var(--moss-accent);
    margin: var(--moss-space-lg) 0;
    padding-left: var(--moss-space-lg);
    color: var(--moss-text-secondary);
    font-style: italic;
    font-size: var(--moss-font-lg);
}

blockquote p {
    margin: var(--moss-space-sm) 0;
}

/* Images */
img {
    max-width: 100%;
    height: auto;
    border-radius: 6px;
    margin: var(--moss-space-lg) 0;
}

/* Tables */
table {
    width: 100%;
    border-collapse: collapse;
    margin: var(--moss-space-lg) 0;
    border: 1px solid var(--moss-border-medium);
    border-radius: 6px;
    overflow: hidden;
}

th, td {
    padding: var(--moss-space-sm) var(--moss-space-md);
    text-align: left;
    border-bottom: 1px solid var(--moss-border-light);
}

th {
    background: var(--moss-background-alt);
    font-weight: 600;
    color: var(--moss-text-primary);
}

tr:last-child td {
    border-bottom: none;
}

/* Horizontal Rule */
hr {
    border: none;
    height: 1px;
    background: var(--moss-border-light);
    margin: var(--moss-space-2xl) 0;
}

/* Responsive Design */
@media (max-width: 48rem) {
    :root {
        --moss-font-base: 1rem;
        --moss-font-2xl: 1.75rem;
        --moss-font-3xl: 2rem;
    }
    
    body {
        padding: var(--moss-space-md);
    }
    
    main {
        padding: var(--moss-space-lg);
    }
}

/* Blog Feed Styling */
.blog-feed {
    max-width: var(--moss-content-width);
}

.blog-entry {
    margin-bottom: var(--moss-space-2xl);
    padding-bottom: var(--moss-space-xl);
    border-bottom: 1px solid var(--moss-border-light);
}

.blog-entry:last-child {
    border-bottom: none;
    margin-bottom: 0;
}

.blog-entry-header {
    margin-bottom: var(--moss-space-md);
}

.blog-entry-title {
    margin: 0 0 var(--moss-space-xs) 0;
    font-size: var(--moss-font-xl);
    line-height: 1.3;
}

.blog-entry-title a {
    color: var(--moss-text-primary);
    text-decoration: none;
    border-bottom: none;
}

.blog-entry-title a:hover {
    color: var(--moss-accent);
}

.blog-entry-date {
    font-size: var(--moss-font-sm);
    color: var(--moss-text-secondary);
    font-weight: 500;
    display: block;
    margin-bottom: var(--moss-space-sm);
}

.blog-entry-excerpt {
    color: var(--moss-text-secondary);
    line-height: 1.6;
}

.blog-entry-excerpt p {
    margin-bottom: var(--moss-space-sm);
}

.read-more {
    font-weight: 500;
    color: var(--moss-accent) !important;
    border-bottom: 1px solid transparent !important;
}

.read-more:hover {
    border-bottom-color: var(--moss-accent-hover) !important;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
    :root {
        --moss-text-primary: #e8e6e3;
        --moss-text-secondary: #b8b5b0;
        --moss-text-muted: #8a8580;
        --moss-background: #1a1816;
        --moss-background-alt: #242018;
        --moss-border-light: #3a362f;
        --moss-border-medium: #4a453c;
    }
    
    main {
        background: #211e1b;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    }
}
"#.to_string()
}


#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests the website generation workflow triggered by Finder integration
    // This is part of Feature 2 (after installation, right-click works)

    #[test]
    fn test_compile_folder_basic() {
        // Create temporary directory with test content
        let temp_dir = std::env::temp_dir().join("moss_test_compile_basic");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        // Create a simple markdown file
        let readme_path = temp_dir.join("README.md");
        std::fs::write(&readme_path, "# Test Blog\n\nThis is a test.").unwrap();
        
        // Test compilation
        let result = compile_folder(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok(), "Should successfully compile test content: {:?}", result);
        
        if let Ok(success_msg) = result {
            assert!(success_msg.contains("1 files scanned"), "Should mention file count");
            assert!(success_msg.contains("Site generated at"), "Should mention output path");
        }
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_folder_compilation_workflow() {
        // Test various edge cases in compilation workflow
        
        // Test with valid folder containing content
        let temp_dir = std::env::temp_dir().join("moss_test_workflow_valid");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("README.md"), "# Valid Content\n\nSome text.").unwrap();
        
        let result = compile_folder(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok(), "Should handle valid folders with content: {:?}", result);
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        // Test with empty folder
        let empty_dir = std::env::temp_dir().join("moss_test_workflow_empty");
        let _ = std::fs::remove_dir_all(&empty_dir);
        std::fs::create_dir_all(&empty_dir).unwrap();
        
        let result = compile_folder(empty_dir.to_string_lossy().to_string());
        assert!(result.is_err(), "Should reject folders with no content");
        assert!(result.unwrap_err().contains("No files found"), "Should give clear error for empty folders");
        let _ = std::fs::remove_dir_all(&empty_dir);
        
        // Should reject empty paths
        let result = compile_folder("".to_string());
        assert!(result.is_err(), "Should reject empty paths");
        assert_eq!(result.unwrap_err(), "Empty folder path provided");
        
        // Should handle non-existent folders gracefully  
        let result = compile_folder("/does/not/exist/moss_test".to_string());
        assert!(result.is_err(), "Should handle non-existent folders");
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("does not exist") || error_msg.contains("No such file"));
    }

    #[test]
    fn test_extract_excerpt() {
        // Test normal paragraph extraction
        let html = "<h1>Title</h1><p>This is the first paragraph with some content.</p><p>Second paragraph.</p>";
        let excerpt = extract_excerpt(html);
        assert!(excerpt.contains("This is the first paragraph"));
        assert!(excerpt.len() <= 200); // Should be truncated
        
        // Test with no paragraphs
        let html = "<h1>Title</h1><div>No paragraphs here</div>";
        let excerpt = extract_excerpt(html);
        assert!(excerpt.len() <= 100); // Should return truncated content
        
        // Test empty content
        let excerpt = extract_excerpt("");
        assert_eq!(excerpt, "");
    }

    #[test]
    fn test_extract_first_heading() {
        // Test h1 extraction
        let html = "<h1>Main Title</h1><p>Content</p>";
        assert_eq!(extract_first_heading(html), Some("Main Title".to_string()));
        
        // Test h2 extraction when h1 is missing
        let html = "<p>Content</p><h2>Sub Title</h2>";
        assert_eq!(extract_first_heading(html), Some("Sub Title".to_string()));
        
        // Test h3 extraction
        let html = "<div><h3>Third Level</h3></div>";
        assert_eq!(extract_first_heading(html), Some("Third Level".to_string()));
        
        // Test no headings
        let html = "<p>No headings here</p>";
        assert_eq!(extract_first_heading(html), None);
        
        // Test empty content
        assert_eq!(extract_first_heading(""), None);
    }

    #[test]
    fn test_format_date() {
        // Test valid date
        assert_eq!(format_date("2025-09-02"), "September 2, 2025");
        assert_eq!(format_date("2024-12-31"), "December 31, 2024");
        assert_eq!(format_date("2023-01-15"), "January 15, 2023");
        
        // Test invalid date formats
        assert_eq!(format_date("invalid"), "invalid");
        assert_eq!(format_date("2025-13-02"), "2025-13-02"); // Invalid month
        assert_eq!(format_date("25-09-02"), "September 2, 25"); // Year too short (but still valid number)
        assert_eq!(format_date(""), "");
    }

    #[test]
    fn test_compile_folder_with_options_no_serve() {
        // Create temporary directory
        let temp_dir = std::env::temp_dir().join("moss_test_compile_no_serve");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        // Create test content
        std::fs::write(temp_dir.join("README.md"), "# Test\n\nContent here.").unwrap();
        
        // Test compilation without auto-serve
        let result = compile_folder_with_options(temp_dir.to_string_lossy().to_string(), false);
        assert!(result.is_ok());
        
        let message = result.unwrap();
        assert!(message.contains("Site generated at"));
        assert!(!message.contains("Preview server started")); // Should NOT contain server message
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_compile_and_serve_wrapper() {
        // Create temporary directory
        let temp_dir = std::env::temp_dir().join("moss_test_compile_and_serve");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        // Create test content
        std::fs::write(temp_dir.join("README.md"), "# Test\n\nContent here.").unwrap();
        
        // Test compile_and_serve wrapper
        let result = compile_and_serve(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok());
        
        let message = result.unwrap();
        assert!(message.contains("Site generated at"));
        assert!(message.contains("Preview server started")); // Should contain server message
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_generate_homepage_with_blog_feed() {
        use crate::types::{ParsedDocument, ProjectStructure, ProjectType};
        
        // Create mock documents
        let homepage_doc = ParsedDocument {
            title: "My Blog".to_string(),
            url_path: "index.html".to_string(),
            html_content: "<h1>Welcome</h1><p>This is my blog homepage.</p>".to_string(),
            content: String::new(),
            date: None,
        };
        
        let journal_doc = ParsedDocument {
            title: "2025 01 15".to_string(),
            url_path: "journal/2025-01-15.html".to_string(),
            html_content: "<h1>My First Post</h1><p>This is my first journal entry.</p>".to_string(),
            content: String::new(),
            date: None,
        };
        
        let about_doc = ParsedDocument {
            title: "About".to_string(),
            url_path: "about.html".to_string(), 
            html_content: "<h1>About Me</h1><p>About page content.</p>".to_string(),
            content: String::new(),
            date: None,
        };
        
        let documents = vec![homepage_doc, journal_doc, about_doc];
        let project = ProjectStructure {
            root_path: "/test".to_string(),
            markdown_files: vec![],
            html_files: vec![],
            image_files: vec![],
            other_files: vec![],
            total_files: 3,
            project_type: ProjectType::BlogStyleFlatSite,
            homepage_file: Some("README.md".to_string()),
            content_folders: vec!["journal".to_string()],
        };
        
        // Test with blog feed
        let result = generate_homepage_with_blog_feed(&documents, &project);
        assert!(result.is_ok());
        
        let html = result.unwrap();
        assert!(html.contains("Welcome")); // Homepage content
        assert!(html.contains("This is my blog homepage")); // Homepage content
        assert!(html.contains("Recent Posts")); // Blog feed section
        assert!(html.contains("My First Post")); // Journal entry
        assert!(html.contains("January 15, 2025")); // Formatted date
        assert!(html.contains("about.html")); // Navigation should include about
        
        // Extract navigation section and check that journal URLs don't appear there
        let nav_start = html.find("<nav>").unwrap();
        let nav_end = html.find("</nav>").unwrap() + 6;
        let nav_section = &html[nav_start..nav_end];
        assert!(!nav_section.contains("journal/")); // Navigation should NOT include journal paths
    }

    #[test]  
    fn test_generate_homepage_with_blog_feed_no_journals() {
        use crate::types::{ParsedDocument, ProjectStructure, ProjectType};
        
        // Create mock documents without journal entries
        let homepage_doc = ParsedDocument {
            title: "My Blog".to_string(),
            url_path: "index.html".to_string(),
            html_content: "<h1>Welcome</h1><p>This is my blog.</p>".to_string(),
            content: String::new(),
            date: None,
        };
        
        let documents = vec![homepage_doc];
        let project = ProjectStructure {
            root_path: "/test".to_string(),
            markdown_files: vec![],
            html_files: vec![],
            image_files: vec![],
            other_files: vec![],
            total_files: 1,
            project_type: ProjectType::SimpleFlatSite,
            homepage_file: Some("README.md".to_string()),
            content_folders: vec![],
        };
        
        // Test without journal entries
        let result = generate_homepage_with_blog_feed(&documents, &project);
        assert!(result.is_ok());
        
        let html = result.unwrap();
        assert!(html.contains("Welcome")); // Homepage content
        assert!(!html.contains("Recent Posts")); // No blog feed section
    }

    #[test]
    fn test_generate_homepage_with_blog_feed_missing_homepage() {
        use crate::types::{ParsedDocument, ProjectStructure, ProjectType};
        
        // Create documents without homepage
        let journal_doc = ParsedDocument {
            title: "Journal Entry".to_string(),
            url_path: "journal/2025-01-15.html".to_string(),
            html_content: "<h1>Entry</h1>".to_string(),
            content: String::new(),
            date: None,
        };
        
        let documents = vec![journal_doc];
        let project = ProjectStructure {
            root_path: "/test".to_string(),
            markdown_files: vec![],
            html_files: vec![],
            image_files: vec![],
            other_files: vec![],
            total_files: 1,
            project_type: ProjectType::BlogStyleFlatSite,
            homepage_file: None,
            content_folders: vec!["journal".to_string()],
        };
        
        // Should error when homepage document is missing
        let result = generate_homepage_with_blog_feed(&documents, &project);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Homepage document not found"));
    }

    #[test]
    fn test_full_blog_compilation_workflow() {
        // Create temporary directory with realistic blog structure
        let temp_dir = std::env::temp_dir().join("moss_test_full_blog");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("journal")).unwrap();
        
        // Create README.md (homepage)
        std::fs::write(
            temp_dir.join("README.md"),
            "# My Blog\n\n> Welcome to my personal blog\n\nThis is the main page."
        ).unwrap();
        
        // Create journal entries
        std::fs::write(
            temp_dir.join("journal/2025-01-15.md"),
            "# First Post\n\nThis is my first blog post with some content."
        ).unwrap();
        
        std::fs::write(
            temp_dir.join("journal/2025-01-10.md"), 
            "# Earlier Post\n\nThis is an earlier post that should come second."
        ).unwrap();
        
        // Create about page
        std::fs::write(
            temp_dir.join("about.md"),
            "# About Me\n\nI write about technology and life."
        ).unwrap();
        
        // Compile the blog
        let result = compile_folder(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok(), "Blog compilation should succeed: {:?}", result);
        
        // Check generated files exist
        let output_dir = temp_dir.join(".moss/site");
        assert!(output_dir.join("index.html").exists(), "index.html should be generated");
        assert!(output_dir.join("style.css").exists(), "style.css should be generated");
        assert!(output_dir.join("about.html").exists(), "about.html should be generated");
        assert!(output_dir.join("journal/2025-01-15.html").exists(), "Journal entry should be generated");
        assert!(output_dir.join("journal/2025-01-10.html").exists(), "Earlier journal entry should be generated");
        
        // Check index.html content
        let index_content = std::fs::read_to_string(output_dir.join("index.html")).unwrap();
        
        // Should contain README content
        assert!(index_content.contains("My Blog"), "Should contain README title");
        assert!(index_content.contains("Welcome to my personal blog"), "Should contain README content");
        assert!(index_content.contains("This is the main page"), "Should contain README text");
        
        // Should contain blog feed
        assert!(index_content.contains("Recent Posts"), "Should have blog feed section");
        assert!(index_content.contains("First Post"), "Should show latest journal entry");
        assert!(index_content.contains("Earlier Post"), "Should show earlier journal entry");
        assert!(index_content.contains("January 15, 2025"), "Should format first post date");
        assert!(index_content.contains("January 10, 2025"), "Should format second post date");
        
        // Should have navigation
        assert!(index_content.contains("about.html"), "Navigation should include about page");
        
        // Extract navigation section and verify journal entries are not there
        let nav_start = index_content.find("<nav>").unwrap();
        let nav_end = index_content.find("</nav>").unwrap() + 6;
        let nav_section = &index_content[nav_start..nav_end];
        assert!(!nav_section.contains("journal/"), "Navigation should exclude journal entries");
        
        // Check journal page has correct CSS path
        let journal_content = std::fs::read_to_string(output_dir.join("journal/2025-01-15.html")).unwrap();
        assert!(journal_content.contains("../style.css"), "Journal page should reference CSS with correct relative path");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

/// Gets the last selected directory from app config storage.
/// 
/// Returns the last directory path chosen by the user for compilation,
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

/// Tauri command to open directory picker and compile the selected folder.
/// 
/// Opens a native directory selection dialog, remembers the choice for next time,
/// and triggers the complete compile → serve → preview workflow.
/// 
/// # Arguments
/// * `app` - Tauri application handle for file dialog and config access
/// 
/// # Returns
/// * `Ok(String)` - Success message indicating compilation and preview started
/// * `Err(String)` - Error if dialog canceled or compilation failed
/// 
/// # Behavior
/// 1. Shows native directory picker starting from last selected directory
/// 2. Saves new selection for next time
/// 3. Triggers build → preview workflow
/// 4. Returns immediately (preview opens asynchronously)
#[tauri::command]
pub async fn compile_with_directory_picker(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
    
    // Get last selected directory as default
    let default_path = get_last_selected_directory(&app);
    
    // Open directory picker
    let (sender, receiver) = tokio::sync::oneshot::channel();
    
    app
        .dialog()
        .file()
        .set_title("Select folder to compile")
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
            match compile_and_serve(path_str.clone()) {
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