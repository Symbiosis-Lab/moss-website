//! Static site generation module for moss compilation system
//! 
//! Handles the conversion of markdown files and other content into complete
//! static websites with beautiful default styling, navigation, and metadata.
//! 
//! # Key Features
//! - Markdown to HTML conversion with frontmatter support
//! - Responsive typography-first design
//! - Dark mode support
//! - Content organization and navigation generation
//! - Image asset copying
//! - Journal/blog feed generation

use crate::types::*;
use std::path::Path;
use std::fs;
use pulldown_cmark::{Parser, Options, html};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::{Deserialize, Serialize};

/// Frontmatter structure for parsing YAML metadata from markdown files
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct FrontMatter {
    /// Optional title override from frontmatter
    pub title: Option<String>,
    /// Optional publication date
    pub date: Option<String>,
    /// Topics or tags for categorization
    pub topics: Option<Vec<String>>,
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
pub fn generate_static_site(source_path: &str, project_structure: &ProjectStructure) -> Result<SiteResult, String> {
    
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
        
        let html_page = generate_html(Some(doc), &documents, project_structure, false)?;
        fs::write(&output_file_path, html_page).map_err(|e| format!("Failed to write HTML file: {}", e))?;
        page_count += 1;
    }
    
    // Generate CSS
    let css_content = DEFAULT_CSS;
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
            // Find the homepage document
            let homepage_doc = documents.iter().find(|d| d.url_path == "index.html");
            let index_html = generate_html(homepage_doc, &documents, project_structure, true)?;
            fs::write(output_dir.join("index.html"), index_html).map_err(|e| format!("Failed to write index.html: {}", e))?;
            page_count += 1;
        } else {
            // No homepage file - generate pure blog feed
            let index_html = generate_html(None, &documents, project_structure, false)?;
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
pub fn process_markdown_file(file_path: &str, content: &str) -> Result<ParsedDocument, String> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(content);
    
    // Extract title - for index files, prefer H1 content over filename
    let filename_title = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .replace("-", " ")
        .replace("_", " ");
    
    // Convert markdown to HTML first to extract H1
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    
    let parser = Parser::new_ext(&result.content, options);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);
    
    // Determine final title based on file type
    let title = if is_index_file(file_path) {
        // For index files, prefer H1 content over filename
        extract_first_heading(&html_content).unwrap_or(filename_title)
    } else {
        // For regular files, use filename (could extend this for frontmatter later)
        filename_title
    };
    
    // Generate URL path
    let url_path = if file_path.to_lowercase() == "index.md" || file_path.to_lowercase() == "readme.md" {
        "index.html".to_string()
    } else {
        file_path.replace(".md", ".html").replace(".markdown", ".html")
    };
    
    // Parse frontmatter if present
    let frontmatter: FrontMatter = if let Some(ref matter_data) = result.data {
        // Try to deserialize directly from the Pod
        matter_data.deserialize().unwrap_or_default()
    } else {
        FrontMatter::default()
    };
    
    // Extract date from frontmatter
    let date = frontmatter.date;
    
    // Extract topics/categories from frontmatter
    let topics = frontmatter.topics.unwrap_or_default();
    
    // Calculate reading time (200 words per minute)
    let word_count = result.content.split_whitespace().count();
    let reading_time = std::cmp::max(1, (word_count / 200) as u32);
    
    // Generate excerpt from content
    let excerpt = extract_excerpt(&html_content);
    
    Ok(ParsedDocument {
        title,
        content: result.content,
        html_content,
        url_path,
        date,
        topics,
        reading_time,
        excerpt,
    })
}

