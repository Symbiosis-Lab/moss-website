# How moss Works

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

## Plugin Architecture

### Minimal Core + Plugin Ecosystem

moss core (~5MB) handles only essential functions:
- Content analysis and folder structure detection
- Plugin discovery, loading, and orchestration
- File system abstraction and virtual structure mapping
- Local preview server and deployment coordination

Everything else is a plugin, downloaded on-demand:

**Static Site Generation**
- Default minimal SSG (bundled)
- Jekyll plugin (Ruby ecosystem + 300+ themes)
- Hugo plugin (Go-based, high performance)
- Zola plugin (Rust-native, minimal)
- Eleventy plugin (Node.js, modern)

**Theme System**
- Theme marketplace integration
- Visual theme browser
- Automatic SSG detection from theme choice
- Theme installation and management

**Publishing & Distribution**
- moss.pub hosting plugin
- GitHub Pages plugin
- Netlify/Vercel deployment plugins
- Custom hosting adapters

**Social Features (Lichen Plugin)**
- Post-build HTML injection (works with any SSG)
- Comments and discussions via Spore backend
- WebMention support for IndieWeb integration
- ActivityPub federation

**Content Enhancement**
- Syntax highlighting plugins
- Image optimization and galleries
- SEO optimization and sitemaps
- Analytics integration (privacy-focused)

### Plugin Communication

**Process Isolation**: Each plugin runs as separate process for security and stability

**JSON-RPC Protocol**: Language-agnostic communication enables plugins in any language

**Sandboxed Execution**: Plugins have limited filesystem and network access

**Manifest System**: Each plugin declares capabilities, dependencies, and permissions

### User Experience

**Zero Configuration**: Plugins auto-install when needed (with user consent)

**Progressive Enhancement**: Start with minimal SSG, add plugins as requirements grow

**Visual Integration**: All plugins follow consistent UI patterns:
- Circular controls in floating zones
- Platform-specific colors for recognition
- Advanced options hidden until needed

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

## Technical Architecture

### Build Pipeline
```
1. Content Detection    → Analyze user's folder structure
2. Plugin Selection     → Choose appropriate SSG plugin based on content/theme
3. Plugin Loading       → Load plugin if not already available
4. Folder Adaptation    → Plugin maps user structure to its requirements
5. Site Generation      → Plugin executes SSG against adapted structure
6. Post-Build           → Apply enhancement plugins (Lichen.js, etc.)
7. Preview/Deploy       → Serve locally or upload via publisher plugins
```

### File Structure
```
.moss/                  # All moss files (hidden from user)
├── build/             # Temporary SSG-compatible structure (per plugin)
├── site/              # Generated HTML output
├── plugins/           # User-installed plugins
│   ├── jekyll/        # Plugin directory with manifest and executable
│   └── hugo/          # Another plugin
├── cache/             # Downloaded plugin binaries and themes  
├── themes/            # Theme installations
├── config.toml        # User configuration
└── logs/              # Plugin execution logs
```

### Philosophy
moss doesn't compete with existing tools—it orchestrates them through a plugin architecture. By working WITH the SSG ecosystem rather than against it, moss provides instant access to thousands of themes while preserving user folder freedom. The plugin system adds the workflow features SSGs lack: right-click publishing, visual preview, one-click deployment, and social integration.

This makes moss the "Homebrew for static sites"—a package manager and orchestration layer that makes powerful tools accessible without complexity.

---

*moss handles the technical complexity so you can focus on creating. Right-click, publish, reach your audience.*