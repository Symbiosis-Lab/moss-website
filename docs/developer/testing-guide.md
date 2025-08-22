# Moss Testing Guide

## Philosophy: Test Behavior, Not Implementation

Following the "test behavior, not implementation" principle, we focus on **user-observable outcomes** rather than internal mechanics. Our tests validate what users experience, not how code is structured.

**The Critical Distinction:**
- ❌ **Implementation testing**: Verify icon pixel data is correct
- ✅ **Behavior testing**: Verify tray icon appears and responds to clicks

## Current Test Architecture

### Streamlined Test Suite (4 essential tests)

After eliminating useless implementation tests, we maintain only tests that validate real behavior:

```
moss/
├── src-tauri/src/main.rs     # 4 behavioral tests (business logic only)
├── src-tauri/Cargo.toml     # Tauri 2.8 available for future testing
└── [No JS tests]            # Removed: were testing browser APIs, not our code
```

**Test Categories:**
- **Behavioral Tests (4)**: Core business logic that users depend on
- **Mock runtime tests**: Removed - could not test actual tray/menu creation logic
- **JavaScript tests**: Removed - were testing browser APIs, not our application logic

### Test Categories

#### **1. Content Analysis Tests** (Core Business Logic)
Tests the heart of Moss functionality - analyzing folders for website generation:

```rust
#[test]
fn test_content_analysis_homepage_detection() {
    // Tests priority: index.md > index.pages > index.docx > README.md
    let files = vec![/* various homepage candidates */];
    let result = detect_homepage_file(&files);
    assert_eq!(result, Some("index.md".to_string()));
}
```

#### **2. Finder Integration Tests** (User Workflow)
Tests the complete right-click → publish workflow:

```rust
#[test] 
fn test_deep_link_url_parsing() {
    // Tests moss://publish?path=... URLs from Finder integration
    let test_cases = [
        ("moss://publish?path=/Users/test/folder", Some("/Users/test/folder")),
        ("moss://publish?path=/Users/test/My%20Documents", Some("/Users/test/My Documents")),
    ];
    // Validates URL parsing logic that handles real user folders
}
```

#### **3. Publishing Workflow Tests** (End-to-End Logic)
Tests folder analysis and publishing pipeline:

```rust
#[test]
fn test_folder_publishing_workflow() {
    // Tests complete publishing logic: validation → analysis → results
    let result = publish_folder("/valid/path");
    assert!(result.is_ok());
    
    let error = publish_folder("/invalid/path");
    assert!(error.is_err());
}
```

#### **4. Mock Runtime Tests** (App Logic Verification)
Tests that our app correctly attempts to create tray icons and manage windows:

```rust
#[tokio::test]
async fn test_tray_icon_creation_attempt() {
    // Tests: App logic successfully attempts to create tray icon
    let app = tauri::test::mock_builder()
        .build(mock_context(noop_assets()))
        .unwrap();
        
    let tray = app.tray_by_id("main");
    assert!(tray.is_some(), "App should attempt to create tray icon with ID 'main'");
}

#[tokio::test] 
async fn test_tray_menu_structure() {
    // Tests: App sets up correct menu items (Settings, About, Quit)
    // Note: Mock runtime can't test if menu actually appears to user
}

#[tokio::test]
async fn test_window_hide_behavior() {
    // Tests: App logic correctly handles window close → hide (not quit)
    // Note: Mock runtime can't test if window actually stays hidden
}
```

**Important**: Mock runtime tests verify our **app logic works correctly** but cannot test **user-visible behavior**. They ensure we attempt to create tray icons and menus, but can't verify if users actually see them.

## Running Tests

### Backend Tests
```bash
cd src-tauri && cargo test      # Run all tests (behavioral + mock runtime)
cargo test test_content_        # Run content analysis tests only
cargo test test_tray_           # Run mock runtime tray tests only
cargo test test_folder_         # Run publishing workflow tests only
```

### Removed Commands
```bash
# These no longer exist (were testing pointless things):
npm test                        # Removed: tested browser APIs
npm run test:integration        # Removed: tested hardcoded values
```

## What We Don't Test (And Why)

### **Removed: Fake Simulation Tests**
```rust
// REMOVED - This was completely useless:
fn test_settings_menu_behavior() {
    let simulate_click = || "show_main_window";  // Hardcoded return
    assert_eq!(simulate_click(), "show_main_window");  // Tests nothing
}
```

**Why removed**: Tests that hardcoded strings equal themselves. No actual menu clicks, window management, or user behavior.

