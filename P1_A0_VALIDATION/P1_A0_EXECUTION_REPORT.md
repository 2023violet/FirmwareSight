---
title: "P1-A0 Execution Report"
doc_id: "FS-P1A0-001"
product: "FirmwareSight"
version: "0.1.0"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P1-A0 Execution Report

Bounded Pre-G1 slice: Real Artifact Intake + Analyze Summary. Authorized by
`09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md`, which supersedes only the ADR-0020
clause `P1 Product MVP implementation 只有两者都 PASS 后开始`. The two governing prompts are recorded
in `BASELINE.yaml` under `pre_g1_execution.parent_prompt` and `…_addendum`.

This slice is not P1. `P1` stays `NOT PASS / NOT CLOSED`, G1 is not claimed, V0 participants are
still 0/8, and the baseline stays **0.6.0** - no v0.7.0 was created by this work.

Status: **LOCAL PASS on every §26 command, desktop smoke PASS in a real window.** Nothing here is
`CI PASS`: no push was made, so no remote run covers these commits.

## 1. What was built

| Surface | Change |
| --- | --- |
| `apps/desktop/src-tauri/src/intake.rs` (new) | Four use-case commands: `select_artifact`, `attach_map`, `clear_map`, `analyze_selection`. The two pickers open the native dialog from Rust inside `spawn_blocking`; `on_artifact_picked` / `on_map_picked` are pure and testable without a display. |
| `apps/desktop/src-tauri/src/lib.rs` | `StagedSelection` + `SelectionStore` (session-local, `sel-<pid>-<n>` ids), `stage_artifact` / `stage_map` / `clear_map` / `selection` / `analyze_selection`, shared `store(...)` helper, `envelope_from_artifact(..., hidden: &[&Path])` redaction, `.plugin(tauri_plugin_dialog::init())`, the four new commands in `invoke_handler`. |
| `apps/desktop/src-tauri/src/ipc.rs` | New `SelectionDto { selection_id, file_name, map_file_name?, map_attached }`; `AnalysisSummaryDto.fixture` became `source` (`"fixture"` \| `"artifact"`). |
| `apps/desktop/src-tauri/src/service.rs` | `analyze_paths(artifact, map)` + `project_artifact(file_name, …)`; `display_name(path)` is the only path→text conversion the DTOs use. |
| `apps/desktop/ui/src/App.tsx`, `App.module.css`, `ipc/bridge.ts` | The screen is now "Analyze": choose an artifact, optionally attach a MAP, analyze, and keep the last good report visible when the current candidate fails. |
| `tauri.conf.json` | Window title `FirmwareSight - Analyze` (was `FirmwareSight - P0 Technical Summary`). |

No parser, diff, gate or accounting logic lives in the UI: the screen renders what Core returned,
which is the same `pipeline::analyze` the CLI calls.

## 2. Security boundary (prompt §8)

| Forbidden | How it is prevented | Evidence |
| --- | --- | --- |
| Generic filesystem permission | `capabilities/main.json` permissions stay exactly `["core:default"]`; no `dialog:*` and no `fs:*` permission was added, so the WebView cannot call the plugin's own JS commands | file read after the change; the smoke ran with this file in place |
| `fs` plugin / `shell` plugin | neither is a dependency; only `tauri-plugin-dialog` was added | `Cargo.toml` diff, `Cargo.lock` census below |
| Network | no network plugin, no HTTP client | unchanged from P0 |
| `read_file(path)`-style command | the four commands take a `SelectionId` or nothing at all; none accepts a path from the WebView | `intake.rs` signatures + `no_selection_or_summary_payload_carries_a_path` |
| `get_any_path` | no command returns a path; `SelectionDto` carries file names only, produced by `service::display_name` | `a_selected_artifact_yields_an_opaque_id_and_a_file_name_only`, UI test `never renders the directory the artifact came from` |
| Path leakage through errors | `envelope_from_artifact` redacts every staged path out of `details`; the smoke showed `unsupported format: Binary at not-a-firmware.elf` for a file under `%LOCALAPPDATA%\Temp\…` | `P1_A0_DESKTOP_SMOKE_REPORT.md` step 15 |
| SelectionId as a secret or as identity | ids are session-local (`sel-<pid>-<n>`), never persisted, never in the snapshot identity, and no randomness dependency was added | `selection_ids_are_unique_within_a_session` |

## 3. Dependency admission (prompt §9)

`tauri-plugin-dialog = "2.8.0"`, requested by §8's own design (native dialog, Rust-side, no fs
plugin). Measured, not assumed:

