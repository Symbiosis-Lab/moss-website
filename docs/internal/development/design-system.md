# moss Visual Design System

> Detailed specifications for visual consistency and implementation

## Spatial Metaphors

**Content is Sacred**
```
┌─────────────────┐
│   PURE CONTENT  │  ← Rectangle: Structure, readability, focus
│                 │
└─────────────────┘
```

**Actions are Organic**
```
    ●  ●  ●         ← Circles: Natural, tactile, immediate
```

This creates moss's signature: **rectangular sanctuaries for content, circular satellites for action**.

### Color Philosophy

**Content First**
- Content windows use pure backgrounds (white/black) for maximum readability
- No brand colors compete with user's written words
- Platform neutrality preserves content portability

**Platform Recognition**
- Action circles inherit platform brand colors (Twitter blue, LinkedIn blue, etc.)
- Each syndication target becomes instantly recognizable
- User's mental models remain intact across contexts

**System Integration**
- macOS: Template icons adapt to system dark/light modes
- Windows: Respects system accent colors where appropriate
- Linux: Falls back to universal, high-contrast alternatives

### Typography

**Content Typography**
- User's content renders exactly as it will appear on their website
- No moss-specific styling interferes with content preview
- Perfect fidelity between preview and published result

**Interface Typography**
- System fonts for all UI elements (San Francisco, Segoe UI, system-ui)
- Minimal text labels - prefer universally understood symbols
- When text appears, it's brief and actionable

## Interface Architecture

### Three-Window Implementation Specifications

**Main Preview Window**: Pure Content Canvas
- Rectangular window: `width: 800px, height: 600px` (resizable)
- Background: Pure white (#FFFFFF) or black (#000000) based on content theme
- No chrome, controls, or moss branding visible
- Window title matches site title from content

**Website Controls**: Top Satellite Zone
- Window size: `width: 240px, height: 60px`
- Position: `x: preview.x + (preview.width/2) - 120, y: preview.y - 80`
- Background: Transparent with 20% opacity dark overlay
- Circular controls: `60px diameter, 10px spacing`
- Controls: Publish (primary), Edit, Settings
- Shadow: `0 4px 12px rgba(0,0,0,0.15)`

**Syndication Controls**: Right Satellite Zone  
- Window size: `width: 60px, height: variable (60px per platform)`
- Position: `x: preview.x + preview.width + 20, y: preview.y + 60`
- Circular controls: `60px diameter, 10px vertical spacing`
- Each uses platform brand colors and official icons
- Auto-expands/contracts based on active integrations

### Interaction Specifications

**Magnetic Positioning Implementation**
- Update satellite positions on `appWindow.onMoved()` event
- Recalculate positions using preview window bounds
- Smooth transition: `transition: all 150ms ease-out`
- Maximum update frequency: 60fps to prevent performance issues

**Progressive Disclosure Rules**
- Show max 3 controls in top zone by default
- Additional controls via context menu on Settings button  
- Show max 5 platforms in right zone initially
- Scroll/paginate for additional syndication targets

**Animation Specifications**
- Hover scale: `transform: scale(1.1)` over `100ms ease-out`
- Click feedback: `transform: scale(0.95)` for `50ms`, then return
- Loading states: Subtle rotation animation at `1 revolution per 2 seconds`
- Success flash: Brief green border `#4CAF50` for `300ms`

## Visual Hierarchy Implementation

### Z-Index Layers
- **Content window**: `z-index: 1000`
- **Control windows**: `z-index: 1100` 
- **Context menus**: `z-index: 1200`
- **Modals/dialogs**: `z-index: 1300`

### Control Priority Sizing
- **Primary actions** (Publish): `60px diameter`
- **Secondary actions** (Edit, Settings): `50px diameter`  
- **Platform syndication**: `45px diameter`
- **Utility controls**: `40px diameter`

### Color Priority System
- **Primary action**: moss green `#2E7D32` (high contrast)
- **Secondary actions**: System gray `rgba(100,100,100,0.8)`
- **Platform specific**: Use official brand colors
- **Disabled state**: `rgba(100,100,100,0.3)`

## Animation Implementation Details

**Transition Specifications**
- Default transitions: `all 150ms cubic-bezier(0.4, 0, 0.2, 1)`
- Hover scale: `transform: scale(1.1)` - use `transition: transform 100ms ease-out`
- Position changes: `transition: left 150ms ease-out, top 150ms ease-out`
- Opacity changes: `transition: opacity 200ms ease-in-out`

**Performance Optimization**
- Use `transform` and `opacity` for animations (GPU accelerated)
- Avoid animating `width`, `height`, `left`, `top` directly where possible
- Maximum 200ms duration to maintain responsiveness
- Use `will-change: transform` sparingly and remove after animation

**State Feedback Implementation**
- **Loading**: Rotate icon 360° every 2s using `@keyframes rotation`
- **Success**: Flash border color to `#4CAF50` for 300ms then fade
- **Error**: Flash border color to `#F44336` for 300ms then fade
- **Progress**: Linear progress bar in button, `width: 0-100%` over action duration

## Platform Adaptation

### macOS Implementation
- **Tray icon**: SF Symbols template icon, auto-adapts to light/dark mode
- **Window vibrancy**: Use `NSVisualEffectMaterial.menu` for control windows
- **Shadows**: System-standard `NSShadow` with 4pt blur, 2pt offset
- **Transparency**: Full `NSVisualEffectView` support with backdrop filtering

### Windows Implementation  
- **Corner radius**: `border-radius: 6px` to match Windows 11 aesthetic
- **Backdrop**: Use `window-vibrancy` plugin with `blur` effect  
- **Accent colors**: Query system accent via Windows API for secondary elements
- **Shadows**: CSS `box-shadow: 0 8px 16px rgba(0,0,0,0.24)`

### Linux Implementation
- **Compositor detection**: Check for Wayland/X11 transparency support
- **Fallback**: Solid backgrounds with `rgba(240,240,240,0.95)` when transparency unavailable
- **Window decorations**: Standard decorations for accessibility
- **Theme neutrality**: Avoid GTK/Qt specific styling

## Plugin Integration Guidelines

### Plugin Control Standards
- **Shape requirement**: All controls must be circular, no exceptions
- **Size tiers**: Choose from `60px` (primary), `50px` (secondary), `45px` (platform), `40px` (utility)
- **Icon guidelines**: 24px icon centered in control, use platform official icons where available
- **Color freedom**: Use platform brand colors, but ensure 3:1 contrast ratio minimum

### Zone Assignment Rules
- **Top zone plugins**: Website functionality (themes, settings, export)
- **Right zone plugins**: Syndication and sharing (social platforms, newsletters)
- **Priority system**: Core plugins take precedence, others accessible via "More" control
- **Maximum density**: 3 in top zone, 5 in right zone before scrolling

### Implementation Requirements  
- **Hover states**: Required for all interactive elements
- **Loading states**: Must provide visual feedback during async operations
- **Error handling**: Use standard error flash animation pattern
- **No content interference**: Plugins cannot modify main preview window content

---

*These specifications ensure visual consistency across all moss implementations while maintaining the flexibility for platform-specific optimizations and plugin integration.*