---
title: "QA Acceptance Checklist"
doc_id: "FS-DEL-004"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# QA Checklist

## Import
- [ ] drag/drop
- [ ] file picker
- [ ] corrupted ELF
- [ ] wrong extension with valid ELF magic
- [ ] valid extension with wrong content
- [ ] very long path
- [ ] Unicode path
- [ ] read-only file

## Analyze
- [ ] stripped
- [ ] no DWARF
- [ ] zero symbols
- [ ] huge symbols
- [ ] endian handling
- [ ] section overflow display

## Compare
- [ ] same build
- [ ] completely different build
- [ ] symbol added
- [ ] removed
- [ ] renamed behavior documented
- [ ] baseline missing

## Gate
- [ ] clean/dirty git
- [ ] tag mismatch
- [ ] missing artifact
- [ ] over budget
- [ ] exact threshold
- [ ] unknown metadata

## Export
- [ ] destination exists
- [ ] permissions denied
- [ ] filename collision
- [ ] disk full simulation where possible
- [ ] manifest hashes verify
- [ ] partial export not presented as complete

## UI
- [ ] 100/125/150% scale Windows
- [ ] 1024x720
- [ ] keyboard only
- [ ] high contrast
- [ ] long localized strings
