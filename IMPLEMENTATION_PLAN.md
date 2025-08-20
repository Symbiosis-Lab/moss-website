# Moss Implementation Plan - Phase 0: Silent Foundation

> Building the irreducible core: right-click → publish → done

## Stage 1: Tray Foundation ✅ COMPLETED
**Goal**: Establish stable system tray presence
**Success Criteria**: Moss appears in menu bar with functional menu
**Status**: Complete - All tests passing, tray icon visible and responsive

## Stage 2: Core Publishing Pipeline 🚧 IN PROGRESS
**Goal**: Implement the essential "folder → website" transformation
**Success Criteria**: User can select folder, generate static site, and publish to moss.pub

### 2.1 OS Context Menu Integration ✅ COMPLETED
- [x] Implement moss:// deep link protocol for receiving folder paths
- [x] Add handler to process deep link publish requests  
- [x] Create automatic Automator Quick Action installation
- [x] Programmatically generate .workflow bundle with proper plist structure
- [x] Fix context menu placement (top-level, not Quick Actions submenu)
- [x] Test complete integration workflow - VERIFIED WORKING
- [ ] Add file system scanning (markdown, html, images)
- [ ] Create basic project structure detection
- [ ] Add ~/.moss/config.toml initialization

### 2.2 Static Site Generation  
- [ ] Add pulldown-cmark dependency for Markdown parsing
- [ ] Implement basic MD → HTML conversion
- [ ] Create minimal CSS framework (beautiful defaults)
- [ ] Add image processing and optimization
- [ ] Generate RSS feed automatically

### 2.3 Zero-Config Publishing
- [ ] Implement moss.pub deployment API
- [ ] Add progress indicators during publish
- [ ] Handle errors gracefully with user feedback
- [ ] Store published sites for re-publishing

## Stage 3: Polish & User Experience
**Goal**: Make first-time experience delightful
**Success Criteria**: <5 minute first publish, no configuration required

### 3.1 Interface Improvements
- [x] Design proper moss-themed tray icon (green plant - perfect for moss theme)
- [ ] Improve menu structure and tooltips
- [ ] Add "Recently Published" submenu
- [ ] Implement notification system for publish status

### 3.2 Developer Features  
- [ ] Add GitHub Pages deployment option
- [ ] Implement file watching for auto-republish
- [ ] Create basic plugin system foundation
- [ ] Add command-line interface for power users

## Stage 4: Community Beta
**Goal**: 20 beta users from personal network
**Success Criteria**: Daily active use, core bugs eliminated
- [ ] Create documentation and onboarding
- [ ] Set up feedback collection system
- [ ] Implement telemetry (opt-in only)
- [ ] Create moss.pub infrastructure

## Technical Debt & Improvements
- [ ] Replace programmatic tray icon with designed asset
- [ ] Add proper error handling throughout
- [ ] Implement configuration persistence
- [ ] Add comprehensive logging system
- [ ] Create automated testing for publish pipeline

## Dependencies to Add
```toml
# Cargo.toml additions needed
pulldown-cmark = "0.9"           # Markdown parsing
tera = "1.19"                   # Template engine  
tokio = { version = "1.0", features = ["full"] }  # Async runtime
reqwest = { version = "0.11", features = ["json"] }  # HTTP client
serde = { version = "1.0", features = ["derive"] }   # Serialization
toml = "0.8"                    # Config parsing
uuid = { version = "1.0", features = ["v4"] }       # Unique IDs
walkdir = "2.4"                 # Directory traversal
```

---

## ✅ Automatic Finder Integration (macOS)

The Quick Action is now **automatically installed** when Moss first launches:

1. **Automatic Installation:** First time the app starts, it creates:
   - `~/Library/Services/Publish to Web.workflow` bundle
   - Proper Info.plist with bundle identifier
   - Complete Automator workflow with shell script action
   
2. **Zero Configuration:** No manual setup required

