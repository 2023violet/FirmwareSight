---
title: "Baseline Changelog"
doc_id: "FS-ROOT-CHANGELOG"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-28"
---

# Changelog

## Unreleased — P0 Technical Vertical Slice executed (2026-09-28)

No version is claimed here on purpose: `0.5.1` stays the only baseline, and `v0.6.0` is reserved for
an unconditional P0 `PASS`. This entry records work on the working tree.

### P0 source (first production code in this repository)

- Rust workspace, pinned toolchain 1.98.1, exactly four Phase-0 library crates:
  `firmwaresight-core` (declares zero dependencies), `-artifact`, `-report`, `-storage`;
- `fwsight` CLI: synchronous, deterministic JSON, exit codes 0 / 2 / 3 exercised, 4 / 5 / 6 not
  registered because no code path returns them;
- untrusted-input intake order `stat -> size guard -> streaming SHA-256 -> magic detect -> single
  read -> parse`, with the 512 MiB guard measured from both sides of the boundary;
- ELF parsing plus a memory-accounting model whose evidence ladder limits what a verdict may claim;
- GNU ld MAP adapter producing a real verified region fact, and refusing foreign-toolchain maps
  instead of approximating them;
- minimal SQLite persistence via `rusqlite + bundled`, one migration, transactional import,
  content-addressed dedupe;
- thin Tauri 2 desktop shell with a use-case-oriented command surface (`FixtureKey`, never a path),
  ts-rs-generated TypeScript bindings, and analysis off the calling thread;
- React 19.3 + TypeScript 6.0.3 summary screen, token-driven, five-state badges;
- real ARM ELF fixtures with recorded provenance and a regeneration script, four malformed inputs,
  goldens, and generated 100 / 256 / 512 MiB workloads (gitignored);
- one verification gate, `scripts/check.py`, that CI calls unchanged (14 steps, four job groups);
- `deny.toml` encoding the AGENTS.md dependency red lines as a ban list.

### P0 status

- `EXECUTED — CONDITIONAL_PASS (LOCAL)`; 102 Rust tests and 19 UI tests pass locally.
- Conditions, all outside this environment: CI has never run, `cargo deny` has never run, peak RSS is
  `NOT MEASURED`, and the desktop window was never opened.
- `P0_TECHNICAL_VALIDATION/` holds the evidence pack: the 17 documents the prompt names, plus
  `P0_EXECUTION_PROVENANCE.md` from the takeover and `P0_DESIGN_CHECKLIST.md` required by
  `AGENTS.md` 11 - 19 files, each citing executed commands.
- G1 is not claimed; V0 remains deferred with `0 / 8` external sessions; P1 is not authorized.

### Corrections made to this pack's own claims during P0

- `fixture.toml` recorded the linker invocation as `-p0-dual-region.ld`; the toolchain was called with
  `-T p0-dual-region.ld`. Regenerating proved both ELF files byte-identical, so the record was wrong
  and the artifacts were not.
- The release desktop binary was described as embedding the built UI. It did not: without Tauri's
  `custom-protocol` feature the dev-mode codegen embeds nothing. The feature was added and the
  production link re-measured (10,398,208 bytes against 10,330,112).
- The Rust test total was recorded as 101 before a ninth storage test landed; it is 102.
- `AGENTS.md` 11 requires every UI change to pass `templates/DESIGN_CHECKLIST_TEMPLATE.md`, and no
  filled copy existed. Running it found four things instead of confirming the work: `DESIGN.md` 3
  never tokenised border width while `DESIGN.md` 4 requires hairline borders, so the eight `1px`
  borders are a gap in the design contract that needs a frozen-asset version bump to close; the
  summary screen has **no live region**, so a screen reader is never told that an analysis started or
  finished; the fixture `select` styles only hover and focus; and capability badges display Core's enum
  words ("supported", "not-provided") rather than product copy. All four are recorded in
  `P0_KNOWN_LIMITATIONS.md`. The two accessibility findings were fixed in source and covered by a
  new test rather than filed for later; the other two stay open, one because it needs a frozen design
  asset to change and one because it is a boundary decision about who owns user-facing wording.
- The UI's `packageManager` pin drifted one patch below the frozen baseline: it read
  `pnpm@12.6.0` where `examples/package.baseline.json` and `04_TECH/10_TOOLCHAIN_BASELINE.md` both
  name 12.7.0. Nothing failed — which is why it is recorded as a catch of the baseline cross-check
  rather than of the gate. It is now pinned at 12.7.0, and the only lockfile change was that
  metadata plus the toolchain's own `@pnpm/exe` entries; no application dependency moved.
- `apps/desktop/ui/package.json` is the only package.json in the tree. The repo-structure baseline
  sketches a root `package.json`, `pnpm-workspace.yaml` and `.node-version`; P0 ships one frontend
  package with the node floor expressed as `engines: ">=24.0.0 <25.0.0"`, which meets that sketch's
  stated intent ("Frontend is one workspace package at MVP") without the root plumbing. Recorded as
  a named simplification in `P0_KNOWN_LIMITATIONS.md` rather than left implicit.

### Unchanged

- `BASELINE.yaml` still reports `baseline_version: 0.5.1`; `SHA256SUMS` still describes the frozen
  v0.5.1 package and was deliberately not regenerated; `DIRECTORY_TREE.txt` still lists that package,
  not this working tree. Six manifest entries now differ — all governance or execution records P0 was
  authorized to change, listed in `.ai/DECISIONS.md`.

## 0.5.1 — 2026-09-27

### V0 execution
- executed the authorized V0 Prompt v1.1 against the unique v0.5.0 baseline;
- created the `V0_VALIDATION/` execution workspace;
- implemented the seven-screen clickable static prototype;
- encoded MAP STATE A→B and parse-failure recovery states;
- implemented demo review acceptance and conditional Bundle flow;
- added moderator, screening, task, interview and privacy protocols;
- added session template and analysis registers;
- ran internal Node/state-machine/scope/design dry-run checks.

### Evidence limitation
- external participant sessions completed: 0;
- minimum required: 8;
- V0 status: `INCOMPLETE — insufficient external sample`;
- no V0 PASS/G1 claim.

### Scope
- no Cargo/Rust/Tauri/SQLite production implementation;
- P0 remains unauthorized in this V0 execution;
- no E1/E2/E3/GX feature implementation.

## 0.5.0 — 2026-09-27
Expert-review verification + whole-product consolidation baseline; complete seven-screen UI source set; Post-MVP candidate governance.

## 0.4.0 — 2026-09-27
Audit resolution + UI baseline freeze.

## 0.3.0 — 2026-09-26
Lifecycle baseline.

## 0.2.0 — 2026-09-26
Technical baseline.

## 0.1.0 — 2026-09-26
Initial product baseline.
