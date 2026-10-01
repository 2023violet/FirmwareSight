---
title: "P4 Release Bundle Exit Checklist"
doc_id: "FS-P4-EXIT"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P4_RELEASE_BUNDLE"
owner: "Engineering"
last_updated: "2026-10-01"
---

# P4 Release Bundle — exit checklist (prompt §72)

Sixty-one boxes, each settled by a command, a named test or a numbered smoke step. `S` cites
`P4_BUNDLE_DESKTOP_SMOKE_REPORT.md`, `E` cites `P4_BUNDLE_EXECUTION_REPORT.md`.

## Architecture

- [x] **no sixth crate** — `ls crates` lists five: artifact, core, project, report, storage; the apps
  are `apps/cli` and `apps/desktop`, as before P4.
- [x] **no new third-party dependency** — `git diff 323afad HEAD -- Cargo.lock` is empty; the portable
  reader is Python stdlib with `jsonschema` optional, and contract validation uses Core's own
  dependency-free `schema_check` subset.
- [x] **Core owns release semantics only** — `crates/firmwaresight-core/src/domain/release.rs` holds
  the release id, the plan and the canonical byte list; `grep -rn "std::fs" crates/firmwaresight-core/src/`
  counts **0**.
- [x] **filesystem assembly outside Core** — staging, publish and rollback live in
  `crates/firmwaresight-project/src/bundle.rs` (`publish`, bundle.rs:371, with the rollback arm at
  bundle.rs:1387) and are driven by the CLI and the desktop engine.

## Gate binding

- [x] **Bundle revalidates Gate context** — a moved workspace invalidates the run before anything is
  written: `a_moved_workspace_invalidates_the_run_before_anything_is_written`, and `S48` shows
  `ERR-BUNDLE-6103` on the shipped binary.
- [x] **run id must still match** — the plan carries the stored run id and the verifier recomputes
  the release id from it; `S17`/`S23` show one id across page, bundle and database.
- [x] **disposition PASS required** — `a_run_that_has_not_passed_produces_no_plan`; the CLI smoke's
  REVIEW section is refused before any directory exists.
- [x] **accepted Reviews exact** — `an_accepted_review_lets_the_same_run_prepare_and_counts_it`;
  `S30` reads the acceptance back out of the bundle verbatim (actor, reason, moment, original state).
- [x] **UNKNOWN cannot be waived** — P3's rule stands: storage answers a non-review acceptance with
  `ERR-STORAGE-4008`, and no acceptance control exists for an UNKNOWN row.

## Version

- [x] **deterministic project release version** — the tag the policy's pattern extracts; the golden
  subject pins its Git facts and `apps/cli/tests/p4_golden.rs` asserts the pinned HEAD, so the same
  inputs produce `1.2.3` and the same release id forever.
- [x] **no arbitrary export-time version** — nothing in the plan reads a clock; `S14` and the
  portability reader's clock rule (only `accepted_at` may carry a date) close this from both ends.

## Source safety

- [x] **ELF rehashed before copy** — `S46`–`S48`: one flipped byte, `ERR-BUNDLE-6103`, nothing written.
- [x] **MAP rehashed before copy** — the same revalidation covers every artifact row; the engine
  hashes each source at write time against the plan digest it stored.
- [x] **source mutation fails closed** — `S48`'s disk state: tree fingerprint unchanged, no staging
  residue anywhere under the smoke root.
- [x] **notes mutation fails closed** — `rewritten_notes_are_refused_rather_than_shipped_under_the_old_digest`.

## Portable

- [x] **analysis:1** — `schemas/analysis.schema.json`, contract-tested through Core's `schema_check`.
- [x] **diff:1 when baseline** — `schemas/diff.schema.json`; absent without a baseline, present here.
- [x] **gate-results:1** — `schemas/gate-results.schema.json`, P3's contract unchanged.
- [x] **accepted-reviews:1** — `schemas/accepted-reviews.schema.json`, P3's contract unchanged.
- [x] **release-manifest:1** — `schemas/release-manifest.schema.json`, closed items
  (`additionalProperties: false`).
- [x] **deterministic report HTML** — `golden/reports/p4-release/release-report.html` compared byte
  for byte by `p4_golden.rs`; no script, no remote reference, no timestamp beyond the acceptance.
- [x] **SHA256SUMS** — `sha256sum`-compatible text; `S32` recomputed all nine listed digests.

## Integrity

- [x] **no self-hash lie** — `SHA256SUMS` lists every payload except itself and the manifest; no
  zeroed or empty digest appears anywhere (`the_two_indexes_cover_each_other_the_way_the_integrity_model_states`).
- [x] **explicit self-reference model** — the rule is written into the manifest's
  `extensions.integrity_model` and into the report's section 8, so a reader verifies without asking
  FirmwareSight.
- [x] **manifest covers every non-manifest bundle file** — the reader asserts
  `listed − sums = {SHA256SUMS}` on both the golden and the desktop bundle.
- [x] **hashes/sizes verify** — `S32`, plus the verifier's per-file size and digest pass (64/64).

## Filesystem

