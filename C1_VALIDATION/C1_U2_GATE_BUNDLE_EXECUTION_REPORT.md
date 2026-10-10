---
title: "C1-U2 Gate and Bundle attached byte consistency execution report"
doc_id: "FS-C1-U2-002"
product: "FirmwareSight"
unit: "C1_U2_GATE_BUNDLE_ATTACHED_BYTE_CONSISTENCY"
status: "REPORT"
owner: "Engineering"
authority: "10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U2_Gate_Bundle_Six_Link_Consistency_v1.0.txt + ADR-0030 + 04_TECH/28 + AGENTS.md 2, 3, 6, 7, 8, 10"
written_at: "2026-10-09, before the product commit exists"
last_updated: "2026-10-09"
---

# FirmwareSight C1-U2 Gate + Bundle Execution Report

Nineteen parts, in the order §16 of the authorization lists them. Every figure is printed by a command named beside it;
where a claim could not be measured, the row reads `NOT_VERIFIED` and says what would have measured it. Parts 13 and 14
are deliberately empty of hashes: this file is committed with the product commit it reports, and a commit cannot contain
the run its own push produces. §13 of the authorization routes that read-back to the successor.

## 1. Preflight: authority, tree, and the archive — with its provenance weakened, stated

| item | command | measured |
| --- | --- | --- |
| HEAD | `git rev-parse HEAD` | `1b33bbcb52b441efc4e63d7b6c91e829dc232d76` |
| local `origin/main` | `git rev-parse origin/main` | the same value |
| remote ref | `git ls-remote origin refs/heads/main` | the same value |
| tree | `git status --short` | empty |
| worktrees | `git worktree list` | one entry, this directory |
| §0's local marks | re-read at those bytes | migrations `0001`–`0006` with `pub const SCHEMA_VERSION: i64 = 6;`, `.ai/ACTIVE_TASK.md` opening `NONE.`, the frozen V1 cohort `11573661113`, `BIN_HEX_ANALYSIS = UNSUPPORTED` |

**The archive of this prompt is a reconstruction and is weaker than every archived prompt before it.** §1 asked for the
delivered file "with actual source file bytes and both transport" digests. The delivered path
`C:\Users\16429\Downloads\FirmwareSight_C1_U2_Gate_Bundle_Six_Link_Consistency_v1.0.txt` was **already absent** when the
archive was written: `ls -l` of that exact path returned `No such file or directory`, a folder listing showed six
unrelated entries from another project, `find /c/Users/16429 -maxdepth 4 -iname '*Six_Link*' -o -iname '*C1_U2*'` returned
nothing, and a content grep of the CLI temp cache for `CANONICAL_UNIT: C1_U2` returned nothing. A content grep of Desktop
and Documents was attempted and timed out, so it is evidence neither way. The archived bytes therefore come from this
session's first `Read` of that path. What is measured: `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U2_Gate_Bundle_Six_Link_Consistency_v1.0.txt`
is 36,363 bytes, 632 `LF`, 0 `CR`, sha256 `30fa19579f5d676f65256f51cabd8a2d3e41866d85b1e593f1e75088db15ae59`, and its
stage-0 blob `8779dc528b1363010d469212f321d528b5ea2491` has that same digest (`cmp` of the index blob against the working
copy: identical; `git check-attr text eol --` answers `text: set`, `eol: lf`). So **archived bytes = staged bytes**; what
is *not* claimed is any delivered-transport digest — which is what §1's own words require ("do not hand-edit checksums
or … invent a delivered digest"). Two byte properties of a reconstruction are unprovable: that the delivery was LF-only,
and that it ended with exactly one newline (the reader numbers the empty trailing piece of a newline-terminated file, so
that artifact line was removed and `wc -l` returns 632). If the owner re-supplies the original, a `cmp` result belongs in
that register entry.

No destructive git form was used at any point in the round, per §1's list: no `reset --hard`, no `clean -fd`, no `stash`,
no rebase or amend of published history, no `--force` or `--force-with-lease` in any spelling.

## 2. The U1 boundary, proven from source rather than from the handover's wording

§10 forbids trusting a prior round's line numbers, so each premise was re-read at `1b33bbc`. U1's boundary was exactly
where its report said it was, and it was proven by making it fail:

- `crates/firmwaresight-project/src/evidence.rs` — `build_context` produced an empty attachment set for every caller.
- `crates/firmwaresight-core/src/domain/gate.rs` — `rule_required_artifacts` and `rule_artifact_hashes` read
  `self.artifacts` only, so a `required = ["bin"]` policy could only answer BLOCK.
- U1's own lock test `an_attachment_still_satisfies_nothing_because_the_rules_do_not_read_it_yet` asserted that gap.
  **It was converted, not deleted**: `git diff` shows one removed `fn` name and its replacement
  `a_bound_bin_attachment_satisfies_a_required_bin_and_an_unbound_one_still_blocks`, which now requires the positive
  binding *and* retains the missing-attachment negative. `git diff -U0 crates/ apps/ | grep '^- *fn [a-z_]*()'` returns
  that one line and nothing else: no other test in the workspace was removed.

## 3. The plan, and the path audit that authorized it

