---
title: "C1-U1 Data and Identity implementation plan"
doc_id: "FS-C1-U1-001"
product: "FirmwareSight"
unit: "C1_U1_DATA_AND_IDENTITY"
status: "PLAN"
owner: "Engineering"
authority: "10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U1_Data_And_Identity_Implementation_v1.0.txt + ADR-0030 + 04_TECH/28 + AGENTS.md 2, 3, 6, 7, 8, 10"
written_before_code: true
last_updated: "2026-10-09"
---

# C1-U1 plan — what gets built, what is locked first, what is out of reach

This plan was written at `114a84e3cb3e9e5fb7f23ee17f044f8d46337bc7` before any source file was edited, as §六 Step 1 of
the authorization requires. Every line number below was opened at that head in this round, not inherited from the round
that wrote `04_TECH/28` (`04_TECH/28:20-22` cites `d41284e`, and §一.3 forbids trusting it unread).

## 1. Authority and unit

§执行定位 grants exactly one unit: **C1-U1 (Data + Identity)** of the design `ADR-0030` and `04_TECH/28` froze. It is
not a re-argument of the direction, not `C1-U2`/`U3`/`U4`, not a new cohort freeze, and not a launch of BIN/HEX as a
product capability. The authorization names the expected handover HEAD `114a84e…`; §一.2 makes a mismatch a STOP.

