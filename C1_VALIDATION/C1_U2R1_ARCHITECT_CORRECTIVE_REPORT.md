---
title: "C1-U2R1 Architect Corrective Report"
doc_id: "FS-C1-U2R1-REPORT"
product: "FirmwareSight"
version: "0.6.0"
status: "LOCAL_CORRECTIVE_GATE_PASS / ARCHITECT_REVIEW_PENDING"
owner: "Engineering"
last_updated: "2026-10-10"
---

# C1-U2R1 — targeted safety and Architect-review closure, as executed

Stage 1 of the three-prompt 《FirmwareSight — C1 Continuous Delivery》 packet delivered 2026-10-10 and archived in
`10_AUDIT/SOURCE_PROMPTS/`. The cover returned **`REWORK REQUIRED`** on `C1-U2` and named four holes; this round
closed exactly those four and nothing else. The checkpoint word this report may issue is
**`C1_U2R1 = LOCAL_CORRECTIVE_GATE_PASS / ARCHITECT_REVIEW_PENDING`** — which is not `COMPLETE`, not `USER_AVAILABLE`,
not `SUPPORTED_WITH_LIMITS`, and not a claim that an independent reviewer agreed.

## 1. What was authorized, and the sentence that bounds it

The unit's own MISSION: "Do NOT rebuild C1-U2. Product implementation facf287, docs head 2859bb2, CI #96/#97 are a
strong baseline. Complete only review-proved holes." Its SCOPE line admits "minimal Core/project/storage test files
and existing error-handling code" and forbids schemas, front-end, Tauri, CLI, migrations, toolchain, lockfiles, design
tokens, workflows, feature flags, new product families and any BIN/HEX parser. The cover adds the boundary that
matters most here: "There is NO blanket authority to push … Separate owner push approval is needed", so this round
ends at a local commit.

## 2. Preflight, measured rather than accepted

`git status --porcelain=v1` empty · `main` · HEAD `268440a90bafc7d26e4658c1445fd2161ffbe5f1` · `origin/main` and
`git ls-remote origin refs/heads/main` both `2859bb2a7335b8cdea5be2f817a5b848dce126a1` · ahead 3 / behind 0 · one
worktree · no stash · `cargo test --workspace` **953 passed / 0 failed across 48 `test result:` lines** at the
untouched head · `verify_baseline_artifacts.py` `RESULT PASS` at 789 / 787 / 917. The three local docs-only commits
were checked the way the cover demanded — by tree OID, not by name filter: `apps 5cf38bbe`, `crates 5979f1c6`,
`schemas a8e7e542`, `golden 99352bef`, `fixtures 0fcc2114`, `.github 174bd58f`, `scripts 76c1e1cb`,
`assets da1f6d16`, `templates d34358ec`, `Cargo.toml 0d096b38`, `Cargo.lock c5f6a38b`, `deny.toml 913072c4`,
`DESIGN.md 5961d40b` are identical at `2859bb2` and at `HEAD`.

**One reported name did not resolve, and it is reported rather than repaired.** The cover cites the local docs commits
as `98596e21`, `7a6af54`, `268440a`. `git cat-file -t 98596e21` → `fatal: Not a valid object name`; no tracked file
contains that string; `git reflog show main` puts the first closeout at `main@{2}` =
`99a1bfd801623d119e6e6e85f683b504a91dc5aa`. Three docs-only commits exist in the stated position with a different
first name and no rewrite, so this is a transcription difference and the cover's STOP trigger ("differs materially")
did not fire. Had it been treated as a divergence, the round would have stopped on a typo.

**Prompt provenance, and one measurement this round re-took.** The four delivered files were still present at the
path the owner pointed to. The received zip (`bf8222f625b4eb9ed4f52128f500a7ffa43aac16e74703a5071d1456cf0244a4`,
4 entries, `testzip()` reporting no CRC error) holds bytes equal to the extracted files, and each archived copy
equals those by sha256 with `cmp` identical. All four are LF-only. The first CR survey printed `CR=39/27/23/29`
through `grep -c $'\r'`; that is a property of that command form, not of the bytes — `tr -cd '\r' | wc -c` returns 0
on every delivered and archived file, and the archived digests therefore *are* the delivered digests. Recorded in
`10_AUDIT/SOURCE_PROMPTS/README.md`, where the entries live.

## 3. The four adjudications, and where each landed

