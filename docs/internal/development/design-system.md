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

## Cognitive Load in Generated Sites

**Mental Model Alignment**: Site structure matches users' existing expectations. Navigation follows familiar web patterns without requiring learning new interaction models.

**Flat Over Deep**: Avoid complex hierarchical navigation. Information architecture prioritizes discoverability over organizational perfection.

**Progressive Disclosure**: Essential features visible immediately, advanced options discoverable when needed. Users shouldn't think about site mechanics to access content.

**Predictable Layouts**: Consistent patterns reduce mental mapping overhead. Every page type follows established visual logic users can internalize once.

## CSS Architecture Patterns

### Container Pattern Implementation
**Universal Container Class**: Single responsibility for width, centering, and padding across all layout sections.

```css
.container {
    max-width: var(--moss-site-max-width);
    margin: 0 auto;
    padding: 0 var(--moss-container-padding);
}
```

**Class Composition Over Complexity**: Combine classes for specific needs rather than creating complex calculations.

```html
<!-- Compose behaviors -->
<nav class="main-nav container">
<main class="content-area container">

<!-- Not complex single-purpose classes -->
<nav class="nav-with-calculated-width-and-padding">
```

### Separation of Layout Concerns
- **Container**: Width, centering, padding
- **Inner Elements**: Visual styling (borders, grids)
- **Component Classes**: Component-specific behavior

**Borders Inside Containers**: Visual elements like borders belong on inner elements that don't have padding, not on containers that need spacing.

```css
/* Correct */
.nav-container { /* width + padding */ }
.nav-content { border-bottom: 1px solid; /* visual styling */ }

/* Wrong */  
.nav-container { /* width + padding + border = width conflicts */ }
```

### Pattern Recognition for CSS Issues
**Symptom**: Alignment issues persist through 5+ different fixes  
**Diagnosis**: Architecture problem, not implementation problem  
**Solution**: Step back and research industry patterns

**Complex Calculations Signal Wrong Abstractions**:
```css
/* Warning sign */
max-width: calc(var(--width) + 2 * var(--padding) - var(--gap));

/* Better abstraction */
.container { max-width: var(--width); }
.grid { gap: var(--gap); }
```

**Typography as Wayfinding**: Clear hierarchy guides attention without conscious effort. Readers focus on content, not figuring out page structure.

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

### Right Panel Architecture

**Immersive Full-Screen Preview**
```
┌─────────────────────────────────────────────┐
│  moss Preview - Full Screen               ▼ │ ← Minimal title bar  
├─────────────────────────────────────────┬─┐ │
│                                         │●│ │ ← Right panel (collapsible)
│     SACRED PREVIEW CONTENT              │●│ │   ● Publish controls
│     (Immersive, distraction-free)       │●│ │   ● Plugin actions
│                                         │●│ │   ● Settings  
│                                         │ │ │
│                                         │ │ │
└─────────────────────────────────────────┴─┘ │
```

**Panel Interaction Patterns**
- Auto-hide after 3 seconds of inactivity for immersion
- Reveal on hover near right edge (20px trigger zone)  
- Persistent during active operations (modals, loading states)
- Smooth slide transitions: `transform: translateX()` over `150ms ease-out`
- Panel width: `280px` (optimized for tool controls)

**Panel Sections Design**
```
┌─ Right Panel (280px) ─┐
│  ┌─ Publish ▼ ────┐   │ ← Collapsible section
│  │  ● Setup Git   │   │   (60px circular buttons)
│  │  ● Connect     │   │
│  │  ● Deploy      │   │
│  └────────────────┘   │
│  ┌─ Plugins ▼ ────┐   │
│  │  ● Theme       │   │ ← Plugin-contributed actions
│  │  ● Export      │   │   (50px circular buttons)  
│  │  ● Analytics   │   │
│  └────────────────┘   │
│  ┌─ Settings ▼ ───┐   │
│  │  ● Dark Mode   │   │ ← System preferences
│  │  ● Shortcuts   │   │   (45px circular buttons)
│  └────────────────┘   │
└────────────────────────┘
```

**Keyboard Shortcuts**
- `Cmd+Shift+P` (macOS) / `Ctrl+Shift+P` (Win/Linux): Command palette
- `Cmd+/` / `Ctrl+/`: Toggle right panel visibility
- `Cmd+Enter` / `Ctrl+Enter`: Execute primary publish action
- `Escape`: Hide panel or close modal dialogs

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

## Smart Publish Button Design Specifications

### Button State Visual Design
Following the circular action metaphor, the publish button transforms based on git detection:

