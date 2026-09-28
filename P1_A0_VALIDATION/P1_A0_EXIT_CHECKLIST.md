---
title: "P1-A0 Exit Checklist"
doc_id: "FS-P1A0-004"
product: "FirmwareSight"
version: "0.1.0"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P1-A0 Exit Checklist

Every box from prompt §29, with the evidence that closed it. `LOCAL PASS` means it ran on this
machine and produced the quoted result; nothing here says `CI PASS`, because no push was made.

The slice is bounded: closing these boxes completes **P1-A0**, not P1, and claims nothing about G1.

## Governance state before any code (prompt §5, §6, §25)

| Box | Status | Evidence |
| --- | --- | --- |
| Run #6 verified 7/7 | LOCAL PASS (read from the runner, before this task) | `gh run view 36419864513 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs` → `success` on head `7d2f38a`, 7 of 7 jobs. Recorded in `BASELINE.yaml` `last_remote_ci`; Run #5 is kept beside it so the promotion-commit measurement stays visible |
| ADR-0025 Accepted | LOCAL PASS | `09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md`, `status: "BASELINE"` like its five siblings, superseding only ADR-0020's clause `P1 Product MVP implementation 只有两者都 PASS 后开始`; the architect-reviewed prompt is the acceptance act and both prompt paths are recorded under `pre_g1_execution` |
| Bounded Pre-G1 sequencing documented | LOCAL PASS | `BASELINE.yaml` `pre_g1_execution` (`identifier: P1_A0`, `is_stage_gate: false`, `closes_p1: false`, `stop_condition: P1_A0_ONLY`) plus `06_DELIVERY/06_STAGE_GATES.md` and `08_MILESTONE_DELIVERABLE_MATRIX.md` |
| P0 PASS unchanged | LOCAL PASS | `BASELINE.yaml` `p0.status: PASS` untouched; the P0 pack keeps its own records, including the failed runs |
| V0 still 0/8 | LOCAL PASS | `v0_track.external_participants_completed: 0`, `status: INCOMPLETE_INSUFFICIENT_EXTERNAL_SAMPLE`. No transcript, session or metric was manufactured for this slice (§23) |
| G1 NOT CLAIMED | LOCAL PASS | `g1_claimed: false` in both places that carry it; `validation.current_gate: G0_PASS_V0_ACTIVE_WAITING_PARTICIPANTS_P0_PASS_G1_NOT_CLAIMED` |
| P1-A0 authorized / completed | LOCAL PASS | authorized by `311f9fc governance: authorize bounded pre-G1 P1-A0`; completed by the implementation commit this document ships in |
| P1 NOT PASS / NOT CLOSED | LOCAL PASS | `pre_g1_execution.p1_stage_status: NOT_PASS_NOT_CLOSED` |
| P2 / P3 / P4 unauthorized | LOCAL PASS | `p1_a1/p2/p3/p4` all false in `pre_g1_execution`; the UI test `offers no way into a stage this build does not have` asserts the same boundary on screen |
| No v0.7.0 | LOCAL PASS | `creates_new_baseline: false`, `baseline_version_stays: 0.6.0`; `git tag` still has no v0.7.0 |
| Governance assertions re-checked | LOCAL PASS | `python %LOCALAPPDATA%\Temp\governance_assert.py` → 42/42 assertions on the authorized state; `BASELINE.yaml` reloaded through a duplicate-key-rejecting parser → 0 repeated keys |

## Behaviour and boundary (prompt §8, §12, §15, §16)

| Box | Status | Evidence |
| --- | --- | --- |
| Native artifact dialog works | LOCAL PASS | window 2230426 / 269576 titled `Choose the firmware artifact to analyze`, same pid as the app - see the smoke report |
| Native MAP dialog works | LOCAL PASS | window 592502 titled `Choose the linker MAP for this artifact` |
| No generic fs / shell / network capability | LOCAL PASS | `capabilities/main.json` is still exactly `["core:default"]`; no `fs` or `shell` plugin in `Cargo.toml` |
| Full path not exposed in normal UI/IPC | LOCAL PASS | `no_selection_or_summary_payload_carries_a_path`, `never renders the directory the artifact came from`, and the smoke's `Why we know` line showing only `not-a-firmware.elf` |
| Opaque selection id | LOCAL PASS | `sel-<pid>-<n>`, session-local, never persisted, never in the snapshot identity; `a_selected_artifact_yields_an_opaque_id_and_a_file_name_only`, `selection_ids_are_unique_within_a_session` |
| Real ELF analyzed through the existing pipeline | LOCAL PASS | `a_user_chosen_file_produces_the_same_core_facts_the_cli_reports` + the smoke's field-by-field CLI table |
| MAP strengthens evidence | LOCAL PASS | `attaching_a_map_strrengthens_the_evidence_and_detach_takes_it_back`; in the window: not admissible → admissible, `layout_source` none → map, `map` capability not-provided → provided, evidence 10 → 11 |
| Last-good preserved | LOCAL PASS | §15 semantics asserted by three UI tests (`keeps the last good analysis visible…`, `never presents the last good analysis as the failed candidate`, `replaces the last good analysis when a new candidate succeeds`) and seen in the window at step 16 |
| Cancel non-error | LOCAL PASS | `a_cancelled_dialog_is_not_an_error`, `treats a cancelled artifact dialog as a normal outcome, not an error`, `treats a cancelled MAP dialog as a normal outcome`; step 13 in the window |
| Malformed input typed, no panic | LOCAL PASS | `ERR-FORMAT-0001` in the window with diagnostics id `op-609c-18d98be1034ea388`; `format_truth_comes_from_the_bytes_not_from_an_elf_extension`, `an_artifact_deleted_after_selection_is_a_typed_error_not_a_panic`, `an_unknown_selection_id_is_a_typed_error_and_not_a_panic` |
| No DB migration | LOCAL PASS | `the_selection_path_adds_no_schema_migration`; the live database still lists `schema_migrations` = (1, 2) and `SCHEMA_VERSION` is 2 |

