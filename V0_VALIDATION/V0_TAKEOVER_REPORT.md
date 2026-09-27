---
title: "FirmwareSight V0 Takeover Report"
doc_id: "FS-V0-001"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-09-27"
---

# FirmwareSight V0 接手报告

## 1. Authority read

The V0 authority set has been read:
- root baseline/brand/design/governance;
- MVP/product module and capability portfolio;
- all seven accepted UI screenshots;
- validation lifecycle and interview bank;
- ADR-0018 / 0020 / 0023 / 0024;
- v0.5 audit/adoption decisions;
- `.ai` handoff/current state/active task.

## 2. Product understanding

FirmwareSight is a local-first Embedded Firmware Release Workbench.

Its stable mental model is:

`Analyze → Compare → Gate → Release`

The product is not an ELF viewer, IDE, debugger, flasher, OTA platform, cloud compliance portal or AI release judge.

Its value is evidence depth, repeatability and explicit uncertainty.

## 3. V0 objective

V0 validates:
- comprehension;
- discoverability;
- four-verb workflow;
- evidence mental model;
- Gate mental model;
- trust;
- perceived value;
- repeat/pilot/payment signals.

V0 does not validate parser correctness, Rust/Tauri architecture, SQLite, large-file performance or real Release Bundle I/O.

## 4. V0 / P0 boundary

V0 uses static demo fixtures and a deterministic prototype state machine.

P0 remains unauthorized in this execution:
- no Cargo workspace;
- no Rust crates;
- no `src-tauri`;
- no parser/storage/provenance production implementation.

G1 still requires both V0 and P0 PASS; this execution cannot change that rule.

## 5. Seven-screen workflow understanding

1. Overview — readiness + capability/evidence availability.
2. Analyze — Sections/Symbols/Dependencies + Evidence Inspector.
3. Compare — old/new/delta and contributors.
4. Release Gate — BLOCK/REVIEW/UNKNOWN/PASS/N/A + immutable review acceptance.
5. Bundle & History — self-contained release evidence and historical audit.
6. Parse Failure — evidence-rich recoverable error; Last Good Artifact preserved.
7. Unknown Dependency — Observed/Declared/Unknown distinction and explicit manual declaration.

## 6. MAP state timeline

Canonical V0 state model:

- STATE A: ELF loaded, MAP absent, Git absent. Section facts available; symbols unavailable; MAP-dependent Gate rules UNKNOWN.
- A→B: explicit Add MAP action.
- STATE B: `firmware.map` loaded, 1,284 symbols resolved; dependencies/symbol views enabled; MAP-dependent Gate rules re-evaluated.
- STATE C: invalid candidate ELF replacement fails; Last Good Artifact and project evidence remain intact.
- STATE D: recover to valid workspace and inspect Bundle/History.

Screenshot chips never override this state timeline.

## 7. MVP / candidate boundary

No E1/E2/E3/GX capability is implemented.

Export controls may exist for information-architecture continuity but are deliberately disabled/prototype-only.

## 8. Top risks

1. Users interpret “Can we ship now?” as regulatory/safety approval.
2. UNKNOWN is interpreted as error, PASS or “nothing found”.
3. Review acceptance is interpreted as rewriting REVIEW to PASS.
4. Declared dependency data is interpreted as observed/detected truth.
5. MAP capability transition is not understood, contaminating Analyze/Gate comprehension.

## 9. New authority conflicts

No new unresolved authority conflict was found.

The v1.1 Prompt already resolves the known MAP cross-screen inconsistency and is consistent with v0.5.0 authority.

## 10. V0 workspace

Created under `V0_VALIDATION/`, physically separate from `apps/` and `crates/`.

## 11. Blocking issues

### Prototype/protocol execution
NONE.

### Formal V0 completion
External participant access is required.

No real participant sessions were provided or can be invented by the execution environment.

Formal V0 Gate therefore remains incomplete until minimum N=8 eligible external sessions are recorded.
