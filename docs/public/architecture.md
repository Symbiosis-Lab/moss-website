# architecture

```
Local files → moss → Static site → Your infrastructure
                ↓
          Syndication → Existing platforms
                ↓
        Spore/Lichen → Social layer
```

**moss** - Tauri app, lives in menu bar, compiles and deploys  
**Spore** - Optional ActivityPub/WebMention server  
**Lichen** - JavaScript widget for comments on any static site

## System Overview

```
Local files → moss → Static site → Your infrastructure
                ↓
          Syndication → Existing platforms
                ↓
        Spore/Lichen → Social layer
```

**moss** - Desktop app that transforms folders into websites  
**Spore** - Optional server for social features (ActivityPub/WebMentions)  
**Lichen** - Embeddable comment widget for any static site

## The Publishing Pipeline

### 1. Content Detection

When you right-click a folder, moss analyzes your content:

- **Homepage discovery**: Looks for `index.md`, `README.md`, or first document file
- **Site structure**: Detects flat directory vs homepage + collections
- **File support**: Markdown, Pages, Word documents, plus images and assets
- **Smart defaults**: No configuration required - moss understands common patterns

### 2. Folder Adaptation

moss preserves your preferred folder structure while making it compatible with any static site generator:

**Virtual Build Environment**

- Your files stay in their original locations
- moss creates temporary SSG-compatible structure in `.moss/build/`
- Uses symlinks for efficiency (copies as fallback)
- SSG runs against adapted structure, never your original files

**Universal Mapping**

```
Your Structure           →    SSG Structure
articles/                →    _posts/ (Jekyll) or content/ (Hugo)
media/                   →    assets/ or static/
design/styles.css        →    _sass/ or assets/css/
```

**Zero Configuration**

- Automatic detection of your folder patterns
- Intelligent mapping to SSG conventions
- Override via optional `.moss/config.toml` if needed

### 3. SSG Integration

moss works with existing static site generators rather than replacing them:

**Default Minimal Plugin**

- Ultra-lightweight Rust implementation (~200 lines)
- Single beautiful default template
- Instant results for new users
- Bundled with moss core (no additional download)

**SSG Plugin Ecosystem**

- Auto-detect required SSG from theme selection
- Download SSG plugins on-demand (Jekyll, Hugo, Zola, Eleventy)
- Plugins run in isolated processes with adapted folder structure
- Cache downloaded plugins and binaries for future use

**Supported SSGs** (Priority Order)

1. **Jekyll** - Largest theme ecosystem (300+ themes)
2. **Hugo** - High performance, quality themes (100+ themes)
3. **Zola** - Rust-native, growing ecosystem (12 MB binary)
4. **Eleventy** - Modern, flexible (83+ themes)

### 4. Theme Marketplace

**Theme-First User Journey**

1. User browses visual theme gallery
2. Selects theme they love
3. moss auto-detects required SSG
4. Downloads SSG if not present
5. Applies theme with user's content

**Theme Sources**

- Jekyll: jekyllthemes.io, Best Jekyll Themes (300+ options)
- Hugo: themes.gohugo.io (curated collection)
- Zola: Tera-based themes (growing ecosystem)
- Eleventy: 11tythemes.com (83+ modern themes)

### 5. Three-Window Preview

**Main Preview**

- Your content rendered exactly as it will appear online
- No moss branding or interface elements
- Full-size window showing your actual website

**Floating Control Zones**

- **Top zone**: Core actions (Publish, Edit, Settings) as transparent circular buttons
- **Right zone**: Platform syndication controls using familiar brand colors
- **Magnetic positioning**: Control windows follow the preview automatically

### 6. Deployment & Syndication

**Publishing**

- moss.pub: Zero-configuration subdomain (username.moss.pub)
- GitHub Pages: Direct integration for developers
- Custom hosting: Netlify, Vercel, S3-compatible services

**Syndication**

- Automatic cross-posting to social platforms
- RSS feed generation for discovery
- Email newsletter integration
- All through plugins - enable what you need

## Current Architecture (Phase 0)

### Integrated Rust Implementation

✅ **Currently Implemented:** moss uses a unified Rust-based static site generator with all functionality embedded in a single binary:

**Content Processing**

- Markdown parsing with pulldown-cmark
- Frontmatter extraction with gray_matter
- Automatic project structure detection
- Beautiful default CSS styling
- Responsive HTML generation

**Local Development Server**

- Axum-based HTTP server on port 8080
- Automatic port detection and fallback
- Static file serving with proper headers
- Live preview during development

**System Integration**

- macOS Finder Services integration
- Menu bar tray icon presence
- Deep link handling (moss:// protocol)
- CLI and GUI operation modes

### Future Plugin Architecture (Phase 1+)

📋 **Planned for Phase 1-2:** Transition to plugin-based architecture:

**Plugin System Foundation**

- Process isolation for security and stability
- JSON-RPC protocol for language-agnostic plugins
- Manifest system for capabilities and dependencies
- Zero-configuration auto-installation

**Planned Plugin Types**

- **SSG Plugins**: Jekyll, Hugo, Zola, Eleventy support
- **Theme System**: Visual theme browser and marketplace
- **Publishing**: moss.pub, GitHub Pages, Netlify, Vercel
- **Social Features**: Comments, WebMentions, ActivityPub
- **Content Enhancement**: Syntax highlighting, SEO, analytics

**Migration Strategy**

- Current Rust SSG becomes first "bundled plugin"
- Gradual extraction of features into discrete plugins
- Maintain backward compatibility throughout transition

## Platform Integration

### macOS

- Native Services integration (right-click "Publish")
- Menu bar presence with system-appropriate icon
- Full vibrancy effects on control windows

### Windows

- Context menu integration
- Windows 11 design language compliance
- System accent color adaptation

### Linux

- Cross-desktop environment compatibility
- High-contrast fallbacks for accessibility
- Standard window decorations when needed

### All Platforms

- Native file dialogs and system integration
- OS-appropriate fonts and spacing
- Automatic light/dark mode adaptation

## Current Build Pipeline

### Compilation Workflow

✅ **Current Phase 0 Implementation:**

```
1. Folder Scan          → Recursively analyze directory structure
2. Content Detection    → Identify markdown files, images, and homepage
3. Project Classification → Determine site type (homepage+collections, flat, blog-style)
4. Markdown Processing  → Parse frontmatter, convert to HTML
5. Site Generation     → Create static HTML with beautiful default CSS
6. Local Server        → Start preview server on localhost:8080
7. Preview Window      → Open with Publish/Edit controls
```

### Current File Structure

```
.moss/                  # moss working directory (hidden from user)
└── site/              # Generated HTML output
    ├── index.html     # Homepage or blog feed
    ├── style.css      # Default responsive styling
    ├── about.html     # Individual pages
    └── journal/       # Content collections
        └── *.html     # Generated from markdown
```

### Philosophy

**Current Approach (Phase 0):** moss provides a beautiful, zero-configuration static site generator that works immediately without setup. The embedded Rust implementation ensures reliability and consistency while we validate the core workflow.

**Future Vision (Phase 1+):** Transition to a plugin orchestration system that works WITH the existing SSG ecosystem rather than against it. This will provide access to thousands of themes while preserving user folder freedom and adding the workflow features SSGs lack: right-click publishing, visual preview, one-click deployment, and social integration.

The goal is to become "Homebrew for static sites"—a package manager and orchestration layer that makes powerful tools accessible without complexity.

---

_moss handles the technical complexity so you can focus on creating. Right-click, publish, reach your audience._
