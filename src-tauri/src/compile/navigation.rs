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
    github_url: Option<&'a str>,
    content_folders: &'a [String],
}

impl<'a> NavigationBuilder<'a> {
    pub fn new(
        documents: &'a [ParsedDocument],
        site_title: &'a str,
        depth: usize,
        current_page_url: Option<&'a str>,
        github_url: Option<&'a str>,
        content_folders: &'a [String],
    ) -> Self {
        Self {
            documents,
            site_title,
            depth,
            current_page_url,
            github_url,
            content_folders,
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
        
        // Navigation items on the right, sorted by weight
        let path_prefix = "../".repeat(self.depth);

        // Collect and sort documents by weight (lower numbers first), then by title
        // Filter out collection documents and index page
        let mut nav_documents: Vec<&ParsedDocument> = self.documents.iter()
            .filter(|doc| {
                // Exclude index page
                if doc.url_path == "index.html" {
                    return false;
                }

                // Exclude documents from any detected content collection folders
                for folder in self.content_folders {
                    if doc.url_path.starts_with(&format!("{}/", folder)) {
                        return false;
                    }
                }

                true
            })
            .collect();

        nav_documents.sort_by(|a, b| {
            match (a.weight, b.weight) {
                (Some(a_weight), Some(b_weight)) => a_weight.cmp(&b_weight),
                (Some(_), None) => std::cmp::Ordering::Less,    // Weighted items first
                (None, Some(_)) => std::cmp::Ordering::Greater, // Unweighted items last
                (None, None) => a.display_title.cmp(&b.display_title), // Alphabetical fallback
            }
        });

        let page_items: Vec<String> = nav_documents.iter()
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
        
        // Build navigation structure: [hamburger] [page links] [icons]

        // Hamburger menu button (mobile only)
        let hamburger = r#"<button class="mobile-menu-button" onclick="toggleMobileMenu()" aria-label="Toggle menu"><svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="18" x2="21" y2="18"/></svg></button>"#;

        // Page links container (collapsible on mobile)
        let nav_links = if page_items.is_empty() {
            String::new()
        } else {
            format!(r#"<div class="nav-links">{}</div>"#, page_items.join(""))
        };

        // Icons container (always visible)
        let mut icon_items = Vec::new();

        // Add vertical separator before icons
        icon_items.push(r#"<span class="nav-divider"></span>"#.to_string());

        // Add theme toggle with sun/moon icons
        icon_items.push(r#"<button class="theme-toggle" onclick="toggleTheme()" aria-label="Toggle dark mode"><svg class="sun-icon" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg><svg class="moon-icon" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg></button>"#.to_string());

        // Add GitHub link if available
        if let Some(github_url) = self.github_url {
            icon_items.push(format!(
                r#"<a href="{}" class="github-link" aria-label="GitHub Repository" target="_blank" rel="noopener"><svg viewBox="0 0 16 16" width="20" height="20" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/></svg></a>"#,
                github_url
            ));
        }

        let nav_icons = format!(r#"<div class="nav-icons">{}</div>"#, icon_items.join(""));

        let nav_right = format!(r#"<div class="nav-right">{}{}{}</div>"#, hamburger, nav_links, nav_icons);
        
        format!("{}{}", site_name, nav_right)
    }

    /// Generates breadcrumb navigation for article pages
    pub fn generate_breadcrumb(&self, doc: &ParsedDocument) -> String {
        let path_parts: Vec<&str> = doc.url_path.split('/').collect();
        
        if path_parts.len() < 2 {
            return String::new(); // No breadcrumb for root pages
        }
        
        let folder_name = path_parts[0]; // e.g., "journal"
        let article_title = &doc.display_title; // Use actual article title
        
        // Create folder link that points to collection index
        let folder_link = if self.depth == 1 {
            "./".to_string() // Link to collection index (e.g., journal/index.html)
        } else {
            format!("{}index.html", "../".repeat(self.depth - 1))
        };
        
        format!(
            "<nav class=\"breadcrumb\"><a href=\"{}\">{}</a> / {}</nav>",
            folder_link, folder_name, article_title
        )
    }

    /// Generates latest section with the most recent entry from any content collection
    pub fn generate_latest_sidebar(&self, project: &ProjectStructure) -> String {
        let mut content_entries: Vec<&ParsedDocument> = self.documents.iter()
            .filter(|doc| {
                // Include documents from any detected content collection folder
                for folder in &project.content_folders {
                    if doc.url_path.starts_with(&format!("{}/", folder)) {
                        return true;
                    }
                }
                false
            })
            .collect();

        // Sort by date (newest first) - use actual dates when available, fallback to URL path
        content_entries.sort_by(|a, b| {
            match (&a.date, &b.date) {
                (Some(date_a), Some(date_b)) => date_b.cmp(date_a), // Newer dates first
                (Some(_), None) => std::cmp::Ordering::Less,        // Dated entries first
                (None, Some(_)) => std::cmp::Ordering::Greater,     // Undated entries last
                (None, None) => b.url_path.cmp(&a.url_path),        // Fallback to URL path
            }
        });

        if content_entries.is_empty() {
            return "<p class=\"no-posts\">No posts yet</p>".to_string();
        }

        let items: Vec<String> = content_entries.iter()
            .take(1)
            .map(|doc| {
                // Extract date from frontmatter or filename
                let date_display = extract_date_from_doc(doc, project);

                // Adjust link paths based on current depth
                let link_path = if self.depth == 0 {
                    // From root: use full path
                    doc.url_path.clone()
                } else {
                    // Determine if we're in the same collection directory
                    let doc_folder = doc.url_path.split('/').next().unwrap_or("");
                    if self.depth == 1 && self.content_folders.contains(&doc_folder.to_string()) {
                        // From collection directory: use filename only
                        doc.url_path.split('/').last().unwrap_or(&doc.url_path).to_string()
                    } else {
                        // From other depths: go back to root then to full path
                        format!("{}{}", "../".repeat(self.depth), doc.url_path)
                    }
                };

                format!(
                    "<p><span class=\"date\">{}</span>&nbsp;&nbsp;<a href=\"{}\" style=\"text-decoration: underline; color: var(--moss-text-primary);\">{}</a></p>",
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

/// Generates breadcrumb for collection index pages
pub fn generate_collection_breadcrumb(collection_name: &str) -> String {
    format!("<nav class=\"breadcrumb\" style=\"font-size: 1.1em; margin-bottom: 1.5em;\">{}</nav>", collection_name)
}

/// Extract date from file path and format as "YYYY · MM"
pub fn extract_date_from_path(url_path: &str, root_path: Option<&str>, content_folders: &[String]) -> String {
    // Try to extract date from pattern like "posts/2025-01-15.html" for any content folder
    for folder in content_folders {
        let prefix = format!("{}/", folder);
        if let Some(filename) = url_path.strip_prefix(&prefix) {
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

/// Extract date from document preferring frontmatter over filename
/// Returns formatted date as "YYYY · MM"
pub fn extract_date_from_doc(doc: &ParsedDocument, project: &ProjectStructure) -> String {
    // First priority: Use frontmatter date if available
    if let Some(frontmatter_date) = &doc.date {
        return format_date_string(frontmatter_date);
    }

    // Fallback: Use existing filename-based extraction
    extract_date_from_path(&doc.url_path, Some(&project.root_path), &project.content_folders)
}

/// Format various date string formats to "YYYY · MM"
pub fn format_date_string(date_str: &str) -> String {
    // Handle common date formats: YYYY-MM-DD, YYYY/MM/DD, YYYY-MM, etc.
    let cleaned = date_str.trim().replace('/', "-");

    // Try to parse YYYY-MM-DD or YYYY-MM pattern
    let parts: Vec<&str> = cleaned.split('-').collect();
    if parts.len() >= 2 {
        let year = parts[0];
        let month = parts[1];

        // Validate year and month are numeric
        if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) &&
           month.len() <= 2 && month.chars().all(|c| c.is_ascii_digit()) {
            // Pad month to 2 digits if needed
            let month_padded = format!("{:0>2}", month);
            return format!("{} · {}", year, month_padded);
        }
    }

    // If parsing fails, return as-is
    date_str.to_string()
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

    /// Navigation Collection Filtering Tests
    /// Tests that collection documents are properly excluded from main navigation
    #[test]
    fn test_navigation_collection_filtering() {
        use crate::types::ParsedDocument;

        // Setup test documents
        let documents = vec![
            ParsedDocument {
                url_path: "about.html".to_string(),
                title: "About".to_string(),
                display_title: "About".to_string(),
                content: "About content".to_string(),
                html_content: "<p>About content</p>".to_string(),
                date: None,
                topics: vec![],
                reading_time: 1,
                excerpt: "About excerpt".to_string(),
                slug: "about".to_string(),
                permalink: "/about.html".to_string(),
                weight: Some(1),
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "posts/first-post.html".to_string(),
                title: "First Post".to_string(),
                display_title: "First Post".to_string(),
                content: "Post content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "posts/second-post.html".to_string(),
                title: "Second Post".to_string(),
                display_title: "Second Post".to_string(),
                content: "Post content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "contact.html".to_string(),
                title: "Contact".to_string(),
                display_title: "Contact".to_string(),
                content: "Contact content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: Some(2),
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "index.html".to_string(),
                title: "Home".to_string(),
                display_title: "Home".to_string(),
                content: "Home content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
        ];

        // Test with posts collection
        let content_folders = vec!["posts".to_string()];
        let nav_builder = NavigationBuilder::new(
            &documents,
            "Test Site",
            0,
            None,
            None,
            &content_folders,
        );

        let navigation_html = nav_builder.generate_main_navigation();

        // Should include root-level pages
        assert!(navigation_html.contains("About"), "Should include About page");
        assert!(navigation_html.contains("Contact"), "Should include Contact page");

        // Should exclude collection posts
        assert!(!navigation_html.contains("First Post"), "Should exclude posts from collection");
        assert!(!navigation_html.contains("Second Post"), "Should exclude posts from collection");

        // Should exclude index page
        assert!(!navigation_html.contains("Home"), "Should exclude index page");
    }

    #[test]
    fn test_navigation_multiple_collections() {
        use crate::types::ParsedDocument;

        let documents = vec![
            ParsedDocument {
                url_path: "services.html".to_string(),
                title: "Services".to_string(),
                display_title: "Services".to_string(),
                content: "Services content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "blog/article.html".to_string(),
                title: "Blog Article".to_string(),
                display_title: "Blog Article".to_string(),
                content: "Article content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "docs/guide.html".to_string(),
                title: "Guide".to_string(),
                display_title: "Guide".to_string(),
                content: "Guide content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
        ];

        // Test with multiple collections
        let content_folders = vec!["blog".to_string(), "docs".to_string()];
        let nav_builder = NavigationBuilder::new(
            &documents,
            "Test Site",
            0,
            None,
            None,
            &content_folders,
        );

        let navigation_html = nav_builder.generate_main_navigation();

        // Should include root-level page
        assert!(navigation_html.contains("Services"), "Should include root-level Services page");

        // Should exclude all collection items
        assert!(!navigation_html.contains("Blog Article"), "Should exclude blog collection items");
        assert!(!navigation_html.contains("Guide"), "Should exclude docs collection items");
    }

    #[test]
    fn test_navigation_empty_collections() {
        use crate::types::ParsedDocument;

        let documents = vec![
            ParsedDocument {
                url_path: "about.html".to_string(),
                title: "About".to_string(),
                display_title: "About".to_string(),
                content: "About content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
        ];

        // Test with empty content folders
        let content_folders: Vec<String> = vec![];
        let nav_builder = NavigationBuilder::new(
            &documents,
            "Test Site",
            0,
            None,
            None,
            &content_folders,
        );

        let navigation_html = nav_builder.generate_main_navigation();

        // Should include all non-index pages when no collections defined
        assert!(navigation_html.contains("About"), "Should include About page when no collections");
    }

    #[test]
    fn test_navigation_backwards_compatibility() {
        use crate::types::ParsedDocument;

        let documents = vec![
            ParsedDocument {
                url_path: "journal/old-post.html".to_string(),
                title: "Old Journal Post".to_string(),
                display_title: "Old Journal Post".to_string(),
                content: "Journal content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "about.html".to_string(),
                title: "About".to_string(),
                display_title: "About".to_string(),
                content: "About content".to_string(),
                date: None,
                html_content: "<p>Content</p>".to_string(),
                reading_time: 1,
                excerpt: "Excerpt".to_string(),
                slug: "slug".to_string(),
                permalink: "/permalink".to_string(),
                weight: None,
                topics: vec![],
                github: None,
                head_scripts: None,
                email_form: None,
            },
        ];

        // Test with journal as content folder (backwards compatibility)
        let content_folders = vec!["journal".to_string()];
        let nav_builder = NavigationBuilder::new(
            &documents,
            "Test Site",
            0,
            None,
            None,
            &content_folders,
        );

        let navigation_html = nav_builder.generate_main_navigation();

        // Should still exclude journal items
        assert!(!navigation_html.contains("Old Journal Post"), "Should exclude journal items for backwards compatibility");
        assert!(navigation_html.contains("About"), "Should include About page");
    }

    /// Latest Section Functionality Tests
    /// Tests that the latest sidebar correctly shows content from dynamic content folders
    #[test]
    fn test_latest_section_dynamic_content_folders() {
        use crate::types::{ParsedDocument, ProjectStructure, ProjectType};

        // Create test documents with different content folders
        let documents = vec![
            ParsedDocument {
                url_path: "posts/recent-post.html".to_string(),
                title: "Recent Post".to_string(),
                display_title: "Recent Post".to_string(),
                content: "Recent post content".to_string(),
                html_content: "<p>Recent post content</p>".to_string(),
                date: Some("2025-10-01".to_string()),
                topics: vec![],
                reading_time: 2,
                excerpt: "Recent post excerpt".to_string(),
                slug: "recent-post".to_string(),
                permalink: "/posts/recent-post.html".to_string(),
                weight: None,
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "journal/diary-entry.html".to_string(),
                title: "Diary Entry".to_string(),
                display_title: "Diary Entry".to_string(),
                content: "Diary entry content".to_string(),
                html_content: "<p>Diary entry content</p>".to_string(),
                date: Some("2025-10-02".to_string()),
                topics: vec![],
                reading_time: 1,
                excerpt: "Diary entry excerpt".to_string(),
                slug: "diary-entry".to_string(),
                permalink: "/journal/diary-entry.html".to_string(),
                weight: None,
                github: None,
                head_scripts: None,
                email_form: None,
            },
            ParsedDocument {
                url_path: "about.html".to_string(),
                title: "About".to_string(),
                display_title: "About".to_string(),
                content: "About content".to_string(),
                html_content: "<p>About content</p>".to_string(),
                date: None,
                topics: vec![],
                reading_time: 1,
                excerpt: "About excerpt".to_string(),
                slug: "about".to_string(),
                permalink: "/about.html".to_string(),
                weight: None,
                github: None,
                head_scripts: None,
                email_form: None,
            },
        ];

        // Test with posts content folder only
        let content_folders = vec!["posts".to_string()];
        let project_structure = ProjectStructure {
            root_path: "/test".to_string(),
            markdown_files: vec![],
            html_files: vec![],
            image_files: vec![],
            other_files: vec![],
            total_files: 0,
            project_type: ProjectType::HomepageWithCollections,
            homepage_file: None,
            content_folders: content_folders.clone(),
        };
        let nav_builder = NavigationBuilder::new(
            &documents,
            "Test Site",
            0,
            None,
            None,
            &content_folders,
        );

        let latest_sidebar = nav_builder.generate_latest_sidebar(&project_structure);

        // Should include posts from posts folder (only Recent Post since journal isn't in content_folders)
        assert!(latest_sidebar.contains("Recent Post"), "Should include the latest post from posts folder");
        // Should exclude journal entries when only posts is in content_folders
        assert!(!latest_sidebar.contains("Diary Entry"), "Should exclude journal entries when not in content_folders");
        // Should exclude regular pages
        assert!(!latest_sidebar.contains("About"), "Should exclude regular pages from latest section");

        // Test with both posts and journal content folders
        let content_folders = vec!["posts".to_string(), "journal".to_string()];
        let project_structure = ProjectStructure {
            root_path: "/test".to_string(),
            markdown_files: vec![],
            html_files: vec![],
            image_files: vec![],
            other_files: vec![],
            total_files: 0,
            project_type: ProjectType::HomepageWithCollections,
            homepage_file: None,
            content_folders: content_folders.clone(),
        };
        let nav_builder = NavigationBuilder::new(
            &documents,
            "Test Site",
            0,
            None,
            None,
            &content_folders,
        );

        let latest_sidebar = nav_builder.generate_latest_sidebar(&project_structure);


        // Should include the most recent entry (Diary Entry with 2025-10-02 is newer than Recent Post with 2025-10-01)
        assert!(latest_sidebar.contains("Diary Entry"), "Should show most recent entry from any content folder");
        assert!(!latest_sidebar.contains("Recent Post"), "Should only show the most recent entry");
        // Should still exclude regular pages
        assert!(!latest_sidebar.contains("About"), "Should exclude regular pages from latest section");

        // Test with empty content folders (no latest content)
        let content_folders: Vec<String> = vec![];
        let project_structure = ProjectStructure {
            root_path: "/test".to_string(),
            markdown_files: vec![],
            html_files: vec![],
            image_files: vec![],
            other_files: vec![],
            total_files: 0,
            project_type: ProjectType::HomepageWithCollections,
            homepage_file: None,
            content_folders: content_folders.clone(),
        };
        let nav_builder = NavigationBuilder::new(
            &documents,
            "Test Site",
            0,
            None,
            None,
            &content_folders,
        );

        let latest_sidebar = nav_builder.generate_latest_sidebar(&project_structure);

        // Should show "No posts yet" when no content folders are configured
        assert!(latest_sidebar.contains("No posts yet"), "Should show 'No posts yet' when no content folders are configured");
        assert!(!latest_sidebar.contains("Recent Post"), "Should not include any posts when no content folders");
        assert!(!latest_sidebar.contains("Diary Entry"), "Should not include any journal entries when no content folders");
    }
}