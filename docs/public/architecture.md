# How moss Works

## System Overview

```
Local files → moss → Static site → Your infrastructure
                ↓
          Syndication → Existing platforms
                ↓
        Spore/Lichen → Social layer
```

**moss** - Desktop app that transforms folders into websites  
**Spore** - Optional server for social features (ActivityPub/WebMentions)  
**Lichen** - Embeddable comment widget for any static site

## The Publishing Pipeline

### 1. Content Detection

When you right-click a folder, moss analyzes your content:

- **Homepage discovery**: Looks for `index.md`, `README.md`, or first document file
- **Site structure**: Detects flat directory vs homepage + collections
- **File support**: Markdown, Pages, Word documents, plus images and assets
- **Smart defaults**: No configuration required - moss understands common patterns

### 2. Three-Window Preview

**Main Preview**
- Your content rendered exactly as it will appear online
- No moss branding or interface elements
- Full-size window showing your actual website

**Floating Control Zones**
- **Top zone**: Core actions (Publish, Edit, Settings) as transparent circular buttons
- **Right zone**: Platform syndication controls using familiar brand colors
- **Magnetic positioning**: Control windows follow the preview automatically

### 3. Deployment & Syndication

**Publishing**
- moss.pub: Zero-configuration subdomain (username.moss.pub)
- GitHub Pages: Direct integration for developers
- Custom hosting: Netlify, Vercel, S3-compatible services

**Syndication**
- Automatic cross-posting to social platforms
- RSS feed generation for discovery
- Email newsletter integration
- All through plugins - enable what you need

## Plugin System

### Core + Extensions

moss core handles folder → website transformation. Everything else is a plugin:

**Content Enhancement**
- Themes and visual customization
- Syntax highlighting for code
- Image optimization and galleries

**Distribution & Analytics**
- Social platform integration
- Privacy-focused visitor statistics
- SEO optimization and sitemaps

**Social Features**
- Comments and discussions
- WebMention support
- Community interaction

### Visual Integration

All plugins follow consistent patterns:
- Circular controls in floating zones
- Platform-specific colors for recognition
- Zero-configuration defaults
- Advanced options hidden until needed

## Platform Integration

### macOS
- Native Services integration (right-click "Publish")
- Menu bar presence with system-appropriate icon
- Full vibrancy effects on control windows

### Windows
- Context menu integration
- Windows 11 design language compliance
- System accent color adaptation

### Linux
- Cross-desktop environment compatibility
- High-contrast fallbacks for accessibility
- Standard window decorations when needed

### All Platforms
- Native file dialogs and system integration
- OS-appropriate fonts and spacing
- Automatic light/dark mode adaptation

---

*moss handles the technical complexity so you can focus on creating. Right-click, publish, reach your audience.*