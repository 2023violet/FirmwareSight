---
title: "Release Attached Byte Evidence"
doc_id: "FS-TECH-029"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-09"
---

# Release-Attached Byte Evidence — the C1 design baseline

## 1. Scope and authority

This document is the technical specification `ADR-0030` froze. It specifies **Release-Attached Byte Evidence**
(option C1): a release owner attaches the `.bin` and Intel HEX files they actually ship, FirmwareSight records their
raw bytes, byte size and SHA-256, binds them into the Gate run identity and the release identity, verifies them into
the bundle, and claims nothing else about them.

It is a specification, not an implementation. Nothing here is built. Every file path and line number cited was read at
head `d41284e54f723d74687cf351d85f1f2fc0b0be2a` on 2026-10-09, and each citation names today's behaviour — the
behaviour a C1 unit must inherit or extend, never one it may quietly drop. Status of the capability:
**`BIN_HEX_RELEASE_ATTACH = DESIGN_APPROVED / NOT_IMPLEMENTED`**, and this document is `NOT_RUNTIME_VERIFIED`: no
build, install or UI run was performed to write it.

Two vocabulary notes carried forward deliberately:

- The four implementation units are named **`C1-U1`…`C1-U4`**. The Owner's authorization named them U1–U4; the `C1-`
  prefix exists because `U1` is already the identifier of a closed stage in this repository
  (`.ai/CURRENT_STATE.md`, `U1_VALIDATION/`), and reusing that name would make two different things callable U1.
  The four units, their order and their boundaries are exactly as the authorization set them.
- **Attachment** in this document never means the MAP attachment. The existing MAP is an *analysis input* supplied
  beside an ELF (`pipeline.rs:105-115`). A release attachment is not analyzed at all.

## 2. Entities, vocabulary and types

| entity | what it is | authority | owner crate |
| --- | --- | --- | --- |
| `BuildSnapshot` | one analyzed ELF plus an optional GNU ld MAP. Unchanged by C1 | Observed analysis | `firmwaresight-core` |
| `ReleaseAttachment` | one file a release owner chose to ship, with its size and SHA-256, and no structural claim | Observed bytes, Declared kind | `firmwaresight-project` |
| `GateContext.attachments` | the fact vector the Gate judges and canonicalizes | Derived from the above | `firmwaresight-core` |
| `VerifiedRow` (attachment form) | a shipped attachment re-read and re-checked at packaging | Observed | `firmwaresight-project` |

Shape of the new Core fact type, deliberately parallel to `GateArtifactFact` (`gate.rs:405-413`) but not merged with
it, because the two answer different questions and their prose must stay distinguishable:

```text
GateAttachmentFact {
    kind:       ArtifactKind,          // Bin | IntelHex | Unknown only — see §2.2
    sha256:     Fact<String>,          // Observed: digest of the bytes actually read
    byte_size:  u64,                   // Observed: length of those bytes
    kind_basis: KindBasis,             // Declared | DerivedFromLeadingBytes
}

KindBasis  = Declared                       // the release owner chose this kind
           | DerivedFromLeadingBytes(String) // intake's `detect_format` shaped it (intake.rs:179-202)
```

### 2.1 Kinds

`ArtifactKind` already contains `Bin` and `IntelHex` (`identity.rs:115-121`), and the canonical words already map:
`bin` and `hex` at `gate.rs:1492-1500` and `identity.rs:127-135`. No new kind and no new word.

### 2.2 Which kind an attachment may carry

An attachment may be `bin`, `hex` or `unknown`. It may **not** be `elf` or `map`. Those two kinds are produced only by
the analysis path, and accepting them as attachments would let a file nobody parsed satisfy `artifacts.required` for the
ELF a release is supposed to be about — the exact decoupling `ADR-0030` D-3 forbids. The refusal is a named error
(`§4.3 E-2`), not a silent drop.

### 2.3 What an attachment is never used for

Not `analysis.json`, not `diff.json`, not Compare, not `memory.flash_budget` / `memory.ram_budget`, not `diff.growth`,
not the `unknown.count` of `evidence.unknown_review`, and not `SnapshotId`. `§11` lists the conclusions this makes
unreachable.

## 3. The one production path

There is exactly one path per stage. A second path is a design change requiring a successor to `ADR-0030`.

