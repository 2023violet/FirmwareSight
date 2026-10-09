---
title: "C1-U1 Execution Report — Release Attachment Data and Identity"
doc_id: "FS-C1-U1-EXEC"
product: "FirmwareSight"
version: "0.1.0"
status: "RECORD"
owner: "Engineering"
last_updated: "2026-10-09"
---

# C1-U1 — Release Attachment Data & Identity, execution report

Round record for 《FirmwareSight — C1-U1 / Release Attachment Data & Identity, Coding Agent 实施授权 v1.0》 and its
in-flight continuation. The plan this round worked from, including the ERR-BUNDLE registry findings and the four
mutation proofs, is `C1_U1_DATA_AND_IDENTITY_PLAN.md` beside it (§11 and §12 are its dated additions).

## 1. Authority and preflight

- Expected handover HEAD from §1: `114a84e3cb3e9e5fb7f23ee17f044f8d46337bc7`. Measured with `git rev-parse HEAD` at
  the start of the round: identical. `git status --porcelain` was empty, one worktree, no stash entry.
- Scope read verbatim from the authorization: **only** C1-U1. Its own words — "这不是重做产品方向论证，不是
  C1-U2/U3/U4 的授权，不是对 V1 研究构建的新冻结，也不是 BIN/HEX 产品功能上线许可."
- Prompt provenance: `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U1_Data_And_Identity_Implementation_v1.0.txt`
  (15,072 bytes, 132 LF, 0 CR, sha256 `61a33d5381eca5828f8e6c979c25e58db6c2289b88fddccb37cdc71a0c484b65`, blob
  `76ecf114c46311803c30dc322fc81aa0c3c28ac9`, `cmp` identical against the delivered file at archive time) and
  `…_ERR_BUNDLE_Code_Check_And_TDD_Continuation_v1.0.txt` (10,162 bytes, 84 LF, 0 CR, sha256
  `79a91df69ff80dbd19fcb12c724b52630a9c52dda92e6a49db0dee3418ec3c0a`, blob `c49c589d8c0955612e4fa2d8ccbf24273bd020b6`,
  `cmp` re-run after archiving and still identical). Both arrived under percent-encoded names.
- §1's stop conditions and §7's ten hard stops were each checked against source before writing. **None triggered.**
  `SnapshotId::compose` needed no change, the `/1` text needed no change, no analysis input or Gate rule semantic
  moved, no schema major moved, no dependency was added, no attachment byte enters SQLite, no host path is stored or
  exported, stored facts re-read accurately, a repeated id over different content is refused, and no pre-C1 run id,
  policy fingerprint, SnapshotId or unrelated golden drifted.
- §一.3 ("不要依据前轮报告的文件行号假定代码仍未移动") was honored by re-reading every target file at this head; the
  corrections that produced are recorded in the plan's §11.3.

## 2. Change set and intent

The line figures below are `git diff --numstat 114a84e 8e2aab9`, not estimates.

