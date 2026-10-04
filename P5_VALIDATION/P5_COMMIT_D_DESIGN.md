---
title: "P5 Commit D design record"
doc_id: "FS-P5-COMMITD-DESIGN"
product: "FirmwareSight"
version: "1.0"
status: "DESIGN_RECORDED_IMPLEMENTATION_IN_PROGRESS"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-04"
---

# P5 Commit D — design record (continuation prompt §5)

Written before the remaining production edits, and after the ones the §2 list told this round not to
redo. Every number here was produced by a command run on this machine on 2026-10-04 and is quoted with
the command, because the previous round's lesson was that a remembered figure is not evidence.

Start state: `HEAD` = `origin/main` = `2cdfced48594ec15f368ab15e4eaef8dbb26dc98`, branch `main`,
P5 = `IN_PROGRESS`, baseline version `0.6.0`, direct counts before Commit D = Rust 813 / UI 201, local
gate 16/16.

## 0. Authority correction (first action, §1)

**Result: `AUTHORITY_PROMPT_BYTES_UNAVAILABLE` for the canonical v1.1, and the round's authority is the
file that actually arrived.**

| role | file | bytes | LF lines | sha256 |
| --- | --- | --- | --- | --- |
| active authority, this round | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitD_Continue_v1.2.txt` | 31,588 | 1,472 | `600d70a358778280b667d1c423b7309079f3a5b575ad0c4b919b75de8c274033` |
| superseded historical prompt | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitD_Diagnostics_Recovery_v1.0.txt` | 35,119 | 1,355 | `5ec32b8b3d88dd512a9fb4b6aeab56bcf0f5f40f12559c8b671801acd8a8105f` |
| §1's canonical v1.1 | *not on this machine* | 27,834 | 1,014 | `f38ee5f2c1d640058f5ed84e5ef00f78577bd6c0d68536aca5e9ff4c7e8ab045` |

- `find` over `C:/Users/16429` (maxdepth 5, excluding `AppData`) and over `D:/study` for `*CommitD*`
  returned only the v1.0 and v1.2 files. **The v1.1 bytes were never delivered here, so no byte-exact
  archive of v1.1 is claimed anywhere in this repository, and none was reconstructed from memory.**
- The v1.2 archive was verified against its source: `cmp` reports the two files identical, and
  `stat`/`wc -l`/`sha256sum` on both sides printed the same three numbers.
- Both prompt files are pure LF with a final newline (measured: CRLF count 0), which is the fact ADR-0028
  is about — the archive's identity is its bytes, not its rendered text.
- v1.0 stays in the repository as history. Nothing in the repo ever claimed v1.0 was *this* round's
  authority: grep over `*.md`/`*.yaml` for either prompt filename returned zero hits before this file was
  written, so there was no false statement to retract, only a claim to avoid making.
- The owner chose "archive v1.2 as this round's authority" when asked, after the v1.1 gap was reported.

## 1. Storage integrity API (§8)

`crates/firmwaresight-storage/src/health.rs:55` — `Database::integrity_check(&self) -> Result<StoreHealth,
StorageError>` with `StoreHealth::{Healthy, Unhealthy { summary }}`.

- The only statement is `PRAGMA integrity_check`, prepared inside the storage crate. No IPC command takes
  SQL, a PRAGMA name, or a database path (`AGENTS.md` 7).
- `Healthy` requires exactly one row equal to `ok` and no early stop.
- A damaged store answers problem rows and *then* fails mid-iteration. That is measured, not assumed: on
  the fixture in `tests/integrity_and_backup.rs` the uncapped report names **73 problem lines** across 3
  rows before `rows.next()` returns `Err`. A failing iteration after problems have been named is therefore
  evidence of ill health (`cut_short`), and a failure before any row is a typed `StorageError`.
- Bounded both ways: at most 3 problems (`health.rs:35`), 120 characters each (`:38`), joined by `"; "`,
  with `only the first problems are listed` (`:41`) whenever the list was cut — including when a caller
  hands over a longer list than the cap, which is asserted directly by
  `a_list_longer_than_the_cap_is_cut_and_says_so`.