The archive of the authorization is
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U1_Data_And_Identity_Implementation_v1.0.txt` — measured
**15,072 bytes, 132 `LF`, 0 `CR`, ends with a newline**, SHA-256
`61a33d5381eca5828f8e6c979c25e58db6c2289b88fddccb37cdc71a0c484b65`, stored as blob `76ecf114c46311803c30dc322fc81aa0c3c28ac9`,
`cmp` identical against the delivered file. It arrived under a percent-encoded name, so the digest and `cmp` are the
provenance and the filename is not.

## 2. Preflight, measured

| check | command | result |
| --- | --- | --- |
| local head | `git rev-parse HEAD` | `114a84e3cb3e9e5fb7f23ee17f044f8d46337bc7` = the expected handover HEAD |
| remote head | `git ls-remote origin main` | the same value, so `main` is the formal baseline and no divergence STOP opens |
| cleanliness | `git status --porcelain` | empty before this round wrote anything |
| worktrees / stashes | `git worktree list`, `git stash list` | one worktree on `main`; no stash |
| prior round's CI | `gh run list` | `37955752208` (#91, `b5f732d`) and `37957442415` (#92, `114a84e`), both `success` |
| migration numbers | `ls crates/firmwaresight-storage/migrations/` | `0001`–`0005` exist, `0006` is free |
| partial C1-U1 | `grep -c -i attachment crates/firmwaresight-core/src/domain/gate.rs` | **0** — the unit is genuinely unstarted, so §一.2's "already partly implemented" STOP does not fire |

## 3. Protection assertions, written before the code that must satisfy them

These are the assertions §六 Step 1 asks for. They are the first tests committed, and they must be **RED for the right
reason** (the feature does not exist yet) and never weakened afterwards.

**PA-1 — the zero-attachment `/1` text is byte-frozen.** The project fixture context
(`crates/firmwaresight-project/src/fixtures.rs:15-49`, one ELF row, clean tagged workspace, default policy) currently
produces, measured at this head:

```text
canonical text bytes   1044
canonical text sha256  72d5c108e9f5b362e2f108b112389a04abfa10934207045926e1a02bd8c5da00
run_id                 gate-72d5c108e9f5b362e2f108b112389a04abfa10934207045926e1a02bd8c5da00
first line             firmwaresight-gate-input/1
artifacts block        artifacts[ · elf known:bbbb…(64) · ]
```

After C1-U1 the same context with an **empty** attachment set must produce exactly these bytes: same length, same
digest, same `run_id`, first line still `/1`, and **no `attachments[` line and no empty block** (U1-03 forbids adding
one for `/1`). The probe that measured them was scratch code and is gone; the constants are what the committed test
carries.

**PA-2 — the released goldens are unchanged, unmodified, and still reproducible.**
`golden/reports/p4-release/gate-results.json` (4,803 bytes) carries
`run_id = gate-a17e9861ce643da5e7061ac349793f80d1be87736ad52b2daf3433dc8cb8bc26` and
`snapshot_id = snap-4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374-p0-normalize-…`. This round does not
run `scripts/update_goldens.py`. The proof is the gate step `drift/goldens unchanged`, which compares the committed
goldens against what the binary emits: if any attachment work moved a byte of them, that step reddens.

**PA-3 — `SnapshotId::compose` is untouched.** `crates/firmwaresight-core/src/domain/build_snapshot.rs:34-41` seals
primary + optional MAP and nothing else; §STOP-2 fires the moment the unit seems to need it changed. No call site,
signature or golden changes.

**PA-4 — the policy fingerprint does not move (T-C1-16).** `policy_sha256`
(`crates/firmwaresight-project/src/fingerprint.rs:28-30`) hashes `canonical_policy_text`
(`gate.rs:547`, measured at this head; `04_TECH/28` quoted `d41284e` numbers), which describes the policy alone. No `[artifacts]` key, no size cap, no strictness switch is added
(§7.6), so a config's fingerprint before and after this unit must be identical. The test asserts the equality against a
literal digested value taken at this head.

**PA-5 — no Gate rule changes.** `evaluate` (`gate.rs:640-667`) and the ten rules, `EffectiveSeverity`, the five states
and `unknown.count` aggregation stay as they are. `attachments` is carried by the context and canonicalized; nothing in
`rule_required_artifacts` (`gate.rs:959`) or `rule_artifact_hashes` (`gate.rs:1008`) reads it in this unit — reading it is
`C1-U2`'s work. Consequence, stated plainly: **`required = ["bin"]` still BLOCKs after C1-U1.**

**PA-6 — the release path does not change.** `canonical_release_text` (starts at `release.rs:334`) keeps
`firmwaresight-release-input/1`, `gate_run=` and the `artifact=` lines untouched; no bundle code accepts an attachment,
so **the bundle still cannot ship a BIN/HEX**, which is `C1-U2`'s and `C1-U3`'s boundary.

## 4. Source whitelist, with the reason each file is touched

Nothing outside this table may appear in the diff. §三 allows caller edits for compile compatibility and forbids any new
user behaviour.

| path | why | expected external behaviour |
| --- | --- | --- |
| `crates/firmwaresight-core/src/domain/gate.rs` | `GateAttachmentFact`, `KindBasis`, `GateContext.attachments`, the conditional `attachments[ … ]` block, the `/1`-vs-`/2` label rule, the canonical ordering helper | none: no rule reads the set |
| `crates/firmwaresight-project/src/evidence.rs` | `ReleaseAttachment`, `observe_attachment`, `AttachmentError` (E-1/E-2/E-3), `GateRunRequest.attachments`, canonical assembly in `build_context` | none: no UI or CLI surface calls it yet (§二 U1-02) |
| `crates/firmwaresight-project/src/lib.rs` | export the new project types if the module layout requires it | none |
| `crates/firmwaresight-project/src/fixtures.rs` | explicit `attachments: Vec::new()` on the shared fixture | none |
| `crates/firmwaresight-project/src/fingerprint.rs` | tests only: PA-1 constants and the `/2` determinism locks | none |
| `crates/firmwaresight-project/tests/` (one new attachment test file) | T-C1-01/02/03/05-subset and the typed refusals, at the Project layer | none |
| `crates/firmwaresight-storage/migrations/0006_release_attachments.sql` | the additive `gate_run_attachments` table | none until a run binds attachments |
| `crates/firmwaresight-storage/src/db.rs` | `SCHEMA_VERSION` 5 → 6, `MIGRATION_0006`, one entry in `MIGRATIONS` | store reports v6; the Help page renders the store's own version from a DTO, so it follows without a code edit |
| `crates/firmwaresight-storage/src/gate.rs` | write attachment rows inside the existing `persist_gate_run` transaction, read them into `StoredGateRun`, extend `matches_existing` | a stored run keeps its facts |
| `crates/firmwaresight-storage/src/lib.rs` | export the stored attachment row type | none |
| `crates/firmwaresight-storage/tests/gate_history.rs`, `…/history_reads.rs`, `…/release_records.rs`, `…/compare_candidates.rs`, `…/integrity_and_backup.rs` | drafts gain the empty attachment field; the named-stage migration list gains `0006` with its stage named; the v4-store replay proves the upgrade | none |
| `crates/firmwaresight-core/src/domain/gate.rs` tests, `crates/firmwaresight-report/tests/gate_schema_contract.rs`, `crates/firmwaresight-project/tests/bundle_builder.rs` | struct-literal sites that must name the new field explicitly (§二 U1-03's "explicit empty attachments" rule) | none |
| `apps/desktop/src-tauri/src/release.rs` | the one production `GateRunRequest` and `GateRunDraft` construction gains an explicit empty set | none: the desktop attaches nothing, and no DTO or IPC changes |
| `apps/cli/src/main.rs` | same, for the CLI's Gate run construction | none: no `--attach` flag appears (§二 U1-02 forbids it) |
| `04_TECH/02_DOMAIN_MODEL.md`, `04_TECH/15_STORAGE_DATABASE_BASELINE.md` | the two documents `04_TECH/28:515` assigns to this unit | — |
| `C1_VALIDATION/` (this plan + the round's execution report) | §五's "本轮记录"; §八's delivery content needs a durable home beside the conversation | — |
| `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`, `.ai/HANDOFF.md`, `.ai/ACTIVE_TASK.md`, `.ai/README.md`, `INDEX.md`, `BASELINE.yaml`, `10_AUDIT/SOURCE_PROMPTS/README.md`, the archived prompt, `DIRECTORY_TREE.txt`, `SHA256SUMS` | governance pointers, ledger and the ADR-0029 artifacts, generated only by `scripts/generate_baseline_artifacts.py` | — |

**Explicitly not touched, and the reason it matters:** `schemas/`, `golden/`, `fixtures/`, `assets/`, `templates/`,
`scripts/`, `.github/`, every `Cargo.toml`/`Cargo.lock`/`package.json`/`pnpm-lock.yaml`/`tauri.conf.json`/`deny.toml`,
`DESIGN.md`, `design-tokens.json`, `.gitattributes`, and `09_ADR/ADR-0030` + `04_TECH/28` (their normative text is not
rewritten to fit code — §五). No new dependency, no network, no Tauri capability, no IPC command, no React file, no
migration rewrite, no destructive statement.

## 5. The `/2` grammar this round adds, fixed before it is coded

```text
firmwaresight-gate-input/2          ← only when at least one attachment row exists
snapshot=…
baseline=…
artifacts[
  elf known:<sha>
]
attachments[
  bin known:<sha>
  hex known:<sha>
]
git.available=…                     ← every later line is unchanged from /1
```

Ordering is `(kind word, lowercase sha256)`; an exact `(kind, sha256)` pair appears once; two files of one kind with
different bytes both appear; the name, path, mtime, inode and size are absent from the block (size is stored and checked,
not hashed, per §5 rule 1). Sorting and deduplication happen in one place — a Core helper the Project assembly calls —
so the vector `GateContext` holds is already canonical, and storage's `ordinal` is that vector's index. A rename of
identical bytes must leave `run_id` alone; a changed byte or a re-declared kind must move it.

## 6. Migration `0006`, and the two places where the sketch is tightened deliberately

`04_TECH/28:325` says the sketch "is written by C1-U1, not here", and U1-04 says to review rather than copy it. Two
tightenings, both inside the approved design's intent, both listed so a reviewer can refuse them:

1. `sha256` gets a lowercase-hex CHECK, not only `length(sha256) = 64`, because U1-04 asks for "64 位 lowercase hex
   摘要" and a 64-character uppercase row would otherwise be storable while being a different Gate identity than the
   text the digest was taken from.
2. `kind` keeps `('bin','hex','unknown')` and `byte_size > 0` exactly as sketched; `ordinal` stays part of the primary
   key so a re-read restores canonical order without sorting again.

`gate_runs.id`'s length-69 CHECK (`0003_gate_history.sql:31`) keeps holding — `gate-` plus 64 hex is 69 — and the
`artifacts` table is not reused, because its `parser_id`/`architecture`/`bitness`/`endianness` NOT NULL columns would
force invented analysis values onto a BIN row.

## 7. Spec-versus-source notes reconciled before coding, not after

- **`observe_attachment` takes a file path, not `(project_root, relative)`.** `04_TECH/28:84` writes the signature
  shorthand as `observe_attachment(project_root, declared_kind)`. U1-02 requires observation of "用户显式选择的单个文件",
  and an attachment is not a project-relative config path the way `release_notes_path` is (`evidence.rs:270-302` reads
  that one *inside* the root because the config named it). So the entry point takes the chosen path plus the declared
  kind, keeps no path in the returned fact, and never joins or scans. This is the same shape §4.4 requires; it is
  recorded here because it is a reading of the spec, not a line in it.
- **The attachment errors are their own enum, not `BundleError` variants.** §4.3's table lists E-1…E-4 beside reused
  `BundleError` variants, but the reused ones are all export-time checks (`C1-U2`). A `U1` observation refusal added to
  `BundleError` would put bundle-path vocabulary into a unit that must not touch the bundle flow, so `AttachmentError`
  lives with the observation entry, with `stable_code()` continuing the release family (`ERR-BUNDLE-6115/6116/6117`).
  `C1-U2` decides how — or whether — they surface through `BundleError`.
- **Two tests enumerate migrations by name.** `compare_candidates.rs:633-642` asserts the exact file list and
  `names.len() == SCHEMA_VERSION`; `map_companion_persistence.rs` asserts the applied `(version, name)` pairs against
  `SCHEMA_VERSION` symbolically. The first is a named-stage guard whose comment says a migration appearing without a
  stage is exactly what it catches — so `0006` is added **to** the list with its stage named, keeping the guard whole.
  Weakening it (deleting the assertion) would be the forbidden move.
- **`SCHEMA_VERSION` 5 → 6 is the additive migration's own consequence,** not a schema major: §三 forbids "Schema major"
  meaning the portable JSON contracts, which stay at version 1 each. Nothing user-facing computes from the constant
  except the store's own reported version, which comes from `schema_migrations` (`counts.rs:99-107`), deliberately read
  from the file rather than from the build constant.

## 8. Minimal test scope, and what each test proves

| id | proves | layer |
| --- | --- | --- |
| PA-1 / T-C1-01 | the `/1` bytes, digest and `run_id` above reproduce with an empty attachment set | Project unit |
| T-C1-02 | same snapshot, policy and Git, only the BIN bytes changed ⇒ `/2` `run_id` changes; `snapshot_id` identical | Project unit |
| T-C1-03 | enumeration order irrelevant; exact `(kind, sha)` deduped; distinct-sha same-kind both bound; rename of identical bytes keeps the id; re-declared kind changes it | Project unit |
| T-C1-05 (U1 subset) | a context rebuilt from stored attachment rows plus snapshot rows reproduces the same canonical text and `run_id` | Storage integration |
| T-C1-16 | the policy fingerprint of an unchanged config is unchanged across the unit | Project unit |
| refusal tests | an ELF or MAP offered as an attachment, a 0-byte file, a directory, and an unreadable path each return a typed error and produce **no** fact, no digest and no structural field | Project unit |
| storage tests | atomic write, re-read in ordinal order, same-id-different-attachment-set refused as an invariant, migration idempotency, transaction rollback, the `kind`/`sha256`/`byte_size` CHECKs, and the existing v4-store replay reaching v6 | Storage integration |

One deliberate mutation proof per §四's closing paragraph: after the tests are green, `KindBasis`-aware ordering, the
dedupe, and the `byte_size > 0` CHECK are each broken in turn to show the tests bite, then fully restored with no residue
in the commit.

## 9. What this unit must not be reported as

No BIN/HEX analysis; no snapshot row that is not ELF or MAP; no `attachment:` evidence locator in a finding; no
`extensions.attachments` in any portable document (that is `C1-U3`, and T-C1-05's external-JSON half is explicitly not
claimed here); no UI section, no `--attach`, no IPC command (`C1-U4`); no bundle containing an attachment (`C1-U2`).
`BIN_HEX_RELEASE_ATTACH` moves only from `DESIGN_APPROVED / NOT_IMPLEMENTED` to
`DESIGN_APPROVED / NOT_USER_AVAILABLE`, and `BIN_HEX_ANALYSIS` stays `UNSUPPORTED`.

## 10. Runtime posture

This unit is verified by Rust tests at the Core, Project and Storage layers. Nothing is installed, launched or
screenshot: the attachment observation path has no user-facing entry point until `C1-U4`, so **the desktop and CLI
binaries are not run against real files by this round, and `NOT_RUNTIME_VERIFIED` is the honest label for the installed
behaviour** — even though the code paths themselves execute under test.

---

## 11. Addendum, 2026-10-09: the ERR-BUNDLE registry check and the four readings this plan got wrong

This section is additive. Nothing above is rewritten, so the plan keeps the answer it gave before the
code existed. The mid-round authorization
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U1_ERR_BUNDLE_Code_Check_And_TDD_Continuation_v1.0.txt`
(10,162 bytes, 84 lines, 0 CR, terminates with a newline, sha256
`79a91df69ff80dbd19fcb12c724b52630a9c52dda92e6a49db0dee3418ec3c0a`, blob `c49c589d8c0955612e4fa2d8ccbf24273bd020b6`)
asked for the error-code registry to be located and measured before any number was allocated.

### 11.1 Where the codes are actually registered

There is no registry document. The authoritative map is the Rust `code()` matches themselves, and the
local line numbers measured in this worktree are:

| codes | site |
| --- | --- |
| 6101, 6110 | `crates/firmwaresight-core/src/domain/release.rs:600-601` |
| 6102..6109 | `crates/firmwaresight-project/src/bundle.rs:171-178` |
| 6105 again, 6113 | `apps/desktop/src-tauri/src/lib.rs:730` (`bundle_missing`), `:749` (`destination_missing`) |
| 6111, 6112, 6114 | `apps/desktop/src-tauri/src/bundle.rs:317, 367, 388` |

`05_ENGINEERING/03_ERROR_MODEL.md` holds no `ERR-*-NNNN` literal at all; its only adjacent line is `:34`,
the presentation contract item "Diagnostics ID / copy details". `04_TECH/21_OBSERVABILITY_DIAGNOSTICS.md`
holds none either. So §四.5's warning was real: that document is a taxonomy, not a table, and it does not
need a wholesale rewrite to admit three more codes. The nearest registry-shaped text in the repository is a
historical tally row, `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md:474` (`BUNDLE | 6101–6114 | 14`), which is a
closed stage report and was not edited.

`grep -rhoE '"ERR-BUNDLE-[0-9]{4}"' apps crates` yields exactly fourteen distinct codes, 6101 through 6114.
6115, 6116 and 6117 appear in no Rust source: before this addendum the only occurrence anywhere was line 171
of this plan, which is a proposal of mine and not an implementation. No duplicate, no same-code-different-
meaning and no different-code-same-meaning exists in the family, so no blocking conflict was found and the
allocation in §11.2 proceeded.

### 11.2 The allocation

`ERR-BUNDLE-6115` is E-1 `AttachmentUnreadable`, `6116` is E-2 `AttachmentKindNotAttaching` and `6117` is
E-3 `AttachmentEmpty` (`04_TECH/28` §4.3). E-4 `AttachmentSetChanged` is not represented: it is a
preview-to-export membership change and belongs to C1-U2, and its future number is deliberately unassigned.
All three live in `crates/firmwaresight-project/src/evidence.rs` as `AttachmentError`, which is a Project-layer
enum rather than a `BundleError` variant, so this unit's refusal cannot depend on the export path.

### 11.3 Four readings above that the source corrected

1. **`stable_code()` is the wrong method name.** §7 above says `AttachmentError` would carry
   `stable_code()`. That is the `firmwaresight-artifact` and `firmwaresight-storage` convention; the Project
   layer uses `code()` plus `remediation()` (`bundle.rs:168` and `:186`, `error.rs:68`). The implementation
   follows the Project convention.
2. **There is no CLI `GateRunDraft`.** §4's table lists `apps/cli/src/main.rs` as a caller that would gain an
   explicit empty set. Measured: `persist_gate_run` has exactly one production caller,
   `apps/desktop/src-tauri/src/release.rs:255`. The CLI stores no run, so it changed not at all.
3. **Four tests enumerate the migrations, not two.** §7 named `compare_candidates.rs` and
   `map_companion_persistence.rs`. The full set also includes `release_records.rs`'s applied-pair list and
   `apps/desktop/src-tauri/tests/real_artifact_intake.rs:762`. Each gained `0006` with its stage named; none
   was weakened, and §四.4 of the original authorization's uniqueness requirement is met by
   `the_new_codes_take_no_number_that_an_existing_refusal_already_uses`, which calls the engine's own `code()`
   rather than copying strings.
4. **The schema number is asserted by more than the lists.** `release_records.rs` and `unknown_reasons.rs`
   pinned `SCHEMA_VERSION == 5` by literal, and two Desktop tests hand-build a v4-shaped file by removing
   what the later stages add. All of these are the additive migration's own consequence and had to move with
   it, which is a wider path list than §4 predicted.

### 11.4 The one user-visible consequence of the number, stated plainly

`pre_migration_backup_path` names the copy after the transition (`backup.rs:38`), so a v4 store opened by this
build is now snapshotted as `firmwaresight-p0.pre-migration-v4-to-v6.sqlite` where the installed P5 proofs
recorded `…v4-to-v5.sqlite`. `apps/desktop/src-tauri/tests/diagnostics.rs` asserts that name and moved with it.
Nothing else about the backup rule changed: the copy is still taken before the first pending migration, still
verified, and still reported by name rather than by folder.

Two assertions were considered and deliberately left alone. `apps/desktop/ui/src/help.test.tsx:616` uses a
`…v4-to-v5.sqlite` string as *input* to a component that displays whatever the store reports, and
`apps/desktop/src-tauri/src/startup.rs:334` fabricates an error carrying that name to test redaction. Neither
asserts the product's naming, so both still say what they meant to say. (The same `startup.rs` file *is* in
§11.5, because a different test in it builds a v4-shaped store by hand.) They are named here so the next round
can refresh the example text rather than inherit it as a mystery.

