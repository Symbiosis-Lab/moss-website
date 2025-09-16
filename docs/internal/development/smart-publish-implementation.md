# Smart Publish Button Implementation Plan

## Overview

Implementation of a simplified, intelligent publish button that detects git configuration automatically and provides appropriate actions with minimal user intervention.

## Current State Analysis

### Existing Components
- **PreviewState** (`preview/state.rs`) - Manages preview window state including publication status
- **publish_from_preview** (`preview/commands.rs:87`) - Current publish command with mock implementation
- **validate_publish_request** (`preview/commands.rs:14`) - Validates site build state before publishing

### Integration Points
- Preview window opens with compiled site
- Publish button triggers `publish_from_preview` command
- State management through `PreviewState` struct

## Simplified Design Requirements

### Button State Logic
```
┌─ No Git ─┐    ┌─ No Remote ─┐    ┌─ Ready ─┐
│  Setup   │ →  │   Connect   │ →  │ Publish │
│   Git    │    │  to GitHub  │    │   Live  │
└──────────┘    └─────────────┘    └─────────┘
```

### Smart Detection
- **Pre-flight checks** run when preview window opens
- Check for `.git/config` → `remote.origin.url`
- Determine appropriate button state immediately
- No platform selection - single GitHub Pages target

### Single Modal Flow
When "Connect to GitHub" clicked:
- Repository name pre-filled with folder name
- Private by default (safe choice)
- Single action: "Create & Publish"
- Creates repo + sets remote + publishes

## Implementation Architecture

### New Components

#### 1. Git Detection Utilities (`preview/git.rs`)
```rust
pub struct GitRemoteInfo {
    pub url: String,
    pub is_github: bool,
    pub repo_name: String,
    pub owner: String,
}

pub fn detect_git_remote(folder_path: &Path) -> Result<Option<GitRemoteInfo>, String>
pub fn has_git_repository(folder_path: &Path) -> bool
pub fn create_github_repo_and_remote(folder_path: &Path, repo_name: &str, is_public: bool) -> Result<GitRemoteInfo, String>
```

#### 2. Publish State Enhancement (`preview/state.rs`)
```rust
pub struct PreviewState {
    // ... existing fields ...
    pub git_remote: Option<GitRemoteInfo>,
    pub publish_button_state: PublishButtonState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublishButtonState {
    SetupGit,           // No .git directory
    ConnectToGitHub,    // Has git, no GitHub remote
    Publish,            // Ready to publish
    Published(String),  // Already published to URL
}
```

#### 3. GitHub Integration (`preview/github.rs`)
```rust
pub struct GitHubPages {
    remote_info: GitRemoteInfo,
}

impl GitHubPages {
    pub async fn deploy_to_pages(folder_path: &Path, remote_info: &GitRemoteInfo) -> Result<String, String>
    pub async fn create_repository(repo_name: &str, is_public: bool) -> Result<String, String>
}
```

### Updated Commands

#### Enhanced `publish_from_preview`
```rust
pub async fn publish_from_preview(
    manager: PreviewManagerState<'_>,
    preview_id: String,
    action: PublishAction, // New enum: Publish | SetupGit | ConnectToGitHub
) -> Result<String, String>
```

#### New Commands
```rust
pub async fn setup_github_repository(
    preview_id: String,
    repo_name: String,
    is_public: bool,
) -> Result<String, String>

pub async fn refresh_publish_state(
    preview_id: String,
) -> Result<PublishButtonState, String>
```

## Implementation Phases

### Phase 1: Git Detection
1. Create `preview/git.rs` with detection utilities
2. Add git remote detection logic
3. Test with various git configurations

### Phase 2: State Management
1. Extend `PreviewState` with git information
2. Add publish button state enum
3. Update state initialization to detect git

### Phase 3: GitHub Integration
1. Create `preview/github.rs` for GitHub API integration
2. Implement repository creation
3. Implement GitHub Pages deployment

### Phase 4: UI Integration
1. Update preview window to show correct button state
2. Add repository setup dialog
3. Connect frontend to new commands

### Phase 5: Testing & Polish
1. Test complete workflow end-to-end
2. Handle error cases gracefully
3. Add user feedback and progress indicators

## API Design

### Frontend Interface
```javascript
// Check current publish state
const state = await invoke('get_preview_state', { preview_id });

// Publish or setup based on state
switch (state.publish_button_state.type) {
    case 'SetupGit':
        await invoke('setup_git_repository', { preview_id });
        break;
    case 'ConnectToGitHub':
        await invoke('setup_github_repository', { 
            preview_id, 
            repo_name: 'my-site', 
            is_public: false 
        });
        break;
    case 'Publish':
        await invoke('publish_from_preview', { preview_id });
        break;
}
```

### Error Handling Strategy
- **Git not installed**: Clear message with installation instructions
- **GitHub authentication**: Guide to create personal access token
- **Repository creation failed**: Suggest alternative names or manual creation
- **Deployment failed**: Show GitHub Actions logs and troubleshooting tips

## Success Metrics

### Developer Experience
- **Zero configuration**: Works out of box with existing git repos
- **Single command**: `cargo build && ./moss compile folder --serve`
- **Fast iteration**: Changes to publish logic don't require full rebuild

### User Experience
- **Sub-30-second setup**: From no remote to live website
- **Clear feedback**: User always knows what will happen when they click
- **Recoverable errors**: Every error state has clear next steps
- **No surprises**: Button label matches exactly what will happen

## Future Extensibility

### Plugin Architecture Foundation
While implemented as monolithic GitHub Pages integration, the architecture supports future plugin expansion:

```rust
trait PublishPlugin {
    fn detect_configuration(&self, folder_path: &Path) -> PublishButtonState;
    fn setup_dialog(&self) -> SetupDialogConfig;
    fn publish(&self, folder_path: &Path) -> Result<String, String>;
}
```

Each plugin follows the same pattern:
- Detect current state
- Provide minimal setup dialog if needed
- Handle publishing with clear feedback

This foundation enables future Netlify, Vercel, or custom deployment plugins while maintaining the simplified user experience.