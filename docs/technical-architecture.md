# Moss Technical Architecture

## Core Stack

**Desktop Application**
- Tauri v2 (Rust backend, web frontend)
- Menu bar/system tray only (no main window)
- Native OS integration for right-click context menu on folders
- ~15MB binary size target

**System Tray Menu**
- Settings (preferences, configuration)
- About
- Quit

**Folder Context Menu Integration**
- "Publish to Web" option when right-clicking any folder
- Triggers site generation and deployment workflow
- No folder selection dialog needed - uses the clicked folder

**Static Site Generator**
- Custom Rust implementation
- Embedded in Tauri binary
- Input: Markdown, HTML, images
- Output: Optimized static HTML/CSS/JS

**Frontend Runtime**
- Vanilla JS for core
- React for preferences UI only
- No framework for generated sites (pure HTML/CSS)

## File System Architecture

```
~/.moss/
├── config.toml          # Global configuration
├── themes/              # Installed themes
├── plugins/             # Installed plugins
└── cache/               # Build cache

[User's Writing Folder]/
├── .moss/               # Site-specific config (optional)
├── posts/               # Content (any structure)
├── images/              # Assets (any structure)
└── _site/               # Generated output (git-ignored)
```

## Build Pipeline

```rust
fn build_site(source: PathBuf) -> Result<Site> {
    let files = scan_directory(source)?;
    let posts = parse_content(files)?;      // Markdown → AST
    let html = render_templates(posts)?;    // AST → HTML
    let site = optimize_assets(html)?;      // Minify, compress
    Ok(site)
}
```

## Deployment Architecture

**moss.pub** (Zero-config default)
```rust
// No setup required
// Automatic subdomain: username.moss.pub
// Just works on first publish
```

**GitHub Pages** (Developer favorite)
```rust
// Use git2-rs for libgit2 bindings
// Commit to gh-pages branch
// No git CLI dependency
```

**Netlify/Vercel** (Modern static hosting)
```rust
// Direct API integration
// Drag-and-drop folder via API
// Automatic deploys on change
```

**S3-Compatible** (Advanced users)
```rust
// Cloudflare R2, AWS S3, Backblaze B2
// For users who need specific regions
// Or want full infrastructure control
```

## Plugin System

**JavaScript/TypeScript plugins** (not WASM)
```typescript
interface MossPlugin {
  // Metadata
  name: string
  version: string
  
  // Desktop UI extensions
  desktop?: {
    settings?: () => SettingsPanel
    menuItems?: () => MenuItem[]
    previewTools?: () => PreviewTool[]
  }
  
  // Build pipeline hooks
  build?: {
    // Process any file type
    onFile?: (file: File) => File | null
    // Transform parsed content
    onContent?: (content: Content) => Content
    // Modify final HTML
    onHTML?: (html: string) => string
    // After build completes
    postBuild?: (site: Site) => void
  }
  
  // Runtime injection
  inject?: {
    css?: string[]      // Stylesheets to inject
    js?: string[]       // Scripts to inject  
    head?: string       // HTML for <head>
    body?: string       // HTML before </body>
  }
  
  // Deployment providers
  deploy?: {
    name: string
    deploy: (site: Site) => Promise<Result>
  }
}
```

**Core (Rust/Tauri) - Minimal:**
- File system watching
- Basic Markdown → HTML
- Plugin loader/sandbox
- moss.pub deployment

**Everything Else is a Plugin:**
- Themes (CSS injection)
- Syntax highlighting (build hook)
- Image optimization (file processor)
- Social features (runtime injection)
- Analytics (JS injection)
- SEO optimization (head injection)
- RSS generation (post-build)
- Deployment targets (deploy providers)

## Social Protocols

**Micropub** (Incoming)
```rust
// Accept submissions from other sites
POST /micropub
{
  "type": ["h-entry"],
  "properties": {
    "content": ["Submission content"],
    "mp-channel": ["submissions"]
  }
}
```

**ActivityPub** (Federation)
```rust
// Optional, via Spore server
// Not embedded in desktop app
// Separate deployment
```

**WebMention** (Bidirectional)
```rust
// Send mentions on publish
// Receive via static webhook → rebuild
```

## Spore Server (Optional)

Separate Rust binary for social features:

```toml
# Deployment options
[deployment]
mode = "single-tenant"  # or "multi-tenant"
database = "sqlite"     # or "postgres"
storage = "local"       # or "s3"
```

## Lichen Widget

Pure JavaScript, no framework:

```javascript
// Single file, <50KB
// No dependencies
// Works on any static site
class Lichen {
  constructor(element, options) {
    this.backend = options.backend || 'https://spore.moss.pub';
    this.auth = options.auth || 'anonymous';
    this.render();
  }
}
```

## Performance Targets

- **Cold start**: <500ms
- **Build 100 posts**: <5s
- **Incremental build**: <1s
- **Deploy to GitHub**: <30s
- **Binary size**: <20MB
- **Memory usage**: <100MB

## Security Model

- **No network access** without user action
- **Sandboxed plugins** via WASM
- **Local-only by default** (explicit publish)
- **No telemetry** ever
- **No auto-updates** (user controls)

## Dependencies

**Rust Crates** (Core)
- `tauri` - Desktop framework
- `tokio` - Async runtime
- `pulldown-cmark` - Markdown parsing
- `tera` - Template engine
- `git2` - Git operations

**Avoid**
- Node.js dependencies in core
- Native binary dependencies
- Platform-specific code (where possible)
- External services for core features

## Open Decisions

- Template engine: Tera vs Handlebars
- Plugin language: WASM vs JavaScript
- Config format: TOML vs YAML
- Theme format: CSS-only vs full templates

---

*Architecture decisions prioritize: simplicity, portability, user control, and zero-configuration operation.*