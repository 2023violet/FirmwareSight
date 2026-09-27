---
title: "CI/CD Baseline v0.4"
doc_id: "FS-ENG-007"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-27"
---

# CI/CD Baseline

## Event matrix

| Event | Windows | Linux | macOS | Packaging |
|---|---|---|---|---|
| Pull Request | required core + UI | required core | optional/cost-controlled smoke | no |
| main push | required | required | **required core smoke** | no |
| nightly | required | required | required | optional package smoke |
| release tag | required | required | required | **all declared release targets** |

这解决 v0.3.0 中“PR macOS optional”与“main required macOS”未说明事件边界的问题。

## PR checks

Rust:
- fmt
- clippy
- workspace tests
- fixture/golden tests
- schema validation

Frontend:
- frozen pnpm install
- TS typecheck
- lint/test/build
- generated IPC type clean check
- UI design checklist mechanical subset when available

## Security/dependencies

- cargo-deny
- dependency update bot
- license/source/advisory policy
- no automatic major merge

## Release

Protected tag/manual approval:
1. version/tag validation
2. full test
3. build matrix
4. code signing/notarization where required
5. bundle/installers
6. package smoke
7. checksums
8. draft release
9. updater artifacts only after updater ADR conditions are active

## Secrets

Signing/updater/notarization secrets never exposed to untrusted fork PRs.
