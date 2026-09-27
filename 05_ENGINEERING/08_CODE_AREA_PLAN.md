---
title: "FirmwareSight Code Area Plan"
doc_id: "FS-ENG-009"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-27"
---

# FirmwareSight Code Area Plan

## Provenance note

The expert-team transcript states that a fuller `FS-ENG-009` draft was produced externally, but that standalone 41.92 KB file was **not included among the current uploaded attachments**.

This v0.5.0 document therefore does not pretend to be a byte-for-byte import. It reconstructs the authoritative plan from:
- the supplied transcript summary;
- `05_ENGINEERING/00_REPO_STRUCTURE.md`;
- the frozen v0.5 architecture/ADRs;
- the accepted DESIGN/tokens governance.

## 1. Root coexistence model

```text
FirmwareSight/
├── 00_GOVERNANCE/ … 10_AUDIT/    # product/document authority
├── assets/                         # frozen design/reference assets
├── examples/                       # baseline examples
├── schemas/                        # portable public/versioned contracts
├── templates/                      # governance templates
│
├── crates/                         # Rust library workspace
│   ├── firmwaresight-core/
│   ├── firmwaresight-artifact/
│   ├── firmwaresight-storage/
│   └── firmwaresight-report/
├── apps/
│   ├── cli/
│   └── desktop/
│       ├── src-tauri/
│       └── ui/
├── fixtures/
├── golden/
├── scripts/
└── .github/
```

No separate empty `docs/` mirror is introduced. The numbered root folders remain the documentation authority.

## 2. Rust workspace

### `firmwaresight-core`
Owns:
- domain models;
- normalization;
- evidence classification;
- diff;
- Gate;
- policy semantics;
- release model.

Must not depend on:
- Tauri;
- rusqlite;
- Tokio types;
- React/TS;
- OS shell primitives.

### `firmwaresight-artifact`
Owns:
- object/gimli integration;
- format detection;
- ELF/MAP adapters;
- normalized parser input/output mapping.

Must not own business verdicts.

### `firmwaresight-storage`
Owns:
- rusqlite bundled;
- migrations;
- transaction/repository adapters;
- query pagination.

SQLite types do not cross into Core public contracts.

### `firmwaresight-report`
Owns:
- report IR;
- deterministic JSON/HTML rendering;
- Release Bundle portable output;
- future renderer candidates only after approval.

### Phase-0 crate budget

These four library crates already consume the Phase-0 library-crate budget.

A fifth library crate requires:
- actual repeated responsibility evidence;
- dependency-direction justification;
- architecture review;
- ADR when it changes the architecture family/boundary.

### Applications

`apps/cli`
- Clap boundary;
- human + machine output;
- stable exit codes;
- calls shared application/Core logic.

`apps/desktop/src-tauri`
- thin process/window/capability shell;
- Tauri commands map to use cases;
- blocking parser/hash/storage work does not become async domain code;
- no business truth in command handlers.

## 3. React UI

```text
apps/desktop/ui/src/
├── app/             # shell/router/providers
├── pages/           # Overview / Analyze / Compare / ReleaseGate / BundleHistory
├── features/        # use-case UI composition
├── components/
│   ├── primitives/  # project-owned generic controls
│   ├── data/        # tables, ranked bars, diff rows
│   └── evidence/    # state/evidence/inspector components
├── ipc/             # the only Tauri invocation boundary
│   ├── client.ts
│   └── generated/   # ts-rs output; never hand-edit
├── styles/
│   └── tokens.css   # generated from root assets/design-tokens.json
└── test/
```

Rules:
- Pages do not parse binaries or evaluate Gate rules.
- Components consume DTOs/view models only.
- `ipc/` is the unique frontend gateway to Rust commands.
- root `assets/design-tokens.json` is numerical authority;
- generated `tokens.css` must have CI drift checking.

## 4. Shared schemas

Root `schemas/` is for durable/interchange contracts:
- release manifest;
- gate results;
- accepted reviews;
- future config/report contracts when made public/versioned.

Do not put transient Tauri IPC types here simply because they are serializable.

## 5. Fixtures

```text
fixtures/
├── elf/
├── map/
├── provenance/
├── malformed/
└── projects/
```

Each fixture directory should carry:
- artifact(s);
- `fixture.toml`;
- expected support classification;
- source/provenance/license note;
- purpose.

Real compiled fixtures are product infrastructure, not test clutter.

## 6. Golden outputs

```text
golden/
├── core/
├── cli/
└── reports/
```

Golden updates require:
1. semantic diff review;
2. reason in commit/PR;
3. explicit approval when public contract output changes.

Never bulk-regenerate goldens just to make CI green.

## 7. Naming/admission

- Rust crates: `firmwaresight-*`
- feature modules: domain vocabulary from Glossary
- new top-level directory requires concrete ownership distinct from existing zones
- new library crate triggers crate-budget review
- new public schema triggers compatibility/version review
- new dependency follows dependency policy
- architecture boundary changes require ADR

## 8. Phase-0 creation scope

Create only when V0/P0 implementation is explicitly authorized:

### Create in P0
- root Cargo workspace;
- four library crates;
- CLI app;
- desktop shell;
- React UI minimal shell;
- fixtures needed for P0;
- minimal goldens;
- generation scripts actually required by the slice.

### Do not pre-create
- plugin system;
- cloud/server packages;
- shared UI package;
- GPU package;
- SBOM/CVE service modules;
- updater service;
- editor extension;
- GitHub Action package;
- speculative renderer crates.

Empty architecture is not progress.