- Path-free: `sanitize()` reduces any token containing `/` or `\` to its final component. Real integrity
  output names trees, pages and index marks and carries no separator, so that reduction is proven against
  constructed text (`a_problem_that_carries_a_directory_is_reduced_to_the_name_it_ends_with`) rather than
  left as an untested claim.
- Error detail text never copies rusqlite's message, which can quote the file it failed on: `describe()`
  emits `sqlite_error_code()` only.
- No repair, no reset, no rewrite: `an_opened_store_with_rows_is_healthy_and_the_check_writes_nothing`
  compares the store's bytes before and after the check.

New codes, both inside the storage family §10 asks for: `ERR-STORAGE-4011` (backup), `ERR-STORAGE-4012`
(integrity check), in `crates/firmwaresight-storage/src/error.rs:63-64`.

## 2. Backup mechanism (§6, §11) and why it is WAL-safe

`crates/firmwaresight-storage/src/backup.rs:65` `snapshot_to()` uses SQLite's **online backup API**
(`rusqlite::Connection::backup("main", &staging, None)`).

- Measured first, per §6: the existing `rusqlite 0.40.2 / features = ["bundled"]` was asked whether it
  could already do this, and the answer is that the `backup` feature of the *same* package does. The
  feature is declared `backup = []` in `rusqlite-0.40.2/Cargo.toml:75` — an empty dependency list. So
  enabling it adds no package, and that was proven by toggling rather than only by reading: with
  `features = ["bundled", "backup"]` the graph is storage **22** / workspace **246** unique packages, and
  with `features = ["bundled"]` — the pre-Commit-D state, measured with the manifest temporarily reverted
  and then restored byte-exact, sha256 `85cd1858…` — it is the same **22** / **246**. The command for both
  numbers is `cargo tree --workspace -e normal --prefix none | sed 's/ .*//' | grep -v '^$' | sort -u |
  wc -l`, run per package and for the workspace. `rusqlite v0.40.2 backup,bundled,cache,default,…` sits
  above `libsqlite3-sys v0.38.2`, which was already in the graph, and **`Cargo.lock` is unmodified**
  (`git diff --stat -- Cargo.lock` empty). No new dependency family, so §24's STOP was not owed.
- A naive `std::fs::copy` of the `.sqlite` file is not acceptable and is not what ships: the store runs in
  WAL, so the committed state is the file *plus* whatever frames the log still holds. The backup API reads
  through the live connection and produces a consistent image.
- `VACUUM INTO` was tested and rejected. It needs no feature, but it binds its filename as SQL `TEXT`, so
  a non-UTF-8 store path either fails or names a different file; the probe also showed that pointing it at
  an occupied path yields an engine message containing an **absolute path**, which the shell would then
  have to redact. The backup API takes `AsRef<Path>` and cannot leak that way.
- Destination-collision behaviour was tested (the other thing §6 asked about): the API refuses an existing
  staging file, so `snapshot_to()` removes any `<name>.staging-<pid hex>` it left behind on an earlier
  crash before writing.
- Platform behaviour: the mechanism is the same C API on Windows, macOS and Linux, and the tests are
  written to be deterministic on all three — the blocked-backup fixture occupies the destination with a
  **directory**, which fails identically everywhere, instead of relying on `chmod`, which does not mean the
  same thing on Windows.

## 3. Backup ordering (§10, §13)

`Database::open → establish(conn, Some(path)) → migrate_from(store)`
(`crates/firmwaresight-storage/src/db.rs:62`, `:82`, `:127`); the snapshot is taken at `db.rs:141`, after
`ensure_bookkeeping` and `current_version` and **before** `apply(MIGRATIONS, current)`.

`Database::migrate()` — the explicit re-migration path — passes no store, so it cannot write a snapshot:
only an open owes the file a copy.

The ordering is proved by content, not by a timestamp: the snapshot of a v4 store **lacks**
`sections.file_offset_unknown`, which 0005 adds. A copy taken after the ALTERs would carry it.

## 4. Naming and retention (§12)

- Name: `<store stem>.pre-migration-v<from>-to-v<to>.sqlite`, beside the store
  (`backup.rs:33`, `pre_migration_backup_path`, pure and unit-tested). For the shipped store that is
  `firmwaresight-p0.pre-migration-v4-to-v5.sqlite`. No username, no project name, no artifact name: the
  stem is the product's own constant.
- Retention rule, stated exactly: **one verified snapshot per schema transition.** Staging → verify →
  `rename`, so the previously verified copy is replaced only by a copy that has already opened, reported
  `MAX(version) == from` and answered `integrity_check` with `ok`. A repeat of the same transition leaves
  one file (`a_repeated_transition_keeps_one_snapshot_and_replaces_it_only_with_a_written_one`); a store at
  the target version backs nothing up on reopen
  (`a_store_already_at_this_version_backs_nothing_up_on_reopen`); a fresh store backs nothing up
  (`a_fresh_store_writes_no_migration_backup`); an in-memory store has no path to write to
  (`backup.rs:48 backup_plan`, arm-tested).
- Bounded in aggregate: the transitions this product can perform are finite because the migration list is
  finite, and Diagnostics reports at most 8 names (`MAX_BACKUP_NAMES`). No timestamp, no growth per launch.
- Nothing is deleted by this code path. A snapshot a user keeps is theirs; the product only ever replaces
  the one it owns by name.

## 5. Backup-failure semantics (§14)

Fail-closed. `snapshot_to()` returns `StorageError::Backup` (`ERR-STORAGE-4011`) and `migrate_from`
propagates it before any migration runs, so the store keeps its schema, its rows and its shape.

`a_snapshot_that_cannot_be_written_stops_the_upgrade_and_leaves_the_store_at_v4` asserts: the code, schema
still 4, no `sections.file_offset_unknown`, all seven counted tables unchanged, no `*staging*` debris, the
occupied destination still a directory rather than a half-written snapshot, and no `/` or `\` anywhere in
the message.

One claim in that test was corrected by measurement rather than kept because it sounded right. The store is
**not** byte-for-byte unchanged: opening a rollback-journal file switches it to WAL, and that writes the
header. Measured on this fixture — four bytes, offsets 18 and 19 (format read/write version 1 → 2) and 27
and 95 (change counter and its version-valid-for mirror, 38 → 39). The test now asserts the stronger and
true thing: every differing offset is below 100, so no page of stored data moved.

## 6. Migration matrix (§15)

Ran fresh→5, v1→5, v2→5, v3→5, v4→5 and v5-reopen, file-backed, each checking schema result, row
preservation, backup result and integrity:
`every_older_file_backed_schema_is_snapshotted_at_the_version_it_was_found` (v1-v4, one snapshot each,
named for the version *found*), plus the fresh and reopen arms, plus the pre-existing matrix tests in
`release_records.rs` and `unknown_reasons.rs`. In-memory migration tests require no filesystem backup and
do not take one.

## 7. Diagnostics allowlist (§16) and forbidden data (§17)

Seven closed structs, no map, no `Value`, no path type. Fields and where each comes from:
| section | fields | source |
| --- | --- | --- |
| product | name, version, binary, identifier | `tauri::AppHandle` config + package info (`support.rs:43`) |
| product | logical store file name | the `Session` that opened it (`lib.rs:298`), never its directory |
| runtime | OS family, architecture, platform | `std::env::consts` |
| runtime | OS version | `not_reported` — no trusted source without a new platform dependency (§21) |
| runtime | Tauri version | `tauri::VERSION` |
| runtime | WebView version | `tauri::webview_version()`, read only in a command; `not_reported` on `Err` |
| storage | store schema, supported schema, journal mode | the file's own bookkeeping (`counts.rs`) |
| storage | health, bounded summary, or a code | `integrity_check()` |
| storage | counts of projects, builds, Gate runs, accepted reviews, release records | `counts.rs`, each `null` if unreadable |
| storage | pre-migration snapshot **names** | the store's parent directory, names only, ≤ 8 |
| git | available, version | `git --version` through `project::git::run_bounded`, fixed argv, no repository, no remote, no path |
| support | input cohort, install channel | a stated constant; `tauri::utils::platform::bundle_type()` |
| policy | config schema version, `require_clean_git` + its caveat, `require_release_notes` | the loaded `GatePolicy` — never `LoadedProject.path` or `.root` |
| — | recent error codes | a fixed-capacity (8) in-memory list of stable codes only |

Absent by construction: artifact and MAP source paths, project root, database path, bundle destination,
home, username, Git remote or repository URL, firmware bytes, MAP contents, Release Notes body, symbol
names, environment dumps, credentials, tokens, stack traces, SQL, raw SQLite rows, any app/session dump.
`LoadedProject` holds `path` and `root`, and no DTO field can receive them.

No automatic upload, no telemetry, no network. The store file is not renamed (§20): `STORE_FILE_NAME` is
now the single constant, and `run`'s setup uses it instead of a second literal.

Recent error codes were judged proportionate, so §21's escape hatch was not taken: the ring is fed by the
five converters that turn a typed domain error into an envelope (`envelope_from_artifact`,
`envelope_from_storage`, `envelope_from_project`, `envelope_from_diff`, `envelope_from_bundle`). Codes
only — a message can quote the path it failed on.

**Two counts in this section were wrong as first written, and both are now measured.** The allowlist is
**41 keys**, not the "31-field" figure this round's governance sentences carried: `ALLOWED_KEYS: [&str; 41]`
in `apps/desktop/src-tauri/tests/diagnostics.rs:47` names every key the payload may hold,
`no_field_of_the_payload_can_hold_a_path` fails if the set moves, and the eight `Diagnostics*` structs in
`ipc.rs` declare 41 public fields between them — counted two ways, and they agree. "Seven closed structs"
above counted the seven sub-sections and forgot the root `DiagnosticsDto` that holds `schema`, `generatedAt`
and `recentErrorCodes`, so it is **eight**. The 31 was not a stale value that drifted; it was written into
`3400981`'s documentation without being checked against the constant that already existed in the same commit,
and six sentences repeated it. The exported file from §27 carries **9 top-level keys**, which is a different
count of a different thing and is what that row claims.

## 8. Export surface and DTO (§19, §20)

Two commands, both argument-free, registered beside the other 28 (`lib.rs:958`, now 30):

- `diagnostics::collect_diagnostics` → `DiagnosticsDto` for the Help/About section.
- `diagnostics::export_diagnostics` → `ExportOutcomeDto`; opens the native Save dialog from Rust through
  the existing `compare::save_path` / `confirm_replacement` / `write_export` (temp file + rename,
  `cancelled` and `kept-existing` as outcomes rather than errors), suggested name
  `firmwaresight-diagnostics.json`, no timestamp in the name, no `.txt`, no ZIP.

`DiagnosticsDto` derives ts-rs and emits 8 generated `.ts` files; optional counts are
`number | null` (`ts(type)` attrs), so "the store would not answer" is not printable as zero.
`generatedAt` is labelled observation metadata only — nothing hashes any field of this payload.

Placement: the existing Help/About page gains a Diagnostics section with the five §20 minimum visible
facts (version, store schema, store health, Git availability, and the logical store name under the label
"Local FirmwareSight data store") plus the export action. No fifth top-level navigation verb, no Settings
page, no support portal.

Three of the UI decisions were judgement calls rather than consequences, so they are recorded here:

- **Health is prose with colour as an aid, not one of the five state badges.** `PASS`/`BLOCK` on this page
  would read as a release verdict the Gate was never asked to make — the same reasoning that already keeps
  the document list in neutral grey (`.documentFlag` in `Help.module.css`) — and DESIGN.md 9 allows colour
  only as an aid beside a word. An unrecognised health value renders `unknown`, never `healthy`.
- **The section owns its own call and its own state.** `get_app_identity` and `collect_diagnostics` are two
  independent reads whose failures stay inside their own section, so neither blanks the other and neither is
  a proxy for the other. The Version row appears in both on purpose: it is the payload's copy of the same
  runtime answer, and the section says so instead of leaving a reader to work out which of two numbers is
  newer.
- **The export reports what the dialog did, in the Compare export's vocabulary.** `written`, `cancelled` and
  `kept-existing` are outcomes, and declining to overwrite is stated as the decision it is. The button is
  taken away while the native dialog is open, because a second click would open a second dialog.

Four sentences became stale the moment Diagnostics existed, and were corrected rather than left to imply a
capability that would then have to be built: "Where it sits on this machine is a Diagnostics question"
(`Help.tsx`), the same claim in `getAppIdentity`'s doc comment (`bridge.ts`), the same claim in
`AppIdentityDto`'s Rust doc comment and therefore its generated binding, and the matching paragraph in
`support.rs`. Diagnostics deliberately does **not** answer where the store sits, so all four now say that no
surface reports the directory. `DOCUMENTS` lost its `not in this build yet` entry for Diagnostics, and the
test that asserted that string asserts the shipped wording instead.

## 9. Startup boundary (§22)

Measured starting state: `run()`'s `setup` mapped a `Session::open` failure to a
`format!("{err} (code {code})")` string, which Tauri surfaced as `Error::Setup`, and the call chain ended in
`.expect("error while running the FirmwareSight desktop application")` — a panic for four anticipated storage
conditions: backup failure, migration failure, unsupported schema, storage open failure. Two of the four also
reach `setup` through the folder that holds the store (`app_data_dir()`, `create_dir_all`).

**What was built.** `apps/desktop/src-tauri/src/startup.rs`: one typed `StartupFailure` (stable code, one
sentence about the person's data, an optional filtered reason, one next step) with `from_storage` covering the
four storage conditions plus the two folder conditions, seven unit tests, and `abort()` — the hook prints the
sentence and leaves with `startup::EXIT_CODE`, so a launcher can tell a refusal (1) from a crash (101).

That last shape was forced by a measurement, not chosen. The first version returned the failure from `setup`
and matched `tauri::Error::Setup` on the value from `Builder::run`, reading `run` as
`self.build(context)?.run(...)` (`tauri-2.12.0/src/app.rs:2617`) and concluding a `setup` error returns before
the event loop starts. The installed binary disproved it: it exited **101** with a framework `panic!`, because
the hook runs inside the loop's `Ready` arm and a returned `Err` is panicked by Tauri itself
(`tauri-2.12.0/src/app.rs:1443-1445`). `Error::Setup` was unreachable from the match written for it, and the
first line of what reached the user named a build-machine path. `run()` is back to
`.expect("error while running the FirmwareSight desktop application")` for everything that is *not* a store
refusal, so an unknown runtime fault still shows itself as one. The full transcript, including the exit code
measured after the change, is `P5_DIAGNOSTICS_RECOVERY_REPORT.md` §3.

Text discipline, measured rather than assumed: `StorageError::Open`'s `Display` does embed the database path,
and the open test proves the redaction bites by taking that error from the real engine (`Database::open` on a
path whose parent provably does not exist), asserting the engine text contains the directory, then asserting
the startup line contains the file name and not the directory. `without_directories` reduces every
whitespace-separated word containing `/` or `\` to its final component and bounds the result at 200
characters, so a Windows path and a POSIX one are both handled by the same rule; the migration arm deliberately
says "that one step was rolled back" rather than "nothing was changed", because earlier steps in the same run
were already committed. Version 0 is the exception that the installed walk exposed: `ensure_bookkeeping()`
reports its own failure as `Migration { version: 0 }` (`crates/firmwaresight-storage/src/db.rs:184-192`), which
is a file that is not a database rather than a step that half-ran, so it gets its own sentence and carries the
engine's filtered phrase — "file is not a database" and "database is locked" ask for different repairs.

**What stopped, per §22.** A window that can explain itself. The store open lives in `setup`, and the session
it produces is what every command needs, so on this path there is nothing for a page to ask; the dialog
plugin's blocking API may not be called from the main thread
(`tauri-plugin-dialog-2.8.0/src/lib.rs:369-372`), and a recovery window — or moving the open out of `setup` so
a window exists first, which changes how `Arc<Session>` is managed — is exactly the architecture §22 says stops
here. It returns to the Architect with one correction to how this document first described it: a window *is*
created, and it is visible. Polled at 20 ms intervals through a failing startup, a titled
`FirmwareSight - Analyze` window exists from t=20 ms and the process is gone by t=360 ms with exit code 1. What
is impossible is a window with something true to say, and in a release build
(`main.rs:2`, `windows_subsystem = "windows"`) the sentence has no console to reach unless a launcher redirects
it. The honest summary of the refusal as a user meets it today is: the application appears, leaves, and leaves
a code in an exit status nobody sees. The typed failure, the exit code and the tests are the part this round can
afford; the visible part is the stop, and the stop is now described as it measures rather than as it was
reasoned.

`envelope_from_storage` (`lib.rs:818`) still passes `StorageError`'s `Display` through unredacted, and this
round does not change it: `Database::open` has exactly one production call site (`lib.rs:286`, reached only
from `setup`), so `Open` — the one variant whose message names a path — cannot arrive through a command. That
is a measured fact about reachability, not an argument that the shape is safe forever; if a second open site is
ever added, that converter needs the same `hidden` treatment `envelope_from_project` already has.

Runtime proof of the exit path ran on the packaged binary against a deliberately broken disposable store, with
the owner's store parked, backed up on a second volume and hash-verified first; nothing was run against the
owner's live store, and it came back byte-identical. `P5_DIAGNOSTICS_RECOVERY_REPORT.md` §3 and §5 carry it,
and §4 carries the mutation proof for the version-0 arm.

## 10. Capabilities and dependencies (§24)

**No new Tauri capability is required or added.** `capabilities/main.json` stays `["core:default"]`; the
dialog plugin remains Rust-side only (`ADR-0025`); CSP is unchanged at `tauri.conf.json:25`, whose
`connect-src` is `'self' ipc: http://ipc.localhost` — no remote origin.
No network, no shell, no generic filesystem, no SQL, no telemetry. If any of these turns out to be needed,
this round stops before adding it.

