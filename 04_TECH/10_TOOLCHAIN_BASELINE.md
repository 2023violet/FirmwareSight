---
title: "Toolchain and Version Baseline"
doc_id: "FS-TECH-011"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Toolchain Baseline

Snapshot date: 2026-09-26

## Rust

- toolchain: `1.98.1`
- channel: stable point release
- edition: `2024`
- Cargo resolver: `3`
- components:
  - rustfmt
  - clippy
- targets initially:
  - x86_64-pc-windows-msvc
  - x86_64-unknown-linux-gnu
  - aarch64-apple-darwin
  - x86_64-apple-darwin

### Policy
`rust-toolchain.toml` pins the exact validated release.
Do not use floating `stable` in release CI.

Upgrade:
1. dedicated PR;
2. all fixtures;
3. cross-platform CI;
4. package smoke;
5. update baseline/current-state if minimum changes.

## Node

- baseline: Node.js 24 LTS
- repository should pin via `.node-version` or equivalent
- do not use Node Current for release reproducibility

## pnpm

- line: pnpm 12
- baseline research date current: 12.7.0
- repository `packageManager` field pins exact version

## Frontend

- React 19.3
- TypeScript current compatible release pinned in lockfile
- Vite 8.x
- no experimental Vite mode by default

## Tauri

- Tauri major: 2 stable
- do not adopt Tauri 3 alpha
- Tauri crates/plugins stay on mutually compatible stable release set

## Version ownership

Exact dependency versions live in:
- `Cargo.lock`
- `pnpm-lock.yaml`
- `rust-toolchain.toml`
- package manager pin

Baseline docs own:
- architecture family;
- major-version policy;
- update procedure.

This prevents docs from becoming a second lockfile.
