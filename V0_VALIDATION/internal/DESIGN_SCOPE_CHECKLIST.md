---
title: "V0 Internal Design and Scope Checklist"
doc_id: "FS-V0-040"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Design / Engineering"
last_updated: "2026-09-27"
---

# Internal Design / Scope Checklist

## Design
- [x] Light-only
- [x] no gradients
- [x] no glass/neon/purple styling
- [x] no decorative card/panel/table shadows
- [x] overlay shadow only
- [x] state = icon + label
- [x] mono numbers/hash/version/path/time
- [x] one primary action per local action region
- [x] table + ranked bars only
- [x] `prefers-reduced-motion`
- [x] visible focus ring
- [x] dialog focus trap + Escape + focus return
- [x] responsive fallback around 1024px-class viewport

## Semantics
- [x] five-state Gate
- [x] Unknown is not rewritten
- [x] acceptance does not turn finding into PASS
- [x] Declared remains separate from Observed
- [x] Added/Removed uses absence marker, not zero
- [x] MAP A→B explicit transition
- [x] Last Good Artifact preserved on parse failure

## Scope
- [x] no Rust
- [x] no Cargo
- [x] no Tauri
- [x] no SQLite
- [x] no parser
- [x] no real export
- [x] no E1/E2/E3/GX implementation
