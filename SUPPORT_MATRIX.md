# Support Matrix

Updated: 2026-02-08

This document defines the supported scope for the `0.9 RC -> 1.0` cycle.

## Platform Support

| Platform | Status | Distribution | Notes |
|---|---|---|---|
| Windows (x86_64) | Supported | GitHub Release binary | Primary desktop target for RC users |
| Linux (x86_64 / arm) | Supported | GitHub Release binary | Requires system GUI/runtime dependencies |
| macOS (aarch64) | Best effort RC | GitHub Release binary | Included in cross-build matrix |
| Web (wasm32 + trunk) | Supported (limited) | GitHub Pages / trunk build | Browser sandbox limits file-system features |

## Capability Matrix

| Capability | Desktop | Web |
|---|---|---|
| Load TXT via native file picker | ✅ | ⚠️ Limited by browser/runtime |
| Select output folder | ✅ | ❌ Not available |
| Open output folder/file in system manager | ✅ | ❌ Not available |
| Cover/font/image selection via native dialogs | ✅ | ⚠️ Limited by browser/runtime |
| TXT -> EPUB conversion flow | ✅ | ✅ |
| Chapter preview/editor flow | ✅ | ✅ |
| CSS/style customization | ✅ | ✅ |

## Known Limitations (1.0 Scope)

- Browser runtime cannot guarantee native OS file/folder dialog behavior.
- Browser runtime cannot launch system file manager (`open folder/file`).
- Extremely large media assets may fail due to browser memory limits.
- Runtime behavior differs by browser engine and security policy.

## Recommended User Guidance

- Use desktop build for heavy file workflows and local output management.
- Use web build for light conversion and quick preview/edit scenarios.
- Include platform/browser info when reporting issues.

