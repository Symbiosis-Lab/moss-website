//! Moss compilation module - Static site generation from folders
//! 
//! This module provides the core functionality for transforming folders containing
//! documents (markdown, Pages, Word) into static websites. It orchestrates three
//! main phases: analysis, generation, and serving.
//!
//! ## Module Structure
//! - `analysis` - Folder scanning and project structure detection
//! - `generator` - Static site generation and HTML templating  
//! - `server` - Preview server for generated sites
//!
//! ## Core Workflow
//! 1. **Analysis**: Scan folder → detect structure → classify project type
//! 2. **Generation**: Process content → generate HTML → copy assets
//! 3. **Serving**: Start preview server → enable live preview

pub mod analysis;
pub mod generator; 
pub mod navigation;
pub mod server;

// Re-export public functions for backward compatibility
pub use analysis::scan_folder;
pub use generator::generate_static_site;
pub use server::start_preview_server;

#[cfg(test)]
pub use analysis::{detect_homepage_file, detect_content_folders, detect_project_type_from_content};

use crate::types::*;
use std::path::Path;



/// Synchronous compilation function for CLI usage.
/// 
/// This is a blocking compilation function for CLI usage without frontend communication.
/// 
/// # Arguments
/// * `folder_path` - Absolute path to the folder containing website content
/// * `auto_serve` - Whether to automatically start preview server after compilation (default: false)
/// 
/// # Returns
/// * `Ok(String)` - Success message with compilation summary (and server info if started)
/// * `Err(String)` - Error message describing what went wrong
pub fn compile_folder_sync(folder_path: String, auto_serve: Option<bool>) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // Replicate compile_folder logic but without channel support for CLI usage
        let auto_serve = auto_serve.unwrap_or(false);
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

        // Optionally start preview server (CLI mode - no reuse)
        if auto_serve {
            let port = start_preview_server(&site_result.output_path, None).await?;
            let preview_url = format!("http://localhost:{}", port);
            Ok(format!("{}\n🌐 Preview server ready! Access at {}", base_message, preview_url))
        } else {
            Ok(base_message)
        }
    })
}

