# Finder Sync Extension for moss

## Overview

This document describes the design and implementation of a macOS Finder Sync Extension to provide reliable right-click "Publish" integration in Finder.

**Status:** Planned
**Priority:** High (Better UX than Automator Services)
**Date:** 2025-10-08

## Why Finder Sync Extension?

### Problems with Current Approach (Automator Services)

1. **Cache Issues**: Services don't appear immediately after installation
2. **Unreliable**: Requires `killall Finder` or logout to register
3. **No Visual Feedback**: No way to show syncing status or badges
4. **Limited Scope**: Can only add menu items, nothing else

### Benefits of Finder Sync Extension

1. **Reliable**: Appears immediately, no cache issues
2. **Industry Standard**: How Dropbox, Google Drive, OneDrive work
3. **Rich Features**: Badges, toolbar buttons, context menus
4. **Better UX**: Can show status, errors, progress
5. **Native**: Official Apple API since macOS 10.10+

## Architecture

```
moss.app
├── Contents/
│   ├── MacOS/
│   │   └── moss                    # Main Tauri app
│   ├── PlugIns/                    # Extension bundles
│   │   └── MossFinderSync.appex/   # Finder Sync Extension
│   │       ├── Contents/
│   │       │   ├── MacOS/
│   │       │   │   └── MossFinderSync  # Extension binary (Rust)
│   │       │   └── Info.plist      # Extension configuration
│   │       └── ...
│   └── Resources/
│       └── ...
```

### Communication Flow

```
User right-clicks folder in Finder
  ↓
Finder Sync Extension context menu appears
  ↓
User clicks "Publish"
  ↓
Extension sends notification → Main app via:
  - NSDistributedNotificationCenter (async)
  - Or XPC Service (sync, more reliable)
  ↓
Main app receives folder path
  ↓
Opens moss://publish?path=<folder> deep link
  ↓
Existing compilation workflow runs
```

## Implementation Plan

### Phase 1: Create Extension Target

**Tools:** Xcode + Cargo

1. Create new directory: `src-tauri/MossFinderSync/`
2. Add `Cargo.toml` for extension
3. Create Xcode project for bundling (or use tauri-bundle modifications)
4. Configure Info.plist with:
   - `NSExtension` configuration
   - App Group identifier
   - Monitored directory URLs

### Phase 2: Implement FIFinderSync in Rust

**Dependencies:**
- `objc2-finder-sync` - Rust bindings for FIFinderSync
- `objc2-foundation` - NSNotification, NSURL, etc.
- `block2` - Objective-C blocks

**Core Implementation:**

```rust
use objc2_finder_sync::{FIFinderSync, FIFinderSyncController};
use objc2_foundation::{NSMenu, NSMenuItem, NSURL};

#[objc2::extern_class(
    unsafe,
    name = "MossFinderSync",
    inherits = FIFinderSync,
)]
pub struct MossFinderSync;

impl MossFinderSync {
    // Register root directory to monitor all folders
    pub fn init_monitoring(&self) {
        let controller = FIFinderSyncController::defaultController();
        let root_url = NSURL::fileURLWithPath("/");
        controller.setDirectoryURLs([root_url]);
    }

    // Add "Publish" context menu item
    pub fn menuForMenuKind(&self, menu_kind: FIMenuKind) -> NSMenu {
        let menu = NSMenu::new();

        if menu_kind == FIMenuKindContextualMenuForItems {
            let item = NSMenuItem::new();
            item.setTitle("Publish");
            item.setTarget(self);
            item.setAction("publishAction:");
            menu.addItem(item);
        }

        menu
    }

    // Handle menu action
    pub fn publishAction(&self, sender: NSMenuItem) {
        let controller = FIFinderSyncController::defaultController();
        if let Some(target_url) = controller.targetedURL() {
            self.send_publish_notification(target_url);
        }
    }

    // Send notification to main app
    fn send_publish_notification(&self, folder_url: NSURL) {
        let center = NSDistributedNotificationCenter::defaultCenter();
        let notification = NSNotification::new(
            "com.moss.publisher.publish",
            folder_url.path()
        );
        center.postNotification(notification);
    }
}
```

### Phase 3: Main App Integration

**Modify main.rs to listen for notifications:**

```rust
use objc2_foundation::{NSDistributedNotificationCenter, NSNotification};

fn setup_finder_sync_listener(app: &tauri::AppHandle) {
    let center = NSDistributedNotificationCenter::defaultCenter();
    let app_handle = app.clone();

    center.addObserverForName(
        "com.moss.publisher.publish",
        block2::block!(|notification: NSNotification| {
            if let Some(folder_path) = notification.object() {
                let url = format!("moss://publish?path={}", folder_path);
                handle_deep_link_url(&app_handle, &url);
            }
        })
    );
}
```

### Phase 4: Bundle Configuration

**Extension Info.plist:**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>moss Finder Sync</string>
    <key>CFBundleExecutable</key>
    <string>MossFinderSync</string>
    <key>CFBundleIdentifier</key>
    <string>com.moss.publisher.FinderSync</string>
    <key>CFBundleName</key>
    <string>MossFinderSync</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSExtension</key>
    <dict>
        <key>NSExtensionPointIdentifier</key>
        <string>com.apple.FinderSync</string>
        <key>NSExtensionPrincipalClass</key>
        <string>MossFinderSync</string>
    </dict>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 moss. All rights reserved.</string>
</dict>
</plist>
```

**App Group Configuration (for shared data):**

```xml
<!-- In main app Info.plist -->
<key>com.apple.security.application-groups</key>
<array>
    <string>group.com.moss.publisher</string>