### **Removed: Browser API Tests**
```javascript
// REMOVED - This was testing JavaScript's built-in functions:
it('should parse URLs', () => {
    const url = new URL('moss://publish?path=/test');
    expect(url.searchParams.get('path')).toBe('/test');  // Of course it does!
});
```

**Why removed**: Tests that `URL()` and `decodeURIComponent()` work correctly. We don't need to test the JavaScript standard library.

## Current Testing Gap: Tray Icon and Menu Behavior

### **What We Cannot Test (Current Architecture Limitation)**
- ❌ **Tray icon creation**: Our setup() function is not accessible to tests
- ❌ **Menu structure**: Mock runtime cannot test our actual menu creation logic  
- ❌ **Icon visibility**: Whether icon appears in system tray
- ❌ **Menu interaction**: Whether right-click menus actually work
- ❌ **Finder integration**: Whether "Publish to Web" appears in context menus

### **Why Mock Runtime Tests Were Removed**
We attempted to create mock runtime tests but encountered fundamental blockers:

1. **Setup function scope**: Cannot import/call setup() from test module
2. **Tray creation architecture**: Logic is inline, not in testable functions
3. **Menu API limitations**: Mock runtime doesn't expose menu item details
4. **Integration vs unit**: Real tray testing requires actual app setup, not mocks

**Result**: Mock runtime tests would have been useless - testing fake implementations instead of our real code.

### **Current Testing Strategy**
- **Unit tests**: 4 essential tests for business logic users depend on
- **Manual testing**: Verify tray icon appears and menus work during development
- **Future platform tests**: For automated verification in release cycles

### **Future: Refactor Setup for Better Testing**

**Extract Business Logic from Setup Closure**
```rust
// Current: Inline setup closure (hard to test)
.setup(|app| {
    // 120+ lines of initialization logic mixed together
    create_tray_icon_inline(app)?;
    setup_deep_links_inline(app)?;
    configure_windows_inline(app)?;
    Ok(())
})

// Future: Extracted testable functions
.setup(|app| {
    setup_application(app)
})

fn setup_application(app: &AppHandle) -> Result<(), Box<dyn Error>> {
    create_tray_icon(app)?;
    setup_deep_links(app)?;
    configure_windows(app)?;
    Ok(())
}

fn create_tray_icon(app: &AppHandle) -> Result<TrayIcon, Box<dyn Error>> {
    // Extracted tray creation logic - now testable
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tray_icon_creation_logic() {
        // Test extracted function without full app setup
        let mock_app = create_mock_app();
        let result = create_tray_icon(&mock_app);
        assert!(result.is_ok());
    }
}
```

**Benefits of Extraction:**
- Business logic becomes unit testable
- Setup function stays clean and focused
- Easier to mock dependencies
- Logic can be reused in different contexts

### **Future: Integration/E2E Testing** (For Release Cycles)

**Platform-Specific System Testing**
```rust
// Future: Real system integration testing
#[test]
fn test_real_tray_icon_visibility() {
    #[cfg(target_os = "macos")]
    {
        // Use macOS accessibility APIs to verify icon exists
        let is_visible = check_macos_tray_icon_with_accessibility_api();
        assert!(is_visible, "Icon should be visible in macOS menu bar");
    }
    
    #[cfg(target_os = "windows")]
    {
        // Use Windows system tray APIs to verify icon
        let is_visible = check_windows_system_tray_icon();
        assert!(is_visible, "Icon should be visible in Windows system tray");
    }
}

#[test]
fn test_finder_integration_e2e() {
    // Launch app, install Finder integration, verify context menu appears
    let app_process = launch_app_in_test_mode();
    let integration_result = install_finder_integration();
    
    // Simulate right-click on folder in Finder
    let context_menu = simulate_finder_right_click("/test/folder");
    assert!(context_menu.contains("Publish to Web"));
    
    cleanup_test_app(app_process);
}
```

**WebDriver Testing for UI Workflows**
```javascript
// Future: E2E testing with WebDriver
describe('Moss UI Integration', () => {
  test('Settings window opens from tray menu', async () => {
    // Start app, click tray icon, verify settings window appears
    await driver.click('[data-testid="tray-settings"]');
    const settingsWindow = await driver.findElement('#settings-window');
    expect(settingsWindow).toBeVisible();
  });
});
```

## Testing Strategy

### **Current Approach: Fast Unit Tests + Future Refactoring**
- **Business Logic**: Test core functionality (content analysis, publishing workflow) ✅
- **Setup Function**: Inline closure - manual testing only ⚠️
- **Future Refactoring**: Extract setup logic for better unit testing 📋
- **Future Integration**: Platform-specific tests for user-visible behavior 📋

