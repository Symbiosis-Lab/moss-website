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
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tauri::Emitter;



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
/// preview server startup with detailed progress information. Optionally enables
/// file watching for live development mode.
///
/// # Arguments
/// * `app` - Tauri app handle for preview URL communication
/// * `folder_path` - Absolute path to the folder containing website content
/// * `auto_serve` - Whether to automatically start preview server after compilation (default: false)
/// * `watch` - Whether to start file watching for live development mode (default: false)
/// * `on_progress` - Channel for real-time progress updates
///
/// # Returns
/// * `Ok(String)` - Success message with compilation summary (and server info if started)
/// * `Err(String)` - Error message describing what went wrong
///
/// # File Watching Mode
/// When `watch = true`:
/// - Monitors source folder for content file changes (.md, .pages, .docx, images)
/// - Ignores generated files in .moss/ directory
/// - Performs silent recompilation on file changes (no progress updates)
/// - Automatically refreshes browser preview if server is running
#[tauri::command]
#[specta::specta]
pub async fn compile_folder(
    app: tauri::AppHandle,
    folder_path: String,
    auto_serve: Option<bool>,
    watch: Option<bool>,
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

        let final_message = format!("{}\n🌐 Preview server ready! Access at {}", base_message, preview_url);

        // Start file watching if requested
        println!("🔍 File watching parameter: {:?}", watch);
        if watch.unwrap_or(false) {
            println!("👁️ Starting file watching for: {}", folder_path);
            start_file_watching(app.clone(), folder_path.clone(), Some(port)).await;
            Ok(format!("{}\n👁️ Watching for file changes...", final_message))
        } else {
            println!("🚫 File watching disabled");
            Ok(final_message)
        }
    } else {
        send_progress("complete", "Compilation completed", 100, true, None);

        // Start file watching if requested (without server)
        println!("🔍 File watching parameter (no server): {:?}", watch);
        if watch.unwrap_or(false) {
            println!("👁️ Starting file watching (no server) for: {}", folder_path);
            start_file_watching(app.clone(), folder_path.clone(), None).await;
            Ok(format!("{}\n👁️ Watching for file changes...", base_message))
        } else {
            println!("🚫 File watching disabled (no server)");
            Ok(base_message)
        }
    }
}

/// Start file watching for live development mode.
///
/// Monitors the source folder for content file changes and triggers silent
/// recompilation for seamless preview updates.
///
/// # Arguments
/// * `app` - Tauri app handle for browser refresh integration
/// * `folder_path` - Source folder to watch for changes
/// * `server_port` - Optional preview server port for browser refresh
async fn start_file_watching(app: tauri::AppHandle, folder_path: String, server_port: Option<u16>) {
    use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::{channel, Receiver};

    tokio::spawn(async move {
        let (tx, rx): (std::sync::mpsc::Sender<notify::Result<Event>>, Receiver<notify::Result<Event>>) = channel();

        // Create file system watcher
        let mut watcher: RecommendedWatcher = match recommended_watcher(tx) {
            Ok(watcher) => watcher,
            Err(e) => {
                eprintln!("❌ Failed to create file watcher: {}", e);
                return;
            }
        };

        // Start watching the folder recursively
        if let Err(e) = watcher.watch(Path::new(&folder_path), RecursiveMode::Recursive) {
            eprintln!("❌ Failed to start watching folder '{}': {}", folder_path, e);
            return;
        }

        println!("👁️ File watcher started for: {}", folder_path);

        // Debouncing state - track last event time for each file
        let mut last_event_times: HashMap<String, Instant> = HashMap::new();
        let debounce_duration = Duration::from_millis(300);

        // Process file system events
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    if let Err(e) = handle_file_event(
                        &event,
                        &folder_path,
                        &mut last_event_times,
                        debounce_duration,
                        &app,
                        server_port
                    ).await {
                        eprintln!("❌ Error handling file event: {}", e);
                    }
                }
                Ok(Err(e)) => eprintln!("❌ File watcher error: {}", e),
                Err(e) => {
                    eprintln!("❌ File watcher channel error: {}", e);
                    break;
                }
            }
        }
    });
}

