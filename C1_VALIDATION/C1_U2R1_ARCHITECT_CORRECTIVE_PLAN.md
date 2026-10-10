---
title: "C1-U2R1 Architect Corrective Plan"
doc_id: "FS-C1-U2R1-PLAN"
product: "FirmwareSight"
version: "0.6.0"
status: "PLAN_WRITTEN_BEFORE_CODE"
owner: "Engineering"
last_updated: "2026-10-10"
---

# C1-U2R1 — targeted safety and Architect-review closure, plan written before code

## 1. Authority, unit, and the archive this plan is written against

Stage 1 of 3 in 《FirmwareSight — C1 Continuous Delivery》, delivered 2026-10-10 as a four-file packet and archived
verbatim at `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U2R1_Targeted_Safety_Closure_v1.0.txt` (6,591 bytes, 27 lines,
0 `CR`, sha256 `8d498e3b7cfa84ae…0fa34b`), beside its read-first cover and the two later stages, all four proven
against the received zip by CRC and `cmp`. The cover's own words bound the act: "There is NO blanket authority to
push, rewrite history, distribute, release, recruit V1 participants, change V1 cohort or invent test results.
Separate owner push approval is needed."

The unit is a *narrow corrective*, and the cover is the first independent read of `C1-U2`: it records
`ARCHITECT REVIEW = REWORK REQUIRED` over `facf287`, which supersedes this repository's `READY_FOR_ARCHITECT_REVIEW`
as the live word for that round, and its MISSION says "Do NOT rebuild C1-U2 … Complete only review-proved holes".
The four holes are `T-C1-08`'s acceptance pairing, `T-C1-13`'s exact growth comparison, the M5 folded-case
diagnostic, and the M11 source-side symlink rule. Its adjudications A–D answer the three questions `C1-U2` §18 left
to the Architect and one more (the error precedence), so this round changes **one sentence of remediation prose and
nothing else in the behaviour**: A accepts the `field → siblings` carrier, C states the symlink rule as
follow-the-target, D keeps Gate-before-staleness ordering.

## 2. Preflight, measured

`git status --porcelain=v1` empty · branch `main` · `git rev-parse HEAD` = `268440a90bafc7d26e4658c1445fd2161ffbe5f1`
· `git rev-parse origin/main` and `git ls-remote origin refs/heads/main` both
`2859bb2a7335b8cdea5be2f817a5b848dce126a1` · `git rev-list --left-right --count origin/main...HEAD` = `0 3` ·
`git worktree list` one entry · `git stash list` empty. The three local commits change only `.ai/CURRENT_STATE.md`,
`.ai/HANDOFF.md`, `.ai/README.md`, `04_TECH/28…`, `06_DELIVERY/06_STAGE_GATES.md`, `README.md`, `SHA256SUMS`; the tree
OIDs of `apps 5cf38bbe`, `crates 5979f1c6`, `schemas a8e7e542`, `golden 99352bef`, `fixtures 0fcc2114`,
`.github 174bd58f`, `scripts 76c1e1cb`, `assets da1f6d16`, `templates d34358ec`, `Cargo.toml 0d096b38`,
`Cargo.lock c5f6a38b`, `deny.toml 913072c4`, `DESIGN.md 5961d40b` are **the same at `2859bb2` and at `HEAD`**, which is
the tree-OID proof the cover demands rather than a name filter. `python scripts/verify_baseline_artifacts.py` prints
`RESULT PASS` at tracked 789 / entries 787 / tree lines 917, and `cargo test --workspace` on this host printed 48
`test result:` lines summing to **953 passed / 0 failed / 0 ignored** before any edit here.

One reported name does not resolve: the cover cites local docs commits `98596e21`, `7a6af54`, `268440a`, and
`git cat-file -t 98596e21` returns `Not a valid object name` while `git reflog show main` puts the first closeout at
`main@{2}` = `99a1bfd8016…`. Three docs-only commits exist in the stated position with a different first name and no
rewrite is visible, so this is a transcription difference, not the material divergence that would force a STOP. It is
recorded in the register entry rather than corrected silently, and it is the reason this plan quotes tree OIDs and
`rev-parse` output instead of trusting any reported head.

## 3. The packet's premises, each re-read at this head

