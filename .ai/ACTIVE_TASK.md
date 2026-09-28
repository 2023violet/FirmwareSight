---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-09-28"
---

# ACTIVE TASK

```text
NONE
```

P0 Technical Vertical Slice is closed `PASS` and promoted to the
`FirmwareSight_Project_Baseline_v0.6.0` P0 Technical Foundation Baseline by the architect-signed
*P0 Final Promotion / v0.6.0 Baseline Closure v1.0* prompt. There is no active engineering task, and
no work may be started from this file.

## What is true now

```text
Baseline:      v0.6.0 — P0 Technical Foundation Baseline
G0:            PASS
V0:            DEFERRED / NOT YET EVIDENCE-VALIDATED — 0 / 8 eligible external sessions
P0:            PASS — remote CI Run #3 36399805005 (7/7) and Run #4 36402637251 (7/7)
Formal G1:     NOT CLAIMED — g1_requires V0_PASS + P0_PASS; V0 is the missing half
P1:            NOT AUTHORIZED
```

What P0 passed with: 104 Rust tests, 19 UI tests, one shared gate (`python scripts/check.py`) that CI
calls unchanged, real ARM ELF/MAP fixtures with recorded provenance, deterministic CLI JSON, a typed
ts-rs IPC boundary, SQLite migrations at schema version 2, a 512 MiB guard measured from both sides of
the boundary, and a desktop window opened and driven on the shipping configuration.

## What promotion did not do

`v0.6.0` names the validated *technical foundation* only. It does not mean V0 passed, G1 passed, the
product MVP is complete, P1 is authorized, Compare/Gate/Bundle exist, an installer or signed build is
ready, user value is validated, or the tree is free of vulnerabilities. The durable gaps survive the
signature unchanged:

```text
Peak RSS:      NOT MEASURED — measurement gap the P0 prompt accepted with a stated reason
Fuzz:          NOT RUN
RustSec:       RUSTSEC-2024-0429 (glib 0.18.5, unsound) and RUSTSEC-2024-0370
               (proc-macro-error 1.0.4, unmaintained) — ACCEPTED AS EXPLICIT P0 TRANSITIVE RISK,
               not resolved, not suppressed silently; five revisit triggers
V0:            0 / 8 formal external sessions
Two design-checklist findings remain open: capability labels show Core's enum words, and eight 1px
borders have no token, which needs a frozen-asset design-tokens.json bump.
```

## If you are resuming this repository

Read `README.md`, `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`, `BASELINE.yaml` and `.ai/HANDOFF.md`
before writing anything, in the AGENTS.md §1 order. Then:

- The only legitimate next moves are a real V0 participant source resuming the research track, or an
  explicit architect-issued P1 sequencing prompt. Neither exists.
- `ACTIVE_TASK: NONE` means do not invent work. If asked to continue, report the empty task and the
  unauthorized items above rather than picking a task.
- Nothing in `V0_VALIDATION/` may be edited to make the deferred track look validated.

## Technical items that remain unauthorized, not deferred-by-accident

```text
P1 slice                       needs its own authorization prompt
installer / NSIS / MSI         outside the P0 baseline promotion
signing / notarization         requires an ADR before any key work
updater                        requires the signing/key-management ADR (AGENTS.md §7)
fuzzing                        not run; not part of the P0 gate
peak RSS on large workloads    not measured; see P0_PERFORMANCE_REPORT.md
dependency-architecture ADR    not needed for the two accepted advisories; needed before changing
                               the Tauri/gtk-rs line
```