/// Handle individual file system events with debouncing and filtering
async fn handle_file_event(
    event: &notify::Event,
    folder_path: &str,
    last_event_times: &mut HashMap<String, Instant>,
    debounce_duration: Duration,
    app: &tauri::AppHandle,
    server_port: Option<u16>
) -> Result<(), String> {
    use notify::EventKind;

    // Track file changes for frontend notification
    let mut file_changes = FileChangeEvent::new();
    let mut should_recompile = false;

    // Process different event types
    match event.kind {
        EventKind::Modify(modify_kind) => {
            use notify::event::ModifyKind;
            match modify_kind {
                ModifyKind::Name(rename_mode) => {
                    // File renamed - track for frontend and trigger recompilation
                    should_recompile = true;
                    println!("🔄 File(s) renamed: {:?}", rename_mode);

                    // Handle different rename modes
                    use notify::event::RenameMode;
                    match rename_mode {
                        RenameMode::Both => {
                            // Both old and new paths in single event (order: from, to)
                            if event.paths.len() >= 2 {
                                let old_path = event.paths[0].to_string_lossy().to_string();
                                let new_path = event.paths[1].to_string_lossy().to_string();

                                if should_watch_file(&old_path) || should_watch_file(&new_path) {
                                    let old_relative = get_relative_path(folder_path, &old_path);
                                    let new_relative = get_relative_path(folder_path, &new_path);
                                    file_changes.add_renamed(old_relative, new_relative);
                                }
                            }
                        },
                        RenameMode::From => {
                            // This is the "from" path of a rename (old name)
                            for path in &event.paths {
                                let path_str = path.to_string_lossy().to_string();
                                if should_watch_file(&path_str) {
                                    let relative_path = get_relative_path(folder_path, &path_str);
                                    // For now, treat as deletion since we don't have the "to" path
                                    file_changes.add_deleted(relative_path);
                                }
                            }
                        },
                        RenameMode::To => {
                            // This is the "to" path of a rename (new name)
                            // Without the old path, we can't track the rename properly
                            println!("🔄 Rename 'to' event (partial): {:?}", event.paths);
                        },
                        _ => {
                            // Any or Other rename modes - handle as generic rename
                            println!("🔄 Rename event (unknown mode): {:?}", event.paths);

                            // For directories especially, any rename should trigger recompilation
                            // Process all paths and treat as modifications
                            for path in &event.paths {
                                let path_str = path.to_string_lossy().to_string();
                                if should_watch_file(&path_str) {
                                    println!("🔄 Processing rename for watched path: {}", path_str);
                                    // Don't try to track specific rename pairs for unknown modes
                                    // Just trigger recompilation which will rebuild the entire site
                                }
                            }
                        }
                    }
                },
                _ => {
                    // Other modify events (content, metadata changes)
                    should_recompile = true;
                    println!("📝 File(s) modified");
                }
            }
        },
        EventKind::Create(_) => {
            // New file created - trigger recompilation
            should_recompile = true;
            println!("📄 File(s) created");
        },
        EventKind::Remove(_) => {
            // File deleted - track for frontend and trigger recompilation
            should_recompile = true;
            println!("🗑️ File(s) deleted");

            // Add deleted paths to change event
            for path in &event.paths {
                let path_str = path.to_string_lossy().to_string();
                if should_watch_file(&path_str) {
                    // Convert to relative path from folder_path
                    let relative_path = get_relative_path(folder_path, &path_str);
                    file_changes.add_deleted(relative_path);
                }
            }
        },
        _ => {
            // Other events (access, metadata changes, etc.) - ignore
            return Ok(());
        }
    }

    if !should_recompile {
        return Ok(());
    }

    // Process each affected file path for debouncing
    for path in &event.paths {
        let path_str = path.to_string_lossy().to_string();

        // Filter out files we don't care about
        if !should_watch_file(&path_str) {
            continue;
        }

        // Implement debouncing - ignore events that are too close together
        let now = Instant::now();
        if let Some(last_time) = last_event_times.get(&path_str) {
            if now.duration_since(*last_time) < debounce_duration {
                continue; // Skip this event, too soon after last one
            }
        }

        // Update last event time
        last_event_times.insert(path_str.clone(), now);

        // Trigger silent recompilation
        if let Err(e) = silent_recompile(folder_path, app, server_port).await {
            eprintln!("❌ Silent recompilation failed: {}", e);
        }

        // Always emit file change event to frontend for any file modification
        if let Err(e) = app.emit("file-changed", &file_changes) {
            eprintln!("❌ Failed to emit file change event: {}", e);
        } else {
            if file_changes.has_changes() {
                println!("📡 Emitted file change event with special changes: {:?}", file_changes);
            } else {
                println!("📡 Emitted file change event for regular modification");
            }
        }

        // Only handle the first changed file to avoid multiple rebuilds
        break;
    }

    Ok(())
}

