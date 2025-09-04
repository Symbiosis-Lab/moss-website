# moss Plugin Architecture

> Everything beyond folder→website transformation is a plugin

## Philosophy

moss achieves unlimited extensibility through a minimal core + plugin system. The core handles only:

- Content analysis and folder structure detection
- Plugin discovery, loading, and coordination
- File system abstraction and virtual structure mapping
- Local preview server and deployment orchestration

Everything else—SSGs, themes, publishers, analytics—lives in plugins. This ensures moss stays lightweight (~5MB) while accessing the entire static site ecosystem.

## Cognitive Load Considerations

**Deep Plugin Principle**: Each plugin provides rich functionality through a simple interface. Users interact with minimal configuration while plugins handle complexity internally.

**Single Responsibility**: Each plugin does exactly one thing well:

- SSG plugins only handle static site generation
- Theme plugins only manage visual presentation
- Publisher plugins only handle deployment
- No plugin overlap or shared concerns

**Self-Contained Architecture**: Plugins are independent units without inheritance chains or complex dependencies. A new developer should understand any plugin in ~10 minutes.

**Clear Boundaries**: Plugin interfaces are explicit contracts. No hidden coupling or shared state between plugins. Framework-agnostic core logic enables easy testing and replacement.

## Plugin Interface

### Core Plugin Trait

```rust
pub trait Plugin: Send + Sync {
    // Plugin metadata
    fn manifest(&self) -> &PluginManifest;
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    // Capability detection
    fn can_handle(&self, request: &PluginRequest) -> bool;

    // Execution
    fn execute(&self, request: PluginRequest) -> Result<PluginResponse>;

    // Lifecycle
    fn initialize(&mut self) -> Result<()>;
    fn cleanup(&mut self) -> Result<()>;
}
```

### Plugin Types

#### SSG Plugins

Transform content using static site generators:

```rust
pub trait SsgPlugin: Plugin {
    fn required_structure(&self) -> FolderMapping;
    fn supported_themes(&self) -> Vec<ThemeInfo>;
    fn install_dependencies(&self) -> Result<()>;
    fn build_site(&self, config: &BuildConfig) -> Result<SiteResult>;
}
```

#### Theme Plugins

Provide visual designs and layouts:

```rust
pub trait ThemePlugin: Plugin {
    fn compatible_ssgs(&self) -> Vec<String>;
    fn preview_url(&self) -> Option<String>;
    fn install_theme(&self, target_dir: &Path) -> Result<()>;
}
```

#### Publisher Plugins

Deploy sites to hosting platforms:

```rust
pub trait PublisherPlugin: Plugin {
    fn supported_platforms(&self) -> Vec<String>;
    fn deploy(&self, site_dir: &Path, config: &DeployConfig) -> Result<DeployResult>;
    fn configure(&self) -> Result<DeployConfig>;
}
```

## Plugin Communication

### Protocol: JSON-RPC over Subprocess

Plugins run as separate processes communicating via JSON-RPC. This provides:

- **Language agnostic**: Plugins can be written in any language
- **Process isolation**: Plugin crashes don't affect moss core
- **Security**: Sandboxed execution environment
- **Compatibility**: Works with existing SSG binaries

### Message Format

```json
{
  "jsonrpc": "2.0",
  "method": "build_site",
  "params": {
    "source_path": "/Users/jane/blog",
    "output_path": "/Users/jane/blog/.moss/site",
    "theme": "minimal",
    "config": {
      "title": "Jane's Blog",
      "author": "Jane Smith"
    }
  },
  "id": 1
}
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "result": {
    "page_count": 12,
    "build_time_ms": 850,
    "site_url": "http://localhost:8080",
    "assets": ["style.css", "main.js"]
  },
  "id": 1
}
```

## Plugin Manifest

Each plugin includes a `plugin.toml` manifest:

```toml
[plugin]
name = "Jekyll"
version = "1.0.0"
type = "ssg"
description = "Jekyll static site generator integration"
author = "moss team"
license = "MIT"

[capabilities]
formats = ["markdown", "liquid"]
themes = "https://jekyllthemes.io/api"
requires = ["ruby >= 2.7"]

[runtime]
executable = "moss-jekyll"
protocol = "json-rpc"
timeout = 60000

[dependencies]
ruby = ">=2.7"
bundler = ">=2.0"

[installation]
check_command = "ruby --version"
install_command = "gem install jekyll bundler"
```