### 11.5 Paths this round touched that §4's whitelist did not predict

`crates/firmwaresight-storage/tests/unknown_reasons.rs`, `apps/desktop/src-tauri/src/startup.rs`,
`apps/desktop/src-tauri/tests/diagnostics.rs` and `apps/desktop/src-tauri/tests/real_artifact_intake.rs`.
Every one is a schema-number or migration-list consequence of the authorized `0006`, each change is the
addition of the sixth named stage or the sixth version, and none of them changes a verdict, a DTO, a
capability or a user-facing behaviour. This is disclosed rather than smoothed over because §4 of the
original authorization asked for the whitelist to be stated before coding, and a plan that turns out to be
incomplete is reported as incomplete.

## 12. Addendum, 2026-10-09 — the four mutation proofs, and what one of them found

§4 of the execution authorization permits a temporary test mutation to prove that a test actually
discriminates, and requires that it be "完整恢复，提交前无残留". Four were run. Each mutated file's
SHA-256 was taken before the mutation (`target/c1_u1_mutation_pre.sha`) and re-checked after the
restore, and the whole suite was re-run afterwards. Logs:
`target/c1_u1_mutation_0_baseline.txt` through `target/c1_u1_mutation_5_restored.txt` (untracked,
under `target/`, not part of the commit).