| Path | Intent |
| --- | --- |
| `crates/firmwaresight-core/src/domain/gate.rs` | `KindBasis`, `GateAttachmentFact` with `canonicalized()`, `GateContext.attachments`, and the `/1`-vs-`/2` canonical rule (107 added, 1 removed) |
| `crates/firmwaresight-project/src/evidence.rs` | `ReleaseAttachment`, `AttachmentError` with the three stable codes, `observe_attachment()`, `attachment_leaf_name()` (163 added, 2 removed) |
| `crates/firmwaresight-project/src/fingerprint.rs` | the PA-1 / T-C1-01 byte-freeze test carrying the constants measured at `114a84e` (31 added) |
| `crates/firmwaresight-project/src/fixtures.rs`, `src/lib.rs` | explicit empty attachment set at the fixture (1 added); re-exports (2 added, 2 removed) |
| `crates/firmwaresight-project/tests/release_attachments.rs` | **new**, 19 tests: identity set, policy/run-id, observation, refusals, code uniqueness |
| `crates/firmwaresight-storage/migrations/0006_release_attachments.sql` | **new**, additive `gate_run_attachments` + immutability trigger |
| `crates/firmwaresight-storage/src/db.rs` | `SCHEMA_VERSION` 5 → 6, `MIGRATION_0006`, the third tuple in `MIGRATIONS` |
| `crates/firmwaresight-storage/src/gate.rs` | draft attachment rows, two pre-write refusals, in-transaction insert, ordinal read-back, `StoredGateAttachment`, extended duplicate check (179 added, 3 removed) |
| `crates/firmwaresight-storage/src/lib.rs` | re-exports for the new types (6 added, 2 removed) |
| `crates/firmwaresight-storage/tests/gate_history.rs` | +9 attachment tests, three of them §四 G's storage proofs (334 added, 3 removed) |
| `crates/firmwaresight-storage/tests/integrity_and_backup.rs` | §四 F's old-store arm: a v4 file holding a real run upgrades, gains an empty table, keeps the run readable (91 added) |
| `crates/firmwaresight-storage/tests/release_records.rs` | its `the_schema_is_at_version_five_…` pin renamed to `the_schema_is_at_this_builds_version_…`, `step_down_to_v3` now also drops `gate_run_attachments`, and its two fixtures gain the empty set (9 added, 4 removed) |
| `crates/firmwaresight-storage/tests/{compare_candidates,map_companion_persistence,history_reads}.rs`, `crates/firmwaresight-report/tests/gate_schema_contract.rs` (1–3 added each), `tests/unknown_reasons.rs` (5 added, 1 removed) | mechanical consequences: the sixth named stage in the migration lists, the sixth version where a literal pinned it, `attachments: Vec::new()` in hand-built contexts |
| `apps/desktop/src-tauri/src/release.rs` | the **only** production `GateRunDraft` construction gains `attachments: &[]` (3 added) |
| `apps/desktop/src-tauri/src/startup.rs` (2 added, 1 removed), `tests/diagnostics.rs` (7 added, 5 removed), `tests/real_artifact_intake.rs` (3 added) | hand-built v4 stores and the expected pre-migration backup name `…v4-to-v6.sqlite` |
| `04_TECH/02_DOMAIN_MODEL.md`, `04_TECH/15_STORAGE_DATABASE_BASELINE.md` | this unit's dated additions (§五) |
| `.ai/ACTIVE_TASK.md`, `10_AUDIT/SOURCE_PROMPTS/README.md`, `C1_VALIDATION/*`, `BASELINE.yaml`, `INDEX.md` | round records and pointers |

Nothing else moved. No file under `assets/`, `schemas/`, `fixtures/`, `golden/`, `templates/`, `.github/`, no
`Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml`, `DESIGN.md` or token
file was touched, and the frontend was not modified at all.

## 3. Identity evidence

**`/1` unchanged.** Three independent authorities, all measured in this build:

1. `fingerprint.rs::the_zero_attachment_canonical_text_is_byte_frozen` — the context `fingerprint.rs` already used,
   with nothing attached, still produces 1,044 bytes whose SHA-256 is
   `72d5c108e9f5b362e2f108b112389a04abfa10934207045926e1a02bd8c5da00` and whose run id is `gate-72d5c108…`; those
   constants were measured at the handover head before `attachments` existed, and the test also asserts the text
   starts with `firmwaresight-gate-input/1` and contains no `attachments[` anywhere.
2. `apps/cli/tests/p4_golden.rs` — the untouched `golden/reports/p4-release/` documents, including
   `gate-results.json` with run id `gate-a17e9861ce643da5e7061ac349793f80d1be87736ad52b2daf3433dc8cb8bc26`, are
   reproduced byte-for-byte by this build. `drift/goldens unchanged` also passes, so the golden files themselves moved
   in no way.
3. The mutation proof M4 (§12 of the plan): emitting an empty `attachments[` block makes both project guards **and
   four p4 golden tests** fail, so the freeze is enforced rather than merely asserted.