/// Check if a file or directory should trigger recompilation
fn should_watch_file(path: &str) -> bool {
    // Content file extensions we care about
    let content_extensions = ["md", "markdown", "pages", "docx", "jpg", "jpeg", "png", "gif", "svg"];

    // Paths to ignore
    let ignore_patterns = [".moss/", "node_modules/", ".git/", ".DS_Store", "thumbs.db"];

    // Check if file/directory should be ignored
    for ignore in &ignore_patterns {
        if path.contains(ignore) {
            println!("🚫 Ignoring path (matches ignore pattern '{}'): {}", ignore, path);
            return false;
        }
    }

    // Check if this is a directory (no extension or ends with /)
    let is_directory = path.ends_with('/') || !path.contains('.');

    if is_directory {
        // For directories, allow content directories but exclude system ones
        let content_directory_patterns = ["content/", "posts/", "journal/", "blog/", "articles/", "docs/", "pages/", "images/", "assets/", "media/"];

        for pattern in &content_directory_patterns {
            if path.contains(pattern) || path.ends_with(pattern.trim_end_matches('/')) {
                println!("✅ Watching directory (content directory): {}", path);
                return true;
            }
        }

        // For root-level directories without specific patterns, be permissive
        // (user might have custom directory structures)
        if !path.contains('/') || path.matches('/').count() <= 1 {
            println!("✅ Watching directory (root-level directory): {}", path);
            return true;
        }

        println!("🚫 Ignoring directory (not a content directory): {}", path);
        return false;
    }

    // Check file extension for files
    if let Some(extension) = path.split('.').last() {
        let should_watch = content_extensions.contains(&extension.to_lowercase().as_str());
        if should_watch {
            println!("✅ Watching file (content file): {}", path);
        } else {
            println!("🚫 Ignoring file (not a content file): {}", path);
        }
        should_watch
    } else {
        println!("🚫 Ignoring file (no extension): {}", path);
        false
    }
}

/// Convert absolute path to relative path from folder_path
fn get_relative_path(folder_path: &str, absolute_path: &str) -> String {
    // Normalize folder path to ensure it ends with separator
    let folder_path = Path::new(folder_path);
    let absolute_path = Path::new(absolute_path);

    // Try to strip the prefix (folder path) from the absolute path
    if let Ok(relative) = absolute_path.strip_prefix(folder_path) {
        relative.to_string_lossy().to_string()
    } else {
        // If we can't make it relative, just use the filename
        absolute_path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| absolute_path.to_string_lossy().to_string())
    }
}

