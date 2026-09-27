---
title: "ADR-0002 Desktop Stack"
doc_id: "ADR-0002"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted, refined in v0.2.0.

# Context
FirmwareSight is a data-heavy engineering desktop application with tables, filters, diff views and reports. It is not a custom GPU editor.

The supplied Rust ecosystem census found Tauri + Web UI to be the most frequent route in its reviewed high-star ordinary desktop product sample, while also showing that Native Rust GUI is appropriate for different constraints.

# Decision
Use:
- Tauri 2 stable
- React 19
- TypeScript strict
- Vite 8
- CSS Modules + project tokens

Do not use Tauri 3 alpha.

# Why React rather than Svelte
Both are valid in the research.
React is chosen for:
- larger component/testing ecosystem；
- easier access to engineers/AI training corpus；
- mature table/accessibility tooling；
- no performance requirement that favors a framework change。

# Consequences
Positive:
- high UI iteration speed；
- bespoke design system；
- Rust core remains reusable；
- mature accessibility/testing ecosystem。

Negative:
- WebView platform differences；
- Rust + TS two-language stack；
- IPC boundary must be disciplined。

# Revisit trigger
Only if:
- WebView blocks a required product capability；
- measured UI performance remains unacceptable after data/virtualization optimization；
- OS integration requires a different shell。

A shell migration must not require rewriting Core.