```
┌─ Setup Git ─┐    ┌─ Connect GitHub ─┐    ┌─ Publish ─┐
│      ⚙️      │ →  │        🔗        │ →  │    🚀    │
│   (60px)    │    │     (60px)      │    │  (60px)  │
│ #6B7280     │    │    #2E7D32     │    │ #2E7D32  │
└─────────────┘    └─────────────────┘    └──────────┘
```

### State-Based Color Coding
- **Setup Git**: System gray `#6B7280` - indicates preparation needed
- **Connect GitHub**: moss green `#2E7D32` - ready for configuration  
- **Publish**: moss green `#2E7D32` - ready for action
- **Published**: Success green `#4CAF50` with checkmark icon

### Interactive Feedback
- **Hover**: Scale to `transform: scale(1.1)` with 100ms ease-out
- **Click**: Scale to `transform: scale(0.95)` for 50ms, then return
- **Loading**: Rotation animation at 1 revolution per 2 seconds
- **Success**: Brief flash to `#4CAF50` border for 300ms

### Repository Setup Modal

#### Modal Specifications
- **Size**: `400px × 280px` (compact footprint)
- **Position**: Centered over preview window
- **Background**: `rgba(0,0,0,0.4)` overlay with backdrop blur
- **Border radius**: `12px` matching system aesthetics
- **Shadow**: `0 8px 32px rgba(0,0,0,0.24)`

#### Form Layout
```
┌─── Connect to GitHub ──────────────────┐
│                                        │
│  Repository name: [my-awesome-blog  ]  │ ← Pre-filled with folder name
│                                        │
│  ☐ Make repository public             │ ← Unchecked by default (safe)
│                                        │
│  ☐ Enable GitHub Pages                │ ← Checked by default
│                                        │
│              [Cancel]  [Create & Publish]│
│                        ← moss green    │
└────────────────────────────────────────┘
```

#### Form Validation
- **Repository name**: Auto-sanitized (spaces→hyphens, lowercase)
- **Duplicate names**: Suggest alternatives with incremental numbers
- **Invalid characters**: Real-time correction with visual feedback
- **Empty name**: Disabled submit button until valid name entered

## Right Panel Plugin Architecture

### Plugin Registration Pattern
Plugins register actions in specific panel sections:

```rust
trait PanelPlugin {
    fn section(&self) -> PanelSection; // Publish, Plugins, or Settings
    fn get_action_button(&self) -> ActionButton;
    fn get_command_palette_entry(&self) -> CommandEntry;
    fn execute_action(&self, context: &PreviewContext) -> Result<String, String>;
}

enum PanelSection {
    Publish,   // Core publishing workflow (GitHub, Netlify, etc.)
    Plugins,   // Content tools (themes, export, analytics)
    Settings,  // App preferences (dark mode, shortcuts)
}
```

### Section Design Standards

**Publish Section (Priority 1)**
- Reserved for deployment platforms (GitHub Pages, Netlify, Vercel)
- 60px circular buttons with platform brand colors
- Smart state detection (Setup → Connect → Deploy → Published)
- Maximum 3 buttons before "More" overflow menu

**Plugins Section (Priority 2)**  
- Content manipulation tools (themes, export formats, SEO)
- 50px circular buttons with functional icons
- Grouped by related functionality with section dividers
- Maximum 5 buttons before scrolling

**Settings Section (Priority 3)**
- App-wide preferences and configuration
- 45px circular buttons with system-style icons
- Toggle states for boolean preferences (dark mode, auto-save)
- Always at bottom of panel

### Collapsible Section Behavior
- **Auto-collapse**: Sections with >3 items auto-collapse after 5 seconds
- **Smart persistence**: Active sections stay expanded during operations
- **State memory**: User collapse preferences saved per project
- **Visual hierarchy**: Expanded sections have subtle background tint

### Command Palette Integration
All plugin actions must be accessible via command palette:
- **Search by name**: Type action name for fuzzy matching
- **Search by section**: Prefix with "Publish:", "Plugin:", or "Settings:"
- **Keyboard shortcuts**: Custom shortcuts assignable to any action
- **Recent actions**: Most used actions appear at top

### Plugin Visual Standards
- **Consistent icons**: 24px centered icons, preferably from SF Symbols or Feather
- **Hover feedback**: Scale(1.1) with 100ms ease-out transition
- **Loading states**: Subtle rotation animation during async operations  
- **Status indication**: Badge icons for success/error/warning states
- **Brand colors**: Platform colors allowed but must meet 3:1 contrast ratio

---

*These specifications ensure visual consistency across all moss implementations while maintaining the flexibility for platform-specific optimizations and plugin integration.*