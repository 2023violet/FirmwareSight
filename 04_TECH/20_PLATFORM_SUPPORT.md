---
title: "Platform Support Matrix"
doc_id: "FS-TECH-021"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Platform Support Matrix

## Product support tiers

| Platform | MVP dev | Release target | Tier |
|---|---|---|---|
| Windows 11 x64 | Yes | Yes | 1 |
| Windows 10 x64 | Test/compat | Yes if Tauri/WebView support validated | 1 |
| macOS Apple Silicon | CI/release | Yes | 2 |
| macOS Intel | CI/release | Yes where toolchain supports | 2 |
| Ubuntu LTS x64 | CI/release | Yes | 2 |
| Other Linux distros | best effort | no blanket claim | 3 |
| Windows ARM64 | deferred | no MVP | future |
| Linux ARM64 | deferred | no MVP | future |
| Mobile | no | no | out of scope |

## Why Windows Tier 1

Embedded/MCU toolchains and existing customer workflows are disproportionately Windows-centric.
FirmwareSight must be excellent on Windows rather than merely “builds on Windows”.

## WebView policy

Tauri uses platform WebViews.
Therefore:
- avoid browser-experimental APIs；
- test CSS/keyboard/file dialogs on each platform；
- do not assume Chromium-only behavior。

## Filesystem

All internal code uses `Path` / `PathBuf`.
Never concatenate path strings manually.
Portable exported paths should be release-relative where possible.