| Question | Answer |
| --- | --- |
| Why is it needed | A native file dialog cannot be opened from the WebView without granting it filesystem or shell rights, which §8 forbids. The official plugin keeps the picker on the Rust side. |
| Stdlib / existing deps | None: `rfd` is not reachable from the current graph, and hand-writing a Win32 dialog would mean first-party `unsafe` in a Core-adjacent crate. |
| License | read from the vendored manifests: `tauri-plugin-dialog 2.8.0`, `tauri-plugin 2.7.0` and `tauri-plugin-fs 2.6.0` are all `Apache-2.0 OR MIT`; the `rfd 0.16.0` picker behind them is `MIT`. All four sit inside the allow-list this workspace already permits, and `cargo deny check licenses bans sources advisories` returned 0 errors. |
| Compatibility | The plugin manifest requires `tauri ^2.12` and `rust-version 1.90`; the workspace pins `tauri = "2.12.0"` and the toolchain is newer than 1.90. |
| Lock delta | `Cargo.lock` +159/−1 lines: **14 package entries added, 0 removed** (`tauri-plugin-dialog`, `tauri-plugin`, `tauri-plugin-fs`, `rfd`, and 10 `windows*` crates). Census comment updated 466 → 480 packages (460 → 474 third-party). |
| Build impact | The shipping release binary is 11,006,464 bytes; the added graph is one more Tauri-line crate plus the OS dialog bindings. Build wall time was not re-measured this round, so no claim is made about it. |
| Capability impact | None observed: `core:default` only, and the smoke's dialogs opened without any new permission. |
| cargo-deny `wrappers` | The anyhow ban grew from `["tauri", "tauri-build", "tauri-utils"]` to include `tauri-plugin`, `tauri-plugin-fs` - the only other upstream parents of anyhow in this graph. The ban keeps its teeth: a first-party dependency on anyhow is still `error[banned]`, which `firmwaresight-desktop` proves by not having one. |
| Linux feature choice | The default `gtk3` feature was kept because Tauri already forces the gtk line on Linux; opting out would swap in `xdg-desktop-portal` and change a platform contract P0 never tested. Recorded as a decision, not a silent default. |

## 4. Storage and identity (prompt §12)

* `P0_PROJECT_ID = "p0-desktop"` is still used for the fixture path, and user-chosen analyses are
  recorded under `LOCAL_PROJECT_ID = "local-desktop"` / `LOCAL_PROJECT_NAME = "Local analyses"`.
  The smoke's database contained exactly one build, under `local-desktop`, and 0 under `p0-desktop`.
* Dedupe preserved for one input set: two successful analyses of the same bytes produced **1**
  `builds` row. What counts as one input set changed in the closure round - an ELF alone and that
  same ELF with its MAP are now two addresses and therefore two builds. See section 9.
* No new migration: `schema_migrations` holds `(1, 0001_initial)` and `(2, 0002_evidence_keyed_by_build)`
  and `SCHEMA_VERSION` is still 2. No schema semantic change was made, and none was needed.
* Raw artifact bytes are not copied into SQLite. `artifacts.path` keeps the location the user chose -
  that column is P0's existing schema and is local-only; it never crosses IPC.

## 5. Tests written before the implementation

TDD order was kept: each suite was added, run red, and only then implemented.

* `apps/desktop/src-tauri/tests/real_artifact_intake.rs` - **18 tests** as written in this round,
  recorded failing to compile
  (no `intake` module, no `SelectionDto`) before `intake.rs` existed. Coverage: opaque id, no path in
  any payload, unknown id / missing file / non-ELF-extension / non-GNU-ld MAP as typed errors, CLI
  equality for a user-chosen file, MAP attach/detach strengthening and reverting, MAP isolation
  between selections, local project identity, dedupe, no new migration, cancel-is-not-an-error,
  bounded payload, unique ids, `Send + Sync` worker-thread use.
* `apps/desktop/ui/src/intake.test.tsx` - **20 tests**, written before the screen rewrite.
* `apps/desktop/ui/src/ipc/bridge.test.ts` - **8 tests** over the four new commands.
* `apps/desktop/ui/src/App.test.tsx` was removed with `git rm`: every case it owned either moved to
  `intake.test.tsx` or became unreachable once the fixture selector left the screen.

Totals from this tree at the end of the original P1-A0 round: **123 Rust tests, 0 failed**
(`cargo test --workspace`), **31 UI tests in 3 files, 0 failed**. The closure round added 19 Rust
tests and left the UI count where it was - see section 9.

