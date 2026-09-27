---
title: "P0 Architecture Check"
doc_id: "FS-P0-004"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Architecture Check

Requirement: the four Phase-0 library crates, their dependency direction and the headless-Core
rules must survive being implemented, not just being written down.

Status: **LOCAL PASS**

## 1. Exactly four Phase-0 library crates

```
$ python -c "import tomllib;d=tomllib.load(open('Cargo.toml','rb'));print(d['workspace']['members'])"
['apps/cli', 'apps/desktop/src-tauri', 'crates/firmwaresight-core', 'crates/firmwaresight-artifact',
 'crates/firmwaresight-storage', 'crates/firmwaresight-report']
```

Library crates: `firmwaresight-core`, `firmwaresight-artifact`, `firmwaresight-storage`,
`firmwaresight-report` - four, matching the Phase-0 budget. The two applications
(`fwsight`, `firmwaresight-desktop`) sit outside that budget by design. No fifth library crate
appeared during implementation, which is the usual way this limit fails.

## 2. Dependency direction

Read from each manifest's `[dependencies]` table:

| Crate | Depends on | Never depends on |
| --- | --- | --- |
| `firmwaresight-core` | (nothing) | every other crate, all third-party code |
| `firmwaresight-artifact` | core, `object`, `sha2`, `thiserror` | tauri, rusqlite, serde, ts-rs |
| `firmwaresight-report` | core, serde, serde_json, thiserror | artifact, storage, tauri |
| `firmwaresight-storage` | core, rusqlite (bundled), thiserror, tracing | artifact, report, tauri |
| `fwsight` (CLI) | artifact, report, clap, tracing | tauri, ts-rs |
| `firmwaresight-desktop` | core-adjacent crates via artifact/report/storage, tauri, ts-rs, serde | tokio declared directly |

The arrow never points towards the UI: `firmwaresight-core` has an empty `[dependencies]` table
and is the only crate with no third-party code at all.

Evidence command:

```
$ grep -rn "firmwaresight_core::" crates apps --include=*.rs
crates/firmwaresight-artifact: 28    crates/firmwaresight-report: 17    crates/firmwaresight-storage: 10
```

## 3. Four findings corrected during the check

- **Declared but unused dependencies.** The CLI declared `firmwaresight-core` and
  `firmwaresight-storage`, and the artifact crate declared `tracing`, none of which any source
  file referenced. Removed; `cargo test --workspace` still passes with 102 tests. An unused
  dependency is still a dependency in AGENTS.md 10's "no unrelated dependency" sense.
- **A desktop build claim that measurement reversed.** This pack first said the release binary
  embedded the built UI. It did not: without Tauri's `custom-protocol` feature the codegen runs in
  dev mode and embeds nothing, which is why deleting `dist/` did not break `cargo check`, `cargo
  test --no-run` or `cargo build --release`. Declaring the feature made
  `clippy --all-targets --all-features` fail on a missing `dist/` with
  `The `frontendDist` configuration is set to "../ui/dist" but this path doesn't exist` - the
  shipping configuration is now the one the gate compiles. See `P0_IMPLEMENTATION_LOG.md` 18.
- **`firmwaresight-storage` uses `firmwaresight-artifact` once.** That reference is inside
  `tests/storage.rs` and is declared under `[dev-dependencies]`, so the production graph stays
  clean. It is noted here because a grep alone cannot tell the two apart.
- **ts-rs placement.** An earlier draft exposed an optional `ipc-ts` feature on
  `firmwaresight-report` so its DTOs could derive `TS`. That made a portable-output crate aware
  of one consumer's UI tooling. It was removed in favour of DTO mapping in the desktop crate
  (see `P0_IMPLEMENTATION_LOG.md` 9).

## 4. Core rules checked directly

| Rule (AGENTS.md 3) | Evidence |
| --- | --- |
| Headless, no Tauri/React/SQLite | `[dependencies]` of core is empty; grep finds no such name in the crate |
| No Tokio types exposed | `grep -rn "tokio" crates/` returns nothing; the desktop uses `tauri::async_runtime` only |
| Deterministic and mostly synchronous | every public entry point is a plain `fn`; no `async` appears in core, artifact, report or storage |
| Project-authored `unsafe` forbidden | `#![forbid(unsafe_code)]` in core, artifact, report, storage and the desktop lib; `grep -rn "unsafe " crates apps --include=*.rs` returns no hits outside dependency code |
| Business facts originate in Core | the desktop projection reads only `AnalyzeResultDto`; `analysis_runs_entirely_off_the_calling_thread` proves the same code path runs on a worker |

## 5. Storage and UI boundaries

- SQLite appears only in `firmwaresight-storage` (`rusqlite` with `features = ["bundled"]`),
  per AGENTS.md 6.
- The web frontend receives no shell or filesystem permission: the capability file is the frozen
  baseline verbatim, with `permissions = ["core:default"]` and `windows = ["main"]`
  (`apps/desktop/src-tauri/capabilities/main.json`, compared against
  `examples/tauri-capability.baseline.json`).
- No network plugin is declared anywhere in the desktop manifest.

## 6. What the P0 tree actually adds

`DIRECTORY_TREE.txt` and `SHA256SUMS` still describe the frozen v0.5.1 package, so the source layout
P0 created is recorded here instead of being written over a baseline artifact.

133 files added since start HEAD `e086e98`, by top-level area:

| Area | Files | What is in it |
| --- | --- | --- |
| `apps/` | 49 | `cli/` (`fwsight`), `desktop/src-tauri/` (shell, IPC DTOs, capability, icons, parity tests), `desktop/ui/` (React summary, generated bindings, tests) |
| `crates/` | 30 | the four Phase-0 libraries and their test suites |
| `P0_TECHNICAL_VALIDATION/` | 18 | this evidence pack: the 17 the prompt names plus the takeover provenance doc |
| `fixtures/` | 14 | two real ELF fixtures with sources and `fixture.toml`, four malformed inputs, manifest |
| `scripts/` | 8 | the gate, fixture and workload generators, golden updater, token and icon generators |
| `golden/` | 5 | CLI and Core goldens, plus `reports/README.md` explaining what may live there |
| root and config | 9 | workspace `Cargo.toml` and lock, `rust-toolchain.toml`, `deny.toml`, `.cargo/config.toml`, `.github/workflows/p0-check.yml`, `.gitignore`, `.gitattributes`, the prompt copy under `10_AUDIT/SOURCE_PROMPTS/` |

Sizes, tracked files only: 33 Rust files / 7,239 lines (of which 1,366 lines live in `tests/`), and
22 TypeScript files / 1,131 lines. Excluded by `.gitignore` and therefore absent from the
repository: `target/`, `node_modules/`, `apps/desktop/ui/dist/`, and the generated
100 / 256 / 512 MiB workloads in `fixtures/generated/`.

## Command

```
$ RUSTUP_TOOLCHAIN=stable cargo test --workspace
... 102 passed; 0 failed
$ RUSTUP_TOOLCHAIN=stable cargo clippy --workspace --all-targets --all-features -- -D warnings
(no output; exit 0)
```