**`/2` deterministic.** `the_canonical_block_orders_by_kind_word_then_digest_and_ignores_how_they_arrive` builds one
set in two arrival orders and asserts both the canonical row list and the canonical text are equal, then pins the
exact sequence `bin sha(1)`, `bin sha(9)`, `hex sha(0)`, `hex sha(3)` with digests chosen so the kind key and the
digest key disagree. `an_exact_kind_and_digest_pair_is_bound_once_however_many_times_it_arrives` proves one `(kind,
digest)` is bound once and that the duplicate arrives at the same run id; `two_files_of_one_kind_with_different_bytes
_are_both_bound` proves the set is not a per-kind map; `redeclaring_the_kind_of_identical_bytes_changes_the_run_id`
proves `(kind, digest)` and not `digest` is the key; `the_attachment_block_carries_no_name_no_path_and_no_size` and
`a_size_claim_difference_alone_is_not_a_gate_identity_input` hold the exclusion.

## 4. Contract tests, itemized

| Test | Result | Evidence |
| --- | --- | --- |
| `T-C1-01` zero-attachment context reproduces the current canonical text and run id | **PASS** | §3 authority 1 and 2 |
| `T-C1-02` changing only an attached BIN's bytes changes the run id | **PASS** | `changing_only_the_attached_bytes_changes_the_run_id_and_not_the_snapshot` |
| `T-C1-03` the same inputs twice give the same id | **PASS** | the arrival-order test above, plus `an_empty_attachment_set_keeps_the_label_at_one_and_writes_no_block` |
| `T-C1-05`, U1 subset: re-reading stored rows reproduces the stored id | **PASS** | `gate_history.rs::a_re_read_of_the_stored_rows_rebuilds_the_same_canonical_input_and_run_id` |
| `T-C1-05`, the `extensions.attachments` JSON half | **NOT_VERIFIED — not this unit.** No report code writes that field; it is C1-U3 |
| `T-C1-16` the policy fingerprint is untouched | **PASS** | `the_projects_own_policy_fingerprint_and_the_recorded_run_id_are_untouched`: a live `LoadedProject` policy hashes to the frozen `b87aa057854d8a0089594ad9445b0626174dc0308d636dd6d6854902ae1e5e98` and the recorded run id is unchanged |
| Every pre-C1 run id unchanged | **PASS** | the 1,044-byte lock, the p4 goldens, and `integrity_and_backup.rs`'s v4-store replay, which asserts the upgraded file's stored run keeps its id, its finding and its summary |
| `SnapshotId` untouched | **PASS** | `git diff --name-only` lists no snapshot path, and the existing composition tests still pass |
| `T-C1-07` a pre-C1 stored run re-judges after its files are gone | **PASS, and it is this unit's compatibility proof** — `apps/desktop/src-tauri/tests/release_gate.rs::the_run_is_judged_from_stored_rows_after_the_files_are_gone` still passes with the new draft field. Extending it to an attached run is C1-U2 |
| `T-C1-04`, `06`, `08`–`15`, `17`–`19` | **NOT_STARTED — C1-U2/U3 scope by §9's own matrix.** No `extensions.attachments` document, no locator, no `verify_attachments`, no bundle inclusion, no UI |

## 5. Storage proofs

- Migration number: `0006` was verified free (`ls crates/firmwaresight-storage/migrations/` held `0001`–`0005`) and
  `SCHEMA_VERSION` is 6, asserted by name in `release_records.rs::the_schema_is_at_this_builds_version_and_the_release_table_starts_empty`.
- Additive only: a fresh database applies six steps, each named by its stage, and the pre-existing tables are byte-for-byte
  the migrations before them; `map_companion_persistence.rs` and `compare_candidates.rs` still fail if a step appears
  that belongs to no named stage.
- Old-store replay (§四 F): `integrity_and_backup.rs::a_v4_store_holding_a_gate_run_gains_the_attachment_table_and_keeps_the_run_readable`
  builds a real v4 file with a `PASS` run and its finding, upgrades it in place, and requires the new table to exist and
  hold nothing while the run reads back with its id, policy, finding and summary intact, then re-runs `migrate()` and
  asserts it is a no-op.
- Constraints: `the_attachment_table_refuses_an_illegal_row_at_the_boundary` re-checks the table by hand-written SQL —
  an ordinal of −1, a kind of `elf`, `map`, `ELF` and an unknown word, an uppercase digest, a 63-character digest, a
  non-hex digest, a byte size of 0 and a negative size, and a `kind_basis` of `guessed` — and allows the three kind words
  the design permits (`bin`, `hex`, `unknown`). `an_attachment_row_that_violates_a_check_leaves_no_half_stored_run`
  proves the rollback, and a raw `UPDATE` is refused by the trigger.