| stage | single path | file that owns it |
| --- | --- | --- |
| selection | the release owner names one file in a file dialog (desktop) or repeats `--attach <FILE>` (CLI). No directory scan, no glob, no recursive attach | `apps/desktop/src-tauri/src/intake.rs:35`, `apps/cli/src/main.rs` args |
| observation | `observe_attachment(project_root, declared_kind) -> GateAttachmentFact`: stat → regular file → non-empty → streaming SHA-256 → length | `crates/firmwaresight-project/src/evidence.rs`, beside `observe_release_notes` (`:270`) |
| judging | `GateRunRequest.attachments` → `build_context` (`evidence.rs:362-380`) → `GateContext.attachments` → `evaluate` and `run_id` (`fingerprint.rs:34-39`) | `firmwaresight-project` assembles, `firmwaresight-core` decides |
| storing | the run's attachment rows are written with the run, so it can be re-judged and exported later | `crates/firmwaresight-storage`, migration `0006` (§7.6) |
| planning | `BundleRequest.attachments` → `observe()` → `verify_attachments()` → the same `shipped` list → `ReleaseArtifact` rows → `bundle_names()` | `crates/firmwaresight-project/src/bundle.rs:650-670`, `:1035-1054` |
| shipping | preview→publish re-check (`InputChecks`), then `SHA256SUMS` + `release-manifest.json` byte verification | `bundle.rs:519-543`, `:1506-1560` |

Observation reads the file **twice at most and never into memory as a parse target**: once streamed for the digest, once
streamed again at export for the re-check. `GuardedInput::load` (`intake.rs:99-174`) is the analysis-side reader and is
**not** reused: it allocates the whole file for parsing, and its `DEFAULT_MAX_FULL_BUFFER_BYTES` 512 MiB guard
(`intake.rs:23`) is a parser limit, not an evidence limit. An attachment is hashed by
`fingerprint::file_sha256` (`fingerprint.rs:42-56`), which streams a 64 KiB buffer and holds nothing.

### 3.1 Data flow

```text
                       ┌──────────────── ANALYZE (unchanged by C1) ───────────────┐
 ELF ──┐               │ intake.rs  pipeline.rs      build_snapshot.rs              │
       ├─ GuardedInput─▶ detect ──▶ elf::parse ──▶ seal ──▶ BuildSnapshot          │
 MAP ──┘  (full read)   │  non-ELF refused: pipeline.rs:83-97                       │
                        └───────────────────────────────┬──────────────────────────┘
                                                        │ snapshot artifact rows (elf, map)
                        RELEASE PREPARATION             │
                        ┌───────────────────────────┐   │
 .bin ─┐                │ observe_attachment()      │   │
 .hex ─┴─ user pick ──▶ │ stat · regular · non-empty│   │
 (no read of content)   │ streaming sha256 · length │   │
                        └────────────┬──────────────┘   │
                                     │ attachment rows  │
                                     ▼                  ▼
                        GateContext{ artifacts, attachments }
                                     │
                    canonical_input()│ gate.rs:676-729   /1 when attachments empty
                                     │                   /2 when non-empty
                                     ▼
                     run_id = gate-<sha256> ──▶ evaluate() ──▶ 10 findings
                                     │                          artifacts.required now satisfiable
                                     ▼                          by an attachment row
                        stored with the run (migration 0006)
                                     │
                                     ▼
              prepare() ─▶ verify_sources() + verify_attachments() ─▶ BundlePlan (preview)
                                     │                    ▲
                       publish() ────┤ same two calls again│ InputChecks re-check
                                     ▼
        artifacts/<leaf>  ·  release-manifest.json files[] + extensions.attachments
                                     │        │
                        SHA256SUMS ──┘        └── release id (release.rs:334-399, gate_run= already in it)
                                     │
                        verify_bundle() re-derives the release id from the bundle's own bytes
```

## 4. States, transitions, error model and refusals

### 4.1 Lifecycle

```text
unselected → observed → bound → planned → shipped
                │         │        │
                │         │        └─ export re-check fails ─────▶ rejected(AttachmentChanged | PlanStale)
                │         └─ selected run no longer recomputes ──▶ rejected(GateContextChanged)
                └─ stat / read / emptiness fails ────────────────▶ rejected(AttachmentUnreadable | AttachmentEmpty)
```

`observed` holds a digest. `bound` means the digest is inside a judged Gate context. `planned` means the same row is in
a `BundlePlan`'s `InputChecks`. `shipped` means the bytes are in the bundle and are listed by both `SHA256SUMS` and
`release-manifest.json`. There is no state in which an attachment exists in a bundle without having been bound: §6.

### 4.2 Invalidation table

Which event invalidates which artifact. This is the whole anti-stale-PASS contract in one place.

| event | SnapshotId | Gate run id | stored run | bundle preview | release id | already-written bundle |
| --- | --- | --- | --- | --- | --- | --- |
| attached bytes modified | unchanged | **changes** | stays, immutable | **stale** (`SourceArtifactChanged`, then run id mismatch) | changes | untouched; it is a different release |
| attachment replaced by a different file of the same kind | unchanged | **changes** | stays | **stale** | changes | untouched |
| attachment deleted after judging | unchanged | unchanged for the stored row | stays | **refused** — re-observation fails `AttachmentUnreadable` | cannot be computed | untouched |
| attachment added to a run that had none | unchanged | `/1` → `/2`, **changes** | stays | **stale** | changes | untouched |
| attachment renamed, bytes identical | unchanged | **unchanged** (name is not Gate identity, `ADR-0030` D-5) | stays | **stale** (`InputChecks.sources` carries the recorded path) | **changes** (name is release identity, `release.rs:354-363`) | untouched |
| attachment kind re-declared, bytes identical | unchanged | **changes** (the block prints the kind word) | stays | stale | changes | untouched |
| policy or acceptance set changed | unchanged | changes (already true today) | stays | stale | changes | untouched |
| a second run of the identical inputs | unchanged | identical (D-4 stability) | recognized as `AlreadyStored`, and a run whose *answer* differs under one id is refused as an invariant (`crates/firmwaresight-storage/src/gate.rs:206-238`) | n/a | identical | n/a |

