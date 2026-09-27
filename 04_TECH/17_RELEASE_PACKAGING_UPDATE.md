---
title: "Build, Packaging, Signing and Update Baseline"
doc_id: "FS-TECH-018"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Build / Package / Sign / Update

## 1. Product lifecycle

FirmwareSight itself follows:

```text
Build
 ↓
Test
 ↓
Bundle
 ↓
OS Code Sign
 ↓
Installer
 ↓
Release
 ↓
Update Metadata
 ↓
Signed Auto Update (when enabled)
```

## 2. MVP distribution targets

### Windows — Tier 1
- x86_64 MSVC
- NSIS primary installer
- MSI may be produced for enterprise evaluation
- Authenticode signing before stable commercial release

### macOS — Tier 2 initially
- arm64
- x86_64
- Developer ID signing + notarization before stable distribution

### Linux — Tier 2
- x86_64
- AppImage
- `.deb`
- document WebKitGTK dependency expectations

## 3. CLI

Release same version as desktop.
Artifacts:
- Windows zip
- macOS/Linux tar.gz
- SHA-256

Package managers (Scoop/Homebrew/etc.) are post-MVP distribution channels.

## 4. Auto updater

Architecture is planned from v0.2.0, but updater is **disabled in MVP baseline**.

Enable only when:
- release hosting selected；
- update signing key generated；
- private key storage procedure exists；
- rollback/recovery tested；
- OS signing story documented；
- offline/enterprise disable switch exists。

Tauri updater signatures are separate from OS code signing.

## 5. Update key

Private updater key:
- never committed；
- CI secret/HSM/secure vault；
- documented recovery；
- ownership assigned。

Losing updater private key can strand installed clients, so this is a high-risk operational asset.

## 6. Versioning

SemVer:
`MAJOR.MINOR.PATCH`

Tag:
`vX.Y.Z`

Release pipeline must validate:
- Cargo package versions；
- Tauri app version；
- frontend package metadata；
- Git tag
are consistent.

## 7. Release provenance

FirmwareSight release publishes:
- installers/packages；
- CLI archives；
- SHA256SUMS；
- changelog；
- source commit；
- dependency/license report；
- update metadata only when updater enabled。