## 11. Files and tests

Changed or added so far: `crates/firmwaresight-storage/{Cargo.toml,src/db.rs,src/error.rs,src/lib.rs,
src/health.rs,src/backup.rs,src/counts.rs,tests/integrity_and_backup.rs}`;
`apps/desktop/src-tauri/{src/lib.rs,src/ipc.rs,src/support.rs,src/compare.rs,src/bundle.rs,
src/diagnostics.rs,src/startup.rs,tests/diagnostics.rs}`; the UI side `apps/desktop/ui/src/{Help.tsx,
Help.module.css,help.test.tsx,ipc/types.ts,ipc/bridge.ts}` and 8 generated bindings under
`apps/desktop/ui/src/ipc/generated/`. Governance and identity documents changed by §3 and §25:
`09_ADR/ADR-0028-release-identity-bytes-as-evidence.md` (new),
`P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md` (re-statused, not rewritten), `README.md`, `INDEX.md`,
`04_TECH/08_CONFIG_SPEC.md`, `04_TECH/24_BUILD_IDENTITY_EVIDENCE.md`. `.gitattributes` was not touched.

Still to change: `.ai/` governance, `BASELINE.yaml`, `INDEX.md`'s P5 rows, and the evidence pack §30 names.

Added after the design was written, because the installed walk is what asked for them:
`P5_VALIDATION/P5_DIAGNOSTICS_RECOVERY_REPORT.md` (§26's pre-install report, §27 item by item, the startup
proof and the restore), the seventh mutation in §12, and one sentence in
`scripts/verify_package_artifacts.py` that had claimed the payload digest belongs to "the exact file this
installer bundles" — the bundler overwrites 3 bytes of that file to name the bundle type
(`tauri-bundler-2.10.1/src/bundle.rs:41-95`), which the walk measured rather than guessed.

