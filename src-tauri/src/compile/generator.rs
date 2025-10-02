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

use crate::compile::navigation::{NavigationBuilder, extract_date_from_path, extract_date_from_doc, generate_slug, generate_collection_breadcrumb};

/// Template types for different page layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateType {
    /// Homepage and general pages with sidebar
    Page,
    /// Journal entries with centered layout and breadcrumbs
    Article,
    /// Topic listing pages
    Topic,
}

/// Template registry for managing HTML templates
pub struct TemplateRegistry {
    page_template: &'static str,
    article_template: &'static str,
    topic_template: &'static str,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            page_template: PAGE_TEMPLATE,
            article_template: ARTICLE_TEMPLATE,
            topic_template: TOPIC_TEMPLATE,
        }
    }

    pub fn get_template(&self, template_type: TemplateType) -> &'static str {
        match template_type {
            TemplateType::Page => self.page_template,
            TemplateType::Article => self.article_template,
            TemplateType::Topic => self.topic_template,
        }
    }

    /// Determines the appropriate template type for a given document and context
    pub fn select_template_type(doc: Option<&ParsedDocument>, is_homepage: bool, content_folders: &[String]) -> TemplateType {
        match (doc, is_homepage) {
            (Some(doc), false) => {
                // Check if document is in any content collection folder (should use Article template)
                for folder in content_folders {
                    if doc.url_path.starts_with(&format!("{}/", folder)) {
                        return TemplateType::Article;
                    }
                }
                TemplateType::Page
            },
            _ => TemplateType::Page,
        }
    }
}

/// Path resolution utility for handling relative paths based on directory depth
pub struct PathResolver {
    depth: usize,
}

impl PathResolver {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }

    /// Calculate depth from URL path
    pub fn from_url_path(url_path: &str) -> Self {
        let depth = url_path.matches('/').count();
        Self::new(depth)
    }

    /// Generate CSS path based on depth
    pub fn css_path(&self) -> String {
        if self.depth == 0 {
            "style.css".to_string()
        } else {
            format!("{}style.css", "../".repeat(self.depth))
        }
    }

    /// Generate JavaScript path based on depth
    pub fn js_path(&self) -> String {
        if self.depth == 0 {
            "js/theme.js".to_string()
        } else {
            format!("{}js/theme.js", "../".repeat(self.depth))
        }
    }

    /// Generate depth-aware permalink for a document
    pub fn generate_permalink(&self, url_path: &str) -> String {
        if self.depth == 0 {
            format!("/{}", url_path)
        } else {
            format!("/{}{}", "../".repeat(self.depth), url_path)
        }
    }

    /// Generate favicon path based on depth
    pub fn favicon_path(&self) -> String {
        if self.depth == 0 {
            "assets/favicon.svg".to_string()
        } else {
            format!("{}assets/favicon.svg", "../".repeat(self.depth))
        }
    }

    /// Get depth value
    pub fn depth(&self) -> usize {
        self.depth
    }
}

/// Template variable for type-safe template processing
#[derive(Debug)]
pub struct TemplateVars {
    pub title: String,
    pub css_path: String,
    pub js_path: String,
    pub navigation: String,
    pub homepage_content: String,
    pub latest_list: Option<String>,
    pub latest_sidebar: Option<String>,
    pub topics_section: Option<String>,
    pub breadcrumb: Option<String>,
    pub favicon: Option<String>,
    pub head_scripts: Option<String>,
    // Article-specific variables
    pub site_name: Option<String>,
    pub date: Option<String>,
    pub formatted_date: Option<String>,
    pub short_date: Option<String>,
    pub content: Option<String>,
}

/// Template processor for centralized template variable replacement
pub struct TemplateProcessor {
    registry: TemplateRegistry,
}

impl TemplateProcessor {
    pub fn new() -> Self {
        Self {
            registry: TemplateRegistry::new(),
        }
    }

