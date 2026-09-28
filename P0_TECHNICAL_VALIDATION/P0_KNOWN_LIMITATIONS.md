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
- **The windowed application has been launched - once, on one machine.** The earlier claim in this
  file ("never launched") is obsolete: the shipping configuration
  (`cargo build --release -p firmwaresight-desktop --features custom-protocol`) was started under the
  authorized smoke and the results, field by field, are in `P0_DESKTOP_SMOKE_REPORT.md`. What that
  launch does not cover: one Windows 10 (19045) host at 100% scaling, one display, one WebView2
  version (153.0.4234.48). No Windows 11, no DPI scaling above 100%, no Ubuntu or macOS window, and
  no window-management behaviour (resize, minimise, restore, close-while-analyzing). The binary in
  that run is 10,400,768 bytes and embeds `index-CGuVwhkk.js` + `index-DtRzKmrE.css`, the asset pair
  the current `dist/` holds.
- **The loading state has never been seen on screen.** Analysis of a 7 KiB fixture completes before
  the browser paints the pending state, so the `aria-live` announcement and the disabled-control path
  are verified by a jsdom test rather than by an eye. `P0_DESKTOP_SMOKE_REPORT.md` records that row as
  NOT OBSERVED, not as a pass.
- **One visual defect is open and unrepaired.** The capability value `Partial` breaks as
  "Parti / al" inside the badge column at this width. It is a layout defect in a P0 screen, and
  closing it means changing either a token-driven column width or the copy - both of which go through
  `P0_DESIGN_CHECKLIST.md`, not through this file.
- **`cargo` will not tell you the embedded frontend is stale.** `tauri-build` registers
  `cargo:rerun-if-changed` for `dist/` only when that directory already exists at build time
  (`tauri-build-2.7.0/src/codegen/context.rs:92-97`), and a missing `dist/` is not an error in the
  default-feature build. Observed on this machine: after a release link made with `dist/` absent,
  rebuilding the UI and re-running the same cargo command finished in 0.62s without recompiling -
  a binary with no frontend, and no diagnostic. Only the `custom-protocol` link fails loudly, inside
  the macro. P0 neither bundles nor launches anything, so nothing shipped is affected; whoever
  packages the app must build the UI before the Rust link.
- Two design-checklist findings stay open, in `P0_DESIGN_CHECKLIST.md`. Capability badges display
  Core's serialized enum words ("supported", "available", "not-provided"), which is API vocabulary
  rather than product copy - and the CLI prints the same strings, so a copy layer in the view is
  precisely where the two surfaces could start to disagree. That is a boundary decision, not a styling
  one. And the eight `1px` borders have no token: `DESIGN.md` 3 does not list border width among the
  value classes that must be tokenised while `DESIGN.md` 4 requires hairline separators, so closing it
  properly means a `design-tokens.json` version bump - a change to a frozen design asset that P0 is
  not authorized to make. Two other findings from the same pass (no live region for the asynchronous
  state; a `select` with only hover and focus) were fixed in the source and are covered by a test.
- The UI's two interactive controls are a fixture `select` and one button. There is no keyboard
  shortcut, no command palette and no focus management beyond the browser's own order, because P0's
  screen has nothing to navigate between.
- `style-src 'unsafe-inline'` is present in the CSP because Vite injects inline styles in dev mode.
- The application-data database path is resolved from Tauri's platform directory; it has only been
  exercised on Windows.
- Mobile crate-types (`staticlib`, `cdylib`) and `tauri.conf.json` bundle targets were left out
  rather than declared without a test device.

## Storage

- Schema version 2 covers only what P0 writes. Gate and release tables arrive with their own
  migrations.
- `foreign_keys = ON` and a 5 s busy timeout are asserted at open, but multi-process concurrency
  is untested - nothing in P0 runs two FirmwareSight processes against one database.
