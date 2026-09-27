---
title: "ADR-0016 Toolchain Pinning"
doc_id: "ADR-0016"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Decision
Bootstrap pin:
- Rust 1.98.1
- Edition 2024
- Node 24 LTS
- pnpm 12 exact at creation
- Tauri 2 stable
- React 19
- Vite 8

Lockfiles are committed.

# Why
Reproducibility matters more than floating to newest releases.

# Upgrade
Toolchain upgrades are explicit PRs with full CI.
Major architecture dependency upgrades require ADR.
