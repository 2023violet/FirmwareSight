---
title: "V0 Internal Dry Run Report"
doc_id: "FS-V0-041"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Research / Engineering"
last_updated: "2026-09-27"
---

# Internal Dry Run Report

## Scope

This is an internal functional/scope dry run, **not** an external participant session and not counted toward V0 N.

## Passed

- `state.js` syntax: PASS
- `app.js` syntax: PASS
- pure state-machine end-to-end sequence: PASS
- no Cargo.toml under V0 workspace: PASS
- no `src-tauri` under V0 workspace: PASS
- no Rust `.rs` source under V0 workspace: PASS
- CSS gradients: 0
- CSS backdrop-filter: 0
- `prefers-reduced-motion`: present
- `focus-visible`: present
- export controls: visually present where appropriate, implementation disabled
- Blob/download export API: absent
- static HTML asset references: PASS

## State-machine sequence exercised

1. STATE A: MAP absent; 2 Unknown map-dependent rules.
2. Add MAP → STATE B.
3. Accept OTA review → immutable acceptance record with `original_state=REVIEW`.
4. Apply simulated signature fix.
5. Re-run Gate.
6. Bundle becomes eligible.
7. Build Bundle → History.
8. Invalid candidate ELF → STATE C.
9. Recover → Last Good Artifact / MAP state preserved.

## Browser renderer note

A headless Chromium screenshot/DOM smoke was attempted in the execution container, but the container Chromium GPU process terminated before page render.

This is recorded as an **environment/tooling limitation**, not a prototype PASS or FAIL.

No browser-level interaction result is fabricated from that failed renderer attempt.

The prototype is dependency-free static HTML/CSS/JS and is intended to be opened in a normal desktop browser for the moderator dry run and external sessions.

## External usability dry run

Not performed because no external or internal human participant pool was provided.

Formal external N remains 0.
