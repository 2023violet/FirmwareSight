---
title: "P1-A0 Desktop Real-Window Smoke Report"
doc_id: "FS-P1A0-002"
product: "FirmwareSight"
version: "0.1.0"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P1-A0 Desktop Real-Window Smoke Report

Runs prompt §22: seventeen steps, driven through real native dialogs in a real display session,
against the shipping release binary - not `cargo run`, not a dev server, not a mocked picker.
"Native dialog 没有真实打开: P1-A0 不得宣布完成" is taken literally, so every dialog below is a
separate Win32 window owned by the same process.

Status: **PASS - 17 of 17 observed, twice.** The first pass ran against the 23:42 binary and proved
the sequence; a design-checklist pass then changed one CSS declaration (`.control:disabled`), so the
frontend and the release binary were rebuilt and **all seventeen steps were re-run against the binary
that this commit ships** - the table below is that second pass. An even earlier pass could not prove
the project-identity claim at all, because the chosen bytes deduped against P0's local history; the
database was reset to an empty one before either recorded pass.

## Environment and configuration

| | |
| --- | --- |
| Host | Windows 10 x64, build 10.0.19045, console session 1 |
| Binary | `target/release/firmwaresight-desktop.exe`, 11,006,464 bytes, `cargo build --release -p firmwaresight-desktop --features custom-protocol` - `Finished 'release' profile [optimized] target(s) in 3m 28s`, written 2026-09-29 01:11 |
| Embedded assets | `dist/assets/index-JlA6U4Km.css` (8.37 kB) and `dist/assets/index-DEIL8bLH.js` (232.06 kB), built at 01:08 after the last CSS change, so the binary predates nothing in the tree |
| Process / window | pid 24732, window id 2102420, title `FirmwareSight - Analyze`, outer 1056×799, client 1015×768 |
| Document origin | `http://tauri.localhost/` (`RootWebArea` node value in the accessibility tree) |
| Dialog windows | 2230426 `Choose the firmware artifact to analyze`, 592502 `Choose the linker MAP for this artifact`, 661290 (the cancelled pick), 269576 (the malformed pick) - all `processId: 24732`, i.e. opened by the app, not by the test harness |
| Inputs chosen by a "user" | `%LOCALAPPDATA%\Temp\p1a0-smoke\customer-app.elf` - byte-identical to `fixtures/elf/p0-dual-region/firmware.elf` (`c6d0feed0a1e…4e62`); `customer-app.map` - byte-identical to `fixtures/elf/p0-dual-region/firmware.map` (`c4182c5bcc…2a61`); `not-a-firmware.elf` - 41 bytes, first line `this is a text file wearing an ELF name` |
| Database | `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite`, opened from a real user profile, with no P0-era history in it |

The inputs sit outside the repository on purpose: the point of P1-A0 is that a person points the app
at a file the app's catalog never heard of.

## The seventeen steps

| # | Step | Observed |
| --- | --- | --- |
| 1 | Shipping/release configuration opens | window listed, foreground, not minimized; heading `Analyze`, subhead naming the no-diff/no-gate/no-release boundary; no `P0 Technical Summary` and no `Fixture` label anywhere |
| 2 | Choose firmware artifact | real mouse click at window-relative (105, 175), `foregroundStatus: matched` |
| 3 | Native dialog appears | window 2230426 titled `Choose the firmware artifact to analyze`, a Win32 common item dialog (tree of quick access, `文件名(N):` edit focused) |
| 4 | Select real ELF fixture | path typed into the dialog's own filename field, accepted with Return |
| 5 | UI shows filename, not full path | control row read `customer-app.elf` and `MAP: Not provided`; no drive letter, no directory, in the rendered text or in the payload |
| 6 | Analyze succeeds | full report rendered: Artifact, Memory, Capabilities, Counts, footer |
| 7 | Facts match CLI/Core for the same bytes | every field equal to `fwsight analyze --json customer-app.elf` - table below |
| 8 | Add MAP | the button appeared only after a selection existed; clicked at (459, 175) |
| 9 | Native dialog appears | window 592502 titled `Choose the linker MAP for this artifact` |
| 10 | Choose matching GNU ld MAP | `customer-app.map` typed and accepted; control row became `MAP: customer-app.map` and the button pair became `Replace MAP` / `Remove MAP` |
| 11 | Analyze again | clicked Analyze at (727, 175) |
| 12 | MAP-dependent evidence strengthens | `Load evidence`: *Not admissible for a hard limit* → **Admissible for a hard limit**, basis `elf-address-and-flags` → `map-memory-configuration+elf-load` with "region evidence came from the linker"; `Layout source` `none` → `map`; `MAP` capability `not-provided` → `provided`; `Object attribution` `unavailable` → `available`; `Evidence` `10 recorded` → `11 recorded`. The two budget figures stayed Exact 160 / Exact 72, which is correct: the MAP strengthens provenance, it does not invent bytes |
| 13 | Cancel dialog is non-error | Escape in window 661290 closed the picker; no `role="alert"`, selection still `customer-app.elf` with `MAP: customer-app.map`, strengthened report unchanged |
| 14 | Select malformed / unsupported input | second artifact dialog (window 269576) → `not-a-firmware.elf`; control row showed the new name and `MAP: Not provided`, while the previous report stayed on screen |
| 15 | ErrorPanel appears | `Analysis failed` / What happened `That file is not a format FirmwareSight can analyze yet.` / Code `ERR-FORMAT-0001` / Why we know `unsupported format: Binary at not-a-firmware.elf` / What to do `Provide an ELF linker output, or a BIN/HEX image.` / Diagnostics ID `op-609c-18d98be1034ea388` |
| 16 | Last-good summary remains | the panel sat above the note `Previous analysis of customer-app.elf. It is not an analysis of not-a-firmware.elf.`, and the complete prior report stayed rendered below it - visible, labelled, and not passed off as the new candidate |
| 17 | Clean close | Alt+F4; `tasklist` then reports no `firmwaresight-desktop.exe`; the database and its WAL remain in the app-data directory |

