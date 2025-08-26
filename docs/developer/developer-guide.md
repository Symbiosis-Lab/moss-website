# moss Developer Guide

> Implementation details, file structure, and API reference for contributors

## Quick Start

```bash
# Frontend development
npm run dev

# Backend development  
cd src-tauri && cargo run

# Run all tests
npm run test:all

# Generate API documentation
cd src-tauri && cargo doc --open
```

## Project Structure

```
moss/
├── src-tauri/                  # Rust Backend
│   ├── Cargo.toml             # Dependencies & metadata
│   ├── src/
│   │   ├── main.rs            # App entry point + Tauri commands
│   │   └── tray_tests.rs      # System tray unit tests
│   ├── tauri.conf.json        # Tauri app configuration
│   ├── build.rs               # Build script
│   ├── icons/                 # App icons (all platforms)
│   └── target/                # Build output
│
├── Frontend (Web Technologies)
│   ├── index.html             # App entry point
│   ├── main.js                # Frontend logic
│   ├── package.json           # Node dependencies
│   ├── vite.config.js         # Build configuration
│   └── vitest.config.js       # Test configuration
│
├── Documentation
│   ├── docs/                  # Strategic & technical docs
│   │   ├── strategic/         # Business planning & growth strategy
│   │   └── developer/         # Technical docs & implementation guides
│   │       ├── testing-guide.md     # Testing guidelines
│   │       └── implementation-plan.md # Development roadmap
│   └── README.md              # Project overview
│
└── Configuration
    ├── .claude/               # AI assistant context
    └── dist/                  # Build output
```

## File Structure Explained

### Backend (`src-tauri/`)

| File | Purpose | Key Contents |
|------|---------|--------------|
| `Cargo.toml` | Rust package definition | Dependencies, metadata, build features |
| `src/main.rs` | Application entry point | Tauri commands, system tray, app setup |
| `src/tray_tests.rs` | System tray unit tests | Test utilities for tray functionality |
| `tauri.conf.json` | Tauri configuration | App metadata, permissions, build settings |
| `build.rs` | Custom build script | Pre-compile tasks, resource bundling |
| `icons/` | Platform-specific icons | Generated icons for macOS, Windows, Linux |

### Frontend 

| File | Purpose | Key Contents |
|------|---------|--------------|
| `index.html` | Web app entry point | Basic HTML structure, script imports |
| `main.js` | Frontend application logic | UI interactions, Tauri command calls |
| `package.json` | Node.js package definition | Frontend dependencies, scripts |
| `vite.config.js` | Build tool configuration | Development server, build optimization |

## Backend API Reference

The Rust backend exposes commands to the frontend via Tauri's IPC system. All commands are defined in `src-tauri/src/main.rs`.

### Available Commands

#### `greet(name: String) -> String`

Simple greeting function for testing frontend-backend communication.

**Usage:**
```javascript
import { invoke } from '@tauri-apps/api/core';
const result = await invoke('greet', { name: 'World' });
// Returns: "Hello, World! You've been greeted from Rust!"
```

#### `test_tray_icon(app: AppHandle) -> Result<String, String>`

Tests system tray functionality by attempting to update the tooltip.

**Usage:**
```javascript
const result = await invoke('test_tray_icon');
// Success: "Tray icon found and is responsive"
// Error: "Tray icon not found by ID"
```

#### `publish_folder_from_deep_link(folder_path: String) -> Result<String, String>`

Handles deep link requests to publish folders via `moss://publish?path=...` URLs.

**Parameters:**
- `folder_path`: Absolute path to folder to publish

**Usage:**
```javascript
const result = await invoke('publish_folder_from_deep_link', { 
    folderPath: '/Users/username/my-site' 
});
// Success: "Publishing initiated for: /Users/username/my-site"
// Error: "Empty folder path provided"
```

**Implementation Status:** Currently logs request and returns confirmation. Static site generation TODO.

#### `install_finder_integration() -> Result<String, String>`

Installs macOS Finder context menu integration by creating an Automator workflow.

**Usage:**
```javascript
const result = await invoke('install_finder_integration');
// Success: "Finder integration installed successfully! Right-click any folder → Quick Actions → 'Publish to Web'"
// Error: "Could not determine home directory" | "Failed to create Services directory" | etc.
```

