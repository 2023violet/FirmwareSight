---
title: "Release Attached Byte Evidence"
doc_id: "FS-TECH-029"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-10"
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

*(Dated 2026-10-10, added by the closeout that reconciled this file with the tree — the four lines above are the freeze
day's own words and are kept unrewritten, but they are no longer the current state and a reader who stops here will
act on them wrongly.* §14 is the addendum: `C1-U1` is `COMPLETE` and `C1-U2` is `READY_FOR_ARCHITECT_REVIEW`, so at
head `2859bb2` attachment facts exist in storage, carry the Gate run identity, can satisfy a Gate requirement and can
enter a verified Release Bundle. The live state word is therefore
**`DESIGN_APPROVED / NOT_USER_AVAILABLE`**, not `NOT_IMPLEMENTED`, and the two things this paragraph still gets right
are narrower than they look: no person can reach any of it, and nothing was built, installed or launched, so
`NOT_RUNTIME_VERIFIED` stands in exactly the sense written here. *One more consequence of the same two units: every
path and line number above was read at head `d41284e`, and `C1-U1`/`C1-U2` rewrote large regions of
`firmwaresight-core/src/domain/gate.rs`, `firmwaresight-project/src/bundle.rs`, `.../src/evidence.rs` and
`firmwaresight-report/src/release.rs`, so a citation into those four files must be re-verified at the current head
before anyone relies on its line number.)*

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

## 14. Status addendum — `C1-U1` and `C1-U2` have landed (2026-10-09)

Status only. §§1–13 above are the frozen specification as the Owner approved it on 2026-10-09 and are not edited here;
this section says which of it is code, which is not, and which statements in §1 are now the freeze round's answer rather
than the current one.

**The live capability word.** §1's
`BIN_HEX_RELEASE_ATTACH = DESIGN_APPROVED / NOT_IMPLEMENTED`
and its "Nothing here is built" sentence describe the head `b5f732d` this document was written at. After two authorized
implementation units the word is
**`BIN_HEX_RELEASE_ATTACH = DESIGN_APPROVED / NOT_USER_AVAILABLE`**,
and `BIN_HEX_ANALYSIS` stays `UNSUPPORTED`: two units of library code and one migration exist, no surface reaches a
person, and the compatibility matrix column is untouched.

| unit | state | what is now true in code | what it still does not give |
| --- | --- | --- | --- |
| `C1-U1` | `COMPLETE` (2026-10-09, head `8e2aab9`) | `GateAttachmentFact` / `KindBasis`, `GateContext.attachments`, the `/1`-while-empty / `/2`-when-not canonical rule, `ReleaseAttachment` + `observe_attachment()` with E-1/E-2/E-3 at `ERR-BUNDLE-6115`/`6116`/`6117`, migration `0006` and its stored-run re-read | nothing satisfies a rule yet; no bundle path reads the set |
| `C1-U2` | `READY_FOR_ARCHITECT_REVIEW` (2026-10-09, head `facf287`, remote CI 10/10 on attempt 1 — run `38037417546`, whose two Rust jobs print 953 on Windows and 952 on Ubuntu for one `#[cfg(windows)]` case) | §5's two-evidence-class `artifacts.required` / `artifacts.hashes` with `attachment:` locators; §6's six links L-1…L-6 each carried by its own positive and negative test; `verify_attachments` as a sibling of `verify_sources` refusing before any destination write; §4.2's staleness over one typed row set with E-4 at `ERR-BUNDLE-6118`; `verify_bundle` re-deriving a release id over a bundle that ships attachments; §7.5's minimum honest `extensions.attachments` disclosure | §9's sentences to a user (no UI, no `--attach`, no IPC); §7.3's `gate-results:1` extension pair; §12's worked example as a document; the pre-C1 corpus re-validation and golden closure — all `C1-U3` / `C1-U4` |
| `C1-U3` | `NOT_AUTHORIZED / NOT_STARTED` | — | — |
| `C1-U4` | `NOT_AUTHORIZED / NOT_STARTED` | — | — |