## 6. Findings worth the architect's attention

1. **Re-analyzing the same bytes with a MAP did not update the persisted evidence.** *Closed by the
   correctness-closure round, section 9. What follows is what was observed then.* The snapshot
   id is composed from the artifact hash plus normalization version, and the pipeline sealed exactly
   one artifact into the snapshot (`p0_acceptance.rs` asserts `artifacts().len() == 1`), so the MAP
   never entered the identity. `Session::store` then skips the import when that id already exists.
   In the smoke the UI correctly reported 11 evidence items with `layout_source = map`, while the
   database kept the 10 rows from the earlier ELF-only import. This was P0's identity rule left
   half-implemented, newly reachable through user-chosen files, not a P1-A0 regression - and §12 of
   the intake prompt forbade changing snapshot semantics or adding a migration in that round, so it
   was reported rather than fixed. The architect then authorized closing it inside P1-A0.
2. **`evidence_summary.from_map` was a locator kind, not "the user supplied a MAP".** *Closed by the
   correctness-closure round, section 9. What follows is what was observed then.* It was true for
   both fixtures because the pipeline labelled ELF-derived dual charges with `map` provenance, and
   both frozen goldens carried that label. The intake round characterized the behavior instead of
   redefining a P0 field
   (`from_map_reports_a_locator_kind_not_a_supplied_map_file`, now removed with the field) and
   asserted MAP effects through the unambiguous signals: `capabilities.map`,
   `admissibleForHardBlock`, `weakestEvidenceBasis`.
3. **A real defect the window found and the tests did not.** With `text.disabled` and
   `accent.disabled` both `#98A2B3`, the disabled Analyze label was invisible on its own disabled
   fill, and the same token on `bg.subtle` measures 2.32:1 for the secondary buttons. Both
   `.primary:disabled` and `.control:disabled` now pair `text.secondary` (5.22:1 on that surface) with
   a `border-default` outline. The frontend and the release binary were rebuilt and **all seventeen
   smoke steps were re-run against the rebuilt binary**, which is the one this commit ships. No new
   design token was needed - see `P1_A0_DESIGN_CHECKLIST.md`.

## 7. Validation (prompt §26) - all run on this machine, 2026-09-29

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --workspace` | exit 0, 123 passed / 0 failed |
| `corepack pnpm install --frozen-lockfile` / `typecheck` / `lint` / `test` / `build` | 5/5 exit 0, 31 UI tests passed |
| `python scripts/check.py --only drift` | 5/5 PASS (design tokens, desktop icons, ipc bindings, ipc bindings unchanged, goldens unchanged) |
| `python scripts/check.py --only deny` | 1/1 PASS - cargo-deny executed for real, not skipped |
| `python scripts/check.py --only core-smoke` | 3/3 PASS |
| `python scripts/check.py` (full) | **14/14 steps passed**, exit 0 |
| `git diff --check` | exit 0 |

## 9. Correctness closure (prompt §4-§17 of the closure round, 2026-09-29)

The two findings above were closed the same day by *FirmwareSight — P1-A0 Evidence Identity &
Persistence Correctness Closure Execution Prompt v1.0 — Architect Reviewed*. Commit
`P1-A0: close MAP identity and evidence persistence gaps`, on top of `2960173`.

### 9.1 MAP becomes a sealed companion artifact

`pipeline::analyze` now adds a second `Artifact` when a GNU ld MAP is supplied
(`crates/firmwaresight-artifact/src/pipeline.rs`), so `seal()` - which has looked for an artifact of
kind `Map` all along - finally finds one and passes its hash to `SnapshotId::compose`.

| | before | after |
| --- | --- | --- |
| ELF only | 1 artifact, `snap-<elf>-p0-normalize-1` | unchanged |
| ELF + MAP | **1** artifact, `snap-<elf>-p0-normalize-1` | 2 artifacts, `snap-<elf>-p0-normalize-1-<map>` |
| `counts.artifacts` | 1 | 1 / 2 |
| `NORMALIZATION_VERSION` | `p0-normalize-1` | `p0-normalize-1` - **not bumped** |

The MAP artifact carries the real MAP bytes hash and size from its own `GuardedInput`,
`ParserId::gnu_ld_map()`, and `Unknown` architecture / bitness / endianness with `entry_point` and
`build_id` recorded as unknown-with-reason, because a linker map holds none of those. No absolute
host path enters any deterministic output; the artifact `path` field keeps P0's existing behavior of
holding the location for the local database only.

### 9.2 Persistence

No migration, `SCHEMA_VERSION` still 2, migration list still `(1, 0001_initial)` and
`(2, 0002_evidence_keyed_by_build)`. ELF-only and ELF+MAP now land as two distinct immutable builds
and each dedupes against its own repeats; removing the MAP returns to the ELF-only build rather than
creating a third. Proven by `crates/firmwaresight-storage/tests/map_companion_persistence.rs` (6
tests) and by the shipping binary in `P1_A0_CORRECTNESS_SMOKE_REPORT.md`.

`BuildSummary` also stopped betting on row order: the join now names the build's index-0 artifact,
through one `artifact_row_id` helper shared with the writer. A storage test reproduces the hazard by
moving the `#0` row to the highest rowid in a scratch database - it fails against the old query and
passes against the new one, which is the only reason to believe a fix that the happy path never
exposed.