**Implementation Details:**
- Creates `~/Library/Services/Publish to Web.workflow`
- Registers with macOS Services for folder context menus
- Uses Automator + shell script to trigger `moss://` deep links
- Automatically removes existing workflow on reinstall

**Platform Support:** macOS only

### Error Handling

All commands return `Result<String, String>`:
- **Success**: `Ok(message)` with confirmation details
- **Error**: `Err(error_message)` with specific failure reason

Handle errors in frontend:
```javascript
try {
    const result = await invoke('command_name', { param: value });
    console.log('Success:', result);
} catch (error) {
    console.error('Error:', error);
}
```

## System Integration

### macOS Finder Integration

The app integrates with macOS Finder through:

1. **Deep Link Protocol**: `moss://publish?path=encoded_path`
2. **Automator Workflow**: Installed in `~/Library/Services/`
3. **Context Menu**: "Publish to Web" appears when right-clicking folders

**Flow:**
```
Right-click folder → "Publish to Web" → Shell script → Deep link → Tauri command
```

### System Tray

Menu bar app with system tray containing:
- **Settings**: Opens preferences window
- **About**: App information (TODO)  
- **Quit**: Exits application

**Template Icon**: 16x16 black circle, rendered as macOS template icon

## Development Workflow

### Frontend Development
```bash
npm run dev          # Start Vite dev server
npm run build        # Build for production
npm run preview      # Preview production build
```

### Backend Development
```bash
cd src-tauri
cargo run            # Run in development mode  
cargo build          # Build debug binary
cargo build --release # Build optimized binary
cargo test           # Run unit tests
cargo doc --open     # Generate & view API docs
```

### Testing
```bash
npm run test:frontend     # Frontend unit tests
npm run test:integration  # Integration tests
npm run test:backend      # Rust unit tests
npm run test:all         # All tests
npm run test:coverage    # Test coverage report
```

### Full Development Build
```bash
npm run tauri dev    # Run full Tauri app in dev mode
npm run tauri build  # Create production app bundle
```

## Key Dependencies

### Rust Backend
- `tauri = "2.7.0"` - Desktop app framework
- `tauri-plugin-dialog = "2.3"` - File dialogs
- `tauri-plugin-fs = "2.3"` - File system access
- `tauri-plugin-shell = "2.3"` - Shell command execution  
- `tauri-plugin-deep-link = "2.4.1"` - URL scheme handling
- `serde = "1.0"` - JSON serialization

### Frontend
- `@tauri-apps/api = "2.7.0"` - Tauri JavaScript API
- `vite = "^5.4.2"` - Build tool
- `vitest = "^3.2.4"` - Testing framework

## Architecture Patterns

### Command Pattern
Each backend function is a Tauri command:
```rust
#[tauri::command]
fn command_name(param: Type) -> Result<ReturnType, String> {
    // Implementation
}
```

### Error Handling
Consistent error handling using `Result<T, String>`:
```rust
// Return success
Ok("Success message".to_string())

// Return error  
Err("Error description".to_string())
```

### System Integration
Platform-specific features wrapped in cross-platform commands:
```rust
#[cfg(target_os = "macos")]
// macOS-specific implementation

#[cfg(target_os = "windows")]  
// Windows-specific implementation
```

## Debugging

### Backend Logs
```bash
# Development mode shows println! output
cargo run

# Or from npm script
npm run tauri dev
```

### Frontend Debugging
- **Dev Tools**: Available in development Tauri window
- **Console**: `console.log()` works normally
- **Network**: Tauri command calls visible in dev tools

### Test Debugging
```bash
# Run single test
cargo test test_name

# Run with output
cargo test test_name -- --nocapture

# Frontend test debugging
npm run test:ui    # Visual test runner
```

## Contributing

1. **Read the code**: Start with `src-tauri/src/main.rs` for backend, `main.js` for frontend
2. **Run tests**: Ensure `npm run test:all` passes  
3. **Generate docs**: Use `cargo doc --open` to explore API docs
4. **Follow conventions**: Match existing code style and patterns
5. **Document changes**: Update this guide for new APIs or structure changes

---

_For strategic context, see [Technical Architecture](./technical-architecture.md). For project vision, see [Strategic Documentation](../strategic/)._