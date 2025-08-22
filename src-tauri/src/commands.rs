//! Tauri commands and business logic for Moss publishing system

use crate::types::*;
use walkdir::WalkDir;
use std::path::Path;

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
    
    println!("📊 Scanned folder: {} files found", total_files);
    println!("   📝 Markdown: {}", markdown_files.len());
    println!("   🌐 HTML: {}", html_files.len());
    println!("   🖼️  Images: {}", image_files.len());
    println!("   📄 Other: {}", other_files.len());
    println!("   🏠 Homepage: {:?}", homepage_file);
    println!("   📁 Content folders: {:?}", content_folders);
    println!("   🎯 Detected type: {:?}", project_type);
    
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
    
    println!("🌱 Publishing folder '{}'", folder_path);
    
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
    
    // TODO: Generate static site from scanned files
    // TODO: Deploy to moss.pub or configured hosting
    
    println!("✅ Ready to publish '{}' with {} files", folder_name, project_structure.total_files);
    
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
    let workflow_path = format!("{}/Publish to Web.workflow", services_dir);
    
    // Create Services directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&services_dir) {
        return Err(format!("Failed to create Services directory: {}", e));
    }
    
    // Remove existing workflow if it exists (to ensure clean reinstall)
    if Path::new(&workflow_path).exists() {
        if let Err(e) = fs::remove_dir_all(&workflow_path) {
            return Err(format!("Failed to remove existing workflow: {}", e));
        }
        println!("🗑️ Removed existing workflow for clean reinstall");
    }
    
    // Create the .workflow bundle directory
    if let Err(e) = fs::create_dir_all(format!("{}/Contents", workflow_path)) {
        return Err(format!("Failed to create workflow bundle: {}", e));
    }
    
    // Create Info.plist for the workflow bundle
    // NOTE: No NSIconName property to ensure it appears in top-level context menu
    let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.moss.publisher.publish-to-web</string>
    <key>CFBundleName</key>
    <string>Publish to Web</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict>
                <key>default</key>
                <string>Publish to Web</string>
            </dict>
            <key>NSMessage</key>
            <string>runWorkflowAsService</string>
            <key>NSSendFileTypes</key>
            <array>
                <string>public.folder</string>
            </array>
            <key>NSRequiredContext</key>
            <dict>
                <key>NSApplicationIdentifier</key>
                <string>com.apple.finder</string>
            </dict>
        </dict>
    </array>
</dict>
</plist>"#;
    
    // Write Info.plist
    let info_plist_path = format!("{}/Contents/Info.plist", workflow_path);
    if let Err(e) = fs::write(&info_plist_path, info_plist) {
        return Err(format!("Failed to write Info.plist: {}", e));
    }
    
    // Create the main workflow document (document.wflow)
    let workflow_document = include_str!("../workflow_template.xml");
    
    // Write the workflow document
    let document_path = format!("{}/Contents/document.wflow", workflow_path);
    if let Err(e) = fs::write(&document_path, workflow_document) {
        return Err(format!("Failed to write workflow document: {}", e));
    }
    
    println!("📁 Installed Finder integration: {}", workflow_path);
    Ok("Finder integration installed successfully! Right-click any folder → Quick Actions → 'Publish to Web'".to_string())
}


#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests the publishing workflow triggered by Finder integration
    // This is part of Feature 2 (after installation, right-click works)

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