---
title: "P1-A0 Correctness Closure Desktop Smoke Report"
doc_id: "FS-P1A0-005"
product: "FirmwareSight"
version: "0.1.0"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P1-A0 Correctness Closure Desktop Smoke Report

Runs prompt §18: twenty-one steps against the shipping release binary, on a clean validation
database, proving that the two defects the original smoke found are closed where a user actually
sees them - in the window *and* in SQLite. The first smoke proved the intake works; this one proves
what it stores is the same truth it displays.

Status: **PASS - 21 of 21 observed.**

## Environment and configuration

| | |
| --- | --- |
| Host | Windows 10 x64, build 10.0.19045, console session 1 |
| Binary | `target/release/firmwaresight-desktop.exe`, 11,009,536 bytes, SHA-256 `f50441feb0b2f2b23702a2553a9e3c09ad54d494b9c5d4dc1f6404bc255c5c0f`, written 2026-09-29 09:06:16 |
| Build command | `cargo build --release --features custom-protocol -p firmwaresight-desktop` - `Finished 'release' profile [optimized] target(s) in 4m 50s` |
| Frontend origin | `http://tauri.localhost/` (read from the WebView's own accessibility node), i.e. embedded assets, not the Vite dev server |
| App window | id 335530, pid 12444, title `FirmwareSight - Analyze` |
| Native dialogs | id 3277970 `Choose the firmware artifact to analyze`, id 204654 `Choose the linker MAP for this artifact`, id 860000 a second artifact dialog - all three the same pid as the app |
| Validation DB | `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite`, created empty by this run (0 builds before step 2) |

The first release build of this round was `cargo build --release` **without** `--features
custom-protocol`. Its window opened and rendered Edge's `ERR_CONNECTION_REFUSED` page, because
tauri's codegen embeds nothing when the feature is off (documented in
`apps/desktop/src-tauri/Cargo.toml`). That build produced no analysis and wrote nothing; it was
discarded and the run above was performed against the rebuilt binary.

### How the user's own database was kept out of it

`APPDATA` and `LOCALAPPDATA` overrides were tried first and **do not work**: Tauri resolves the
known folder through the shell, not the environment, so the process still opened the real
`com.firmwaresight.desktop` path. That instance was closed before it analyzed anything. The run then
proceeded as follows, and every step is reversible:

1. `firmwaresight-p0.sqlite` copied to scratch and verified byte-identical by SHA-256
   (`ec845c8a840453af077f611ddda21d4a4d65ab11e4c651d67673e514bc08479e`).
2. The live trio renamed to `firmwaresight-p0.cc-pre.sqlite` plus correctly named `-wal` / `-shm`
   sidecars, so the moved-aside set stayed openable - the mistake in `P1_A0_DESKTOP_SMOKE_REPORT.md`
   was renaming the sidecars so SQLite could no longer resolve them, and it was not repeated.
3. The app created a fresh empty database; all 21 steps ran against it.
4. The validation database was checkpointed, copied to scratch as `validation-db-final.sqlite`, and
   removed. The original trio was renamed back.
5. Restored file re-verified: SHA-256 identical to step 1, `PRAGMA integrity_check` = `ok`, and the
   same 1 build / 1 artifact / 10 evidence / 17 sections / 32 symbols under `local-desktop` at schema
   version 2 as before the round started.

Three `*.p0-smoke-history*` files from the previous round are still in that directory. They predate
this round, were not created by it, and were left untouched.

## Inputs