- **Migration 0002 exists because version 1 was wrong, not because time passed.** `evidence` had a
  whole-table primary key on `id`, so recording the same field identifier for a second build raised
  `UNIQUE constraint failed: evidence.id` (`ERR-STORAGE-4006`) - the desktop could not analyze two
  artifacts, which contradicts `04_TECH/15` §4's `Build 1─N Evidence`. It was found by the authorized
  window smoke, reproduced by a failing test, and fixed by rebuilding the table on
  `(build_id, id)` (`P0_DESKTOP_SMOKE_REPORT.md`, `P0_STORAGE_REPORT.md`). What is still limited:
  a SQLite primary key cannot be `ALTER`ed, so the upgrade is copy-and-rename rather than in-place -
  correct and cheap at P0 row counts, and it would want a different shape for a large history;
  there is no down-migration, because a schema that cannot be verified in both directions is not
  worth claiming; and the v1-to-v2 path has been run against a real v1 database on Windows only.

## Repository layout

- `apps/desktop/ui/package.json` is the only `package.json` in the tree.
  `05_ENGINEERING/00_REPO_STRUCTURE.md` sketches a root `package.json`, `pnpm-workspace.yaml` and
  `.node-version` alongside it. With one frontend package, `pnpm -r typecheck|test|build` from a root
  adds nothing over running those scripts in the package, and the Node floor is expressed as
  `engines: ">=24.0.0 <25.0.0"` plus CI's `setup-node: "24"` rather than a `.node-version` file. The
  structure doc's own rule - "Frontend is one workspace package at MVP" - is met; the plumbing around
  it is not. It arrives with the second package, when there is something to hoist.

## Verification gaps

- **CI has run four times: two failures, then green twice.** Run `36360310447` (`f9b8ccb`) had four red
  jobs; Run `36378384225` (`ebda52d`) had one; Run `36399805005` (`1cd6309`) concluded `success` with
  **7 of 7 jobs green**, including `Generated output drift` after the provisioning step round 2 added;
  Run `36402637251` (`5e58f77`) repeated it on the architect-reviewed HEAD. Remote CI is a measured fact
  with a `gh` command behind it, and the promotion the bullet above deferred to the architect has since
  been signed: `P0 = PASS` at `v0.6.0`. None of that closes any limitation in this file - a green
  workflow is not a performance measurement, a fuzzer or a participant session.
- **`cargo deny` runs and passes both places it can be run** - locally (0.20.2, exit 0 over `licenses
  bans sources advisories`) and in CI (Run #2's `Dependency policy` job, four categories ok). What it
  still does not prove: it evaluates only the four targets in `[graph] targets`, so a mobile or
  embedded target later reopens the boundary check - `tauri` declares a non-optional `reqwest` for
  Android and iOS, and the filter is what keeps it out of the product's graph.
- **The drift job's provisioning step is copied, not shared.** Run #3 executed the copy and it worked
  (`Setting up libwebkit2gtk-4.1-dev`, then `5/5 steps passed`), so the remote failure is closed. What
  remains as a limitation is the shape of the fix: two jobs carry the same ten-package apt block, so a
  future job that compiles `firmwaresight-desktop` on Linux must remember to copy it too, and a
  package rename in the runner image would break both at once. Extracting a script was rejected for a
  two-job duplication; a third instance is the trigger to revisit it. The step was NEVER LOCALLY
  EXECUTED - this host has no Ubuntu - and Run #3 is the only place it has run.
- **Two advisories cannot be closed without changing the frozen dependency architecture.**
  `RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`,
  unmaintained, host-only) both arrive through the gtk-rs 0.18 line that Tauri 2.12.0 requires;
  `cargo update -p glib --precise 0.20.0` fails against `gtk = "^0.18"`. They are ignored with
  recorded evidence rather than upgraded. **The architect has since accepted both as explicit P0
  transitive risk that does not block promotion**, with five revisit triggers recorded in
  `P0_DEPENDENCY_REPORT.md` - a decision, not a silence, and `unused-ignored-advisory` stays at `warn`
  so an ignore entry that stops matching becomes visible by itself. Replacing Tauri, forking
  dependencies or moving the frozen desktop dependency family would still require an ADR.
