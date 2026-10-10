# C1-U3 — Portable Contract + Regression: the report the Architect reviews

Executed 2026-10-10 as stage 2 of the four-file continuous-delivery packet. Checkpoint word, and the only one this
round may use:

`C1_U3 = LOCAL_CONTRACT_GATE_PASS / ARCHITECT_REVIEW_PENDING`

Not `PASS_COMPLETE`, not `USER_AVAILABLE`, not `SUPPORTED_WITH_LIMITS`. Nothing here was pushed, installed, or
accepted, and §12 lists what is still owed to the Architect rather than to an implementer.

---

## 1. What this unit was for

`C1-U1` gave attachments an identity, `C1-U2`/`C1-U2R1` let an attached file answer a Gate rule and ship inside a
verified bundle. `04_TECH/28` made five compatibility promises about that machinery and none of them had been
tested against a published contract. This round is the one that proves them, or reports which one fails.

Two things had to be built, because they did not exist:

* §7.3's `gate-results:1` extension pair. `grep -rn "canonical_input_label"` over `crates/**`, `apps/**`,
  `schemas/**` and `golden/**` returned **zero** hits when this round started: a run that judged attached bytes
  wrote a document saying nothing about them.
* The reader-side enforcement of that pair, plus the tie between a Gate document and the manifest that gates it.
  `verify_bundle` read `gate-results.json` for schema validity and never compared it to anything.

Everything else was regression work: the pre-C1 corpus read back under today's contracts, the frozen golden
verified by an unmodified verifier, and the T-C1-01…19 matrix filled in row by row.

## 2. Authority, entry condition, and the discipline that came with it