</array>
```

### Phase 5: Tauri Build Integration

**Modify tauri.conf.json:**

```json
{
  "bundle": {
    "active": true,
    "resources": [
      "../resources",
      "icons/menu-bar"
    ],
    "macOS": {
      "extensions": [
        {
          "path": "target/release/libmoss_finder_sync.dylib",
          "bundlePath": "PlugIns/MossFinderSync.appex/Contents/MacOS/MossFinderSync"
        }
      ]
    }
  }
}
```

**Build script additions:**

```bash
# Build extension
cd src-tauri/MossFinderSync
cargo build --release --lib

# Copy to PlugIns directory
mkdir -p ../target/release/bundle/macos/moss.app/Contents/PlugIns/MossFinderSync.appex/Contents/MacOS
cp target/release/libmoss_finder_sync.dylib \
   ../target/release/bundle/macos/moss.app/Contents/PlugIns/MossFinderSync.appex/Contents/MacOS/MossFinderSync
```

## Technical Considerations

### 1. App Sandbox

Finder Sync Extensions **require** App Sandbox to be enabled:
- Extension must run in sandbox
- Main app should also run in sandbox for consistency
- Requires entitlements for file access

**Entitlements needed:**

```xml
<key>com.apple.security.app-sandbox</key>
<true/>
<key>com.apple.security.files.user-selected.read-write</key>
<true/>
<key>com.apple.security.application-groups</key>
<array>
    <string>group.com.moss.publisher</string>
</array>
```

### 2. Monitored Directories

**Options:**

1. **Monitor root directory (`/`)**: Shows "Publish" for ALL folders
   - Pros: Universal availability
   - Cons: May be considered overreach

2. **Monitor user directories**: `~/Desktop`, `~/Documents`, etc.
   - Pros: More respectful of user space
   - Cons: Limited to specific folders

3. **Dynamic monitoring**: Let user choose folders
   - Pros: User control
   - Cons: Requires settings UI

**Recommendation:** Start with option 2 (user directories), add option 3 later.

### 3. Communication Methods

**Option A: NSDistributedNotificationCenter (Recommended)**

Pros:
- Simple, asynchronous
- Works across processes
- No additional setup

Cons:
- One-way communication only
- No return values
- Delivery not guaranteed

**Option B: XPC Service**

Pros:
- Bidirectional communication
- Synchronous responses possible
- More reliable delivery

Cons:
- More complex setup
- Requires separate XPC service target

**Decision:** Use NSDistributedNotificationCenter for initial implementation.

### 4. Extension Lifecycle

- Extension loaded when Finder starts
- Extension unloaded when not needed (no folders monitored are visible)
- Extension reloaded when monitored folder is viewed
- Main app doesn't need to be running for extension to work

## Testing Strategy

### Manual Testing

1. **Installation:**
   - Install moss.app
   - Open System Settings → Privacy & Security → Extensions
   - Verify "moss Finder Sync" is listed
   - Enable the extension

2. **Context Menu:**
   - Navigate to monitored folder in Finder
   - Right-click folder
   - Verify "Publish" appears in context menu

3. **End-to-End:**
   - Click "Publish"
   - Verify main app launches (if not running)
   - Verify compilation starts
   - Verify preview window opens

### Automated Testing

1. **Extension Loading:**
   - Use `pluginkit` command to verify extension is registered
   - `pluginkit -m -v | grep moss`

2. **Communication:**
   - Mock NSDistributedNotificationCenter
   - Verify notifications are sent with correct data

3. **Integration:**
   - Test deep link handler receives correct folder paths
   - Verify compilation workflow triggers

## Migration Strategy

### Keeping Automator Services as Fallback

**Rationale:** Some users may not enable the extension

**Implementation:**
1. Keep Automator Service installation code
2. Check if Finder Sync Extension is enabled
3. If not, fall back to Automator Service
4. Show prompt to enable extension for better experience

**User Settings:**
```
Finder Integration:
○ Finder Sync Extension (Recommended)
○ Automator Service (Legacy)
○ Disabled
```

## Timeline

| Phase | Effort | Dependencies |
|-------|--------|--------------|
| Phase 1: Create Extension Target | 2 hours | Xcode, Cargo |
| Phase 2: Implement FIFinderSync | 4 hours | objc2-finder-sync |
| Phase 3: Main App Integration | 2 hours | - |
| Phase 4: Bundle Configuration | 2 hours | Tauri build system |
| Phase 5: Testing & Debugging | 4 hours | - |
| **Total** | **14 hours** | |

## Future Enhancements

1. **Badges**: Show sync status on folders
2. **Toolbar Button**: Quick access to moss functions
3. **Sidebar Icon**: Custom icon in Finder sidebar
4. **Progress Indicators**: Show compilation progress
5. **Error States**: Visual indication of failed compilations

## References

- [Apple Finder Sync Programming Guide](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/Finder.html)
- [objc2-finder-sync Rust crate](https://docs.rs/objc2-finder-sync/latest/objc2_finder_sync/)
- [Codecentric Finder Sync Tutorial](https://www.codecentric.de/wissens-hub/blog/finder-sync-extension)
- [Tauri App Extensions Issue #9586](https://github.com/tauri-apps/tauri/issues/9586)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-10-08 | Use Finder Sync Extension over Automator | Better reliability, richer features, industry standard |
| 2025-10-08 | Monitor user directories initially | Balance between availability and respect for user space |
| 2025-10-08 | Use NSDistributedNotificationCenter | Simpler implementation, sufficient for our needs |
| 2025-10-08 | Keep Automator as fallback | Provide option for users who don't enable extension |