The rename row is the one a release owner will hit first, and the asymmetry is intentional: the Gate judged **bytes**,
the release ships a **package**. Renaming does not change what was judged and does change what ships.

### 4.3 Errors

Existing variants are reused where they already mean the right thing; four are added. All are refusals — no error in
this list updates a stored digest, and none writes a partial bundle.

| id | error | condition | remediation |
| --- | --- | --- | --- |
| E-1 | `AttachmentUnreadable { name, detail }` | stat fails, path is not a regular file, or read fails. Name is `sanitize_leaf_name`, never a host path | select the file again, or close what holds it |
| E-2 | `AttachmentKindNotAttaching { kind }` | an `elf` or `map` was offered as an attachment | analyze the ELF instead; the MAP is attached on the Analyze page |
| E-3 | `AttachmentEmpty { name }` | the file is 0 bytes | a required kind must not be satisfied by nothing; ship the real image |
| E-4 | `AttachmentSetChanged { name, change }` | an attachment was added or removed between preview and publish | re-preview; today's `changed_leaf` (`bundle.rs:554-563`) compares pairwise and cannot name a membership change — this error is what closes that gap |
| reuse | `SourceArtifactChanged` | an attachment's digest or size differs at export | the file changed after the preview (`bundle.rs:520-526`) |
| reuse | `GateContextChanged { selected, recomputed }` | the selected run no longer recomputes from these inputs | re-run the Gate (`bundle.rs:596-604`) |
| reuse | `PlanStale` | run id, policy fingerprint or acceptance set moved | re-preview (`bundle.rs:533-541`) |
| reuse | `Release(NoReleaseArtifacts)` | nothing was shipped | an attachment never satisfies this; the analyzed ELF must still be there (`bundle.rs:1012-1014`) |

E-3 is not a convenience. Without it, a 0-byte `.bin` satisfies `artifacts.required` while shipping no bytes at all,
and a Gate `PASS` would then describe an empty file.

### 4.4 Permissions and privacy

- Selection is per-file, by the release owner, through the existing use-case-oriented dialog. `AGENTS.md` §7 forbids a
  "read arbitrary file" command; C1 adds no directory reader, no glob, no recursive walk and no default path.
- No new Tauri capability, no new plugin, no network (`04_TECH/05`), no telemetry, no absolute host path in any
  portable document or in any error message (`gate.rs:673-674`, `bundle.rs:503-513`).
- Source paths stay inside `InputChecks`, where they are already private to the crate.
- Inside a bundle, `verify_bundle` (`bundle.rs:1462-1504`) keeps refusing absolute paths, escaping relative paths and
  symlinks, and keeps refusing two names that fold to one file on a case-insensitive filesystem.
- The `RawInputBytes` rule of `crates/firmwaresight-storage/migrations/0001_initial.sql:8` is unchanged:
  attachment bytes are never stored in SQLite. Only the digest, size, kind, basis and locator are stored.

## 5. Canonicalization, identity and the rename rule

The Gate canonical input gains one block, emitted only when attachments exist, and the label changes exactly when that
block is present:

```text
firmwaresight-gate-input/1        ← no attachment exists; byte-identical to today's text, always
firmwaresight-gate-input/2        ← at least one attachment row exists

  artifacts[
    elf known:<sha256>
    map known:<sha256>
  ]
  attachments[
    bin known:<sha256>
    hex known:<sha256>
  ]
```

Rules, each of which a test must hold:

1. **Ordering.** Attachment rows sort by `(kind word, lowercase sha256 hex)`. The shipped set already sorts by
   `(kind word, sha256, byte_size, file_name)` at `bundle.rs:664-670`; the Gate block needs no name and no size,
   because the digest already fixes both, and fewer moving parts in an identity is a feature.
2. **Multiplicity.** Several distinct files of one kind are all bound. An exact `(kind, sha256)` duplicate is bound
   once — two rows of identical bytes say one thing twice, and a set is the honest model of what the digest binds.
   All copies still ship as distinct bundle files under `bundle_names()`'s collision rule.
3. **Digest-only.** `sha256` is `known:<hex>` because observation read the bytes; there is no `unknown:` case for an
   attachment, since an attachment that could not be hashed stopped at E-1 and never reached a context.
4. **Not in Gate identity.** file name, directory, host path, mtime, import time, `inode`, process id.
5. **In release identity.** file name, kind word, digest and size, via the `artifact=` line that already exists
   (`release.rs:354-363`), and the `gate_run=` line that already chains the Gate id in (`release.rs:343`).
