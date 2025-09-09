# Known Issues & Workarounds

This document tracks known issues with workarounds in the moss codebase. These should be addressed or removed when the underlying dependencies fix the root causes.

## Resolved Issues (2025-09-08)

The following issues have been resolved through implementation of activation policy switching and proper dialog handling. References have been moved to the relevant code locations:

- **Tauri macOS Full-Screen Application Dialog Issues** - Resolved via activation policy switching in `compile.rs`
- **Tauri macOS File Dialog Positioning Bug** - Resolved via activation policy switching in `compile.rs`
- **Menu Bar App Dialog Best Practices** - Implemented following industry standard patterns in `compile.rs`
- **Tauri macOS Dialog Window Visibility Issue** - Resolved via explicit window hide calls
- **Tauri Dialog Positioning Issue** - Resolved via persistent dialog anchor window

All technical references and implementation details have been moved to code comments where the fixes are implemented.