### 9.3 Evidence provenance

`build_evidence` no longer writes `SourceType::MapFile` and `+ map:load-address` beside a rule it
chose elsewhere. One function, `basis_provenance`, returns the rule, source type, locator and
evidence class for a `MemoryEvidenceBasis`, so the four cannot disagree:

| basis | source type | locator suffix | class |
| --- | --- | --- | --- |
| `MapRegionAndElfLoad` | `MapFile` (`map`) | `+ map:load-address` | Observed |
| `RegionConfigAndElfLoad` | `MemoryRegionConfig` | `+ region-config:load-address` | Observed |
| `ElfAddressAndFlags` | `ElfProgramHeader` | `+ elf:sh_flags` | Observed |
| `SectionNameHeuristic` | `ElfSectionHeader` | `+ name-heuristic` | Derived + Low |
| `Insufficient` | `RuleEngine` | none | Unknown |

The `elf:sh_flags` suffix is deliberate: on that rung the charge comes from the section role and its
sizes, and the ELF-only fixture has no load address at all
(`unknown_load_address_stays_unknown_rather_than_defaulting_to_zero`), so `+ elf:load-address` would
have replaced one false locator with another. The totals row keeps the aggregate locator
`memory-accounting/totals` and now names its source from the same basis.

### 9.4 `from_map` removed

`EvidenceSummaryDto.from_map` is gone from the Rust DTO, from the regenerated TypeScript and from the
UI test fixtures, and the characterization test that pinned the old behavior was deleted with it. No
replacement boolean was added; `capabilities.map`, `memory.layoutSource` and
`memory.weakestEvidenceBasis` are the signals, and the UI already read those.

### 9.5 Golden changes (prompt §14)

Only two files moved, 4 lines each, and `golden/core/*-memory.json` did not change at all. The
semantic diff was reviewed before anything was written.

| file | field | old | new | why the old value was false | why the new value holds |
| --- | --- | --- | --- | --- | --- |
| `p0-basic-analyze.json` | `evidence[ev-dual-3].sourceType`, `[ev-dual-4].sourceType` | `map` | `elf.program-header` | that run was given no MAP file at all, so no record could come from one | the charges come from `role_charge`, i.e. section role and sizes, on basis `ElfAddressAndFlags` |
| `p0-basic-analyze.json` | `evidence[ev-dual-3].sourceLocator`, `[ev-dual-4].sourceLocator` | `elf.section_header[N] + map:load-address` | `elf.section_header[N] + elf:sh_flags` | it named a MAP record that does not exist in the run | `sh_flags` is the record a reader can actually re-check in that ELF |
| `p0-dual-region-analyze.json` | `counts.artifacts` | 1 | 2 | the MAP was an input and was not counted | `snapshot.artifacts().len()` after the companion artifact is sealed |
| `p0-dual-region-analyze.json` | `snapshotId` | `snap-c6d0feed…-p0-normalize-1` | `…-p0-normalize-1-c4182c5bcc69…` | an ELF+MAP build collided onto the ELF-only address | `SnapshotId::compose`'s existing optional MAP component, now supplied; `c4182c5b…` is the MAP's own SHA-256 |
| `p0-dual-region-analyze.json` | `evidence[ev-memory-*].sourceType` (2 rows) | `elf.program-header` | `map` | under-claimed: it named only the ELF half of a charge whose region table came from the MAP | prompt §12 requires the source to match the actual basis, and this run's weakest basis is `MapRegionAndElfLoad` |

That last row is beyond §14's four expected categories, so it is called out here rather than left for
a reviewer to find. The ELF-only totals are unchanged, which is why `p0-basic`'s totals do not appear.