## Engineering gates (prompt §19, §20, §21, §22, §26)

| Box | Status | Evidence |
| --- | --- | --- |
| Rust tests PASS | LOCAL PASS | `cargo test --workspace` → 123 passed / 0 failed, including the 18 new `real_artifact_intake.rs` tests (≥10 required) |
| UI tests PASS | LOCAL PASS | `corepack pnpm test` → 31 passed in 3 files: 20 in `intake.test.tsx` (≥12 required), 8 in `bridge.test.ts`, 3 in `format.test.ts` |
| drift PASS | LOCAL PASS | `python scripts/check.py --only drift` → 5/5, including `ipc bindings unchanged` and `goldens unchanged` |
| deny PASS | LOCAL PASS | `python scripts/check.py --only deny` → 1/1; cargo-deny is installed here, so this is a real execution, not the SKIPPED path. `advisories bans licenses sources` all 0 errors |
| core smoke PASS | LOCAL PASS | `python scripts/check.py --only core-smoke` → 3/3 |
| Full gate PASS | LOCAL PASS | `python scripts/check.py` → **14/14 steps passed**, exit 0, re-run after the last CSS and test change |
| Real desktop smoke PASS | LOCAL PASS | `P1_A0_DESKTOP_SMOKE_REPORT.md` - 17/17 against the shipping binary this commit ships, rebuilt after that change |
| Design checklist PASS | LOCAL PASS | `P1_A0_DESIGN_CHECKLIST.md`; every Don't ticked, no new token required, so §21's STOP condition was never reached |
| `git diff --check` clean | LOCAL PASS | exit 0. For completeness: `git diff --cached --check` flags exactly one trailing space, on line 15 of the ts-rs-generated `AnalysisSummaryDto.ts`, which the generator emits once a field carries a doc comment. Editing it would fail `drift/ipc bindings unchanged`, so it stays as generated; no authored file carries trailing whitespace |

## Integrity and history (prompt §28, §31)

| Box | Status | Evidence |
| --- | --- | --- |
| `DIRECTORY_TREE.txt` updated | LOCAL PASS | the tracked path set changed (ADR-0025 added; `intake.rs`, `real_artifact_intake.rs`, `intake.test.tsx`, `SelectionDto.ts`, four `P1_A0_VALIDATION/*.md` added; `App.test.tsx` removed), so the tree was regenerated: **475 lines over 395 tracked paths**, a 13-insertion / 4-deletion diff against the committed record. The generator was first proved to reproduce the committed file from HEAD exactly except for the legitimately added entries |
| `SHA256SUMS` regenerated last | LOCAL PASS | written after every other change: **394 entries** = 395 tracked paths minus the manifest itself |
| No self-entry | LOCAL PASS | `grep -c "SHA256SUMS" SHA256SUMS` → 0 |
| 0 mismatch | LOCAL PASS | `sha256sum -c SHA256SUMS` → exit 0, plus an independent Python checker that rehashes every line and compares digests and path sets |
| 0 duplicate | LOCAL PASS | duplicate-path and duplicate-line scan over the manifest → 0 |
| No CI run number of this task written back | LOCAL PASS | §28's anti-recursion rule is honoured: this tree names Run #6 as the authority fact that predates it and records no run for its own commit. `git log` for the two P1-A0 commits carries no run id |
| Worktree clean after commit | LOCAL PASS | `git status --short` empty once the commit is made |
| No history rewrite | LOCAL PASS | two new commits on top of `311f9fc`; no rebase, no squash, no amend, no force. `7d2f38a`, `738ae78` and the P0 closure history are untouched |
| Not pushed | BY DESIGN | prompt §31: commit locally, the user pushes. Remote CI for this slice is therefore **NOT RUN**, and no document here claims otherwise |

## What is deliberately left open

* The snapshot-identity consequence recorded as finding 1 in `P1_A0_EXECUTION_REPORT.md`: re-analyzing
  the same bytes with a MAP does not refresh the persisted evidence. Fixing it needs a change to
  frozen snapshot semantics or to the schema, and §12 authorizes neither.
* Peak RSS for the desktop path stays `NOT MEASURED`; fuzzing stays `NOT RUN`. Both were P0's open
  gaps and this slice neither closed nor widened them.
* One host, one WebView, one dialog toolkit (Win32 `rfd`) were exercised. The Linux and macOS dialog
  backends are untested by this round, and `tauri-plugin-dialog`'s `gtk3` feature choice is recorded
  as a decision rather than a verified platform path.
