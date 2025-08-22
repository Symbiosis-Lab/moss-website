# Content Detection Logic

> Simple, clear rules that match how people actually organize content

## Core Philosophy

**Keep it simple.** Most users follow one of two patterns:
1. **Flat directory** - everything in one folder
2. **Homepage + subdirectories** - main files at top, collections in folders

Don't over-engineer. Cover these cases well, ignore edge cases that add complexity.

## Simple Detection Rules

### 1. Find the Homepage (Priority Order)

```
1. index.md       → Markdown homepage
2. index.pages    → macOS Pages document
3. index.docx     → Word document
4. README.md      → Project description
5. First document file → Fallback content
```

### 2. Determine Site Type

```
IF has subdirectories with documents:
  → Homepage + Collections
  - Top-level files = main pages (homepage, about, contact)
  - Subdirectories = content collections (blog posts, projects, etc.)

ELIF root has ≤5 document files:
  → Simple Flat Site
  - All files in navigation menu

ELSE:
  → Blog-style Flat Site  
  - Homepage + essential pages in menu
  - Other files listed chronologically on homepage
```

## Two Main Patterns

### Pattern 1: Flat Directory
```
my-site/
├── index.md           # Homepage
├── about.md           # Page
├── contact.md         # Page  
├── post1.md           # Page
├── post2.md           # Page
└── image.jpg          # Asset

Result: Simple site with navigation menu
```

### Pattern 2: Homepage + Collections
```
my-site/
├── index.md           # Homepage
├── about.md           # Main page
├── posts/             # Collection
│   ├── hello.md
│   └── update.md
└── projects/          # Collection
    ├── project1.md
    └── project2.md

Result: Homepage + auto-generated collection pages
```

## Implementation

### Homepage Detection
- Look for homepage file in root directory only
- Use priority order: `index.html` > `index.md` > `README.md` > first `.md`

### Site Structure Detection
- Count files in root vs subdirectories
- If subdirectories have content → Homepage + Collections
- Otherwise → Flat Site

### Navigation Generation
- **Flat Site (≤5 files)**: All document files become menu items
- **Flat Site (>5 files)**: Homepage + essential pages in menu, rest listed on homepage like a blog
- **Homepage + Collections**: Root files + folder names become menu items

### File Count Rules
- **≤5 files**: Simple navigation menu with all files
- **>5 files**: Convert to blog-style listing to avoid menu clutter

## Supported File Types

### Document Files (Priority Order)
- `.md` files - Markdown (convert to HTML)
- `.pages` files - macOS Pages (convert to HTML)
- `.docx` files - Word documents (convert to HTML)
- `.doc` files - Legacy Word documents (convert to HTML)

### Homepage Files
- `index.md` - Markdown homepage
- `index.pages` - Pages homepage
- `index.docx` - Word homepage
- `README.md` - Documentation style
- First document file found - Fallback

### Other Files
- Images (jpg, png, gif, etc.) - Optimize and copy
- PDFs - Copy as-is for download
- Other files - Copy as-is

## Examples

### Small Site (Simple Flat)
```
portfolio/
├── index.md
├── about.md
├── contact.md
└── photo.jpg

→ Simple site with 3-item navigation menu
```

### Large Blog (Blog-style Flat)
```
blog/
├── index.md           # Homepage with post listing
├── about.md           # In menu
├── contact.md         # In menu
├── post1.md           # Listed on homepage
├── post2.md           # Listed on homepage
├── post3.md           # Listed on homepage
├── post4.md           # Listed on homepage
├── post5.md           # Listed on homepage
└── old-post.md        # Listed on homepage

→ Homepage + About/Contact menu + 6 posts listed chronologically
```

### Business Site (Collections)
```
business/
├── index.pages     # Homepage (Pages document)
├── about.docx      # About page (Word document)
├── services/       # Services collection
│   ├── consulting.md
│   └── training.md
└── blog/          # Blog collection
    ├── news1.md
    └── news2.md

→ Homepage + About page + Services section + Blog section
```

### Writer's Portfolio (Mixed Documents)
```
writing/
├── index.md       # Markdown homepage
├── bio.pages      # Pages bio document
├── articles/      # Article collection
│   ├── essay1.docx
│   ├── story.pages
│   └── review.md
└── images/
    └── headshots/

→ Multi-format document site with organized collections
```

---

*Simple rules. Clear outcomes. No surprises.*