**One deviation from §3, chosen because §11 of the unit's authorization outranks it.** §3 names the judging path as a
`GateRunRequest.attachments` **field**. That struct is built by literal in `apps/desktop/src-tauri/**` and `apps/cli/**`,
both of which C1-U2's prompt forbids touching without a new Architect decision, and its §11 instructs "do NOT quietly
patch desktop". `C1-U2` therefore added siblings instead of fields —
`build_context_with_attachments`, `prepare_with_attachments`, `BundlePlan::publish_with_attachments`,
`ReleaseManifestDto::from_parts_with_attachments` — each of which is what §3's field would have become with an empty
list, and the pre-existing entry points delegate to them with `&[]`. The consequence the Architect should accept or
overturn: **a reviewer reading §3 will look for a field that does not exist.** The semantics §3 specifies are unchanged;
only the carrier differs. Nothing in `apps/**` was edited and no `apps/**` compile failure was seen, so §11's STOP-with-
compatibility-proposal branch never fired.

**Four findings §§4–§6 did not decide, left open rather than closed by an implementation choice.**

1. §4.3's E-1…E-4 all reach a caller as branchable codes, but a selection that folds to one host file on a
   case-insensitive filesystem is refused by the manifest's `DuplicatePath`, mapped onto
   `ERR-BUNDLE-6109 VERIFY_FAILED`, with a message about a manifest the caller never wrote. True, not useful. Whether it
   deserves its own code is a portable-contract question and therefore `C1-U3`'s.
2. §4.4 states symlink refusal for what is *inside* a bundle. It says nothing about a symlink offered as a *source*
   selection, and a `std::fs` read follows one. `C1-U2` wrote no test here rather than fixing behaviour by inventing a
   rule, so §8's M11 symlink half is `NOT_VERIFIED` on every platform.
3. §4.2's staleness table and §3's recompute-at-export rule can both be satisfied at once, and the order matters: the
   verdict is recomputed *before* the preview comparison, so withdrawing a policy-required attachment answers
   `ERR-BUNDLE-6101` (`ReleaseError::GateNotReady` — "the Gate disposition is BLOCK…") rather than E-4. `C1-U2` documented this in code and asserts it in
   `a_withdrawn_attachment_the_policy_requires_is_refused_by_the_gate_first`; §4.2 as frozen does not name the precedence.
4. §8's M12 asks that a large attachment be read without loading it. A 1 MiB case proves the streaming path; no >512 MiB
   fixture was created, so that size is `NOT_VERIFIED` rather than assumed.

### 14.1 Adjudicated 2026-10-10 by `C1-U2R1` — the Architect's review, and which of §14's four findings closed

The Architect read `facf287` and returned **`REWORK REQUIRED` (narrow corrective)**, not a completion. That word is now
the live state for the round §14 recorded as `READY_FOR_ARCHITECT_REVIEW`; the four things §14 left open are answered
by its adjudications A–D, and `C1-U2R1` executed exactly those and nothing else. §§1–13 stay as frozen, and §14 stays
as the round wrote it; this section is the dated answer, not a rewrite.

| §14 item | adjudication | what landed |
| --- | --- | --- |
| The §3 field-vs-siblings carrier | **A — ACCEPTED** as a backward-compatibility deviation | The carrier stays four siblings; `GateRunRequest` gains no `attachments` field and no `apps/**` caller is patched. A reviewer following §3 must look for `build_context_with_attachments` (`evidence.rs`), `prepare_with_attachments` and **`BundlePlan::publish_with_attachments`** (`bundle.rs` — §14 and `04_TECH/17` both wrote `BundleRequest::`, which never named the type, and both were corrected in place), `ReleaseManifestDto::from_parts_with_attachments` (`report/release.rs`). This closes the question §14 put to the Architect; it does not make the deviation invisible, it makes it decided. |
| Finding 1, the folded-case sentence | **B — CLOSED** | `ERR-BUNDLE-6109` and the invariant refusal stay exactly as they were. What was wrong was that a *selection* collision spoke about a *document*: the assembly path now answers `BundleError::DuplicateBundleName { name }`, a bundle-relative leaf, whose remediation names the two spellings and the case-fold, while `VerificationFailed` and every other index refusal keep the integrity sentence "keep whatever was written, and report the release id and the Gate run id". Refusal still happens before any destination exists, still names no host path, and no caller's branch moves. Asserted by `a_folded_duplicate_selection_is_refused_by_naming_the_selection_and_not_a_manifest` (portable: two files, one content, fold-equal leaves), `the_ntfs_fold_of_one_host_file_names_its_own_cause` (`#[cfg(windows)]`: one host file, two spellings) and `a_selection_duplicate_and_a_written_manifest_duplicate_are_told_apart`. |
| Finding 2, the source-side symlink | **C — CLOSED as a rule, and the rule is now tested** | A link the release owner chose explicitly **may be followed** to its target when that target is a readable, non-empty regular file; a broken, directory, inaccessible or subsequently re-pointed target fails safely; a directory is never traversed; and nothing a bundle writes is ever a symlink. This is `observe_attachment`'s existing behaviour (`std::fs::metadata` then `is_file()`), which is why §14 could have passed it off as verified — the hole was the absence of tests, and `source_symlink_posix` (four cases) plus `a_source_symlink_on_windows_ships_its_target_bytes_when_the_host_permits_links` now hold it. **Verification status is not uniform and is stated in `C1_VALIDATION/C1_U2R1_ARCHITECT_CORRECTIVE_REPORT.md` §8**: the POSIX module neither compiled nor ran on the Windows host that wrote it, and that host refuses link creation with `ERROR_PRIVILEGE_NOT_HELD`, so M11 remains `NOT_VERIFIED` as *executed evidence* until a Linux run exists. |
| Finding 3, the refusal order | **D — CONFIRMED, unchanged** | "Gate-refusal-before-staleness" is kept: §4.2's table and §3's recompute-at-export rule are honoured in that precedence, and `a_withdrawn_attachment_the_policy_requires_is_refused_by_the_gate_first` continues to assert it. §14's wording stands as the specification of the order. |
| Finding 4, M12's upper bound | **not adjudicated, still open** | No >512 MiB fixture exists. `NOT_VERIFIED` remains the honest word. |