/// Perform silent recompilation without progress updates
async fn silent_recompile(folder_path: &str, app: &tauri::AppHandle, server_port: Option<u16>) -> Result<(), String> {
    // Scan folder for updated content
    let project_structure = scan_folder(folder_path)?;

    // Silently regenerate static site
    generate_static_site(folder_path, &project_structure)?;

    println!("✅ Site regenerated silently");

    // If we have a server, trigger browser refresh
    if let Some(_port) = server_port {
        // Send refresh signal to frontend via Tauri event system
        if let Err(e) = app.emit("file-changed", ()) {
            eprintln!("❌ Failed to send refresh event: {}", e);
        }
    }

    Ok(())
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

    /// Feature 4: File Watching - File Filtering
    /// Tests which files should trigger recompilation
    #[test]
    fn test_file_watching_should_watch_file() {
        // Content files that should trigger recompilation
        assert!(should_watch_file("content/about.md"), "Should watch markdown files");
        assert!(should_watch_file("posts/article.pages"), "Should watch Pages files");
        assert!(should_watch_file("documents/report.docx"), "Should watch Word documents");
        assert!(should_watch_file("images/photo.jpg"), "Should watch image files");
        assert!(should_watch_file("assets/logo.png"), "Should watch PNG images");
        assert!(should_watch_file("styles/custom.svg"), "Should watch SVG files");

        // Generated/system files that should be ignored
        assert!(!should_watch_file(".moss/site/index.html"), "Should ignore generated files");
        assert!(!should_watch_file("node_modules/package/file.js"), "Should ignore node_modules");
        assert!(!should_watch_file(".git/config"), "Should ignore git files");
        assert!(!should_watch_file(".DS_Store"), "Should ignore system files");
        assert!(!should_watch_file("thumbs.db"), "Should ignore Windows thumbnails");

        // Files without recognized extensions
        assert!(!should_watch_file("README"), "Should ignore files without extensions");
        assert!(!should_watch_file("config.txt"), "Should ignore non-content files");
    }

    /// Feature 4b: Directory Watching - Directory Filtering
    /// Tests which directories should trigger recompilation
    #[test]
    fn test_file_watching_should_watch_directory() {
        // Content directories that should trigger recompilation
        assert!(should_watch_file("content/"), "Should watch content directories");
        assert!(should_watch_file("posts/"), "Should watch posts directory");
        assert!(should_watch_file("journal/"), "Should watch journal directory");
        assert!(should_watch_file("blog/"), "Should watch blog directory");
        assert!(should_watch_file("articles/"), "Should watch articles directory");
        assert!(should_watch_file("docs/"), "Should watch docs directory");
        assert!(should_watch_file("pages/"), "Should watch pages directory");
        assert!(should_watch_file("images/"), "Should watch images directory");
        assert!(should_watch_file("assets/"), "Should watch assets directory");
        assert!(should_watch_file("media/"), "Should watch media directory");

        // Root-level directories (permissive approach)
        assert!(should_watch_file("mycontent"), "Should watch root-level directories");
        assert!(should_watch_file("writings"), "Should watch root-level directories");

        // Nested content directories
        assert!(should_watch_file("/Users/user/site/posts/"), "Should watch nested posts directory");
        assert!(should_watch_file("/Users/user/site/content/blog/"), "Should watch nested content directories");

        // System directories that should be ignored
        assert!(!should_watch_file(".moss/"), "Should ignore .moss directory");
        assert!(!should_watch_file("node_modules/"), "Should ignore node_modules directory");
        assert!(!should_watch_file(".git/"), "Should ignore .git directory");
        assert!(!should_watch_file(".moss/site/"), "Should ignore generated .moss/site directory");

        // Deeply nested non-content directories
        assert!(!should_watch_file("deeply/nested/path/unknown/"), "Should ignore deeply nested unknown directories");
    }

    /// Feature 4c: Directory Rename Scenarios
    /// Tests rename events with directories
    #[test]
    fn test_directory_rename_scenarios() {
        // Test that directory paths without extensions are detected as directories
        let test_cases = vec![
            ("journal", true, "Root directory without slash"),
            ("posts/", true, "Directory with trailing slash"),
            ("content/subfolder", true, "Nested directory path"),
            ("/Users/user/site/blog", true, "Absolute directory path"),
            ("file.md", false, "File with extension"),
            ("README", true, "File without extension - treated as directory"),
        ];

        for (path, expected_is_dir, description) in test_cases {
            let is_directory = path.ends_with('/') || !path.contains('.');
            assert_eq!(is_directory, expected_is_dir, "Failed for case: {}", description);
        }
    }

    /// Feature 5: FileChangeEvent Creation and Processing
    /// Tests the FileChangeEvent struct and its methods
    #[test]
    fn test_file_change_event_creation() {
        // Test empty FileChangeEvent
        let empty_event = FileChangeEvent::new();
        assert!(!empty_event.has_changes(), "New FileChangeEvent should have no changes");
        assert!(empty_event.deleted_paths.is_none(), "Should have no deleted paths");
        assert!(empty_event.renamed_paths.is_none(), "Should have no renamed paths");

        // Test adding deleted path
        let mut event_with_deletion = FileChangeEvent::new();
        event_with_deletion.add_deleted("deleted/file.md".to_string());

        assert!(event_with_deletion.has_changes(), "Should have changes after adding deletion");
        assert_eq!(event_with_deletion.deleted_paths.as_ref().unwrap().len(), 1);
        assert_eq!(event_with_deletion.deleted_paths.as_ref().unwrap()[0], "deleted/file.md");

        // Test adding multiple deleted paths
        event_with_deletion.add_deleted("another/file.md".to_string());
        assert_eq!(event_with_deletion.deleted_paths.as_ref().unwrap().len(), 2);

        // Test adding renamed path
        let mut event_with_rename = FileChangeEvent::new();
        event_with_rename.add_renamed("old/path.md".to_string(), "new/path.md".to_string());

        assert!(event_with_rename.has_changes(), "Should have changes after adding rename");
        assert_eq!(event_with_rename.renamed_paths.as_ref().unwrap().len(), 1);
        assert_eq!(event_with_rename.renamed_paths.as_ref().unwrap()[0], ("old/path.md".to_string(), "new/path.md".to_string()));

        // Test adding multiple renamed paths
        event_with_rename.add_renamed("old2.md".to_string(), "new2.md".to_string());
        assert_eq!(event_with_rename.renamed_paths.as_ref().unwrap().len(), 2);

        // Test combined changes
        let mut combined_event = FileChangeEvent::new();
        combined_event.add_deleted("deleted.md".to_string());
        combined_event.add_renamed("old.md".to_string(), "new.md".to_string());

        assert!(combined_event.has_changes(), "Should have changes with both deletions and renames");
        assert!(combined_event.deleted_paths.is_some());
        assert!(combined_event.renamed_paths.is_some());
    }

    /// Feature 6: Path Processing for File Events
    /// Tests relative path conversion and processing
    #[test]
    fn test_get_relative_path() {
        // Test normal relative path conversion
        let folder_path = "/Users/test/project";
        let absolute_path = "/Users/test/project/content/blog.md";

        let result = get_relative_path(folder_path, absolute_path);
        assert_eq!(result, "content/blog.md");

        // Test with trailing slash
        let folder_with_slash = "/Users/test/project/";
        let result_with_slash = get_relative_path(folder_with_slash, absolute_path);
        // Should work the same way
        assert!(result_with_slash.contains("blog.md"));

        // Test with file in root
        let root_file = "/Users/test/project/index.md";
        let result_root = get_relative_path(folder_path, root_file);
        assert_eq!(result_root, "index.md");

        // Test with path that can't be made relative (fallback to filename)
        let unrelated_path = "/completely/different/path/file.md";
        let result_fallback = get_relative_path(folder_path, unrelated_path);
        assert_eq!(result_fallback, "file.md");

        // Test with path that has no filename (fallback to directory name)
        let no_filename_path = "/some/directory/";
        let result_no_filename = get_relative_path(folder_path, no_filename_path);
        assert_eq!(result_no_filename, "directory");
    }

    /// Feature 7: File Event Processing Logic
    /// Tests that different event types are processed correctly
    #[test]
    fn test_file_event_classification() {
        use notify::{EventKind, event::{ModifyKind, CreateKind, RemoveKind}};

        // Test that Modify events are classified correctly
        let modify_event = EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any));
        match modify_event {
            EventKind::Modify(ModifyKind::Name(_)) => panic!("Should not be a name modify"),
            EventKind::Modify(_) => {}, // Expected path
            _ => panic!("Should be a modify event"),
        }

        // Test that Create events are classified correctly
        let create_event = EventKind::Create(CreateKind::File);
        match create_event {
            EventKind::Create(_) => {}, // Expected
            _ => panic!("Should be a create event"),
        }

        // Test that Remove events are classified correctly
        let remove_event = EventKind::Remove(RemoveKind::File);
        match remove_event {
            EventKind::Remove(_) => {}, // Expected
            _ => panic!("Should be a remove event"),
        }

        // Test that Rename events (ModifyKind::Name) are classified correctly
        let rename_event = EventKind::Modify(ModifyKind::Name(notify::event::RenameMode::Both));
        match rename_event {
            EventKind::Modify(ModifyKind::Name(_)) => {}, // Expected path for renames
            _ => panic!("Should be a name modify event"),
        }
    }
}