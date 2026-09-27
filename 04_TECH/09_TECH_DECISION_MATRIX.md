---
title: "Technical Decision Matrix"
doc_id: "FS-TECH-010"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Technical Decision Matrix

## Legend
- **NOW** — enters initial repository
- **EDGE** — allowed only at interface/application edge
- **TRIGGERED** — pre-approved, but not installed until trigger exists
- **DEFER** — intentionally excluded
- **REJECT** — not aligned with current product

| Technology | Decision | Reason | Trigger / Boundary |
|---|---|---|---|
| Rust 2024 | NOW | core correctness/cross-platform | all Rust crates |
| Serde | NOW | stable boundary serialization | contracts/config/export |
| thiserror | NOW | typed library errors | library crates |
| anyhow | EDGE | rich app context | CLI/Tauri boundary only |
| Clap | NOW | first-class CLI | `apps/cli` |
| Tracing | NOW | structured diagnostics | binaries + spans |
| Tokio | EDGE | desktop job coordination | never core |
| Tauri 2 | NOW | cross-platform shell | desktop only |
| React | NOW | product-grade UI ecosystem | desktop UI |
| TypeScript | NOW | typed frontend | strict mode |
| Vite | NOW | frontend build/dev | desktop UI |
| SQLite | NOW | local relational state | storage adapter |
| rusqlite | NOW | SQLite-only sync model | storage crate |
| SQLx | DEFER | async/multi-DB unnecessary | server/multi-DB ADR |
| object | NOW | ELF/object parsing | artifact adapter |
| gimli | NOW/optional | DWARF evidence | feature/use only when needed |
| sha2 | NOW | standard release SHA-256 | artifact/release |
| Reqwest | TRIGGERED | HTTP default if needed | updater/feed/license |
| Rustls | TRIGGERED | Rust-native TLS | with Reqwest |
| wgpu | TRIGGERED | GPU island | measured rendering bottleneck |
| Axum | DEFER | no server | future server product |
| Tonic | DEFER | no gRPC | future RPC requirement |
| egui | REJECT main shell | design mismatch | possible diagnostic subtool only via ADR |
| iced | REJECT main shell | no current Native need | shell migration ADR |
| Slint | REJECT main shell | embedded GUI not product target | shell migration ADR |
| GPUI/Floem | REJECT baseline | ecosystem/custom UI cost | only if UI is core IP |
| Redux | DEFER | state complexity not proven | revisit after real need |
| Tailwind | DEFER | custom engineering design system | not baseline |
| Full UI kit | DEFER | avoid visual lock-in | headless primitive per need |
| Cloud DB | DEFER | local-first | team/cloud phase |
| Telemetry SDK | REJECT MVP | privacy/no need | explicit opt-in ADR |

## Principle

Approved route does not mean installed dependency.

The dependency graph is a product cost:
- security;
- build time;
- update burden;
- licenses;
- binary size;
- cognitive load.

Only `NOW` dependencies belong in initial bootstrap.

## v0.4 Additions

| Technology / Model | Decision | Boundary |
|---|---|---|
| ts-rs | NOW / EDGE | Desktop/Application IPC DTO generation only |
| five-state Gate | LOCKED | Core + UI + portable schema |
| dual memory accounting | LOCKED | Core normalization/gate |
| portable schema strictness | LOCKED | Release Bundle contracts |

`ts-rs` does not become a Core dependency requirement for domain semantics; it belongs to the adapter/contract boundary.
