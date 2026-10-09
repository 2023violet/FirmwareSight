---
title: "Artifact Analysis Pipeline"
doc_id: "FS-TECH-017"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-09"
---

# Artifact Analysis Pipeline

## 1. Pipeline

```text
Path selected
   ↓
Stat / size guard
   ↓
Streaming SHA-256
   ↓
Magic/format detection
   ↓
Read immutable bytes
   ↓
Adapter parse
   ↓
Normalize
   ↓
Evidence classify
   ↓
BuildSnapshot
   ↓
SQLite transaction
```

## 2. File trust

All input is untrusted.

Required:
- bounds checked parser API；
- no project-authored unsafe parsing；
- panic is a bug；
- malformed fixtures；
- fuzz targets for parser boundaries。

## 3. File loading

MVP:
- standard buffered file read for parse；
- hash streamed separately or combined if implementation permits；
- one immutable byte buffer handed to parser。

No `memmap2` baseline.
Reason:
- firmware artifacts are normally modest；
- mmap adds platform/file-mutation edge cases；
- add only after profiling.

Default supported import size guard should be conservative and configurable; initial engineering target is up to hundreds of MB, not multi-GB firmware.

## 4. ELF

Use `object`:
- architecture；
- bitness；
- endianness；
- entry；
- sections；
- symbols；
- build-id where exposed。

Do not assume:
- debug info exists；
- symbol table exists；
- section naming is identical across toolchains。

## 5. DWARF

Use `gimli` only when needed to derive extra evidence:
- source/unit association；
- richer attribution。

Missing/malformed DWARF:
- capability warning；
- base ELF analysis remains available。

## 6. MAP

Toolchain adapter interface:

```text
MapAdapter
  detect(text/header) -> confidence
  parse(...) -> NormalizedMapEvidence
```

MVP:
- GNU ld only.

No generic parser that silently “mostly works”.

## 7. BIN/HEX

BIN:
- hash；
- size；
- release packaging metadata。

Intel HEX:
- hash；
- parsed address span；
- payload byte count；
- validation of records/checksum。

They cannot supply symbols unless paired with ELF/MAP.

> **Dated 2026-10-09 (`ADR-0030`) — this section is the pre-C1 design intent, and C1 changes what it delivers.**
> Analysis stays ELF-only: the pipeline refuses every non-ELF input (`pipeline.rs:83-97`) and a snapshot seals from a
> primary ELF plus an optional MAP (`build_snapshot.rs:123-136`), so nothing in §7 is produced *by analysis*. What is
> decided instead is the Release-side half: BIN and Intel HEX ship as **release-attached byte evidence** — hash and
> size observed on the bytes actually read, kind declared, `release packaging metadata` as the shipped bundle's own
> record. Of the Intel HEX list above, `parsed address span`, `payload byte count` and `validation of
> records/checksum` are **excluded**, not deferred: no parser exists and `ADR-0030` forbids inventing one for C1, so
> those properties stay `Unknown / Not verified` and are never reported as zero. The last sentence above holds exactly
> as written — and C1 does not weaken it into a symbols claim either, because an attachment is never paired with
> anything in a snapshot. Specification: `04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md`. Status: approved design,
> not implemented.

## 8. Normalization

Parser-specific facts become stable core models before persistence.

UI never renders directly from `object::Symbol` or parser-specific types.


## v0.4 Full-buffer guard

MVP default:
`max_full_buffer_bytes = 512 MiB`

Behavior:
- stat before allocation；
- larger artifact returns typed `ArtifactTooLarge`；
- no implicit mmap fallback；
- P0 benchmarks include approximately 100/256/512 MiB fixtures or synthetic valid workloads；
- raising the limit is explicit project/user policy, never silent。

This is a safety/default engineering limit, not a claim that 512 MiB firmware is a typical target.