| | adjudication | landed as |
| --- | --- | --- |
| A | `field → siblings` **accepted** as a backward-compatibility deviation | no `GateRunRequest.attachments` field, no `apps/**` patch; the deviation is now *decided* rather than open, and written up in `04_TECH/28` §14.1 |
| B | folded-case wording | `ERR-BUNDLE-6109` and the invariant refusal unchanged; the *selection* case now answers a selection-naming sentence (§4 below) |
| C | source-side symlink | the rule is followed-target-with-conditions; `observe_attachment` already behaved that way, so the work was tests (§7) |
| D | refusal order | **not changed**: Gate-before-staleness stays, `a_withdrawn_attachment_the_policy_requires_is_refused_by_the_gate_first` still asserts `ERR-BUNDLE-6101` |

## 4. Hole 1 — the folded-case diagnostic (M5, finding 1)

**Before**, quoted from the RED run this round recorded before editing anything:

```text
two entries name `artifacts/bin-6aa56d56-app.bin`, and a manifest cannot hash one file twice
```

and its remediation was the integrity sentence "keep whatever was written, and report the release id and the Gate
run id…" — true of a bundle that exists, useless about a selection that never became one.

**After.** One new arm, same code:

```text
`{name}` is attached twice: two of the selected files differ only in the case of their names, and one bundle cannot hold both
attach the two files under names that differ by more than case, or ship only one of them, then prepare the bundle again: nothing was written, so there is no bundle to repair
```

Mechanism: `fn duplicate_selection(ManifestError) -> BundleError` (`bundle.rs`) is applied with `.map_err()` at the
single manifest-assembly call site (`bundle.rs:1079`), so only a `DuplicatePath` raised against *this release's own
selection* is re-described; every other index refusal still travels through `Index(#[from] ManifestError)`, and
`verify_bundle`'s read-back of a written bundle keeps the integrity sentence. `code()` returns `ERR-BUNDLE-6109` for
both arms, so no caller's branch moves; the CLI's `release_exit_code` still lands on the same 6 (`apps/cli/src/main.rs:582-589`)
and the desktop envelope still reads `code()/to_string()/remediation()` generically (`bundle.rs:352-359`), which is why
no `apps/**` file had to change.

Tests: `a_folded_duplicate_selection_is_refused_by_naming_the_selection_and_not_a_manifest` (portable — two *distinct*
host files with identical content and fold-equal leaves; `bundle_names` renames both halves to `<kind>-<sha8>-<leaf>`,
which still folds, so the refusal fires on POSIX as well as NTFS), `the_ntfs_fold_of_one_host_file_names_its_own_cause`
(`#[cfg(windows)]`), `a_selection_duplicate_and_a_written_manifest_duplicate_are_told_apart` (asserts both arms answer
`6109`, that only the selection arm lost the word "manifest", and that an unrelated index refusal — `NoFiles` — is
still byte-transparent). The pre-existing `one_host_file_offered_under_two_cases_is_refused_rather_than_shipped_twice`
keeps every assertion it had; only its doc comment changed, because the sentence it recorded as left for the Architect
is now closed.

## 5. Hole 2 — `T-C1-08`, an acceptance is keyed to bytes, proved at two layers

**Storage** (`crates/firmwaresight-storage/tests/gate_history.rs::an_acceptance_recorded_over_one_attachment_set_does_not_dispose_of_a_different_one`,
against a real SQLite file): persist run A carrying real `gate_run_attachments` rows, accept its single REVIEW, then
persist run B over the same `snapshot_id` with a different attachment set. Asserted: `canonical_input()` differs while
`snapshot_id` does not; `accepted_reviews_for_run(B)` is empty; `accept_review(B, A's id)` answers
`AcceptReviewError::FindingNotFound` at `ERR-STORAGE-4007`; `aggregate_with_acceptances([A's id])` leaves B `Review`;
B's own finding id does dispose of it; A's attachment row, its finding and its one acceptance still read back
unchanged, with exactly two acceptance rows in the table.

**Project** (`tests/bundle_builder.rs::an_acceptance_recorded_over_one_attachment_set_does_not_make_another_set_release_ready`):
identical-set first — accept, select, `publish_attached`, `verify_bundle` passes, the outcome names run A. Then the
attached bytes move: run id changes, snapshot does not. Then **two** refusals, because two callers can be there.

