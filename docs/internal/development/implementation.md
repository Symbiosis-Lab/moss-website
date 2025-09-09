# moss Implementation Plan

> Building the irreducible core: right-click → publish → done

## Current Status: Phase 0 - Complete ✅

**Core Publishing Pipeline**: End-to-end "folder → website" transformation working
- ✅ Automatic content analysis and site generation
- ✅ Local HTTP server preview with browser opening  
- ✅ Sites output to `.moss/site/` (git-ignored)
- ✅ Deep link protocol handling (`moss://publish?path=...`)
- ✅ Command-line publishing workflow
- ✅ Finder right-click integration ("Publish" appears in main context menu)
- ✅ Automatic installation on first launch

**Recent Achievements**:
- Resolved Tauri v2 deep link API implementation
- Fixed LaunchServices registration conflicts
- Created working Automator service with proper macOS integration
- Implemented automatic workflow installation system
- Added comprehensive unit test coverage for core business logic
- ✅ **Added menu bar "Publish" button for development and production use**
- ✅ **Implemented directory memory using app config storage**
- ✅ **Created complete preview window architecture with state management**

## Phase 1: Deployment Integration

### 1.1 Zero-Config Publishing
- [ ] Implement moss.pub deployment API
- [ ] Add progress indicators during publish  
- [ ] Handle errors gracefully with user feedback
- [ ] Store published sites for re-publishing

### 1.2 Alternative Deployment Options
- [ ] GitHub Pages integration
- [ ] Netlify/Vercel deployment
- [ ] S3-compatible storage options

## Phase 2: Polish & User Experience

### 2.1 Interface Improvements  
- ✅ Improve tray menu structure and tooltips (added Publish button)
- [ ] Add "Recently Published" submenu
- [ ] Implement notification system for publish status
- ✅ Better error messaging and recovery (added error dialogs)

### 2.2 Developer Features
- [ ] File watching for auto-republish
- [ ] Command-line interface for power users
- [ ] Plugin development tools and documentation

## Phase 3: Community Beta

**Goal**: 20 beta users from personal network
**Success Criteria**: Daily active use, core bugs eliminated

- [ ] Create documentation and onboarding
- [ ] Set up feedback collection system  
- [ ] Implement telemetry (opt-in only)
- [ ] Create moss.pub infrastructure

## Technical Debt & Improvements

### Immediate Polish Tasks (Phase 0 Completion)

#### Preview Window Development Integration
- **Problem**: Preview HTML/JS files not served by Vite dev server in development mode
- **Goal**: Seamless preview window functionality during `cargo run` development
- **Success Criteria**: Preview window loads and displays controls correctly in dev mode
- **Status**: Pending - requires Vite configuration or asset serving strategy

#### Environment-Specific URL Handling  
- **Problem**: Preview windows use hardcoded `http://localhost:8080` regardless of dev/production context
- **Goal**: Dynamic URL generation based on runtime environment
- **Success Criteria**: 
  - Dev mode: Uses Vite dev server URL (`http://localhost:1420`)
  - Production: Uses local preview server (`http://localhost:8080`)
- **Status**: Pending - requires environment detection and URL building logic

#### LaunchServices Registration Cleanup
- **Problem**: Multiple moss protocol registrations causing routing conflicts
- **Goal**: Clean protocol registration state and prevent duplicates
- **Success Criteria**: Single, working protocol registration with reliable deep link routing
- **Commands**: 
  ```bash
  /System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister -dump | grep moss
  # Clean up duplicates as needed
  ```
- **Status**: Pending - requires LaunchServices debugging and cleanup

### Future Improvements

- [ ] Replace programmatic tray icon with designed asset
- [ ] Add proper error handling throughout
- [ ] Implement configuration persistence
- [ ] Add comprehensive logging system
- ✅ Create automated testing for publish pipeline

### Testing Strategy

**Unit Tests (Comprehensive ✅)**
- Core content analysis functions (homepage detection, folder classification)
- URL processing and deep link parsing
- Project type identification and business logic
- Publishing workflow components

**Integration & E2E Testing (Planned)**

The following components require end-to-end testing due to their integration with macOS system services and external dependencies:

#### **System Integration Tests**
- **Finder Integration**: Verify "Publish" appears in right-click context menu for folders
- **Deep Link Registration**: Test `moss://publish?path=...` URLs open moss app correctly
- **First Launch Setup**: Validate automatic workflow installation on fresh app install
- **Services Cache**: Ensure workflow persists after system restarts and user sessions

