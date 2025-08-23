# Moss Implementation Plan

> Building the irreducible core: right-click → publish → done

## Current Status: Phase 0 - 90% Complete

**Core Publishing Pipeline**: End-to-end "folder → website" transformation working
- ✅ Automatic content analysis and site generation
- ✅ Local HTTP server preview with browser opening  
- ✅ Sites output to `.moss/site/` (git-ignored)
- ✅ Test Publish button working (Settings UI)
- ⚠️ Finder integration installed but inconsistent

**Remaining Issue**: Deep link protocol events not reaching Tauri event handler
- Protocol appears registered (browser confirms)
- Events fail to trigger from both CLI and Finder workflow
- System restart test pending

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
- [ ] Create automated testing for publish pipeline

## Architecture Decisions

### Technology Stack
- **Backend**: Rust + Tauri 2.8 (stable, excellent documentation)
- **Frontend**: Vanilla JavaScript (sufficient for desktop UI)
- **Static Generation**: pulldown-cmark (fast, spec-compliant)
- **Preview Server**: Axum + tower-http (lightweight, feature-rich)

### Key Design Patterns
- **Local-first**: User data stays on device
- **Zero configuration**: Works out of the box
- **Standards-based**: Use existing protocols and patterns
- **Performance-focused**: <5 minute first publish

---

*Next milestone: First successful folder → moss.pub publish*