Test counts measured from one `cargo test` run per crate, summed over its own targets: storage crate **131**
(5 unit — the four `health.rs` bounds and the one `backup.rs` plan rule — and 126 integration, of which 13 are
`integrity_and_backup.rs`), desktop crate **222** (96 unit, which now include the 7 `startup.rs` mappings, and
126 integration, of which 8 are `tests/diagnostics.rs`), workspace total **854**, UI **210** in 8 files (25 in
`help.test.tsx`, 9 of those new). The corruption fixtures were additionally repeated 10 times in fresh
processes with 0 failures. `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`, `npx tsc --noEmit` and
`npx eslint .` are clean at this point, and the complete local gate ran **16 of 16** on this tree with the
package group **4 of 4** beside it.

**One of those numbers was a race, and the remote found it.** Run `37200245520` on the Commit D product head
came back **8 of 10** — `macOS Core Smoke` and `Rust (ubuntu-latest)` both red, on the same assertion in the
same test, at `integrity_and_backup.rs:814`:
`a_repeated_transition_keeps_one_snapshot_and_replaces_it_only_with_a_written_one` asserted
`assert_ne!(reread_snapshot, kept_snapshot)` and the two byte strings were equal. The product was not wrong —
the fixture was. `build_a_v4_store` writes fixed data on purpose, so the two v4 stores that test builds differ
only by the `applied_at` default on `schema_migrations`, and that default is
`strftime('%Y-%m-%dT%H:%M:%SZ','now')`: **one-second granularity**. A rebuild inside the same second produces a
byte-identical file, so the assertion was a comparison against the clock. It passed on this Windows host and on
`Rust (windows-latest)` and failed on `macOS Core Smoke` and `Rust (ubuntu-latest)`, which is what a race looks
like: same test, same bytes of source, three machines, two outcomes. Both red jobs named the storage suite, not
a product command, so the repair below is test-only and no production behaviour changed — the counts above are
unchanged by it, because no test function was added.