**Usage:** Right-click any folder in Finder → "Publish to Web" (appears directly in context menu)

The workflow calls `moss://publish?path=/encoded/folder/path` which triggers site generation.

---

## Key Implementation Insights & Learnings

### macOS Finder Integration Challenges Solved

**Context Menu Placement:**
- Automator Quick Actions with `NSIconName` property force items into "Quick Actions" submenu
- **Solution:** Remove `NSIconName` from Info.plist and use proper `NSServices` configuration
- **Result:** "Publish to Web" appears at top level like Google Drive's context menu items

**Deep Link Protocol Requirements:**
- macOS deep links only work in built/installed apps, not during `tauri dev`
- **Critical:** Must test with `npm run tauri build` and install in `/Applications`
- Deep link protocol registration requires specific plugin configuration in `tauri.conf.json`

**Bundle Identifier Best Practices:**
- Never end with `.app` (conflicts with macOS app bundle extension)
- Use reverse domain notation: `com.moss.publisher` (not `com.moss.app`)
- Workflow bundle identifiers should match: `com.moss.publisher.publish-to-web`

**Icon Generation Issues:**
- Tauri template sometimes ships with corrupted/empty `.icns` files
- **Solution:** Always regenerate icons using `npx @tauri-apps/cli icon src-tauri/icons/icon.png`
- Clean `src-tauri/target/` directory after icon regeneration to force refresh

### Tauri-Specific Development Patterns

**Menu Bar App Architecture:**
- Set `ActivationPolicy::Accessory` on macOS to prevent dock icon
- Intercept window close events to hide (not quit) the app
- Essential for menu bar utilities that should persist in background

**Testing Strategy:**
- Backend tests for pure logic (Rust functions)
- Integration tests for mocked frontend-backend communication
- Manual testing required for OS-level integrations (context menus, deep links)

**File System Operations:**
- Always use absolute paths in Tauri commands
- Create directories recursively with `fs::create_dir_all`
- Clean up existing files before installation to ensure fresh state
- Use `std::env::var("HOME")` for cross-user compatibility in macOS paths

**Workflow Bundle Creation:**
- `.workflow` files are actually bundles (directories) with `Contents/` subdirectory
- Require both `Info.plist` (bundle metadata) and `document.wflow` (workflow definition)
- Must include proper `NSServices` array in Info.plist for Services menu integration
- Shell script actions need specific parameter structure and UUIDs for Automator compatibility

### User Experience Insights

**Zero-Configuration Philosophy:**
- Manual setup steps are unacceptable UX for consumer tools
- Automatic installation must happen transparently on first launch
- Users expect behavior similar to Google Drive, Dropbox (top-level context menu)

**Error Communication:**
- Focus on user-observable behaviors when describing testing steps
- Avoid implementation details in user-facing messaging
- Always provide clear "what should happen" expectations

**Development Workflow Best Practices:**
- Never commit partial implementations - only complete, testable features
- Manual verification required before committing OS integration features
- Amend commits to include documentation updates rather than separate commits
- Build and test the complete user journey, not just individual components

### Future Development Considerations

**Static Site Generation Next:**
- Folder scanning with `walkdir` crate
- Markdown processing with `pulldown-cmark`
- Template system with `tera` engine
- Asset optimization pipeline

**Cross-Platform Context Menus:**
- Windows: Shell extensions (more complex than macOS)
- Linux: File manager specific (Nautilus, Dolphin, etc.)
- Each platform requires different integration approach

**URL Encoding & Path Handling:**
- Folder paths with spaces require URL encoding in deep link URLs
- Use `python3 -c "import urllib.parse; print(urllib.parse.quote('$path'))"` in shell scripts
- Decode with `decodeURIComponent()` in JavaScript before passing to Rust backend
- Always validate paths are non-empty before processing

---

*Next milestone: First successful folder → moss.pub publish within 1 week*