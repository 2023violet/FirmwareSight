---
title: "P2 Compare Exit Checklist"
doc_id: "FS-P2-EXIT"
product: "FirmwareSight"
version: "1.0"
status: "EXECUTION_RECORD"
stage: "P2_COMPARE"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P2 Compare Exit Checklist (prompt §61)

Every box is marked with the thing that settles it: a test name, a command, or a line of a smoke
report. `LOCAL PASS` means the command was run here and passed; it does not mean a CI run covered it.
Anything not settled is left unticked with the reason beside it.

## Core

| Box | Mark | Settled by |
| --- | --- | --- |
| Deterministic Diff domain | LOCAL PASS | `the_same_inputs_twice_produce_byte_identical_ordering` + the §4.4 re-render: `cmp` silent on both `--json` (64,355 B) and `--html` (41,953 B) |
| Old / new explicit | LOCAL PASS | `a_bigger_target_yields_a_positive_delta_because_delta_is_target_minus_base`; live: `Old 256 bytes → New 376 bytes Delta +120 bytes` |
| Signed delta correct | LOCAL PASS | `a_smaller_target_yields_a_negative_delta`, `reversing_the_comparison_inverts_added_and_removed_and_flips_every_sign`, `an_address_that_moved_is_a_change…`; live reversal table in the smoke report |
| Added / Removed not zero | LOCAL PASS | `an_added_section_is_not_reported_as_a_change_from_zero`, `a_removed_section_is_not_reported_as_a_change_to_zero`, `added_and_removed_symbols_keep_their_size_without_inventing_a_delta`; live: `.calib` New file `Not present`, `Delta Unknown` |
| Unknown not zero | LOCAL PASS | `an_unknown_budget_produces_no_delta_at_all_rather_than_a_number_minus_zero`, `a_side_with_no_stored_footprint_row_reads_as_unknown_not_as_an_empty_budget`, `a_symbol_with_no_size_on_one_side_yields_no_delta_rather_than_a_zero_one` |
| Conservative matching | LOCAL PASS | `a_section_name_that_appears_once_on_each_side_is_matched_by_name`, `a_symbol_matches_only_when_name_kind_and_binding_all_agree`, `an_unnamed_section_cannot_be_matched_and_is_reported_as_unpaired`, `a_section_that_only_differs_by_an_unknowable_field_is_not_claimed_as_changed` |
| Duplicate ambiguity honest | LOCAL PASS | `a_duplicated_section_name_is_left_unpaired_and_counted_as_ambiguous`, `a_duplicated_symbol_key_is_never_paired_by_position`, `a_repeated_symbol_name_across_both_sides_still_completes_and_marks_every_row_ambiguous`; live `SYMBOL-AMBIGUOUS: 67` identical in both directions |
| Object attribution honest | LOCAL PASS | `object_attribution_is_reported_unavailable_with_the_reason_that_is_true`; golden `objectChanges.available = false` with a reason naming the object-file requirement; live `Object attribution — unavailable` |

Extra invariants the §8 list asked for, each with its own test: no input mutation
(`comparing_does_not_mutate_either_snapshot_input`), same-snapshot comparison refused
(`comparing_a_snapshot_with_itself_is_refused_rather_than_answered_with_empty_tables`), missing MAP
never blocks (`a_missing_map_never_blocks_the_comparison_but_always_shows_the_downgrade`,
`both_sides_without_a_map_says_so_instead_of_naming_the_wrong_side`), delta overflow fails closed
(`a_delta_too_large_for_a_signed_count_is_absent_rather_than_wrapped`), one-sided floor stays visible
(`each_side_keeps_its_own_budget_state_so_a_floor_stays_visible`).

## Storage

| Box | Mark | Settled by |
| --- | --- | --- |
| Persisted snapshots compare without source files | LOCAL PASS | `compare_candidates.rs` hydrates a diff input from stored facts only; the desktop smoke ran Compare after Analyze had already finished, and the page states it never re-reads the files |
| Candidate list works | LOCAL PASS | `compare_candidates.rs` (14 tests) — bounded, deterministic ordering, latest-analysed preference; live: both pickers listed both builds and one was marked `· last analyzed` |
| No migration | LOCAL PASS | `SCHEMA_VERSION: i64 = 2`; migrations on disk are `0001_initial.sql`, `0002_evidence_keyed_by_build.sql`; **no 0003 exists**; §58's STOP condition not reached |

## Fixture