    /// Process template with variables and return HTML
    pub fn process(&self, template_type: TemplateType, vars: TemplateVars) -> String {
        let template = self.registry.get_template(template_type);

        let mut result = template
            .replace("{title}", &vars.title)
            .replace("{css_path}", &vars.css_path)
            .replace("{js_path}", &vars.js_path)
            .replace("{navigation}", &vars.navigation)
            .replace("{homepage_content}", &vars.homepage_content);

        // Optional variables - replace with content or empty string
        result = result.replace("{latest_list}", &vars.latest_list.unwrap_or_default());
        result = result.replace("{latest_sidebar}", &vars.latest_sidebar.unwrap_or_default());
        result = result.replace("{topics_section}", &vars.topics_section.unwrap_or_default());
        result = result.replace("{breadcrumb}", &vars.breadcrumb.unwrap_or_default());
        result = result.replace("{favicon}", &vars.favicon.unwrap_or_default());
        result = result.replace("{head_scripts}", &vars.head_scripts.unwrap_or_default());

        // Article-specific variables
        result = result.replace("{site_name}", &vars.site_name.unwrap_or_default());
        result = result.replace("{date}", &vars.date.unwrap_or_default());
        result = result.replace("{formatted_date}", &vars.formatted_date.unwrap_or_default());
        result = result.replace("{short_date}", &vars.short_date.unwrap_or_default());
        result = result.replace("{content}", &vars.content.unwrap_or_default());

        result
    }
}