| # | Mutation | Command | Result |
| --- | --- | --- | --- |
| M1 | `canonicalized()` sorts by the digest alone, dropping the kind word (`crates/firmwaresight-core/src/domain/gate.rs:477`) | `cargo test -p firmwaresight-project --test release_attachments` | **First attempt: the suite stayed green.** See the disclosure below. After the test was strengthened: `FAILED`, `left: ["hex 000…", "bin 111…"], right: ["bin 111…", "bin 999…", "hex 000…", "hex 333…"]` |
| M2 | the `dedup_by` predicate never matches, so an exact `(kind, digest)` pair is bound twice (`gate.rs:482`) | `cargo test -p firmwaresight-project --test release_attachments` | `an_exact_kind_and_digest_pair_is_bound_once_however_many_times_it_arrives ... FAILED` (18 passed, 1 failed) |
| M3 | `0006_release_attachments.sql:37` `CHECK (byte_size > 0)` → `>= 0` | `cargo test -p firmwaresight-storage --test gate_history -- the_attachment_table_refuses` | `the_attachment_table_refuses_an_illegal_row_at_the_boundary ... FAILED` |
| M4 | the empty-set guard is removed, so `/1` emits an `attachments[` block with nothing in it (`gate.rs:786`) | `cargo test -p firmwaresight-project --lib -- the_zero_attachment_canonical_text`, then `cargo test -p fwsight --test p4_golden` | `the_zero_attachment_canonical_text_is_byte_frozen ... FAILED`, `an_empty_attachment_set_keeps_the_label_at_one_and_writes_no_block ... FAILED`, and **4 of the 11 frozen p4-release golden tests FAILED** — the historical bundle that predates this unit is the third authority refusing the change |

