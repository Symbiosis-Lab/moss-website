# moss

> Write anywhere. Publish everywhere. Own everything.

## Philosophy

Like its namesake, moss thrives in the spaces others overlook. It doesn't compete with platforms for sunlight. It creates the foundation layer that enables an entire ecosystem.

- **Invisible but essential** - You don't notice moss until it's gone
- **Resilient** - Survives where others can't
- **Reproductive** - Spreads through reproduction, not control
- **Transformative** - Breaks down inorganic matter into organic matter
- **Symbiotic** - Creates conditions for other life to flourish
- **Patient** - Grows slowly but persistently

## What It Is

A tiny desktop app that turns any folder into a website. Right-click, publish, done. Then syndicate to social networks and channels of your choice.

Your files stay on your computer. Your site lives on your domain. Your audience remains yours.

## [Architecture](./docs/public/architecture.md)

```
Local files → moss → SSG (built-in/external) → Static site → Your infrastructure
                ↓              ↓                      ↓
          Theme selection  Folder adapter      Lichen injection
                ↓              ↓                      ↓
        Download SSG on-demand    ↓             Social features
                           .moss/ directory
                    (all temporary files)
```

**moss** - Desktop app orchestrating SSGs + deployment + syndication  
**Spore** - Optional ActivityPub/WebMention server  
**Lichen** - JavaScript widget adding social features to any static site

moss doesn't compete with static site generators—it completes them. Choose any theme from Jekyll, Hugo, Zola, or Eleventy. moss handles the setup, preserves your folder structure, and adds the missing workflow pieces.

## Principles

- No database
- No server required
- No platform lock-in
- No new protocols
- Everything is a plugin

## Protocols

- **Micropub** for submissions
- **ActivityPub** for federation
- **WebMention** for pingbacks
- **RSS** because it never died

## Usage

```bash
# Development
npm run tauri dev          # Launch development app
# Click "🧪 Test Publish" to test with test-content/simple-blog/

# Production (coming soon)
# Right-click any folder → "Publish to Web" → See instant preview
```

**Current Status**: Core publishing pipeline complete! Folder → Beautiful HTML + localhost preview working. Next: minimal built-in SSG + theme marketplace integration.

For developers who want more:

```javascript
// Everything beyond core is a plugin
interface mossPlugin {
  onBuild?: (site: Site) => Site
  onPublish?: (deploy: Deploy) => void
  syndicate?: (post: Post) => Result
}
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