`C1_VALIDATION/C1_U2_GATE_BUNDLE_PLAN.md` was written before any source file was edited, as §3 orders ("Write
C1_VALIDATION/C1_U2_GATE_BUNDLE_PLAN.md BEFORE CODE"). §14 of that file is its as-executed reconciliation and carries
every place the round differed from its own prediction.

**Touched, in §11's expected list:** `crates/firmwaresight-core/src/domain/gate.rs`,
`crates/firmwaresight-project/src/evidence.rs`, `crates/firmwaresight-project/src/bundle.rs`,
`crates/firmwaresight-project/src/bundle/tests.rs`, `crates/firmwaresight-project/tests/bundle_builder.rs`,
`crates/firmwaresight-project/tests/release_attachments.rs`.
**Touched under §11's "relevant … release/verifier tests" and "narrow storage read-back test":**
`crates/firmwaresight-report/tests/gate_schema_contract.rs`,
`crates/firmwaresight-report/tests/release_schema_contract.rs`,
`crates/firmwaresight-storage/tests/gate_history.rs`.
**Not in §11's list, not forbidden, necessary, and disclosed as a scope note (plan §10):**
`crates/firmwaresight-project/src/lib.rs` (re-exports), `crates/firmwaresight-report/src/release.rs` (the only place a
`release-manifest.json` is composed; re-serializing that JSON anywhere else would have moved every bundle's bytes,
including the P4 golden).
**Forbidden by §11 and untouched:** `apps/desktop/ui/**`, `apps/desktop/src-tauri/**`, `apps/cli/**`, `migrations/**`,
`schemas/**`, `golden/reports/p4-release/**`, unrelated `fixtures/**`, `SnapshotId::compose`, the ELF parser,
`build_snapshot`, analysis and diff identity, every lockfile, `.github/**`, `scripts/**`, `toolchains/**`,
`assets/design-tokens.json`, updater / signing / licensing. `git diff --cached --name-only` at the final staged state
lists **27 paths**, and filtering that list against `^(apps/|assets/|schemas/|migrations/|golden/|fixtures/|templates/|
scripts/|\.github/|toolchains/)` plus the names `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`,
`tauri.conf.json`, `deny.toml`, `rust-toolchain.toml` and `DESIGN.md` returns nothing.

**A stronger scope proof than a name filter, and the reason §11's list can be answered in one line.** `git write-tree`
over the staged index, compared entry by entry with `HEAD`'s tree `bd97f2c6d40036f99d94b6b081acc0e58dad90bb` (the tree of
the handover head `1b33bbc`): every forbidden entry has the **same OID** in both, so no byte inside one can have moved.
The staged root tree itself is deliberately not quoted here — every documentation edit in this round moves it, while the
entries below do not. `apps` `5cf38bbe…`, `apps/desktop/ui` `3ec185a4…`, `apps/desktop/src-tauri` `27802db8…`,
`apps/cli` `e7429b83…`,
`assets` `da1f6d16…`, `assets/design-tokens.json` `b818d113…`, `schemas` `a8e7e542…`, `golden` `99352bef…`,
`golden/reports/p4-release` `86683cd7…`, `fixtures` `0fcc2114…`, `templates` `d34358ec…`, `scripts` `76c1e1cb…`,
`.github` `174bd58f…`, `crates/firmwaresight-storage/migrations` `e4074d96…`, `Cargo.toml` `0d096b38…`,
`Cargo.lock` `c5f6a38b…`, `deny.toml` `913072c4…`, `rust-toolchain.toml` `2c2595fc…`, `DESIGN.md` `5961d40b…`.
Two of those rows carry the identity promises §10 names: `build_snapshot.rs` — which owns `SnapshotId::compose` — is
`46944151…` in both trees, and `crates/firmwaresight-project/src/fingerprint.rs`, which holds U1's `/1` byte-freeze
test, is identical, so the frozen-label assertion is proved unchanged by the same mechanism that proves the goldens are.
The only file that moved under `crates/firmwaresight-core` is `domain/gate.rs` (`ab98a324…` → `41d4427d…`).

## 4. The Gate's two evidence classes, and what the rules now say

§5's contract, in `crates/firmwaresight-core/src/domain/gate.rs`:

| rule | new reading | kind separation |
| --- | --- | --- |
| `artifacts.required` | `elf` / `map` satisfiable **only** by analyzed snapshot rows; `bin` / `hex` **only** by `context.attachments`; `unknown` by neither (`ADR-0030` D-3) | an attachment row is not an artifact row, and the test `an_attachment_never_satisfies_a_snapshot_only_kind` is the mutation-killed proof of that direction |
| `artifacts.hashes` | both classes counted, each cited by its own locator: `artifact:<kind>:<sha256>` and `attachment:<kind>:<sha256>`; a row with no observed digest blocks | the summary carries its limitation with the count, as the RED output below quotes |

The two new sentences a caller reads, taken from the RED run's own `left:` / `right:` pair — which is also §5's "no fake
claim" requirement, shown as text rather than asserted about it:

```text
All 3 input(s) carry a SHA-256 digest: 1 analyzed artifact(s) plus 2 release attachment(s)
whose bytes were hashed and whose origin or build provenance was not verified.
```

Eight new tests in `gate.rs`'s module: `a_bound_bin_and_hex_attachment_pair_satisfies_their_own_requirements`,
`a_required_bin_with_no_attachment_still_blocks`, `an_attachment_never_satisfies_a_snapshot_only_kind`,
`an_unknown_attachment_satisfies_no_required_kind`,
`the_hashes_rule_counts_both_classes_and_cites_each_by_its_own_scheme`,
`an_attachment_row_without_a_digest_blocks_the_hashes_rule`,
`the_two_artifact_rules_say_exactly_what_they_said_when_nothing_is_attached`,
`attaching_a_file_never_moves_the_unknown_evidence_rule`. The last two are §5's two prohibitions: an attachment never
re-describes the no-attachment case, and it never moves `unknown.count` (`ADR-0030` D-7).