/// Unified HTML generation function for all page types.
pub fn generate_html(
    doc: Option<&ParsedDocument>, 
    all_docs: &[ParsedDocument], 
    project: &ProjectStructure,
    is_homepage: bool
) -> Result<String, String> {
    // Get site title for navigation
    let site_title = project.homepage_file.as_ref()
        .and_then(|_| all_docs.iter().find(|d| d.url_path == "index.html"))
        .map(|d| d.title.clone())
        .unwrap_or_else(|| "Site".to_string());

    // Calculate depth for path adjustments
    let depth = doc.map(|d| d.url_path.matches('/').count()).unwrap_or(0);
    
    // Generate navigation and sidebar with depth awareness
    let current_page_url = doc.map(|d| d.url_path.as_str());
    let navigation = generate_navigation(all_docs, &site_title, depth, current_page_url);
    let latest_list = generate_latest_sidebar(all_docs, project, depth);
    
    // Determine CSS path based on document depth
    let css_path = if depth == 0 {
        "style.css".to_string()
    } else {
        "../".repeat(depth) + "style.css"
    };
    
    // Prepare content based on page type
    let (page_title, homepage_content) = match (doc, is_homepage) {
        (Some(doc), true) => {
            // Homepage with document (generate_homepage_with_blog_feed equivalent)
            let mut content = if let Some(start) = doc.html_content.find("<article>") {
                if let Some(end) = doc.html_content.rfind("</article>") {
                    doc.html_content[start + 9..end].to_string() // +9 to skip "<article>"
                } else {
                    doc.html_content.clone()
                }
            } else {
                doc.html_content.clone()
            };
            
            // Remove the first H1 heading only if it matches the document title (to avoid duplication)
            if let Some(_h1_start) = content.find("<h1>") {
                if let Some(h1_end) = content.find("</h1>") {
                    let h1_content = &content[_h1_start + 4..h1_end]; // +4 to skip "<h1>"
                    if h1_content.trim() == doc.title.trim() {
                        content = content[h1_end + 5..].to_string(); // +5 to skip "</h1>"
                    }
                }
            }
            
            (doc.title.clone(), content)
        },
        (Some(doc), false) => {
            // Regular page - use content as-is without adding filename title
            (doc.title.clone(), doc.html_content.clone())
        },
        (None, false) => {
            // Pure blog index page (generate_index_page equivalent)
            let content_table = generate_content_table(all_docs);
            let content = format!(r#"<h1>All Posts</h1>
{}"#, content_table);
            (site_title.clone(), content)
        },
        (None, true) => {
            return Err("Cannot generate homepage without document".to_string());
        }
    };

    // Generate topics section for homepage
    let topics_section = if is_homepage && doc.is_some() {
        let topics_inline = generate_topics_inline(all_docs);
        if topics_inline.is_empty() {
            String::new()
        } else {
            format!(r#"
            <section class="topics">
                <h3>Topics</h3>
                {}
            </section>"#, topics_inline)
        }
    } else {
        String::new()
    };
    
    Ok(PAGE_TEMPLATE
        .replace("{title}", &page_title)
        .replace("{css_path}", &css_path)
        .replace("{navigation}", &navigation)
        .replace("{homepage_content}", &homepage_content)
        .replace("{latest_list}", &latest_list)
        .replace("{topics_section}", &topics_section))
}

/// Generates minimal sidebar with latest 10 journal entries formatted as stephago.com.
pub fn generate_latest_sidebar(documents: &[ParsedDocument], project: &ProjectStructure, depth: usize) -> String {
    let mut journal_entries: Vec<&ParsedDocument> = documents.iter()
        .filter(|doc| doc.url_path.starts_with("journal/"))
        .collect();
    
    // Sort by date (newest first)
    journal_entries.sort_by(|a, b| b.url_path.cmp(&a.url_path));
    
    if journal_entries.is_empty() {
        return "<p class=\"no-posts\">No posts yet</p>".to_string();
    }
    
    let items: Vec<String> = journal_entries.iter()
        .take(10)
        .map(|doc| {
            // Extract date from filename (e.g., "2025-01-15.html" → "2025 · 01")
            let date_display = extract_date_from_path(&doc.url_path, Some(&project.root_path));
            
            let display_title = if doc.title.replace(" ", "-").to_lowercase() 
                == doc.url_path.strip_prefix("journal/").unwrap_or("").strip_suffix(".html").unwrap_or("").to_lowercase() {
                extract_first_heading(&doc.html_content).unwrap_or(doc.title.clone())
            } else {
                doc.title.clone()
            };
            
            // Adjust journal link paths based on current depth
            let link_path = if depth == 0 {
                // From root: use journal/filename.html
                doc.url_path.clone()
            } else if depth == 1 && doc.url_path.starts_with("journal/") {
                // From journal directory: use filename.html only
                doc.url_path.strip_prefix("journal/").unwrap_or(&doc.url_path).to_string()
            } else {
                // From other depths: go back to root then to journal
                format!("{}{}", "../".repeat(depth), doc.url_path)
            };
            
            format!(
                "<li><span class=\"entry-date\">{}</span><a href=\"{}\">{}</a></li>", 
                date_display, link_path, display_title
            )
        })
        .collect();
    
    format!("<ul class=\"latest-list\">{}</ul>", items.join(""))
}

/// Checks if a file path represents an index/homepage file
pub fn is_index_file(file_path: &str) -> bool {
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    matches!(file_name.as_str(), 
        "readme.md" | "index.md" | "home.md" | 
        "readme.markdown" | "index.markdown" | "home.markdown"
    )
}

/// Extract date from file path and format as "YYYY · MM"
pub fn extract_date_from_path(url_path: &str, root_path: Option<&str>) -> String {
    // Try to extract date from pattern like "journal/2025-01-15.html"
    if let Some(filename) = url_path.strip_prefix("journal/") {
        if let Some(date_part) = filename.strip_suffix(".html") {
            // Parse YYYY-MM-DD pattern
            let parts: Vec<&str> = date_part.split('-').collect();
            if parts.len() >= 2 {
                let year = parts[0];
                let month = parts[1];
                return format!("{} · {}", year, month);
            }
        }
    }
    
    // Fallback: try to get file creation date from filesystem
    if let Some(root) = root_path {
        let file_path = std::path::Path::new(root).join(url_path.replace(".html", ".md"));
        if let Ok(metadata) = std::fs::metadata(&file_path) {
            if let Ok(created) = metadata.created() {
                if let Ok(datetime) = created.duration_since(std::time::UNIX_EPOCH) {
                    let secs = datetime.as_secs();
                    // Convert to naive datetime (UTC)
                    if let Some(dt) = chrono::DateTime::from_timestamp(secs as i64, 0).map(|dt| dt.naive_utc()) {
                        let month = dt.format("%m").to_string();
                        return format!("{} · {}", dt.format("%Y"), month);
                    }
                }
            }
        }
    }
    
    "Unknown".to_string()
}

/// Generates inline topics section with comma-separated tags.
pub fn generate_topics_inline(documents: &[ParsedDocument]) -> String {
    let mut all_topics: Vec<String> = Vec::new();
    
    for doc in documents {
        all_topics.extend(doc.topics.clone());
    }
    
    if all_topics.is_empty() {
        return String::new(); // Hide section if no topics
    }
    
    // Remove duplicates and sort
    all_topics.sort();
    all_topics.dedup();
    
    let topic_links: Vec<String> = all_topics.iter()
        .map(|topic| format!("<a href=\"#{}\">{}</a>", topic, topic))
        .collect();
    
    format!("<p class=\"topic-tags\">{}</p>", topic_links.join(", "))
}

/// Generates content table showing all posts with metadata.
pub fn generate_content_table(documents: &[ParsedDocument]) -> String {
    let mut table_rows = Vec::new();
    
    // Sort documents by URL path (reverse for newest first)
    let mut sorted_docs: Vec<&ParsedDocument> = documents.iter()
        .filter(|doc| !doc.url_path.starts_with("journal/") || doc.url_path != "index.html")
        .collect();
    sorted_docs.sort_by(|a, b| b.url_path.cmp(&a.url_path));
    
    for doc in sorted_docs {
        let topics_str = if doc.topics.is_empty() {
            "-".to_string()
        } else {
            doc.topics.join(", ")
        };
        
        let date_str = doc.date.as_ref()
            .map(|d| format_date(d))
            .unwrap_or_else(|| "-".to_string());
        
        table_rows.push(format!(
            "<tr><td><a href=\"{}\">{}</a></td><td>{}</td><td>{}</td><td>{} min</td></tr>",
            doc.url_path, doc.title, date_str, topics_str, doc.reading_time
        ));
    }
    
    if table_rows.is_empty() {
        "<p>No posts yet.</p>".to_string()
    } else {
        format!(
            "<table class=\"content-table\">
                <thead>
                    <tr><th>Title</th><th>Date</th><th>Topics</th><th>Reading Time</th></tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>",
            table_rows.join("\n")
        )
    }
}

/// Generates navigation menu HTML with site name on left and items on right.
pub fn generate_navigation(documents: &[ParsedDocument], site_title: &str, depth: usize, current_page_url: Option<&str>) -> String {
    // Site name on the left - adjust home link based on depth
    let home_path = if depth == 0 { "/" } else { &"../".repeat(depth) };
    let site_name = format!(r#"<div class="nav-left"><a href="{}" class="site-name">{}</a></div>"#, home_path, site_title);
    
    // Navigation items on the right
    let path_prefix = "../".repeat(depth);
    let page_items: Vec<String> = documents.iter()
        .filter(|doc| !doc.url_path.starts_with("journal/") && doc.url_path != "index.html") // Exclude journal entries and homepage from navigation
        .map(|doc| {
            let label = doc.title.clone();
            let href = if depth == 0 { 
                doc.url_path.clone() 
            } else { 
                format!("{}{}", path_prefix, doc.url_path)
            };
            
            // Add active class if this is the current page
            let class = if current_page_url.map_or(false, |url| url == doc.url_path) {
                r#" class="active""#
            } else {
                ""
            };
            
            format!(r#"<a href="{}"{class}>{}</a>"#, href, label)
        })
        .collect();
    
    // Add dark mode toggle
    let mut nav_right_items = page_items;
    nav_right_items.push(r#"<button class="theme-toggle" onclick="toggleTheme()" aria-label="Toggle dark mode">◐</button>"#.to_string());
    
    let nav_right = format!(r#"<div class="nav-right">{}</div>"#, nav_right_items.join(""));
    
    format!("{}{}", site_name, nav_right)
}

/// Extracts an excerpt from HTML content (first paragraph or first 200 characters).
pub fn extract_excerpt(html_content: &str) -> String {
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
pub fn extract_first_heading(html_content: &str) -> Option<String> {
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
pub fn format_date(date_str: &str) -> String {
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

/// Enhanced CSS styling optimized for Writers & Publishers.
/// 
/// Typography-first design system with:
/// - 18px base font size for comfortable long-form reading
/// - 65ch optimal line length for sustained reading
/// - 1.7 line-height for enhanced readability
/// - Warm, paper-inspired color palette reducing eye strain
/// - CSS custom properties for maintainability
/// - Dark mode support with consistent color relationships
/// - Mobile-responsive design with appropriate font scaling
/// 
/// IMPORTANT: This CSS is embedded at compile time using include_str!
/// 
/// This embedded CSS is used for generating static sites, not for preview windows.
/// Preview windows load from the development server at localhost:8080, which serves
/// fresh CSS files from disk.
/// 
/// The embedded CSS ensures consistent styling in generated static sites across
/// different environments. For reliable rebuilds when CSS changes, see build.rs 
/// which uses rerun-if-changed to track asset dependencies.
const DEFAULT_CSS: &str = include_str!("../assets/default.css");
const PAGE_TEMPLATE: &str = include_str!("../assets/templates/index.html");

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_is_index_file() {
        // Test index files
        assert!(is_index_file("index.md"));
        assert!(is_index_file("README.md"));
        assert!(is_index_file("home.md"));
        assert!(is_index_file("index.markdown"));
        
        // Test non-index files
        assert!(!is_index_file("about.md"));
        assert!(!is_index_file("blog-post.md"));
        assert!(!is_index_file("index.txt"));
    }

    #[test]
    fn test_extract_date_from_path() {
        // Test journal path extraction
        let date = extract_date_from_path("journal/2025-01-15.html", None);
        assert_eq!(date, "2025 · 01");
        
        let date = extract_date_from_path("journal/2024-12-31.html", None);
        assert_eq!(date, "2024 · 12");
        
        // Test non-journal path
        let date = extract_date_from_path("about.html", None);
        assert_eq!(date, "Unknown");
        
        // Test invalid journal path (function extracts first two parts separated by dashes)
        let date = extract_date_from_path("journal/invalid-name.html", None);
        assert_eq!(date, "invalid · name");
        
        // Test completely invalid journal path (only one part, needs at least 2)
        let date = extract_date_from_path("journal/nodashes.html", None);
        assert_eq!(date, "Unknown");
    }
}