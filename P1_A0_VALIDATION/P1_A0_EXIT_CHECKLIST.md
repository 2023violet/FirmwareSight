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
| MAP strengthens evidence | LOCAL PASS | `attaching_a_map_strrengthens_the_evidence_and_detach_takes_it_back`; in the window: not admissible → admissible, `layout_source` none → map, `map` capability not-provided → provided, evidence 10 → 11. The closure round established that SQLite now keeps that stronger record rather than only displaying it - see `P1_A0_CORRECTNESS_SMOKE_REPORT.md` |
| Last-good preserved | LOCAL PASS | §15 semantics asserted by three UI tests (`keeps the last good analysis visible…`, `never presents the last good analysis as the failed candidate`, `replaces the last good analysis when a new candidate succeeds`) and seen in the window at step 16 |
| Cancel non-error | LOCAL PASS | `a_cancelled_dialog_is_not_an_error`, `treats a cancelled artifact dialog as a normal outcome, not an error`, `treats a cancelled MAP dialog as a normal outcome`; step 13 in the window |
| Malformed input typed, no panic | LOCAL PASS | `ERR-FORMAT-0001` in the window with diagnostics id `op-609c-18d98be1034ea388`; `format_truth_comes_from_the_bytes_not_from_an_elf_extension`, `an_artifact_deleted_after_selection_is_a_typed_error_not_a_panic`, `an_unknown_selection_id_is_a_typed_error_and_not_a_panic` |
| No DB migration | LOCAL PASS | `the_selection_path_adds_no_schema_migration`; the live database still lists `schema_migrations` = (1, 2) and `SCHEMA_VERSION` is 2 |

## Engineering gates (prompt §19, §20, §21, §22, §26)

| Box | Status | Evidence |
| --- | --- | --- |
| Rust tests PASS | LOCAL PASS | `cargo test --workspace` → **142 passed / 0 failed** after the correctness-closure round. The intake round's own measurement was 123 passed / 0 failed including the 18 new `real_artifact_intake.rs` tests (≥10 required); the closure round added 19 more and removed none |
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
| No CI run number of this task written back | LOCAL PASS | §28's anti-recursion rule is honoured: the implementation commit `2960173` names no run of its own - `git log -1 --format=%b 2960173 | grep -E "Run #\|36419864513\|36499759371"` returns nothing. The governance commit `311f9fc` does quote Run #6, and that is a fact that predates it, not a result it caused |
| Worktree clean after commit | LOCAL PASS | `git status --short` empty once the commit is made |
| No history rewrite | LOCAL PASS | two new commits on top of `311f9fc`; no rebase, no squash, no amend, no force. `7d2f38a`, `738ae78` and the P0 closure history are untouched |
| Not pushed | SUPERSEDED | prompt §31 of the intake round said commit locally and let the owner push, and that is what happened: `2960173` was pushed by the repository owner and remote **Run #9 `36499759371`** measured it `success`, 7 of 7 jobs, before the correctness-closure round began. At the intake round's own closing moment this row read `BY DESIGN / NOT RUN`, which was true then; it is corrected here rather than left to contradict `BASELINE.yaml` |

## Correctness closure (closure prompt §25, 2026-09-29)

All 40 boxes the closure prompt lists, in its own order — 1 remote, 6 identity, 7 persistence,
7 evidence, 11 validation, 8 governance:

