---
title: "Domain Model"
doc_id: "FS-TECH-003"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
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
