---
title: "P0 Desktop Real-Window Smoke Report"
doc_id: "FS-P0-019"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Desktop Real-Window Smoke Report

Requirement: open the shipping Desktop window in a real display session, load the embedded frontend
without a dev server, run a P0 fixture analysis, compare what the page shows against what the CLI
prints for the same bytes, and close it. Authorized by the CI closure remediation prompt, §7.

Status: **PASS — after one source fix this smoke found.** The first pass through the window ended on
a real failure (`ERR-STORAGE-4006`) that 102 Rust tests and 19 UI tests could not reach, because no
test had ever put two builds into one database. See
[Defect found by this smoke](#defect-found-by-this-smoke).

## Environment and configuration

| | |
| --- | --- |
| Host | Windows 10 x64, build 10.0.19045, session 1 (console), display scaling 100% |
| Frontend build | `python scripts/check.py --only frontend` → 5/5 steps, `dist/index.html` + `assets/index-CGuVwhkk.js` (231.19 kB) + `assets/index-DtRzKmrE.css` |
| Binary | `cargo run --release -p firmwaresight-desktop --features custom-protocol` |
| Build result | `Finished 'release' profile [optimized] target(s) in 6m 32s`, then 2m 51s after the fix |
| Binary size | 10,398,208 bytes before the fix, 10,400,768 after it |
| Embedded assets | the binary contains the strings `index-CGuVwhkk.js` and `index-DtRzKmrE.css`; the previous build's `index-BdwhVhyx.js` is absent, so the strings track the current `dist/` |
| WebView runtime | Microsoft Edge WebView2 `153.0.4234.48` |
| Document origin observed | `http://tauri.localhost/` (accessibility tree, `RootWebArea` node value) |
| Window | configured 1040×760, observed outer 1056×799 / client 1015×768, title `FirmwareSight - P0 Technical Summary` |
| Database | `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite` — a real user-profile path, not a temp file |

`tauri.conf.json` declares `build.devUrl = http://localhost:5173`, so the release binary carries that
string as inert configuration. It is not a dependency: nothing was listening on 5173 during either
launch (`netstat -ano | grep ":5173"` → no match), and the page still loaded from
`http://tauri.localhost/`. A dev-mode build without `custom-protocol` would have shown a blank or
refused window in the same conditions.

## The fifteen confirmations

Each row states what was looked at, not what should have been there.

| # | Item | Observed |
| --- | --- | --- |
| 1 | Window opens | `list_windows` returned `firmwaresight-desktop.exe` / `FirmwareSight - P0 Technical Summary`, foreground, not minimized, twice |
| 2 | Embedded frontend loads | heading, subhead, fixture control and Analyze button all rendered from the binary |
| 3 | No Vite dev server needed | no listener on 5173; document origin `http://tauri.localhost/` |
| 4 | Page not blank | full text content captured in every screenshot |
| 5 | Styles/tokens loaded | accent `#2563EB` Analyze button, hairline separators, mono numerals, token spacing — `style-src 'self' 'unsafe-inline'` satisfied, so the CSS file itself came from `'self'` |
| 6 | Fixture selector visible | `select` showing `P0 dual region (ELF + GNU ld MAP)`, and `P0 basic (ELF only, no MAP)` after a keyboard change |
| 7 | Analyze clickable | real mouse click at window-relative (403, 196) accepted; `foregroundStatus: matched` |
| 8 | Busy/loading state | **NOT OBSERVED IN THE WINDOW.** Local analysis of a 7-11 kB fixture finishes inside one frame, so the `role="status"` region was never capturable by hand. It is covered by the UI test that defers the promise and asserts the live region, the disabled `select` and the disabled button. Stated as not-observed rather than passed |
| 9 | Analysis completes | dual-region on the first pass; both fixtures on the second pass after the fix |
| 10 | Typed error state visible when triggered | triggered by a genuine defect, not a simulation: `Analysis failed / database write failed: UNIQUE constraint failed: evidence.id / ERR-STORAGE-4006 / Retry the import… / op-29e0-18d9586f40c2c500` |
| 11 | Facts cross-checked | SHA-256, size, architecture, endianness, entry point, nonvolatile footprint, runtime RAM footprint, load evidence, layout source, dual-accounted sections, excluded metadata, all seven capabilities, section count, symbol count, evidence ladder — table below |
| 12 | Same Core facts as the CLI | every field matched `fwsight analyze --json` for the same file, including the two unattributed section reasons quoted verbatim |
| 13 | Unknown glyph recognisable | `Git — ⚪ unknown` renders as a hollow dashed ring in neutral grey beside a filled green circle for PASS, an amber triangle for REVIEW and a grey dash for N/A. Size is `1em` of `--fs-font-size-body` = **13px**; this UI never renders the glyph at 12px, because the 12px token is `metadata` and carries no icon. At 100% scaling on a physical panel the ring is distinguishable from the solid circle |
| 14 | No CSP break or asset failure | page styled, icon in the title bar drawn from `icons/icon.ico`, no missing-asset symptom; CSP is `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self' ipc: http://ipc.localhost; object-src 'none'; base-uri 'self'; frame-ancestors 'none'` |
| 15 | No crash after close | window closed by its own title-bar button; `cargo run` returned exit code 0 on both launches; no output on stdout/stderr beyond cargo's own lines |

## Field-by-field against the CLI

Reference: `fwsight analyze --json fixtures/elf/<fixture>/firmware.elf` (plus `--map` for
dual-region), compared with the window's rendered rows.

| Field | P0 dual region | P0 basic |
| --- | --- | --- |
| SHA-256 | `c6d0feed…4f7def4e62` = | `1c0ae94e…421ba20557f` = |
| Size | 10,960 bytes = | 7,288 bytes = |
| Format | Arm 32-bit little = | Arm 32-bit little = |
| Entry | `0x08000039` = | `0x00008001` = |
| Build ID | `-` + "no .note.gnu.build-id section is present" = | same reason string = |
| Nonvolatile / load image | ✓ Exact 160 bytes = | ✓ Exact 160 bytes = |
| Runtime RAM | ✓ Exact 72 bytes = | ⚠ Partial, 72 bytes, count 2 = |
| Load evidence | ✓ Admissible for a hard limit; weakest basis `map-memory-configuration+elf-load` = | ⚪ Not admissible for a hard limit; weakest basis `elf-address-and-flags` is name- or flag-derived = |
| Layout source | `map`, rule `adr-0021-dual-budget` = | `none`, same rule = |
| Dual-accounted | `elf.section_header[3]`, `elf.section_header[5]`, count 2 = | `elf.section_header[3]`, `elf.section_header[4]`, count 2 = |
| Device metadata excluded | 1,975 bytes = | 2,227 bytes = |
| Capabilities ELF/Sections/Symbols/Debug | supported / available / available / available = | same = |
| Capability MAP | provided = | not-provided = |
| Capability Object attribution | available = | unavailable = |
| Capability Git | ⚪ unknown = | ⚪ unknown = |
| Counts | Sections 17, Symbols 32 = | Sections 19, Symbols 42 = |
| Evidence | ✓ 11 recorded — 11 observed, 0 derived, 0 declared, 0 unknown = | ✓ 10 recorded — 9 observed, 1 derived, 0 declared, 0 unknown = |
| Footer | `snap-c6d0feed…-p0-normalize-1`, `firmwaresight.analyze/p0-internal-1 · p0-internal · normalization p0-normalize-1 · fwsight 0.1.0` = | `snap-1c0ae94e…-p0-normalize-1`, same identity fields = |

`=` means the window's rendered value and the CLI's JSON value are the same string. The UI computes
none of them: `service::summarize` projects the same `AnalyzeResultDto` the CLI renders.

The pair also shows the evidence ladder working on a real display: identical 160/72 byte totals, and
only the *strength* of the claim changes when the MAP is withheld — `Exact` becomes `Partial` with
two named unattributed sections, and `Admissible for a hard limit` becomes `Not admissible` with the
next step to take.

## Defect found by this smoke

**Symptom.** First analysis (dual-region) rendered correctly. Selecting the second fixture and
pressing Analyze produced the typed error panel quoted in row 10. The window stayed usable, the
error was inspectable, and the process did not crash — the reporting path worked exactly as designed.

**Root cause.** `0001_initial.sql` declared `evidence.id TEXT PRIMARY KEY`, making an evidence
identifier unique across the whole database. The analyzer names its facts within one analysis
(`ev-sha256`, `ev-byte-size`, `ev-entry`, …), so every build legitimately records the same
identifiers, and `04_TECH/15` §4 already states `Build 1─N Evidence`. The constraint contradicted the
relation it was meant to implement: one build could be stored, and no second build ever could. The
desktop's own guard (`build_id_for_snapshot` before importing) prevents re-importing *one* snapshot,
which is why the shell's tests, each on a fresh database, never reached it.

**Fix.** `crates/firmwaresight-storage/migrations/0002_evidence_keyed_by_build.sql` rebuilds the
table on `PRIMARY KEY (build_id, id)`; `migrate()` now applies an ordered list of migrations, each in
its own transaction, and `SCHEMA_VERSION` is 2. No column, no value and no other table changed. The
duplicate-within-one-build case that `a_failed_import_leaves_no_build_visible_as_complete` relies on
still violates the key, so the rollback guarantee is intact.

**Tests, written before the fix.** `two_builds_may_record_the_same_evidence_identifier` and
`a_version_one_database_is_upgraded_without_losing_its_evidence` both failed first — the former with
`Write { detail: "UNIQUE constraint failed: evidence.id" }`, the same message the window showed — and
pass now. Storage went from 9 tests to 11. One existing assertion had to be corrected:
`reopening_an_existing_database_is_idempotent` compared the migration-row count against the literal
`1`, which was a count of migrations rather than a statement about duplicates; it now compares against
`SCHEMA_VERSION`, which also catches a missing migration.

**Regression in the same window, on the same database.** The user-profile database was left at
version 1 with one stored build, and it was not deleted. Relaunching the rebuilt binary produced:

```text
schema_migrations: [(1, '0001_initial'), (2, '0002_evidence_keyed_by_build')]
builds:            two rows, both COMPLETE
evidence rows:     21  (11 + 10, per build 11 and 10)
ev-sha256 rows:    2   (one per build)
primary key:       (build_id, id)
indexes:           sqlite_autoindex_evidence_1, idx_evidence_build, idx_evidence_field
```

Both fixtures then analyzed successfully in sequence, and re-analyzing the already-stored fixture
still rendered the same facts without writing a duplicate build.

## Visual defects recorded, not fixed

- The `Partial` state label breaks mid-word (`Parti` / `al`) in the Memory value column when the row
  also carries a count and a long note. It is a layout consequence of the badge column width, and
  fixing it means changing UI metrics, which the remediation prompt excludes. Recorded for the next
  UI pass.
- The loading state is real in code and covered by a test, but not observable by hand at this file
  size (row 8). If a manual demonstration is required, the honest way is a deliberately slow fixture,
  not a claim.

## What this proves, and what it does not

Proves: the shipping binary embeds and serves the built frontend with no dev server; the WebView
renders the token system; the closed fixture-selection IPC boundary returns Core facts that match the
CLI field for field; a storage failure surfaces as an inspectable typed envelope with an operation id
and leaves the app usable; the window opens and closes cleanly; and one real defect in the persistence
layer was found and fixed, with the user's existing database upgraded in place.

Does not prove: performance under real workloads (peak RSS is still `NOT MEASURED`), anything about
macOS or Linux rendering — this was one Windows session — or that the loading state is visible to a
user, which stayed a test-only claim.