Two things this pass did **not** observe in the window, stated rather than glossed:

* The `Analyzing…` live region. Local analysis of a 10,960-byte fixture finishes inside one frame, so
  it cannot be captured by hand. It is covered by the UI test that defers the promise and asserts
  `role="status"`, the disabled select-equivalent controls, and the disabled button.
* A hard-block or gate state of any kind, because none exists in this build.

## CLI parity for the same bytes (step 7 and step 12)

`fwsight analyze --json` was run against the exact file the dialog selected, with and without the
exact MAP the second dialog selected. Every value the window displayed appears here.

| Fact | CLI, ELF only | Window, ELF only | CLI, ELF + MAP | Window, ELF + MAP |
| --- | --- | --- | --- | --- |
| `sha256` | `c6d0feed0a1e4153d80592519f0bfe1a11be67d756f2ecd811799b4f7def4e62` | same | same | same |
| size | 10,960 bytes | 10,960 bytes | 10,960 bytes | 10,960 bytes |
| format | Arm / 32 / little | `Arm 32-bit little` | same | same |
| parser | `object-elf/0.40` | `object-elf/0.40` | same | same |
| entry | `0x08000039` | `0x08000039` | same | same |
| build id | null + `no .note.gnu.build-id section is present` | `-` + same reason | same | same |
| nonvolatile | exact 160 | `Exact 160 bytes` | exact 160 | `Exact 160 bytes` |
| runtime RAM | exact 72 | `Exact 72 bytes` | exact 72 | `Exact 72 bytes` |
| layout source | `none` | `none` | `map` | `map` |
| weakest basis | `elf-address-and-flags`, admissible false | *Not admissible for a hard limit* | `map-memory-configuration+elf-load`, admissible true | *Admissible for a hard limit* |
| dual-accounted | `elf.section_header[3]`, `elf.section_header[5]` | same two, count 2 | same | same |
| excluded metadata | 1,975 bytes | 1,975 bytes | 1,975 bytes | 1,975 bytes |
| capabilities | supported / available / available / available / not-provided / unavailable / unknown | identical words, each with an icon | …`provided` / `available` / unknown | identical words |
| counts | 17 sections, 32 symbols, 10 evidence (10 observed) | `17`, `32`, `10 recorded 10 observed, 0 derived, 0 declared, 0 unknown` | 17, 32, 11 evidence | `11 recorded 11 observed, 0 derived, 0 declared, 0 unknown` |
| snapshot id | `snap-c6d0feed…-p0-normalize-1` | footer, identical | `snap-c6d0feed…-p0-normalize-1` (see finding 1) | footer, identical |

## What the local database held afterwards

Read-only queries against the same file the app had just closed:

| Check | Result |
| --- | --- |
| Projects | one row: `local-desktop` / `Local analyses`, created `2026-09-28T16:26:06Z` |
| Builds | 1 row, `project_id = local-desktop`, snapshot `snap-c6d0feed…-p0-normalize-1` |
| `p0-desktop` rows in this database | 0 - the user-chosen analysis did not land under the demo identity |
| Dedupe | three successful analyses of the same bytes across the two passes produced one build row |
| Failed analysis | no row at all: a failure never becomes a snapshot |
| Migrations | `schema_migrations` = `(1, 0001_initial)`, `(2, 0002_evidence_keyed_by_build)`; no third version, `SCHEMA_VERSION` still 2 |
| Raw bytes | not stored; `artifacts` keeps metadata plus the path the user chose, and that column never crosses IPC |
| Evidence rows | 10, from the first import. The MAP-strengthened 11-item set was reported by the UI but not re-imported, because the snapshot id is unchanged - documented as finding 1 in `P1_A0_EXECUTION_REPORT.md` |

## Local app state touched by this smoke

To get a clean history, the pre-existing `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite`
and its `-wal`/`-shm` sidecars were renamed with a `.p0-smoke-history` suffix rather than deleted.
Two honest consequences: SQLite resolves sidecars by name, so the renamed trio no longer opens as a
database (the 4,096-byte main file alone shows an empty schema), and the P0-era local build history is
therefore not readable from that path any more. Nothing in the repository depends on it - P0's
evidence is the committed goldens and this machine's regenerable app-data - but the state is recorded
here so nobody has to guess why those files exist. The new database written by this smoke is intact.