/// Tauri command for folder compilation with real-time progress channel.
/// 
/// This function handles GUI compilation with real-time progress updates sent
/// through a Tauri channel. It performs folder scanning, site generation, and
/// preview server startup with detailed progress information.
/// 
/// # Arguments
/// * `app` - Tauri app handle for preview URL communication
/// * `folder_path` - Absolute path to the folder containing website content
/// * `auto_serve` - Whether to automatically start preview server after compilation (default: false)
/// * `on_progress` - Channel for real-time progress updates
/// 
/// # Returns
/// * `Ok(String)` - Success message with compilation summary (and server info if started)
/// * `Err(String)` - Error message describing what went wrong
#[tauri::command]
#[specta::specta]
pub async fn compile_folder(
    app: tauri::AppHandle, 
    folder_path: String, 
    auto_serve: Option<bool>,
    on_progress: tauri::ipc::Channel<crate::types::ProgressUpdate>
) -> Result<String, String> {
    use crate::types::ProgressUpdate;
    
    let auto_serve = auto_serve.unwrap_or(false);
    if folder_path.is_empty() {
        return Err("Empty folder path provided".to_string());
    }
    
    // Helper function to send progress updates
    let send_progress = |step: &str, message: &str, percentage: u8, completed: bool, port: Option<u16>| {
        let update = ProgressUpdate {
            step: step.to_string(),
            message: message.to_string(),
            percentage,
            completed,
            port,
        };
        let _ = on_progress.send(update);
    };

    // Step 1: Start scanning
    send_progress("scanning", "Scanning folder structure...", 10, false, None);
    
    // Scan folder for content suitable for compilation
    let project_structure = scan_folder(&folder_path)?;
    
    // Basic validation - ensure we have some content to compile
    if project_structure.total_files == 0 {
        return Err("No files found in the specified folder".to_string());
    }
    
    // Step 2: Analysis complete
    let folder_name = Path::new(&folder_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unnamed-site");
    
    send_progress(
        "analyzing",
        &format!("Analyzed {} files in '{}'", project_structure.total_files, folder_name),
        30,
        true,
        None
    );
    
    // Step 3: Start generating site
    send_progress("generating", "Generating static site...", 40, false, None);
    
    // Generate static site from scanned files
    let site_result = generate_static_site(&folder_path, &project_structure)?;
    
    send_progress("generating", "Static site generated successfully", 70, true, None);
    
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

    // Optionally start preview server and send preview URL to frontend
    if auto_serve {
        // Step 4: Start server
        send_progress("serving", "Starting preview server...", 80, false, None);
        
        let port = start_preview_server(&site_result.output_path, Some(&app)).await?;
        let preview_url = format!("http://localhost:{}", port);

        // Server is now automatically recorded in state by start_preview_server

        send_progress("serving", &format!("Preview server started on port {}", port), 90, true, None);
        
        // Step 5: Complete
        send_progress("ready", "Ready! Site compiled successfully.", 100, true, Some(port));
        
        Ok(format!("{}\n🌐 Preview server ready! Access at {}", base_message, preview_url))
    } else {
        send_progress("complete", "Compilation completed", 100, true, None);
        Ok(base_message)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileInfo;

    /// Feature 1: Content Analysis - Homepage Detection
    /// Tests the core logic for identifying the main page of a website
    #[test]
    fn test_content_analysis_homepage_detection() {
        // Test priority order: index.md > index.pages > index.docx > README.md
        let files = vec![
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "README.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 150, modified: None },
        ];
        
        let result = detect_homepage_file(&files);
        assert_eq!(result, Some("index.md".to_string()), "Should prioritize index.md over README.md");
        
        // Test fallback to README.md when no index files exist
        let files_no_index = vec![
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "README.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
        ];
        
        let result = detect_homepage_file(&files_no_index);
        assert_eq!(result, Some("README.md".to_string()), "Should fallback to README.md");
    }
    
    /// Feature 2: Content Analysis - Folder Detection  
    /// Tests identification of content organization patterns
    #[test] 
    fn test_content_analysis_folder_detection() {
        let files = vec![
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "posts/first-post.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
            FileInfo { path: "posts/second-post.md".to_string(), file_type: "md".to_string(), size: 150, modified: None },
            FileInfo { path: "projects/app.docx".to_string(), file_type: "docx".to_string(), size: 300, modified: None },
            FileInfo { path: "images/photo.jpg".to_string(), file_type: "jpg".to_string(), size: 5000, modified: None },
        ];
        
        let result = detect_content_folders(&files);
        assert_eq!(result.len(), 2, "Should detect 2 content folders");
        assert!(result.contains(&"posts".to_string()), "Should detect posts folder");
        assert!(result.contains(&"projects".to_string()), "Should detect projects folder");
        assert!(!result.contains(&"images".to_string()), "Should ignore image-only folders");
    }
    
    /// Feature 3: Content Analysis - Project Classification
    /// Tests the logic for determining optimal site structure
    #[test]
    fn test_content_analysis_project_classification() {
        // Test 1: Homepage with collections (has content folders)
        let files_with_collections = vec![
            FileInfo { path: "index.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "posts/post1.md".to_string(), file_type: "md".to_string(), size: 200, modified: None },
        ];
        let content_folders = vec!["posts".to_string()];
        
        let result = detect_project_type_from_content(&files_with_collections, &content_folders);
        assert_eq!(result, ProjectType::HomepageWithCollections, "Should classify as homepage with collections");
        
        // Test 2: Simple flat site (≤5 root documents, no collections)
        let files_simple = vec![
            FileInfo { path: "about.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
            FileInfo { path: "contact.md".to_string(), file_type: "md".to_string(), size: 100, modified: None },
        ];
        let no_folders: Vec<String> = vec![];
        
        let result = detect_project_type_from_content(&files_simple, &no_folders);
        assert_eq!(result, ProjectType::SimpleFlatSite, "Should classify as simple flat site");
        
        // Test 3: Blog-style flat site (>5 root documents, no collections)
        let files_blog: Vec<FileInfo> = (1..=7).map(|i| FileInfo { 
            path: format!("post{}.md", i), 
            file_type: "md".to_string(), 
            size: 100,
            modified: None,
        }).collect();
        
        let result = detect_project_type_from_content(&files_blog, &no_folders);
        assert_eq!(result, ProjectType::BlogStyleFlatSite, "Should classify as blog-style flat site");
    }
}