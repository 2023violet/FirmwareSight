---
title: "P1 Analyze Exit Checklist"
doc_id: "FS-P1-002"
product: "FirmwareSight"
version: "0.1.0"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P1 Analyze Exit Checklist

Every acceptance item from `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` **US-001 Analyze ELF** - the
frozen list the P1 Analyze Acceptance Closure addendum binds P1 `PASS` to - plus every hard boundary in
`.ai/ACTIVE_TASK.md`, with the evidence that closed it.

`LOCAL PASS` means it ran on this machine and produced the quoted result. Nothing in the tables below
says `CI PASS` on local evidence: remote CI for the pushed head is a separate fact, recorded in
`P1_ANALYZE_EXECUTION_REPORT.md` §9 and in the row of the same name below, and added by the successor
commit of the push rather than inside the commit it verifies.

## Acceptance list (US-001)

| Item | Status | Evidence |
| --- | --- | --- |
| Sections visible | LOCAL PASS | Sections tab in the shipped window: `.text A-X 0x080000000 0x080000000 0x00001000 60 bytes 60 bytes ROM`, `.rodata`, `.data`, `.bss`, `.ota`, `.debug_info` - smoke report §5; `sections_cross_ipc_with_addresses_as_hex_and_sizes_as_untouched_bytes` |
| Symbols visible | LOCAL PASS | Symbols tab with ordinal, name, address, size, kind, binding and section ref, including unnamed and sizeless entries rendered as `Unknown` with their reason; `a_symbol_filter_and_sort_travel_as_typed_values` |
| Memory summary present | LOCAL PASS | `Exact 160 bytes` / `Exact 72 bytes` / `Admissible for a hard limit` with the weakest-basis sentence, `map` layout source and `adr-0021-dual-budget` rule, unchanged from P1-A0 and re-read in this smoke |
| Symbol list sortable **and** filterable | LOCAL PASS | Window: filter `task` → 1 row, `sensor` → `sensor_fifo`, sort `Size` ascending then descending with the ordinal tiebreak and unknowns last in both directions. Tests: `a_symbol_filter_matches_a_substring_in_sqlite_rather_than_a_full_name`, `symbol_sizes_sort_known_first_and_unknown_last_in_both_directions`, `symbols_sort_by_address_in_both_directions_over_the_whole_table`, `asks for the other direction when the same column is chosen twice` |
| Search executed in Rust/SQLite, not by shipping every row | LOCAL PASS | The filter is a bound `LIKE` parameter in `query.rs`; the front end never holds a row list to filter. `sends a filter to the shell instead of filtering the rows it already has`, plus the page-bound tests on both layers: `the_detail_commands_apply_the_same_page_bounds_as_the_storage_layer`, `a_limit_above_the_hard_maximum_is_clamped_in_rust_and_cannot_carry_the_whole_table` |
| Artifact hash shown | LOCAL PASS | SHA-256 `c6d0feed0a1e…def4e62` in the window, byte-identical to `sha256sum` output for the same file |
| Unsupported file shows a reason and does not crash | LOCAL PASS | `truncated.elf` → `ERR-PARSE-2002`, "Invalid ELF program header size or alignment", remediation and diagnostics id `op-b80-18d9c0115ce61070`, window alive and usable afterwards (smoke report §7). P1-A0's `ERR-FORMAT-0001` path and `format_truth_comes_from_the_bytes_not_from_an_elf_extension` still pass |
| Debug-info absence does not crash Analyze | LOCAL PASS | `keeps the debug-info row visible and loads details anyway` renders `capabilities.debugInfo: 'unavailable'` and still loads the details; the row is rendered from the Rust DTO, and no second debug-info module exists (addendum §6) |
| Unit switch: default Bytes | LOCAL PASS | `defaults to bytes` asserts the checked radio; the window opened on `10,960 bytes` |
| Unit switch: `1 KiB = 1024 bytes` | LOCAL PASS | `BYTES_PER_KIB = 1024` and `divides by 1024 for KiB, which is the definition the switch promises`; eight independent arithmetic checks in smoke report §4 |
| Unit switch applied to all eight byte quantities | LOCAL PASS | artifact size, both footprints, excluded metadata, section file size, section memory size, contributor size, symbol size - all through one `formatSize(value, unit)`; verified cell by cell in the window |
| Unit switch never applied to addresses, offsets, hashes, counts, ordinals, snapshot ids | LOCAL PASS | `leaves addresses, offsets and counts exactly as they were` compares every `0x…` node before and after; the window kept `Entry 0x080000039`, all addresses and offsets, the hash, the snapshot id, `17 / 32 / 11`, and the inspector's raw `Value` |
| `Unknown` renders as `Unknown`, never `0 B` / `0 KiB` | LOCAL PASS | `formatSize(null)` returns the word; `never rounds a nonzero count down to a zero`, `shows Unknown rather than 0 KiB for a value that has no number` (which asserts no `^0(\.00)? KiB$` anywhere in the Memory or Symbols region), and in the window `.bss` file offset, `.debug_info` virtual address and 26 sizeless symbols all read `Unknown` with a reason |
| Switching units is presentation-only | LOCAL PASS | `re-labels every byte figure and calls no command at all` compares all three command counters; the snapshot id line was unchanged in the window; `drift/goldens unchanged` PASS proves no CLI or golden output moved |