### **Recommended Testing Evolution**

**Phase 1: Current (Completed)**
- ✅ 4 essential behavioral tests for core business logic
- ✅ Removed 17 useless implementation tests
- ✅ Frontend-backend command alignment fixed

**Phase 2: Code Refactoring (Future)**
- 📋 Extract `create_tray_icon()` function from setup closure
- 📋 Extract `setup_deep_links()` function from setup closure  
- 📋 Extract `configure_windows()` function from setup closure
- 📋 Add unit tests for extracted functions
- 📋 Keep setup closure clean and simple

**Phase 3: Integration Testing (Release Cycles)**
- 📋 Platform-specific tray icon visibility tests (macOS/Windows)
- 📋 E2E Finder integration tests (right-click context menu)
- 📋 WebDriver tests for settings UI workflows
- 📋 Deep link end-to-end testing (moss:// URLs)

### **Quality Over Quantity**
- **4 essential tests** instead of 21 pointless ones
- **100% behavioral focus**: Every test validates real business logic users depend on
- **0 useless tests**: No testing of standard libraries, hardcoded values, or fake implementations
- **Clear testing gaps**: Document what we cannot test rather than create fake tests

### **Test Naming Convention**
```rust
#[test]
fn test_content_analysis_homepage_detection() {  // What feature is tested
    // Behavior: App should correctly identify homepage files by priority
    // Tests real file priority logic users depend on
}
```

## Debugging Tests

```bash
cd src-tauri
cargo test -- --nocapture              # Show println! output
cargo test test_specific_name           # Run specific test
cargo test --verbose                    # Detailed output
```

## Adding New Tests

When adding features, ask:

1. **Does this test user-observable behavior?** (If no, don't write it)
2. **Would this test survive refactoring?** (Implementation-agnostic?)
3. **Does this test something our code does?** (Not browser/OS APIs?)

### **Template for Good Tests**
```rust
#[test]
fn test_new_user_facing_feature() {
    // Behavior: When user does X, they should see Y
    
    // Arrange: Set up realistic user scenario
    let user_input = create_realistic_test_data();
    
    // Act: Trigger the actual feature
    let result = our_feature_function(user_input);
    
    // Assert: Verify user-observable outcome
    assert!(result.matches_user_expectation());
}
```

## Key Insights

### **What Makes Tests Valuable**
- Tests survive code refactoring (behavior-focused)
- Tests catch real user-facing bugs
- Tests document expected behavior for new developers
- Tests enable confident code changes

### **Red Flags in Testing**
- Testing that library functions work correctly
- Testing that hardcoded values equal themselves  
- Complex test setup that recreates application logic
- Tests that break when you refactor internal code structure

### **Hard-Learned Lessons from Testing Cleanup**

**1. Architectural Blockers Beat Clever Testing**
- Cannot test `setup()` function from test modules due to Rust visibility rules
- Mock runtime limitations prevent testing actual UI component creation
- Real system integration testing requires platform-specific approaches
- **Insight**: Accept architectural limitations rather than creating fake tests
- **Solution**: Extract business logic from setup closure into testable functions

**2. Frontend-Backend Command Alignment Critical**
- Frontend calling non-existent backend commands fails silently during development
- Build process doesn't catch invalid `invoke()` calls until runtime
- **Best Practice**: Always verify backend commands exist before frontend integration

**3. Tauri Version Compatibility for Testing**
- Tauri 2.7.0 MockRuntime had missing trait implementations
- Tauri 2.8 resolved MockRuntime compilation issues
- **Best Practice**: Use latest stable Tauri for testing infrastructure

**4. Test Removal is a Feature**
- Eliminated 17 useless tests that provided false confidence
- Retained 4 essential tests that validate real business logic
- **Insight**: Fewer, high-quality tests beat many useless ones

**5. "Test Behavior, Not Implementation" in Practice**
```rust
// ❌ Useless: Tests hardcoded return values
fn test_settings_menu_behavior() {
    let simulate_click = || "show_main_window";
    assert_eq!(simulate_click(), "show_main_window");
}

// ✅ Valuable: Tests real content analysis logic
fn test_content_analysis_homepage_detection() {
    let files = vec![/* realistic file data */];
    let result = detect_homepage_file(&files);
    assert_eq!(result, Some("index.md".to_string()));
}
```

**The goal**: Confidence that users get the behavior they expect, regardless of how we implement it internally.