6. **`release-manifest:1`'s label does not move.** `firmwaresight-release-input/1` stays, because its grammar is a
   list of `artifact=` lines and C1 adds rows, not structure.
7. **Locator.** an attachment is cited as `attachment:<kind>:<sha256>`, parallel to the `artifact:<kind>:<sha>`
   locator of prompt §30 (`gate.rs:1471-1478`), which is not changed and not reused for a file that was never
   analyzed. A locator that called an attachment an artifact would be the prose form of the defect.

## 6. Consistency proof obligations

`ADR-0030` D-9 requires that "the Gate judged A while the bundle ships B" be unrepresentable. The proof is a chain of
six links, each of which exists in code today; C1's obligation is that **no link covers snapshot rows only**.

| # | link | today | C1 obligation |
| --- | --- | --- | --- |
| L-1 | the verdict is computed over the judged context | `evaluate(&run_id)` on the same `GateContext` (`gate.rs:640-667`) | unchanged; the context carries `attachments` |
| L-2 | the context's identity binds every byte judged | `run_id = sha256(canonical_input())` (`fingerprint.rs:34-39`) | attachment rows enter the canonical text (§5) |
| L-3 | a selected stored run must still recompute | `GateContextChanged` (`bundle.rs:596-604`) | unchanged in mechanism; inputs now include attachments |
| L-4 | every shipped row must be in the judged set | `verify_sources`' context cross-check (`bundle.rs:993-1003`) | `verify_attachments` performs the sibling check against `context.attachments` |
| L-5 | preview bytes must equal exported bytes | `InputChecks.compare_with` (`bundle.rs:519-543`) | attachment rows join `sources`; membership change raises E-4 |
| L-6 | the bundle is self-verifying | `SHA256SUMS` + manifest byte checks (`bundle.rs:1506-1560`), `verify_bundle` re-derives the release id | unchanged; an attachment is a bundle file like any other |

Underneath all six sits a storage rule that C1 does not need to change and should quote when it is challenged:
`crates/firmwaresight-storage/src/gate.rs:206-238` stores a run atomically **or recognizes it as already stored**, and
raises an invariant when an id arrives carrying a different answer — in its own words, "a changed policy, workspace or
baseline produces a new run id rather than rewriting this one". A stale `PASS` therefore cannot be edited into a fresh
one; it can only be superseded by a new id, which is exactly what an attachment change produces under §5. That is the
difference between this design being stale-proof and merely stale-discouraged.

Two obligations that are *not* inherited and must be newly proven by `C1-U2`:

- **Symmetry obligation.** Where `verify_sources` checks a row, `verify_attachments` checks it: stat, regular file,
  streamed digest, digest equality, size equality, context membership. Any check present for one class and absent for
  the other is a defect, not an optimization.
- **Same-rules-for-preview-and-export obligation.** Preview and export run the same function on the same request
  (`bundle.rs:486-493` and `publish(&request, …)`), so no rule may be expressed once in each. This is already the
  product's shape; C1's hazard is adding an attachment rule to one call and not the other.

## 7. Contract impact matrix and contract tests

### 7.1 `analysis:1` — no change, and required to stay unchanged

| item | value |
| --- | --- |
| fields | none added, renamed or removed |
| semantics | unchanged: one analyzed build's facts |
| version | major 1 |
| why untouched | its `kind` enum already allows `bin`/`hex` (`schemas/analysis.schema.json:141-149`), so an enum is not the leak; putting an attachment row into `analysis.json` would be the pseudo-analysis `ADR-0030` D-1 forbids |
| contract tests | `T-C1-11` `analysis.json` for a release with attachments contains only `elf` and `map` rows; `T-C1-12` the byte-identical `analysis.json` golden before and after C1 for an attachment-free release |

### 7.2 `diff:1` — no change

Fields, semantics and version unchanged. Attachments never appear in a diff and never affect `diff.growth`.
`T-C1-13` asserts a growth verdict identical with and without attachments.

### 7.3 `gate-results:1` — additive disclosure, major stays 1