## Hard boundaries

| Boundary | Status | Evidence |
| --- | --- | --- |
| Bounded payloads: default 100, max 500, server-side | LOCAL PASS | `DEFAULT_QUERY_LIMIT = 100`, `MAX_QUERY_LIMIT = 500`, clamped in `clamp_page` before any SQL is built; a 652-row table asked for `limit = 100_000` returned `MAX_QUERY_LIMIT` rows |
| `rows`, total and next offset returned | LOCAL PASS | `Page<T> { rows, total, offset, limit, next_offset }`; `a_first_page_reports_the_next_offset_and_the_second_page_ends_the_table`, `a_details_page_names_the_next_offset_the_reader_would_ask_for`; the window's pager read `Showing 1 to 11 of 11` with both buttons disabled correctly |
| Exactly three use-case commands | LOCAL PASS | `query_sections`, `query_symbols`, `query_evidence` registered in `lib.rs`; `no_general_purpose_query_surface_was_added_to_the_shell` scans four source files for `run_sql`, `read_table`, `query_any`, `get_database`, `execute_shell` → none |
| No raw SQL or `rusqlite` type across IPC; no Tauri type in Core | LOCAL PASS | `details.rs` maps storage rows onto DTOs field by field; `firmwaresight-core` gained no dependency and no `tauri`/`rusqlite` reference; `desktop_parity.rs` still passes (7 tests) |
| ts-rs stays the boundary, bindings committed | LOCAL PASS | 14 new `ipc::export_bindings_*` tests and 14 new `.ts` files under `ipc/generated`; over the 25 files there, `sha256sum -- * \| sha256sum` is `7fcc246a5a3b26aa…` before and after `cargo test -p firmwaresight-desktop`, and `git status --porcelain` on that directory is empty; `drift/ipc bindings` PASS |
| No schema migration | LOCAL PASS | `SCHEMA_VERSION: i64 = 2` (`db.rs:14`); `migrations/` still `0001`, `0002` only; that directory is clean in `git status`. The two reason-less nullable columns are **reported** (execution report §6.1), not migrated |
| No new dependency | LOCAL PASS | `git diff apps/desktop/src-tauri/Cargo.toml` empty; `Cargo.lock` unmodified; `deny/cargo-deny` PASS with `advisories ok, bans ok, licenses ok, sources ok` |
| No new design token | LOCAL PASS | `assets/design-tokens.json` unmodified at `0.2.1`; `drift/design tokens` PASS; `Details.module.css` has zero numeric literals with units and consumes 23 existing tokens |
| Details bound to the last-good snapshot id | LOCAL PASS | After `truncated.elf` failed, the detail area kept answering for `ota-image.elf` and the Sections tab still returned rows (smoke report §7); `stays bound to the last good snapshot when a later analysis fails` asserts every detail request carried the old snapshot id |
| A detail-query error never wipes the summary | LOCAL PASS | `reports inside the details area and leaves the summary readable` - the alert renders and the artifact hash is still on screen |
| No invented data | LOCAL PASS | `sections_without_evidence_come_back_unknown_with_their_reason_instead_of_zero`, `known_or_unknown` names a `NULL`-without-reason as "no reason was recorded" rather than guessing, addresses stay `0x%08x`, and the ordinal column carries `title="A row position in this build, not an identity"` |
| Out of scope respected | LOCAL PASS | No Compare, Gate, Bundle, History, SBOM, CI, Keil/IAR, wizard, treemap, chart or virtualization library, no settings page, no pricing, no recruitment, no cloud, no accounts, no telemetry. `offers no way into a stage this build does not have` still passes |
| UI rules (AGENTS.md 11, DESIGN.md) | LOCAL PASS | `P1_ANALYZE_DESIGN_CHECKLIST.md`, with the greps run rather than asserted |

