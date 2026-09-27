---
title: "P0 IPC and Core/CLI/Desktop Parity"
doc_id: "FS-P0-009"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 IPC and Core/CLI/Desktop Parity

Requirement: one artifact analyzed through the CLI and through the Desktop must yield identical
Core facts. Neither surface may recompute anything, and the IPC boundary must be typed, bounded
and generated rather than hand-maintained.

Status: **LOCAL PASS**

## The claim, as a test

```
$ RUSTUP_TOOLCHAIN=stable cargo test -p firmwaresight-desktop --test desktop_parity
running 7 tests ... test result: ok. 7 passed; 0 failed
```

`desktop_and_cli_report_the_same_core_facts_for_the_same_bytes` drives the real shell path
(`Session::analyze_and_store`, the same function the Tauri command calls) and compares each field
against the committed CLI golden `golden/cli/p0-dual-region-analyze.json`:

| Field group | Compared |
| --- | --- |
| `sha256`, `byteSize`, `architecture`, `bitness`, `endianness`, `entryPoint` | artifact |
| `nonvolatileImageFootprint.bytes`, `runtimeRamFootprint.bytes` | memory totals, plus an explicit assertion that the two are **not** equal |
| `sections`, `symbols` | counts |
| `elf`, `sections`, `symbols`, `debugInfo`, `map`, `objectAttribution`, `git` | capabilities |

`the_desktop_projection_keeps_the_same_evidence_claims_as_the_cli` adds the provenance: layout
source, weakest evidence basis, dual-accounted section count, evidence total, and the invariant
that observed + derived + declared + unknown equals the total.

`without_region_evidence_the_desktop_reports_weaker_evidence_too` is the negative case: with no
MAP the desktop must report `elf-address-and-flags`, `layoutSource: none` and
`admissibleForHardBlock: false`. Parity on the easy path would still allow the UI to overstate
confidence, so the weaker path is asserted too.

## Why drift is structurally hard

The desktop does not read Core and re-derive numbers. `service::summarize` takes the report
crate's `AnalyzeResultDto` - the same type the CLI renders - and copies fields into the bounded
IPC shape. There is one projection, not two implementations of the same arithmetic.

```
Core domain (firmwaresight-core)
  -> pipeline::analyze                 (artifact)
     -> AnalyzeResultDto               (report)  <- CLI renders this
        -> AnalysisSummaryDto          (desktop) <- IPC sends this; ts-rs generates the TS
```

The UI recomputes nothing: `capabilityState()` and `budgetWord()` in `App.tsx` map a word the
shell already chose onto one of the five frozen states. No byte is summed, no threshold is
compared, no verdict is formed in TypeScript.

## Generated TypeScript, and its drift gate

`ts-rs` is a dependency of exactly one crate: `apps/desktop/src-tauri`. No Core, artifact, report
or storage type derives `TS` (ADR-0019).

```
$ cargo test -p firmwaresight-desktop
test ipc::export_bindings_analysissummarydto ... ok   (10 export tests)
```

Output: 10 files in `apps/desktop/ui/src/ipc/generated/`, written to the directory configured by
`TS_RS_EXPORT_DIR` in `.cargo/config.toml`. The application imports them through the hand-written
`src/ipc/types.ts`, so a rename has one place to be noticed.

```
$ python scripts/check.py --only drift
PASS  drift/design tokens
PASS  drift/desktop icons
PASS  drift/ipc bindings
PASS  drift/ipc bindings unchanged
PASS  drift/goldens unchanged
5/5 steps passed
```

`git diff --exit-code` is the assertion. A committed `.ts` that no longer matches its Rust source
fails the gate; the files themselves are never hand-edited.

## The payload is bounded

`the_ipc_payload_is_bounded_and_carries_no_path` asserts:

- no `symbols`, `sections` or `evidence` array crosses the boundary - only counts and an evidence
  summary with five integer buckets;
- no key named `path`, `filePath` or `directory` appears anywhere in the serialized payload;
- the whole summary stays under 8 KiB for these fixtures.

The full symbol and section lists do exist - they are written to SQLite by the same call - they
simply do not travel into a WebView.

## Blocking work never runs on the event loop

`get_analysis_summary` is an `async` command whose body is one
`tauri::async_runtime::spawn_blocking` hop; `Session` is moved to that thread.

`analysis_runs_entirely_off_the_calling_thread` proves the arrangement is real rather than
intended: it asserts `Session: Send + Sync`, constructs the session on a spawned OS thread,
analyzes there, and compares the resulting SHA-256 against the CLI golden. A core that had been
made async to suit Tauri would fail this differently - the Core API stays synchronous
(`pipeline::analyze(&request) -> Result<Analysis, ArtifactError>`) and the thread hop is owned
entirely by the application layer.

## Contract shape notes

- u64 byte counts are emitted as TypeScript `number` (`@ts(type = "number")`). Every P0 value is
  far below 2^53, so this is exact; promising `bigint` would overstate the precision the artifact
  crate claims.
- Optional fields serialize as `null` rather than being omitted, so the generated type describes
  the payload the UI will actually read. The CLI's own JSON keeps omitting them, because its
  golden must stay free of null noise.

## What is not proven here

Parity is proven for the two committed fixtures through one projection. It is not a proof that
any future field stays in sync - the mechanism that keeps it honest is the drift gate plus the
golden comparison, not a claim of completeness.
