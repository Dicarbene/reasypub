# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Release governance hardening for `1.0.0` (checklist, support matrix, and issue triage).
- Feedback loop improvements based on desktop + web users.

## [0.95.0] - 2026-02-08

### Added
- Optional TOC settings for conversion output:
  - custom TOC title override (empty => locale default)
  - include/exclude gallery chapter from TOC entries
  - unified TOC page toggle state shared by sidebar and output settings panel

### Changed
- Conversion/build API now uses `TocOptions` instead of a standalone `inline_toc` flag.
- Output/Misc panel adds a dedicated TOC settings section.

### Improved
- EPUB generation now keeps gallery content while allowing TOC visibility control.
- TOC metadata/title handling is configurable without breaking default behavior.

### Tests
- Added TOC-focused unit tests for title override/fallback and gallery TOC visibility.
- Added integration flow test covering TOC override + gallery hidden from TOC.

## [0.9.0-rc.1] - 2026-02-08

### Added
- Chapter editor workflow improvements (open/close/edit/reorder/use-for-conversion path).
- Cross-platform `check.sh` and `check.ps1` CI-like local gate scripts.
- Release governance docs:
  - `CHANGELOG.md`
  - `SUPPORT_MATRIX.md`
  - `RELEASE_CHECKLIST.md`
- GitHub Issue templates for bug reporting and reproducible feedback.

### Changed
- Version bump from `0.1.0` to `0.9.0-rc.1`.
- CI artifact naming unified to `reasypub-${target}`.
- README updated with RC -> 1.0 lifecycle guidance.

### Fixed
- `wasm32-unknown-unknown` build compatibility by enabling `uuid` `js` feature.
- Strict clippy failures (`-D warnings`) by code-equivalent refactors.
- Formatting drift across source files (`cargo fmt --all`).
- Trunk/web build pipeline mismatches in CI.
- User-facing startup resiliency (avoid panic/expect crash paths on startup failures).

### Known Limitations
- Web build does not support native OS file/folder dialogs.
- Some desktop-only operations (open folder/file manager) are unavailable in browser runtime.

## [0.1.0] - 2026-02-08

### Added
- Initial TXT -> EPUB desktop/web app baseline with chapter splitting and metadata editing.
- Cover/font/CSS/images customization and i18n-enabled UI.