#### **End-to-End Publishing Workflow**
- **Right-click → Publish**: Complete user journey from Finder to generated website
- **Path Handling**: Test folders with spaces, special characters, nested structures
- **Browser Integration**: Verify generated sites open correctly in default browser
- **File System Operations**: Test `.moss/site/` directory creation and cleanup

#### **Cross-System Compatibility**
- **macOS Version Support**: Test on macOS Big Sur, Monterey, Ventura, Sonoma
- **Permission Handling**: Verify no elevated permissions required
- **App Bundle Distribution**: Test installation from DMG, direct copy, etc.

#### **Error Scenarios & Recovery**
- **Missing Resources**: Behavior when bundled workflow files are corrupted
- **Permission Denied**: Graceful handling when `~/Library/Services` is protected
- **Conflicting Services**: Behavior when other "Publish" services exist
- **Network Issues**: Local HTTP server startup failures, port conflicts

**E2E Test Implementation Strategy**:
- Use temporary test directories for isolated testing
- Mock external services where possible
- Test both successful workflows and error conditions
- Validate user-observable outcomes, not implementation details
- Include performance benchmarks (publish time <5 seconds for typical content)

## Publishing Workflow Definition

The moss publishing workflow consists of these distinct steps:

### 1. **Build** 
Turn source files (Markdown, images, etc.) into web pages
- Parse content and frontmatter
- Generate HTML from Markdown
- Copy assets and create site structure
- Output to `.moss/site/` directory
- Start local HTTP server for preview

### 2. **Preview**
Show user the built site with controls for next actions
- Open preview window with iframe pointing to local server
- Display **floating controls** in two zones around preview window:
  - **Top zone**: Website actions (Publish, Edit, Settings) 
  - **Right zone**: Syndication targets (Twitter, LinkedIn, Dev.to, etc.)
- Allow user to review content before making it public

**Control Window Architecture**:
- Two transparent Tauri windows with `decorations: false` and `transparent: true`
- Magnetic positioning relative to main preview window using `onMoved` events
- Platform-specific transparency: Full vibrancy (macOS), blur effects (Windows), basic transparency (Linux)
- Plugin extensibility: Plugins register controls in appropriate zones

### 3. **Publish**
Upload web pages to hosting platform
- Deploy built site to moss.pub (or other platforms)
- Make content publicly accessible via URL
- Update publication status in preview window

### 4. **Syndicate** 
Share publication across different channels
- Post to social media platforms
- Submit to content aggregators
- Notify subscribers or mailing lists

**Current Status**: Build + Preview architecture implemented. 
- ✅ Build: Compiles content and starts local server
- ✅ Preview: Shows built site in dedicated window with controls  
- 🔄 Publish: Mock implementation (real deployment planned for Phase 1)
- 📋 Syndicate: Planned for Phase 1

## Architecture Decisions

### Technology Stack
- **Backend**: Rust + Tauri 2.8 (stable, excellent documentation)
- **Frontend**: Vanilla JavaScript (sufficient for desktop UI)
- **Static Generation**: pulldown-cmark (fast, spec-compliant)
- **Preview Server**: Axum + tower-http (lightweight, feature-rich)

### Key Design Patterns
- **Local-first**: User data stays on device
- **Zero configuration**: Works out of the box immediately after install
- **Standards-based**: Use existing protocols and patterns (macOS Services, HTTP)
- **Performance-focused**: <5 seconds typical publish time
- **Graceful degradation**: Core functionality works even if system integration fails
- **Test-driven development**: Comprehensive unit tests for business logic, E2E for system integration

## Current Development Status

**Phase 0: Complete ✅ (August 26, 2025)**

**Core Achievement**: Unified publishing interface with dual-entry system
- ✅ **Right-click integration** - Zero-config production experience
- ✅ **Menu bar publishing** - Development tool + production feature  
- ✅ **Preview window architecture** - Complete state management and IPC
- ✅ **Directory memory** - User preference persistence
- ✅ **Consistent branding** - Lowercase "moss" styling across codebase

**Publishing Workflow**: Build → Preview → Publish → Syndicate
- ✅ **Build**: Content analysis, HTML generation, local server startup
- ✅ **Preview**: Dedicated window with Publish/Edit/Syndicate controls
- 🔄 **Publish**: Mock implementation (real deployment in Phase 1)  
- 📋 **Syndicate**: Planned for Phase 1

