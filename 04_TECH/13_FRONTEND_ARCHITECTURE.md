---
title: "Frontend Architecture"
doc_id: "FS-TECH-014"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Frontend Architecture

## Stack

- React 19.3
- TypeScript strict
- Vite 8
- CSS custom properties
- CSS Modules
- browser SVG/Canvas only where useful

## Directory

```text
apps/desktop/ui/src/
├── app/
├── features/
│   ├── analyze/
│   ├── compare/
│   ├── release/
│   └── project/
├── components/
├── design-system/
├── ipc/
├── routes/
├── hooks/
└── types/
```

## State ownership

### Local component state
default.

### Feature state
use reducer/hooks when multiple subcomponents share state.

### Durable domain state
lives in Rust/SQLite, not React store.

### Server-style cache
not needed; Tauri backend is local.

Do not add Redux/Zustand until state complexity is measured.

## Data tables

100k symbols must not be copied to frontend as one payload.

UI requests:
- filter；
- sort；
- page/cursor；
- selected columns。

Backend/SQLite returns bounded rows.

If rendering still becomes bottleneck:
- add row virtualization.

## Styling

No full UI framework baseline.

Reasons:
- FirmwareSight has a strong bespoke engineering-instrument visual language；
- full frameworks lock spacing/typography/component grammar；
- CSS Modules keep ownership local and inspectable。

Accessible headless primitives may be added per component when focus management is non-trivial.

## Charts

MVP:
- CSS bars；
- SVG；
- simple Canvas；
- no chart library.

Treemap only after it improves size analysis.
No 3D/data-decoration charts.

## Error/loading/empty

Each feature must explicitly implement:
- loading；
- partial capability；
- empty；
- error；
- stale；
- success。

`Unknown` is a domain state, not an empty UI state.

## TypeScript strictness

Enable at least:
- `strict`
- `noUncheckedIndexedAccess`
- `exactOptionalPropertyTypes`
- `useUnknownInCatchVariables`

No `any` across IPC.
