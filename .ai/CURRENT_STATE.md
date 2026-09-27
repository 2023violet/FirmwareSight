---
title: "Current Project State"
doc_id: "FS-AI-002"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# Current State

Date: 2026-09-28  
Baseline: v0.5.1

## Product/architecture baseline

v0.5.0 product/architecture/design decisions remain inherited. v0.5.1 is the active baseline.
The P0 slice created the first production code; nothing in the baseline was renegotiated to do it.

## P0 — Technical Vertical Slice

Status: `EXECUTED — CONDITIONAL_PASS (LOCAL)`

The source tree exists and the slice's claim is proven by executed tests: 102 Rust tests, 19 UI
tests, one shared gate script, real ELF/MAP fixtures with recorded provenance, deterministic CLI
JSON, memory accounting reproduced by hand from `readelf`, a typed IPC boundary with generated
TypeScript, SQLite migrations and transactional import, and a 512 MiB guard measured from both
sides of the boundary.

Three conditions stay open, and all three are authorizations rather than engineering unknowns:

| # | Condition | Blocked by |
| --- | --- | --- |
| C1 | The CI workflow has never executed | no push authorization in this task |
| C2 | `cargo deny` has never run | installing the tool was not authorized |
| C3 | The desktop window has never been opened | starting a GUI in the user's session was not authorized |

Full reasoning: `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md`; per-item evidence:
`P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`.

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

The V0 blocker remains real human participants. Deferred does not mean passed, and P0 progress
does not close this gap. P0 changed no V0 artifact and no V0 recommendation.

## Gates

```text
G0: PASS
V0: DEFERRED / UNVALIDATED (0 of 8 eligible external sessions)
P0: EXECUTED — CONDITIONAL_PASS (LOCAL); C1 CI run, C2 cargo-deny run, C3 desktop launch open
Formal G1: NOT CLAIMED (requires V0_PASS and P0_PASS)
P1: NOT AUTHORIZED
```

## Version rule

`v0.6.0` is not created unless P0 reaches `PASS` with real engineering evidence. P0's final status
is `CONDITIONAL_PASS`, so `baseline_version` stays `0.5.1`. A conditional result must not be
relabeled into a PASS baseline.

## Next work

Nothing may be started without authorization. The three open conditions are closed by actions, not
by edits:

1. authorize a push so `p0-check.yml` runs on `main`;
2. let the `deny` job execute `cargo-deny@0.20.2`;
3. launch `cargo run --release -p firmwaresight-desktop --features custom-protocol` once, after
   `pnpm -C apps/desktop/ui build`, and confirm the summary screen. Without that feature the binary
   is a dev-mode build that expects the Vite dev server, so the launch would prove nothing about the
   shipped configuration.

If all three come back clean, P0 may be re-statused to `PASS` and v0.6.0 delivered. If any fails,
the status becomes `FAIL` or `BLOCKED` and the baseline stays at v0.5.1.

V0 resumes only when a real eligible participant/session source exists.