The repair removes the clock from the claim instead of slowing the test down. `mark_a_v4_store` inserts a
`sections` row with `section_index = 99` whose `name` says which store it is, the test marks the first store
`snapshot-marker-first` and the second `snapshot-marker-second`, and after the retention assertion the test
reads the marker **out of the standing snapshot** and asserts it is the second one, plus
`schema_version_at(&backup) == 4` so the file that won is still the before-state. The `assert_ne!` stays — a
replacement has to change the bytes — but it is no longer the only evidence, and the evidence that replaced it
names a fact rather than a timestamp. `rusqlite::Connection::open`, not `Database::open`, because opening the
snapshot with the product would migrate it. No sleep, no retry, no widened timeout: §29's ban on L23-style
timing work applies to a fix for a timing bug more than anywhere else.

Measured after the repair, on this tree: `cargo test -p firmwaresight-storage` **131 passed, 0 failed**; the
repaired test run **20 times in fresh processes**, 20 passed and 0 failed; `python scripts/check.py` **16 of 16
steps passed**; `python scripts/check.py --only core-smoke` — the group whose macOS job went red — **3 of 3**;
`cargo fmt --all -- --check` clean. The package group was not re-run on this host: the only file that changed
after the **4 of 4** packaging above is a test file, and CI's three package jobs built and verified that same
product source on the failed head and came back green, so packaging is proven against the bytes this repair
leaves standing — and it runs again remotely on the repair head.