Restoration: `sha256sum -c target/c1_u1_mutation_pre.sha` reports OK for all three mutated paths,
`cargo fmt --all --check` is clean, and `cargo test --workspace` afterwards reports 48 result lines
and 900 passed, 0 failed.

### 12.1 Disclosure: M1 did not fail at first, so the test was changed

The ordering test written in Step 2a asserted only that the first row started with `bin `, using
digests whose order happened to agree with the kind order. Deleting the kind word from the sort key
therefore passed the entire suite. That is a non-discriminating assertion about the single rule
`04_TECH/28` §5 rule 1 states most precisely, and the mutation found it. The test now pins the whole
sequence — `bin sha('1')`, `bin sha('9')`, `hex sha('0')`, `hex sha('3')` — with digests deliberately
chosen so that the kind key and the digest key disagree, which is what makes M1 fail. The fix is in the
test, not in the production rule, and the production rule was not touched to make the test pass.

### 12.2 Disclosure: the storage ordinal test is a self-consistency check

`gate_history.rs::a_stored_run_keeps_its_attachment_rows_in_the_canonical_order` compares the stored
`ordinal`s against `canonicalized()`'s own output, so under M1 it stayed green — the same mutated
function produced both sides of the comparison. It is worth keeping for what it does prove (the write
path stores the order it was given, and the read path returns it unchanged), but it must not be cited
as the guard for the ordering rule. The Project-level test above is that guard, and the frozen `/1`
constants plus the p4 goldens are the guard for the block-emission rule.

### 12.3 What this section is not

No mutation was left in, no assertion was deleted to accommodate one, and none of the four produced a
product defect: three were caught by the tests written for them and the fourth was caught only after its
test was strengthened. The temporary files under `target/` are this round's own scratch and are ignored.