**What that found, and what was deliberately not changed.** A library caller handing run A's acceptance list to run B
is refused by the bundle layer as `ERR-INTERNAL-9004` — "an acceptance names `gate-…#diff.growth`, which is not a
finding of run `gate-…`". The assertion was written expecting `ERR-BUNDLE-6101`; the engine said otherwise, and the
round filed the fact instead of moving the behaviour, because adjudications A–D do not authorize an error-model
change. It is right for a correct caller (a desktop reads acceptances *for its selected run* from a store that refuses
the cross-run case — the storage test above), and it is unhelpful for a mis-carrying library caller, since `INTERNAL`
reads as "a bug in this crate" when the truth is "that decision belongs to another run". The user-reachable path —
Gate re-run over the new set, review not yet accepted — is asserted separately and answers the ordinary
`ERR-BUNDLE-6101`. Both refusals leave no destination: `out-tc108-b` is proved absent from the project folder.

## 6. Hole 3 — `T-C1-13`, the growth comparison done exactly

`the_growth_rule_answers_identically_with_and_without_release_attachments`, on a new `ATTACHING_REVIEWING_CONFIG`
(required `elf,map,bin,hex` **and** `flash_growth_review_bytes = 100`, so one release holds both an attachment-only
requirement and an acceptance-only REVIEW). Same world, same baseline, `evaluate()` twice: `diff.growth`'s `state`,
`effective_severity`, `summary` and `evidence_refs` are equal; `counts.unknown` is equal; `snapshot_id` is equal;
`run_id` and the finding id are **not** equal, which is what stops the comparison being vacuous; and
`artifacts.required` passes citing `artifact:elf:` beside `attachment:bin:` and `attachment:hex:`. The unknown-count
half `C1-U2` had is untouched and is not counted as this test.

## 7. Hole 4 — M11's source-side symlink, with its platform record

`observe_attachment` uses `std::fs::metadata` (`evidence.rs:427`) then requires `is_file()`, so adjudication C's rule
is the behaviour that already existed: a link to a readable non-empty regular file is followed, a dangling link and a
directory link are E-1 refusals. What was missing was any test, and `C1-U2` said why in its own words: writing one
"would have fixed one behaviour by inventing a rule". The rule now exists, so the tests do — `source_symlink_posix`
(follow-and-ship, including a walk proving no shipped entry is a link; dangling refused naming no host path;
directory refused; re-point-after-preview refused at `ERR-BUNDLE-6103` before any write) and
`a_source_symlink_on_windows_ships_its_target_bytes_when_the_host_permits_links`.

**These are written-and-reviewed, not executed, on this host, in both directions.** Measured, not assumed:

```text
SKIPPED (M11 on Windows): this host refused symlink creation (raw OS error Some(1314)); the rule is asserted by the POSIX module and stays NOT_VERIFIED here
```

WinError 1314 is `ERROR_PRIVILEGE_NOT_HELD`; `docker` is absent and `wsl` is installed with no distribution, so the
`#[cfg(unix)]` module neither compiled nor ran anywhere in this round. A skip that printed nothing would have been a
false PASS, so the Windows case prints the line above and returns — visible under `--nocapture`, and recorded here.
**M11 therefore stays `NOT_VERIFIED` as executed evidence**; it needs one Linux run, which needs the push this packet
withholds.

## 8. Counts, per run, named

| | before | after | how |
| --- | --- | --- | --- |
| Rust, this Windows host | 953 / 48 lines | **960 / 48 lines / 0 failed** | 953 +5 platform-neutral +2 `#[cfg(windows)]` |
| Rust, POSIX | 952 / 48 lines | **961 projected, not measured** | 952 +5 platform-neutral +4 `#[cfg(unix)]`; the 2 new Windows-only cases do not compile there, exactly as the 1 pre-existing one already did not |
| UI | 295 in 9 files | **295 in 9 files** | no frontend path touched |
| baseline | 789 / 787 / 917 | **795 / 793 / 923** | +6 paths: four archived prompts, the plan and this report |

The eleven new `#[test]` functions by name, counted from the working diff rather than recalled
(`git diff -U0 HEAD -- crates/firmwaresight-project/tests/bundle_builder.rs
crates/firmwaresight-storage/tests/gate_history.rs | awk '/^diff --git/{f=$0} /^\+[[:space:]]*#\[test\]/{c[f]++}'`
prints 10 and 1), grouped by the
platform that compiles them. Seven run on this Windows host: `the_ntfs_fold_of_one_host_file_names_its_own_cause`
(Windows only), `a_folded_duplicate_selection_is_refused_by_naming_the_selection_and_not_a_manifest`,
`a_selection_duplicate_and_a_written_manifest_duplicate_are_told_apart`,
`the_growth_rule_answers_identically_with_and_without_release_attachments`,
`an_acceptance_recorded_over_one_attachment_set_does_not_make_another_set_release_ready`,
`a_source_symlink_on_windows_ships_its_target_bytes_when_the_host_permits_links` (Windows only), and
`an_acceptance_recorded_over_one_attachment_set_does_not_dispose_of_a_different_one` (storage). Four more exist only
inside `#[cfg(unix)] mod source_symlink_posix` and were never compiled here:
`a_symlink_offered_as_a_source_ships_its_target_bytes_and_leaves_no_link_behind`,
`a_broken_symlink_offered_as_a_source_is_refused_without_naming_its_host_path`,
`a_symlink_to_a_directory_offered_as_a_source_is_refused_as_not_a_regular_file`, and
`a_source_symlink_re_pointed_after_the_preview_cannot_ship_under_the_old_verdict`. That is why the POSIX figure is a
projection and not a measurement: those four carry assertions no compiler on this host has ever checked, so a type
error in them would first appear on a Linux job.