## 12. Mutation proofs run (§23)

Each mutation was applied to the production source, the affected suite run, and the file restored from a copy
taken before the edit; the restored SHA-256 equals the pre-mutation value in every case, and
`grep -rn MUTATION` over the Rust and TypeScript sources returns nothing. No mutation is committed.

| # | Mutation | Suite | Measured red |
|---|---|---|---|
| A | `health.rs`: `integrity_check` answers `Healthy` without asking the engine | `integrity_and_backup` | 2 failed — the damaged-page test and the many-problems bound, each reporting `: Healthy` |
| B | `db.rs`: upgrade first, owe the snapshot afterwards | `integrity_and_backup` | 4 failed — v4-before-upgrade, the every-older-schema matrix, the cannot-be-written fail-closed test, the one-snapshot retention test |
| C | `diagnostics.rs`: the payload names the store by its full path | `tests/diagnostics.rs` | 4 failed — `no_field_of_the_payload_can_hold_a_path`, the positive control, the snapshot-naming test, the safe-facts test |
| D | `diagnostics.rs`: the loaded project's name leaks into `inputCohort` | `tests/diagnostics.rs` | 1 failed — the positive control, on the planted project name; the other 6 stayed green |
| E | `startup.rs`: `without_directories` returns the engine's words unchanged | `startup` unit tests | 3 failed — the folder-repeat test, the catch-all wording test, the snapshot-wording test |
| F | `Help.tsx`: render an `unhealthy` store as `healthy` | `help.test.tsx` | 1 failed — `describes a damaged store in words…`; the other 24 stayed green |
| G | `startup.rs`: the version-0 arm's pattern widened to `version: 1` | `startup` unit tests | 1 failed — `a_file_that_is_not_a_database_is_reported_as_nothing_having_run`, at its first assertion, because the corrupt file then falls through to the rolled-back sentence |
| H | `backup.rs`: a snapshot already in place is kept instead of replaced (`snapshot_to` returns `Ok` when the destination exists, after deleting its staging file) | `integrity_and_backup` | 2 failed — the retention test on its `assert_ne!` at `:833`, and the un-writable-destination test at `:741`. With the byte assertion temporarily lifted, the marker read-back failed on its own at `:846` with `left: "snapshot-marker-first"`, `right: "snapshot-marker-second"`, which is the proof that the new assertion carries the claim and is not decoration on the old one |