/// Frontmatter structure for parsing YAML metadata from markdown files
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct FrontMatter {
    /// Optional title override from frontmatter
    pub title: Option<String>,
    /// Optional publication date
    pub date: Option<String>,
    /// Topics or tags for categorization
    pub topics: Option<Vec<String>>,
    /// Navigation weight for ordering (lower numbers = higher priority)
    pub weight: Option<i32>,
    /// GitHub repository URL for site-wide navigation
    pub github: Option<String>,
    /// Scripts to inject into <head> section (e.g., analytics)
    pub head_scripts: Option<String>,
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

    // Detect and copy favicon if assets/favicon.svg exists
    let favicon_source = Path::new(source_path).join("assets").join("favicon.svg");
    let has_favicon = favicon_source.exists();
    if has_favicon {
        let favicon_dest_dir = output_dir.join("assets");
        fs::create_dir_all(&favicon_dest_dir).map_err(|e| format!("Failed to create assets directory: {}", e))?;
        let favicon_dest = favicon_dest_dir.join("favicon.svg");
        fs::copy(&favicon_source, &favicon_dest).map_err(|e| format!("Failed to copy favicon: {}", e))?;
    }

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
        
        let html_page = generate_html(Some(doc), &documents, project_structure, false, has_favicon)?;
        fs::write(&output_file_path, html_page).map_err(|e| format!("Failed to write HTML file: {}", e))?;
        page_count += 1;
    }
    
    // Generate topic pages
    let all_topics = collect_all_topics(&documents);
    if !all_topics.is_empty() {
        // Create topics directory
        let topics_dir = output_dir.join("topics");
        fs::create_dir_all(&topics_dir).map_err(|e| format!("Failed to create topics directory: {}", e))?;
        
        // Generate navigation for topic pages
        let homepage_doc = project_structure.homepage_file.as_ref()
            .and_then(|_| documents.iter().find(|d| d.url_path == "index.html"));

        let site_title = homepage_doc
            .map(|d| d.title.clone())
            .unwrap_or_else(|| "Site".to_string());

        let github_url = homepage_doc
            .and_then(|d| d.github.as_ref())
            .map(|s| s.as_str());

        for topic in all_topics {
            let topic_content = generate_topic_page_content(&topic, &documents, project_structure);
            let path_resolver = PathResolver::new(1); // Topics are at depth 1
            let nav_builder = NavigationBuilder::new(&documents, &site_title, 1, None, github_url, &project_structure.content_folders);
            let processor = TemplateProcessor::new();
            
            let vars = TemplateVars {
                title: format!("Topic: {}", topic),
                css_path: path_resolver.css_path(),
                js_path: path_resolver.js_path(),
                navigation: nav_builder.generate_main_navigation(),
                homepage_content: topic_content,
                latest_list: None,
                latest_sidebar: None,
                topics_section: None,
                breadcrumb: None,
                favicon: if has_favicon {
                    Some(format!(r#"<link rel="icon" type="image/svg+xml" href="{}">"#, path_resolver.favicon_path()))
                } else {
                    None
                },
                head_scripts: None,
                // Article-specific variables (not used for topics)
                site_name: None,
                date: None,
                formatted_date: None,
                short_date: None,
                content: None,
            };
            
            let topic_html = processor.process(TemplateType::Topic, vars);
            let topic_file_path = topics_dir.join(format!("{}.html", generate_slug(&topic)));
            fs::write(&topic_file_path, topic_html).map_err(|e| format!("Failed to write topic page: {}", e))?;
            page_count += 1;
        }
    }

    // Generate collection index pages for content folders
    if !project_structure.content_folders.is_empty() {
        let homepage_doc = project_structure.homepage_file.as_ref()
            .and_then(|_| documents.iter().find(|d| d.url_path == "index.html"));

        let site_title = homepage_doc
            .map(|d| d.title.clone())
            .unwrap_or_else(|| "Site".to_string());

        let github_url = homepage_doc
            .and_then(|d| d.github.as_ref())
            .map(|s| s.as_str());

        for folder_name in &project_structure.content_folders {
            // Find all documents in this collection
            let collection_docs: Vec<&ParsedDocument> = documents.iter()
                .filter(|doc| doc.url_path.starts_with(&format!("{}/", folder_name)))
                .collect();

            if !collection_docs.is_empty() {
                // Create collection directory
                let collection_dir = output_dir.join(folder_name);
                fs::create_dir_all(&collection_dir).map_err(|e| format!("Failed to create collection directory: {}", e))?;

                // Generate collection index content
                let collection_content = generate_collection_index_content(folder_name, &collection_docs, project_structure);

                // Generate HTML for collection index
                let path_resolver = PathResolver::new(1); // Collections are at depth 1
                let nav_builder = NavigationBuilder::new(&documents, &site_title, 1, None, github_url, &project_structure.content_folders);
                let processor = TemplateProcessor::new();

                let vars = TemplateVars {
                    title: format!("{}", folder_name),
                    css_path: path_resolver.css_path(),
                    js_path: path_resolver.js_path(),
                    navigation: nav_builder.generate_main_navigation(),
                    homepage_content: collection_content,
                    latest_list: None,
                    latest_sidebar: None,
                    topics_section: None,
                    breadcrumb: Some(generate_collection_breadcrumb(folder_name)),
                    favicon: if has_favicon {
                        Some(format!(r#"<link rel="icon" type="image/svg+xml" href="{}">"#, path_resolver.favicon_path()))
                    } else {
                        None
                    },
                    head_scripts: None,
                    // Article-specific variables (not used for collection index)
                    site_name: None,
                    date: None,
                    formatted_date: None,
                    short_date: None,
                    content: None,
                };

                let collection_html = processor.process(TemplateType::Page, vars);
                let collection_index_path = collection_dir.join("index.html");
                fs::write(&collection_index_path, collection_html).map_err(|e| format!("Failed to write collection index: {}", e))?;
                page_count += 1;
            }
        }
    }

    // Generate CSS and JavaScript
    let css_content = DEFAULT_CSS;
    fs::write(output_dir.join("style.css"), css_content).map_err(|e| format!("Failed to write CSS: {}", e))?;
    
    // Create js directory and copy theme.js
    let js_dir = output_dir.join("js");
    fs::create_dir_all(&js_dir).map_err(|e| format!("Failed to create js directory: {}", e))?;
    
    let js_content = include_str!("../assets/js/theme.js");
    fs::write(js_dir.join("theme.js"), js_content).map_err(|e| format!("Failed to write theme.js: {}", e))?;
    
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

    // Copy other files (CNAME, robots.txt, etc.) to output root
    for file_info in &project_structure.other_files {
        let source_file = Path::new(source_path).join(&file_info.path);
        let dest_file = output_dir.join(&file_info.path);

        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory for other files: {}", e))?;
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
            let index_html = generate_html(homepage_doc, &documents, project_structure, true, has_favicon)?;
            fs::write(output_dir.join("index.html"), index_html).map_err(|e| format!("Failed to write index.html: {}", e))?;
            page_count += 1;
        } else {
            // No homepage file - generate pure blog feed
            let index_html = generate_html(None, &documents, project_structure, false, has_favicon)?;
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

/// Transforms markdown links to HTML links for static site generation.
///
/// Converts relative `.md` and `.markdown` links to `.html` while preserving:
/// - Fragments (e.g., `file.md#section` → `file.html#section`)
/// - External URLs (http/https URLs are not transformed)
/// - Already .html links (no transformation needed)
///
/// # Arguments
/// * `url` - The URL from a markdown link
///
/// # Returns
/// * Transformed URL with .md → .html conversion
///
/// # Examples
/// ```
/// assert_eq!(transform_markdown_link("./roadmap.md"), "./roadmap.html");
/// assert_eq!(transform_markdown_link("file.md#anchor"), "file.html#anchor");
/// assert_eq!(transform_markdown_link("https://example.com/file.md"), "https://example.com/file.md");
/// ```
fn transform_markdown_link(url: &str) -> String {
    // Don't transform external URLs
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("//") {
        return url.to_string();
    }

    // Split URL into path and fragment
    let (path, fragment) = if let Some(pos) = url.find('#') {
        (&url[..pos], Some(&url[pos..]))
    } else {
        (url, None)
    };

    // Transform .md or .markdown to .html
    let transformed_path = if path.ends_with(".md") {
        path.strip_suffix(".md").unwrap().to_string() + ".html"
    } else if path.ends_with(".markdown") {
        path.strip_suffix(".markdown").unwrap().to_string() + ".html"
    } else {
        path.to_string()
    };

    // Recombine with fragment if present
    if let Some(frag) = fragment {
        transformed_path + frag
    } else {
        transformed_path
    }
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
    // HTML is preserved by default in pulldown-cmark
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);

    // Create parser and transform markdown links to HTML links
    let parser = Parser::new_ext(&result.content, options).map(|event| {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link {
                link_type,
                dest_url,
                title,
                id
            }) => {
                // Transform .md links to .html
                let transformed_url = transform_markdown_link(&dest_url);
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link {
                    link_type,
                    dest_url: transformed_url.into(),
                    title,
                    id,
                })
            }
            _ => event,
        }
    });

    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);
    
    // Extract title from markdown content H1, with filename as fallback
    let title = extract_first_heading(&html_content).unwrap_or(filename_title);
    
    // Generate URL path using slugified filenames
    let url_path = if file_path.to_lowercase() == "index.md" || file_path.to_lowercase() == "readme.md" {
        "index.html".to_string()
    } else {
        // Get filename without extension
        let filename_stem = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled");

        // Generate slug and add .html extension
        let slug = generate_slug(filename_stem);

        // Preserve directory structure
        let parent_path = Path::new(file_path).parent()
            .and_then(|p| p.to_str())
            .unwrap_or("");

        if parent_path.is_empty() {
            format!("{}.html", slug)
        } else {
            format!("{}/{}.html", parent_path, slug)
        }
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

    // Extract weight from frontmatter
    let weight = frontmatter.weight;

    // Extract github URL from frontmatter
    let github = frontmatter.github;

    // Extract head scripts from frontmatter
    let head_scripts = frontmatter.head_scripts;

    // Calculate reading time (200 words per minute)
    let word_count = result.content.split_whitespace().count();
    let reading_time = std::cmp::max(1, (word_count / 200) as u32);
    
    // Generate excerpt from content
    let excerpt = extract_excerpt(&html_content);
    
    // Generate enhanced fields following SSG best practices
    let slug = generate_slug(&title);
    let depth = url_path.matches('/').count();
    let path_resolver = PathResolver::new(depth);
    let permalink = path_resolver.generate_permalink(&url_path);
    let display_title = resolve_display_title(&title, &html_content, &url_path);
    
    Ok(ParsedDocument {
        title,
        content: result.content,
        html_content,
        url_path,
        date,
        topics,
        reading_time,
        excerpt,
        slug,
        permalink,
        display_title,
        weight,
        github,
        head_scripts,
    })
}

/// Unified HTML generation function for all page types 
pub fn generate_html(
    doc: Option<&ParsedDocument>,
    all_docs: &[ParsedDocument],
    project: &ProjectStructure,
    is_homepage: bool,
    has_favicon: bool
) -> Result<String, String> {
    // Get site title and github URL from homepage for navigation
    let homepage_doc = project.homepage_file.as_ref()
        .and_then(|_| all_docs.iter().find(|d| d.url_path == "index.html"));

    let site_title = homepage_doc
        .map(|d| d.title.clone())
        .unwrap_or_else(|| "Site".to_string());

    let github_url = homepage_doc
        .and_then(|d| d.github.as_ref())
        .map(|s| s.as_str());

    let head_scripts = homepage_doc
        .and_then(|d| d.head_scripts.as_ref())
        .map(|s| s.as_str());

    // Calculate depth for path adjustments
    let depth = doc.map(|d| d.url_path.matches('/').count()).unwrap_or(0);

    // Initialize utilities
    let path_resolver = PathResolver::new(depth);
    let current_page_url = doc.map(|d| d.url_path.as_str());
    let nav_builder = NavigationBuilder::new(all_docs, &site_title, depth, current_page_url, github_url, &project.content_folders);
    let processor = TemplateProcessor::new();
    
    // Determine template type
    let template_type = TemplateRegistry::select_template_type(doc, is_homepage, &project.content_folders);
    let is_article_page = template_type == TemplateType::Article;
    
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
            // Regular page - remove duplicate title if it matches
            let mut content = doc.html_content.clone();
            
            // Remove the first H1 heading based on page type
            let should_remove_h1 = if is_navigation_page(&doc.url_path, &project.content_folders) {
                // For navigation pages: always remove first H1 (title shown in nav)
                true
            } else if is_article_page {
                // For article pages: remove H1 if it matches title (shown in breadcrumb)
                if let Some(_h1_start) = content.find("<h1>") {
                    if let Some(h1_end) = content.find("</h1>") {
                        let h1_content = &content[_h1_start + 4..h1_end]; // +4 to skip "<h1>"
                        h1_content.trim() == doc.title.trim()
                    } else { false }
                } else { false }
            } else {
                false
            };
            
            if should_remove_h1 {
                if let Some(_h1_start) = content.find("<h1>") {
                    if let Some(h1_end) = content.find("</h1>") {
                        content = content[h1_end + 5..].to_string(); // +5 to skip "</h1>"
                    }
                }
            }
            
            (doc.title.clone(), content)
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

    // Build template variables
    let vars = TemplateVars {
        title: page_title,
        css_path: path_resolver.css_path(),
        js_path: path_resolver.js_path(),
        navigation: nav_builder.generate_main_navigation(),
        homepage_content: homepage_content.clone(),
        latest_list: if template_type == TemplateType::Page {
            Some(nav_builder.generate_latest_sidebar(project))
        } else {
            None
        },
        latest_sidebar: if template_type == TemplateType::Page {
            let latest_content = nav_builder.generate_latest_sidebar(project);
            if latest_content.is_empty() {
                None
            } else {
                Some(format!(r#"<aside class="latest-sidebar">
                <h3>Latest</h3>
                {}
            </aside>"#, latest_content))
            }
        } else {
            None
        },
        topics_section: if is_homepage && doc.is_some() {
            let topics_inline = nav_builder.generate_topics_inline();
            if topics_inline.is_empty() {
                None
            } else {
                Some(format!(r#"
            <section class="topics">
                <h3>Topics</h3>
                {}
            </section>"#, topics_inline))
            }
        } else {
            None
        },
        breadcrumb: if is_article_page && doc.is_some() {
            Some(nav_builder.generate_breadcrumb(doc.unwrap()))
        } else {
            None
        },
        favicon: if has_favicon {
            Some(format!(r#"<link rel="icon" type="image/svg+xml" href="{}">"#, path_resolver.favicon_path()))
        } else {
            None
        },
        head_scripts: head_scripts.map(|s| s.to_string()),
        // Article-specific variables
        site_name: if is_article_page { Some(site_title.clone()) } else { None },
        date: if is_article_page && doc.is_some() { doc.unwrap().date.clone() } else { None },
        formatted_date: if is_article_page && doc.is_some() && doc.unwrap().date.is_some() {
            Some(format_date(doc.unwrap().date.as_ref().unwrap()))
        } else { None },
        short_date: if is_article_page && doc.is_some() && doc.unwrap().date.is_some() {
            Some(crate::compile::navigation::format_date_string(doc.unwrap().date.as_ref().unwrap()))
        } else { None },
        content: if is_article_page { Some(homepage_content) } else { None },
    };
    
    Ok(processor.process(template_type, vars))
}


/// Generates content for a topic page showing all articles with that topic.
/// Follows Hugo taxonomy template patterns for consistent URL and title handling.
/// References: https://gohugo.io/templates/taxonomy-templates/
pub fn generate_topic_page_content(topic: &str, documents: &[ParsedDocument], project: &ProjectStructure) -> String {
    let articles_with_topic: Vec<&ParsedDocument> = documents.iter()
        .filter(|doc| doc.topics.contains(&topic.to_string()))
        .collect();
    
    if articles_with_topic.is_empty() {
        return format!("<p>No articles found for topic: {}</p>", topic);
    }
    
    let article_list: Vec<String> = articles_with_topic.iter()
        .map(|doc| {
            // Use the same date format as Latest section (YYYY · MM)
            let date_display = extract_date_from_doc(doc, project);
            
            // Topic pages are at depth 1, so prepend "../" to URLs
            let article_url = format!("../{}", doc.url_path);
            
            format!(
                "<p><span class=\"date\">{}</span>&nbsp;&nbsp;<a href=\"{}\" style=\"text-decoration: underline; color: var(--moss-text-primary);\">{}</a></p>",
                date_display, article_url, doc.display_title
            )
        })
        .collect();
    
    format!(
        "<h1>Topic: {}</h1>\n<div class=\"topic-articles\">{}</div>",
        topic,
        article_list.join("\n")
    )
}

/// Generates content for a collection index page showing all articles in that collection.
pub fn generate_collection_index_content(collection_name: &str, documents: &[&ParsedDocument], project_structure: &ProjectStructure) -> String {
    if documents.is_empty() {
        return format!("<p>No articles found in collection: {}</p>", collection_name);
    }

    // Sort documents by date (newest first) if available, otherwise by title
    let mut sorted_docs = documents.to_vec();
    sorted_docs.sort_by(|a, b| {
        match (&a.date, &b.date) {
            (Some(date_a), Some(date_b)) => date_b.cmp(date_a), // Newest first
            (Some(_), None) => std::cmp::Ordering::Less,       // Dated items first
            (None, Some(_)) => std::cmp::Ordering::Greater,    // Undated items last
            (None, None) => a.display_title.cmp(&b.display_title), // Alphabetical fallback
        }
    });

    let article_list: Vec<String> = sorted_docs.iter()
        .map(|doc| {
            // Extract just the filename from the url_path for relative linking
            let filename = doc.url_path.split('/').last().unwrap_or(&doc.url_path);

            // Format date display
            let date_display = if let Some(date) = &doc.date {
                format_date(date)
            } else {
                extract_date_from_path(&doc.url_path, None, &project_structure.content_folders)
            };

            format!(
                "<p><span class=\"date\">{}</span>&nbsp;&nbsp;<a href=\"{}\" style=\"text-decoration: underline; color: var(--moss-text-primary);\">{}</a></p>",
                date_display, filename, doc.display_title
            )
        })
        .collect();

    format!(
        "<div class=\"collection-listing\">{}</div>",
        article_list.join("\n")
    )
}

/// Extracts title from filename by removing path and extension, following Jekyll conventions.
/// References: https://jekyllrb.com/docs/posts/#creating-posts
pub fn extract_filename_title(url_path: &str) -> String {
    Path::new(url_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .replace("-", " ")
        .replace("_", " ")
}



/// Resolves the best display title from available sources.
/// Following Eleventy computed data cascade: H1 > frontmatter.title > filename
/// References: https://www.11ty.dev/docs/data-computed/
pub fn resolve_display_title(title: &str, html_content: &str, url_path: &str) -> String {
    // Priority 1: Try to extract H1 from HTML content
    if let Some(h1_content) = extract_first_heading(html_content) {
        // Only use H1 if it's different from filename-derived title
        let filename_title = extract_filename_title(url_path);
        if h1_content != filename_title && !h1_content.trim().is_empty() {
            return h1_content;
        }
    }
    
    // Priority 2: Use provided title (from frontmatter or filename processing)
    title.to_string()
}

/// Determines if a document is a navigation page (appears in main nav).
/// Navigation pages exclude content collection entries, topic pages, and homepage.
/// These pages show their titles in the navigation, so content titles become redundant.
pub fn is_navigation_page(url_path: &str, content_folders: &[String]) -> bool {
    // Check if it's a content collection page
    for folder in content_folders {
        if url_path.starts_with(&format!("{}/", folder)) {
            return false;
        }
    }

    // Exclude topic pages and homepage
    !url_path.starts_with("topics/") &&
    url_path != "index.html"
}

/// Collects all unique topics from documents.
pub fn collect_all_topics(documents: &[ParsedDocument]) -> Vec<String> {
    let mut all_topics: Vec<String> = documents.iter()
        .flat_map(|doc| doc.topics.iter())
        .cloned()
        .collect();
    
    all_topics.sort();
    all_topics.dedup();
    all_topics
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
            doc.url_path, doc.display_title, date_str, topics_str, doc.reading_time
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
const ARTICLE_TEMPLATE: &str = include_str!("../assets/templates/article.html");
const TOPIC_TEMPLATE: &str = include_str!("../assets/templates/topic.html");

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
        let content_folders = vec!["journal".to_string(), "posts".to_string()];

        // Test journal path extraction
        let date = extract_date_from_path("journal/2025-01-15.html", None, &content_folders);
        assert_eq!(date, "2025 · 01");

        let date = extract_date_from_path("journal/2024-12-31.html", None, &content_folders);
        assert_eq!(date, "2024 · 12");

        // Test posts path extraction
        let date = extract_date_from_path("posts/2025-01-15.html", None, &content_folders);
        assert_eq!(date, "2025 · 01");

        // Test non-content folder path
        let date = extract_date_from_path("about.html", None, &content_folders);
        assert_eq!(date, "Unknown");

        // Test invalid content folder path (function extracts first two parts separated by dashes)
        let date = extract_date_from_path("journal/invalid-name.html", None, &content_folders);
        assert_eq!(date, "invalid · name");

        // Test completely invalid content folder path (only one part, needs at least 2)
        let date = extract_date_from_path("journal/nodashes.html", None, &content_folders);
        assert_eq!(date, "Unknown");
    }

    #[test]
    fn test_transform_markdown_link() {
        // Test basic .md to .html transformation
        assert_eq!(transform_markdown_link("./roadmap.md"), "./roadmap.html");
        assert_eq!(transform_markdown_link("roadmap.md"), "roadmap.html");
        assert_eq!(transform_markdown_link("../roadmap.md"), "../roadmap.html");

        // Test .markdown to .html transformation
        assert_eq!(transform_markdown_link("./file.markdown"), "./file.html");
        assert_eq!(transform_markdown_link("docs/README.markdown"), "docs/README.html");

        // Test fragment preservation
        assert_eq!(transform_markdown_link("file.md#section"), "file.html#section");
        assert_eq!(transform_markdown_link("./docs/api.md#overview"), "./docs/api.html#overview");
        assert_eq!(transform_markdown_link("../guide.md#getting-started"), "../guide.html#getting-started");

        // Test that external URLs are not transformed
        assert_eq!(transform_markdown_link("https://example.com/file.md"), "https://example.com/file.md");
        assert_eq!(transform_markdown_link("http://example.com/docs.md"), "http://example.com/docs.md");
        assert_eq!(transform_markdown_link("//cdn.example.com/file.md"), "//cdn.example.com/file.md");

        // Test that .html links are not transformed
        assert_eq!(transform_markdown_link("./page.html"), "./page.html");
        assert_eq!(transform_markdown_link("page.html#anchor"), "page.html#anchor");

        // Test edge cases
        assert_eq!(transform_markdown_link(""), "");
        assert_eq!(transform_markdown_link("#anchor"), "#anchor");
        assert_eq!(transform_markdown_link("./"), "./");
        assert_eq!(transform_markdown_link("README"), "README");

        // Test complex paths
        assert_eq!(transform_markdown_link("./docs/guide/setup.md"), "./docs/guide/setup.html");
        assert_eq!(transform_markdown_link("../../README.md"), "../../README.html");
    }

    #[test]
    fn test_markdown_processing_transforms_links() {
        let markdown = r#"# Test Document

See the [roadmap](./roadmap.md) for details.
Check out the [guide](docs/guide.md#setup) for setup instructions.
Visit our [website](https://example.com/docs.md) for more info."#;

        let result = process_markdown_file("test.md", markdown).expect("Should process markdown");

        // Verify that .md links are transformed to .html
        assert!(result.html_content.contains(r#"<a href="./roadmap.html">roadmap</a>"#),
                "Should transform ./roadmap.md to ./roadmap.html");
        assert!(result.html_content.contains(r#"<a href="docs/guide.html#setup">guide</a>"#),
                "Should transform docs/guide.md#setup to docs/guide.html#setup");

        // Verify that external URLs are not transformed
        assert!(result.html_content.contains(r#"<a href="https://example.com/docs.md">website</a>"#),
                "Should not transform external URLs");
    }
}