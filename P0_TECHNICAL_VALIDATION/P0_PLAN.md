---
title: "P0 Plan"
doc_id: "FS-P0-002"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Plan

## Requirement

P0 proves or breaks the architecture claim, not the product claim: one headless Rust Core
analyzes real firmware artifacts, and the same normalized facts reach a synchronous CLI and a
Tauri 2 desktop through a typed IPC boundary, with structured history in SQLite and a size guard
that holds against a hostile input.

## Authority

- `FirmwareSight_P0_Technical_Vertical_Slice_EXECUTION_PROMPT_v1.1_ARCHITECT_REVIEWED`
  (SHA-256 recorded in `P0_EXECUTION_PROVENANCE.md`)
- ADR-0020 authorizes the P0 track in parallel with the deferred V0 validation
- AGENTS.md 1-11, ADR-0018 (UI), ADR-0019 (ts-rs boundary), ADR-0021 (two memory budgets)
- `04_TECH/09_TECH_DECISION_MATRIX.md`, `04_TECH/16`, `04_TECH/23`, `05_ENGINEERING/03`

## Sequence executed

| # | Step | Exit condition that closed it |
| --- | --- | --- |
| 1 | Governance and provenance first | Prompt archived byte-exact, register updated, `BASELINE.yaml` marked P0 in progress |
| 2 | Workspace and toolchain | `cargo metadata` with exactly 4 Phase-0 library crates + 2 applications |
| 3 | Core domain | `#![forbid(unsafe_code)]`, no external dependency, 39 tests |
| 4 | Untrusted intake and guard | stat before allocation, streaming SHA-256, single immutable read |
| 5 | ELF reader on `object` | section/symbol/identity facts with evidence classification |
| 6 | GNU ld MAP adapter | region table, load addresses, object contributions; foreign MAPs refused |
| 7 | Memory accounting | two budgets proven on a real linked fixture with dual-accounted `.data` |
| 8 | Report and CLI | deterministic JSON, frozen exit codes 0/2/3, goldens |
| 9 | Storage and typed IPC | migrations, transactional import, ts-rs bindings |
| 10 | Desktop shell and UI | thin Tauri 2 commands off the event loop, strict React summary |
| 11 | Parity, performance, CI, reports | CLI/Desktop parity test, measured workloads, one shared gate |

## Deliberate ordering decisions

Domain types were built and tested before any parser existed, so the parser had to fit the model
rather than the model being retrofitted around whatever `object` returned. The MAP adapter came
after the ELF reader for the same reason: the evidence ladder needed a second rung to be
demonstrable at all.

## Scope guards kept

- No `diff`, `gate` or `release` command was registered, so none can be mistaken for working.
- Exit codes 4, 5 and 6 remain unreachable, and the code that would return them does not exist.
- No network, telemetry, updater, signing or cloud surface was added.
- Git provenance was left as an Unknown capability rather than pulling in a Git library.