- **The Ubuntu system packages and the macOS core smoke have each executed once, successfully.** Run
  #2's `Rust (ubuntu-latest)` passed with the apt block installed, and `macOS Core Smoke` passed with
  87 tests on its first execution. The package list is Tauri 2's documented Linux
  prerequisite set plus `libdbus-1-dev` named explicitly because `libdbus-sys` asks for `dbus-1`;
  `pkg-config` itself is not installed, since the failure log shows it ran and returned 1 while the
  `.pc` files were missing. `macos-core` is `python scripts/check.py --only core-smoke` - the same
  script CI and contributors already run, not a fourth place tests are written - and it is wired to
  run on `main` pushes, not on pull requests, per `05_ENGINEERING/06_CI_CD_BASELINE.md`.
- **Desktop icon bytes are no longer asserted across platforms; their pixels are.** `Pillow`'s PNG
  and ICO encoders emit different bytes for identical RGBA data depending on encoder state, so the
  old byte comparison tested the library rather than the icon - it passed on Windows and failed on
  Linux with nothing wrong on screen. The check now decodes: PNG dimensions and RGBA pixels, and for
  the `.ico` the required size set with per-frame pixels, where a missing frame fails. The consequence
  of choosing semantics over bytes is that `git status` will show the generated icons as unchanged
  even when a rebuild would write different bytes, and a future regression that changes only the
  encoder invocation is not a failure. That is intended; a regression that changes a pixel is.
- **`.gitattributes` is now load-bearing for a hash assertion.** The fixture manifest records the
  SHA-256 of the committed blobs, and Run #1 failed because a Windows checkout smudged a `.ld`
  script to CRLF. The text policy is what makes "blob hash" and "file on disk" the same statement on
  every platform, so a contributor who deletes that file, or a later target that adds a `*.map`
  pattern, breaks a test with no product defect behind it. Verified here by fresh `git clone` into a
  temporary directory on Windows, and by Run #2 checking out the same tree on `ubuntu-latest` and
  `windows-latest` with the hash assertion green in both Rust jobs. What no clone has yet proved is a
  macOS checkout, because the macOS job runs the core smoke rather than the artifact tests.
- **No fuzzing.** `cargo-fuzz` needs a nightly toolchain and a separate crate, which the tool
  policy did not authorize. The no-panic claim rests on regression tests over four malformed
  fixtures plus a truncated-ELF and foreign-MAP case.
- **Fixture regeneration needs a toolchain most contributors lack.** `arm-none-eabi-gcc` is only
  required to rebuild fixtures, never to test them - but the ability to regenerate is therefore
  single-machine in practice.
- **One host, one build.** All timings and the window smoke come from one Windows 10 machine with a
  release build; they are not comparable to CI or to another platform.

## Process honesty

- The V0 prototype track remains deferred and unvalidated (ADR-0020). Nothing in P0 upgrades a V0
  claim, and V0's gate recommendation was not altered.
- Where the baseline was silent, P0 made a decision and wrote it down rather than inventing a
  precedent: `SnapshotId` derivation, the RAM discriminator, `p0-internal` schema naming, and
  TypeScript 6.0.3. Each is in `P0_IMPLEMENTATION_LOG.md`.
- One file was deleted during P0 (`golden/reports/p0-basic-summary.json`), because it carried
  wrong values under a right-looking name and no test read it. It is recoverable from git.

## Final disposition — P0 PASS, v0.6.0 (2026-09-28)

P0 closed `PASS` and `FirmwareSight_Project_Baseline_v0.6.0` was frozen. **Every limitation listed in
this file survives that promotion unchanged**, and the promotion prompt forbids describing them as
resolved: peak RSS is `NOT MEASURED`, fuzzing is `NOT RUN`, Compare / Gate / Bundle are unimplemented,
the desktop evidence is one Windows host and one WebView2 runtime, the two RustSec advisories are
accepted rather than fixed, and V0's external sessions remain `0 / 8`.
