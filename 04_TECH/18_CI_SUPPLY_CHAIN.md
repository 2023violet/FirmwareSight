---
title: "CI and Supply Chain Baseline"
doc_id: "FS-TECH-019"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-03"
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

### What those fields do not buy (P5, 2026-10-03)

The list above records a build's *inputs*. It is not a claim that two builds produce equal *bytes*, and P5
measured the difference instead of assuming it: two consecutive builds of one tree on one Windows host gave
payload binaries of identical length with **20 differing bytes**, all of them linker identity — the PE
`TimeDateStamp` (also copied into three debug-directory entries) and the 16-byte RSDS CodeView GUID — and
installers of different sizes, here and on the runner. The full measurement is §5e of
`P5_VALIDATION/P5_PACKAGING_REPORT.md`.

The comparison was then made across two CI runs whose heads differ only in `scripts/` and docs —
`git diff --stat 1055242..53578e9` names no `crates/` or `apps/` file — so every byte the compiler saw was the
same. Every container changed: the `.dmg` at identical length, the `.deb` by four bytes, both CLI archives.
One artifact did not change at all — the macOS `.app`'s aggregate tree digest is the same `eb12407b…` in
both, which means its whole bundle, executable and derived `.icns` included, came back byte-identical from
`macos26` twice. That is a measurement, not a licence: two runs on one image is not reproducibility, and the
`.dmg` wrapped around that identical bundle still changed. The rule below is what the evidence supports.

So: a package digest identifies **the artifact set a stranger was given**, and `SHA256SUMS.txt` exists so
they can check that nothing in it changed after the build; it is not a comparison key across builds, and no
re-release, cache or attestation in this project may be justified by "the digest would have matched". Two
artifacts of the same version built at different times are expected to differ. Where a claim of stability is
actually needed, it belongs to the analysed firmware evidence inside a Release Bundle, which is content
addressed by design, not to a PE file the linker stamps.

## Secrets

CI secrets may contain:
- code signing credentials
- updater signing key
- notarization credentials

Never expose to PRs from untrusted forks.