| Box | Status | Evidence |
| --- | --- | --- |
| Run #9 completed success 7/7 | PASS (remote) | `gh run view 36499759371 --json databaseId,headSha,status,conclusion,jobs` - `completed` / `success` on `29601735a2eff2c9e3677ef1dfa88cf752484694`, 7 jobs each `success`: Rust windows+ubuntu, Desktop UI windows+ubuntu, Generated output drift, Dependency policy, macOS Core Smoke. Verified before any file was written |
| ELF-only 1 artifact | LOCAL PASS | `an_elf_only_snapshot_seals_one_artifact`; smoke step 6 shows 1 artifact row |
| ELF+MAP 2 artifacts | LOCAL PASS | `a_map_makes_the_snapshot_seal_a_companion_artifact`; `counts.artifacts` 1 → 2 in the golden; smoke step 12 shows rows `#0` Elf and `#1` Map |
| Truthful MAP artifact metadata | LOCAL PASS | `the_companion_artifact_reports_the_map_bytes_and_the_gnu_ld_adapter`, `the_companion_artifact_claims_no_elf_facts_a_map_cannot_have`; the stored `#1` row reads `Unknown`/`Unknown`/`Unknown` with an entry reason |
| Snapshot id changes with MAP input | LOCAL PASS | `a_map_changes_the_snapshot_id_through_the_existing_optional_map_component`, `attaching_a_map_changes_the_snapshot_id_and_removing_it_restores_it`, X ≠ Y in the window |
| Same inputs stable | LOCAL PASS | `the_same_pair_produces_the_same_id_and_a_different_map_produces_another`; repeated analysis left the database at 2 builds |
| No normalization bump | LOCAL PASS | `closing_the_identity_gap_needs_no_normalization_version_bump`; `NORMALIZATION_VERSION` is still `p0-normalize-1` and the ELF-only id formula is unchanged |
| ELF-only build persists | LOCAL PASS | smoke step 6; `a_map_makes_a_second_build_rather_than_updating_the_first` |
| ELF+MAP build persists separately | LOCAL PASS | smoke step 11 (`builds` = 2); `the_map_build_persists_both_artifacts_and_the_stronger_evidence` |
| Stronger MAP evidence survives storage | LOCAL PASS | smoke step 13: `layout_source map`, `admissible_hard_block 1`, 11 evidence rows; and `with.evidence_summary.total` equals the stored count in `a_map_after_an_elf_only_analysis_is_stored_as_a_second_build` |
| Repeated analyses dedupe | LOCAL PASS | smoke steps 14-15 and 19; `repeating_either_input_dedupes_and_never_stores_a_third_build` |
| Remove MAP returns to ELF-only snapshot | LOCAL PASS | smoke steps 16-18: the id came back byte-identical to X and no third build appeared |
| No schema migration | LOCAL PASS | `closing_the_identity_gap_adds_no_schema_version_and_no_migration`, `the_selection_path_adds_no_schema_migration`; the live validation database lists migrations (1, 2) only |
| BuildSummary always primary ELF | LOCAL PASS | `a_two_artifact_build_summarizes_the_primary_elf_never_the_map_placeholder` plus the rowid-shuffle reproducer, which is the only one of the two that would have failed before the fix |
| No false MapFile on ELF-only evidence | LOCAL PASS | `an_elf_only_run_names_no_map_in_its_evidence` over both fixtures; in the shipping binary the ELF-only build stored **0** `MapFile` rows |
| No fake `map:` locator without MAP | LOCAL PASS | same test; the live database held **0** locators containing `map:` for the ELF-only build, against 2 such rows before the fix |
| MAP-backed evidence uses MapFile | LOCAL PASS | `a_map_backed_run_uses_the_map_source_and_keeps_observed_items_observed`; 5 `MapFile` rows on the MAP build |
| Heuristics remain Derived/Low | LOCAL PASS | `each_charge_records_the_source_and_class_its_own_rule_actually_supports` covers all five rungs, including Derived ⇒ Low confidence, replacing the hardcoded `Observed` |
| `from_map` removed | LOCAL PASS | no `from_map` in `ipc.rs` or `service.rs`; `an_elf_only_summary_claims_no_map_provenance_and_carries_no_from_map_flag` asserts it is absent from the wire |
| Generated TS updated | LOCAL PASS | `EvidenceSummaryDto.ts` regenerated by ts-rs, no `fromMap`; `the_generated_typescript_binding_carries_no_from_map_field`; `drift/ipc bindings unchanged` 5/5 |
| No redundant replacement boolean | LOCAL PASS | `EvidenceSummaryDto` is the five class counts it was, minus the flag; nothing was added |
| Rust PASS | LOCAL PASS | `cargo test --workspace` → **142 passed, 0 failed** summed over every target; `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` both clean |
| UI PASS | LOCAL PASS | `pnpm install --frozen-lockfile`, `typecheck`, `lint`, `test` → **31 passed (3 files)**, `build` → clean |
| storage PASS | LOCAL PASS | `cargo test -p firmwaresight-storage` → **17 passed**, of which 6 are `map_companion_persistence.rs`; `cargo test -p firmwaresight-artifact --test map_companion_identity` → **10 passed**; `cargo test -p firmwaresight-desktop --test real_artifact_intake` → **21 passed** |
| goldens reviewed | LOCAL PASS | 2 goldens, 4 changed lines each, every line itemised with its old value, new value and the reason in `P1_A0_EXECUTION_REPORT.md` §9.5; `cargo test -p fwsight --test golden` → 5 passed; no bulk regeneration |
| drift PASS | LOCAL PASS | `python scripts/check.py --only drift` → **5/5**, including `drift/ipc bindings unchanged` and `drift/goldens unchanged` |
| deny PASS | LOCAL PASS | `python scripts/check.py --only deny` → **1/1**, `cargo deny 0.20.2` executed for real: advisories ok, bans ok, licenses ok, sources ok |
| core-smoke PASS | LOCAL PASS | `python scripts/check.py --only core-smoke` → **3/3** |
| full gate PASS | LOCAL PASS | `python scripts/check.py` → **14/14 steps passed**, exit 0, 0 SKIPPED and 0 FAIL |
| `git diff --check` PASS | LOCAL PASS | exit 0, and `git diff --cached --check` exit 0 for this round's staged set |
| Real Desktop correctness smoke PASS | LOCAL PASS | `P1_A0_CORRECTNESS_SMOKE_REPORT.md` - 21/21 against `f50441fe…`, the binary this round rebuilt with `--features custom-protocol`, on a validation database that was clean at start and restored the user's live database by hash afterwards |
| integrity PASS | LOCAL PASS | `DIRECTORY_TREE.txt` regenerated - **478 lines over 398 tracked paths**; the delta against HEAD's tracked set is exactly 3 additions (`P1_A0_CORRECTNESS_SMOKE_REPORT.md`, `map_companion_identity.rs`, `map_companion_persistence.rs`) and 0 removals. `SHA256SUMS` written last: **397 entries** = 398 tracked paths minus the manifest itself, **0 duplicates, no self-entry, 0 mismatch** (`sha256sum -c SHA256SUMS` → 397 `: OK`, nothing else) |
| P0 PASS | LOCAL PASS | still `PASS`, frozen at v0.6.0; this round fixed two defects inside the pre-G1 slice and touched no P0 verdict |
| baseline v0.6.0 | LOCAL PASS | `baseline_version: 0.6.0` unchanged, and `0.7.0` appears nowhere in `BASELINE.yaml` (asserted) |
| V0 0/8 | LOCAL PASS | unchanged: 0 eligible external sessions, Batch A 0 / 4-5, and no file in this round claims otherwise |
| G1 NOT CLAIMED | LOCAL PASS | `g1_claimed: false`; `G1 = V0 PASS + P0 PASS` and only the P0 half exists |
| P1-A0 COMPLETE | LOCAL PASS | intake commit `2960173` green on Run #9, and this closure commit completes the identity and provenance work inside the same slice |
| P1 NOT PASS / NOT CLOSED | LOCAL PASS | unchanged - a closed P1-A0 is not P1 progress |
| P1-A1 NOT AUTHORIZED | LOCAL PASS | `p1_a1_authorized: false`; it needs V0 Batch A `>= 4` eligible sessions, an interim architect review and a new prompt |
| P2/P3/P4 NOT AUTHORIZED | LOCAL PASS | all three remain `false`, and §21's non-scope list was honoured: no Sections/Symbols tables, Evidence Inspector, Compare, Gate, Bundle, History, wizard, Git linking, SBOM, Component Evidence, new adapters or new navigation |

## What is deliberately left open

* Peak RSS for the desktop path stays `NOT MEASURED`; fuzzing stays `NOT RUN`. Both were P0's open
  gaps and this slice neither closed nor widened them.
* One host, one WebView, one dialog toolkit (Win32 `rfd`) were exercised. The Linux and macOS dialog
  backends are untested by this round, and `tauri-plugin-dialog`'s `gtk3` feature choice is recorded
  as a decision rather than a verified platform path.
* Four items the closure round surfaced and deliberately did **not** change, each with its reason in
  `P1_A0_EXECUTION_REPORT.md` §9.7: `scripts/update_goldens.py` no longer reproduces the committed
  goldens' key order; `ElfProgramHeader` names a source the ELF parser never reads;
  `schemas/release-manifest.schema.json` describes one artifact per build, which a MAP-bearing
  snapshot id can no longer be projected into without a schema major; and a plain
  `cargo build --release` without `--features custom-protocol` yields a window showing a network
  error rather than the product.
* The previous round's three `*.p0-smoke-history*` files are still in the app-data directory. They
  are not this round's to delete, and the closure smoke restored the live database by hash instead of
  repeating that technique.
