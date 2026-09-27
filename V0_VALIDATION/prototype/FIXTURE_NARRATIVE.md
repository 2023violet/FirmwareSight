---
title: "V0 Fixture Narrative"
doc_id: "FS-V0-003"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Product / Design"
last_updated: "2026-09-27"
---

# Fixture Narrative

Project: `relay-controller`

Baseline: `v0.2.1`  
Current: `v0.3.0-rc2`

## Memory/change facts

- baseline flash image footprint: 474.8 KB
- current flash image footprint: 486.2 KB
- net delta: +11.4 KB
- budget: 512.0 KB
- `.ota_staging`: +24.0 KB
- `.text`: +1.9 KB
- `.rodata`: +1.2 KB
- `.data`: -9.5 KB
- `.legacy_log`: Removed

## Gate narrative

- `signed_image.required`: BLOCK until prototype signing fix
- `flash_budget.soft`: REVIEW, accepted before session start as historical example
- `ota_staging.new_section`: REVIEW, pending
- `map_parity`: UNKNOWN in STATE A, PASS in STATE B
- `symbol_audit`: UNKNOWN in STATE A, PASS in STATE B
- deterministic PASS examples: entry point, hard size budget, ELF structure, version match
- SBOM rule: N/A

## Dependency narrative

- lwIP 2.1.3 — Observed
- FreeRTOS 10.4.0 — Declared
- mbedTLS — Unknown; symbol fingerprint detected, no version string
- vendor_blob_* — Unknown
- CMSIS 5.6.0 — Observed

## Timeline

STATE A:
ELF valid / MAP absent / Git absent.

A→B:
explicit `Add .map file...` → `firmware.map`.

STATE B:
MAP loaded / 1,284 symbols / dependency evidence enabled / MAP-dependent Gate re-evaluated.

STATE C:
invalid ELF replacement attempt; candidate fails; Last Good Artifact preserved.

STATE D:
recover valid workspace → Bundle & History.
