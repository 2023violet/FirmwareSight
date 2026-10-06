---
title: "P5 Diagnostics and Recovery Report"
doc_id: "FS-P5-DIAGNOSTICS-RECOVERY"
product: "FirmwareSight"
version: "0.6.0"
status: "VALIDATED"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 Commit D — diagnostics, integrity, backup and the startup boundary, run against the installed package

**What this document proves.** Prompt §27's focused installed-app validation ran on this host against the
packaged installer — not `cargo run`, not Vite, not a `target/release` executable — and every line below is
a transcript of what the operating system, the store and the application did. It also proves the one
behaviour Commit D could not prove in a unit test: what the application does when it cannot read its own
store at startup.

**What it does not prove.** It is not a P5 verdict, and it says no `P5 PASS`, `BETA`, `RC` or `GA`. It does
not re-run the Post-G2 end-to-end suite (§27 says it need not). It does not exercise a real
migration-with-backup on an installed binary: the installed walk opened a store that was created fresh at
schema 5, so the backup path was proven in tests and in the migration matrix, not here, and §6 below names
that limit rather than folding it into the summary.

Two defects came out of this walk, and both were fixed before the product head commit. A walk that finds
nothing on its first run against a packaged binary is usually a walk that did not look.

## 1. The state before anything was installed (§26's required report)

Recorded before the installer ran, from the commands named in each row. Nothing in this table is a
recollection: the digests are `sha256sum` output over the files that were there.