| Box | Mark | Settled by |
| --- | --- | --- |
| Dedicated P2 pair | LOCAL PASS | `fixtures/elf/p2-diff/{base,target}/` with ELFs, GNU-ld MAPs, sources, `fixture.toml`, generator `scripts/gen_p2_fixtures.py`, registered in `fixtures/manifest.json` — **and both halves now actually tracked**: the target half was ignored by `**/target/` until defect E (`cfee1e5`), so a clean checkout had six of twelve P2 paths |
| Exact expected changes asserted | LOCAL PASS | `p2_fixture_pair.rs` (12 tests) pins the counts and the named rows: sections `+1 -1 ~16`, symbols `+38 -33 ~5`, 67 unpaired, `.ota` / `.calib`, `.debug_loclists +221` |
| Hashes frozen | LOCAL PASS | base ELF `3f615b6247179d93…`, base MAP `832690060a8d25f8…`, target ELF `4b4087e1407ceddc…`, target MAP `a38575cd51ec2730…`; the two artifact hashes are the golden's `base`/`target` sha256 values and the CLI's own `Base/Target` lines. Verified from a clean tree: `git archive HEAD` → 22 manifest entries, **0 missing, 0 hash mismatch**, `p0_acceptance` 19 passed, `p2_fixture_pair` 12 passed |

## Report

| Box | Mark | Settled by |
| --- | --- | --- |
| Diff JSON v1 schema | LOCAL PASS | `schemas/diff.schema.json`, `$id: urn:firmwaresight:schema:diff:1`; `diff_schema_contract.rs` (19 tests) is the mandatory contract test; the CLI document validated with `jsonschema` 4.26.0 `Draft202012Validator` → **0 errors** |
| Deterministic JSON | LOCAL PASS | two renders byte-identical (`1930cdb6…`); `golden/core/p2-diff.json` equal to the CLI document semantically (`True`) |
| Self-contained deterministic HTML | LOCAL PASS | two renders byte-identical (`aa4b08df…`) and identical to `golden/reports/p2-diff.html`; exported file structurally: 1 inline `<style>`, 0 external `src`/`href`, 0 `@import`, 0 `url(`, 0 `<script>`, no unclosed tags |
| No host paths | LOCAL PASS | regex over the document for drive letters, `/home/`, `/Users/`, UNC and `AppData` → **0 matches**; locators are file names; UI never receives a full path (§35) and on-screen notes name the file only |

## CLI

| Box | Mark | Settled by |
| --- | --- | --- |
| `fwsight diff old new` | LOCAL PASS | §4 of the execution report, exit 0; `diff_cli.rs` (15) + `p2_golden.rs` (11) + 8 unit tests |
| MAP support | LOCAL PASS | `--old-map` / `--new-map` parsed with the `gnu_ld` adapter (stderr diagnostics), and both MAP hashes reappear in the snapshot ids |
| JSON | LOCAL PASS | `--json` emits exactly one document on stdout, diagnostics on stderr, trailing bytes = 1 newline |
| HTML | LOCAL PASS | `--html FILE` wrote the golden byte-for-byte |
| Stable exit codes | LOCAL PASS | 0 / 2 / 3 / 6 observed (`ERR-INPUT-0001`, `ERR-FORMAT-0001`, `ERR-PARSE-2002`, `ERR-EXPORT-6001`); **never 1**; no stack trace; `gate` `release` `watch` `doctor` still `unrecognized subcommand` (exit 2) |

## Desktop

| Box | Mark | Settled by |
| --- | --- | --- |
| Analyze + Compare navigation | LOCAL PASS | Smoke step 1; rail lists both pages. Defect B (rail scrolled away) fixed and re-verified at 1,000 px |
| Snapshot selectors | LOCAL PASS | Smoke steps 5–7 |
| Old / new / delta summary | LOCAL PASS | Smoke steps 9–11 |
| Evidence degradation | LOCAL PASS | Smoke step 20, per-side evidence + pair comparability |
| Top growth | LOCAL PASS | Smoke steps 15–16; growth lists Changed rows only |
| Full section changes | LOCAL PASS | `Showing 1 to 18 of 18 sections` |
| Full symbol changes | LOCAL PASS | `Showing 1 to 76 of 76 symbols` |
| Added / Removed / Changed filters | LOCAL PASS | `compare.test.tsx` + the filter row; counts row `Added 1 / Removed 1 / Changed 16` |
| Drill-down | LOCAL PASS | Smoke step 15: clicking `.debug_loclists` filtered to 1 row and wrote the name into the filter field; step 16 clears it |
| Bytes / KiB | LOCAL PASS | Smoke steps 17–19: `/1024` only, addresses unchanged in both units |
| JSON export | LOCAL PASS | Smoke steps 22–23, native dialog, 64,355 B, schema-valid, no host path |
| HTML export | LOCAL PASS | Smoke steps 24–25, 41,953 B, opened independently in a browser window from its own `<title>` |
| Last-good preservation | LOCAL PASS | Smoke step 28; live note after defect C names the standing pair and the selected pair |
| No path leak | LOCAL PASS | Smoke step 29; export notes and the OS overwrite dialog name the file only |
| Cancel is not an error | LOCAL PASS | Smoke step 26: `Export cancelled. No file was written.`, no alert role |
| Failed export leaves the file intact | LOCAL PASS | Answering `No` and then `Keep mine` left sha256 `aa4b08df…` and mtime `13:20:19` unchanged |