- Atomic write and duplicate refusal: `the_same_run_id_with_a_different_attachment_set_is_refused_and_overwrites_nothing`
  (same id, different facts → invariant, stored rows untouched), `a_draft_whose_rows_are_not_the_canonical_set_is_refused_before_any_write`
  and `a_draft_carrying_an_attachment_with_no_observed_digest_is_refused_before_any_write` (both refuse with **zero** rows
  written, checked by counting), `a_run_without_attachments_stores_no_rows_and_reads_back_an_empty_set`, and
  `storing_the_same_run_again_is_a_dedupe_not_a_duplicate` still passes as it did before this unit.
- `a_derived_basis_row_is_reported_rather_than_read_back_as_more_than_it_states`: a `derived_from_leading_bytes` row is
  an invariant on read, because this schema stores the basis word and not the sample it was derived from. The
  alternative — inventing a sample — was refused.

## 6. Observation, refusals and the stable codes

`observe_attachment(path, declared_kind)` performs, in this order: reject `Elf`/`Map` **before any filesystem access**,
`std::fs::metadata`, reject anything that is not a regular file, reject a zero-length file, the **pre-existing**
`fingerprint::file_sha256` (a `BufReader` over 64 KiB chunks, so no whole file is ever held in memory), then record
`metadata.len()`. `GuardedInput::load` (`crates/firmwaresight-artifact/src/intake.rs:99`, whose sequence ends in an immutable read of the
whole file) is deliberately not used: an attachment needs a streaming digest and no retained bytes, which is §2.3's
requirement in one sentence. The kind
basis is always `Declared`, because a person chose it; nothing samples bytes. `attachment_leaf_name()` reduces the name
through `sanitize_leaf_name`, and every refusal message carries a name or a kind, never a host path — `io_reason()` is
path-free by construction.

Registry finding (§2/§3 of the continuation): the only authority for `ERR-BUNDLE-*` is the Rust `code()` arms and
envelope literals — `crates/firmwaresight-core/src/domain/release.rs:600-601` (`6101`, `6110`),
`crates/firmwaresight-project/src/bundle.rs:171-178` (`6102`–`6109`), `apps/desktop/src-tauri/src/bundle.rs:317,367,388`
(`6111`, `6112`, `6114`), `apps/desktop/src-tauri/src/lib.rs:730,749` (`6105` reused for a plan this session dropped,
`6113`). `05_ENGINEERING/03_ERROR_MODEL.md` has **no** code table and was not pretend-edited; `04_TECH/21` has none;
`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md:474`'s `BUNDLE | 6101–6114 | 14` is a closed-stage tally and stays as written.
No registry note was invented: the codes live in the code with `code()` and `remediation()`, protected by

| Code | Variant | Remediation, verbatim |
| --- | --- | --- |
| `ERR-BUNDLE-6115` | `AttachmentError::Unreadable` | choose the file again, or close the program that holds it open |
| `ERR-BUNDLE-6116` | `KindNotAttaching` | analyze the ELF on the Analyze page and attach the MAP there; a release attachment is a BIN, an Intel HEX or an unidentified file |
| `ERR-BUNDLE-6117` | `AttachmentError::Empty` | attach the built image; an empty file ships nothing to verify |

Regression: `the_three_refusals_are_three_distinct_codes_each_with_its_own_next_step` and
`the_new_codes_take_no_number_that_an_existing_refusal_already_uses`, which calls the real `code()` on eleven
constructed `BundleError`/`ReleaseError` values, asserts all eleven land inside the recorded 14-entry `6101`–`6114`
family, and asserts none of `6115`–`6117` collides with it. **E-4 is C1-U2's and is not numbered here** — §4 says so, and
no placeholder constant exists anywhere in the workspace.

## 7. Gate and counts, measured

