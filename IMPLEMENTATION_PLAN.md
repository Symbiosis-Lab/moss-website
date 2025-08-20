# Moss Implementation Plan - Phase 0: Silent Foundation

> Building the irreducible core: right-click → publish → done

## Stage 1: Tray Foundation ✅ COMPLETED
**Goal**: Establish stable system tray presence
**Success Criteria**: Moss appears in menu bar with functional menu
**Status**: Complete - All tests passing, tray icon visible and responsive

## Stage 2: Core Publishing Pipeline 🚧 IN PROGRESS
**Goal**: Implement the essential "folder → website" transformation
**Success Criteria**: User can select folder, generate static site, and publish to moss.pub

### 2.1 OS Context Menu Integration
- [ ] Register "Publish to Web" in macOS Finder context menu
- [ ] Handle folder path passed from OS when right-clicked
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
- [ ] Design proper moss-themed tray icon
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

*Next milestone: First successful folder → moss.pub publish within 1 week*