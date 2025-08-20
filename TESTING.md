# Moss Testing Guide

## Overview

This project follows the CLAUDE.md principle of "Test behavior, not implementation" with comprehensive testing across all layers of the application.

## Test Structure

```
moss/
├── main.test.js              # Frontend unit tests
├── integration.test.js       # Frontend ↔ Backend integration tests
├── vitest.config.js          # Vitest configuration
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          # Contains unit tests for Tauri commands
│   │   └── tray_tests.rs    # Tray icon functionality tests
│   └── Cargo.toml           # Rust dependencies and test config
└── package.json             # Test scripts
```

## Running Tests

### All Tests
```bash
npm run test:all
```

### Frontend Tests (Vitest + JSDOM)
```bash
npm run test:frontend      # Frontend unit tests
npm run test:integration   # Integration tests
npm test                   # Interactive mode
npm run test:ui           # Visual test UI
```

### Backend Tests (Rust + Cargo)
```bash
npm run test:backend      # All Rust tests
# Or directly:
cargo test --manifest-path=src-tauri/Cargo.toml
```

### Coverage
```bash
npm run test:coverage
```

## Test Categories

### 1. Frontend Unit Tests (`main.test.js`)
- **Mock Tauri API** calls using Vitest mocking
- **DOM manipulation** testing with JSDOM
- **Error handling** for backend communication
- **App initialization** flow

**Example:**
```javascript
it('should successfully call greet command', async () => {
  mockInvoke.mockResolvedValue('Hello, Moss!')
  const result = await invoke('greet', { name: 'Moss' })
  expect(result).toContain('Hello, Moss!')
})
```

### 2. Integration Tests (`integration.test.js`)
- **End-to-end** frontend ↔ backend workflows
- **Error scenarios** and recovery
- **State management** across operations
- **Partial failure** handling

**Example:**
```javascript
it('should simulate complete app initialization flow', async () => {
  const greetResult = await invoke('greet', { name: 'Moss' })
  const trayResult = await invoke('test_tray_icon')
  
  expect(greetResult).toContain('Hello, Moss!')
  expect(trayResult).toContain('Tray icon found')
})
```

### 3. Backend Unit Tests (`src-tauri/src/main.rs`)
- **Tauri command** functionality
- **Input validation** and edge cases
- **String handling** with special characters
- **Performance** with large inputs

**Example:**
```rust
#[test]
fn test_greet_function() {
    let result = greet("World");
    assert!(result.contains("World"));
    assert!(result.contains("Hello"));
}
```

### 4. Tray Icon Tests (`src-tauri/src/tray_tests.rs`)
- **Icon creation** and formatting
- **Template icon** design validation
- **Event handling** logic
- **Menu interaction** scenarios
- **Size and transparency** constraints

**Example:**
```rust
#[test]
fn test_tray_icon_template_format() {
    let icon = create_template_icon();
    assert_eq!(icon.width(), 16);
    assert_eq!(icon.height(), 16);
}
```

## Testing Philosophy

Following CLAUDE.md guidelines:

### ✅ **Do:**
- Test **behavior**, not implementation details
- Write tests **before** or **during** feature development
- Keep tests **simple** and **readable**
- Use **descriptive** test names
- Test **error conditions** and edge cases
- Ensure tests are **deterministic**

### ❌ **Don't:**
- Test private implementation details
- Create overly complex test setups
- Skip testing error scenarios
- Write tests that depend on external state
- Disable failing tests instead of fixing them

## Continuous Integration

All tests must pass before committing:

```bash
# Recommended pre-commit flow
npm run test:all
git add .
git commit -m "feat: add new feature with tests"
```

## Test Configuration

### Vitest Configuration (`vitest.config.js`)
- **Environment**: JSDOM for browser-like testing
- **Globals**: Enabled for `describe`, `it`, `expect`
- **Coverage**: V8 provider with multiple output formats

### Mock Strategy
- **Tauri API**: Fully mocked for frontend tests
- **DOM**: JSDOM provides browser environment
- **File System**: No real file operations in tests
- **Network**: No external API calls

## Adding New Tests

When adding new features:

1. **Add unit tests** for individual functions
2. **Add integration tests** for user workflows  
3. **Update documentation** if test patterns change
4. **Ensure all tests pass** before committing

### Example Test Template

```javascript
describe('New Feature', () => {
  beforeEach(() => {
    // Setup
  })

  it('should handle normal case', () => {
    // Arrange
    // Act  
    // Assert
  })

  it('should handle error case', () => {
    // Test error scenarios
  })
})
```

## Performance Testing

Currently focused on functional correctness. Performance tests can be added later using:
- **Vitest benchmarks** for frontend performance
- **Criterion.rs** for Rust performance testing

## Debugging Tests

### Frontend
```bash
npm run test:ui    # Visual debugging interface
npm test          # Interactive mode with watch
```

### Backend
```bash
cargo test -- --nocapture    # Show println! output
cargo test --verbose         # Detailed test output
```

## Future Improvements

- [ ] Add visual regression tests for UI changes
- [ ] Add property-based testing for complex logic
- [ ] Set up automated testing in CI/CD pipeline
- [ ] Add performance benchmarks
- [ ] Add accessibility testing