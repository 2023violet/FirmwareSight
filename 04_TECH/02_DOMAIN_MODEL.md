---
title: "Domain Model"
doc_id: "FS-TECH-003"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-10-09"
---

# Domain Model

## Core entities

### Project
- id
- name
- root_path
- config
- created_at

### Artifact
- id
- path
- kind
- sha256
- byte_size
- parser_id
- observed_metadata

### BuildSnapshot
- id
- project_id
- artifacts[]
- identity
- memory
- sections[]
- symbols[]
- evidence[]
- created_by_fwsight_version

### BuildIdentity
- project_version
- git_commit
- git_tag
- git_dirty
- compiler
- linker
- target
- build_id
- declared_fields[]

### EvidenceItem
- id
- classification: Observed | Derived | Declared | Unknown
- source_type
- source_locator
- field
- raw_value
- parser/rule
- confidence: optional
- notes

### Diff
- base_snapshot
- target_snapshot
- totals
- section_changes
- symbol_changes
- object_changes
- identity_changes

### GatePolicy
- rules[]
- thresholds
- required_artifacts
- version_strategy

### GateResult
- rule_id
- state: Pass | Review | Block | Unknown | N/A
- summary
- evidence_refs[]
- remediation

### GateAttachmentFact
- kind: bin | hex | unknown — an analyzed kind is refused before the file is opened
- sha256: `Fact<String>`, Observed, from a streaming hash of the file's own bytes
- byte_size
- kind_basis: Declared | DerivedFromLeadingBytes(sample)

### ReleaseAttachment
- file_name: a leaf name only, never a host path
- kind
- sha256: `Fact<String>`
- byte_size
- kind_basis

### ComponentEvidence
- component_name
- version
- state
- evidence_refs[]
- confidence
- identifiers (purl/cpe when available)

### ReleaseBundle
- release_id
- snapshot_id
- manifest
- selected_artifacts
- reports
- checksums

## Invariants

1. Snapshot is immutable after creation.
2. Diff never mutates snapshots.
3. Gate evaluation is repeatable for same snapshot + policy.
4. Declared data cannot overwrite Observed evidence; both remain visible.
5. Unknown is stored explicitly.
6. Every exported conclusion references evidence or policy.


## v0.4 Gate finding invariant

`GateResult`/`GateFinding` additionally exposes:

- `state: PASS | REVIEW | BLOCK | UNKNOWN | N/A`
- `effective_severity: PASS | REVIEW | BLOCK`

UNKNOWN remains UNKNOWN even when policy maps it to REVIEW/BLOCK severity.
Review acceptance is a separate immutable audit record.

> **Dated 2026-10-09 (`C1-U1`, first code of `ADR-0030`).** `GateContext`
> (`crates/firmwaresight-core/src/domain/gate.rs`) now carries `attachments: Vec<GateAttachmentFact>`, a set
> separate from `artifacts` by design (`ADR-0030` D-4): `artifacts` are the analyzed rows of the snapshot, while
> attachments are raw bytes a release binds and nothing parses.
>
> - Identity: the canonical text is labelled `firmwaresight-gate-input/2`, with an `attachments[…]` block of
>   `"<kind> <digest>"` rows sorted by `(kind word, digest)` and one exact `(kind, digest)` pair bound once, only
>   when the set is non-empty. With no attachment the text is byte-identical to the `/1` already frozen in
>   `04_TECH/28` §5, so every run id, golden and stored row predating this unit keeps its meaning. A digest is
>   bound as identity; a `byte_size` difference is release identity, not Gate identity.
> - Evidence class: `sha256` and `byte_size` are Observed by `observe_attachment()`
>   (`crates/firmwaresight-project/src/evidence.rs`) from the file's own bytes, streamed, never buffered whole.
>   `kind` is Declared by whoever offered the file, and `kind_basis` records exactly that. `DerivedFromLeadingBytes`
>   is a type-level possibility with no producer: nothing in this build samples bytes to guess a kind.
> - Refusals are typed and happen before any read for a kind that cannot attach: ELF and MAP are analyzed kinds
>   (`ERR-BUNDLE-6116`), an empty file never becomes a digest (`ERR-BUNDLE-6117`), and an unreadable or missing
>   path is reported without naming the host path (`ERR-BUNDLE-6115`).
> - Provenance to the analyzed build stays Unknown permanently and is never counted into `unknown.count`
>   (`04_TECH/28` §4). This unit adds no locator, no bundle verification and no UI or CLI surface: BIN and Intel HEX
>   remain `UNSUPPORTED` for analysis and `DESIGN_APPROVED / NOT_USER_AVAILABLE` as release attachments, because no
>   person can attach a file yet.