## 9. Local validation, each figure beside the command that printed it

`cargo test --workspace` exit 0, 48 `test result:` lines, 960 passed / 0 failed / 0 ignored ·
`cargo fmt --all -- --check` clean · `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean (it
caught one real thing in this round's own test code, `&[x.clone()]` → `std::slice::from_ref`, fixed rather than
allowed) · `python scripts/check.py` **17 of 17** with no `SKIP` and no `FAIL`, exit 0 · `--only drift` 8/8 ·
`--only deny` 1/1 · `--only core-smoke` 3/3 · `--only package` 4/4 · `--only frontend` 5/5 · vitest `Test Files 9
passed (9) / Tests 295 passed (295)` · `python scripts/verify_baseline_artifacts.py` `RESULT PASS` at
795 / 793 / 923 with every mismatch counter at 0.

**Two count caveats, both measured.** The full gate log prints 59 `test result:` lines summing to 1,182, because
`drift/ipc bindings` runs a second `cargo test` to regenerate the ts-rs bindings; the comparable single-invocation
figure is 960. And no tool asserts 960 or 295 — `grep -rn "test result" scripts .github` returns nothing, and
`check.py` records only exit codes (`check.py:95-97`) — so these are readings, not gates, exactly as `C1-U2` said of
its own figures.

## 10. Mutations, and one sequencing disclosure

Four mutations ran, each an edit to production code rather than to a test, each killed by a named test, and each
reverted with the file's digest re-measured.

| mutation | killed by | only? |
| --- | --- | --- |
| M1 — `rule_growth` cites the attachment set | `the_growth_rule_answers_identically_with_and_without_release_attachments` | yes (152 core + 83 project lines still green) |
| M2 — `aggregate()` matches an acceptance by rule suffix | the storage `T-C1-08` test | yes |
| M2b — the bundle layer quietly ignores a stale acceptance | the project `T-C1-08` test (its `ERR-INTERNAL-9004` assertion fell to `6101`) | yes |
| M3 — `duplicate_selection` reverted to a transparent carry | the two M5 wording tests | yes |

M2 versus M2b is the useful half of this table: the two `T-C1-08` layers are guarded by different code, and one test
did not kill the other's mutation, so both were needed. Restored digests re-measured after every revert:
`gate.rs 0c62a9980a5083192ed0c4ed347ad583f1d74dee0a130abf50877d8da128c091` (the same value `C1-U2` recorded when it
restored that file, so this round's net change to Core is nil) and
`bundle.rs 21b28a91d58c286c2f77fc01168de3183505e4cd7926e5f4873ea14a14efc871`. `grep -rn` for `MUTATION-C1U2R1` and
`RED-STAGE` over `crates/**` and `apps/**` returns nothing (the two strings survive only as prose in this report and in
`C1_U2_GATE_BUNDLE_EXECUTION_REPORT.md`, which is why the claim is scoped to sources rather than to the tree). Disclosed: one revert used `git checkout -- crates/firmwaresight-core/src/domain/gate.rs`
instead of an edit; the digest was re-measured against the pre-mutation value and matched, and no other path was
touched by any git state command. M4 (the symlink mutations in §6 of the plan) was **not run**, because the code it
would break does not compile on this host.

## 11. What did not move

`/1` canonical text still 1,044 bytes at `72d5c108e9f5b362e2f108b112389a04abfa10934207045926e1a02bd8c5da00` under
`fingerprint.rs::the_zero_attachment_canonical_text_is_byte_frozen`, which ran inside the 960 ·
`golden/reports/p4-release/` untouched with `drift/goldens unchanged` PASS · P4 run id `gate-a17e9861…bc26` ·
policy fingerprint `b87aa057…e5e98` · `SnapshotId::compose`, the five-state vocabulary, `EffectiveSeverity`,
Unknown aggregation, the release-id grammar · `SCHEMA_VERSION` 6 and migration `0006` (no new migration) · the six
links L-1…L-6 and all 78 pre-existing tests in `bundle_builder.rs` · `schemas/**`, `fixtures/**`, `assets/**`,
`templates/**`, `scripts/**`, `.github/**`, every lockfile, `DESIGN.md`, `deny.toml` · the 30 IPC commands
(`drift/ipc bindings unchanged` PASS) · 295 UI tests.

**Paths changed by this round** — nothing else:
`crates/firmwaresight-project/src/bundle.rs` (+34/−4 lines of the 694/4 code delta), `crates/firmwaresight-project/tests/bundle_builder.rs`,
`crates/firmwaresight-storage/tests/gate_history.rs`; documents `04_TECH/28` (§14.1 added, one method name corrected),
`04_TECH/17` (the same method name corrected), `10_AUDIT/SOURCE_PROMPTS/README.md` (+ four archived prompts),
`C1_VALIDATION/C1_U2R1_ARCHITECT_CORRECTIVE_PLAN.md`, this report, `BASELINE.yaml`, `.ai/CURRENT_STATE.md`,
`.ai/ACTIVE_TASK.md`, `.ai/README.md`, `.ai/HANDOFF.md`, `README.md`; plus the regenerated `DIRECTORY_TREE.txt` and
`SHA256SUMS`. **No `apps/**` path, no schema, no migration, no fixture, no golden, no workflow, no lockfile, no token.**

## 12. Outstanding, and what a reviewer should look at

1. `M11` is stated and tested but **unexecuted on this host on both platforms**; one Linux run closes it, and that run needs a push.
2. `M12`'s >512 MiB upper bound is still `NOT_VERIFIED`; no fixture was created.
3. `ERR-INTERNAL-9004` for a cross-run acceptance (§5) is an error-model question this round reported rather than decided.
4. `a_selection_duplicate_and_a_written_manifest_duplicate_are_told_apart` pins the two sentences but constructs both
   arms, so it does not pin the *mapping*; the mapping is pinned by the two end-to-end tests, which is exactly why M3
   reddened those and left this one green.
5. `T-C1-17`/`T-C1-18`'s rendered-page halves, the `gate-results:1` extension pair, the full `release-manifest:1`
   disclosure, the pre-C1 corpus re-validation and the golden closure belong to `C1-U3`; every user-facing sentence
   belongs to `C1-U4`.
6. Documentation drift found during onboarding and left where it is (outside this round's adjudicated scope, and not
   patchable one number at a time without a governance pass over the whole paragraph): `.ai/HANDOFF.md`'s U1 block
   opened with a stale "Machine state now" count line — date-qualified in this round instead of deleted, per
   the packet's "preserve date-qualified history"; `README.md`'s command comment still says 15 steps against a 17-step
   gate, and the paragraph above it still says "minimal SQLite at schema version 2" against `SCHEMA_VERSION` 6 and
   "exactly four Phase-0 library crates" against the `firmwaresight-project` crate that P3 added;
   `01_PRODUCT/01_PRD_MVP.md` lists four Gate states in one section and five in another.

## 13. Reproduce it

```text
git rev-parse HEAD origin/main; git status --porcelain=v1; git ls-remote origin refs/heads/main
cargo test --workspace
cargo test -p firmwaresight-project --test bundle_builder -- folded manifest_duplicate growth_rule_answers an_acceptance_recorded
cargo test -p firmwaresight-storage --test gate_history -- an_acceptance_recorded
cargo test -p firmwaresight-project --test bundle_builder -- --nocapture a_source_symlink_on_windows
python scripts/check.py
python scripts/verify_baseline_artifacts.py
```

## 14. STOP

`C1_U2R1 = LOCAL_CORRECTIVE_GATE_PASS / ARCHITECT_REVIEW_PENDING`, at one local commit on `main`, with **no push** —
`origin/main` stays at `2859bb2` and the cover requires separate owner approval for any fast-forward. Independent
architect acceptance is what turns `LOCAL_..._PASS` into anything stronger, and this round cannot issue it. Stage 2
(`C1-U3`) is authorized by the packet only after stage 1's objective gate, which is green; it starts on the owner's
word that this round should continue. Nothing here resumes V1, moves its cohort, installs the locally built
`target/dist-package/` artifact, or makes release attachment reachable by a person.