### Immediate Next Steps (Phase 0 Polish)

**Priority Order**:
1. **Fix dialog positioning and full-screen issues** - Critical for user experience
2. **Fix preview window HTML serving in dev mode** - Critical for development workflow
3. **Update window URLs for dev vs production environments** - Environment detection
4. **Clean up LaunchServices duplicate registrations** - Deep link reliability

#### Dialog Positioning Fix - Implementation Plan

**Issue**: File dialogs appear incorrectly positioned and crash app on cancel (discovered 2025-01-09)

**Root Cause**: 
- `ActivationPolicy::Accessory` causes dialogs to position relative to menu bar icon
- Canceling dialog returns error, causing app crash in dev mode
- Full-screen apps prevent dialog visibility due to window layering

**Solution**: Dynamic activation policy switching (industry standard pattern)

**Implementation Steps**:

1. **Modify `compile_with_directory_picker()` in `src-tauri/src/commands/compile.rs`**:
   ```rust
   // Before dialog
   app.set_activation_policy(tauri::ActivationPolicy::Regular);
   tokio::time::sleep(Duration::from_millis(100)).await;
   
   // Show dialog (now properly centered, can handle full-screen)
   let folder_path = app.dialog().file()...
   
   // In all completion paths (success, error, cancel)
   app.set_activation_policy(tauri::ActivationPolicy::Accessory);
   ```

2. **Handle cancellation as success case, not error**:
   ```rust
   match folder_path {
       Some(path) => { /* proceed with compilation */ },
       None => {
           // User canceled - this is normal, not an error
           app.set_activation_policy(tauri::ActivationPolicy::Accessory);
           Ok("User canceled folder selection".to_string())
       }
   }
   ```

3. **Add proper async/await support**:
   - Change function signature to `async fn`
   - Update Tauri command registration for async

4. **Test all scenarios**:
   - Dialog appears centered ✓
   - Works from full-screen apps ✓
   - Cancel doesn't crash app ✓
   - Dock icon appears/disappears correctly ✓

**Files to modify**:
- `src-tauri/src/commands/compile.rs` (main implementation)
- `src-tauri/src/main.rs` (async command registration)

**Estimated time**: 1 development session

**Estimated Completion**: 2-3 development sessions

### Transition to Phase 1

**Ready for deployment integration** once polish tasks complete:
- moss.pub hosting implementation
- Alternative deployment options (GitHub Pages, Netlify)
- Progress indicators and user feedback systems

## Content Detection Logic

> Simple, clear rules that match how people actually organize content

### Core Philosophy

**Keep it simple.** Most users follow one of two patterns:
1. **Flat directory** - everything in one folder
2. **Homepage + subdirectories** - main files at top, collections in folders

Don't over-engineer. Cover these cases well, ignore edge cases that add complexity.

### Simple Detection Rules

#### 1. Find the Homepage (Priority Order)

```
1. index.md       → Markdown homepage
2. index.pages    → macOS Pages document
3. index.docx     → Word document
4. README.md      → Project description
5. First document file → Fallback content
```

#### 2. Determine Site Type

```
IF has subdirectories with documents:
  → Homepage + Collections
  - Top-level files = main pages (homepage, about, contact)
  - Subdirectories = content collections (blog posts, projects, etc.)

ELIF root has ≤5 document files:
  → Simple Flat Site
  - All files in navigation menu

ELSE:
  → Blog-style Flat Site  
  - Homepage + essential pages in menu
  - Other files listed chronologically on homepage
```

### Two Main Patterns

#### Pattern 1: Flat Directory
```
my-site/
├── index.md           # Homepage
├── about.md           # Page
├── contact.md         # Page  
├── post1.md           # Page
├── post2.md           # Page
└── image.jpg          # Asset

Result: Simple site with navigation menu
```

#### Pattern 2: Homepage + Collections
```
my-site/
├── index.md           # Homepage
├── about.md           # Main page
├── posts/             # Collection
│   ├── hello.md
│   └── update.md
└── projects/          # Collection
    ├── project1.md
    └── project2.md

Result: Homepage + auto-generated collection pages
```

### Supported File Types