No new rule category, no new state, no new severity, no UI, no CLI, no IPC (§5's F). `evidence.unknown_review` is
untouched by construction and has a test.

## 5. The journey a caller actually takes

```text
library caller
  └─ evidence::AttachmentSelection { path, declared_kind }            (project)
      └─ evidence::observe_attachment(path, kind)                     (U1: stat → regular file → non-empty → streamed SHA-256 → length)
          └─ evidence::build_context_with_attachments(&GateRunRequest, &rows)   (evidence.rs:538)
              ├─ GateContext { artifacts, attachments, … }            (core)
              ├─ fingerprint::run_id(&context)                        (U1's /2 canonical text)
              │   └─ if request.selected_run_id differs → BundleError::GateContextChanged   (bundle.rs:757, L-3)
              └─ context.evaluate(&run_id) → findings with attachment: locators
                  └─ bundle::observe(request, selections)             (the one sanctioned path; there is no second)
                      └─ InputChecks { sources: [SourceRow { class, … }] }
                          ├─ prepare_with_attachments → BundlePlan    (bundle.rs:552)
                          └─ BundlePlan::publish_with_attachments     (bundle.rs:420)
                              ├─ observe() again: recompute, require_packageable
                              ├─ checks.compare_with(&now)            (L-5: 6103 or E-4 6118)
                              ├─ verify_sources + verify_attachments  (L-4, before any destination write)
                              ├─ artifacts/<leaf> + SHA256SUMS + files[]
                              └─ ReleaseManifestDto::from_parts_with_attachments(..., disclosure)
```

`build_context`, `prepare`, `publish` and `from_parts` each delegate to their sibling over `&[]`, so every existing
caller — `apps/cli/src/main.rs`, `apps/desktop/src-tauri/src/release.rs`, storage, and the P4 golden reproducer — keeps
its exact behaviour. **The one sanctioned path (§5's C) is one function with one parameter list, not one function with a
field**: see part 11 for why, and for the deviation the Architect is asked to accept or overturn.

## 6. The six links, each with its positive and its negative

| link | what binds it | positive test (file) | negative control |
| --- | --- | --- | --- |
| **L-1** the verdict's subject | `observe()` builds the context from the rows it will ship | `a_release_that_requires_bin_and_hex_passes_only_when_both_are_attached_and_bound` (bundle_builder) + `a_bound_bin_and_hex_attachment_pair_satisfies_their_own_requirements` (gate) | `a_required_bin_with_nothing_attached_produces_no_bundle_at_all` (bundle_builder) + `a_required_bin_with_no_attachment_still_blocks` (gate) |
| **L-2** identity | U1's `/2` block, now reachable | `attaching_a_file_moves_the_run_id_and_leaves_the_snapshot_alone` | `the_same_bytes_declared_as_a_different_kind_move_the_run_id`; U1's `/1` byte-freeze in `fingerprint.rs` stays in force; `a_size_claim_difference_alone_is_not_a_gate_identity_input` (U1) |
| **L-3** stored-run recompute | `observe()` recomputes `run_id` and refuses a mismatch before anything is read for shipping | `a_stored_run_that_no_longer_recomputes_is_refused_before_any_write` | `a_stored_run_that_bound_attachments_rejudges_from_its_own_rows` (storage, real SQLite: rows persisted, files never existed, canonical text and verdict reproduced) + the run row is immutable under one id (U1's trigger, re-run green) |
| **L-4** membership and bytes | `observe_attachment`'s stat → regular-file → non-empty → streamed-SHA-256 → length chain, re-run by publish, plus `verify_attachments`: a sibling of `verify_sources` that does **not** re-hash but refuses a non-attachable kind, a non-`declared` basis, a missing observed digest, or a row absent from `context.attachments` as a `(kind, digest)` pair | `every_shipped_attachment_is_a_row_the_judged_context_carries` | `an_attachment_the_judged_context_does_not_carry_is_refused` (white-box, `bundle/tests.rs`) + `the_attachment_rows_a_context_binds_are_the_facts_the_observation_produced` + `a_directory_or_an_empty_file_offered_as_an_attachment_is_refused_before_any_write` |
| **L-5** preview / export | one typed `SourceRow` set in `InputChecks.sources`; `compare_with` classifies; `changed_leaf` keyed by path | `a_changed_attachment_between_preview_and_publish_is_refused_by_name` and `an_attachment_that_changed_size_after_the_preview_is_refused_too` (6103) | `an_attachment_added_between_preview_and_publish_names_itself_as_added`, `…_removed…`, `…_renamed…` (E-4, three named cases), `a_set_change_is_named_as_one_and_never_as_a_file_whose_bytes_moved`, `a_snapshot_row_is_never_read_as_a_set_change`, and the precedence case `a_withdrawn_attachment_the_policy_requires_is_refused_by_the_gate_first` |
| **L-6** self-verifying output | `artifacts/<leaf>` + `SHA256SUMS` + `files[]` + `extensions.attachments` + `verify_bundle`'s two-direction check | `a_bundle_that_ships_attachments_verifies_and_re_derives_its_release_id` and `a_relocated_bundle_with_attachments_verifies_with_its_sources_deleted` | `tampering_with_a_shipped_attachment_breaks_the_bundle_it_sits_in`, `an_attached_file_the_manifest_never_disclosed_is_refused`, `a_disclosed_attachment_the_bundle_does_not_ship_is_refused`, `a_disclosure_that_overstates_itself_is_refused`, `a_shipped_file_claimed_as_both_analyzed_and_attached_is_refused`, `an_attachment_free_bundle_carries_no_disclosure_key_at_all` |

Preview and publish call the **same** `observe()`, the **same** `verify_attachments` and the **same** `compare_with`
(§6's requirement), and §10's "no second rule" is held by that structure: there is no export-only validator.

## 7. E-1/E-2/E-3 preserved; E-4 allocated at 6118 with a live collision proof

`04_TECH/28` §4.3's four refusals, as a caller reaches them through the bundle surface:

| id | variant | code | bridged how | remediation direction |
| --- | --- | --- | --- | --- |
| E-1 | `AttachmentError::Unreadable` | `ERR-BUNDLE-6115` | `BundleError::Attachment(#[from] AttachmentError)` → `code()` returns `error.code()` | fix the file, then re-preview |
| E-2 | `KindNotAttaching` | `ERR-BUNDLE-6116` | same | an ELF or MAP is an analysis input, not an attachment |
| E-3 | `Empty` | `ERR-BUNDLE-6117` | same | a zero-byte file is not evidence |
| E-4 | `BundleError::AttachmentSetChanged { name, change }` | `ERR-BUNDLE-6118` | new variant, `change` in `added` / `removed` / `renamed` | "prepare the bundle again against the files this release actually ships: a preview names the bytes it judged, and a set that moved is not that preview" |

Vacancy was measured before the number was taken, as §7 demands: every `code()` implementation in the workspace was
inventoried (`core/domain/diff.rs`, `core/domain/release.rs`, `project/bundle.rs`, `project/error.rs`,
`project/evidence.rs`), `git grep -rhoE "ERR-BUNDLE-[0-9]{4}" | sort | uniq -c` shows 6101–6117 occupied, and `git grep
-n "6118"` returned **nothing** anywhere in the repository — no code, no document, no schema. Because there is still no
markdown registry, the allocation is proved by execution rather than by a list: `every_refusal_a_caller_can_branch_on_has_a_code_and_a_remediation`
and `the_new_codes_take_no_number_that_an_existing_refusal_already_uses` construct a real `AttachmentSetChanged` and each
bridged `AttachmentError`, call the live `code()`, and assert 6118 collides with none of 6101–6117 while E-1/E-2/E-3 still
answer 6115/6116/6117 **through the bundle surface** (`the_bundle_surface_reports_each_attachment_refusal_as_its_own_code`).

The bridge is typed: no catch-all string, no fabricated `Unknown` finding, and no host path in any envelope. E-4's
`{name}` is a sanitized leaf (`sanitize_leaf_name`), and §7's "remediation says re-preview" is the sentence above.
`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md:474`'s closed-stage `6101–6114` tally was left as that round wrote it.

## 8. M1–M16: the multiplicity, naming and path-safety matrix

Every row runs against **real regular files in a real temporary project** and, where storage is involved, a **real
SQLite file** (§8's no-mocked-observation requirement).

| row | test | result |
| --- | --- | --- |
| M1 one BIN + one HEX, both bound and shipped | `a_release_that_requires_bin_and_hex_passes_only_when_both_are_attached_and_bound` | PASS |
| M2 same kind/digest chosen twice | `the_same_path_offered_twice_ships_once`, `identical_bytes_at_two_paths_are_one_identity_row_and_two_shipped_files` | PASS — one identity row (U1's dedupe), one shipped file for one path, two files for two paths |
| M3 same kind, different bytes | `two_attachments_of_one_kind_with_different_bytes_both_bind_and_both_ship` | PASS |
| M4 leaf colliding with the ELF / MAP | `an_attachment_named_like_the_elf_is_disambiguated_and_both_survive` | PASS — `bundle_names` renames **both** halves (`bin-<sha8>-firmware.elf` beside `elf-<sha8>-firmware.elf`), each file keeping its own bytes |
| M5 leaves differing only by case | `two_attachment_leaves_differing_only_in_case_are_named_apart_by_content` (portable, two directories) and `one_host_file_offered_under_two_cases_is_refused_rather_than_shipped_twice` (`#[cfg(windows)]`) | PASS as run. **On NTFS the two selections name one file**, so the case-folded pair is refused through `DuplicatePath` → `ERR-BUNDLE-6109`; the message's usefulness is finding 1 in `04_TECH/28` §14 |
| M6 rename, bytes intact | `a_renamed_attachment_keeps_the_run_id_and_moves_the_release_id` + M6's identity half | PASS |
| M7 same bytes, different declared kind | `the_same_bytes_declared_as_a_different_kind_move_the_run_id` | PASS — no reclassification, both facts kept |
| M8 deletion between judging and preview, and between preview and export | `a_stored_run_that_no_longer_recomputes_is_refused_before_any_write`, `an_attachment_removed_between_preview_and_publish_names_itself_as_removed`, `a_withdrawn_attachment_the_policy_requires_is_refused_by_the_gate_first` | PASS — with the precedence stated: a *required* attachment that goes answers 6101, because the verdict is recomputed first |
| M9 bytes or size mutate between preview and publish | `a_changed_attachment_between_preview_and_publish_is_refused_by_name`, `an_attachment_that_changed_size_after_the_preview_is_refused_too` | PASS — refusal before any write, and the destination folder is proved to stay empty |
| M10 added / removed members | `an_attachment_added_between_preview_and_publish_names_itself_as_added`, `…_removed…`, `…_renamed…` | PASS, three named E-4 cases |
| M11 directory, missing, zero byte, traversal, absolute path, **symlink** | `a_directory_or_an_empty_file_offered_as_an_attachment_is_refused_before_any_write`, `a_selection_that_tries_to_leave_the_project_root_is_refused_without_naming_it`, `an_elf_offered_as_an_attachment_is_refused_as_an_analysis_input`, plus U1's `a_directory_or_a_missing_file_is_refused_as_unreadable_without_naming_its_host_path` | PASS **except the symlink half, which is NOT_VERIFIED**: `04_TECH/28` §4.4 states symlink refusal for bundle *contents* and is silent on a symlink offered as a *source*, and `observe_attachment` follows one. No test was written, because writing one would have chosen the rule by accident. **Platform record:** the `#[cfg(windows)]` case ran on this host and cannot run on CI's ubuntu job; no `#[cfg(unix)]` case exists, so no Unix-only path ran anywhere; every other row is platform-neutral and ran on both |
| M12 large attachment, streaming, never `GuardedInput::load` | `a_large_attachment_is_hashed_and_shipped_by_the_same_streaming_read` (1 MiB through `fingerprint::file_sha256`'s 64 KiB buffer) | PASS at 1 MiB; **>512 MiB NOT_VERIFIED** — no such fixture was created, so that size is unmeasured rather than assumed safe |
| M13 `required = ['hex']` with non-HEX bytes | `a_declared_hex_whose_bytes_are_not_hex_is_shipped_and_disclosed_as_declared` | PASS — satisfied as raw bytes of a declared kind, with the limitation sentence and **no** claim that the content is valid Intel HEX |
| M14 `unknown` satisfies neither kind | `an_unknown_attachment_satisfies_no_bin_requirement` (project) + `an_unknown_attachment_satisfies_no_required_kind` (core) | PASS |
| M15 empty set, old golden byte-identical | `an_attachment_free_bundle_carries_no_disclosure_key_at_all`, `a_release_with_nothing_attached_writes_no_disclosure_key_at_all`, and `drift/goldens unchanged` PASSing with `apps/cli/tests/p4_golden.rs` untouched | PASS |
| M16 no installed artifact required | by construction: the whole matrix is library-level, and `--only package` was still run for the round's §12 list | PASS (this is code acceptance, not installed acceptance — see part 17) |

## 9. RED, GREEN, mutations, and the Rust count with its names

**RED, on unmodified U1 code at `1b33bbc`, for §10's five behaviours.**

1. *`artifacts.required` satisfied by kind mapping* and 2. *hashes plus `attachment:` locators* — one run,
   `cargo test -p firmwaresight-core --lib domain::gate::tests`, printing
   `test result: FAILED. 36 passed; 4 failed` with the four names
   `a_bound_bin_and_hex_attachment_pair_satisfies_their_own_requirements`,
   `the_hashes_rule_counts_both_classes_and_cites_each_by_its_own_scheme`,
   `an_attachment_row_without_a_digest_blocks_the_hashes_rule`,
   `an_unknown_attachment_satisfies_no_required_kind`, and with the old behaviour quoted in the failure text itself:
   `left: Block / right: Pass` over the summary `Required artifact kind(s) missing from the snapshot: bin.` and
   `left: "All 1 snapshot artifact(s) carry a SHA-256 digest."`.
3. *an attachment-bearing request reaching real `observe` / `prepare` / `publish* — recorded as a compile failure in the
   test crate, exactly as the plan said it would be rather than dressed up as an assertion:
   `cargo test -p firmwaresight-project --test release_attachments` printed
   `error[E0599]: no variant named `AttachmentSetChanged` found for enum `BundleError`` at
   `tests/release_attachments.rs:595` and three `error[E0277]: the trait bound `BundleError: From<AttachmentError>` is
   not satisfied` at `:628`, `:635`, `:641`.
4. *membership refusal including a stale-preview set change* — shown by holding the classifier back in `bundle.rs` with
   a `RED-STAGE(C1-U2 L-5)` marker and running the E-4 tests against the old pairwise blame:
   `left: "ERR-BUNDLE-6103" / right: "ERR-BUNDLE-6118"` for the changed-and-removed cases and
   `left: "ERR-BUNDLE-6101"` for the withdrawn-required case, i.e. the old code called a set change a file whose bytes
   moved. The marker was then removed; `grep -rn "RED-STAGE"` over the tree now returns **nothing**.
5. *unmodified no-attachment behaviour* — GREEN before the rest, as §10 requires: `/1`'s 1,044 bytes and
   `72d5c108…c5da00`, the P4 run id `gate-a17e9861…bc26`, the policy fingerprint `b87aa057…e5e98` and the goldens all
   held at `1b33bbc` before any edit, and were re-asserted after (part 10).

**GREEN: 53 new `#[test]` items, measured from the staged diff**, not recalled:

```text
34 crates/firmwaresight-project/tests/bundle_builder.rs
 8 crates/firmwaresight-core/src/domain/gate.rs
 6 crates/firmwaresight-report/tests/release_schema_contract.rs
 3 crates/firmwaresight-project/src/bundle/tests.rs
 1 crates/firmwaresight-project/tests/release_attachments.rs
 1 crates/firmwaresight-storage/tests/gate_history.rs
= 53
```

counted by `git diff -U0 | awk '/^diff --git/{f=$0} /^\+[[:space:]]*#\[test\]/{c[f]++}'`. The `1` in
`release_attachments.rs` is the *added* refusal-code bridge test; the converted boundary test changes only its `fn` line,
which is why it is not counted as an addition.

**Mutations, seven, each killed by a named real test and each restored byte-identically.**

| # | mutation | killed by | restored digest |
| --- | --- | --- | --- |
| 1 | `gate.rs`: let an attachment row satisfy an `elf`/`map` requirement (`|| attachments.iter().any(...)` added to the snapshot arm) | `an_attachment_never_satisfies_a_snapshot_only_kind` — `test result: FAILED. 1 passed; 1 failed` at `gate.rs:2293` | `0c62a998…c091`, equal to the pre-mutation value |
| 2 | `bundle.rs`: bypass `verify_attachments`' membership test (`if false {`) | `an_attachment_the_judged_context_does_not_carry_is_refused` — `FAILED` at `bundle/tests.rs:509` with the unbound row printed | `192ae541…1873` |
| 3 | `bundle.rs`: disable the rename identification (`&& was.digest == is.digest && was.size == is.size` → `&& false`) | `an_attachment_renamed_between_preview_and_publish_names_itself_as_renamed` — `77 passed; 1 failed` | `192ae541…1873` |
| 4 | `bundle.rs`: `changed_leaf` back to positional blame | `a_snapshot_row_is_never_read_as_a_set_change` — `left: "firmware.elf" / right: "a shipped artifact"` | `192ae541…1873` |
| 5 | `report/release.rs`: remove `#[serde(skip_serializing_if = "Vec::is_empty")]` from the disclosure | `a_release_with_nothing_attached_writes_no_disclosure_key_at_all` (`32 passed; 1 failed`) and `an_attachment_free_bundle_carries_no_disclosure_key_at_all` (`77 passed; 1 failed`) — the old bundles' bytes are what this protects | `d6634bc8…6815` |
| 6 | `bundle.rs`: the set-change classifier returns `None` for every pair | all three E-4 tests at once — `1 passed; 3 failed`, the surviving one being the byte-change case, which correctly stays 6103 | `192ae541…1873` |
| 7 | `report/release.rs`: `#[serde(rename = "kind_word")]` on the disclosure's `kind` | `an_attachment_disclosure_names_the_entry_it_ships_and_states_its_limits` — `left: Null / right: String("bin")`, `0 passed; 1 failed`, which is the proof that the key-set lock added for T-C1-18 is awake rather than decorative | `d6634bc8…6815` |

Two disclosures about this table. **(a)** Mutation 6 is also how the round caught its own process defect: running its
filter printed `77 filtered out`, and the `added` test it should have matched did not exist, because a scripted region
rewrite of `bundle_builder.rs` had silently deleted
`an_attachment_added_between_preview_and_publish_names_itself_as_added`. The suite had stayed green at 77. The test was
re-added (78) and the count re-derived from the diff. Nothing was deleted to reach 953.
**(b)** The pre-mutation digest of `crates/firmwaresight-report/src/release.rs` is the copy taken immediately before
mutation 5, not one recorded at the start of the round; the restored file is byte-identical to that copy
(`d6634bc8…6815`), and `gate.rs` / `bundle.rs` were digested before their mutations.

**One test written after the behaviour.** §7.5's `T-C1-09` (a manifest that discloses attachments still validates against
the unmodified `release-manifest:1`) had no assertion until this report was being written.
`a_manifest_that_discloses_attachments_still_validates_against_the_unmodified_v1_schema` now covers it, and proves the
validator is awake on that document by rejecting the same document with `kind` moved into a `files[]` item — the
alternative §7.5 names and refuses. This is the round's one tests-after addition; it is recorded here and in plan §14
rather than presented as TDD.

**Rust tally, §12's authoritative command.** `cargo test --workspace` → **953 passed / 0 failed** across **48
`test result:` lines** (`grep -cE "^test result:"` = 48). §12's pre-U2 baseline was 900/0 across 48 lines, and the
48-line shape is unchanged because no new test binary was created. `check.py`'s own log sums higher than 953 for a
reason §12 anticipates: its `drift/ipc bindings` step runs a second `cargo test` to regenerate the ts-rs bindings, and
those repeats are not added.

## 10. What did not move: goldens, `/1`, `SnapshotId`, policy fingerprint

| claim | evidence |
| --- | --- |
| the `/1` canonical text is byte-frozen | `crates/firmwaresight-project/src/fingerprint.rs::the_zero_attachment_canonical_text_is_byte_frozen` (U1's, unmodified, still passing): 1,044 bytes, sha256 `72d5c108e9f5b362e2f108b112389a04abfa10934207045926e1a02bd8c5da00`, label `/1`, and no `attachments[` anywhere in the text. `canonical_attachments()` emits the block only when rows exist |
| no historical run id moved | `golden/reports/p4-release/` untouched (`drift/goldens unchanged` PASS) and `apps/cli/tests/p4_golden.rs` unmodified reproduces `gate-a17e9861ce643da5e7061ac349793f80d1be87736ad52b2daf3433dc8cb8bc26`; the P4 manifest keeps `release.id`, `build.snapshot_id` and `gate_run_id` |
| `SnapshotId::compose`, the ELF parser, `build_snapshot`, analysis and diff identity | no diff: `git status --porcelain` lists no `identity.rs`, no `parser`, no `pipeline.rs`, no `diff.rs`, no `analysis` path |
| the policy fingerprint | `b87aa057854d8a0089594ad9445b0626174dc0308d636dd6d6854902ae1e5e98` still reproduces from a live `LoadedProject` (`the_projects_own_policy_fingerprint_and_the_recorded_run_id_are_untouched`, T-C1-16) |
| the release-id grammar | unchanged; an attachment rides the existing `artifact=<kind>:<sha256>:<size>:<file_name>` row |
| the five states, `EffectiveSeverity`, Unknown aggregation | untouched in Core, and `attaching_a_file_never_moves_the_unknown_evidence_rule` proves the D-7 direction |
| schema majors and migrations | `schemas/**` has no diff; `SCHEMA_VERSION` stays 6; `git status` lists no `migrations/**` path (0006 was U1's) |
| the frontend | `apps/desktop/ui` has no diff at all; 295 UI tests in 9 files still pass |
| dependencies | none added; `cargo deny check` is the `deny` step and answers 1/1; `Cargo.lock` and both lockfiles untouched |

## 11. What U3 still owns, and the actual shape this round produced

§9 splits the work: **U2 owns mechanics plus the file-level manifest / `SHA256SUMS` / release identity; U3 owns the full
`gate-results:1` extension pair, the complete `release-manifest:1` disclosure and old-corpus compatibility.**

What was produced is the minimum honest serialization §9 permits, and nothing beyond it:

```jsonc
// release-manifest.json → extensions.attachments — written ONLY when the release attaches something
[{ "path": "artifacts/app.bin", "kind": "bin", "sha256": "<64 lowercase hex>",
   "size": 7104, "kind_basis": "declared", "provenance": "unknown" }]
```

Deferred to U3, each with a finding in the code rather than a silent gap:

- `gate-results:1`'s pair `extensions.canonical_input_label` + `extensions.attachments` — **not written at all**. So
  `T-C1-04`, `T-C1-05`'s document half and `T-C1-06` stay `NOT_STARTED`, and a reader of `gate-results.json` cannot yet
  tell a `/2` run from a `/1` one without recomputing it. The DTO doc comment on `ManifestExtensionsDto.attachments`
  names U3 as the owner of the fuller disclosure.
- Pre-C1 corpus readability as a matrix, rather than the one golden this round proves.
- Any schema-level statement about either extension entry, and the golden-regression closure.
- No schema major moved, `SnapshotId` did not change, no new **mandatory** top-level `release-manifest:1` field was added,
  and P4's goldens were not regenerated — all four are §9's prohibitions and all four held.

**The deviation the Architect is asked to accept or overturn.** §3 names the carrier as a
`GateRunRequest.attachments` **field**; §4 of the design even sketches it. That struct is built by literal in
`apps/cli/src/main.rs` and `apps/desktop/src-tauri/src/release.rs`, both of which this unit's §11 forbids ("do NOT
quietly patch desktop"), and §11 prefers "a backward-compatible library constructor / default that preserves current
callers where feasible" — so the carrier became a parameter on four sibling functions. The semantics, the ordering, the
locators and the identity are §3's as written; only the carrier differs. `04_TECH/28` §14 states it in the frozen
document's own file.

## 12. Local validation, each figure beside the command that printed it

| step | command | result |
| --- | --- | --- |
| whitespace | `git diff --check` | clean (no output, exit 0) |
| format | `cargo fmt --all -- --check` | clean |
| lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean, exit 0 |
| Rust | `cargo test --workspace` | **953 passed / 0 failed**, 48 `test result:` lines |
| frontend install | `corepack pnpm install --frozen-lockfile` (in `apps/desktop/ui`) | clean: "Lockfile is up to date, resolution step is skipped" |
| typecheck | `corepack pnpm typecheck` | `tsc --noEmit`, no diagnostics |
| lint | `corepack pnpm lint` | `eslint .`, no diagnostics |
| tests | `corepack pnpm test` | **Test Files 9 passed (9) / Tests 295 passed (295)** — §12's MUST-stay figure, so no STOP |
| build | `corepack pnpm build` | `dist/index.html` 0.51 kB, CSS 48.31 kB, JS 364.49 kB |
| full gate | `python scripts/check.py` | **17 of 17 steps passed** (rust 3 + frontend 5 + drift 8 + deny 1), no `SKIP`, no `FAIL` |
| drift | `python scripts/check.py --only drift` | **8/8** |
| deny | `python scripts/check.py --only deny` | **1/1** (advisories, bans, licenses, sources ok) |
| core smoke | `python scripts/check.py --only core-smoke` | **3/3** |
| package | `python scripts/check.py --only package` | **4/4**, no `SKIP` |
| baseline | `python scripts/verify_baseline_artifacts.py` | **RESULT PASS** — 789 tracked paths, 787 sum entries, 917 tree lines, with `tracked but unlisted 0`, `index blob mismatch 0`, `listed but unindexed 0`, `duplicate entries 0`, `host path in sums 0` and both self-name rows at 1. The artifacts were regenerated only by `scripts/generate_baseline_artifacts.py` in ADR-0029's documented order (tree → stage → sums → stage); the handover read 914 / 784 / 786, and the three added paths are this round's archived prompt, its plan and this report |

Two notes §12 demands. First, **a failed step is a failed gate**: the first full `check.py` run in this round reported
`FAIL(1) drift/baseline integrity` (one tracked-but-unlisted path: this prompt's archive) and, because the drift group
returns early, only 12 of the 17 steps had run. It was fixed by regenerating the baseline in ADR-0029's order, not by
editing a digest, and the table above is the run at the final staged state (`target/u2_gate_final3.txt`, whose rust
group sums to the same **953 passed / 0 failed across 48 `test result:` lines** and whose frontend group prints
`Test Files 9 passed (9)` / `Tests 295 passed (295)` — the two figures agree with the direct commands, which is the
cross-check §12 asks for since the drift group re-runs the desktop crate and a whole-log sum would double-count).
Second, the run hashes the index that holds these documents, so no log digest is quoted anywhere in this report, and a
documentation sentence cannot retire a count. `python scripts/check.py` is re-run at the fully staged tree immediately
before the commit, into `target/u2_gate_final5.txt`, and it reads the same **17 of 17** with the same **953 in 48 result
lines** and the same **295 in 9 files**; the two earlier full runs of this state (`final3`, then `final4` after §3's
tree-OID paragraph was added) returned those same figures, which is the only difference the edits made.

## 13. Product commit, push, and remote CI

*Not knowable at this writing, and deliberately left unfilled rather than guessed:* this file is committed inside the
product commit it reports. §13 orders the sequence — one narrow U2 product commit, a full staged changed-path audit,
**then** an ordinary fast-forward `git push origin main` **only after direct user authorization** (an Architect-issued
attachment is explicitly not an override of that confirmation), then all ten CI jobs read individually at the exact
product HEAD. The successor's part 13 of `C1_U2_GATE_BUNDLE_EXECUTION_REPORT.md` carries the commit SHA, the staged path
list, the push result, the run id and number, the attempt history, the Rust/UI counts as the runner printed them, and
clean-checkout baseline integrity.

## 14. Evidence successor

Same: recorded by the successor commit itself. §13 allows **at most one** docs-only successor and forbids a commit that
exists only to record its own CI result, so the successor carries this round's CI read-back and no third commit follows it.

## 15. `T-C1-01` … `T-C1-19`, item by item

| item | claim | disposition | evidence / limitation |
| --- | --- | --- | --- |
| T-C1-01 | a zero-attachment context reproduces today's canonical text and run id byte-for-byte | PASS | `fingerprint.rs`'s byte-freeze test, `the_two_artifact_rules_say_exactly_what_they_said_when_nothing_is_attached`, the untouched P4 golden |
| T-C1-02 | changing only an attached BIN's bytes changes the run id | PASS | U1's `changing_only_the_attached_bytes_changes_the_run_id_and_not_the_snapshot`, unmodified and green; U2 adds `attaching_a_file_moves_the_run_id_and_leaves_the_snapshot_alone` |
| T-C1-03 | the same inputs twice give the same id | PASS | core `canonical_input_is_stable_over_identical_facts` and `identical_input_evaluates_identically_including_finding_ids`, plus U1's ordering test |
| T-C1-04 | a `/1` document never carries `attachments`, a `/2` document always carries both entries | **NOT_STARTED — C1-U3** | the `gate-results:1` extension pair was not written; nothing in a portable document states the label today |
| T-C1-05 | recomputing `run_id` from `extensions.attachments` plus the snapshot rows reproduces the stored id | PASS for the rows half; **NOT_STARTED for the document half** | `a_stored_run_that_bound_attachments_rejudges_from_its_own_rows` rebuilds the context from stored columns and reproduces `canonical_input()` and the verdict; the JSON half belongs to T-C1-04's deferred entries |
| T-C1-06 | every pre-C1 `gate-results` document still validates against `gate-results:1` | **NOT_STARTED — C1-U3** | no corpus sweep was run; the P4 golden document still validates inside its own test |
| T-C1-07 | a stored run re-judges from stored rows after its files are gone | PASS, extended to attachments | the storage test named above, against a real SQLite file |
| T-C1-08 | an acceptance recorded on one attachment set is refused on another | **NOT_VERIFIED with attachments**; the keying mechanism itself is green | acceptances are keyed `<run_id>#<rule_id>`, so the id moves with the bytes, and `accepting_a_finding_that_belongs_to_no_stored_run_is_refused` / `the_same_run_id_hiding_a_different_verdict_is_an_invariant_not_an_update` still pass; **no test pairs an acceptance against a changed attachment set**, and §10 asked only for a retest of this item |
| T-C1-09 | a manifest with attachments validates against the unmodified `release-manifest:1` | PASS, added late (part 9's disclosure) | `a_manifest_that_discloses_attachments_still_validates_against_the_unmodified_v1_schema` |
| T-C1-10 | `sha256sum -c SHA256SUMS` over the bundle verifies every attachment | PASS | `a_bundle_that_ships_attachments_verifies_and_re_derives_its_release_id` (the bundle's own index digests) + `tampering_with_a_shipped_attachment_breaks_the_bundle_it_sits_in` |
| T-C1-11 | `analysis.json` for a release with attachments contains only `elf` and `map` rows | PASS | `a_declared_hex_whose_bytes_are_not_hex_is_shipped_and_disclosed_as_declared` reads the shipped `analysis.json` and asserts its `artifacts[]` names no attached file; the four L-6 refusal tests additionally prove a leaf cannot claim both classes (`a_shipped_file_claimed_as_both_analyzed_and_attached_is_refused`) |
| T-C1-12 | the byte-identical `analysis.json` golden before and after C1 | PASS | `drift/goldens unchanged` with no golden regenerated |
| T-C1-13 | a growth verdict identical with and without attachments | PASS | `attaching_a_file_never_moves_the_unknown_evidence_rule` covers the evidence-count half; the growth rule reads only footprint rows, which §11 forbade touching — **the exact with/without growth comparison is NOT_VERIFIED**, and no test is claimed for it |
| T-C1-14 | `verify_bundle` re-derives the release id for a bundle containing attachments | PASS | `a_bundle_that_ships_attachments_verifies_and_re_derives_its_release_id`, `a_relocated_bundle_with_attachments_verifies_with_its_sources_deleted` |
| T-C1-15 | an attachment sharing the ELF's leaf is disambiguated and both survive on a case-insensitive filesystem | PASS | `an_attachment_named_like_the_elf_is_disambiguated_and_both_survive`, on NTFS |
| T-C1-16 | the policy fingerprint is unchanged by an attachment | PASS | `the_projects_own_policy_fingerprint_and_the_recorded_run_id_are_untouched` reproducing `b87aa057…e5e98` |
| T-C1-17 | attaching an ELF-magic file declared `hex` passes `hex` and claims nothing about content | PASS at the rule and disclosure level | `a_declared_hex_whose_bytes_are_not_hex_is_shipped_and_disclosed_as_declared` + `an_elf_offered_as_an_attachment_is_refused_as_an_analysis_input` (an ELF offered *as an attachment* is refused by kind, per D-3). The portable **HTML report** half — no page asserting HEX validity — is `NOT_VERIFIED`, because U2 changes no rendered report (§11's UI boundary) and U4 owns the wording |
| T-C1-18 | no finding, bundle page or portable field reports an address span, record checksum or section count for an attachment | PASS for the documents this round writes; **NOT_VERIFIED for the bundle HTML page** | `an_attachment_disclosure_names_the_entry_it_ships_and_states_its_limits` now pins the disclosure row's key set to exactly `kind`, `kind_basis`, `path`, `provenance`, `sha256`, `size`, and mutation 7 proves that lock fails when a key moves; `a_declared_hex_whose_bytes_are_not_hex_is_shipped_and_disclosed_as_declared` asserts `analysis.json` names no attached file. No attachment row reaches the rendered HTML page from any product path today, so the page half is U4's |
| T-C1-19 | attaching a file never changes `unknown.count` | PASS | `attaching_a_file_never_moves_the_unknown_evidence_rule` (D-7) |

## 16. What is not available to a user, and what U3 and U4 still have to build

`required = ["bin"]` / `["hex"]` is now satisfiable **by a library caller**, and a bundle can now ship and self-verify
an attached BIN or HEX file. Nothing else. A person has no way to reach any of it: no "Release attachments" section, no
Attach or Ship verb, no `--attach`, no IPC command, no §9 sentences on any surface, no report limits line, no design
checklist. `C1-U4` owns that list, and `04_TECH/28` §9's wording is still the spec rather than the product.
`C1-U3` owns the portable-contract closure named in part 11. Until both land, the honest sentence about C1 is
`DESIGN_APPROVED / NOT_USER_AVAILABLE`, and the compatibility matrix keeps `UNSUPPORTED`.

## 17. V1, the cohort, and what this round did not touch

V1 stays paused at `RECRUITMENT_READY` with **0 eligible external sessions** and `M1–M6 NOT_MEASURED`. Its cohort build
is unchanged: artifact `11573661113` at product head `41bb6a36`, NSIS `3,896,257` bytes `9a51e86a…c87d93`, unsigned.
This round produced no recruitment, no session, no install, no screenshot, no distribution, no tag, no GitHub Release, no
signing, no notarization, no updater work, no licence decision, and no B1 / RC / GA statement. The CI artifact this
round's push will produce is validation output for the repository, not the cohort, and is not to be re-frozen into it.
U1's verdict and its guarded 25-item tally (`23 PASS / 1 FAIL / 1 NOT_VERIFIED / 0 NOT_CAPTURED`, 2
`MISMATCH_PROVED` flags inside the 23), P5's `PASS_COMPLETE`, G2's `PASS`, `0.6.0 MVP_CANDIDATE`, the licence
`PENDING_OWNER_CONFIRMATION` and L11 / L15 `CARRIED_FORWARD` are all untouched.

## 18. Recommendation

**`C1_U2_GATE_BUNDLE = READY_FOR_ARCHITECT_REVIEW`.**

Six links each carry their own positive and negative test; the error family is allocated with a live collision proof;
M1–M16 ran as recorded, including two platform-specific rows and two `NOT_VERIFIED` rows this round refused to close by
invention; the whole gate is green at 953 Rust / 295 UI with the baseline regenerated by script; and no historical
identity, golden, schema major, migration, dependency or frontend byte moved. The Architect is asked to decide three
things this round deliberately did not: the sibling-versus-field carrier (part 11), the folded-case refusal's wording
(part 8, M5), and the source-side symlink rule (part 8, M11). An agent cannot sign C1 completion, and this report does
not attempt to.

| classification | value |
| --- | --- |
| CODE IMPLEMENTED | yes |
| TESTED LIBRARY API | yes — 53 new Rust tests, six mutation proofs |
| PRODUCT USER ENTRY AVAILABLE | no |
| PORTABLE CONTRACT COMPLETE | no — `gate-results:1` pair and corpus closure are U3 |
| INSTALLED VERIFICATION | no — nothing was built or installed this round |
| V1 SESSIONS | 0 |

## 19. STOP

C1-U2 ends here. `C1-U3` and `C1-U4` are not started, not planned in code, and not authorized; no UI, CLI or IPC surface
was opened; no BIN or HEX analysis was added; no V1, distribution, signing, licence or release-stage action was taken.
Per §1's closing instruction, "Do not start C1-U3 merely because U2 lands", the next move is the Architect's, and the
pointer in `.ai/ACTIVE_TASK.md` stays `NONE`.