| premise | measured |
| --- | --- |
| "T-C1-08 … no test pairs an acceptance against a changed attachment set" | `C1_U2_GATE_BUNDLE_EXECUTION_REPORT.md:465` says exactly this, and `grep -n "accept" crates/firmwaresight-project/tests/bundle_builder.rs` finds `World::accept` at `:302` keying `format!("{}#{}", recomputed_run_id, rule.as_str())`, with no attachment-bearing acceptance pairing anywhere |
| "T-C1-13 … the exact with/without growth comparison is NOT_VERIFIED" | report `:470`; the only growth-with-attachment assertion today is `attaching_a_file_never_moves_the_unknown_evidence_rule`, which is the evidence-count half |
| M5's remediation "is the integrity-failure sentence rather than a naming sentence" | `bundle.rs:187` maps `Index(_) \| VerificationFailed{..}` to `ERR-BUNDLE-6109`, and `bundle.rs:214-217` gives both the same "keep whatever was written…" remediation — a sentence about a bundle that was written, while `DuplicatePath` is raised while the index is still being assembled. The test's own doc comment at `tests/bundle_builder.rs:2321-2323` records that it was left for the Architect |
| "M11's symlink half is NOT_VERIFIED" | report `:217`; `grep -rni symlink --include=*.rs crates apps` returns only `bundle.rs:2058-2068` (the bundle-**contents** refusal, using `symlink_metadata`), and **no test at all** for that branch, source-side or output-side |
| "observe_attachment follows a symlink" | `evidence.rs:427` calls `std::fs::metadata` (follows) and `:431` requires `is_file()`, so a link to a readable non-empty regular file already passes and a broken link or a directory target already fails with E-1 `6115` — adjudication C is therefore *existing behaviour that has no test*, which is the honest way to describe this hole |
| adjudication A's carrier | the four siblings exist (`build_context_with_attachments` `evidence.rs:538`, `prepare_with_attachments` `bundle.rs:552`, `publish_with_attachments` `bundle.rs:420` on `BundlePlan`, `from_parts_with_attachments` `report/release.rs:264`) and `GateRunRequest` still has no `attachments` field, so A closes as accepted with the `BundlePlan`-versus-`BundleRequest` naming error in `04_TECH/28:558` and `04_TECH/17:281` still to fix |

## 4. The four targets, and what each test pins

**T-C1-08 — an acceptance is keyed to the bytes, at two layers.**
*Project layer* (`tests/bundle_builder.rs`): a `REVIEWING_CONFIG` world with a baseline, `bin` + `hex` attached,
`accept(diff.growth)`, publishes → the identical-set case is proved to still work. Then the release owner attaches a
different byte: the run id moves while `snapshot_id` does not, and carrying the same acceptance list over the new set
is refused with `ERR-BUNDLE-6101` before any destination exists. A fresh acceptance recorded against the new run id
then publishes, which proves the refusal names keying and not "attachments block releases".
*Storage layer* (`crates/firmwaresight-storage/tests/gate_history.rs`): persist run A over a context that carries
attachment rows, accept its `REVIEW` finding, persist run B over the same snapshot with a different attachment set,
and assert `accepted_reviews_for_run(B)` is empty, that `accept_review` of A's finding id under B answers
`AcceptReviewError::FindingNotFound` at `ERR-STORAGE-4007`, and that run A's finding, acceptance and attachment rows
still read back unchanged. No mocked identity and no mocked store: both halves call the functions the desktop calls.

**T-C1-13 — the growth rule says the same thing either way.** One world, one baseline, evaluated twice: no
attachment, then `bin` + `hex`. Assert the `diff.growth` finding's `state`, `effective_severity`, `evidence_refs` and
`summary` are equal, `counts.unknown` is equal, `snapshot_id` is equal, and the two run ids are **not** equal — that
last assertion is what keeps the comparison from being vacuous.

**M5 — the folded-case refusal tells the truth about a selection.** Implement adjudication B: keep the code
`ERR-BUNDLE-6109` and keep the invariant refusal, split `BundleError::remediation()` so a `Index(
ManifestError::DuplicatePath{..})` answers the duplicate-selection/destination-name sentence while
`VerificationFailed` keeps the "keep whatever was written, and report the release id" integrity sentence. Two tests:
the portable one (two real files, same bytes, leaves differing only in case) asserts the code, the naming
remediation, that no host path appears in either string, and that the destination never comes into existence; the
`#[cfg(windows)]` one asserts the same about NTFS's one-file-two-spellings case, and the existing test's doc comment
is corrected because the sentence it described as "recorded as a finding for the Architect" is now fixed.

