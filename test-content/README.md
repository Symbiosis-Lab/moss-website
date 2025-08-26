# Test Content for moss Development

This directory contains test websites for developing and testing moss publishing functionality.

## Test Sites

### `simple-blog/`
Basic 3-page site with homepage, about, and contact pages.
- Tests: Homepage detection, navigation generation, markdown processing
- Use for: Basic publishing workflow verification

## Usage

### Development Testing
1. **Start dev app**: `npm run tauri dev`
2. **Click test button**: "🧪 Test Publish" in the moss window  
3. **View result**: Opens `http://localhost:3000` with beautiful generated site

### Production Testing  
Right-click any folder and select "Publish to Web" to test the full workflow.

**Note**: Deep links (`moss://`) only work in production builds, not development mode on macOS.

## Adding New Test Sites

Create new directories with different content patterns:
- `portfolio/` - Image-heavy site
- `documentation/` - Nested folders with organized content
- `blog-posts/` - Many markdown files for blog-style sites

Each test site should represent a different user scenario.