## Engineering gates

| Gate | Status | Evidence |
| --- | --- | --- |
| Rust fmt / clippy / test | LOCAL PASS | `python scripts/check.py` steps `rust/fmt`, `rust/clippy`, `rust/test` → PASS; `cargo test --workspace` → **184 passed / 0 failed / 0 ignored** |
| Frontend typecheck / lint / test / build | LOCAL PASS | `frontend/typecheck`, `frontend/lint`, `frontend/test`, `frontend/build` → PASS; `corepack pnpm test` → **58 passed** with no stderr |
| Drift: tokens, icons, bindings, goldens | LOCAL PASS | 5/5, including `drift/ipc bindings unchanged` and `drift/goldens unchanged` |
| cargo-deny | LOCAL PASS | `deny/cargo-deny` PASS; installed locally, so this is an execution and not the SKIPPED path |
| Whole gate | LOCAL PASS | `python scripts/check.py` → **14/14 steps passed** |
| Real desktop smoke | LOCAL PASS | `P1_ANALYZE_DETAILS_SMOKE_REPORT.md`, against `82de3bbadebac5f8…`, the binary this commit ships |
| Baseline stays v0.6.0 | LOCAL PASS | `product.baseline_version` is `0.6.0` at both `872ad7e` and `HEAD`; of the 15 `BASELINE.yaml` key paths this round changed, none is a version field; `p1_execution.no_new_baseline: true`; no `v0.7.0` tag |
| V0 untouched | LOCAL PASS | still `0 / 8`, `NON_BLOCKING_USER_FEEDBACK_TRACK`; no participant, transcript, quote, timing or willingness-to-pay figure appears anywhere in this pack |
| P2 not implemented | LOCAL PASS | no Compare surface exists in the tree or on screen |
| Remote CI, pushed head | **CI PASS** | Run #13 `36556735551` on `e63afaf` - `completed`, `success`, 7 of 7 jobs, read with `gh run view 36556735551 --repo 2023violet/FirmwareSight`. Added by the successor commit of `e63afaf`; the two intermediate commits and `872ad7e` had no run of their own, because the gate is per push - see execution report §9 |

## Not measured, not run

| Item | State |
| --- | --- |
| CI on the three commits that were not heads | `NOT RUN` - `872ad7e`, `f649afd` and `ae7759a` have no run of their own, because the gate fires per push and all four commits left together. Only the pushed head's run exists, and it is in the table above; nothing here claims a per-commit CI history |
| Peak RSS | `NOT MEASURED`, unchanged from the state this task started against |
| Fuzzing | `NOT RUN`, unchanged |
| macOS / Linux desktop smoke | `NOT RUN` here; only the Windows window was driven on this machine |
| Keyboard-only traversal of the shipped binary | `NOT RUN` in the window; roles and labels are asserted in jsdom, which is not the same evidence |
| The one dead `Apply filter` click | `UNRESOLVED` - see smoke report §6. Not claimed as fixed |

## Verdict

```text
P1                 = PASS / COMPLETE   (capabilities 1-9 of Analyze, US-001 acceptance list green)
P1_ANALYZE_DETAILS = COMPLETE
G1                 = PASS (basis P0 PASS, per ADR-0026; unchanged by this slice)
P2 Compare         = NEXT_AUTHORIZABLE_STAGE - not implemented here
P3 Gate / P4 Bundle= NOT_STARTED
Baseline           = v0.6.0, unchanged
```

`STOP AFTER P1. DO NOT IMPLEMENT P2. NO PRICING. NO COMMERCIAL VALIDATION. NO CLOUD. NO ACCOUNTS.`