| item | value |
| --- | --- |
| prompt | 《FirmwareSight — C1-U3 Portable Contract + Regression, Architect task prompt v1.0》, stage 2 of 3, 2026-10-10 |
| archived | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U3_Portable_Contract_And_Regression_v1.0.txt` — 5,949 B, 23 lines, 0 `CR`, sha256 `6e4053db37fd5ba9560e7941bfa9ba2773d912c13f5261a6d590364e16345f83`; equality to the delivered bytes was proved in the stage-0 register entry, not re-asserted here |
| design | `04_TECH/28` §§5, 6, 7.1–7.7, 8, 12, 13; `ADR-0030`; `04_TECH/26`; the six `schemas/*.schema.json` |
| plan | `C1_VALIDATION/C1_U3_CONTRACT_REGRESSION_PLAN.md`, written before the first product edit |
| entry condition | stage 1's objective gate: `920e764…` HEAD, clean tree, 17/17, 960 Rust / 48 lines / 0 failed, report written |

The packet's entry line also says *"do not interpret a coding agent's SELF-REVIEW as final architect PASS"*. This
round therefore treated stage 1's `LOCAL_CORRECTIVE_GATE_PASS` as an unverified-but-objective precondition, not as
an accepted corrective: `C1_U2R1` still reads `ARCHITECT_REVIEW_PENDING` in `BASELINE.yaml`, and this report does
not upgrade it.

## 3. Preflight, measured

`git status --porcelain=v1` empty · branch `main` · HEAD `920e7649f47b51a7a0ffe40b7fb123ac1dc8425d` ·
`origin/main` `2859bb2a7335b8cdea5be2f817a5b848dce126a1`, and `git ls-remote origin refs/heads/main` agrees ·
`git rev-list --left-right --count origin/main...HEAD` → `0	4` · one worktree · `git stash list` empty ·
ancestry `2859bb2 → 99a1bfd → 7a6af54 → 268440a → 920e764`.

Nothing material diverged from what the packet asserts, so its STOP-before-writes condition never fired.

## 4. The contract that landed

### 4.1 Writer

* Core (`crates/firmwaresight-core/src/domain/gate.rs`): `GateContext::INPUT_LABEL_UNBOUND` /
  `INPUT_LABEL_BOUND` and `canonical_input_label()`, and `canonical_input()` now takes its first line from that
  method. Before this round the two grammar strings existed only inside `canonical_input()`'s body; a document
  that wanted to name the grammar had to restate it, which is how a label and an identity drift.
* Report (`crates/firmwaresight-report/src/gate.rs`): `GateAttachmentDto { kind, sha256, byte_size }`, and
  `GateExtensionsDto` gained `canonical_input_label: Option<&'static str>` and `attachments:
  Vec<GateAttachmentDto>`, **appended** after the four P3 fields and each `skip_serializing_if`. Appending matters:
  field declaration order is emitted key order in this crate, so an inserted field would have moved every key
  after it in every document ever written.
* `GateResultsDto::from_evaluation(context, evaluation, policy_sha256, project_config_schema_version)` — the
  context is now required and there is no attachment-free second constructor. A projection built from an
  evaluation alone could describe a `/2` run as though it had judged nothing, and §7.3's "and vice versa" is
  exactly that defect named in advance. Both production callers moved: `crates/firmwaresight-project/src/bundle.rs`
  (the bundle's `gate-results.json`) and `apps/cli/src/main.rs:396` (the CLI's `gate --json`). No CLI flag, IPC
  command or UI path was added — `apps/cli` changed by one argument at one existing call site.
* Both entries come from one predicate: `bound = !context.canonical_attachments().is_empty()`, label from
  `context.canonical_input_label()`, rows from the same canonicalized vector. The pair cannot be half-written by
  this build, and mutation M-A/M-B (§8) prove that.

### 4.2 Reader (`check_gate_disclosure`, called from `verify_bundle`)

It refuses, each with `ERR-BUNDLE-6109` — the code every other read-back refusal already carries, so **no new
number in the `ERR-BUNDLE-61xx` family was allocated**:

1. rows without a label; a label with no rows (including an empty array); a label that is not exactly
   `firmwaresight-gate-input/2`; a label that is not a string.
2. a disclosed row that is not an object; a row carrying any key beyond `byte_size`/`kind`/`sha256`; a kind word
   nothing can attach; a digest that is not 64 lowercase hex; a `byte_size` that is absent, zero, or not an
   integer.
3. rows not strictly increasing by `(kind word, digest)` — one check that refuses both a reordered block and a
   `(kind, digest)` pair bound twice, because §5 rules 1 and 2 are the same statement about a set.
4. a disclosed set that is not the deduplicated `release-manifest:1` `extensions.attachments` set.
5. a `run_id` that is not the manifest's `extensions.gate_run_id`. **This is the check `C1-U2`'s six links did not
   have**: a bundle could ship one run's verdict beside another run's gate and still self-verify.

`check_gate_disclosure` runs *after* `model_from_documents`, deliberately: the manifest's own structural refusals
(`AttachmentNotShipped`, `ShippedEntryMismatch`, the `kind_basis`/`provenance` guards) name a broken disclosure more
precisely than a cross-document disagreement does, and two `C1-U2` tests assert those messages. Ordering is a
diagnostic-quality decision, and it is tested from both sides (§8's M-E/M-F).

### 4.3 Why the iff is not a schema rule

`gate-results:1` and `release-manifest:1` both open `extensions` with `"additionalProperties": true`; that is the
slot v1 reserved precisely so facts could be added without a major. Expressing "label present ⇔ rows present" in
JSON Schema would mean narrowing that open object — a semantics change, needing a major, forbidden by the packet and
by `ADR-0030` D-4's compatibility rule. So the schema proves **containment** and the reader proves **pairing**:
`the_strict_boundaries_that_make_extensions_the_only_slot_are_still_closed` asserts that both roots are still
closed, that `files[]` items are still closed, that `extensions` is still open, and that `analysis:1` has no
`extensions` key at all — which is why an attached file has nowhere to hide as a pseudo-analysis row.

## 5. The T-C1 matrix, row by row, with the test that holds it

Statuses are what this round measured. "Carried" means an earlier round wrote it and this round re-ran it without
change; no row was closed by citing prose.

| row | what it demands | status now | where |
| --- | --- | --- | --- |
| T-C1-01 | zero-attachment text **and document** unchanged | **CLOSED both halves** | text: `project/src/fingerprint.rs:107` (1,044 bytes, pinned id); document: `report/tests/gate_schema_contract.rs:370` |
| T-C1-02 | an attached byte moves the run id | CLOSED (carried) | `project/tests/release_attachments.rs:101`, `:116`; `bundle_builder.rs:2128` |
| T-C1-03 | same inputs, same id | CLOSED (carried) | `release_attachments.rs:178`; `core/domain/gate.rs:2810` |
| T-C1-04 | `/1` never carries `attachments`; `/2` always carries both | **CLOSED both directions** | writer `gate_schema_contract.rs:393`, `:458`, `:514`; end-to-end `bundle_builder.rs:3224`; reader refusals `bundle_builder.rs:3315`, `:3331`, `:3351`, `:3370`, `:3397`, `:3418`, `:3439`, `:3461`, `:3486`, `:3520`, `:3549` |
| T-C1-05 | recompute the id from schema-supported fields plus saved context | **CLOSED both halves** | storage half carried (`storage/tests/gate_history.rs:1157`); document half `release_attachments.rs:919` |
| T-C1-06 | every pre-C1 `gate-results` document still validates | **CLOSED over the corpus the repository carries** | `release_attachments.rs:759` (enumerates `golden/reports/p4-release/`, all five JSON documents against the schema each names); gate-specific `gate_schema_contract.rs:548` |
| T-C1-07 | a stored run re-judges after its files are gone | **CLOSED — and the design's citation was corrected** | `gate_history.rs:1516` `the_run_is_judged_from_stored_rows_after_the_files_are_gone`; attachment-run half carried at `gate_history.rs:1422` |
| T-C1-08 | an acceptance does not cross attachment sets | CLOSED (carried from `C1-U2R1`) | `gate_history.rs:1589` (storage) and `bundle_builder.rs:3912` (bundle) |
| T-C1-09 | a manifest with attachments validates against unmodified `:1` | CLOSED (carried) + boundary pin | `report/tests/release_schema_contract.rs:1387`; `:1422` |
| T-C1-10 | `sha256sum -c SHA256SUMS` verifies every attachment | **CLOSED in-process / the literal command NOT_EXECUTED** | in-process: `bundle_builder.rs:2998` and its negative `:3078`; golden index `release_attachments.rs:819`. No test shells out to `sha256sum` — see §12 |
| T-C1-11 | `analysis.json` holds elf/map only | **CLOSED with the exact-row-set half** | `bundle_builder.rs:3571` |
| T-C1-12 | byte-identical `analysis.json` golden before/after | **CLOSED, and by a test this round did not write** | `apps/cli/tests/p4_golden.rs:339` compares all eight golden documents byte-for-byte; document-side pair absence `gate_schema_contract.rs:370` |
| T-C1-13 | growth identical with and without attachments | CLOSED (carried from `C1-U2R1`) | `bundle_builder.rs:3832` |
| T-C1-14 | `verify_bundle` re-derives a release id for a bundle with attachments | **CLOSED, now including the immutable corpus** | `bundle_builder.rs:2998`, `:3040`; golden read-back `release_attachments.rs:862` |
| T-C1-15 | a colliding leaf name is disambiguated and both files survive | CLOSED (carried) | `bundle_builder.rs:2261`, `:2306`, `:2357`, `:2386` |
| T-C1-16 | the policy fingerprint is unchanged for unchanged config | CLOSED (carried) | `release_attachments.rs:250` |
| T-C1-17 | ELF-magic bytes declared `hex` satisfy `hex` and claim nothing about HEX | **CLOSED, with real ELF magic** | `bundle_builder.rs:3609` (`0x7f 'E' 'L' 'F' …` attached as `IntelHex`; PASS state, limit sentence, no analysis row, disclosure says `declared`/`unknown`, bundle verifies) |
| T-C1-18 | nothing reports a span, record checksum or section count for an attachment | **CLOSED for the three surfaces a bundle carries** | `bundle_builder.rs:3667` sweeps every finding that cites an `attachment:` locator, every report line naming an attached leaf, and both disclosure rows — and asserts the sweeps are not vacuous |
| T-C1-19 | attaching never moves `unknown.count` | CLOSED (carried) | `core/domain/gate.rs:2437` |

Nineteen rows: **19 with a named test**, of which 12 were already held, 7 gained their missing half here, and 2
(T-C1-04's document half, T-C1-17/18) went from nothing to covered. `BASELINE.yaml`
`c1_u3_portable_contract_and_regression.final_state` carries the same 19 rows as machine-readable state.

## 6. Old-corpus read-back, and the exact no-attachment bytes

`golden/reports/p4-release/` is the only pre-C1 portable corpus inside Git. Measured this round:

* its `gate-results.json` `extensions` object is exactly `policy_sha256`, `project_config_schema_version`,
  `baseline_snapshot_id`, `overall_effective_severity` — neither C1 key — with
  `run_id = gate-a17e9861ce643da5e7061ac349793f80d1be87736ad52b2daf3433dc8cb8bc26`;
* all five JSON documents (`analysis`, `diff`, `gate-results`, `accepted-reviews`, `release-manifest`) validate
  against today's `schemas/`, each against the contract its own name declares, and all six majors are still 1;
* its `SHA256SUMS` has 8 lines; six re-hash to the line they carry, and the two that do not exist on disk are
  exactly `artifacts/firmware.elf` and `artifacts/firmware.map` — the corpus is documents-only by design, which
  `apps/cli/tests/p4_golden.rs:354` already pins;
* given the two committed fixtures those lines name (`fixtures/elf/p2-diff/target/`), the relocated bundle
  verifies under this build's *stricter* verifier and re-derives
  `release-e400a8ac51a57d34e309d254536f8850044ef4dd53e26245045c24fdc86fdcc5`, the id it claims.

The last point is the compatibility proof that matters most: a verifier that learned five new refusal classes still
accepts a bundle published before any of them existed. Had it not, §10's STOP would have fired.

Byte-frozen `/1` identity text is unchanged: 1,044 bytes, sha256
`72d5c108e9f5b362e2f108b112389a04abfa10934207045926e1a02bd8c5da00`, asserted by
`the_zero_attachment_canonical_text_is_byte_frozen` inside this round's 987.

## 7. `path` versus `file`, decided and written down

| | `04_TECH/28` §7.5 / §12 sketch | shipped, and now canonical |
| --- | --- | --- |
| locator | `file` | **`path`** — bundle-relative, the same spelling `files[]` and `disclosed_attachments` use |
| provenance | absent | **`provenance: "unknown"`, mandatory**, because an attached file's origin is permanently unestablished (`ADR-0030` D-7) |
| other fields | `kind`, `kind_basis`, `sha256`, `size` | unchanged |

The Architect's ruling (U3 prompt §4) is that `path` wins because the index and the verifier already speak it, and
that two aliases would drift. The superseded sketch is **named and dated** in `04_TECH/28` §14.2 and in
`04_TECH/26`'s C1 section rather than deleted from §7.5, so a reader of the frozen design sees both the plan and
the ruling. `files[]` was not renamed and no `file` alias was emitted anywhere.

## 8. Verification, each figure beside the command that printed it

`cargo test --workspace` → exit 0, **48** `test result:` lines, **987 passed / 0 failed / 0 ignored** ·
`cargo fmt --all -- --check` clean ·
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean ·
`python scripts/check.py` → **17 of 17**, no `SKIP`, no `FAIL`, exit 0 ·
`--only drift` 8/8 · `--only deny` 1/1 · `--only core-smoke` 3/3 · `--only package` 4/4 · `--only frontend` 5/5 ·
vitest `Test Files 9 passed (9) / Tests 295 passed (295)` ·
`python scripts/verify_baseline_artifacts.py` → `RESULT PASS` at **797 / 795 / 925**.

Twenty-seven new `#[test]` functions across five **existing** targets, counted per target after the final writes:

| target | before | after |
| --- | --- | --- |
| `report/tests/gate_schema_contract.rs` | 13 | 19 |
| `report/tests/release_schema_contract.rs` | 34 | 35 |
| `project/tests/bundle_builder.rs` | 84 | 99 |
| `project/tests/release_attachments.rs` | 20 | 24 |
| `storage/tests/gate_history.rs` | 33 | 34 |

No test binary was added, so the line count stayed 48, and the gate log still double-counts for the same known
reason: `drift/ipc bindings` runs a second `cargo test` to regenerate the ts-rs bindings.

**POSIX: 988 projected, not measured.** 961 was stage 1's projection and every one of this round's 27 cases is
platform-neutral — the working diff adds no cfg-gated test attribute in any of the five targets. The host still has
no POSIX environment (`docker` absent, `wsl` installed with no distribution), so `C1-U2R1`'s four `#[cfg(unix)]`
cases have *still* never been compiled, and `M11` stays `NOT_VERIFIED`. Nothing in this round changed that.

**Mutations, seven, each killed by named tests, each reverted.**

| mutation | killed by |
| --- | --- |
| M-A — drop both `skip_serializing_if` | `a_run_that_bound_no_attachment_writes_neither_new_extension_entry`, `a_pre_c1_bundle_that_attached_nothing_still_verifies_without_the_pair`, **and** the pre-existing `apps/cli/tests/p4_golden.rs` byte comparison |
| M-B — write the label whatever the run bound | the first two of those |
| M-C — disclose `context.attachments` uncanonicalized | `a_run_that_bound_attachments_writes_the_label_and_the_rows_together`, `an_exact_kind_and_digest_pair_is_disclosed_once_however_many_copies_arrive`, `the_disclosed_rows_rebuild_the_text_the_run_id_was_taken_over`, and `a_bundle_that_ships_attachments_discloses_the_same_bytes_in_both_documents` — the last only after this round strengthened its fixture to attach a BIN named `zz-image.bin` beside a HEX named `aa-image.hex`, because the publish path sorts rows by *path* and `app.bin`/`boot.hex` happened to make the path order and the identity order coincide. That coincidence was a hole in the end-to-end claim, found by running the mutation, and it is why the four killers are listed rather than one. |
| M-D — remove the `run_id` ⇔ `gate_run_id` tie | `a_gate_document_that_reports_another_run_than_the_bundle_gates_is_refused`, alone |
| M-E — accept rows without a label | `a_gate_document_that_discloses_rows_without_a_label_is_refused`, alone |
| M-F — stop comparing the two disclosed sets | `a_gate_document_that_discloses_bytes_the_manifest_does_not_ship_is_refused` and `an_attachment_release_documented_as_though_it_judged_nothing_is_refused`, and nothing else |
| M-G — open `files[]` items in `schemas/release-manifest.schema.json` | `a_manifest_that_discloses_attachments_still_validates_against_the_unmodified_v1_schema` and `the_strict_boundaries_that_make_extensions_the_only_slot_are_still_closed` |

Reverts re-measured: `report/src/gate.rs` `ad8dded370b17add33b47dfb932033ebe7ee321c0a7b7ccd188aa8cad253632e`,
`project/src/bundle.rs` `aa8cef2f40a55ba4f2a1fa149892d4334f561a6779e747cea60b46337bea0eca`,
`core/src/domain/gate.rs` `76b964faca3958558cb657926b746800d3835eaca795e126981e494b9411f4d3`. `schemas/` is clean
against `HEAD` (`git diff --exit-code -- schemas/` prints nothing), and a grep for `MUTATION-C1U3` across `crates`,
`apps` and `schemas` returns zero. Two disclosures: **M-G was reverted with
`git checkout -- schemas/release-manifest.schema.json`** — that file had no other change this round, so the command
discarded only this round's own mutation, and its byte identity was then proved twice (empty `git diff`, then the
gate's drift step). And **one transient failure is unattributed**: while measuring, a five-target loop reported
`bundle_builder 98 passed / 1 failed`, and the grep in use dropped the failing test's name. It did not reappear in
five later runs of that target (99/99 each) or in either full workspace run (987/0/48 twice). Reported as a lost
measurement; no cause is claimed for it.

## 9. What did not move

`/1` canonical bytes and the P4 run id · `golden/reports/p4-release/**` (read, never rewritten;
`drift/goldens unchanged` PASS) · `scripts/update_goldens.py` never run · the policy fingerprint
`b87aa057…e5e98` · `SnapshotId::compose`, the five states, `EffectiveSeverity`, Unknown aggregation, the release-id
grammar · `SCHEMA_VERSION` 6 and migration `0006` · all six schema majors and every closed property list ·
the `ERR-BUNDLE-61xx` allocation table (no new number) · the refusal order adjudication D confirmed · the 30 IPC
commands and their generated bindings (`drift/ipc bindings unchanged` PASS) · 295 UI tests in 9 files ·
`Cargo.toml`/`Cargo.lock`/`deny.toml`/`rust-toolchain.toml` · `assets/design-tokens.json`, `DESIGN.md` ·
`.github/**`, `migrations/**`, `fixtures/**`, `schemas/**` · the V1 cohort `11573661113` at `41bb6a36`, 0 sessions.

**Paths this round changed, and nothing else:**
`crates/firmwaresight-core/src/domain/gate.rs`, `crates/firmwaresight-report/src/gate.rs`,
`crates/firmwaresight-report/src/lib.rs`, `crates/firmwaresight-project/src/bundle.rs`,
`apps/cli/src/main.rs` (one argument at one existing call site), the five test files named in §8,
`04_TECH/26_PORTABLE_SCHEMA_POLICY.md`, `04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md` (§14.2 only),
`BASELINE.yaml`, `.ai/CURRENT_STATE.md`, `.ai/ACTIVE_TASK.md`, `.ai/README.md`, `.ai/HANDOFF.md`, `README.md`,
`C1_VALIDATION/C1_U3_CONTRACT_REGRESSION_PLAN.md`, this report, and the regenerated `DIRECTORY_TREE.txt` /
`SHA256SUMS`. **No `apps/desktop/**` path, no schema file, no migration, no fixture, no golden, no workflow, no
lockfile, no design token.**

## 10. STOP conditions, and which ones came close

Fixed in the plan before the first write, restated with their outcomes:

* *"If this cannot validate with existing `:1` schemas, STOP rather than bump a major."* Did not fire: every
  document validates, and §4.3 records why the pairing rule belongs to the reader rather than to a schema.
* *"If a golden document has to be rewritten to stay green, STOP."* Did not fire; no golden byte moved, and the
  stricter verifier accepts the frozen bundle (§6).
* *"If a pre-C1 document stops validating, STOP and report it as the compatibility break it is."* Did not fire.
* *Close, and worth naming:* the first cut of the reader check ran **before** `model_from_documents`, and two
  `C1-U2` tests that assert specific manifest-side messages (`no document says what kind`, `does not carry`)
  began failing on a *different* true refusal. The fix was not to relax those tests — it was to order the checks so
  the narrower cause reports first. Both tests now pass unmodified, and §4.2 records the ordering as a deliberate
  rule rather than an accident.

## 11. How to reproduce

```text
cargo test --workspace                       # 48 lines / 987 passed / 0 failed on a Windows host
cargo test -p firmwaresight-report --test gate_schema_contract
cargo test -p firmwaresight-project --test bundle_builder
cargo test -p firmwaresight-project --test release_attachments
cargo test -p firmwaresight-storage --test gate_history
corepack pnpm --dir apps/desktop/ui test     # unchanged: 295 in 9 files
python scripts/verify_baseline_artifacts.py  # RESULT PASS at 797 / 795 / 925
python scripts/check.py                      # 17 of 17
```

No tool asserts 987, 295 or 797: `grep -rn "test result" scripts .github` returns nothing, and `scripts/check.py`
records exit codes only. These are readings, beside the command that printed them, exactly as the previous rounds
said of their own figures.

## 12. Outstanding, and what a reviewer should look at

1. **`sha256sum -c` has never been executed** against a bundle's `SHA256SUMS` by anything in this repository.
   T-C1-10 is closed in-process. A reviewer wanting the literal command should run it over a published bundle;
   this round does not claim an external SHA-256 implementation verified what only Rust's `sha2` did.
2. **"Every pre-C1 document" is bounded by the corpus Git carries.** The P4 golden bundle is the only portable
   corpus in the repository; other historical documents live in evidence directories outside Git. Say the bound,
   not the universal.
3. **`04_TECH/28` §8's kind-conflict sentence is not implemented.** The design says a declared kind and the leading
   bytes should both be kept and the disagreement shown; the attach path never calls `detect_format`, so
   `kind_basis` is always `declared` and no conflict is reported anywhere. T-C1-17 asserts the honest negative (no
   HEX claim) and this report files the missing positive. Displaying a conflict would be a Gate behaviour change,
   which is not this unit's to make.
4. **A consistently forged bundle still self-verifies.** The run-id tie closes "documents disagree with each
   other"; it cannot close "every document was rewritten together", because nothing in a bundle carries an external
   anchor and `release-report.html`/`SHA256SUMS` never claim authenticity. Narrowed, not closed.
5. **`M11` (POSIX symlinks) and `M12` (>512 MiB) are still `NOT_VERIFIED`**, and `ERR-INTERNAL-9004` for a
   cross-run acceptance is still an error-model question the Architect owns, filed by `C1-U2R1` and untouched here.
6. **§9's sentences are still nobody's to read.** No surface tells a person that a release can attach bytes, so
   `BIN_HEX_RELEASE_ATTACH` is `DESIGN_APPROVED / NOT_USER_AVAILABLE` and `C1-U4` remains the unit that changes it.

## 13. Stop

Stage 2's objective checkpoint is met and recorded: a green local contract gate, the full T-C1 matrix with a named
test per row, the old corpus read back, no schema major moved, no golden rewritten, one scoped local commit, no
push.

What this round cannot give itself is the two things the packet reserves for others: the Architect's acceptance of
`C1-U2R1` and of `C1-U3`, and the owner's authorization to push. `origin/main` stays `2859bb2`; the five local heads
on top of it (`99a1bfd`, `7a6af54`, `268440a`, `920e764`, and this round's) are a fast-forward away whenever the
owner chooses. Stage 3 (`C1-U4`) starts only under the same continuous program and only on this round's objective
gate, not on its self-assessment.