## Plugin Discovery and Loading

### Discovery Locations

1. **Bundled plugins**: `Contents/Resources/plugins/` (shipped with moss)
2. **User plugins**: `~/.moss/plugins/` (user-installed)
3. **System plugins**: `/usr/local/lib/moss/plugins/` (system-wide)

### Loading Sequence

```rust
pub struct PluginManager {
    loaded_plugins: HashMap<String, Box<dyn Plugin>>,
    plugin_registry: PluginRegistry,
}

impl PluginManager {
    pub fn load_all_plugins(&mut self) -> Result<()> {
        // 1. Discover plugin manifests
        let manifests = self.discover_plugins()?;

        // 2. Validate compatibility
        let compatible = self.filter_compatible(manifests)?;

        // 3. Load in dependency order
        let ordered = self.resolve_dependencies(compatible)?;

        // 4. Initialize plugins
        for manifest in ordered {
            let plugin = self.load_plugin(manifest)?;
            plugin.initialize()?;
            self.loaded_plugins.insert(plugin.name().to_string(), plugin);
        }

        Ok(())
    }
}
```

## Default Plugins

### 1. Minimal SSG Plugin (`moss-ssg-minimal`)

The built-in SSG that provides instant functionality:

```rust
pub struct MinimalSsgPlugin {
    name: String,
    version: String,
}

impl SsgPlugin for MinimalSsgPlugin {
    fn build_site(&self, config: &BuildConfig) -> Result<SiteResult> {
        // Ultra-lightweight Rust implementation
        // - Parse markdown with pulldown-cmark
        // - Apply single beautiful template
        // - Generate site in <1 second
    }
}
```

**Characteristics**:

- ~200 lines of Rust code
- Single template with moss green branding
- No external dependencies
- <1MB binary size
- <1 second build time for typical content

### 2. Jekyll Plugin (`moss-ssg-jekyll`)

Wrapper for Jekyll with automatic dependency management:

```rust
pub struct JekyllPlugin {
    binary_path: Option<PathBuf>,
    gem_home: PathBuf,
}

impl SsgPlugin for JekyllPlugin {
    fn install_dependencies(&self) -> Result<()> {
        // Check for Ruby/Bundler
        // Install Jekyll gems in isolated environment
        Command::new("gem")
            .args(&["install", "jekyll", "bundler", "--user-install"])
            .env("GEM_HOME", &self.gem_home)
            .status()?;
    }

    fn build_site(&self, config: &BuildConfig) -> Result<SiteResult> {
        // 1. Create virtual Jekyll structure in .moss/build/
        // 2. Map user folders to Jekyll conventions
        // 3. Generate _config.yml
        // 4. Run `jekyll build`
        // 5. Return site metadata
    }
}
```

### 3. Hugo Plugin (`moss-ssg-hugo`)

Fast builds with Hugo's performance:

```rust
pub struct HugoPlugin {
    binary_path: Option<PathBuf>,
}

impl SsgPlugin for HugoPlugin {
    fn install_dependencies(&self) -> Result<()> {
        // Download Hugo binary for current platform
        // Cache in ~/.moss/cache/hugo/
        self.download_hugo_binary()?;
    }

    fn build_site(&self, config: &BuildConfig) -> Result<SiteResult> {
        // Hugo-specific folder adaptation and build
    }
}
```

## Plugin Security Model

### Subprocess Sandboxing

- Each plugin runs in isolated subprocess
- Limited file system access via explicit paths
- No network access unless explicitly granted
- Resource limits (CPU, memory, timeout)

### Permission Model

```toml
# plugin.toml
[permissions]
filesystem = ["read:source", "write:output", "read:themes"]
network = ["download:themes"]
subprocess = ["ruby", "node", "python"]
```

### Installation Security

- Plugins verified with cryptographic signatures
- Source code review for bundled plugins
- User confirmation for network access requests
- Automatic updates only for verified plugins