| §26 item | Measured |
| --- | --- |
| Owner store present | `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite` — 155,648 bytes, `d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468`, last written 2026-09-30 |
| WAL | `firmwaresight-p0.sqlite-wal` — 0 bytes, `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (the digest of an empty file) |
| SHM | `firmwaresight-p0.sqlite-shm` — 32,768 bytes, `fd4c9fda9cd3f9ae7c962b0ddf37232294d55580e1aa165aa06129b8549389eb` |
| Siblings left alone | four named smoke stores (`*.g2-smoke`, `*.p0-smoke-history`, `*.p2-smoke`, `*.p4-smoke`) with their own `-wal`/`-shm`, and two `postg2-e2e-*` directories. None is the live store; none was opened, moved or deleted |
| Process state | `tasklist /FI "IMAGENAME eq firmwaresight-desktop.exe"` → no matching process |
| Installer | `target/dist-package\FirmwareSight-0.6.0-windows-x86_64-nsis.exe` — 3,887,033 bytes, `2fbbb1b57d63485c54096d8e6ecbbe573f58bd30bf6458fe6fbeb4f9b1c9b32b`; `sha256sum -c SHA256SUMS.txt` → `OK`; byte-identical (`cmp`) to `target\release\bundle\nsis\FirmwareSight_0.6.0_x64-setup.exe`; payload digest `f0bb4dd1a4aa87ff6a68cd24f401791ec19741ebf8b285b6fc2a98eafee25b9f` |
| Install state | the application was **not** installed: `%LOCALAPPDATA%\FirmwareSight` existed and held 0 entries, `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall` held no FirmwareSight entry, and Start Menu had no FirmwareSight folder |
| Planned park path | `%TEMP%\FirmwareSight-P5-CommitD-20261004\PARKED_OWNER_STORE\` |
| Independent backup | `D:\tmp\FirmwareSight-P5-CommitD-20261004\INDEPENDENT_BACKUP\` — a different volume from the store's C:, `cmp`-verified against the original before the original moved |

The owner's store was then parked by **in-place rename** inside its own directory (`.parked-d14`), which
never destroys bytes and is reversible with one command, rather than by a cross-volume move. §26's sequence
ran as written: hash → park → independent backup and hash → disposable store → test → close → restore →
hash compare.

## 2. Section 27, item by item

| # | Item | What happened | How it was checked |
| --- | --- | --- | --- |
| 1 | install / launch package | NSIS ran with `/S`; `%LOCALAPPDATA%\FirmwareSight\firmwaresight-desktop.exe` (15,350,272 bytes) and `uninstall.exe` appeared; the app was started from `Start Menu\Programs\FirmwareSight\FirmwareSight.lnk`, and `MainWindowTitle` reported `FirmwareSight - Analyze` | directory listing, `tasklist`, window enumeration |
| 2 | Help/About | the rail's fifth verb opened Help; About read out Product `FirmwareSight`, Version `0.6.0`, Executable `FirmwareSight`, identifier `com.firmwaresight.desktop`, Running on `windows-x86_64`, Store schema `v5`, History store `firmwaresight-p0.sqlite` | screenshot transcript |
| 3 | Diagnostics visible | the Diagnostics section rendered below About, with its own heading and five rows | screenshot transcript |
| 4 | DB schema correct | `v5` on screen, and the store it came from reports `schema_migrations` = 1 `0001_initial` … 5 `0005_unknown_reasons`, all applied `2026-10-04T10:49:57Z` | screen, then `sqlite3`-equivalent read-only query (`file:…?mode=ro`) |
| 5 | DB health correct | `healthy`, in the same words on screen and in the file, and the engine agrees: `PRAGMA integrity_check` → `ok` | screen, exported file, read-only query |
| 6 | Git availability correct | `available`, with `git version 2.55.0.windows.4` in the exported file and nothing about a remote | screen, exported file |
| 7 | logical store filename only | `firmwaresight-p0.sqlite`. No folder appears on screen or in the file, and the section says so: "The store is named by file, not by folder." | screen, exported file, and §3's separator count |
| 8 | export through the native dialog | the button opened a Windows save dialog titled `Export the diagnostics file` with filter `FirmwareSight json export (*.json)` and default name `firmwaresight-diagnostics.json`; the dialog is a separate top-level window of the same PID, which is what "Rust-side, not WebView-side" looks like from outside | window enumeration by PID, dialog screenshot |
| 9 | exported file parsed independently | Python's `json.load` read it with no product code in the way: 9 top-level keys, exactly the allowlist | `scripts/privacy_check.py` in the evidence root |
| 10 | safe fields correct | `osVersion` is `not_reported` rather than a guess; `tauriVersion` `2.12.0`; `webviewVersion` `154.0.4258.53`; `journalMode` `wal`; five bounded counts; `backupFiles` `[]`; `policy` `null`; `recentErrorCodes` `[]` | the parsed file |
| 11 | host / project / DB paths absent | the file contains **zero** `/` and **zero** `\` characters — not "none that we looked for", but none of either separator anywhere in 1,122 bytes, so no word in it can be a path. Also absent: `Users`, the account name, the repository directory, `AppData`, any drive letter | regex sweep over the raw bytes |
| 12 | close / reopen | the close button ended the process (`tasklist` → no instances); relaunching from the same shortcut produced a fresh window whose title read `FirmwareSight - Analyze`, and the page again said nothing had been analyzed in this session | `tasklist`, window title, screenshot |
| 13 | Diagnostics still works | after the reopen all five rows rendered again, and a second export 10 minutes later differs from the first in exactly two sections: `generatedAt` and `store` | two files, compared field by field |
| 14 | minimal Analyze smoke | `fixtures\elf\p0-basic\firmware.elf` chosen through the native open dialog; the store gained `projects` 1, `builds` 1, `artifacts` 1, `evidence` 10, and the diagnostics counts moved from all-zero to `projects: 1, builds: 1` | read-only query, plus the second export |
| 15 | close app | closed by its title-bar button; 0 instances afterwards | `tasklist` |
| 16 | restore owner DB | the disposable store was moved aside and the parked originals renamed back | §5 |
| 17 | owner hashes byte-exact | all three digests equal the §1 values, and the store reads back `integrity_check = ok` with 1 project, 2 builds, 3 artifacts and 34 sections — the data that was there before the walk | `sha256sum`, `cmp` against the D: backup, read-only query |

`ORIGINAL_DB_RESTORED = YES`
`ORIGINAL_DB_SHA_MATCH = YES`

The two exports are the same length, 1,122 bytes, because the counts that moved went from one digit to one
digit. Equality of length is not the claim; the field-by-field comparison above is.

## 3. The startup boundary, proven on the packaged binary — and the two defects it found

Every other Commit D behaviour has a test that runs it. This one did not: `startup::StartupFailure` had
seven unit tests, but the code that *invokes* it is the `setup` hook of a Tauri application, and no test in
this repository starts a Tauri event loop. §26's flow is the only way to reach it, so it was reached with a
deliberately broken **disposable** store — 8,192 bytes of random data at the canonical file name, with the
owner's store parked and hash-verified first, and no connection to any live data.

**Defect 1: the exit path was a panic, not the typed refusal.** The first run exited **101**, and stderr
carried:

```text
thread 'main' panicked at C:\...\tauri-2.12.0\src\app.rs:1444:11:
Failed to setup app: error encountered during setup hook: FirmwareSight could not finish upgrading
firmwaresight-p0.sqlite to schema version 0. … Code ERR-STORAGE-4003. …
```

The sentence was right; the plumbing was not. `lib.rs` had matched `tauri::Error::Setup` on the value
returned from `Builder::run`, on the reading that `run` is `self.build(context)?.run(…)`
(`tauri-2.12.0/src/app.rs:2617`) so a `setup` failure returns before the loop starts. That reading was
wrong: the hook runs inside the loop's `Ready` arm, and a returned `Err` is turned into a `panic!` by the
framework itself (`tauri-2.12.0/src/app.rs:1443-1445`). `Error::Setup` was therefore unreachable from the
match it was written for, and the refusal arrived as a framework panic whose first line names a
build-machine path — the exact shape this round's privacy discipline spends its effort keeping out of
user-visible text.

The fix moves the departure into the hook: `StartupFailure::abort()` prints the sentence and exits with
`startup::EXIT_CODE`, and `run()` goes back to `.expect(…)` for everything that is not a store refusal, so
an unknown runtime fault still shows itself as one. Measured on the rebuilt package:

```text
$ echo $?
1
FirmwareSight could not start: FirmwareSight could not read firmwaresight-p0.sqlite as its own data store,
so no schema step ran and nothing was changed. Code ERR-STORAGE-4003. Reason: file is not a database.
Start FirmwareSight again. If it fails the same way, report this code; no firmware file is needed.
```

No panic, no `.cargo` path, no account name — checked by searching the captured stderr for `panic`,
`cargo`, `registry` and the account name: 0 matches. One other line does arrive
(`ERROR:ui\gfx\win\window_impl.cc:172] Failed to unregister class Chrome_WidgetWin_0`), emitted by the
WebView2 loader during teardown; it is not this application's text and carries no path or name.

**Defect 2: "upgrading … to schema version 0" described a step that never ran.** `ensure_bookkeeping()`
reports its failure as `StorageError::Migration { version: 0 }`
(`crates/firmwaresight-storage/src/db.rs:184-192`), and the migration sentence was written for a step that
failed partway, so on a file that is not a database at all it claimed an upgrade had begun and been rolled
back. Nothing had been applied, so nothing was rolled back. `from_storage` now answers version 0 in its own
words — *"could not read … as its own data store, so no schema step ran and nothing was changed"* — and
carries the engine's phrase, filtered, because "file is not a database" and "database is locked" ask a
person for different repairs. `a_file_that_is_not_a_database_is_reported_as_nothing_having_run` reaches
that arm through the real engine rather than a constructed value, and the mutation proof is §4.

## 4. Mutation proof for the arm this walk added

`StorageError::Migration { version: 0, source }` → `{ version: 1, source }`. The new test reddens at the
first assertion (`startup.rs:418`), because the corrupt file then falls through to the rolled-back
sentence. Restored byte-exact: `startup.rs` sha256
`dd3432aec13dd5957212c7ec4b8f9fe2b659ebae9738b514befd2b8c2ba4d06e`, `grep -c MUTATION` → 0, and the
module is 7/7 green again. This is Commit D's seventh mutation proof; the other six are in
`P5_COMMIT_D_DESIGN.md` §12.

## 5. The store came back as it was

| File | Before the walk | After the restore | Byte-identical to the D: backup |
| --- | --- | --- | --- |
| `firmwaresight-p0.sqlite` | `d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468` | same | `cmp` → identical |
| `firmwaresight-p0.sqlite-wal` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | same | identical |
| `firmwaresight-p0.sqlite-shm` | `fd4c9fda9cd3f9ae7c962b0ddf37232294d55580e1aa165aa06129b8549389eb` | same | identical |

The parking was done twice — once for the §27 walk, once for the startup proof — and re-hashed at each of
the four boundaries. Nothing in the owner's directory was deleted at any point; the two disposables this
walk created (the fresh store the app wrote, and the 8,192-byte non-database) were removed by name, and the
only files that ever left were the ones the walk renamed back.

The owner's store is at **schema 2**, not 5: `schema_migrations` holds `0001_initial` and
`0002_evidence_keyed_by_build`, and the tables 0003 and 0004 add (`gate_runs`, `release_records`) are not
there. That is why the first real launch of an installed P5 build will run three migrations against it —
and why Commit D's backup is load-bearing rather than decorative. This walk deliberately did not be that
first launch.

## 6. What this walk does not prove, and what it contradicts

- **The installed app never migrated a file.** Its disposable store was created at schema 5, so
  `backupFiles` was `[]` on a store that owed none — the correct behaviour, and the trivial case. The
  v4→v5 load-bearing test, the matrix and the backup-failure refusal are storage-level proofs
  (`P5_COMMIT_D_DESIGN.md` §2, §5, §6), not installed-app proofs.
- **A release build has nowhere to say its sentence.** `src/main.rs:2` sets
  `windows_subsystem = "windows"`, so stderr reaches a file only when a launcher redirects it, as this walk
  did. A user who double-clicks into one of these four refusals sees a window appear and the application
  leave. Measured, not inferred: polling window state at 20 ms intervals through the failure, a titled,
  visible `FirmwareSight - Analyze` window exists from t=20 ms and the process is gone by t=360 ms with
  exit code 1. §22's stop was written as "no window is possible"; what is actually true is that a window
  exists for about a third of a second and cannot be told what to say, because the session it would ask is
  the thing that failed. That is a worse user experience than the design doc described, and it is recorded
  here so the Architect is handed the measured version.
- **The uninstall residue is not stable.** `P5_INSTALL_RECOVERY_REPORT.md` §2 H measured
  `%LOCALAPPDATA%\FirmwareSight` left behind **empty but present**. This round the same
  install → uninstall cycle left no such directory at all. Both measurements stand for their own run; the
  rule has not been worked out, and the user-facing uninstall wording is Commit F's to write against a
  re-measurement rather than against either row.
- **The walk drove the installed app, not a human.** The dialog was operated through the accessibility
  tree and, after that connector dropped mid-walk, through Win32 messages and screen coordinates. The
  product-visible facts each row claims were read off the rendered window; the input mechanism is not part
  of what is under test.
- **Keyboard-only and high-DPI paths were not re-checked here.** They belong to the Post-G2 acceptance
  record, and §27 says the full suite need not run again.

## 7. Where each part of this feeds the closure set

`P5_MIGRATION_RECOVERY_REPORT.md` and `P5_HISTORY_DIAGNOSTICS_REPORT.md` are named by the P5 prompt as
closure documents, and Commit D is what gives them content: §1-§2 and §5 of this file are the installed-app
half of the first, §2's rows 2-11 are the diagnostics half of the second, and the design reasoning — why
the online backup API rather than `VACUUM INTO`, why version 0 gets its own sentence, what the allowlist
excludes and how that is proven — is in `P5_COMMIT_D_DESIGN.md`. Nothing here is a P5 verdict; the state
stays `IN_PROGRESS`.

## 8. §32 read-back — the chain of heads, and the twelve facts it asks for

Read back with `gh api repos/…/actions/runs/<id>` and `…/runs/<id>/jobs`, one call per run, every job named
individually rather than inferred from a green overall state (§31). No run was re-attempted: each of the three
runs below is `run_attempt = 1`, and the one failure stays failed in the record.

| Head | What it is | Run | Attempt | Jobs |
| --- | --- | --- | --- | --- |
| `3400981` | Commit D's product head | `37200245520` | 1 | **8 of 10 — failure.** `macOS Core Smoke` and `Rust (ubuntu-latest)` red on one assertion in a test this head added |
| `bccea88` | the test-only repair of it | `37202016141` | 1 | **10 of 10 — success.** `macOS Core Smoke` and `Rust (ubuntu-latest)` both green on the same test |
| `90aa69d` | the record commit (the audit's §0a, §G and L26) | `37202301591` | 1 | **10 of 10 — success** |
| this commit | §32's read-back successor | — | — | its own run is external evidence; §32 forbids a further commit to record it |

The ten jobs, named as Actions reports them, from the two 10-of-10 runs: `Rust (windows-latest)`,
`Rust (ubuntu-latest)`, `Desktop UI (windows-latest)`, `Desktop UI (ubuntu-latest)`, `Generated output drift`,
`Dependency policy`, `macOS Core Smoke`, `Package Windows`, `Package Ubuntu`, `Package macOS`. The three
package jobs cannot close on a skipped group: the gate returns 1 when CI records a skip, and each of these
uploaded its set (`if-no-files-found: error` is the second net).

**The twelve items §32 names, each with the measurement behind it.**

1. **Product head SHA** — `3400981`, `git rev-parse` of the commit that carries the code, not of the docs
   commit that followed.
2. **Run id and attempt** — `37200245520`, attempt 1, conclusion `failure`; the repair's `37202016141` and the
   record's `37202301591`, both attempt 1, both `success`.
3. **The exact ten jobs** — above, listed from the API rather than from the workflow file.
4. **Rust count** — **854** passed / 0 failed, summed over the 46 test targets of one
   `python scripts/check.py` `rust/test` run on the repaired tree (that log's `frontend/test` line:
   **210 passed (210)** in 8 files).
5. **Local gate** — `check.py` **16 of 16** steps; `--only core-smoke` **3 of 3**, which is the group whose
   macOS job went red; `--only package` **4 of 4** on the packaged build before the repair, and the repair
   touches no source the installer contains.
6. **Backup mechanism** — SQLite's own online backup through `rusqlite`'s `backup` feature
   (`Connection::backup("main", &staging, None)`), staged beside the destination, opened, `integrity_check`ed
   and schema-checked, then `rename`d into place. Rejected: `std::fs::copy`, wrong under WAL, and
   `VACUUM INTO`, whose filename binds as SQL `TEXT` and whose engine error carries an absolute path.
   Adds no package: 22 storage / 246 workspace either way, `Cargo.lock` untouched.
7. **Integrity results** — `integrity_check()` is storage-owned, read-only, repairs nothing; healthy proof,
   unhealthy proof, a many-problems store bounded to ≤ 4 segments and 600 characters with a truncation notice
   and no path separator, and a check that writes nothing to a store with rows. `integrity_and_backup.rs` is
   13 tests; storage is 131; the corruption fixtures were repeated 10 times in fresh processes with 0 failures.
8. **Migration matrix** — fresh → v5 writes no snapshot; v1, v2, v3 and v4 file-backed stores each snapshot
   **at the version they were found in**, each named `…pre-migration-v<from>-to-v5.sqlite`, each preserving its
   rows through the upgrade; a v5 reopen backs nothing up; and a snapshot that cannot be written stops the
   upgrade with the store left at the schema it had.
9. **Diagnostics privacy evidence** — a closed **41-key** allowlist
   (`ALLOWED_KEYS: [&str; 41]`, `apps/desktop/src-tauri/tests/diagnostics.rs:47`, enforced by
   `no_field_of_the_payload_can_hold_a_path`), eight `Diagnostics*` structs, no map and no `Value`;
   positive controls that plant an absolute artifact path, a project name, a symbol name and a Git remote in a
   real session and assert none of them can reach the payload or the exported file;
   and the installed export measured: 1,122 bytes, **zero `/` and zero `\` characters in it**, the store named
   by file and never by folder.
10. **Installed-app focused evidence** — §27's 17 items, row by row in §2, against the packaged NSIS installer
    rather than `cargo run`: dialog driven from Rust, the file parsed by a JSON reader with no product code in
    the way, a minimal Analyze smoke that moved the counts from zero to one project and one build, and the
    close → reopen → re-export cycle.
11. **Owner DB restoration** — parked before the walk with its three digests recorded, copied to a second
    volume, restored at both cycles and byte-identical to §1: `ORIGINAL_DB_RESTORED = YES`,
    `ORIGINAL_DB_SHA_MATCH = YES`, and the store reading `integrity_check = ok` with the same 1 project,
    2 builds, 3 artifacts and 34 sections it held before.
12. **`ADR-0028`** — release identity uses the exact bytes observed on disk; `status: BASELINE`; no release
    identity production code changed in Commit D, the line-ending premise stays a green test
    (`a_notes_file_that_differs_only_in_line_endings_is_a_different_release`), and §32's draft is re-statused
    `EXECUTION_RECORD` rather than rewritten.

**What this successor changed.** Documentation only: `git diff --stat` against `90aa69d` names no file under
`crates/`, `apps/` or `scripts/`. **What the repair changed** was one test file — `bccea88`'s diff touches
`crates/firmwaresight-storage/tests/integrity_and_backup.rs` and documentation, no production source, which is
why the packaged evidence in §1-§5 still describes the binary this commit closes on.

**Two counts this file's own round got wrong, corrected here rather than left standing:** the allowlist is 41
keys, not the "31-field" figure six governance sentences carried from the product head, and the DTO is eight
structs, not seven (§7 of the design doc records both measurements). **And one limitation the read-back
exposed:** `SHA256SUMS` verifies on the host that wrote it and disagrees with a clean checkout for 17 entries;
it is L26 in the audit, deliberately unfixed, because deciding which bytes a baseline checksum file means is
Commit F's call.
