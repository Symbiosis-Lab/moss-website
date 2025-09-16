//! Navigation generation module for moss static site generator
//! 
//! Handles the creation of navigation elements including:
//! - Main site navigation
//! - Breadcrumbs for article pages
//! - Sidebar content (latest posts, topics)

use crate::types::*;

/// Builder for generating navigation components
pub struct NavigationBuilder<'a> {
    documents: &'a [ParsedDocument],
    site_title: &'a str,
    depth: usize,
    current_page_url: Option<&'a str>,
}

impl<'a> NavigationBuilder<'a> {
    pub fn new(
        documents: &'a [ParsedDocument], 
        site_title: &'a str, 
        depth: usize, 
        current_page_url: Option<&'a str>
    ) -> Self {
        Self {
            documents,
            site_title,
            depth,
            current_page_url,
        }
    }

    /// Generates main navigation menu HTML with site name on left and items on right
    pub fn generate_main_navigation(&self) -> String {
        // Site name on the left - adjust home link based on depth
        let home_path = if self.depth == 0 { "/" } else { &"../".repeat(self.depth) };
        let site_name = format!(
            r#"<div class="nav-left"><a href="{}" class="site-name">{}</a></div>"#, 
            home_path, 
            self.site_title
        );
        
        // Navigation items on the right
        let path_prefix = "../".repeat(self.depth);
        let page_items: Vec<String> = self.documents.iter()
            .filter(|doc| !doc.url_path.starts_with("journal/") && doc.url_path != "index.html")
            .map(|doc| {
                let label = doc.display_title.clone();
                let href = if self.depth == 0 { 
                    doc.url_path.clone() 
                } else { 
                    format!("{}{}", path_prefix, doc.url_path)
                };
                
                // Add active class if this is the current page
                let class = if self.current_page_url.map_or(false, |url| url == doc.url_path) {
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

    /// Generates breadcrumb navigation for article pages
    pub fn generate_breadcrumb(&self, doc: &ParsedDocument) -> String {
        let path_parts: Vec<&str> = doc.url_path.split('/').collect();
        
        if path_parts.len() < 2 {
            return String::new(); // No breadcrumb for root pages
        }
        
        let folder_name = path_parts[0]; // e.g., "journal"
        let file_name = path_parts[1].strip_suffix(".html").unwrap_or(path_parts[1]);
        
        // Create folder link that goes back to parent directory
        let folder_link = if self.depth == 1 {
            "../".to_string() // From journal folder, go back to root
        } else {
            format!("{}index.html", "../".repeat(self.depth - 1))
        };
        
        format!(
            "<nav class=\"breadcrumb\"><a href=\"{}\">{}</a>/{}</nav>",
            folder_link, folder_name, file_name
        )
    }

    /// Generates latest section with only the most recent journal entry
    pub fn generate_latest_sidebar(&self, project: &ProjectStructure) -> String {
        let mut journal_entries: Vec<&ParsedDocument> = self.documents.iter()
            .filter(|doc| doc.url_path.starts_with("journal/"))
            .collect();
        
        // Sort by date (newest first)
        journal_entries.sort_by(|a, b| b.url_path.cmp(&a.url_path));
        
        if journal_entries.is_empty() {
            return "<p class=\"no-posts\">No posts yet</p>".to_string();
        }
        
        let items: Vec<String> = journal_entries.iter()
            .take(1)
            .map(|doc| {
                // Extract date from filename (e.g., "2025-01-15.html" → "2025 · 01")
                let date_display = extract_date_from_path(&doc.url_path, Some(&project.root_path));
                
                // Adjust journal link paths based on current depth
                let link_path = if self.depth == 0 {
                    // From root: use journal/filename.html
                    doc.url_path.clone()
                } else if self.depth == 1 && doc.url_path.starts_with("journal/") {
                    // From journal directory: use filename.html only
                    doc.url_path.strip_prefix("journal/").unwrap_or(&doc.url_path).to_string()
                } else {
                    // From other depths: go back to root then to journal
                    format!("{}{}", "../".repeat(self.depth), doc.url_path)
                };
                
                format!(
                    "<p>{} <a href=\"{}\">{}</a></p>", 
                    date_display, link_path, doc.display_title
                )
            })
            .collect();
        
        items.join("")
    }

    /// Generates inline topics section with comma-separated tags
    pub fn generate_topics_inline(&self) -> String {
        let mut all_topics: Vec<String> = Vec::new();
        
        for doc in self.documents {
            all_topics.extend(doc.topics.clone());
        }
        
        if all_topics.is_empty() {
            return String::new(); // Hide section if no topics
        }
        
        // Remove duplicates and sort
        all_topics.sort();
        all_topics.dedup();
        
        let topic_links: Vec<String> = all_topics.iter()
            .map(|topic| format!("<a href=\"topics/{}.html\">{}</a>", generate_slug(topic), topic))
            .collect();
        
        format!("<p class=\"topic-tags\">{}</p>", topic_links.join(", "))
    }
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

/// Converts a string to a URL-safe slug by replacing spaces with hyphens and converting to lowercase
pub fn generate_slug(text: &str) -> String {
    text.replace(" ", "-").to_lowercase()
}