Selected through the real dialogs from `%TEMP%\p1a0cc-smoke\inputs\`:

| File | SHA-256 |
| --- | --- |
| `customer-app.elf` | `c6d0feed0a1e4153d80592519f0bfe1a11be67d756f2ecd811799b4f7def4e62` |
| `customer-app.map` | `c4182c5bcc69b155e6a9ccc641990a6d6d8649ebb6b57e3ea63a3fd6f0ce2a61` |
| `not-a-firmware.elf` | `e66ed6d8214f6dbcef606a4a907fc998d1a18b3a894249b3a7009955d7642365` |

The ELF and MAP are a genuine GNU ld pair, so the MAP really does match the artifact; they are
copies of the committed fixtures under user-style names, which is what makes the snapshot ids below
comparable with the goldens.

## The 21 steps

| # | Step | Observed |
| --- | --- | --- |
| 1 | Launch shipping binary | window 335530, pid 12444, `FirmwareSight - Analyze`, WebView on `http://tauri.localhost/` |
| 2 | Choose real ELF | selection row read `customer-app.elf` |
| 3 | Native dialog appears | separate Win32 window 3277970, same pid, title set by Rust |
| 4 | Analyze without MAP | report rendered: `Arm 32-bit little`, `object-elf/0.40`, entry `0x08000039`, nonvolatile 160, runtime 72 |
| 5 | Record snapshot X | `snap-c6d0feed0a1e4153d80592519f0bfe1a11be67d756f2ecd811799b4f7def4e62-p0-normalize-1` |
| 6 | DB: one build, one artifact, weaker evidence | `builds` = 1 `COMPLETE`; `artifacts` = 1 row `#0` kind `Elf`; `memory_footprints` = `layout_source none`, `weakest_basis ElfAddressAndFlags`, `admissible_hard_block 0`; `evidence` = 10, of which **0** rows `source_type = 'MapFile'` and **0** rows whose locator contains `map:` |
| 7 | Attach matching MAP | dialog 204654 opened; header then read `MAP: customer-app.map`, `Add MAP` became `Replace MAP` / `Remove MAP`; the ELF-only report stayed visible, because attaching is not analyzing |
| 8 | Analyze | `Admissible for a hard limit`, `Weakest basis map-memory-configuration+elf-load; region evidence came from the linker.`, `Layout source map`, evidence 11 recorded / 11 observed |
| 9 | Record snapshot Y | `snap-c6d0feed0a1e4153d80592519f0bfe1a11be67d756f2ecd811799b4f7def4e62-p0-normalize-1-c4182c5bcc69b155e6a9ccc641990a6d6d8649ebb6b57e3ea63a3fd6f0ce2a61` |
| 10 | X != Y | differ by exactly the MAP's SHA-256 suffix, the component `SnapshotId::compose` already defined |
| 11 | DB: two builds | `builds` = 2, both `COMPLETE`, both under `local-desktop` / `Local analyses` |
| 12 | MAP build has two artifacts | `#0` `Elf` `c6d0feed0a1e` 10,960 bytes `Arm`/`Bits32`/`Little` `object-elf/0.40`; `#1` `Map` `c4182c5bcc69` 3,149 bytes `Unknown`/`Unknown`/`Unknown` `map-adapter/gnu_ld`, entry recorded as a reason rather than a value |
| 13 | MAP build stores stronger evidence | `layout_source map`, `admissible_hard_block 1`, 11 evidence rows against the weaker build's 10; all 5 `MapFile` rows belong to this build |
| 14 | Analyze the same pair again | same Y on screen |
| 15 | DB remains two builds | `builds` = 2, artifacts still (1, 2) - the repeat deduped |
| 16 | Remove MAP | header back to `MAP: Not provided`, `Add MAP` restored; the MAP-backed report stayed visible |
| 17 | Analyze again | `Not admissible for a hard limit`, `Layout source none`, evidence 10 recorded |
| 18 | Snapshot returns to X | on screen, byte-identical to step 5 |
| 19 | DB remains two builds | `builds` = 2; no third row was created by going back |
| 20 | BuildSummary returns ELF primary facts | the fixed statement (`JOIN artifacts a ON a.build_id = b.id AND a.id = ?2` with `?2 = {build}#0`) returns `c6d0feed0a1e…`, 10,960 bytes, `Arm`/`Bits32`/`Little` for the **MAP** build. See the caveat below |
| 21 | No path leak, then close cleanly | every visible string is a bare file name: `customer-app.elf`, `customer-app.map`, and in the error path `unsupported format: Binary at not-a-firmware.elf`. No drive letter, no directory, anywhere in the window. `Alt+F4` ended the process (`tasklist` reports no match) and SQLite checkpointed normally |

Two extra observations the run produced. A failed analysis stored nothing: after the malformed
selection the database still held 2 builds and 3 artifacts. And the last-good rule held through a
changed input set, which the original suite had only tested against a failed analysis: the panel read
`Previous analysis of customer-app.elf. It is not an analysis of not-a-firmware.elf.` while the
surviving report stayed the ELF-only one.

## Cross-check against the CLI and the goldens

The two snapshot ids recorded in the window are exactly the two committed goldens now carry:

| | Golden | Window |
| --- | --- | --- |
| ELF only | `golden/cli/p0-basic-analyze.json` formula `snap-<elf>-p0-normalize-1` | X matches, same formula, same ELF hash |
| ELF + MAP | `golden/cli/p0-dual-region-analyze.json` `…-p0-normalize-1-c4182c5b…`, `counts.artifacts` 2 | Y matches character for character |

## Caveats, stated rather than smoothed over

* **Step 20's determinism is not proven by this run.** The old unqualified join also returned the ELF
  row here, because the `#0` row happened to be visited first. That is the hazard's shape: it is
  invisible until row order changes. The proof lives in
  `crates/firmwaresight-storage/tests/map_companion_persistence.rs`, in
  `the_summary_still_names_the_primary_elf_when_the_rows_are_not_in_insertion_order`, which moves the
  `#0` row to the highest rowid in its own scratch database and asserts the summary still names the
  ELF. It fails against the old query and passes against the fixed one.
* `artifacts.path` holds the absolute host path for both rows, e.g.
  `%LOCALAPPDATA%\Temp\p1a0cc-smoke\inputs\customer-app.map`, written here in env-var form so the
  account name stays out of the repository. That is P0's existing
  local-only column and the same treatment the ELF row already got; it is not part of the
  deterministic payload, the IPC payload or the portable schema, and step 20 above is the observation
  that it stays out of the window.
* The MAP artifact row carries no evidence of its own. The MAP's observed facts are the build's
  `ev-map-*` rows, so repeating them per artifact would count one claim twice.
* One host, one WebView runtime, one Win32 common-dialog backend. The Linux and macOS picker paths
  remain unexercised, as does anything about peak RSS (`NOT MEASURED`) or fuzzing (`NOT RUN`).
