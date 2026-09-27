---
title: "CI and Supply Chain Baseline"
doc_id: "FS-TECH-019"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# CI / Supply Chain

## PR checks

### Rust
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`

### Frontend
- frozen pnpm install
- TypeScript typecheck
- lint
- unit/component tests
- production build

### Contract
- schema/golden validation
- fixture smoke

## Cross-platform matrix

Minimum:
- windows-latest
- ubuntu-latest
- macos-latest

Parser/core tests run all three where practical.

## Security / dependency

Adopt:
- `cargo-deny` for license/source/advisory/bans policy
- dependency update bot
- pnpm frozen lockfile
- explicit allowed licenses

Do not auto-merge major dependency changes.

## Release CI

Release is tag-triggered only after manual approval.

Stages:
1. validate tag/version
2. test
3. build matrix
4. code sign
5. bundle
6. smoke artifacts
7. checksums
8. draft release
9. updater artifacts only when enabled

## Build reproducibility

Record:
- rustc version
- Cargo.lock hash
- Node version
- pnpm version
- pnpm-lock hash
- Tauri version set
- runner image
- Git commit

## Secrets

CI secrets may contain:
- code signing credentials
- updater signing key
- notarization credentials

Never expose to PRs from untrusted forks.
