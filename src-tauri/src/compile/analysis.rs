//! Folder analysis and project structure detection for moss compilation
//! 
//! This module handles the first phase of website compilation: analyzing a directory
//! to understand its content and structure. It scans folders recursively, categorizes
//! files by type, and makes intelligent decisions about how the site should be organized.

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
pub fn scan_folder(folder_path: &str) -> Result<ProjectStructure, String> {
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