#### Document Files (Priority Order)
- `.md` files - Markdown (convert to HTML)
- `.pages` files - macOS Pages (convert to HTML)
- `.docx` files - Word documents (convert to HTML)
- `.doc` files - Legacy Word documents (convert to HTML)

#### Homepage Files
- `index.md` - Markdown homepage
- `index.pages` - Pages homepage
- `index.docx` - Word homepage
- `README.md` - Documentation style
- First document file found - Fallback

#### Other Files
- Images (jpg, png, gif, etc.) - Optimize and copy
- PDFs - Copy as-is for download
- Other files - Copy as-is

*Simple rules. Clear outcomes. No surprises.*

## Plugin Architecture

### Core Strategy: Minimal Core + Plugin Ecosystem

moss achieves unlimited extensibility through a plugin-first architecture. The core remains minimal (~5MB), handling only folder analysis and plugin orchestration. Everything else—SSGs, themes, publishers—lives in plugins.

**Reference**: See [Plugin Architecture Documentation](plugin-architecture.md) for complete technical specifications.

### Phase 1: Plugin Infrastructure Foundation

**Goal**: Establish plugin system with default minimal SSG

**Core Components**:
- Plugin discovery and loading system
- JSON-RPC communication protocol  
- Plugin manifest specification
- Security sandbox for plugin execution

**Default Plugin**: Minimal SSG
- Ultra-lightweight Rust implementation (~200 lines)
- Single beautiful template with moss green branding
- <1 second build time for typical content
- No external dependencies
- Bundled with moss core

**Plugin Interface**:
```rust
pub trait Plugin: Send + Sync {
    fn manifest(&self) -> &PluginManifest;
    fn can_handle(&self, request: &PluginRequest) -> bool;
    fn execute(&self, request: PluginRequest) -> Result<PluginResponse>;
}

pub trait SsgPlugin: Plugin {
    fn required_structure(&self) -> FolderMapping;
    fn supported_themes(&self) -> Vec<ThemeInfo>;
    fn build_site(&self, config: &BuildConfig) -> Result<SiteResult>;
}
```

### Phase 2: Core SSG Plugins

**Goal**: Transform existing SSG integrations into standalone plugins

**Plugin Development Priority**:
1. **Jekyll Plugin** - Largest theme ecosystem (300+ themes)
2. **Hugo Plugin** - High performance, quality themes  
3. **Zola Plugin** - Rust-native, minimal binary
4. **Theme Plugin System** - Visual marketplace integration

**Jekyll Plugin Example**:
```rust
struct JekyllPlugin {
    gem_home: PathBuf,
}

impl SsgPlugin for JekyllPlugin {
    fn build_site(&self, config: &BuildConfig) -> Result<SiteResult> {
        // 1. Create virtual Jekyll structure in .moss/build/
        self.adapt_folder_structure(config.source, config.build_dir)?;
        
        // 2. Generate _config.yml with user preferences
        self.generate_jekyll_config(config)?;
        
        // 3. Execute jekyll build via subprocess
        let result = self.run_jekyll_build(config.build_dir)?;
        
        Ok(SiteResult {
            page_count: result.page_count,
            build_time: result.duration,
            output_path: config.output_dir,
        })
    }
}
```

### Plugin Communication Architecture

**Protocol**: JSON-RPC over subprocess for language-agnostic plugins

