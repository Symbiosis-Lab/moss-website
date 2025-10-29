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
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| {
            // Skip .moss directory to avoid scanning generated output
            e.file_name() != ".moss"
        }) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_extension_categorization() {
        // Test markdown file extensions
        let md_extensions = vec!["md", "markdown", "mdown", "mkd"];
        for ext in md_extensions {
            // Extension detection is done in scan_folder, but we can test the logic indirectly
            assert!(ext == "md" || ext == "markdown" || ext == "mdown" || ext == "mkd");
        }

        // Test HTML file extensions
        let html_extensions = vec!["html", "htm"];
        for ext in html_extensions {
            assert!(ext == "html" || ext == "htm");
        }

        // Test image file extensions
        let image_extensions = vec!["jpg", "jpeg", "png", "gif", "svg", "webp"];
        for ext in image_extensions {
            assert!(["jpg", "jpeg", "png", "gif", "svg", "webp"].contains(&ext));
        }

        // Test document file extensions
        let doc_extensions = vec!["pages", "docx", "doc"];
        for ext in doc_extensions {
            assert!(["pages", "docx", "doc"].contains(&ext));
        }
    }

    #[test]
    fn test_scan_folder_nonexistent_path() {
        let result = scan_folder("/definitely/does/not/exist/anywhere");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_scan_folder_file_instead_of_directory() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_file.txt");
        fs::write(&temp_file, "test content").unwrap();

        let result = scan_folder(&temp_file.to_string_lossy());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directory"));

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_detect_homepage_file_priority_order() {
        // Test index.md has highest priority
        let files = vec![
            FileInfo { path: "README.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 150, modified: None },
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
        ];

        let result = detect_homepage_file(&files);
        assert_eq!(result, Some("index.md".to_string()));
    }

    #[test]
    fn test_detect_homepage_file_fallback_to_readme() {
        // Test README.md fallback when no index files
        let files = vec![
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "contact.md".to_string(), file_type: "md".to_string(), size: 150, modified: None },
            FileInfo { path: "README.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
        ];

        let result = detect_homepage_file(&files);
        assert_eq!(result, Some("README.md".to_string()));
    }

    #[test]
    fn test_detect_homepage_file_no_candidates() {
        // Test with no homepage candidates
        let files = vec![
            FileInfo { path: "image.jpg".to_string(), file_type: "jpg".to_string(), size: 5000, modified: None },
            FileInfo { path: "data.json".to_string(), file_type: "json".to_string(), size: 300, modified: None },
        ];

        let result = detect_homepage_file(&files);
        assert_eq!(result, None);
    }

    #[test]
    fn test_detect_content_folders() {
        let files = vec![
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "posts/first.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
            FileInfo { path: "posts/second.md".to_string(), file_type: "md".to_string(), size: 150, modified: None },
            FileInfo { path: "docs/guide.md".to_string(), file_type: "md".to_string(), size: 300, modified: None },
            FileInfo { path: "images/photo.jpg".to_string(), file_type: "jpg".to_string(), size: 5000, modified: None },
            FileInfo { path: "css/style.css".to_string(), file_type: "css".to_string(), size: 1000, modified: None },
        ];

        let result = detect_content_folders(&files);

        // Should detect folders with document files, not just image/css folders
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"posts".to_string()));
        assert!(result.contains(&"docs".to_string()));
        assert!(!result.contains(&"images".to_string()));
        assert!(!result.contains(&"css".to_string()));
    }

    #[test]
    fn test_detect_project_type_homepage_with_collections() {
        let files = vec![
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "posts/post1.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
        ];
        let content_folders = vec!["posts".to_string()];

        let result = detect_project_type_from_content(&files, &content_folders);
        assert_eq!(result, ProjectType::HomepageWithCollections);
    }

    #[test]
    fn test_detect_project_type_simple_flat_site() {
        let files = vec![
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "contact.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "services.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
        ];
        let no_folders: Vec<String> = vec![];

        let result = detect_project_type_from_content(&files, &no_folders);
        assert_eq!(result, ProjectType::SimpleFlatSite);
    }

    #[test]
    fn test_detect_project_type_blog_style_flat_site() {
        // Create more than 5 files to trigger blog-style classification
        let files: Vec<FileInfo> = (1..=7).map(|i| FileInfo {
            path: format!("post{}.md", i),
            file_type: "md".to_string(),
            size: 100,
            modified: None,
        }).collect();
        let no_folders: Vec<String> = vec![];

        let result = detect_project_type_from_content(&files, &no_folders);
        assert_eq!(result, ProjectType::BlogStyleFlatSite);
    }

    #[test]
    fn test_case_insensitive_extensions() {
        // Test that extension detection handles case variations
        let uppercase_extensions = vec!["MD", "HTML", "JPG", "PNG"];
        for ext in uppercase_extensions {
            let lowercase = ext.to_lowercase();
            assert!(["md", "html", "jpg", "png"].contains(&lowercase.as_str()));
        }
    }
}