- `python scripts/check.py` — **17 of 17 PASS, no SKIP, no FAIL**. Four runs are on the record and each names the
  state it measured: `target/c1_u1_check_pre_docs.txt` before the documents of this unit were written,
  `target/c1_u1_gate_final.txt` at `8e2aab9`'s committed bytes (the run whose figures are quoted above),
  `target/c1_u1_gate_successor.txt` at the first staged state of the docs-only successor, and
  `target/c1_u1_gate_successor_final.txt` at that successor's final bytes. All returned 17 of 17. A gate run is a
  measurement of one byte-state, so a later edit retires the run that preceded it rather than leaving it as evidence
  for something it never saw.
- Rust: `cargo test --workspace` → **900 passed, 0 failed across 48 result lines** (`target/c1_u1_final_workspace.txt`).
  The handover recorded 870, so this unit adds 30 tests. Note for the next reader: `check.py`'s own log sums to a larger
  number than 900 because its `drift/ipc bindings` step runs a second `cargo test` to regenerate the ts-rs bindings; the
  single-invocation figure above is the one to compare against 870.
- Frontend: **295 tests in 9 files**, typecheck, lint and build PASS — all unchanged, because this unit touched no UI path.
- `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` exits 0.
- `deny/cargo-deny`: advisories, bans, licences and sources all ok — no dependency moved.

## 8. Boundary: what this unit does not make true

- **No person can attach a file.** There is no UI control, no CLI `--attach`, no IPC command and no file chooser change;
  the only production draft in the workspace binds `attachments: &[]`, and the Project builder's set is empty by
  construction. `BIN_HEX_RELEASE_ATTACH = DESIGN_APPROVED / NOT_USER_AVAILABLE`.
- **`artifacts.required = ["bin"]` and `["hex"]` still evaluate to BLOCK.** The rules do not read the attachment set, so
  a policy that requires a BIN still cannot be satisfied — `an_attachment_still_satisfies_nothing_because_the_rules_do
  _not_read_it_yet` asserts this rather than letting it be inferred. `a_bound_bin_attachment_moves_the_label_to_two_and_
  binds_its_digest` proves the binding is identity, not satisfaction.
- **A bundle still cannot ship a BIN or HEX file.** No `attachment:` locator, no `verify_attachments`, no
  `BundleInputChecks` membership, no preview/export staleness rule, no collision handling and no E-4. Test suite
  `T-C1-10`/`T-C1-14`/`T-C1-15` remain U2's.
- **The external documents are byte-identical.** `gate-results`, `analysis.json`, `diff.json` and the release bundle
  gained no field and no new output; `drift/goldens unchanged` passes. This report therefore does **not** claim BIN or
  HEX is available in a Release, and `BIN_HEX_ANALYSIS = UNSUPPORTED` with no record checksum, address span, section,
  symbol or object attribution anywhere.
- **NOT_RUNTIME_VERIFIED**: nothing was installed, no real user store was migrated, no screenshot or session exists for
  this round. The storage proofs run against real SQLite files created by the tests, which is stronger than a mock and
  weaker than an installed upgrade over the owner's store.
- The new CI artifact from this round's push is **not** the V1 cohort. The cohort stays `11573661113` at `41bb6a36`
  (`9a51e86a…c87d93`), untouched, and V1 stays paused at recruitment with 0 eligible external sessions and M1–M6
  `NOT_MEASURED`.

## 9. Residual issues, and the recommendation for C1-U2 (advice only, not executed)

1. The `/2` grammar is reachable in code and unreachable in product. Until U2, an attached run can only exist in tests;
   that is the intended boundary, but it means the `/2` text has no production witness.
2. `KindBasis::DerivedFromLeadingBytes` has no producer and, by design, no readable storage form. If C1-U2 or U3 decides
   derived kinds are in scope, that is an ADR-level conversation, not a column addition — this round deliberately
   refused to invent a sample on read.
3. The `6115`–`6117` codes live in code only, because no markdown registry exists to hold them. If the next error-model
   round creates one, these three move into it verbatim; until then `05_ENGINEERING/03_ERROR_MODEL.md` still has no
   table, and that is a documented fact rather than a defect this unit was authorized to fix.
