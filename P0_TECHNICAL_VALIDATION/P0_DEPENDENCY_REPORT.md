---
title: "P0 Dependency Report"
doc_id: "FS-P0-013"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Dependency Report

Requirement (AGENTS.md 4): every dependency must name the need it serves, say why the standard
library or an existing dependency was insufficient, and state its license, maintenance and build
impact. Nothing enters because it is common in the ecosystem.

Status: **LOCAL PASS**, with the license and advisory check reported as **NOT RUN locally**.

## Scale

`Cargo.lock` resolves **466 packages**, of which 6 are workspace members: **460 third-party Rust
packages**. The Tauri 2 shell accounts for the overwhelming majority; the four library crates
together add a small direct set. The UI adds 15 packages under pnpm, Python tooling adds one.

## Rust: direct dependencies, one line of justification each

| Crate | Version | Used by | Why it is here | License | Trusted core? |
| --- | --- | --- | --- | --- | --- |
| `object` | 0.40.0 | artifact | ELF section/symbol/header reading; writing and maintaining a container parser is a larger risk than depending on one | MIT OR Apache-2.0 | no - confined to `elf.rs`, no `object` type crosses a boundary |
| `sha2` | 0.11.0 | artifact | SHA-256 over a streaming reader; the standard library has no cryptographic hash | MIT OR Apache-2.0 | no |
| `thiserror` | 2.0.21 | artifact, report, storage | typed error enums per `05_ENGINEERING/03`; hand-writing `Display`/`Error`/`From` for nine variants per crate is boilerplate with a panic surface | MIT OR Apache-2.0 | no |
| `serde` / `serde_json` | 1.0.229 / 1.0.151 | report, desktop | deterministic JSON with declaration-order keys | MIT OR Apache-2.0 | no |
| `rusqlite` (bundled) | 0.40.2 | storage | the frozen SQLite binding; `bundled` keeps the build self-contained | MIT OR Apache-2.0 | no |
| `tracing` | 0.1.44 | storage, cli | structured diagnostics with an operation id | MIT | no |
| `tracing-subscriber` | 0.3.23 | cli | env-filtered stderr logging; only the CLI configures a subscriber | MIT | no |
| `clap` (derive) | 4.6.7 | cli | argument parsing and the frozen usage/`--version` surface | MIT OR Apache-2.0 | no |
| `tauri` | 2.12.0 | desktop | the frozen desktop framework; 2.x stable, not the 3.0 alpha | Apache-2.0 OR MIT | no |
| `tauri-build` | 2.7.0 | desktop (build) | required by the Tauri 2 build pipeline | Apache-2.0 OR MIT | no |
| `ts-rs` | 12.0.1 | desktop | generate the TypeScript IPC contract (ADR-0019); declared in exactly one crate | MIT OR Apache-2.0 | no |

`firmwaresight-core` still has an empty `[dependencies]` table. That is the strongest structural
statement P0 makes about trust: the domain layer has no code in it that anyone else wrote.

Build impact: a release `fwsight.exe` is 1,689,088 bytes; the SQLite `bundled` feature requires a
C compiler on the build host, which both CI images provide.

## Refused, with the reason

| Candidate | Why not |
| --- | --- |
| `hex` | a ten-line lowercase encoder; `Sha256::parse` validates the same alphabet, so encoder and validator cannot drift |
| `anyhow` | typed errors are the frozen error model. It does appear in the graph, but only inside Tauri - no first-party crate declares it |
| `tokio` (direct) | Core stays synchronous. `tokio 1.53.1` is Tauri's own runtime, reached only through `tauri::async_runtime`; `grep "tokio::" crates apps` returns nothing |
| `memmap2` | the guard's contract is "one bounded buffer or a typed refusal", and an mmap fallback would make the limit a soft suggestion |
| `sqlx` | rusqlite is frozen (AGENTS.md 6) |
| `gix` | Git facts in P0 come from declared provenance or stay Unknown; a repository library is a larger attack surface than the feature warrants |
| `reqwest` / `rustls` / `hyper` | no network in a local-first product; **zero occurrences in the graph** |
| `rayon` | P0 is synchronous; parallelism needs an ADR |
| `wgpu`, `axum`, `tonic` | explicitly forbidden by AGENTS.md 2 and 4 |
| `cargo-fuzz` | needs a nightly toolchain and a separate crate; regression tests carry the no-panic claim instead |
| a UI component library, a chart library, a state library | DESIGN.md 5 is a ruleset, not a dependency, and P0 shows one summary screen |

## Node and pnpm

`apps/desktop/ui/package.json` pins `packageManager: "pnpm@12.6.0"` and exact versions;
`.npmrc` sets `save-exact=true` and `engine-strict=true`. Installs run through Corepack so the
version that wrote `pnpm-lock.yaml` is the version that resolves it - the global pnpm on this
machine is 11.21.0 and was not used.

Runtime: `react 19.3.0`, `react-dom 19.3.0`, `@tauri-apps/api 2.12.0`.
Toolchain: `typescript 6.0.3`, `vite 8.3.1`, `@vitejs/plugin-react 6.1.1`, `eslint 10.11.0`,
`typescript-eslint 8.70.1`, `@eslint/js 10.0.1`, `vitest 5.0.2`, `jsdom 30.1.1`,
`@testing-library/react 16.3.3`, `@testing-library/dom 10.4.2`, `@types/react` /
`@types/react-dom` 19.3.0.

**TypeScript 6.0.3 rather than 7.0.2** is a real dependency decision, not an oversight:
`typescript-eslint@8.70.1` declares `typescript: ">=4.8.4 <6.1.0"`, and AGENTS.md 10 requires
frontend lint. Taking 7.x would have meant an unsupported parser or no lint gate. Recorded in
`P0_IMPLEMENTATION_LOG.md` 12.

## Python

Only `Pillow==12.3.0` (`scripts/requirements.txt`), used by `gen_desktop_icons.py` to write the
`.ico` tauri-build requires on Windows. Every other repository script is standard library only.
`gen_p0_fixtures.py` additionally needs `arm-none-eabi-gcc`, by hand only - no CI job runs it.

## Licenses

`deny.toml`'s allow list was derived from the licenses actually present in the resolved tree, read
out of each `.crate` archive: 220 `MIT OR Apache-2.0`, 113 `MIT`, 32 `Apache-2.0 OR MIT`, 18
`Unicode-3.0`, 17 `Zlib OR Apache-2.0 OR MIT`, 15 `MIT/Apache-2.0`, 10 `Unlicense OR MIT`, plus
single digits of MPL-2.0, BSD-3-Clause, ISC, 0BSD, CC0-1.0, MIT-0 and
`Apache-2.0 WITH LLVM-exception`.

Two notes recorded rather than smoothed over:

- 22 older transitive crates write their license as `MIT/Apache-2.0` style strings rather than an
  SPDX expression, so cargo-deny sees an opaque name. They are listed literally in the allow list
  instead of being ignored.
- LGPL-2.1-or-later appears once, inside `r-efi`'s
  `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. A disjunction is satisfied by its MIT option, so
  nothing here becomes copyleft-bound.

## Verification status

```
$ python scripts/check.py --only deny
=== [deny] cargo-deny
SKIPPED: `cargo deny` is not installed on this machine. CI installs it, ...
```

The policy file is therefore **written but not executed locally**, and CI has not run: there is no
push authorization in this task. `cargo-deny@0.20.2` is pinned in the workflow. The license,
duplicate and advisory results are **NOT RUN**, and no document in this pack claims they passed.
