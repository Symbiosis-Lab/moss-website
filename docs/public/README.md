# moss

> Write anywhere. Publish everywhere. Own everything.

## What It Is

A tiny desktop app that turns any folder into a website. Right-click, publish, done. Then syndicate to social networks and channels of your choice.

Your files stay on your computer. Your site lives on your domain. Your audience remains yours.

## Philosophy

Like its namesake, moss thrives in the spaces others overlook. It doesn't compete with platforms for sunlight. It creates the foundation layer that enables an entire ecosystem.

- **Invisible but essential** - You don't notice moss until it's gone
- **Resilient** - Survives where others can't
- **Reproductive** - Spreads through reproduction, not control
- **Transformative** - Breaks down inorganic matter into organic matter
- **Symbiotic** - Creates conditions for other life to flourish
- **Patient** - Grows slowly but persistently

## Principles

- No database
- No server required
- No platform lock-in
- No new protocols
- Everything is a plugin

## Architecture

```
Local files → moss → Static site → Your infrastructure
                ↓
          Syndication → Existing platforms
                ↓
        Spore/Lichen → Social layer
```

**moss** - Tauri app, lives in menu bar, compiles and deploys  
**Spore** - Optional ActivityPub/WebMention server  
**Lichen** - JavaScript widget for comments on any static site

## Theme Ecosystem

moss works with themes from any static site generator:

**Theme-First Journey**
1. Browse themes from Jekyll, Hugo, Zola, Eleventy
2. Choose the design you love
3. moss downloads the appropriate SSG automatically
4. Your content + chosen theme = beautiful website

**No Learning Curve**
- Keep your folder structure exactly as you want it
- moss adapts your files to work with any SSG
- No configuration files to learn
- No build tools to install

**Progressive Enhancement**
- Start simple: moss built-in SSG (instant, zero config)
- Need themes? Automatic SSG integration
- Want advanced features? Full SSG ecosystem access

For technical details, see [Architecture](architecture.md).

## Current Status

**Phase 0 Complete**: Core publishing pipeline working! Folder → Beautiful HTML + localhost preview. Right-click folder → "Publish to Web" → See instant preview.

Follow our progress in the [Development Journal](journal/).

## Usage

```bash
# Development
npm run tauri dev          # Launch development app
# Click "🧪 Test Publish" to test with test-content/simple-blog/

# Production (coming soon)
# Right-click any folder → "Publish to Web" → See instant preview
```

## What It's Not

- Not a CMS
- Not a platform
- Not an editor
- Not blockchain-based
- Not VC-fundable

## Why

Platforms are failing creators. Algorithms are eating culture. The web is recentralizing.

But the technology is ready. Rust is fast. WASM is portable. IndieWeb protocols work.

Time for infrastructure that does almost nothing, beautifully.

## License

MIT. Take it. Fork it. Make it yours.

---

_Like moss in nature, we grow slowly but inevitably. Not by replacing what exists, but by creating conditions for new growth._