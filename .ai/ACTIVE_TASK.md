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
P1A0_REAL_ARTIFACT_INTAKE  — authorized as a bounded pre-G1 slice by ADR-0025.
                              It is NOT P1, it does not close P1, and it stops at P1-A0.
```

Authorization: *FirmwareSight — Pre-G1 Sequencing Revision + P1-A0 Real Artifact Intake, Execution
Prompt v1.0 — Architect Reviewed*, with the *P1-A0 Design Contract Closure Addendum v1.0* applied first
(it authorized design-tokens v0.2.1 and nothing else). Both were supplied inline, so
`10_AUDIT/SOURCE_PROMPTS/README.md` records them without a SHA-256 rather than inventing one.

## The user outcome

A person can choose a real ELF file from their own machine through a native dialog, optionally attach a
GNU ld MAP, and have FirmwareSight analyze it with the pipeline P0 already validated. Today the desktop
can only analyze two committed fixtures, which is the limitation this slice removes.

```text
Launch → Analyze → choose firmware artifact → (optional) Add MAP → Analyze → real summary
```

## Hard boundaries of this task

- **One slice.** P1-A1, P2 Compare, P3 Gate and P4 Bundle are not authorized. Completing P1-A0 is the
  stop condition, not a licence to continue.
- **`P1` is not PASS and not closed.** `G1 = V0 PASS + P0 PASS` is unchanged and G1 stays NOT CLAIMED.
- **No new baseline.** `baseline_version` stays `0.6.0`; `v0.7.0` is not created by a development slice.
- **Security shape is fixed.** Native dialog, explicit user selection, opaque session-local
  `SelectionId` held on the Rust side, no generic filesystem/shell/network permission, no
  `read_file(path)` or `get_any_path` command, no full path in normal IPC or UI. No new `rand`
  dependency for the selection id.
- **No schema change.** Real artifacts use an explicit local/default project identity instead of the
  P0 demo one; schema version 2 keeps working, migrations are not authorized, raw bytes are not copied
  into SQLite.
- **Fixtures stay engineering evidence.** `fixtures/**` and the P0 parity/golden tests remain green;
  the product UI stops offering a fixture selector.
- **Design contract.** Existing tokens plus the new `border.width.hairline` only: no drop-zone, no
  dashed border, no decorative upload card, no new visual semantic. If one is genuinely needed, stop.
- **A real Windows desktop smoke with a really opened native dialog is part of the definition of
  done.** Without it, P1-A0 is not complete.

## State this task runs against

```text
P0                  PASS — frozen at v0.6.0; no further P0 closure or promotion prompt will be written
Baseline            v0.6.0
Design tokens       v0.2.1 (border.width.hairline added by the addendum)
V0                  ACTIVE EXTERNAL VALIDATION / WAITING FOR REAL PARTICIPANTS — 0 / 8, Batch A 0 / 4-5
Formal G1           NOT CLAIMED
Run #6              36419864513 on 7d2f38a — success, 7 of 7 jobs (the fact before this task started)
Peak RSS            NOT MEASURED      Fuzz: NOT RUN
RustSec             two accepted transitive advisories, unchanged by this slice
```

## What the coding side may not do here

Fabricate V0 evidence of any kind — no synthetic transcripts, sessions or metrics. The governance
wording may move to "resumed, waiting for real participants", and `external_participants_completed`
stays `0`. `V0_VALIDATION/**` is not edited.

Do not treat a green P1-A0 as P1 progress: `06_DELIVERY/06_STAGE_GATES.md` registers `P1-A0` as
neither a stage gate nor a step toward closing P1.

## Next gate after this task

The next engineering authorization requires **V0 Batch A ≥ 4 eligible external sessions with real
participants, an interim architect review of that evidence, and a new architect prompt**. Until then the
correct state of this file after P1-A0 ships is `NONE`, not an inferred follow-on task.
