---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-09-29"
---

# ACTIVE TASK

```text
NONE — P2 Compare reached PASS / COMPLETE on 2026-09-29. There is no live coding task.

AGENTS.md 1: with no active task, no agent may create business functionality or pick the next stage from
the roadmap. P3 Release Gate is the next *authorizable* stage; it starts only when the architect issues a
P3 prompt. That sentence is a record of where the delivery stands, not permission to begin.
```

## What just closed

`P2_COMPARE` — the second MVP stage of the open-source MVP-first line ADR-0026 opened, authorized by
*FirmwareSight — P2 Compare MVP Implementation, Execution Prompt v1.0 — Architect Reviewed*.

```text
Analyze two builds → select Base / Target → Core diff → [Section | Symbol] changes → export JSON / HTML
```

The acceptance list was the frozen **US-002 Compare Builds** criteria in
`01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:23-31`, all five items, plus the PRD **P0-3 Build Diff** content
list and its rule that a bare `+8 KB` is not an answer.

| Document | Holds |
| --- | --- |
| `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md` | §61 box by box, each settled by a named test, a command or a line of a smoke report; boundary conditions; the verdict |
| `P2_VALIDATION/P2_COMPARE_EXECUTION_REPORT.md` | what was built, every §62 command and its measured output, the §55 CLI smoke, the cross-renderer agreement, the four defects, what is not verified |
| `P2_VALIDATION/P2_COMPARE_DESKTOP_SMOKE_REPORT.md` | the release binary in a real Windows window: 30 steps, the reversal re-checked in the shipped app, native dialogs, overwrite guard, database handled reversibly |
| `P2_VALIDATION/P2_COMPARE_DESIGN_CHECKLIST.md` | `AGENTS.md` 11 / `DESIGN.md` review with the greps and token counts run rather than asserted |

Measured on this machine: `cargo test --workspace` **345 passed / 0 failed / 0 ignored**, UI **99 passed**
in 5 files, `python scripts/check.py` **15/15** (the drift group is 6 steps now, not 5). `fwsight diff`
renders the same portable document the desktop exports — byte-identical, `1930cdb6…` — and its HTML is
byte-identical to `golden/reports/p2-diff.html`. A clean `git archive HEAD` checkout carries all 22
manifest paths with 0 hash mismatch. Baseline stays **v0.6.0**; no migration (`SCHEMA_VERSION` 2,
migrations still 0001 and 0002), no new third-party dependency, no new design token. Two P2 goldens moved
by four lines, and the reason is in the smoke report: the code was wrong, not the gate.

`NOT MEASURED`: peak RSS, and no diff throughput or latency number is claimed anywhere — the 50,000-row
test is a guard against an accidental quadratic rewrite, not a benchmark. `NOT RUN`: fuzzing, a macOS or
Linux window, keyboard-only traversal of the shipped binary. `NOT VERIFIED`: desktop smoke step 27, the
same-pair lock in the shipped window — a native `<select>` popup cannot be driven or captured through the
available window path, so it stays covered only by `compare.test.tsx` and the IPC tests and is recorded as
PARTIAL.

**The remote was red at the mid-round push and is green at the head.** Two P2 commits went up during the
round; run #17 `36596452341` on `c7fc2a3` concluded `completed/failure`, because `.gitignore`'s
`**/target/` had hidden the entire `fixtures/elf/p2-diff/target/` half of the fixture pair — a clean
checkout carried six of the twelve P2 paths the manifest records, while every local run passed on the
bytes the generator had left on disk. That is defect E in
`P2_VALIDATION/P2_COMPARE_EXECUTION_REPORT.md` §5.1: the fixtures are tracked, the path is un-ignored with
its reason in the file, the drift group gained a `fixtures tracked` step, and the fix was re-verified from
`git archive HEAD` rather than from the authoring directory.

The owner then pushed the head, and the remote agrees with what was measured locally: run #18
`36648718199` on `4a77ea1` concluded `completed` / `success`, **7 of 7 jobs** — `Generated output drift`,
`Dependency policy`, `Rust (windows-latest)`, `Rust (ubuntu-latest)`, `Desktop UI (windows-latest)`,
`Desktop UI (ubuntu-latest)`, `macOS Core Smoke` — read with
`gh run view 36648718199 --repo 2023violet/FirmwareSight`. Defect E is therefore closed on the remote, not
only on this machine.