**M11 — the source-side symlink rule, with its platform limits stated.** A `#[cfg(unix)]` module: a symlink whose
target is a readable non-empty regular file ships exactly the target's digest and size, the shipped entry is a
regular file rather than a link, and `verify_bundle` passes; a broken link and a directory link are refused at
`6115` naming no host path; a target whose bytes change, and a link re-pointed to a different file between preview
and export, are refused rather than shipped under the old verdict; and no symlink ever exists inside the bundle. A
`#[cfg(windows)]` counterpart that attempts `symlink_file`, prints an explicit `SKIPPED: … privilege` line and
returns when the OS refuses, because a test that quietly avoids its assertion would be a false PASS.

## 5. Public surface this round touches

One function body: `BundleError::remediation()` in `crates/firmwaresight-project/src/bundle.rs`. No new error
variant, no new code, no changed `code()` arm, no changed display, no new public type, no signature change, no
`apps/**`, no schema, no migration, no frontend, no CLI, no `.github`, no dependency, no design token. Tests are the
rest of the diff: two new project tests, one new project `#[cfg(windows)]` test, one new storage test, one new
`#[cfg(unix)]` symlink module, and one corrected doc comment on an existing test.

## 6. TDD order, and how RED is produced honestly

Two of the four targets are **preservation proofs**, and their tests are expected to be GREEN on unmodified code —
`T-C1-08` and `T-C1-13` pin behaviour that already exists but was never asserted. Claiming a RED for them would be a
fabrication, so instead each is proven *discriminating* by a mutation that must redden it, and the mutation is
reverted and re-measured:

1. `T-C1-13`: make `rule_baseline_growth` read `context.attachments` (a one-line injection) → the test must redden.
2. `T-C1-08` storage half: key an acceptance by `rule_id` instead of the full finding id → must redden.
3. `T-C1-08` project half: have `aggregate` treat any `REVIEW` as dispositioned when the acceptance list is non-empty
   → must redden.
4. M5: return the integrity sentence for `DuplicatePath` (i.e. the pre-fix code) → the naming assertion reddens —
   this one has a genuine RED, written and run before the fix.
5. M11 unix half: make `observe_attachment` refuse any symlink by using `symlink_metadata` → the positive case
   reddens; make it accept a directory target → the negative reddens. Not runnable on this host (§7), so the
   mutation is recorded as source-level reasoning plus the CI job that would run it, and labelled as such.

Order of work: M5 RED test → M5 fix → M5 GREEN; then the two preservation tests with their mutations; then the
symlink module; then the full gate.

## 7. Platform evidence, decided in advance

This host is Windows 10.0.19045, x64, Git Bash. Measured facts that shape what U2R1 can honestly claim:
`os.symlink` here fails with WinError 1314 (`客户端没有所需的特权`, ERROR_PRIVILEGE_NOT_HELD), so the `#[cfg(windows)]`
symlink test self-skips by design; `docker` is absent and `wsl` is present with **no installed distribution**, so no
`#[cfg(unix)]` test compiles or runs here. Therefore M11 ships as **written-but-unverified on this host on both
halves**, and the only place it can turn green is a Linux runner — which needs the push this packet withholds. The
count consequence is recorded per run, not smoothed: a `#[cfg(windows)]` test raises the Windows total only, and a
`#[cfg(unix)]` test raises the POSIX total only, so after this round the two platform numbers are expected to differ
by more than one, and both are reported from the run that printed them.

## 8. STOP decisions already made

STOP and report rather than proceed if: a symlink privilege turns out to be obtainable in a way that changes the rule
rather than the evidence; the canonical-path question turns out to contradict `AGENTS.md` §7 or the Tauri capability
boundary (adjudication C's own stop clause); an acceptance-pairing test forces a change in `apps/**`, a schema, or a
migration; or `DuplicatePath` cannot be distinguished from `VerificationFailed` without touching
`firmwaresight-report`'s public error type. Any of those is reported with the exact repro and a minimal proposal, per
the packet's "If an unavoidable change would cross a forbidden boundary: STOP".

## 9. What this round must not be reported as

Not `C1 COMPLETE`, not `USER_AVAILABLE`, not `SUPPORTED_WITH_LIMITS`, not "installed", not "all platforms verified".
The checkpoint word this round may write is `C1_U2R1 = LOCAL_CORRECTIVE_GATE_PASS / ARCHITECT_REVIEW_PENDING`, and
`M12`'s >512 MiB bound, `M11`'s symlink rule on this host, every UI-facing half of `T-C1-17`/`T-C1-18` and the whole
`gate-results:1` pair stay exactly where the previous round left them. The V1 cohort
(`11573661113` at `41bb6a36`, 0 sessions, M1–M6 `NOT_MEASURED`) is not an instrument this round may touch, and no
artifact built here is a cohort identity.