| item | value |
| --- | --- |
| fields | unchanged (top level stays `schema_version`, `run_id`, `snapshot_id`, `findings`, `extensions`; `additionalProperties: false` is not loosened) |
| new content, inside `extensions` only | `canonical_input_label: "firmwaresight-gate-input/2"` and `attachments: [{kind, sha256, byte_size}]`, in §5 order, **present iff** at least one attachment was bound |
| semantics of existing fields | unchanged. `run_id` is an opaque `gate-<64 hex>` in the schema and in `04_TECH/15:131`; `snapshot_id` still denotes the analyzed build; `findings[].state` keeps the five states and `effective_severity` stays separate (`ADR-0023`) |
| version decision | **major 1.** The extension carries information that was already bound into an existing field; it does not redefine that field. This is the P3 technique for `policy_sha256` and `overall_effective_severity` (`04_TECH/26:43-54`) |
| rejected alternative | `gate-results:2`, which would force every v1 document — in storage, in `golden/reports/p4-release/`, and in bundles already distributed — to be re-read under a new major, i.e. the silent rewrite §D3 forbids |
| reading rules | no `attachments` extension ⇒ the run bound none ⇒ read exactly as before, which is true of every pre-C1 document. `extensions.canonical_input_label` present ⇒ recompute from the named grammar, and a mismatch is a hard error, never a re-interpretation (`04_TECH/26:31`) |
| contract tests | `T-C1-01` zero-attachment context reproduces the current canonical text and run id byte-for-byte; `T-C1-02` changing only an attached BIN's bytes changes the run id; `T-C1-03` the same inputs twice give the same id; `T-C1-04` a `/1` document never carries `attachments`, and a `/2` document always carries both entries (the discriminability rule `ADR-0030` D-6); `T-C1-05` recomputing `run_id` from `extensions.attachments` plus the snapshot rows reproduces the stored id; `T-C1-06` every pre-C1 `gate-results` document still validates against `gate-results:1`; `T-C1-07` a pre-C1 stored run still re-judges from stored rows after its files are gone (the existing `the_run_is_judged_from_stored_rows_after_the_files_are_gone` test extended to a no-attachment run) |

### 7.4 `accepted-reviews:1` — no change

Acceptances are keyed to `<run_id>#<rule_id>` (`gate.rs:442-443`), so an acceptance granted against a run that bound
attachment bytes can never be silently reused for a run that bound different bytes: the finding id moves with the run
id. `T-C1-08` asserts that an acceptance recorded on one attachment set is refused on another.

### 7.5 `release-manifest:1` — additive disclosure, major stays 1

| item | value |
| --- | --- |
| `files[]` | **no new field.** An attachment is a bundle file, so it is already listed as `{path, sha256, size}` like `artifacts/firmware.elf` is today |
| new content, inside `extensions` only | `attachments: [{file, kind, kind_basis, sha256, size}]`, naming the bundle-relative path each entry describes |
| why `extensions` and not `files[]` | `files` items are `additionalProperties: false`; adding `kind` there is a schema change. `kind` is exactly the fact `files[]` cannot express, and the P4 precedent (`integrity_model`) is the same move in the same field |
| semantics of existing fields | unchanged, including `build.artifact_sha256`, which stays the primary analyzed artifact's digest and must never be repurposed as "some shipped digest" |
| release id | no grammar change, no label change; the `artifact=` rows and `gate_run=` do all the work |
| contract tests | `T-C1-09` a manifest with attachments validates against the unmodified `release-manifest:1`; `T-C1-10` `sha256sum -c SHA256SUMS` over the bundle verifies every attachment; `T-C1-14` `verify_bundle` re-derives the release id for a bundle containing attachments; `T-C1-15` an attachment with the same leaf name as the ELF is disambiguated by `bundle_names`, and both files survive with distinct names on a case-insensitive filesystem |

### 7.6 `project-config:1` — no change, and the reason it matters

`[artifacts] required` keeps its four-word enum (`config.rs:23-25`, `schemas/project-config.schema.json`). Its meaning
—"this kind must be present at release time" — is unchanged; C1 makes it satisfiable rather than redefining it.
Narrowing the vocabulary would have been the semantics change requiring a major, which is one of the reasons option B
was rejected. No new config key: a size cap or a kind-strictness switch would move `policy_sha256`, and therefore every
run id, for a change that has nothing to do with the project's policy. `T-C1-16` asserts the policy fingerprint is
unchanged for an unchanged config across the C1 implementation.

### 7.7 Storage

One additive migration, `0006_release_attachments.sql`, sketch only — it is written by `C1-U1`, not here:

```sql
CREATE TABLE gate_run_attachments (
    run_id      TEXT NOT NULL REFERENCES gate_runs(id) ON DELETE CASCADE,
    ordinal     INTEGER NOT NULL,
    kind        TEXT NOT NULL CHECK (kind IN ('bin','hex','unknown')),
    sha256      TEXT NOT NULL CHECK (length(sha256) = 64),
    byte_size   INTEGER NOT NULL CHECK (byte_size > 0),
    kind_basis  TEXT NOT NULL CHECK (kind_basis IN ('declared','derived_from_leading_bytes')),
    PRIMARY KEY (run_id, ordinal)
);
```

`ordinal` is the §5 canonical position, so a re-read rebuilds the identical context without sorting again. The
`byte_size > 0` CHECK is E-3 enforced at the storage boundary as well.
`crates/firmwaresight-storage/migrations/0001_initial.sql:31-47`'s `artifacts` table is deliberately not reused:
it requires `parser_id`, `architecture`, `bitness` and `endianness` to be `NOT NULL`, so a BIN row there would
have to carry invented analysis values. `gate_runs.id`'s length-69 CHECK (`0003_gate_history.sql:31`) keeps
holding, and the immutability triggers are untouched.