**Benefits**:
- Process isolation (plugin crashes don't affect core)
- Security sandbox 
- Language independence
- Compatibility with existing SSG binaries

**Message Flow**:
```rust
// Core → Plugin request
{
  "jsonrpc": "2.0",
  "method": "build_site",
  "params": {
    "source_path": "/Users/jane/blog",
    "output_path": "/Users/jane/blog/.moss/site",
    "theme": "minimal"
  },
  "id": 1
}

// Plugin → Core response  
{
  "jsonrpc": "2.0",
  "result": {
    "page_count": 12,
    "build_time_ms": 850,
    "assets": ["style.css", "main.js"]
  },
  "id": 1
}
```

### Folder Adaptation Through Plugins

**Problem**: SSGs expect specific structures, users prefer their organization

**Solution**: Each plugin defines its required structure mapping

```rust
pub struct FolderMapping {
    pub content_dirs: HashMap<String, String>,
    pub asset_dirs: HashMap<String, String>, 
    pub config_files: Vec<ConfigFile>,
}

impl JekyllPlugin {
    fn required_structure(&self) -> FolderMapping {
        FolderMapping {
            content_dirs: hashmap!{
                "articles" => "_posts",
                "pages" => ".",
            },
            asset_dirs: hashmap!{
                "images" => "assets/images",
                "styles" => "_sass",
            },
            config_files: vec![
                ConfigFile::Generate {
                    path: "_config.yml",
                    template: include_str!("jekyll_config.yml"),
                },
            ],
        }
    }
}
```

### Plugin Discovery and Loading

**Discovery Locations**:
1. Bundled plugins: `Contents/Resources/plugins/`
2. User plugins: `~/.moss/plugins/`  
3. System plugins: `/usr/local/lib/moss/plugins/`

**Loading Process**:
```rust
pub struct PluginManager {
    loaded_plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn load_all_plugins(&mut self) -> Result<()> {
        let manifests = self.discover_plugin_manifests()?;
        let compatible = self.filter_compatible_plugins(manifests)?;
        let ordered = self.resolve_plugin_dependencies(compatible)?;
        
        for manifest in ordered {
            let plugin = self.instantiate_plugin(manifest)?;
            plugin.initialize()?;
            self.loaded_plugins.insert(plugin.name().to_string(), plugin);
        }
        Ok(())
    }
}
```

### Integration with Current Architecture

**Updated Compilation Flow**:
```rust
pub fn compile_folder_with_options(folder_path: String, auto_serve: bool) -> Result<String> {
    // 1. Analyze folder structure (core responsibility)
    let project = analyze_folder_structure(&folder_path)?;
    
    // 2. Select appropriate SSG plugin
    let plugin_manager = PluginManager::instance();
    let ssg_plugin = plugin_manager.select_ssg_plugin(&project)?;
    
    // 3. Execute plugin
    let request = PluginRequest::BuildSite {
        source_path: folder_path.clone(),
        output_path: format!("{}/.moss/site", folder_path),
        project: project,
        theme: None, // Future: user selection
    };
    
    let response = ssg_plugin.execute(request)?;
    
    // 4. Handle result (core responsibility)
    if auto_serve {
        start_site_server(&response.output_path)?;
    }
    
    Ok(response.build_message)
}
```

### Migration Strategy

**Phase 1: Infrastructure** (Current Sprint)
- Implement plugin loading system
- Define plugin API traits
- Create plugin communication protocol
- Extract minimal SSG as first plugin

**Phase 2: Plugin Conversion**
- Convert Jekyll integration to plugin
- Convert Hugo integration to plugin  
- Add theme plugin system
- Implement plugin marketplace

**Phase 3: Ecosystem Growth**
- Third-party plugin support
- Plugin development documentation
- Remote plugin registry
- Community contribution tools

**Backward Compatibility**: Existing functionality preserved during transition

### Plugin Directory Structure

```
.moss/
├── build/              # Temporary SSG-compatible structure (per plugin)
├── site/               # Generated HTML output 
├── plugins/            # User-installed plugins
│   ├── jekyll/         # Plugin directory
│   │   ├── plugin.toml # Plugin manifest
│   │   └── moss-jekyll # Plugin executable
│   └── hugo/           # Another plugin
├── cache/              # Downloaded binaries and themes
├── themes/             # Installed theme files  
├── config.toml         # User configuration
└── logs/               # Plugin execution logs
```

### Success Metrics

**Phase 1 Success (Plugin Infrastructure)**:
- Plugin loading time: <100ms
- Default minimal plugin builds in <1 second
- Zero configuration for new users
- Seamless transition from current implementation

**Phase 2 Success (Core Plugins)**:  
- Jekyll/Hugo plugins functional
- Plugin communication overhead: <5% of build time
- Theme selection via plugins working
- 100+ themes accessible through plugin marketplace

**Long-term Success (Ecosystem)**:
- Third-party plugins available
- Community plugin contributions
- Universal social features via Lichen plugin
- moss as orchestration platform for entire static site ecosystem

## Platform Integration Notes

Platform-specific implementation details and technical references have been moved to code comments where they're directly applicable. Key integration points:

- **Deep Link Registration**: References moved to deep link setup in `main.rs`
- **macOS Services Integration**: References moved to Finder integration code
- **LaunchServices Management**: Debugging commands moved to relevant setup functions

This keeps technical details close to the code where they're needed while maintaining cleaner documentation.

---

*Next milestone: Complete Phase 0 polish → Begin moss.pub deployment integration*