One new observation goes to the Architect rather than being decided here. A cross-run acceptance — a caller handing
run A's finding id to run B — is refused by the bundle layer as `ERR-INTERNAL-9004`, an internal invariant, because a
correct caller reads acceptances from storage keyed to the run it selected and could not produce one. The store does
refuse it as a user-facing, typed answer (`AcceptReviewError::FindingNotFound` at `ERR-STORAGE-4007`, now tested by
`an_acceptance_recorded_over_one_attachment_set_does_not_dispose_of_a_different_one`), and the path a person actually
reaches — the Gate re-run over a new set whose review is not yet accepted — answers `ERR-BUNDLE-6101`. So the
isolation holds at both layers that a product path can reach, and the question left open is only whether a library
caller that mis-carries an acceptance list deserves a branchable code rather than an invariant. `T-C1-08` is
`CLOSED` for the attachment-set pairing; that classification question is not part of it.

§9's user-facing sentences, §7.3's `gate-results:1` extension pair and the pre-C1 corpus re-validation are untouched
here: `C1-U3` and `C1-U4` still own them, and `BIN_HEX_RELEASE_ATTACH` is still
**`DESIGN_APPROVED / NOT_USER_AVAILABLE`**.

### 14.2 Executed 2026-10-10 by `C1-U3` — the pair lands, the corpus is read back, and two sentences here are corrected

Status and reconciliation only; §§1–13 stay as the Owner froze them and §14/§14.1 stay as their rounds wrote them.
`C1-U3` is stage 2 of the four-file continuous-delivery packet archived in
`10_AUDIT/SOURCE_PROMPTS/`, and its own record is `C1_VALIDATION/C1_U3_CONTRACT_REGRESSION_REPORT.md` with the
T-C1-01…19 matrix row by row. Its status word is **`LOCAL_CONTRACT_GATE_PASS / ARCHITECT_REVIEW_PENDING`**.

**§7.3's extension pair is code.** `extensions.canonical_input_label` and `extensions.attachments` are written
together and only when a run bound attached bytes, in §5's `(kind, digest)` order with an exact pair bound once;
an attachment-free document still has neither key, which `apps/cli/tests/p4_golden.rs`'s byte-for-byte golden
comparison proves independently of any test this round wrote. No schema file moved: both `gate-results:1` and
`release-manifest:1` already reserve `extensions` as `additionalProperties: true`, and every closed property list —
both roots and `files[]` items — is asserted still-closed by `the_strict_boundaries_that_make_extensions_the_only_slot_are_still_closed`.
No major moved, no mandatory root field was added, and no new `ERR-BUNDLE-61xx` number was allocated: the new
reader refusals travel on `ERR-BUNDLE-6109`, the code every other read-back refusal already uses.

