---
title: "Project Handoff"
doc_id: "FS-AI-004"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-28"
---

# Handoff — FirmwareSight v0.5.1 / P0 executed, CONDITIONAL_PASS (LOCAL)

## Purpose

Execute the P0 Technical Vertical Slice to its stop condition, then stop. The slice has reached that
stop condition; nothing further in P0 is open except the two items listed below.

## Read first

1. README.md
2. PRODUCT_BASELINE.md
3. BASELINE.yaml
4. AGENTS.md
5. DESIGN.md + assets/design-tokens.json (for the Desktop UI step)
6. `10_AUDIT/SOURCE_PROMPTS/README.md` — which prompt authorizes what
7. `P0_TECHNICAL_VALIDATION/P0_EXECUTION_PROVENANCE.md` — start HEADs and environment
8. `P0_TECHNICAL_VALIDATION/P0_IMPLEMENTATION_LOG.md` — decisions already taken
9. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md` — what is proven, and how
10. `P0_TECHNICAL_VALIDATION/P0_DESIGN_CHECKLIST.md` — the AGENTS.md 11 review for the UI, with four open findings
11. `.ai/ACTIVE_TASK.md`

V0 status summary only (do not re-run V0 from this handoff):

- `V0_VALIDATION/README.md`
- `V0_VALIDATION/V0_EXECUTION_PROVENANCE.md`
- `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`

## Current state

P0 final status: **`EXECUTED — CONDITIONAL_PASS (LOCAL)`**. Baseline stays at `0.5.1`; `v0.6.0` is
reserved for an unconditional `PASS`, and `P0_EXIT_CHECKLIST.md` names the conditions.

What is proven locally: 102 Rust tests and 19 UI tests, a single gate
(`python scripts/check.py`) that CI calls unchanged - 14 steps on a tree that already has the built
frontend, 16 when it has to build that too, and 16/16 green from `cargo clean` - real ARM ELF/MAP
fixtures with recorded provenance, deterministic CLI JSON, memory accounting reproduced by hand from
`readelf`, a typed ts-rs IPC boundary, SQLite migrations with transactional import, a 512 MiB guard
measured from both sides of the boundary, and Core/CLI/Desktop parity on the same bytes.

What is not proven, and why nobody here could prove it:

| Open | Closes when |
| --- | --- |
| CI has never run | someone with push authorization runs `.github/workflows/p0-check.yml` |
| `cargo deny` never ran | the same run, or `cargo install cargo-deny@0.20.2 --locked` locally |
| Peak RSS not measured | a measurement tool is authorized; see `P0_PERFORMANCE_REPORT.md` |
| The window was never opened | a human runs the desktop app on a real display |
| Two design-checklist findings are open: capability labels show Core's enum words, and eight `1px` borders have no token | the first is a boundary decision about who owns user-facing wording; the second needs a `design-tokens.json` version bump, a frozen-asset change P0 may not make. Two others the same pass found - no live region, and a `select` with only hover and focus - were fixed and are covered by a test. `P0_DESIGN_CHECKLIST.md` records all four with the commands that found them |

V0:

`DEFERRED / NOT YET EVIDENCE-VALIDATED` — formal external sessions `0 / 8 minimum`.
The prototype, protocol and internal functional dry run are complete; the missing input is
real participants. Deferral is not completion.

## Boundaries

- Four Phase-0 library crates only; a fifth requires architecture review plus an ADR.
- Core stays headless and synchronous: no Tauri, rusqlite, Tokio types or `object::*` leakage.
- No Compare/Gate/Bundle product workflow, no cloud/auth/telemetry/updater/wgpu/SQLx, no E1/E2/E3/GX.
- `v0.6.0` only on a real P0 `PASS`.
- P1 requires a separate authorization prompt. Do not enter it.

## Continuation rules

- Re-run the read-only Git preflight before writing, and report start HEADs again; do not trust
  the HEAD recorded in this file.
- Never clean, reset, stash or restore to get a tidy tree. User work outranks tree cleanliness.
- Reports cite real commands and real output. Unmeasured stays `NOT MEASURED` with a reason.
- Do not weaken or delete a test to reach green.
- Do not edit V0 evidence, and do not turn the V0 gate recommendation into a PASS.
- Measure before describing a build. Two claims in this pack were false until re-measured: the
  fixture `fixture.toml` linker invocation, and the desktop binary "embedding the built UI"
  (it did not, without `custom-protocol`). Write what the command printed, not what it should print.
