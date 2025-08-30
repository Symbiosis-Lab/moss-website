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
- [ ] Basic plugin system foundation

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
1. **Fix preview window HTML serving in dev mode** - Critical for development workflow
2. **Update window URLs for dev vs production environments** - Environment detection
3. **Clean up LaunchServices duplicate registrations** - Deep link reliability

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

## Platform Integration Details

### Deep Link Development Limitation

**macOS Issue**: Protocol registration requires app installation, not available in `npm run tauri dev`

**Production Reality**: Deep links work fine after build/install - development limitation only

**Workaround**: Direct command testing via UI buttons during development

**Reference**: [Tauri Deep Link Plugin Documentation](https://v2.tauri.app/plugin/deep-link/)

### macOS Services Integration

#### Bundle File Copying vs Programmatic Creation

**Preferred Approach**: Copy pre-built `.workflow` bundles from app resources to `~/Library/Services/`

**Why This Works**:
- More reliable than AppleScript automation
- Version controlled workflow files
- Cross-macOS version compatibility
- No runtime XML generation errors

**Resource Path Discovery**: Tauri bundles resources at `Contents/Resources/_up_/resources/` in production, not the expected `Contents/Resources/`

**Reference**: [Apple Automator Documentation](https://developer.apple.com/library/archive/documentation/LanguagesUtilities/Conceptual/MacAutomationScriptingGuide/MakeaSystem-WideService.html)

#### Services Menu Placement

**Key Finding**: Remove `NSIconName` property from Info.plist to show service in main context menu

**Behavior**:
- **Without NSIconName**: Service appears in main context menu (1 click)
- **With NSIconName**: Service appears in "Quick Actions" submenu (2 clicks)

**Command to Remove**: 
```bash
plutil -remove NSServices.0.NSIconName /path/to/workflow/Contents/Info.plist
```

**Impact**: One property removal saves one click in user workflow

#### First Launch Integration

**Pattern**: Automatic installation on first launch using marker files

**Implementation**:
- **Marker Location**: `~/Library/Application Support/com.moss.publisher/finder_integration_installed`
- **Security**: No elevated permissions required (`~/Library/Services/` is user-writable)
- **Graceful Degradation**: App functions even if automatic installation fails
- **User Control**: Can be manually triggered through Settings if needed

#### LaunchServices Database Management

**Conflict Resolution**: Multiple app registrations (debug/release/production) can create routing conflicts

**Commands for Debugging**:
```bash
# Reset entire LaunchServices database
lsregister -kill -r

# Register specific app
lsregister -f /Applications/moss.app

# View current registrations
lsregister -dump | grep moss
```

---

*Next milestone: Complete Phase 0 polish → Begin moss.pub deployment integration*