**The "present iff" rule is a reader rule, and this document's §7.3 wording is why.** §7.3 says the pair is
"present **iff** at least one attachment was bound" and calls `additionalProperties: false` "not loosened". Both
are true, and the second is what makes the first unstatable in the schema: `extensions` is open *by design*, so a
schema that required the two keys to arrive together would have to narrow that reserved slot, which is a semantics
change and would need a major. The rule therefore lives in `verify_bundle`, which refuses half a pair, a `/1` or
unknown grammar label, rows out of §5 order or bound twice, a disclosed set that disagrees with the manifest's
`extensions.attachments`, and — the check that closes the gap §6's L-6 left open — a `gate-results.json` whose
`run_id` is not the manifest's `extensions.gate_run_id`. §7.3's reading rules said a mismatch "is a hard error,
never a re-interpretation"; that sentence is now held by tests rather than by intent.

**Correction 1 — the locator name in §7.5 and §12.** §7.5 sketches `attachments: [{file, kind, kind_basis, sha256,
size}]` and §12's worked example writes `"file": "artifacts/app.bin"`. The shipped field is **`path`**, and it also
carries **`provenance: "unknown"`**. The Architect ruled this on 2026-10-10 in the U3 prompt §4: `path` is the
canonical bundle-relative locator because `files[]` and the verifier already speak it, and `provenance` is
mandatory disclosure so no reader can mistake an attached file for an analyzed one. **The `file` spelling in §7.5
and §12 is superseded by this sentence, not deleted from them**, and `04_TECH/26`'s C1 section records the same
ruling in the policy document that owns the vocabulary. Two aliases would drift; silently renaming `files[]` would
break readers; pretending the sketch never said `file` would lose the reason the shipped row has six fields.

**Correction 2 — §7.3's `T-C1-07` cites a test that did not exist.** That row reads "the existing
`the_run_is_judged_from_stored_rows_after_the_files_are_gone` test extended to a no-attachment run". No test by
that name existed in the repository when `C1-U3` read the row — the design referred to a planned test as an
existing one. `C1-U3` wrote it under that exact name, for the case the row names (a stored run with no attachment
rows, its source files deleted, re-judged from the stored columns to the same findings and the same `/1` canonical
text), in `crates/firmwaresight-storage/tests/gate_history.rs`. The row is now true; the record of it having been
false stays here.

**One reading caution for §§3–8.** Those sections cite source line numbers as they stood when the design was frozen
(`bundle.rs:664-670`, `:993-1003`, `:1506-1560`, `gate.rs:1471-1478`, `release.rs:343` and the rest). Every one of
them has since moved, some by more than a hundred lines, because the units this document describes were built on
top of it. Read the symbols and the rules, and locate them with `git grep`; a numbered pointer in a frozen design
is a snapshot, not an address.

**What is still not code.** §9's user-facing sentences — the Release page's "Release attachments" section, the
Attach and Ship verbs, the repeatable `--attach`, the report's limits line, the IPC command — are `C1-U4`'s and
this round touched no `apps/desktop/**` path and added no CLI argument (the one `apps/cli` edit is the new
`from_evaluation` parameter at the existing call site). So `BIN_HEX_RELEASE_ATTACH` remains
**`DESIGN_APPROVED / NOT_USER_AVAILABLE`**, `ADR-0030`'s exclusions still bind, and the Intel HEX address-span
promise remains withdrawn rather than deferred. What `C1-U3` added is the guarantee that a release which *does*
attach bytes documents them in a way the five published contracts accept without a version bump — and the evidence
that they do, measured against P4's frozen bundle rather than asserted.

**That paragraph is now superseded, by name and by date.** It was true when `C1-U3` wrote it, and the same is true of
§14's table rows reading `C1-U3 / C1-U4 = NOT_AUTHORIZED / NOT_STARTED` — that table is the state 2026-10-09 left
behind and is kept as it was written. `C1-U4` landed the surfaces they list; §14.3 is the record.

### 14.3 Executed 2026-10-10 by `C1-U4` — §9's sentences reach a person, and the installed walk corrects this document's §9 reading

Status and reconciliation only; §§1–13 stay as the Owner froze them and §14/§14.1/§14.2 stay as their rounds wrote
them. `C1-U4` is stage 3 of the same packet, and its own record is
`C1_VALIDATION/C1_U4_USER_WORKFLOW_INSTALLED_REPORT.md`. Its status word is
**`LOCAL_USER_WORKFLOW_GATE_PASS / ARCHITECT_REVIEW_PENDING`**, and the product word it proposes but does not grant
is `READY_FOR_ARCHITECT_PRODUCT_ACCEPTANCE`.