## 8. Evidence grading, declared kinds and the Intel HEX limits

| statement | class | basis |
| --- | --- | --- |
| the file existed at read time, and was a regular file | **Observed** | `stat` (`intake.rs:100-109` precedent) |
| its length is N bytes | **Observed** | the same read that produced the digest |
| its SHA-256 is H | **Observed** | streaming digest of the bytes actually read |
| its kind is `bin` / `hex` | **Declared** | the release owner's choice; retained as declared even when leading bytes agree |
| its leading bytes look like Intel HEX / like ASCII-HEX-shaped text / neither | **Derived** | `detect_format` (`intake.rs:195-201`), with its basis named and its uncertainty stated |
| it was produced by the analyzed build | **Unknown** | no proof path exists; `ADR-0030` D-7 makes this permanent |
| it was produced by the recorded Git commit | **Unknown** | same |
| its Intel HEX records are well-formed, their checksums are valid, its address span, its loaded content, its target device | **Unknown / Not verified** | no parser exists; must never be reported as `0`, as empty, or as absent-because-zero |
| its sections, symbols, memory regions, entry point, object attribution | **Unknown / Not applicable to this class** | these are properties of a container format, not of a raw image |

Kind conflicts. When a Declared kind and the leading bytes disagree, both are kept and the disagreement is shown:
`kind_basis = Declared`, plus a reported conflict naming what `detect_format` saw. No silent reclassification, in either
direction — `04_TECH/24:34` already states that different sources must not overwrite one another and that the UI may show
a conflict. `ADR-0030` D-8 fixes the consequence for the Gate: a `.hex`-named file of non-HEX bytes satisfies the `hex`
requirement **as raw bytes of a kind the owner declared**, and the finding says so; it never asserts valid Intel HEX.

Negative tests for this section: `T-C1-17` attaching an ELF-magic file declared `hex` → `artifacts.required` PASSes on
`hex`, and the summary plus `analysis`-free document still contains no claim that the content is Intel HEX;
`T-C1-18` no finding, bundle page or portable field reports an address span, record checksum or section count for an
attachment; `T-C1-19` attaching a file never changes `unknown.count`, so `evidence.unknown_review` cannot move because
of an attachment (`ADR-0030` D-7).

## 9. Capability wording per surface

Approved wording, which is the wording an implementation unit must ship and a reviewer must hold it to. None of these
sentences may claim more than §8 grades, and none may be softened into "supports BIN/HEX".

| surface | today | target under C1 |
| --- | --- | --- |
| Analyze refusal (`error.rs:86-89`) | "Choose the ELF linker output this build produced; BIN and Intel HEX images are not analyzed by this entry point." | kept verbatim; a following clause adds "Attach them on the Release page to ship them as byte evidence." |
| Analyze file selector | one ELF, optional MAP | unchanged — attachments are not an Analyze concept |
| Release page, existing caption (`Release.tsx:839-841`) | "P3 analyzes ELF and reads GNU ld MAP files; a required BIN or HEX is checked for presence only." | "Required `bin`/`hex` are satisfied by release files you attach; FirmwareSight records their bytes and SHA-256 and does not analyze them." |
| Release page, new section | — | "Release attachments" listing file name, byte size, full SHA-256, evidence class and the limitation sentence; verbs **Attach** and **Ship**, never Analyze; no section, symbol or memory table beside an attachment |
| Gate `artifacts.required` summary | "Every required artifact kind is present in the snapshot (…)" / "…missing from the snapshot" (`gate.rs:983,994`) | names which set satisfied each kind, e.g. "satisfied by a release attachment; bytes verified, provenance not verified" — and it must stop saying "in the snapshot" when it did not come from one |
| Gate `artifacts.hashes` summary | "All N snapshot artifact(s) carry a SHA-256 digest." (`gate.rs:1023`) | counts and locates both classes, with `attachment:` locators distinguishable from `artifact:` |
| Bundle preview and export | shipped rows under `artifacts/` | the same list with kind, size, full hash and evidence class shown for attachments exactly as for analyzed rows |
| `release-report.html` limits table | states ELF/MAP-only analysis | gains one line: attachments ship as byte evidence; their structure and provenance are not verified |
| CLI | `--artifact`, `--map`, `--baseline`, `--baseline-map`, `--out` | repeatable `--attach <FILE>` accepted by `gate` and `release prepare`; exit codes unchanged (`main.rs:434-440`) |
| Compatibility matrix (`P5_COMPATIBILITY_MATRIX.md:68`) | `HEX / BIN / UF2 / Mach-O / PE — UNSUPPORTED` | stays exactly as written until an implementation round produces a fixture and a measured run; `C1-U4`-era evidence then moves BIN/HEX to `SUPPORTED_WITH_LIMITS` with the limits named in the evidence column. The column is closed to the five normalized status values, so `NOT_IMPLEMENTED` is not written into it — it is the `.ai/` and `BASELINE.yaml` state word |
| Storage invariant message (`crates/firmwaresight-storage/src/gate.rs:232-233`) | "a changed policy, workspace or baseline produces a new run id rather than rewriting this one" | gains the fourth cause, "or attachment set", in the same sentence — it is the one message in this design that a release owner reads when a stale `PASS` is refused, and naming the cause is the point |
| Help / Getting Started | format scope sentences | one sentence pointing at `ADR-0030`; no new page, no new navigation |