A–D are §23's four. E, F, G and H are additions this round earned: the startup redaction and the health sentence
are new behaviour in this commit, and a regression that cannot be reddened is not evidence for it. G came out
of the installed-app walk rather than out of imagination — §9 explains why the sentence it protects was wrong
before it was written. H came out of the remote's red job, and it is the one that checks a *test*: the claim
under repair was the new marker read-back, so the mutation had to remove the replacement rather than the
message. D and F are the
two worth reading twice — each reddens exactly one test, which is what a targeted leak and a targeted mistranslation
look like, and each proves that the remaining suite is not the one carrying that claim.

The SHA-256 each mutated file was restored to, recorded so the byte-exact claim is checkable rather than
asserted: `health.rs e314a85e2f2b0685…`, `db.rs 6a65434a0b402a9b…`, `diagnostics.rs a96c172239ed9c49…` (both
after C and after D), `startup.rs e4bb8d583ffd814e…`, `Help.tsx 3ea38cbb193bdf59…`,
`backup.rs af7bac846bf5e1d2…` (after H). For H the test file was edited too, to lift one assertion so the
other could be measured alone; it was copied before the edit and restored from that copy, and
`sha256sum` matched the pre-edit value `f8f6a0ec6c8439ec…` afterwards. The restored `backup.rs` and a
`git diff --stat` that names only the test file are the two facts that say no mutation landed.

