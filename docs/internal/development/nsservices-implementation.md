# NSServices Implementation for moss Context Menu

## Overview

Research findings on implementing macOS context menu integration using NSServices (the approach used by c2pa-preview), comparing it with our current Automator workflow approach and Finder Sync Extension plan.

**Date:** 2025-10-08
**Status:** Research Complete, Implementation Planned
**Reference Project:** [c2pa-preview](https://github.com/ok-nick/c2pa-preview)

---

## Key Discovery: Top-Level Context Menu is Achievable

### The Secret: Removing NSIconName

**Finding:** NSServices can appear at the TOP LEVEL of context menus (not in Services submenu) by:

1. **NOT including `NSIconName` key** in Info.plist
2. Having **fewer than 5 active services** for the given file type

**Evidence:**
- c2pa-preview achieves top-level menu placement
- Our Automator workflow also appeared at top level (when it worked)
- Confirmed by Stack Overflow: [MacOS quick action or service automator](https://stackoverflow.com/questions/69771698/)

### Quick Actions vs Services

**macOS Mojave (10.14+) introduced confusion:**

| Type | Info.plist | Menu Location | Conversion |
|------|-----------|---------------|------------|
| **Quick Action** | Has `NSIconName` key | "Quick Actions" submenu | Remove `NSIconName` → becomes Service |
| **Service** | NO `NSIconName` key | Top-level context menu* | Add `NSIconName` → becomes Quick Action |

*If ≤5 services for that file type; otherwise goes to "Services" submenu

**Command to convert:**
```bash
plutil -remove NSServices.0.NSIconName /path/to/action.workflow/Contents/Info.plist
```

---

## Current Implementation Analysis

### What We Have: Automator Workflow

**Location:** `resources/services/Publish.workflow/`

**Info.plist Configuration:**
```xml
<key>NSServices</key>
<array>
  <dict>
    <key>NSBackgroundColorName</key>
    <string>background</string>
    <key>NSMenuItem</key>
    <dict>
      <key>default</key>
      <string>Publish</string>
    </dict>
    <key>NSMessage</key>
    <string>runWorkflowAsService</string>
    <key>NSRequiredContext</key>
    <dict>
      <key>NSApplicationIdentifier</key>
      <string>com.apple.finder</string>
    </dict>
    <key>NSSendFileTypes</key>
    <array>
      <string>public.folder</string>
    </array>
  </dict>
</array>
```

**Status:** ✅ No `NSIconName` - should appear at top level

**Installation:** [system.rs:176-227](../../src-tauri/src/system.rs#L176)
- Copies workflow from bundled resources to `~/Library/Services/`
- Triggered on first launch ([main.rs:66-93](../../src-tauri/src/main.rs#L66))

**Problem:** Not appearing in context menu after installation
**Likely Cause:** Services cache not refreshed (requires `killall pbs` or logout)

---

## Implementation Options Comparison

### Option 1: Fix Current Automator Workflow (Minimal Change)

**Keep:** Automator workflow approach
**Fix:** Services cache refresh issue

**Pros:**
- ✅ Already implemented
- ✅ No code changes needed
- ✅ Appears at top level (when it works)
- ✅ Minimal risk

**Cons:**
- ❌ Still has cache issues
- ❌ Relies on Automator runtime
- ❌ Harder to debug
- ❌ Not pure Rust

**Implementation:**
1. Add post-install cache refresh: `killall -u $USER pbs`
2. Add user prompt if automatic refresh fails
3. Test on clean system

**Effort:** 1-2 hours

---

### Option 2: Pure Rust NSServices (c2pa-preview Style) ⭐ RECOMMENDED

**Replace:** Automator workflow with native Rust implementation
**Keep:** NSServices approach (top-level menu)

**Pros:**
- ✅ Pure Rust implementation
- ✅ Better control and debugging
- ✅ Can programmatically refresh cache
- ✅ Appears at top level
- ✅ Simpler than Finder Sync Extension
- ✅ Matches c2pa-preview proven approach

**Cons:**
- ❌ Requires objc2 bindings
- ❌ Need to handle NSPasteboard
- ❌ Still has cache issues (inherent to NSServices)
- ❌ Moderate implementation effort

**Architecture:**

```
Main App (moss.app)
├── Info.plist                      # Declares NSServices
├── Contents/MacOS/moss             # Contains service handler
└── Service Handler (Rust)
    ├── Registers with NSApplication
    ├── Receives NSPasteboard from Finder
    └── Triggers moss://publish deep link
```

**Implementation:**
1. Add NSServices declaration to main app Info.plist
2. Implement service handler in Rust using objc2
3. Register handler on app startup
4. Handle file paths from NSPasteboard
5. Trigger existing deep link workflow

**Effort:** 4-6 hours

**Key Difference from c2pa-preview:**
- c2pa-preview: Standalone app, service always available
- moss: Menu bar app, service only available when app running
- **Solution:** Make moss auto-start or use launch agent

---

### Option 3: Finder Sync Extension (Original Plan)

**See:** [finder-sync-extension.md](finder-sync-extension.md)

**Pros:**
- ✅ No cache issues
- ✅ Appears immediately
- ✅ Rich features (badges, toolbar)
- ✅ Industry standard

**Cons:**
- ❌ Much more complex (14+ hours)
- ❌ Requires separate extension binary
- ❌ Only works in monitored folders
- ❌ Requires user permission
- ❌ Overkill for our use case

**Effort:** 14+ hours

---

## Recommended Approach: Hybrid Strategy

### Phase 1: Fix Current Implementation (Quick Win)

**Goal:** Get Automator workflow working reliably

**Actions:**
1. Verify `NSIconName` is NOT in Info.plist ✅ (already correct)
2. Add automatic cache refresh after installation
3. Add fallback: show user instructions if refresh fails

**Code Changes:**

```rust
// In system.rs, after copy_dir_all()
pub fn refresh_services_cache() -> Result<(), String> {
    use std::process::Command;

    // Kill Launch Services cache daemon to refresh services
    let output = Command::new("killall")
        .args(&["-u", &std::env::var("USER").unwrap_or_default(), "pbs"])
        .output()
        .map_err(|e| format!("Failed to refresh services: {}", e))?;

    if !output.status.success() {
        // pbs might not be running, that's okay
        eprintln!("Note: Could not refresh services cache automatically");
    }

    Ok(())
}

// Call after workflow installation
pub fn install_finder_integration() -> Result<String, String> {
    // ... existing code ...
    copy_dir_all(&resource_path, &workflow_path)?;

    // Refresh services cache
    refresh_services_cache().ok(); // Don't fail if refresh fails

    Ok("Finder integration installed! Right-click any folder → 'Publish'")
}
```

**Testing:**
1. Uninstall app completely
2. Delete `~/Library/Services/Publish.workflow`
3. Clear caches: `killall pbs`
4. Reinstall and test

**Effort:** 1 hour

---

### Phase 2: Migrate to Pure Rust (Better Long-term)

**Goal:** Replace Automator with native Rust NSServices

**Why:**
- More control over service lifecycle
- Better error handling
- Can implement programmatic cache refresh
- Easier to debug
- Pure Rust codebase

**Implementation Plan:**

#### Step 1: Add NSServices to main app Info.plist

**File:** `src-tauri/Info.plist`

```xml
<key>NSServices</key>
<array>
  <dict>
    <key>NSMenuItem</key>
    <dict>
      <key>default</key>
      <string>Publish</string>
    </dict>
    <key>NSMessage</key>
    <string>publishFolder:userData:error:</string>
    <key>NSRequiredContext</key>
    <dict>
      <key>NSApplicationIdentifier</key>
      <string>com.apple.finder</string>
    </dict>
    <key>NSSendFileTypes</key>
    <array>
      <string>public.folder</string>
    </array>
  </dict>
</array>
```

**Note:** NO `NSIconName` key → appears at top level

#### Step 2: Implement Service Handler in Rust

**New File:** `src-tauri/src/services.rs`

```rust
use objc2::rc::Id;
use objc2::runtime::AnyObject;
use objc2_app_kit::NSApplication;
use objc2_foundation::{
    NSArray, NSError, NSObject, NSPasteboard, NSString, NSUpdateDynamicServices
};

/// Service handler class for macOS NSServices
#[objc2::extern_class(
    unsafe,
    name = "MossServiceHandler",
    inherits = NSObject
)]
pub struct ServiceHandler;

impl ServiceHandler {
    /// Handle "Publish" service invocation from Finder
    ///
    /// Called when user selects "Publish" from context menu on a folder.
    /// Receives folder path via NSPasteboard and triggers moss:// deep link.
    #[objc2::method(publishFolder:userData:error:)]
    unsafe fn publish_folder(
        &self,
        pboard: &NSPasteboard,
        user_data: &NSString,
        error: *mut *mut NSError,
    ) {
        // Get file paths from pasteboard
        let file_type = NSString::from_str("public.file-url");

        if let Some(paths) = pboard.propertyListForType(&file_type) {
            // Cast to NSArray
            let paths_array: Id<NSArray<NSString>> = unsafe {
                Id::cast(paths)
            };

            if let Some(path) = paths_array.first() {
                let path_str = path.as_str();

                // Trigger existing deep link handler
                let deep_link = format!("moss://publish?path={}",
                    urlencoding::encode(path_str));

                // Send to main app
                if let Err(e) = crate::handle_deep_link_url(&deep_link) {
                    eprintln!("Failed to handle publish: {}", e);

                    // Set error if pointer provided
                    if !error.is_null() {
                        // Create NSError...
                    }
                }
            }
        }
    }
}

/// Register NSServices handler on app startup
pub fn register_services(app: &tauri::AppHandle) -> Result<(), String> {
    unsafe {
        // Get NSApplication instance
        let ns_app = NSApplication::sharedApplication();

        // Create and register service provider
        let handler = ServiceHandler::new();
        ns_app.setServicesProvider(Some(&handler));

        // Force refresh of services cache
        NSUpdateDynamicServices();

        println!("✅ Registered NSServices handler");
    }

    Ok(())
}
```

**Dependencies to add:**

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-app-kit = "0.2"
objc2-foundation = "0.2"
urlencoding = "2.1"
```

#### Step 3: Wire up in main.rs

```rust
#[cfg(target_os = "macos")]
mod services;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                // Register NSServices handler
                if let Err(e) = services::register_services(app) {
                    eprintln!("⚠️ Failed to register services: {}", e);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### Step 4: Remove Automator Workflow

1. Delete `resources/services/Publish.workflow/`
2. Remove installation code from `system.rs`
3. Update `tauri.conf.json` resources list

**Effort:** 4-6 hours

---

## Cache Refresh Solutions

### The Root Problem

NSServices are cached by `pbs` (Pasteboard Services) daemon. Changes don't appear until:
1. `pbs` daemon is restarted: `killall pbs` (user-specific)
2. User logs out/in
3. System reboots

### Solutions Comparison

| Method | Reliability | User Impact | Implementation |
|--------|------------|-------------|----------------|
| `killall pbs` | High | None (automatic) | Easy |
| `NSUpdateDynamicServices()` | Low | None | Easy (Rust only) |
| Logout prompt | High | Requires logout | Easy |
| Launch Agent | Medium | None | Complex |

### Recommended: Multi-layer Approach

```rust
pub fn ensure_services_registered() -> Result<(), String> {
    // Layer 1: Programmatic refresh (Rust NSServices only)
    #[cfg(target_os = "macos")]
    unsafe { NSUpdateDynamicServices(); }

    // Layer 2: Kill pbs daemon
    let _ = std::process::Command::new("killall")
        .args(&["-u", &std::env::var("USER").unwrap_or_default(), "pbs"])
        .output();

    // Layer 3: Check if service appears (TODO)
    // Could use System Events to query Services menu

    // Layer 4: Show user instructions as fallback
    // "Please log out and log back in to activate Publish menu"

    Ok(())
}
```

---

## Testing Strategy

### Test Cases

1. **Fresh Install**
   - Clean system, no existing services
   - Verify "Publish" appears after install
   - Location: Top-level context menu (not submenu)

2. **Reinstall**
   - Existing `Publish.workflow` in Services
   - Verify clean removal and reinstall
   - No duplicate entries

3. **Cache Refresh**
   - Service appears without logout
   - Test `killall pbs` effectiveness
   - Test `NSUpdateDynamicServices()` effectiveness

4. **Multiple Services**
   - Install 5+ other services for folders
   - Verify "Publish" goes to Services submenu
   - Still functional

5. **Menu Placement**
   - Verify "Publish" NOT in "Quick Actions" submenu
   - Verify "Publish" NOT in "Services" submenu (if <5 services)
   - Verify "Publish" IS at top level

### Testing Checklist

- [ ] Right-click folder → "Publish" appears at top level
- [ ] Click "Publish" → moss window opens
- [ ] Click "Publish" → compilation starts
- [ ] Works on Desktop
- [ ] Works in Documents
- [ ] Works in arbitrary folders
- [ ] Works after app restart
- [ ] Works after system restart
- [ ] No duplicate menu items
- [ ] Menu item disabled/grayed appropriately (files vs folders)

---

## Decision Matrix

### Automator vs Pure Rust vs Finder Sync

| Criteria | Automator Workflow | Pure Rust NSServices | Finder Sync Extension |
|----------|-------------------|---------------------|----------------------|
| **Implementation Time** | Done (just fix) | 4-6 hours | 14+ hours |
| **Top-Level Menu** | ✅ Yes | ✅ Yes | ✅ Yes |
| **Cache Issues** | ⚠️ Yes | ⚠️ Yes | ✅ No |
| **Debugging** | ❌ Hard | ✅ Easy | ⚠️ Medium |
| **Code Quality** | ❌ Automator | ✅ Pure Rust | ✅ Pure Rust |
| **Maintenance** | ⚠️ Medium | ✅ Easy | ❌ Complex |
| **Works Everywhere** | ✅ Yes | ✅ Yes | ❌ Monitored only |
| **Rich Features** | ❌ No | ❌ No | ✅ Yes |
| **User Permissions** | ✅ None | ✅ None | ❌ Required |
| **Rust Integration** | ❌ None | ✅ Native | ✅ Native |

**Scores:**
- Automator: 5/10 (works but messy)
- Pure Rust NSServices: **9/10** (best balance)
- Finder Sync: 7/10 (overkill)

---

## Recommended Path Forward

### Immediate (This Week)

**Fix Current Implementation:**
1. Add `killall pbs` after workflow installation
2. Test on clean system
3. Document logout requirement if needed

**Deliverable:** Working "Publish" context menu

**Effort:** 1-2 hours

### Short-term (Next Sprint)

**Migrate to Pure Rust:**
1. Implement Rust NSServices handler
2. Test side-by-side with Automator
3. Switch to Rust implementation
4. Remove Automator workflow

**Deliverable:** Pure Rust context menu integration

**Effort:** 4-6 hours

### Long-term (Future)

**Consider Finder Sync Extension IF:**
- We add file sync features
- We need status badges
- We need toolbar buttons
- Users request it

**Otherwise:** Pure Rust NSServices is sufficient

---

## Key Learnings

### NSServices Best Practices

1. **Top-Level Menu:** Don't include `NSIconName` in Info.plist
2. **Cache Refresh:** Always call `killall pbs` or `NSUpdateDynamicServices()`
3. **Always NSRequiredContext:** Even if empty dict
4. **Service Handler:** Must match `NSMessage` value exactly
5. **Pasteboard Types:** Use `public.file-url` for file paths
6. **App Must Run:** NSServices only work when app is running (use launch agent)

### Common Pitfalls

1. **"Services" submenu:** Happens with `NSIconName` or "Quick Actions" in menu text
2. **Cache not refreshed:** Service registered but not visible
3. **Wrong pasteboard type:** Using `NSStringPboardType` instead of `public.file-url`
4. **Method signature:** Must match Objective-C conventions exactly
5. **App not running:** Service grayed out if app isn't running

### Why c2pa-preview Works

1. NO `NSIconName` → top-level menu ✅
2. Proper NSRequiredContext configuration ✅
3. Handles `public.file-url` from NSPasteboard ✅
4. Registers handler on app startup ✅
5. Calls `NSUpdateDynamicServices()` ✅

We should copy this exact approach.

---

## References

### Documentation
- [Apple: Services Properties](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/SysServices/Articles/properties.html)
- [Apple: Providing Services](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/SysServices/Articles/providing.html)
- [macOS Automation: Automator Quick Actions](https://www.macosxautomation.com/automator/services/)

### Code Examples
- [c2pa-preview Finder Integration](https://github.com/ok-nick/c2pa-preview/blob/main/backend/src/file_ext/finder.rs)
- [c2pa-preview Info.plist](https://github.com/ok-nick/c2pa-preview/blob/main/backend/Info.plist)

### Stack Overflow
- [MacOS quick action in context menu NOT quick actions submenu](https://stackoverflow.com/questions/69771698/)
- [How to add a mac OS service item directly in the contextual menu](https://stackoverflow.com/questions/41566158/)

### Crates
- [objc2](https://crates.io/crates/objc2) - Rust Objective-C bindings
- [objc2-app-kit](https://crates.io/crates/objc2-app-kit) - AppKit framework
- [objc2-foundation](https://crates.io/crates/objc2-foundation) - Foundation framework

---

## Conclusion

**Answer: YES, we should follow the c2pa-preview approach.**

Pure Rust NSServices provides the best balance of:
- Simplicity (simpler than Finder Sync Extension)
- Performance (native code, no Automator runtime)
- Maintainability (pure Rust codebase)
- User Experience (top-level context menu)

The implementation is straightforward:
1. Add NSServices declaration to Info.plist (WITHOUT `NSIconName`)
2. Implement Objective-C service handler in Rust using objc2
3. Register handler on app startup
4. Handle file paths from NSPasteboard
5. Refresh services cache

This gives us exactly what we need: a simple, reliable "right-click to publish" workflow without the complexity of Finder Sync Extensions.
