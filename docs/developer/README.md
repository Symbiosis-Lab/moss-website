# Developer Documentation

> Technical implementation, architecture, and contributor guides

## Quick Start

```bash
# Clone and setup
git clone [repository-url]
cd moss
npm install

# Development
npm run dev              # Frontend development
cd src-tauri && cargo run # Backend development
npm run test:all         # Run all tests

# Documentation
cd src-tauri && cargo doc --open # Generate API docs
```

## Documentation Overview

### 🚀 [Developer Guide](./developer-guide.md)

**Start here** for hands-on development work.

- **Project Structure** - Complete file organization explained
- **Backend API Reference** - All Tauri commands with examples
- **Development Workflow** - Build, test, and debug processes
- **Contributing Guidelines** - Code conventions and best practices

### 🏗️ [Technical Architecture](./technical-architecture.md)

High-level system design and technology decisions.

- **Core Stack** - Tauri, Rust, frontend technologies
- **File System Architecture** - Data organization patterns
- **Build Pipeline** - Static site generation process
- **Plugin System** - Extensibility framework design
- **Deployment Architecture** - Publishing and hosting options

### 📋 [Development Roadmap](./dev-roadmap.md)

Technical milestones and implementation phases.

- **Current Sprint** - Active development priorities
- **Upcoming Features** - Planned functionality
- **Architecture Evolution** - Long-term technical direction

### 🚀 [Implementation Plan](./implementation-plan.md)

Detailed development progress and learnings.

- **Completed Milestones** - What's been built and tested
- **Current Progress** - Active development work
- **Technical Insights** - Lessons learned during implementation
- **Next Steps** - Immediate development priorities

### 🧪 [Testing Guide](./testing-guide.md)

Comprehensive testing documentation and philosophy.

- **Test Structure** - How tests are organized
- **Running Tests** - Commands and workflows
- **Testing Philosophy** - Best practices and guidelines
- **Debugging Tests** - Troubleshooting and development

## Key Technologies

- **Backend**: Rust, Tauri v2, system integration
- **Frontend**: Vanilla JS, Vite, web technologies
- **Testing**: Vitest (frontend), Cargo (backend)
- **Documentation**: Rust docs, Markdown

## Development Principles

1. **Local-first** - User data stays on device
2. **Zero configuration** - Works out of the box
3. **Standards-based** - Use existing protocols
4. **Performance-focused** - <5 minute first publish
5. **Cross-platform** - macOS, Windows, Linux support

## Getting Help

- **API Reference**: Run `cargo doc --open` for detailed Rust documentation
- **Architecture Questions**: See [Technical Architecture](./technical-architecture.md)
- **Implementation Help**: Check [Developer Guide](./developer-guide.md)
- **Strategic Context**: See [../strategic/](../strategic/) documentation

---

_For strategic planning and business context, see [Strategic Documentation](../strategic/)._