**Every §9 row is now on a surface.** The CLI carries a repeatable `--attach <FILE>` on `gate` and
`release prepare` with all six exit codes unchanged; the desktop carries four use-case commands behind
session-local `att-<pid>-<n>` handles so no host path crosses to the WebView; the Release page carries the
"Release attachments" section with leaf, declared kind, byte size, full SHA-256 and the evidence class, with the
verbs Attach and Ship and the word Analyze never appearing as an action there; the Analyze refusal carries its
pointing clause; `release-report.html` carries the limits line; and the storage invariant sentence now names the
attachment set. §9's frozen sentences were read off an **installed** Windows binary, not off a dev server, which is
what §13 requires.

**§9's "exactly as for analyzed rows" was read as *the same columns*, not *the same role*, and the installed walk
is what forced that reading into the open.** §9 says the bundle preview must show an attachment "the same list with
kind, size, full hash and evidence class shown for attachments exactly as for analyzed rows". The first installed
build gave an attachment the identical columns — and the identical **role**, `artifact`, which is the label the
analyzed ELF and MAP carry. A reader of that preview could not tell the two evidence classes apart, which is the
distinction the whole of C1 exists to keep. `C1-U4` therefore read "exactly as" as being about the *columns*, and
added `FileRole::ReleaseAttachment` (`release-attachment`) so the row that carries the same facts also carries the
class those facts belong to. `FileRole` reaches the preview DTO and no portable document, so no schema, golden or
digest moved. **This is a reading the Architect should confirm or overturn, and it is recorded here rather than
buried in a commit message because it is the one place §9's wording underdetermined the behaviour.**

**Two §9 rows were being violated by surfaces that re-decided a Core question, and both were fixed in `073c83c`.**
The first is the one §9's Gate row exists to prevent. On a build packaged from `d95acea`, the Gate page's own
"Required artifacts" table printed `bin Required Missing` underneath an `artifacts.required` **PASS** whose evidence
list cited `attachment:bin:3080a363…`: the table had looked at analyzed snapshot rows only and answered the question
for itself. §9's requirement that the rule "must stop saying 'in the snapshot' when it did not come from one" is
about the same failure one level up — a surface stating an origin it did not observe. The fix keeps the answer in
one place rather than duplicating it: Core gained `RequirementSource::{Analyzed, Attachment}` and
`requirement_evidence(word)`, the `artifacts.required` rule now *reads that predicate* instead of owning the
mapping, `GateArtifactRowDto` carries `satisfied_by` computed from it, and the table renders *Analyzed build* /
*Release attachment* / *Missing*. A Core test pins each required word to the one set that can satisfy it, so the
words and the table cannot drift apart again. Both fixes were then read back off a build packaged from `073c83c`.

**§7.7's invariant sentence gained the fourth term** — "a changed policy, workspace, baseline **or attachment set**
produces a new run id" — and the installed walk demonstrated why the term belongs there. Mutating a tracked
attachment after the preview did not produce a digest-mismatch refusal; it dirtied the project's workspace, which
changed the canonical Gate input, which changed the run id, so `Export bundle` refused at `ERR-BUNDLE-6102` on
identity and left the destination folder empty before and after.

**§8's declaration rule, observed rather than asserted.** The same bytes reached two different locators depending
only on the file's name: `image.bin` → `attachment:bin:3080a363…`, `payload.img` → `attachment:unknown:3080a363…`,
and the second satisfies no required kind. Nothing opened either file to ask what it was. §8's kind-conflict
sentence remains unimplemented and §14's finding list still owns it: `kind_basis` is always `declared`,
`detect_format` is never called on the attach path, so a file named `.hex` holding raw binary ships as a declared
HEX with no disagreement shown. `C1-U4` re-confirmed the behaviour without resolving that question.

**The live capability word is now** `BIN_HEX_RELEASE_ATTACH = DESIGN_APPROVED / USER_VERIFIED_ON_WINDOWS`.
`BIN_HEX_ANALYSIS` stays **`UNSUPPORTED`** and nothing this round parsed a BIN or a HEX. macOS and Linux installed
acceptance stay **`NOT_RUNTIME_VERIFIED`**: no artifact for either platform was installed or run, and
`P5_COMPATIBILITY_MATRIX`'s status column moves only where a runtime run exists. The session boundary the walk
exposed — `ERR-BUNDLE-6114` naming the current build and then, after that build was re-analyzed, naming the
**baseline** — is a `C1-U2` composition rule that §4 never stated in words: a bundle carrying a comparison needs
both sides of it analyzed since the app was launched. It is reported in §14.3's own terms and left to the
Architect to place in §4 or accept as unstated.