**Goldens were corrected in place, not reformatted.** `python scripts/update_goldens.py --confirm`
emits serde declaration order while both committed CLI goldens are key-sorted, so running it rewrote
~350 lines per file. That is unrelated drift, which §14 forbids, so the two files were restored to
HEAD and given only the reviewed leaf changes, then verified against the real binary by
`cargo test -p fwsight --test golden` (5/5) and by `drift/goldens unchanged`. The mismatch between
the sanctioned updater and the committed files is recorded as finding 4 below, not silently repaired.

### 9.6 Tests added by the closure round

* `crates/firmwaresight-artifact/tests/map_companion_identity.rs` - **10 tests**, written first and
  run red: 7 failed against the old pipeline (companion artifact, truthful MAP metadata, id changes
  with MAP, stable for the same pair, no false map source, no fake `map:` locator, rule/class/source
  agreement) and 3 passed as guards. Covers prompt §15 items 1-14.
* `crates/firmwaresight-storage/tests/map_companion_persistence.rs` - **6 tests**, covering §16 A-H,
  including the rowid-shuffle reproducer for §9.
* `apps/desktop/src-tauri/tests/real_artifact_intake.rs` - 18 → **21 tests**: the characterization
  test was removed and four were added (no map provenance and no `fromMap` on the wire, generated
  TypeScript carries no `fromMap`, attach/repeat/detach snapshot-id behaviour, and the two-build
  persistence claim end to end through `Session`).

Workspace total: **142 Rust tests, 0 failed** (123 + 19 added, none removed but the one that blessed
the old `from_map` behavior). UI total unchanged at **31**.

### 9.7 New findings for the architect

1. **`update_goldens.py` no longer reproduces the committed goldens.** Its `pretty()` keeps serde
   declaration order and says so in a comment; both CLI goldens on disk are key-sorted. Anyone
   running the sanctioned updater gets ~700 lines of formatting churn on top of any semantic change.
   One of the two has to be declared authoritative; that is a P0-baseline-adjacent decision, so
   neither was changed here.
2. **`ElfProgramHeader` may be the wrong source type for `ElfAddressAndFlags`.** Prompt §12 says
   "normally ElfProgramHeader", and that is what was implemented, but `elf.rs` parses no program
   headers at all (no `program_header` / `PT_LOAD` call site in the crate); the role and sizes this
   basis charges come from section headers. The label is therefore inherited imprecision, not new
   imprecision - it predates this round in both goldens - and correcting it would move rows §14 does
   not list, so it is reported instead of fixed.
3. **`schemas/release-manifest.schema.json` describes one artifact per build.** Its `build` object
   requires `snapshot_id` and carries a single `artifact_sha256` under `additionalProperties: false`,
   so a snapshot id that now means "ELF + MAP" cannot be described there without `extensions` or a
   schema major. Nothing emits that schema yet and P4 is not authorized, so this is a note for when
   it is, not a change.
4. **`cargo build --release` without `--features custom-protocol` produces a window showing a
   network error.** Already documented in the desktop crate's own comment and used correctly by
   `check.py` and the previous smoke report; this round rediscovered it the hard way and the wasted
   build is recorded in the smoke report so the next reader does not repeat it.

### 9.8 Validation (closure prompt §22) - all run on this machine, 2026-09-29

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --workspace` | exit 0, **142 passed / 0 failed** across 19 result groups |
| `corepack pnpm install --frozen-lockfile` / `typecheck` / `lint` / `test` / `build` | 5/5 exit 0, 31 UI tests passed |
| `python scripts/check.py --only drift` | 5/5 PASS |
| `python scripts/check.py --only deny` | 1/1 PASS, cargo-deny executed for real |
| `python scripts/check.py --only core-smoke` | 3/3 PASS |
| `python scripts/check.py` (full) | **14/14 steps passed**, exit 0 |
| `cargo test -p fwsight --test golden` | 5/5 PASS, i.e. the hand-corrected goldens equal the real binary's output |
| `git diff --check` | exit 0 |
| `git diff --cached --check` | exit 0 for this round's staged set |
| real Desktop correctness smoke | **21/21**, `P1_A0_CORRECTNESS_SMOKE_REPORT.md` |

## 10. Explicitly not built (prompt §24)

No diff view, no gate/decision, no release action, no project switcher, no drag-and-drop, no
multi-file batch, no recent-files list, no persistence of selections, no cross-session selection
restore, no updater, no network, no telemetry, no Dark theme, no new design token beyond the
hairline the addendum authorized, and no P1-A1 work of any kind.
