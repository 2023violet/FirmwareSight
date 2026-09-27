---
title: "Git Provenance Adapter"
doc_id: "FS-TECH-023"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Git Provenance Adapter

## v0.2 Decision

Use installed `git` CLI as the first read-only provenance adapter.

Do not add `gix` to MVP dependency graph yet.

## Why

Needed facts are small:
- repository root；
- HEAD commit；
- exact tag；
- dirty state；
- branch/reference where useful。

System Git:
- matches developer repository semantics；
- avoids a large additional Rust dependency；
- is normally present in Git-based firmware workflows。

## Security

- spawn executable directly；
- never use shell command strings；
- pass repository path as argument；
- use stable machine-readable formats (`--porcelain` etc.)；
- timeout；
- capture exit status/stdout/stderr；
- sanitize logged paths。

## Degraded state

No Git installed or repo unavailable:
- Artifact analysis still works；
- Git provenance = Unknown；
- Git-dependent Gate rules become Review/N/A according to policy。

## Revisit trigger

Adopt `gix` if:
- users require Git provenance without system Git；
- process startup becomes measurable bottleneck；
- Git CLI compatibility causes real support burden。

Requires ADR because it changes adapter dependency and behavior.
