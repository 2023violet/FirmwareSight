---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.5.1"
status: "ACTIVE"
owner: "Engineering"
last_updated: "2026-09-27"
---

# ACTIVE TASK

## P0 — Technical Vertical Slice

Status:

# ACTIVE — TECHNICAL VERTICAL SLICE

Authorized by `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P0_Technical_Vertical_Slice_EXECUTION_PROMPT_v1.1_ARCHITECT_REVIEWED.txt`
(SHA-256 `60a59196708a53592be4d828a0c1275cf74b3bfad0bdc6455f863ab38d32929b`), consistent with
`09_ADR/ADR-0020-validation-sequence.md`.

Goal: prove the core technical skeleton, and prove the same real Core facts reach CLI and Desktop.

In scope: Rust workspace, the four Phase-0 library crates, `fwsight` CLI, minimal Tauri 2 shell,
minimal React/TypeScript summary, real ELF fixtures, golden tests, deterministic JSON,
memory accounting proven on fixtures, typed IPC via ts-rs, minimal SQLite foundation,
large-file guard, CI.

## V0

Status:

`DEFERRED / NOT YET EVIDENCE-VALIDATED — NON-BLOCKING RESEARCH TRACK`

Durable fact, not deleted by this change:

Formal eligible external participants completed `0 / 8 minimum`; Batch A target `0 / 4–5`.
The V0 blocker is the absence of real human participants, not a technical failure.
`V0_VALIDATION/` and every Batch A recruitment artifact remain intact and unmodified.

## Gate status

```text
Formal G1: NOT CLAIMED  (g1_requires V0_PASS + P0_PASS; V0 still unvalidated)
P1:        NOT AUTHORIZED
```

## Version gate

Do not generate `FirmwareSight_Project_Baseline_v0.6.0` unless P0 final status is `PASS`
with real engineering evidence. `CONDITIONAL_PASS`, `FAIL` and `BLOCKED` must not be
renamed into a PASS baseline.

## Not authorized

- P1–P4 Product MVP implementation
- E1 / E2 / E3 / GX candidates
- Cloud / account / auth / telemetry / AI
- SBOM / CVE / OTA / flashing / HIL / updater / wgpu / SQLx
- A fifth Phase-0 library crate without architecture review plus ADR
- Synthetic Persona substitution for V0 evidence
- Claiming V0 PASS, G1 PASS, or that product value has been user-validated
