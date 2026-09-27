---
title: "V0 Execution Provenance"
doc_id: "FS-V0-000"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Audit / Product"
last_updated: "2026-09-27"
---

# V0 Execution Provenance

## Unique baseline

`FirmwareSight_Project_Baseline_v0.5.0`

ZIP SHA-256:
`986b817f7266828e6d54daa2028539129691d13fb5b1f6227e8b77a475e3e6b0`

## Authorized execution prompt

`FirmwareSight_V0_Workflow_Prototype_Validation_PROMPT_v1.1.txt`

SHA-256:
`3431a688c65c08aa6104fe4cb785ef25b6ae9df4e2d3272dddda653704e1df20`

## Authorization interpretation

The user explicitly authorized V0 execution through the prompt.

Authorized:
- V0 clickable prototype;
- fixture/state-machine data;
- validation protocol;
- internal functional dry run;
- session/analysis/report templates;
- V0 execution records.

Not authorized:
- P0;
- Rust/Cargo/Tauri production implementation;
- P1–P4 Product MVP implementation;
- E1/E2/E3/GX features.

## Evidence limitation

No external participant pool or recorded external sessions were supplied to this execution environment.

Therefore:
- no participant behavior is fabricated;
- no usability/commercial numerator/denominator is invented;
- no V0 PASS is declared;
- V0 remains `INCOMPLETE — insufficient external sample` until at least 8 eligible external participants complete formal sessions.
