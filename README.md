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

## Protocols

- **Micropub** for submissions
- **ActivityPub** for federated social
- **WebMention** for pingbacks

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

MIT
