# Moss Implementation Plan

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
- [ ] Improve tray menu structure and tooltips
- [ ] Add "Recently Published" submenu
- [ ] Implement notification system for publish status
- [ ] Better error messaging and recovery

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
- **Deep Link Registration**: Test `moss://publish?path=...` URLs open Moss app correctly
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

---

*Next milestone: First successful folder → moss.pub publish*