Design-rule compliance (`AGENTS.md` §11): four verbs unchanged, no new theme or token, no magic colour or spacing,
mono for hashes and numbers, five states as icon + label, Unknown rendered as neutral hollow with a fact sentence and a
next step — which is exactly how "provenance not verified" must appear. React never computes a digest, a diff or a
verdict: every value on these surfaces arrives from a DTO.

## 10. Boundaries to other stages, and the frozen V1 cohort

C1 is design here and nothing else, and it is not a reason to touch any running stage:

- **V1** stays paused at recruitment: `IN_PROGRESS / RECRUITMENT_READY`, 0 eligible external sessions, M1–M6
  `NOT_MEASURED`, cohort build `11573661113` at product head `41bb6a36`, NSIS `3,896,257` bytes /
  `9a51e86a…c87d93`, unsigned. This round built nothing, installed nothing and re-froze nothing.
- The cohort build still ships the **pre-A0** refusal string ("Provide an ELF linker output, or a BIN/HEX image"),
  because it predates A0. Participants will therefore see copy the product has already corrected. That is a known,
  recorded consequence of the freeze and **not** an argument for swapping the instrument: a build identity is not a
  verdict, and format-scope work is not a reason to re-freeze.
- **U1** keeps its architect verdict and its guarded 25-item tally. C1's UI changes are new surfaces, not a reopening
  of that acceptance, and they will need their own installed visual evidence.
- **P5 / G2** closures are unchanged, and the compatibility matrix keeps its status vocabulary.
- **B1, RC, GA, signing, notarization, updater, licence** remain `NOT_AUTHORIZED`.

## 11. Excluded conclusions, and rollback to C3

No reading of a C1 document, bundle or screen may establish: that a BIN/HEX file has sections, symbols, a memory
footprint, an entry point or an architecture; that an Intel HEX file's records, checksums, address span or loaded
content were checked; that an attachment was built by the analyzed build, the recorded commit, or any particular
toolchain; that two attachments of the same kind are consistent with each other; or that `required = ["hex"]` passing
means the release contains a valid HEX image.

Each excluded conclusion is a negative test, not a comment: §7 and §8 name them, and a `C1-U3` regression that deletes
one of those tests must fail.

Rollback to C3 (ship without gating) stays representable without touching any contract, because attachment rows occupy
their own canonical block: stop binding the block, keep shipping the files. That is the withdrawal path `ADR-0030`
records, and taking it would reopen the unsatisfiable-requirement defect, so it needs its own ADR.

## 12. Worked example

One analyzed build, one MAP, one BIN, one HEX, `required = ["elf","map","bin","hex"]`.

Observation produces, in §5 order:

```json
"attachments": [
  { "kind": "bin", "sha256": "11b6945e7e9b3b8e5f1c9a1e0e5c0b3b9c2f0a1d3e5f7b90c1e3f5a7b9d1e3f5", "byte_size": 245760 },
  { "kind": "hex", "sha256": "7d3f9a1c5e0b2f4d6a8c0e2f4b6d8f0a1c3e5f7b9d1e3f5a7c9b1d3f5e7a9c1b", "byte_size": 683214 }
]
```

Canonical text fragment — note the block is present and the label moved to `/2`:

```text
firmwaresight-gate-input/2
snapshot=snap-4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374-p0-normalize-1-a38575cd…
artifacts[
  elf known:4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374
  map known:e2a1…
]
attachments[
  bin known:11b6945e7e9b3b8e5f1c9a1e0e5c0b3b9c2f0a1d3e5f7b90c1e3f5a7b9d1e3f5
  hex known:7d3f9a1c5e0b2f4d6a8c0e2f4b6d8f0a1c3e5f7b9d1e3f5a7c9b1d3f5e7a9c1b
]
```

`artifacts.required` becomes:

```json
{
  "rule_id": "artifacts.required",
  "state": "PASS",
  "effective_severity": "PASS",
  "summary": "Every required artifact kind is present: elf and map from the analyzed build; bin and hex from release attachments (bytes verified, provenance not verified).",
  "evidence_refs": [
    "policy:artifacts.required=[elf, map, bin, hex]",
    "artifact:elf:4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374",
    "artifact:map:e2a1…",
    "attachment:bin:11b6945e7e9b3b8e5f1c9a1e0e5c0b3b9c2f0a1d3e5f7b90c1e3f5a7b9d1e3f5",
    "attachment:hex:7d3f9a1c5e0b2f4d6a8c0e2f4b6d8f0a1c3e5f7b9d1e3f5a7c9b1d3f5e7a9c1b"
  ],
  "remediation": null
}
```

