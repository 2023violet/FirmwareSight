---
title: "Current Project State"
doc_id: "FS-AI-002"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-27"
---

# Current State

Date: 2026-09-27  
Baseline: v0.5.1

## Product/architecture baseline

v0.5.0 product/architecture/design decisions remain inherited.
v0.5.1 is the active baseline. No production code existed before the P0 slice began.

## P0 — Technical Vertical Slice

Status: `ACTIVE — TECHNICAL VERTICAL SLICE`

Authorized by the archived P0 execution prompt (see `10_AUDIT/SOURCE_PROMPTS/README.md`) and
consistent with `ADR-0020`, which authorizes V0 and P0 in parallel.

Target: first real source tree — Rust workspace, four Phase-0 library crates, `fwsight` CLI,
minimal Tauri 2 shell plus React summary, real ELF fixtures, goldens, typed IPC,
minimal SQLite, large-file guard, CI, and `P0_TECHNICAL_VALIDATION/` reports.

Final P0 status is not yet determined. Per the authorized taxonomy the only permitted
outcomes are `PASS`, `CONDITIONAL_PASS`, `FAIL`, `BLOCKED`.

## V0

Status:

`DEFERRED / NOT YET EVIDENCE-VALIDATED — NON-BLOCKING RESEARCH TRACK`

This is a re-sequencing of the research track, not a completion claim.

Completed:
- clickable prototype;
- fixture/state machine;
- protocol/session templates;
- internal functional dry run;
- Batch A takeover;
- Batch A recruitment-ready package.

Formal external sessions:
`0 / 4–5 Batch A target`
`0 / 8 V0 minimum`

The V0 blocker remains real human participants. Deferred does not mean passed, and P0
progress does not close this gap.

## Gates

```text
G0: PASS
V0: DEFERRED / UNVALIDATED (0 of 8 eligible external sessions)
P0: ACTIVE — in progress, status undetermined
Formal G1: NOT CLAIMED (requires V0_PASS and P0_PASS)
P1: NOT AUTHORIZED
```

## Version rule

`v0.6.0` is not created unless P0 reaches `PASS` with real engineering evidence.
`baseline_version` stays `0.5.1` until then.

## Next work

Execute the P0 slice to its stop condition, then stop for a separate P1 authorization.
V0 resumes only when a real eligible participant/session source exists.