## 13. Security and dependency review (§24)

Measured, not assumed:

- **No new dependency.** The `backup` feature A/B in §2: 22 storage / 246 workspace unique packages with the
  feature on, and the same two numbers with it off. `Cargo.lock` is unmodified, `rusqlite-0.40.2/Cargo.toml:75`
  declares `backup = []`, and `libsqlite3-sys v0.38.2` was already in the graph. No dependency was added to
  any crate, and no dev- or build-dependency appeared.
- **Dependency policy passes with documented risks**: `python scripts/check.py --only deny` →
  `advisories ok, bans ok, licenses ok, sources ok`, exit 0. That is the most this section claims — a green
  advisory run over the current database is not a statement that the tree is secure.
- **No capability change.** `apps/desktop/src-tauri/capabilities/` and `tauri.conf.json` (whose CSP keeps
  `connect-src 'self' ipc: http://ipc.localhost`) are untouched, as is `.gitattributes`;
  `git status --short` over those three paths prints nothing. No plugin was added, and `tauri-plugin-dialog`
  stays Rust-side only, so the WebView gains no filesystem, shell, network or SQL surface (`ADR-0025`).
- **Two new commands, both argument-free.** `collect_diagnostics` and `export_diagnostics` accept no
  parameters at all: the UI cannot name a path, a table, a query, a `PRAGMA`, a destination or a filter.
  `envelope_from_storage` and its four siblings are unchanged in shape, and every error that reaches the UI is
  still a closed `ErrorEnvelopeDto`.
- **`unsafe_code` remains forbidden** in both crates that grew code (`lib.rs:32`, `storage/src/lib.rs:12`).
- **The health check cannot repair anything.** The only SQL the new production code issues is
  `PRAGMA integrity_check` (`health.rs:63`, `backup.rs:111`) and `PRAGMA journal_mode` (`counts.rs:79`); every
  `DROP`, `DELETE` and `ALTER` this round wrote lives inside a `#[cfg(test)]` fixture. There is no VACUUM, no
  rebuild, no reset path — which is why the Help screen can honestly say the product will not touch a damaged
  store.
- **A snapshot is written, verified and then moved.** `snapshot_to` stages a hidden file, opens the copy and
  runs the health check on it, and only then renames it into place; each failure removes the staging file and
  returns `ERR-STORAGE-4011` naming the snapshot by file name, never by directory.

Risks this round does not close, stated where they are:

1. A startup refusal has no on-screen surface that can explain itself (§9). The code names the condition, the
   data state and the next step, and exits 1; measured on the packaged binary, a titled window is nevertheless
   mapped from t=20 ms until the process leaves at t=360 ms, so what a double-click actually shows is the
   application appearing and then going away, with the sentence reaching a console only when something
   redirects stderr. Returned to the Architect as the recovery-window question §22 stops.
2. `envelope_from_storage` passes `StorageError`'s `Display` through unfiltered. Today that is not reachable
   with a path in it, because `Database::open` has exactly one production call site and it is in `setup`; it
   becomes a leak the moment a second open site exists. The one-line fix is the `hidden` treatment
   `envelope_from_project` already has, and it is deliberately not smuggled into this commit on a theoretical
   reachability argument.
3. `release.require_clean_git = false` still lets release identity move without `git.clean` blocking it. That
   is now ADR-0028's documented, accepted consequence rather than a surprise, and Diagnostics carries the
   sentence beside the flag.
4. Two allowlisted fields answer `not_reported` on platforms that do not say — OS version, and the WebView
   version when `tauri::webview_version()` declines. They are present so a reader knows they were asked.