## Engineering

| Box | Mark | Settled by |
| --- | --- | --- |
| `cargo fmt --all -- --check` | LOCAL PASS | no output |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | LOCAL PASS | exit 0 |
| `cargo test --workspace` | LOCAL PASS | **345 passed, 0 failed** |
| Frontend typecheck | LOCAL PASS | `tsc --noEmit`, no diagnostics |
| Frontend lint | LOCAL PASS | `eslint .`, no diagnostics |
| UI tests | LOCAL PASS | **99 passed** in 5 files (`compare.test.tsx` carries 37) |
| Frontend build | LOCAL PASS | `vite build`, 274.49 kB / gzip 82.09 kB |
| `check.py --only drift` | LOCAL PASS | 6/6, once the regenerated goldens were committed — the sixth step, `fixtures tracked`, exists because of defect E |
| `check.py --only deny` | LOCAL PASS | cargo-deny ran for real: `advisories ok, bans ok, licenses ok, sources ok` |
| `check.py --only core-smoke` | LOCAL PASS | 3/3 |
| Full `check.py` | LOCAL PASS | 15/15, reported in §10 of the completion report |
| `git diff --check` | LOCAL PASS | no whitespace errors |
| Windows Desktop smoke | PARTIAL, honest | 30 steps in `P2_COMPARE_DESKTOP_SMOKE_REPORT.md`; **step 27 not verified in the shipped window** (native `<select>` popup cannot be driven or captured here) — covered only by tests |
| Clean-checkout reproducibility | LOCAL PASS | `git archive HEAD` extracted to a fresh directory and the fixture tests run there (numbers in the Fixture table above). This is what defect E was missing |
| Remote CI over the final tree | **NOT GREEN, NOT COVERED** | `origin/main` is at `c7fc2a3`, whose run `36596452341` (#17) is `completed/failure` on `committed_fixtures_match_their_recorded_hashes`. The cause is defect E and the fix is `cfee1e5`, but the fix is unpushed, so no run covers the final tree. Closing the red needs a push, which is a shared-state action left to the owner |

## Boundary conditions from the prompt

| § | Condition | Reached? |
| --- | --- | --- |
| §57 | No new dependency | **Not reached.** `git diff 7a13660..HEAD -- Cargo.lock \| grep '^+name ='` → empty; the lock's only lines are two intra-workspace edges. `pnpm-lock.yaml` and `package.json` untouched |
| §58 | Schema change needed | **Not reached.** `SCHEMA_VERSION = 2`, migrations `0001`/`0002`, no `0003` |
| §56 | A genuinely required new design token | **Not reached.** `assets/design-tokens.json` still v0.2.1 and `DESIGN.md` unchanged; 394 `var(--fs-*)` references resolve into the generated `tokens.css`, 0 undefined |
| §33 | Gate exit codes activated | **No.** Only 0/2/3/6; a diff never returns a gate verdict |
| §43 | Per-symbol / per-section evidence provenance claimed | **No.** The report states what the pairing did and what it could not decide; attribution is unavailable and says why |
| §63 | A future CI run number written into the commit that triggers it | **No.** The two commits pushed mid-round record no run number; the run they produced is recorded by the commits that follow them |
| §65 | Baseline promoted to v0.7.0 | **No.** `baseline_version` stays **0.6.0** |
| §66 | P3 / pricing / commercial validation / cloud / accounts / telemetry / AI | **None implemented.** P3 is recorded as `NEXT_AUTHORIZABLE_STAGE` only |

## Verdict

**P2 = PASS / COMPLETE** on every §61 box, with two things stated rather than folded away:

1. Desktop smoke step 27 is not observed in the shipped window, so that one sub-item stays `PARTIAL`.
2. The remote branch is red at the mid-round commit `c7fc2a3` for defect E. The fix is committed
   (`cfee1e5`) and verified from a clean checkout, but it is unpushed, so **no CI run covers this
   tree** — every result above is `LOCAL PASS`, and turning the red into a green run is a push
   decision for the owner, not a claim this checklist can make.

P3 = `NEXT_AUTHORIZABLE_STAGE`. Do not implement P3.
