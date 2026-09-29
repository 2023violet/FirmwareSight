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
NONE — P1 Analyze reached PASS / COMPLETE on 2026-09-29. There is no live coding task.

AGENTS.md 1: with no active task, no agent may create business functionality or pick the next stage from
the roadmap. P2 Compare is the next *authorizable* stage; it starts only when the architect issues a P2
prompt. That sentence is a record of where the delivery stands, not permission to begin.
```

## What just closed

`P1_ANALYZE_DETAILS` — capabilities 5 to 9 of the Analyze verb, authorized as the first MVP stage under
ADR-0026 by the parent prompt *FirmwareSight Open-Source MVP-First Governance Reset + P1 Analyze
Completion v1.0* and its *P1 Analyze Acceptance Closure v1.0* addendum.

```text
intake → summary → top contributors → [Sections | Symbols | Evidence] → Evidence Inspector
```

The acceptance list was the frozen **US-001 Analyze ELF** criteria in
`01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md`, not a subset of them. Every mandatory item is green, with
the evidence named per item:

| Document | Holds |
| --- | --- |
| `P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md` | US-001 item by item, hard boundary by boundary, the verdict |
| `P1_VALIDATION/P1_ANALYZE_EXECUTION_REPORT.md` | what was built, the seven defects found and fixed, the four findings reported rather than fixed, every gate command and its output |
| `P1_VALIDATION/P1_ANALYZE_DETAILS_SMOKE_REPORT.md` | the shipped release binary in a real window at the frozen 1024 px minimum, native dialogs, KiB arithmetic checked independently, the failed-analysis last-good path, a clean close |
| `P1_VALIDATION/P1_ANALYZE_DESIGN_CHECKLIST.md` | `AGENTS.md` 11 / `DESIGN.md` review, with the greps run rather than asserted |

Measured on this machine: `cargo test --workspace` **184 passed / 0 failed / 0 ignored**, UI **58 passed**
with no stderr, `python scripts/check.py` **14/14**. Baseline stays **v0.6.0**; no migration
(`SCHEMA_VERSION` 2), no new dependency, no new design token, no golden changed.

`NOT MEASURED`: peak RSS. `NOT RUN`: fuzzing, a macOS or Linux window, keyboard-only traversal of the
shipped binary. One dead `Apply filter` click stays `UNRESOLVED` and is reported as such.

Remote CI ran on the pushed head and is measured: run #13 `36556735551` on `e63afaf`, `completed`,
`success`, 7 of 7 jobs, read with `gh run view 36556735551 --repo 2023violet/FirmwareSight`. It is
recorded by `e63afaf`'s successor commit rather than inside `e63afaf`, per the rule that held through P0
and P1-A0, and the three earlier commits of the same push have no run each.

## State this closure leaves behind

```text
P0                  PASS — frozen at v0.6.0
G1                  PASS — basis P0 PASS, per ADR-0026 (2026-09-29)
P1-A0               COMPLETE (+ correctness closure COMPLETE)
P1                  PASS / COMPLETE — this task
V0                  NON_BLOCKING_USER_FEEDBACK_TRACK — 0 / 8, gates nothing, nothing fabricated
P2 / P3 / P4        NOT STARTED — each still needs its own architect prompt
G2                  NOT REACHED
Baseline            v0.6.0 — unchanged by this slice; no v0.7.0
Design tokens       v0.2.1 — unchanged
Pricing/commercial  DEFERRED_POST_MVP
Run #12             36520562718 on be09c65 — success, 7 of 7 jobs (this round's start fact)
```

Two findings are open on purpose, because closing them needs authority this round did not have:
`sections.file_offset` and `symbols.address` are nullable columns with no reason column, so a reason is
lost at write time (fixing that is a migration); and the `drift/ipc bindings unchanged` gate cannot see a
binding file that is not yet tracked, which is why the regeneration was verified by hash instead. Both
are in `P1_ANALYZE_EXECUTION_REPORT.md` §6 and in `.ai/CURRENT_STATE.md`'s known-gaps list.

## If a P2 prompt arrives

Read first, in order: root `README.md`, `.ai/README.md`, `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`, this
file, `04_TECH/09_TECH_DECISION_MATRIX.md`, and the ADRs the prompt names. `AGENTS.md` 2 / 3 / 6 / 7 / 8 /
11 are unchanged and still bind: Core stays headless and synchronous, the technology baseline does not
move silently, SQLite stays behind `rusqlite + bundled` with every schema change going through a
migration, the WebView keeps no general shell or filesystem power, every fact keeps its evidence class,
and UI work goes through `DESIGN.md` plus the frozen tokens.
