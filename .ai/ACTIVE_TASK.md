---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-09-30"
---

# ACTIVE TASK

```text
NONE — P3 Release Gate reached PASS / COMPLETE on 2026-09-30. There is no live coding task.

AGENTS.md 1: with no active task, no agent may create business functionality or pick the next stage from
the roadmap. P4 Release Bundle is the next *authorizable* stage; it starts only when the architect issues
a P4 prompt. That sentence is a record of where the delivery stands, not permission to begin.
```

## What just closed

`P3_RELEASE_GATE` — the third MVP stage of the open-source MVP-first line ADR-0026 opened, authorized by
*FirmwareSight — P3 Release Gate MVP Implementation, Execution Prompt v1.1 — Architect Reviewed*
(supplied inline, registered without a SHA-256 because no source file reached this repository) and by
`ADR-0027`, which keeps project policy and Git provenance out of Core.

```text
firmwaresight.toml → project / Git evidence → GatePolicy → Core Gate
→ PASS / REVIEW / BLOCK / UNKNOWN / N/A → immutable GateRun
→ immutable Review acceptance → CLI gate → Desktop Release
```

The acceptance list was the frozen **US-003 Prepare Release** criteria in
`01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:33-40`, all four items, plus the PRD **P0-5 Release Gate** ten
built-in checks at `01_PRODUCT/01_PRD_MVP.md:66-77`.

| Document | Holds |
| --- | --- |
| `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md` | §69's forty boxes, each settled by a `gh` query, a command, a named test or a smoke step; what the checklist does not cover; the verdict |
| `P3_VALIDATION/P3_GATE_EXECUTION_REPORT.md` | what was built, every §66 command and its measured output, the §62 CLI smoke, the three defects, the carried-forward limits, the license gap |
| `P3_VALIDATION/P3_GATE_DESKTOP_SMOKE_REPORT.md` | the release binary in a real Windows window: all 40 §61 steps, the CLI/desktop run-id parity measurement, native dialogs, database handled reversibly |
| `P3_VALIDATION/P3_GATE_DESIGN_CHECKLIST.md` | `AGENTS.md` 11 / `DESIGN.md` review with the greps and token counts run rather than asserted |

Measured on this machine: `cargo test --workspace` **715 passed / 0 failed / 0 ignored across 42 suites**,
UI **135 passed** in 6 files, `python scripts/check.py` **15/15**, `cargo fmt --all -- --check` and
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean, `git diff --check` clean,
`drift` 6/6, `deny/cargo-deny` executed for real, `core-smoke` 3/3. The CLI and the desktop produced the
same run id — `gate-a64631b287fcb33478e23ac6462dacc80e1a39cac1e321e047d02bb50ff152cc` — for one project,
one policy and one HEAD. `fwsight gate` exits 0 / 4 / 5 as the policy decides. Baseline stays **v0.6.0**;
`SCHEMA_VERSION` moved 2 → 3 through additive `0003_gate_history.sql` with v1→v3 and v2→v3 upgrade tests;
no new third-party package entered the graph beyond what ADR-0027 admitted; no new design token.

`NOT MEASURED`: peak RSS, and no Gate throughput or latency number is claimed anywhere. `NOT RUN`:
fuzzing, a macOS or Linux window, keyboard-only traversal of the shipped binary, and any CI run over P3's
own commits. `NOT VERIFIED`: desktop smoke step 30's prior-run re-read *through the window* — the Release
page has no run-id input, so `get_gate_run` is tested but was not observed reopening an older record; and
the gate's own first pass was **13/14** — `frontend/test` failed on a `compare.test.tsx` row query racing
the table's own `Loading symbol changes…` state (defect I, a test-side race in P2's file, now fixed with
the assertions unchanged and the gate re-run green). That is recorded rather than smoothed over.

**P3's code is not on the remote.** `origin/main` is still `32b23aa`, whose Run #19 `36665007523` is
`completed / success` with 7 of 7 jobs, following Run #18 `36648718199` on the P2 implementation tree
`4a77ea1`. The five P3 commits sit locally on top of it, so every number above is labelled as locally
measured and no CI result is claimed for them. Pushing is the owner's act, and §67 forbids writing a
future CI run into the commit that would trigger it.

Three product defects were found by this round's own validation and fixed in product code: a version
pattern embedded in an evidence locator made a Gate run unpersistable for any project using the only MVP
version source; one build produced two run ids across the two surfaces; and a disabled primary button kept
its accent border. Each has a regression test named in the closure entry of `.ai/DECISIONS.md`.

The license gap deliberately did not close with the stage: `license = "Proprietary"` stands at
`Cargo.toml:17`, there is no root `LICENSE`, and `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`
remains. `AGENTS.md` 9 puts that decision in front of a human.