- [x] **staged export** — bytes land in a sibling staging directory, are verified there, and publish
  is a rename (`bundle.rs` `publish`).
- [x] **no partial final bundle** — `S48`: after a refused export the destination is byte-identical
  to before and holds no staging directory.
- [x] **no implicit overwrite** — `S42`: `ERR-BUNDLE-6106` and a question, never a silent write.
- [x] **explicit replace confirmation** — `S44`: the named button, naming the folder it replaces.
- [x] **arbitrary directory never deleted** — `overwrite_never_replaces_a_folder_this_engine_did_not_write`
  and the CLI smoke's `--force` section: exit 6, `ERR-BUNDLE-6107`, the stranger's file surviving.
- [x] **rollback path tested** — `crates/firmwaresight-project/src/bundle/tests.rs`'s rollback case
  (the previous bundle is moved aside and put back if the swap fails).

## Storage

- [x] **additive 0004** — `0004_release_records.sql` adds one table and touches no existing column.
- [x] **SCHEMA_VERSION = 4** — `the_schema_is_at_version_four_and_the_release_table_starts_empty`.
- [x] **old DB upgrades** — v1→v4, v2→v4 and v3→v4 tests, plus
  `an_acceptance_written_before_the_upgrade_is_still_there_after_it` and
  `a_failed_0004_leaves_the_v3_database_recoverable`; the smoke's handed-back database (migrations
  1–2) is the live proof that a pre-P4 file still opens.
- [x] **release_records immutable** — `a_stored_record_refuses_update_and_refuses_delete`;
  `one_release_id_with_different_facts_is_an_invariant_not_an_update`;
  `the_same_release_recorded_twice_dedupes_instead_of_duplicating` (observed in `S45`: the
  replacement wrote no second record).
- [x] **no path stored** — `the_release_table_has_no_column_that_could_hold_a_path_or_a_blob`.

## CLI

- [x] **release prepare** — `fwsight release prepare …`, `E` §3.
- [x] **non-interactive** — every refusal is an exit code and a stderr envelope; the smoke ran with
  no tty input.
- [x] **stable exits** — 0 PASS, 2/3 analysis and usage failures, 4 REVIEW, 5 BLOCK, 6 bundle refusal.
- [x] **--force safe** — refuses a stranger directory, replaces only a recognized bundle, byte-identical.
- [x] **--json manifest** — the stdout manifest equals the on-disk `release-manifest.json` byte for byte.
- [x] **no project DB side effect** — the CLI prints "this run is not stored"; the smoke's DB gained
  its record from the desktop only.

## Desktop

- [x] **Bundle under Release, no fifth verb** — the rail assertion pins three pages; `S4`.
- [x] **preview** — `S12`–`S20`: ten rows, roles, sizes, digests, no host path.
- [x] **native destination** — `S21`/`S41`: the OS folder dialog, a token back, never a path in IPC.
- [x] **no path leak** — `S20`, `S38`, and the reader's sweep over every composed document.
- [x] **overwrite confirmation** — `S42`–`S44`.
- [x] **stale plan rejection** — `S47`–`S48`, and the plan is consumed so it cannot be re-exported.
- [x] **success summary** — `S23`/`S45`: release, folder, manifest digest, file count, and the honest
  sentence about the release record.

## Portability

- [x] **relocated bundle independently readable** — `S39`–`S40`, 59/59.
- [x] **source/project/DB not required** — inputs renamed away, and after the clean close the
  database absent too; both runs 59/59.
- [x] **HTML offline** — `S33`: zero scripts, links, images, iframes or subresources in the live page.
- [x] **schemas valid** — 64/64 with `--schemas schemas`.
- [x] **SHA256SUMS valid** — every listed digest recomputed; the coverage relation holds.

## Engineering

- [x] **fmt** — `cargo fmt --all -- --check`, exit 0 (`E` §5).
- [x] **clippy** — `cargo clippy --workspace --all-targets --all-features -- -D warnings`, exit 0.
- [x] **direct workspace tests correctly counted** — one `cargo test --workspace`: **769 passed,
  0 failed**; no rerun summed in, and the retired 715 appears nowhere in this pack.
- [x] **UI tests** — 155 passed in 6 files.
- [x] **frontend build** — `corepack pnpm build`, exit 0.
- [x] **drift** — tokens, icons, IPC bindings, bindings-unchanged, fixtures tracked, goldens unchanged.
- [x] **deny** — `advisories ok, bans ok, licenses ok, sources ok` on a refreshed index.
- [x] **core-smoke** — pass.
- [x] **full gate** — `python scripts/check.py`: 15/15 steps.
- [x] **git diff --check** — clean.
- [x] **shipping build** — `E` §6, with the source-equivalence measurement.
- [x] **real Windows smoke** — all fifty §59 steps, `S`.

## Verdict

Every box above is checked by something that ran. Therefore:

```text
P4 = PASS / COMPLETE
G2 = READY_FOR_ENGINEERING_GATE_REVIEW
```

Not `G2 = PASS`: the whole-MVP closure audit is the architect's, in a round of its own. Baseline stays
**0.6.0**; this closure creates no `v0.7.0` and opens no P5, V1, B1, RC1 or GA1.
