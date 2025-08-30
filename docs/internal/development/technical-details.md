# moss Technical Implementation Details

## Core Stack

**Desktop Application**
- Tauri v2 (Rust backend, web frontend)
- Menu bar/system tray only (no main window)
- Native OS integration for right-click context menu on folders
- ~15MB binary size target

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

## Plugin System Implementation

**JavaScript/TypeScript plugins**

```typescript
interface mossPlugin {
  // Metadata
  name: string;
  version: string;

  // Desktop UI extensions
  desktop?: {
    settings?: () => SettingsPanel;
    menuItems?: () => MenuItem[];
    previewTools?: () => PreviewTool[];
  };

  // Build pipeline hooks
  build?: {
    // Process any file type
    onFile?: (file: File) => File | null;
    // Transform parsed content
    onContent?: (content: Content) => Content;
    // Modify final HTML
    onHTML?: (html: string) => string;
    // After build completes
    postBuild?: (site: Site) => void;
  };

  // Runtime injection
  inject?: {
    css?: string[]; // Stylesheets to inject
    js?: string[]; // Scripts to inject
    head?: string; // HTML for <head>
    body?: string; // HTML before </body>
  };

  // Deployment providers
  deploy?: {
    name: string;
    deploy: (site: Site) => Promise<Result>;
  };
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

## Social Protocols Implementation

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

## Lichen Widget Implementation

Pure JavaScript, no framework:

```javascript
// Single file, <50KB
// No dependencies
// Works on any static site
class Lichen {
  constructor(element, options) {
    this.backend = options.backend || "https://spore.moss.pub";
    this.auth = options.auth || "anonymous";
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

## Window Management Implementation

**Transparent Window Implementation**

```rust
WindowBuilder::new(app, "control_zone", WindowUrl::App("zone.html"))
    .decorations(false)      // Remove window chrome
    .transparent(true)       // Transparent background
    .always_on_top(true)     // Float above other windows  
    .skip_taskbar(true)      // Hide from taskbar
    .build()
```

**Platform-Specific Transparency Support:**
- **macOS**: Full vibrancy effects via `NSVisualEffectMaterial`
- **Windows**: Blur effects with `window-vibrancy` plugin
- **Linux**: Basic transparency (compositor-dependent)

**Magnetic Positioning System**

Windows maintain spatial relationships using real-time position tracking:

```javascript
// Main preview window coordinates satellite positions
await appWindow.onMoved(async ({ payload: position }) => {
    await updateSatellitePositions(position);
});
```

## API Design Philosophy

**Build only what we know we need right now.**

- Start with the smallest possible API surface
- Add functions only when current implementation proves insufficient
- Prefer single-purpose functions over flexible abstractions
- Easy to extend minimal APIs, impossible to simplify complex ones

**Current Minimal API:**
```rust
// Three functions, no more
publish_folder(path) -> Result<Url, Error>
install_finder_integration() -> Result<(), Error>
get_system_status() -> Result<SystemInfo, Error>
```

When simple functions become insufficient, **then** we refactor to sessions/polling/management.

## Code Organization

**Rust Codebase Structure**

```
src-tauri/src/
├── main.rs           # ~100 lines - app setup only
├── types.rs          # ~100 lines - data structures
└── commands.rs       # ~400 lines - business logic
```

**File Responsibilities:**
- **`main.rs`** - Tauri app setup, tray icon creation, event listeners, entry point
- **`types.rs`** - All structs and enums (`ProjectStructure`, `SystemInfo`, `TrayVisibilityStatus`, etc.)
- **`commands.rs`** - Tauri commands, file scanning, content analysis, system detection

**Design Principles:**
- Keep tests with the code they test (no separate test files)
- Separate data from logic from app setup
- Follow Rust conventions and idiomatic patterns
- Maintain manageable file sizes (100-400 lines)
- Self-explanatory file names that match their contents

This structure balances maintainability with simplicity, avoiding over-engineering while keeping the codebase organized as it grows from the current single 1,300+ line file.

## Implementation Decisions

### HTTP Server vs file:// for Preview

**Problem**: `file://` URLs have CORS restrictions and don't match real deployment behavior

**Solution**: Local Axum HTTP server provides proper web environment

**Benefits**:
- No CORS issues during development
- Relative paths work correctly
- Behavior matches deployed sites exactly
- Cross-platform browser opening
- Automatic port detection (starting at 8080)

**Trade-offs**: +50 lines of code, +2 dependencies (Axum, tower-http), but eliminates entire class of preview/deployment mismatches

### Output Directory Strategy

**Problem**: System temp directories are hard to find and get cleaned up unexpectedly

**Solution**: Co-locate generated sites with source in `.moss/site/` (git-ignored)

**Benefits**:
- Discoverable and persistent
- Follows standard patterns (`.next`, `dist/`, `target/`)
- User can examine generated HTML
- Survives system cleanup cycles
- Clear association between source and output

**Implementation**: Create `.moss/site/` in same directory as source files, add to `.gitignore` patterns

## Open Decisions

- Template engine: Tera vs Handlebars
- Plugin language: WASM vs JavaScript
- Config format: TOML vs YAML
- Theme format: CSS-only vs full templates

---

_Implementation decisions prioritize: simplicity, portability, user control, and zero-configuration operation._