4. Recommendation, **not started**: C1-U2 should take the six-link stale/export proof of `04_TECH/28` §6 one link per
   test, keep `attachment:` locators out of any evidence class stronger than Declared for kind, and preserve the
   `T-C1-07` extension it needs (a run with attachments re-judged after its files are gone) as a new test rather than a
   modification of the pre-C1 one. E-4's number should be allocated by the same registry-first procedure this round used,
   and E-4 is the only remaining `61xx` this author knows of that has no number.
5. `apps/desktop/ui/src/help.test.tsx:616` and `apps/desktop/src-tauri/src/startup.rs:334` still carry `…v4-to-v5.sqlite`
   example strings. They assert nothing about naming, so they were left alone; the next UI round may refresh them.

## 10. Ending state

`C1_PRODUCT_DIRECTION = OWNER_APPROVED`, `C1_ADR_AND_DESIGN_FREEZE = COMPLETE`, `C1_U1_DATA_AND_IDENTITY = COMPLETE`,
`C1_U2_U3_U4 = NOT_AUTHORIZED / NOT_STARTED`, `BIN_HEX_ANALYSIS = UNSUPPORTED`,
`BIN_HEX_RELEASE_ATTACH = DESIGN_APPROVED / NOT_USER_AVAILABLE`, `V1 = IN_PROGRESS / RECRUITMENT_READY` with 0 eligible
external sessions and M1–M6 `NOT_MEASURED`, `COHORT_BUILD = 11573661113 at 41bb6a36` unchanged,
`B1 / RC / GA = NOT_AUTHORIZED`, `ACTIVE_TASK = NONE`.

The round stops here. C1-U2 needs its own authorization.

## 11. Self-corrections carried by the docs-only successor

The product commit `8e2aab9` wrote three numbers about itself that its own files contradict, and each is answered by a
dated successor rather than by an amend, because §Step 7 of the authorization forbids amend and force and `AGENTS.md` 9
puts a pushed-history rewrite behind a human decision — the same route `114a84e` took for the freeze round's 186-to-185
slip.

1. `gate_history.rs` gained **9** attachment tests, not 10. The unit's +30 decomposes as 19 in the new
   `release_attachments.rs`, 9 in `gate_history.rs`, one v4-store replay in `integrity_and_backup.rs` and the `/1`
   byte-freeze test in `fingerprint.rs`;
   `git diff 114a84e HEAD | grep -c '^+[[:space:]]*#\[test\]'` returns 30 and 900 − 870 confirms the total. The
   `[[:space:]]*` in that pattern is load-bearing — without it the same diff returns 29, because the `fingerprint.rs`
   test is indented inside its existing `mod tests` and the other 29 attributes sit at column 0, so a command quoted in a
   document has to be the one that actually produces the number next to it. The **totals were never wrong** — only the
   breakdown was — which is why a sum alone does not audit a claim.
2. The change table in §2 quoted estimated line counts. It now carries `git diff --numstat 114a84e 8e2aab9` for every
   path it names, so the next reader has the command rather than my arithmetic.
3. §7 quoted one gate log for a round that ran three. The names and the state each measured are listed there now.

Nothing in this section changes a verdict, a count that the gate prints, or a claim about the product: the successor
touches three documents, `DIRECTORY_TREE.txt` regenerated byte-identical because no path moved, and `SHA256SUMS`
re-hashes the changed blobs.

A fourth defect belongs here because it was caught before it could be committed rather than after. Midway through the
successor's edits I ran the closeout order but wrote
`python scripts/generate_baseline_artifacts.py sums` **without the `> SHA256SUMS` redirection** the generator's own
docstring puts in steps 4 and 6 — it emits to stdout by design, so that the manifest is never written in text mode on
Windows. Nothing failed: the script printed 784 rows, the shell discarded them, and `git add SHA256SUMS` staged a file
still carrying the digests of the *previous* index. The independent verifier then reported
`index blob mismatch` on exactly the two paths edited since that manifest was regenerated, and `RESULT FAIL`. This is
the control ADR-0029 installed for this specific mistake — the header says the guard exists because the manifest "has
made this mistake twice" — and it worked: a stale manifest could not pass as a fresh one. The correction is the
redirection, plus re-reading the generator's documented order instead of reconstructing it from memory.
