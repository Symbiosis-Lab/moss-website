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

/// Converts a string to a URL-safe slug by:
/// - Converting to lowercase
/// - Replacing spaces and underscores with hyphens
/// - Removing or replacing special characters
/// - Removing consecutive hyphens
/// - Trimming hyphens from start/end
pub fn generate_slug(text: &str) -> String {
    let result = text.to_lowercase()
        // Replace spaces and underscores with hyphens
        .replace([' ', '_'], "-")
        // Replace common special characters with safe alternatives
        .replace("&", "and")
        .replace("@", "at")
        .replace("+", "plus")
        .replace("#", "hash")
        .replace("%", "percent")
        // Keep dots for numbers, but remove other special chars by filtering
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '.' { c } else { '-' })
        .collect::<String>()
        // Remove consecutive hyphens and clean up
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
        .trim_matches('-')
        .chars()
        .take(100) // Limit length to avoid extremely long URLs
        .collect::<String>()
        .trim_end_matches('-')
        .to_string();

    // Fallback for empty results
    if result.is_empty() {
        "untitled".to_string()
    } else {
        result
    }
}

/// Extension trait to provide if_empty_then method
trait StringExt {
    fn if_empty_then(self, fallback: String) -> String;
}

impl StringExt for String {
    fn if_empty_then(self, fallback: String) -> String {
        if self.is_empty() { fallback } else { self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug_basic() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
        assert_eq!(generate_slug("My Blog Post"), "my-blog-post");
    }

    #[test]
    fn test_generate_slug_special_characters() {
        assert_eq!(generate_slug("API & Documentation"), "api-and-documentation");
        assert_eq!(generate_slug("Contact @ Email"), "contact-at-email");
        assert_eq!(generate_slug("C++ Programming"), "cplusplus-programming");
        assert_eq!(generate_slug("50% Off"), "50percent-off");
        assert_eq!(generate_slug("FAQ #1"), "faq-hash1");
    }

    #[test]
    fn test_generate_slug_underscores() {
        assert_eq!(generate_slug("file_name_example"), "file-name-example");
        assert_eq!(generate_slug("snake_case_title"), "snake-case-title");
    }

    #[test]
    fn test_generate_slug_remove_invalid_chars() {
        assert_eq!(generate_slug("Title (with) brackets"), "title-with-brackets");
        assert_eq!(generate_slug("Price: $99.99"), "price-99.99");
        assert_eq!(generate_slug("User/Admin/Settings"), "user-admin-settings");
    }

    #[test]
    fn test_generate_slug_consecutive_hyphens() {
        assert_eq!(generate_slug("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(generate_slug("Too---Many-Hyphens"), "too-many-hyphens");
        assert_eq!(generate_slug("Mixed _-_ Separators"), "mixed-separators");
    }

    #[test]
    fn test_generate_slug_edge_cases() {
        assert_eq!(generate_slug(""), "untitled");
        assert_eq!(generate_slug("   "), "untitled");
        assert_eq!(generate_slug("---"), "untitled");
        assert_eq!(generate_slug("$@%!"), "atpercent");
    }

    #[test]
    fn test_generate_slug_unicode() {
        // Unicode characters should be removed, leaving English alphanumeric
        assert_eq!(generate_slug("Café Menu"), "caf-menu");
        assert_eq!(generate_slug("Résumé Template"), "r-sum-template");
    }

    #[test]
    fn test_generate_slug_length_limit() {
        let very_long = "a".repeat(200);
        let result = generate_slug(&very_long);
        assert!(result.len() <= 100);
        assert_eq!(result, "a".repeat(100));
    }

    #[test]
    fn test_generate_slug_trim_hyphens() {
        assert_eq!(generate_slug("-leading hyphen"), "leading-hyphen");
        assert_eq!(generate_slug("trailing hyphen-"), "trailing-hyphen");
        assert_eq!(generate_slug("-both-"), "both");
    }
}