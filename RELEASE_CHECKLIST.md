# Release Checklist

Updated: 2026-02-08

This checklist is the Go/No-Go gate for `0.9 RC -> 1.0.0`.

## 1) Scope Freeze

- [ ] Target version is declared (`0.9.0-rc.N` or `1.0.0`).
- [ ] RC branch receives only low-risk changes (no schema/core algorithm rewrites).
- [ ] Every late feature change includes rollback commit strategy.

## 2) Quality Gates (must be green)

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all`
- [ ] `cargo check --workspace --all-targets`
- [ ] `cargo check --workspace --all-features --lib --target wasm32-unknown-unknown`
- [ ] `cargo test --workspace --all-targets --all-features`
- [ ] `cargo test --workspace --doc`
- [ ] `trunk build`
- [ ] `bash ./check.sh`
- [ ] `./check.ps1`

## 3) Defect Bar

- [ ] P0 open defects = `0`
- [ ] P1 open defects = `0`
- [ ] No repeated crash class during RC observation window

Severity convention:
- P0: crash / data corruption / conversion impossible
- P1: core workflow broken
- P2: UX issue / non-blocking behavior

## 4) Manual Regression

- [ ] Desktop: TXT import -> chapter preview -> chapter editor -> EPUB output
- [ ] Desktop: cover + header image + illustration + font + CSS mixed scenario
- [ ] Web: startup + key interactions + conversion flow + unsupported capability hint
- [ ] Error path: empty text / invalid regex / missing config / oversized image

## 5) Release Artifacts

- [ ] Git tag exists and matches version
- [ ] Release binaries uploaded with `reasypub-${target}` naming
- [ ] `SHA256SUMS` uploaded to GitHub Release assets
- [ ] Release notes include support matrix and known limitations

## 6) Post-release

- [ ] Announce feedback channels and issue template usage
- [ ] Track P0/P1 SLA (P0: 24h, P1: 72h)
- [ ] `1.0.x` bugfix-only policy acknowledged