## Plugin Development

### Creating a Plugin

1. **Initialize plugin structure**:

```bash
moss plugin init my-plugin --type=ssg
```

2. **Implement plugin trait**:

```rust
// src/lib.rs
use moss_plugin_api::{Plugin, SsgPlugin, PluginManifest};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn manifest(&self) -> &PluginManifest { /* ... */ }
    fn execute(&self, request: PluginRequest) -> Result<PluginResponse> { /* ... */ }
}
```

3. **Create manifest**:

```toml
# plugin.toml
[plugin]
name = "my-plugin"
type = "ssg"
# ...
```

4. **Build and test**:

```bash
moss plugin build
moss plugin test
```

### Plugin API Crate

The `moss-plugin-api` crate provides:

- Core traits and types
- JSON-RPC helpers
- Common utilities
- Testing framework

```rust
// Cargo.toml
[dependencies]
moss-plugin-api = "1.0"
serde = { version = "1.0", features = ["derive"] }
```

## Plugin Registry

### Local Registry

Installed plugins stored in:

```
~/.moss/
├── plugins/
│   ├── jekyll/
│   │   ├── plugin.toml
│   │   └── moss-jekyll
│   └── hugo/
│       ├── plugin.toml
│       └── moss-hugo
├── cache/
│   ├── themes/
│   └── binaries/
└── config.toml
```

### Future: Remote Registry

Phase 2 will add a remote plugin marketplace:

- Plugin discovery and installation
- Automatic updates
- Community ratings and reviews
- Plugin monetization support

## Integration with Current Architecture

### Compilation Flow with Plugins

```rust
pub fn compile_folder_with_options(folder_path: String, auto_serve: bool) -> Result<String, String> {
    // 1. Analyze folder structure (core)
    let project = analyze_folder_structure(&folder_path)?;

    // 2. Select appropriate SSG plugin
    let plugin_manager = PluginManager::instance();
    let ssg_plugin = plugin_manager.select_ssg_plugin(&project)?;

    // 3. Execute plugin
    let request = PluginRequest::BuildSite {
        source_path: folder_path.clone(),
        output_path: format!("{}/.moss/site", folder_path),
        project_type: project.project_type,
        theme: None, // User selection in future
    };

    let result = ssg_plugin.execute(request)?;

    // 4. Handle result (core)
    if auto_serve {
        start_site_server(&result.output_path)?;
    }

    Ok(result.message)
}
```

### Backward Compatibility

During transition, current embedded SSG logic becomes the MinimalSsgPlugin:

- Existing functionality preserved
- No breaking changes for users
- Plugin system adds capabilities without changing core UX

## Performance Considerations

### Plugin Loading

- Lazy loading: Only load plugins when needed
- Plugin caching: Keep frequently-used plugins in memory
- Startup optimization: Load core plugins first, others on-demand

### Communication Overhead

- JSON-RPC batching for multiple operations
- Persistent plugin processes for repeated operations
- Binary protocol for large data transfers (future)

### Build Performance

- Parallel plugin execution where possible
- Incremental builds through plugin API
- Caching of plugin results

## Migration Path

### Phase 1: Infrastructure (Current)

- Implement plugin loading system
- Extract minimal SSG as default plugin
- Create plugin API crate

### Phase 2: Core Plugins

- Port Jekyll/Hugo logic to plugins
- Add theme plugin system
- Implement publisher plugins

### Phase 3: Ecosystem

- Remote plugin registry
- Third-party plugin support
- Plugin development tools

### Phase 4: Advanced Features

- WASM plugins for performance
- Plugin composition and workflows
- Advanced security model

## Success Metrics

### Developer Experience

- Time to create new plugin: <30 minutes
- Plugin API documentation completeness
- Community plugin contributions

### Performance

- Plugin loading time: <100ms
- Communication overhead: <5% of total build time
- Memory usage per plugin: <50MB baseline

### Ecosystem Health

- Number of available plugins
- Plugin update frequency
- User adoption of third-party plugins

---

_The plugin architecture transforms moss from a static site generator into an orchestration platform for the entire static site ecosystem._