The distinction still holds and is the reason the run is written here rather than in its own commit: the
P2 gate results were measured before any push, so the round's own verdict remains `LOCAL PASS`, and #18 is
the successor verification of the tree that carries it. This commit's push will start a further run; its
number belongs to a later document, not to this one.

## State this closure leaves behind

```text
P0                  PASS — frozen at v0.6.0
G1                  PASS — basis P0 PASS, per ADR-0026 (2026-09-29)
P1-A0               COMPLETE (+ correctness closure COMPLETE)
P1                  PASS / COMPLETE
P2                  PASS / COMPLETE — this task, with step 27 of its desktop smoke left open on purpose
P3                  NEXT_AUTHORIZABLE_STAGE — NOT STARTED, needs its own architect prompt
P4                  NOT STARTED
V0                  NON_BLOCKING_USER_FEEDBACK_TRACK — 0 / 8 honest humans, gates nothing, nothing fabricated
G2                  NOT REACHED
Baseline            v0.6.0 — unchanged by this stage; no v0.7.0
Design tokens       v0.2.1 — unchanged
Public schemas      analysis v1, diff v1 (urn:firmwaresight:schema:diff:1) — both compatibility promises
Pricing/commercial  DEFERRED_POST_MVP
Run #16             36576568426 on 7a13660 — success, 7 of 7 jobs (this round's start fact)
Run #17             36596452341 on c7fc2a3 — FAILURE, 3 of 7 jobs (the mid-round push; defect E)
Run #18             36648718199 on 4a77ea1 — success, 7 of 7 jobs (defect E closed on the remote)
Remote              origin/main at 4a77ea1, green
```

Three things are open on purpose, because closing them needs authority or a decision this round did not
have:

1. **Object / module attribution is Unavailable, not implemented.** PRD P0-3 makes it conditional on
   sufficient evidence, and ELF symbol values carry no per-object size in the facts FirmwareSight stores.
   The stage reports the gap with its reason instead of inventing an Object tab; closing it is a real
   feature decision, not a P2 leftover.
2. **`urn:firmwaresight:schema:diff:1` is now a public compatibility promise** (§59). Nothing in P2 can
   be reinterpreted silently any more; a breaking semantic change needs a major version. That constraint
   belongs to whoever writes P3 and P4 exports, and is recorded in the completion report rather than only
   in the schema file.
3. **Desktop smoke step 27 was never observed in the shipped window**, and the P1 `Apply-filter` dead
   click is still unresolved. Both are recorded as `NOT VERIFIED` / `UNRESOLVED` with what does cover them,
   rather than counted as passes.

Two carry-overs from P1 are unchanged and still open: `sections.file_offset` and `symbols.address` are
nullable columns with no reason column, so a reason is lost at write time (a migration fix), and
`drift/ipc bindings unchanged` cannot see a binding file that is not yet tracked.

## If a P3 prompt arrives

Read first, in order: root `README.md`, `.ai/README.md`, `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`, this
file, `04_TECH/09_TECH_DECISION_MATRIX.md`, and the ADR or prompt P3 names. `AGENTS.md` 2 / 3 / 6 / 7 / 8 /
11 still bind unchanged: Core stays headless, synchronous and `forbid(unsafe_code)`; the technology
baseline does not move silently; SQLite stays behind `rusqlite + bundled` with every schema change going
through a migration; the WebView keeps no general shell or filesystem power — it now has six Compare
commands and no more; every fact keeps its evidence class; and UI work goes through `DESIGN.md` plus the
frozen tokens.

P3 has one input P2 did not have: the Gate must read the same persisted snapshots Compare reads, and its
verdict is a five-state answer (PASS / REVIEW / BLOCK / UNKNOWN / N/A), not a diff vocabulary. Added,
Removed and Changed are not Gate states, and P2's own screens must not be reused to imply that they are.