Bundle side, `release-manifest.json` — `files[]` already covered the bytes; `extensions` adds the kind and its basis:

```json
"files": [
  { "path": "artifacts/app.bin", "sha256": "11b6…e3f5", "size": 245760 },
  { "path": "artifacts/app.hex", "sha256": "7d3f…9c1b", "size": 683214 }
],
"extensions": {
  "attachments": [
    { "file": "artifacts/app.bin", "kind": "bin", "kind_basis": "declared", "sha256": "11b6…e3f5", "size": 245760 },
    { "file": "artifacts/app.hex", "kind": "hex", "kind_basis": "declared", "sha256": "7d3f…9c1b", "size": 683214 }
  ]
}
```

Replacing `app.bin`'s bytes changes the run id (L-2), so a plan previewed against the old run is refused with
`GateContextChanged` (L-3) before any byte is written, and the release id changes too, because `gate_run=` and the
`artifact=` lines are already in its canonical text. Deleting `app.hex` after judging refuses at re-observation (E-1).
Renaming `app.bin` to `app_f.bin` leaves the Gate verdict standing and re-plans the package (§4.2 row 5).

## 13. Implementation Readiness Matrix — `C1-U1` … `C1-U4`

Future task decomposition only. **This round authorizes none of them.** Each needs its own prompt naming its unit, and
`ADR-0030`'s exclusions bind all four.

| unit | scope | inputs it needs | depends on | acceptance | STOP conditions |
| --- | --- | --- | --- | --- | --- |
| `C1-U1` Data + Identity | `ReleaseAttachment` and `GateAttachmentFact` types; `observe_attachment`; `GateContext.attachments`; the `/1`-vs-`/2` canonical rule; migration `0006`; storage write and re-read | §2, §3, §5, §7.7 | `ADR-0030` only | `T-C1-01`, `T-C1-02`, `T-C1-03`, `T-C1-05`, `T-C1-16`; every pre-C1 run id unchanged; `SnapshotId` untouched | any need to change `SnapshotId::compose`; any need to widen the `/1` grammar |
| `C1-U2` Gate + Bundle | `artifacts.required` and `artifacts.hashes` over two sources; `attachment:` locators; `verify_attachments`; `InputChecks` membership; E-1…E-4; `changed_leaf` set-aware fix; naming and collision | §4, §5, §6 | `C1-U1` | the six links each covered by a test; `T-C1-07`, `T-C1-08`, `T-C1-09`, `T-C1-10`, `T-C1-14`, `T-C1-15`; preview and export run one rule set | any check that exists for snapshot rows and not for attachments; any rule stated once for preview and once for export |
| `C1-U3` Contract + Regression | the two `extensions` entries; schema contract tests; goldens; the pre-C1 corpus readability proof | §7, §12 | `C1-U1`, `C1-U2` | all of §7's tests; no schema major moves; no `release-manifest:1` field added; the P4 golden bundle verifies with an unmodified verifier | a discovered need for a schema major; a pre-C1 document that stops validating |
| `C1-U4` UI + CLI | Release "Release attachments" section, Attach/Ship verbs, the §9 sentences, `--attach`; report limits line; design checklist | §9 | `C1-U1`, `C1-U2`, `C1-U3` | copy matches §9 verbatim; no React-side hashing or verdict logic; `AGENTS.md` §11 checklist passes; installed visual evidence, not a screenshot of a dev server | any wording that implies analysis, structure or provenance; any new navigation or token |

Ordering note for whoever plans the rounds: `C1-U1` alone changes no user-visible behaviour and can be verified entirely
by tests; `C1-U3` is the unit that proves the compatibility claims this document makes, and skipping it is how an
approved design turns into an unnoticed contract break.

Documents each unit owns, so that "docs match behavior" (`AGENTS.md` §10) stays true as the design lands:

| unit | documents it must update when it lands |
| --- | --- |
| `C1-U1` | `04_TECH/15_STORAGE_DATABASE_BASELINE.md` (the `0006` table and its invariants), `04_TECH/02_DOMAIN_MODEL.md` (`ReleaseAttachment`) |
| `C1-U2` | `04_TECH/17_RELEASE_PACKAGING_UPDATE.md`, `04_TECH/24_BUILD_IDENTITY_EVIDENCE.md` (attachment provenance is permanently Unknown), this document's status line |
| `C1-U3` | `04_TECH/26_PORTABLE_SCHEMA_POLICY.md` (the two `extensions` entries, in the same style as its P3 and P4 landing sections) |
| `C1-U4` | `04_TECH/07_CLI_SPEC.md` (`--attach`), `04_TECH/14_IPC_DATA_CONTRACTS.md`, `P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md` (only with runtime evidence), `01_PRODUCT/01_PRD_MVP.md` and `04_TECH/03_FORMAT_SUPPORT.md` status sentences, plus `templates/DESIGN_CHECKLIST_TEMPLATE.md`'s answers in the round's own evidence root |
