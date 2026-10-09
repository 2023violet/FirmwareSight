---
title: "ADR-0030 Release Attached Byte Evidence"
doc_id: "ADR-0030"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-09"
---

# ADR-0030 — BIN and Intel HEX Ship as Release-Attached Byte Evidence, Not as Analyzed Builds

## Status

**Accepted as product direction and as a design baseline. Not accepted as an implementation authorization.**

The Owner delivered 《FirmwareSight — Option C1 / Release-Attached Byte Evidence｜产品方向裁决与 ADR +
技术规格定稿｜Coding Agent 执行授权 v1.0》 on 2026-10-09, archived verbatim at
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_Product_Direction_Adr_Specification_Freeze_v1.0.txt`
(15,545 bytes, 185 lines, 0 `CR`, SHA-256 `3dbd878181655271d95df4f283102853c5996081b0a06df911e6c46ff7e8f9c5`,
`cmp` clean against the delivered bytes). Its §0 states the authorization semantics, and its §6 step 0 required this
record to display that provenance before writing anything:

> OWNER_PRODUCT_DIRECTION = OPTION_C1
> OWNER_AUTHORIZED_UNIT = C1_ADR_AND_DESIGN_FREEZE_DOCS_ONLY

> 这是一项产品方向与文档设计的授权，不是实现授权，也不是批准升级或替换任何 V1 cohort 构建。

Therefore, as of this ADR:

| claim | status |
| --- | --- |
| C1 is the chosen product direction | **OWNER APPROVED** — this ADR |
| The design and its contracts are frozen enough to build against | **APPROVED** — `04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md` |
| BIN / Intel HEX release attachment works in the product | **NOT IMPLEMENTED** |
| BIN / Intel HEX structural analysis | **UNSUPPORTED, and out of scope for C1** |
| Any of `C1-U1`…`C1-U4` (the four implementation units) | **NOT AUTHORIZED** — each needs its own prompt |
| V1's cohort build, sessions, metrics, thresholds | **UNCHANGED** — artifact `11573661113` at product head `41bb6a36` |

This ADR is a **qualified supersession of ADR-0006**, and of nothing else. It does not rewrite ADR-0006, does not
re-date it, and does not delete its wording; see *Relationship to ADR-0006* for exactly which clause it replaces and
which clause it leaves standing.

## Context

### The conflict this decides

Five tracked documents describe BIN and Intel HEX support, and they do not agree. Each row below was read at head
`d41284e54f723d74687cf351d85f1f2fc0b0be2a` on 2026-10-09.

| source | what it says | class |
| --- | --- | --- |
| `09_ADR/ADR-0006-mvp-format-scope.md:18` | MVP 正式支持 includes "BIN/Intel HEX 基础 metadata" | Declared promise |
| `01_PRODUCT/01_PRD_MVP.md:19-23` (P0-1) | Import input list includes "BIN / HEX（仅基础 metadata/hash）"; MUST 记录 SHA-256 | Declared promise |
| `01_PRODUCT/07_MVP_COHORT_AND_CLI_POLICY.md:19` | MVP Supported cohort: "BIN/Intel HEX 仅基础 metadata/hash" | Declared promise |
| `04_TECH/03_FORMAT_SUPPORT.md:30-31` | `.bin` → `Basic / hash/size/package only`; Intel HEX → `Basic / hash/address span/package only` | Declared promise, with a capability the product never had |
| `P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md:68` | "HEX / BIN / UF2 / Mach-O / PE — `UNSUPPORTED` — Not in the cohort, not claimed." | Observed result |

`A0_PRODUCT_TRUTHFULNESS_CORRECTIVE` (2026-10-09) corrected the *user-visible refusal copy* and explicitly left this
range conflict to the Owner, and its report says so. This ADR is that Owner decision arriving.

### The concrete defect, not the documentary one

The defect is bigger than inconsistent prose, and it is the reason a documentation-only fix (option B) was rejected.

- `crates/firmwaresight-project/src/config.rs:23-25` publishes `REQUIRED_ARTIFACT_VOCABULARY = ["elf","map","bin","hex"]`,
  and `schemas/project-config.schema.json` repeats that enum. A project may therefore write
  `[artifacts] required = ["elf","hex"]`.
- `crates/firmwaresight-core/src/domain/gate.rs:959-1004` evaluates `artifacts.required` against `GateContext.artifacts`,
  and `gate.rs:405` documents that vector as "One artifact **the snapshot** holds".
- The only producer of snapshot artifact rows is the analysis pipeline, and `crates/firmwaresight-artifact/src/pipeline.rs:83-97`
  returns `UnsupportedFormat` for every non-ELF input before any snapshot row exists. `crates/firmwaresight-core/src/domain/build_snapshot.rs:123-136`
  seals a snapshot from a primary ELF plus an optional MAP, and nothing else.
- ⇒ **`required = ["hex"]` is lexically legal and can never be satisfied.** Following the PRD earns a permanent
  `BLOCK`, and `apps/desktop/ui/src/Release.tsx:839-841` describes that rule as "checked for presence only".
  A rule that can never pass is not a strict rule; it is a dead end presented as a policy.

### What already exists, and therefore what C1 does not need to invent

Four mechanisms were verified as already doing, for the kinds they cover, exactly what C1 needs for the new class:

1. **Identity.** `gate.rs:676-729` renders the Gate canonical input, and `gate.rs:683-691` already emits one
   `"<kind> <digest>"` line per artifact row. `crates/firmwaresight-project/src/fingerprint.rs:34-39` makes the run id
   the digest of that text. Bytes that enter the list enter the identity with no new machinery.
2. **Staleness.** `bundle.rs:506-543` re-checks run id, policy fingerprint, acceptance set, and every source
   digest-and-size pair between preview and export, and `bundle.rs:596-604` refuses the whole plan with
   `GateContextChanged` when a selected stored run no longer recomputes.
3. **Byte checks at packaging.** `bundle.rs:934-1016` stats, refuses non-regular files, streams a SHA-256
   (`fingerprint.rs:42-56`), compares it to the recorded digest, compares the size, and finally cross-checks that the
   row is present in the judged Gate context.
4. **Release identity.** `crates/firmwaresight-core/src/domain/release.rs:334-399` already carries `gate_run=` at
   `:343` and one `artifact=<kind>:<sha256>:<size>:<file_name>` line per shipped file at `:354-363`, under
   `firmwaresight-release-input/1`.

What is missing is the class itself — an entity that is *not* a build, plus the one honest limit that class carries.

### Why a HEX parser is not part of this

`crates/firmwaresight-artifact/src/intake.rs:179-202` inspects leading bytes and classifies `:`-prefixed input as
`IntelHex`, ≥8 all-ASCII-graphic bytes as `IhexLike`, and anything else as `Binary`. Nothing else exists: a search of
the workspace for record-checksum, extended/segmented addressing, address-span or data-record handling returns no
implementation. `04_TECH/03_FORMAT_SUPPORT.md:31` nevertheless promises Intel HEX an **address span**. C1 delivers
bytes, and the address-span promise is withdrawn rather than quietly kept.

## Decision

**C1 — Release-Attached Byte Evidence.** Adopt the following as the product direction and as binding design.

**D-1 — One analysis boundary, unchanged.** Analyze takes a required ELF linker output plus an optional GNU ld MAP,
and nothing else. `pipeline.rs:83-97`, `build_snapshot.rs:123-136` and `SnapshotId::compose`
(`build_snapshot.rs:34-41`) do not change in meaning or in bytes. No BIN-only snapshot, no HEX-only snapshot, and no
BIN/HEX row in `analysis.json`, `diff.json` or Compare — an empty section table for a file that has no sections is a
fabricated fact, not a degraded one.

**D-2 — A new entity, `ReleaseAttachment`.** A release attachment is a file the release owner chooses to ship, whose
only claims are raw bytes, byte size and SHA-256. Its authoritative holder is `firmwaresight-project`, the crate
ADR-0027 created for project-scoped observation; `firmwaresight-core` owns the fact *type* and the canonicalization
and never opens a file. The UI owns a user's selection and never a digest. There is exactly one production path per
stage, named in `04_TECH/28` §3, and no second path is permitted.

**D-3 — Requirement kinds map to one source each.** An `elf` or `map` requirement is satisfied **only** by a snapshot
row (an analyzed build). A `bin` or `hex` requirement is satisfied **only** by an attachment row. An attachment whose
kind is `unknown` satisfies no requirement. Attaching a second ELF does not satisfy `elf`: the ELF a release ships is
the ELF that was analyzed, and this rule is what keeps "present" from meaning "somewhere on this machine".

**D-4 — Attachment bytes enter the Gate identity, under a label that discriminates.** `GateContext` gains an
`attachments` vector, canonicalized as its own block:

```text
firmwaresight-gate-input/2
artifacts[
  elf known:<sha256>
]
attachments[
  bin known:<sha256>
  hex known:<sha256>
]
```

The block is emitted only when the set is non-empty, and a context with attachments is canonicalized under
`firmwaresight-gate-input/2`. A context with **no** attachments is canonicalized exactly as it is today, under `/1`,
and the writer must never emit an `attachments[` block under `/1`. That single rule is what makes old ids stable,
new ids distinguishable, and the two grammars impossible to confuse.

**D-5 — Canonical order, multiplicity, and what is deliberately not identity.** Attachment rows sort by
`(kind word, sha256 lower hex)`; an exact `(kind, sha256)` duplicate is bound once; several distinct files of one kind
are all bound, and `artifacts.required` is satisfied by any of them. A file name and a source path are **not** part of
Gate identity — a rename is not a change of bytes, and a host path has no place in a portable identity
(`gate.rs:673-674` already states that absolute paths are absent by construction). File name *is* part of release
identity, because `release.rs:354-363` already carries it. The consequence is deliberate and stated in `04_TECH/28` §5:
**renaming an attachment invalidates the bundle plan and the release id, but not the Gate verdict.**

**D-6 — Public contracts stay at major 1.** `gate-results:1`, `release-manifest:1`, `analysis:1`, `diff:1`,
`accepted-reviews:1` and `project-config:1` keep their version and every existing field's meaning. New disclosure goes
in the declared extension point — `extensions`, `additionalProperties: true` — which is exactly how P3 carried
`policy_sha256` and `overall_effective_severity` into `gate-results:1` and how P4 carried `gate_run_id` and
`integrity_model` into `release-manifest:1` (`04_TECH/26_PORTABLE_SCHEMA_POLICY.md:43-54`, `:77-80`). For
`gate-results:1`, a run that bound attachments **must** carry `extensions.canonical_input_label` and
`extensions.attachments`; a run that bound none must carry neither. Absence is a true statement, not a missing one, and
the pair is what makes the two grammars mutually identifiable from the document alone.

**D-7 — Provenance of an attachment is `Unknown`, permanently.** The product can prove an attachment's bytes, size and
digest, and can prove it was shipped. It cannot prove the file was produced by the analyzed build or the recorded Git
commit, and it must never imply that it can: no rule summary, no bundle page and no portable field may assert
"built from this commit" for an attachment. This Unknown is **not** counted in `unknown.count`
(`gate.rs:415-420` defines that count as *snapshot* evidence items), because letting it in would move
`evidence.unknown_review` — and therefore the verdict — whenever someone attaches a file. It is disclosed in the
finding summary and in an `attachment:<kind>:<sha256>` locator instead, leaving the `artifact:` locator of prompt §30
untouched.

**D-8 — Intel HEX is never claimed.** Record checksums, record validity, address span, loaded content, sections,
symbols, memory regions, entry point and object attribution are `Unknown / Not verified` for an attachment, never `0`
and never omitted. A `.hex` file whose bytes are not Intel HEX may still ship as an unverified raw-byte attachment,
with its declared kind kept as `Declared` and any disagreement with the leading bytes reported as a **conflict**, not
silently reclassified — the same rule `04_TECH/24_BUILD_IDENTITY_EVIDENCE.md:34` already applies to version sources.
No HEX parser, no record walker, no new format-recognition engine.

**D-9 — Attachments are held to the same consistency bar as snapshot rows.** The export re-check, the preview→publish
staleness list, the context membership cross-check, the size and digest comparisons, the empty-file refusal, the
regular-file requirement, the collision-free bundle naming and the `SHA256SUMS`/manifest byte verification all cover
attachment rows on the same terms. "Gate judged A while the bundle ships B" must stay unrepresentable, and
`04_TECH/28` §6 states the six-link proof chain that keeps it so.

**D-10 — Vocabulary stays, capability wording changes.** `bin` and `hex` stay in `[artifacts] required` and in the UI
kind list, because after C1 they name something real. Until C1 is implemented, every surface that lists them must say
what is true today. `04_TECH/28` §9 carries the exact replacement sentences. The compatibility matrix keeps its
`UNSUPPORTED` status, because that column is closed to the five values the P5 closure normalization fixed and
`NOT_IMPLEMENTED` is not one of them; `04_TECH/28` §9 names `SUPPORTED_WITH_LIMITS` as the word a later round would
use, and only with an attached-file fixture plus a measured run. `NOT_IMPLEMENTED` is a governance state word for
`.ai/` and `BASELINE.yaml`, so the Owner's instruction "兼容矩阵仍应标 UNSUPPORTED / NOT_IMPLEMENTED" is honoured as:
matrix = `UNSUPPORTED`, governance state = `DESIGN_APPROVED / NOT_IMPLEMENTED`.

### Qualified supersession of ADR-0006

ADR-0006's Decision list stays in the repository exactly as written. This ADR replaces one clause of it and keeps the
rest:

| ADR-0006 clause | after this ADR |
| --- | --- |
| "ELF 32/64 基础分析" | stands, unchanged |
| "GNU ld MAP" | stands, unchanged |
| "BIN/Intel HEX **基础 metadata**" | **qualified supersession.** There is no BIN/HEX metadata: no architecture, no entry point, no sections, no address span. What C1 promises instead is *release-attached byte evidence* — raw bytes, size, SHA-256, declared kind — and it arrives at the Release verb, not the Analyze verb |
| "Git provenance" | stands, unchanged |
| "Keil/ArmClang/IAR 只有在 fixture + adapter tests 完成后升级为 Supported" | stands, unchanged, and C1 does not touch it |

The supersession is qualified rather than total because ADR-0006 was right about the boundary it drew (narrow, fixture
first, no blanket promises) and wrong only about what "基础 metadata" could mean for a container FirmwareSight does not
open. `01_PRODUCT/01_PRD_MVP.md:23` and `04_TECH/03_FORMAT_SUPPORT.md:31` are re-scoped the same way by dated
clarifications, not by rewriting their rounds.

## Contract and identity semantics

The full matrix, with per-contract contract tests, is `04_TECH/28` §7. The ruling, in one place:

- **`gate-results:1`** — major stays 1. Field set, five-state vocabulary and `effective_severity` separation are
  untouched (ADR-0023). `run_id` remains an opaque `gate-<64 hex>` (`04_TECH/15_STORAGE_DATABASE_BASELINE.md:131`);
  what it binds is disclosed by two mandatory-on-presence extension entries. A `gate-results:2` was rejected: it would
  make every v1 document already written, in storage and in `golden/reports/p4-release/`, require reinterpretation —
  the silent rewrite §D3 forbids.
- **`release-manifest:1`** — major stays 1. Attachments are bundle files, so `files[]` already lists each one with
  `path`, `sha256` and `size` and no field is added. `extensions.attachments` adds the kind and the kind's basis, which
  `files[]` cannot express without a schema change.
- **`release id`** — no grammar change and no label change. `gate_run=` and the per-artifact lines already chain the
  new identity in, which is why an attachment change cannot leave a stale PASS attached to a new package.
- **`SnapshotId`** — no change. A D-1 boundary change here would be the silent identity drift §5 of this round's
  authorization lists as a hard stop.
- **`analysis:1`, `diff:1`, `accepted-reviews:1`, `project-config:1`** — no field, no semantics, no version. C1 makes
  `project-config:1`'s existing `bin`/`hex` vocabulary *satisfiable*; it does not redefine it. Narrowing that vocabulary
  would have been the semantics change requiring a major, which is one reason option B was rejected.
- **Storage** — one additive table in a future migration `0006`, because a Gate run must be re-judged and exported
  from stored rows (`apps/desktop/src-tauri/src/release.rs:846` `SnapshotFacts::from_stored`, whose contract test is
  `the_run_is_judged_from_stored_rows_after_the_files_are_gone`). `gate_runs.id`'s length-69 CHECK still holds, and
   `crates/firmwaresight-storage/migrations/0001_initial.sql:31-47`'s `artifacts` table is not reused for
   attachments: it declares `parser_id`, `architecture`, `bitness` and `endianness` `NOT NULL`, so a BIN row
   there would force invented analysis columns — the structural form of the same defect A0-01 closed in the
   capability report.

## Alternatives

**Option A — analyze BIN/HEX as first-class builds.** Rejected. It needs a container reader for formats that have no
sections or symbols, then forces empty structural tables into `analysis.json`, Compare and the memory rules, and it
would move `SnapshotId` while `build_snapshot.rs:26-27` promises the id changes "when any input byte changes" for the
inputs it actually covers. The earlier read-only round ranked the four options' effort as an estimate, never as a
measurement: A is the largest, because it needs identity work plus a container reader for formats that have no
structure to read.

**Option B — documentation only.** Rejected on the defect, not on the prose. Two variants were examined: *B(S)*, which
narrows the promise text, leaves `required = ["hex"]` permanently `BLOCK`ing, so the honesty hole survives its own
fix; *B(M)*, which also narrows `[artifacts] required`, and that is a `project-config:1` semantics change requiring a
major and breaking every project config written before it. B does not answer the Owner's question, which was whether
the bytes actually shipped deserve evidence.

**Option C2 — BIN/HEX rows inside the snapshot.** Rejected as a hard stop, not a preference.
`build_snapshot.rs:123-136` seals from primary plus optional MAP; adding rows would make the snapshot id stop covering
its own inputs while its documentation claims otherwise, and `verify_sources` (`bundle.rs:942-952`) would have to stop
refusing the very rows it refuses today. It would also put pseudo-builds into Compare.

**Option C3 — ship attachments but keep them out of the Gate.** Rejected because the Owner's own choice of C1 over C3
settles it: under C3, `required = ["bin","hex"]` remains unsatisfiable, and the shipped bytes gain no verdict. It is
the cheapest option and the one that leaves the defect open, and `04_TECH/28` §11 records it as the rollback target if
the Gate-attachment binding ever has to be withdrawn.

**"Extensions only, keep the `/1` label."** Rejected. It would put two canonicalization grammars under one label, so a
reader could not tell an attachment-bound run from an attachment-free one without re-running the product — the
indistinguishable-semantics case §D3 rules out. The extension is required *alongside* the label, not instead of it.

## Consequences

Positive:

- `required = ["bin"]` and `required = ["hex"]` acquire a real, testable satisfying path, and the caption at
  `Release.tsx:839-841` can say something true instead of describing a dead end.
- The bytes a release owner actually ships become evidence: Observed size and SHA-256, bound into the Gate run id,
  bound into the release id, verified at packaging and re-verified inside the bundle.
- Stale-PASS protection is inherited rather than built: `GateContextChanged`, `PlanStale`, `SourceArtifactChanged` and
  `require_packageable` already exist and already cover the shipped rows.
- No schema major, no `SnapshotId` change, no migration to existing tables, no new dependency, no network, no parser,
  no design token.
- Old evidence keeps its meaning: every pre-C1 `gate-results:1` document, every stored `gate_runs` row and every
  already-published bundle validates and reads exactly as it does today.

Negative, and accepted:

- Two canonical-input grammars now exist. The discrimination rule (D-4 plus D-6) is what pays for that, and it must be
  asserted by contract tests, not by care.
- `artifacts.required` and `artifacts.hashes` summary wording moves, and finding locators gain a second prefix. Run ids
  are unaffected (summaries are not canonicalized), but goldens and stored-vs-recomputed prose will differ across
  versions. The ADR states plainly that stored summaries are presentation and the id is identity.
- A required kind can be satisfied by a file that has nothing to do with the build, and no rule can detect that. The
  mitigation is disclosure at every surface, not a claim.
- Renaming an attachment re-plans a bundle that was already-previewed; that is correct and will read as friction.
- Hashing a very large attachment is streaming and therefore safe on memory, but not free in time, and C1 adds no size
  cap. If that becomes a real problem, it is a new decision, not a detail.
- The `04_TECH/03_FORMAT_SUPPORT.md:31` "address span" promise is withdrawn. Any reader of the older document must be
  told so by the dated clarification, not by editing the old line out.

## Exclusions — what C1 never claims

No BIN/HEX sections, symbols, architecture, bitness, endianness, entry point, build-id, address span, record checksum,
loaded content, memory footprint, growth delta or object attribution. No BIN/HEX row in `analysis.json`, `diff.json`,
the Compare tables or the memory budgets. No claim that an attachment was produced by the analyzed build or the
recorded commit. No new file-format recognition engine, no new parser, no HEX validator. No change to the four-verb
navigation, to design tokens, to the IPC command count, or to any frozen design asset.

## Rollback

C1 is additive in every layer, so withdrawal is the removal of an option rather than a repair:

1. Before implementation, rollback is this ADR's status. No product byte has changed, so nothing in the product needs
   unwinding.
2. After implementation, rollback is policy-level: stop attaching and select a `/1` run. The `/1` path is the
   pre-C1 path unchanged, which is precisely why D-4 froze it, so an ordinary release continues with no migration and
   no data repair. `gate_run_id`, `release id` and `manifest_sha256` recompute on their own.
3. The `0006` table then holds rows for runs that no new release references. Removing it would be a destructive schema
   migration and needs a human decision under `AGENTS.md` §9; it is not part of rollback.
4. A partial rollback to C3 (ship without gating) is representable without touching any contract, because the
   attachment rows live in their own canonical block. It would reopen the D-3 defect and needs its own ADR.

## Risks

| risk | exposure | control |
| --- | --- | --- |
| Silent SnapshotId drift | a stored build id stops covering its own inputs | D-1 forbids snapshot rows; the `/1` invariance test asserts existing ids reproduce byte-for-byte |
| Stale PASS kept while bytes change | release ships bytes no verdict covered | `run_id` binds attachment digests (D-4) and `GateContextChanged` refuses the plan (`bundle.rs:596-604`) |
| Two meanings for `gate-results:1` | old evidence unreadable, or new evidence misread | major stays 1 with a documented field-meaning test; the label and the paired extension entries discriminate |
| A wrong file satisfying a requirement | a release "passes" on unrelated bytes | D-3 kind-source mapping, the empty-file refusal, Declared-kind wording, D-7's permanent Unknown provenance |
| Host path leakage | privacy boundary broken | names only in identity-bearing paths already excluded (`gate.rs:673-674`); leaf names in findings; `InputChecks` keeps source paths private (`bundle.rs:503-505`) |
| V1's instrument being swapped "to show off" C1 | a research build changed for a roadmap reason | this round authorizes no build, and V1's cohort stays `11573661113` at `41bb6a36`; see `04_TECH/28` §10 |

## Acceptance criteria for this decision record

Each is refutable by a future test, and each is what a later round is measured against:

1. `AC-1` — No implementation unit is started on this ADR's authority; `ACTIVE_TASK` is `NONE` at its close.
2. `AC-2` — No BIN or HEX row is representable in a `BuildSnapshot`, and `SnapshotId::compose` is untouched.
3. `AC-3` — A context with zero attachments produces the identical canonical text, and therefore the identical run id,
   that this repository produces today.
4. `AC-4` — Changing only an attached BIN's bytes changes the run id and makes an exported bundle from the older plan
   impossible, not merely discouraged.
5. `AC-5` — Every `gate-results:1` document written before C1 still validates and is read with its original meaning.
6. `AC-6` — No portable document, UI surface or bundle page asserts that an attachment came from the analyzed build.
7. `AC-7` — No surface claims BIN/HEX sections, symbols, memory, entry point or address span, and the compatibility
   matrix keeps its `UNSUPPORTED` status column until an implementation round produces runtime evidence.
8. `AC-8` — `apps/`, `crates/`, `schemas/`, `migrations/`, `fixtures/`, `golden/`, `assets/`, `templates/`, `scripts/`,
   `.github/` and every named lock/config file are byte-identical before and after this round.
9. `AC-9` — Four independently authorizable units exist, and none of them is executed here.

## Revisit trigger

Reopen this ADR if any of the following becomes true:

- a real requirement arrives to *understand* BIN or Intel HEX content (address span, record validity, per-object
  attribution). That is option A, and it is a new ADR plus new fixtures, not an extension of this one;
- the Gate canonical input needs a third grammar, or an attachment's name turns out to have to be identity-bearing —
  either one changes D-4/D-5 and is a decision, not an edit;
- an attachment must be provably produced by a build, which needs a toolchain or build-system side evidence
  (an adapter, a manifest from the linker, or a signature) that this repository does not have;
- the empty-file, size-cap or same-kind multiplicity rules prove wrong in practice;
- `project-config:1` vocabulary changes are proposed again, or a schema major is proposed for any contract listed here
  as unchanged — the "unchanged" claim is part of this decision and needs a successor to alter it;
- a V1 or cohort decision touches release attachment UX, because D-6's wording then becomes participant-facing and the
  frozen instrument must be re-frozen under its own authority, not this one.
