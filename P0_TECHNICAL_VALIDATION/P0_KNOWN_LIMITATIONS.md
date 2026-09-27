---
title: "P0 Known Limitations"
doc_id: "FS-P0-015"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Known Limitations

Stated plainly, because a slice that hides its edges will be trusted where it should not be.

## Product surface deliberately absent

| Missing | Why it is absent |
| --- | --- |
| `diff`, `gate`, `release prepare` | later phases. They are not registered as CLI subcommands, so they cannot be mistaken for working features |
| Exit codes 4, 5, 6 | unreachable; no code path returns them |
| Compare / Gate / Bundle screens | P0's UI is one summary screen; the seven-screen V0 prototype is explicitly not reimplemented |
| Portable public schemas | the emitted shape is stamped `p0-internal`. `release-manifest v1` remains unpublished until the Gate and Bundle phases exist |
| Symbol table in the UI | not needed in P0, and 100k symbols over IPC would be the boundary design failing rather than the product lacking a view |
| Installer, signing, updater | `bundle.active = false`; AGENTS.md 7 reserves these for a signing ADR |
| Git provenance | the capability reports `unknown`. No Git library was added, and no subprocess is spawned |

## Formats and platforms

- **ELF only.** BIN/HEX/MotS records, PE and Mach-O are detected as unsupported rather than
  parsed partially.
- **GNU ld MAP only.** ArmClang, IAR and vendor IDE maps are refused with `MapUnsupported`; there
  is no fallback reader and no attempt to approximate them.
- **No DWARF consumption.** Debug sections are recognized and excluded from both budgets, but no
  compile-unit, source-line or inlining fact is extracted.
- **Program headers are read for identity only.** Segment-level accounting is not implemented.
- **Two linker layouts tested.** Executable-in-RAM, external SRAM, DMA pools, overlay regions and
  MPU-aligned sections are unproven.

## Memory accounting

- Footprint, not headroom: stack high-water, heap usage and allocator fragmentation are unknown,
  because P0 has no device run data to consult.
- `exact` means "exact with respect to what the file declares". A linker script that misstates a
  region is reproduced faithfully as a well-labelled wrong number.
- Peak RSS is **NOT MEASURED** (`P0_PERFORMANCE_REPORT.md`), so the guard's memory behaviour is
  argued from the code path rather than observed.

## Desktop

- The app analyzes two closed fixture keys. There is no file picker: `analyze(path)` would be
  "read arbitrary file" with extra steps, so the user-authorized selection path arrives with a
  dialog capability and its own ADR.
- **The windowed application was never launched.** The release binary builds and links, the UI
  production build succeeds and the rendering logic is tested under jsdom, but nobody opened the
  actual WebView on a desktop. Layout on a real display, WebView2 availability and CSP behaviour
  in the shipped shell are therefore unverified. Start it with
  `cargo run --release -p firmwaresight-desktop` after `pnpm -C apps/desktop/ui build`.
- `style-src 'unsafe-inline'` is present in the CSP because Vite injects inline styles in dev mode.
- The application-data database path is resolved from Tauri's platform directory; it has only been
  exercised on Windows.
- Mobile crate-types (`staticlib`, `cdylib`) and `tauri.conf.json` bundle targets were left out
  rather than declared without a test device.

## Storage

- Schema version 1 covers only what P0 writes. Gate and release tables arrive with their own
  migrations.
- `foreign_keys = ON` and a 5 s busy timeout are asserted at open, but multi-process concurrency
  is untested - nothing in P0 runs two FirmwareSight processes against one database.
- No legacy migration exists or is needed: there is no GA database to upgrade.

## Verification gaps

- **CI has never run.** The workflow is committed and locally validated; `p0-check.yml` has no
  execution history, so `CI PASS` is claimed nowhere.
- **`cargo deny` has never run.** The policy file is written; licenses, duplicates and advisories
  are NOT RUN.
- **No fuzzing.** `cargo-fuzz` needs a nightly toolchain and a separate crate, which the tool
  policy did not authorize. The no-panic claim rests on regression tests over four malformed
  fixtures plus a truncated-ELF and foreign-MAP case.
- **Fixture regeneration needs a toolchain most contributors lack.** `arm-none-eabi-gcc` is only
  required to rebuild fixtures, never to test them - but the ability to regenerate is therefore
  single-machine in practice.
- **One host, one build.** All timings come from one Windows 10 machine with a release build;
  they are not comparable to CI or to another platform.

## Process honesty

- The V0 prototype track remains deferred and unvalidated (ADR-0020). Nothing in P0 upgrades a V0
  claim, and V0's gate recommendation was not altered.
- Where the baseline was silent, P0 made a decision and wrote it down rather than inventing a
  precedent: `SnapshotId` derivation, the RAM discriminator, `p0-internal` schema naming, and
  TypeScript 6.0.3. Each is in `P0_IMPLEMENTATION_LOG.md`.
- One file was deleted during P0 (`golden/reports/p0-basic-summary.json`), because it carried
  wrong values under a right-looking name and no test read it. It is recoverable from git.
