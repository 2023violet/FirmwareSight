---
title: "P0 Exit Checklist"
doc_id: "FS-P0-016"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Exit Checklist

Every item from the prompt's P0 PASS list, with the evidence that closed it. `LOCAL PASS` means
it ran on this machine and produced the quoted result. Nothing here says `CI PASS`.

## Baseline authority minimum (prompt §58)

| Item | Status | Evidence |
| --- | --- | --- |
| real ELF fixture parses repeatably | LOCAL PASS | 19 acceptance tests; `python scripts/gen_p0_fixtures.py --force` reproduced both `firmware.elf` byte-for-byte |
| normalized `BuildSnapshot` stable | LOCAL PASS | `snapshot_id_is_stable_for_the_same_input_bytes`, `the_same_bytes_produce_the_same_snapshot_identically_twice` |
| CLI JSON and Desktop core result agree | LOCAL PASS | `desktop_and_cli_report_the_same_core_facts_for_the_same_bytes` |
| parser does not panic on user input | LOCAL PASS | `malformed_inputs_return_typed_errors_and_never_panic`, `truncated_elf_is_reported_as_invalid_container_not_as_a_missing_file` |
| Tauri UI not blocked by parse work | LOCAL PASS | `analysis_runs_entirely_off_the_calling_thread`; command body is one `spawn_blocking` hop |
| golden fixture passes | LOCAL PASS | 5 CLI golden tests + `drift/goldens unchanged` |

## v0.5 P0 track

| Item | Status | Evidence |
| --- | --- | --- |
| deterministic facts | LOCAL PASS | two runs `cmp` byte-identical; payload sha `96f0645d...` on both; no timestamp, UUID or host path |
| Core/CLI/Desktop parity | LOCAL PASS | 7 parity tests against the committed golden |
| no parser panic | LOCAL PASS | as above |
| memory accounting proven on fixtures | LOCAL PASS | nonvolatile 160 / runtime 72 with `.data` and `.ota` dual-accounted at `MapRegionAndElfLoad`; cross-checked against `readelf` arithmetic |
| typed IPC path proven | LOCAL PASS | 10 generated `.ts` files, drift gate green, 19 UI tests over the same shapes |
| large-file guard benchmark | LOCAL PASS | 100/256/512 MiB measured; 512 MiB + 1 KiB rejected in 7.7 ms with zero hash time |

## Prompt §59, item by item

| # | Item | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Rust workspace builds | LOCAL PASS | `cargo test --workspace`, 6 members |
| 2 | exactly 4 Phase-0 library crates | LOCAL PASS | workspace members list; no fifth library crate (`P0_ARCHITECTURE_CHECK.md` 1) |
| 3 | real ELF fixture parse repeatable | LOCAL PASS | as above |
| 4 | malformed inputs return typed errors, no panic | LOCAL PASS | 4 malformed fixtures + foreign MAP + no-region MAP |
| 5 | SHA-256 stable | LOCAL PASS | `committed_fixtures_match_their_recorded_hashes`, streaming hash |
| 6 | `BuildSnapshot` normalized/stable | LOCAL PASS | 3 core snapshot tests |
| 7 | Evidence model preserved | LOCAL PASS | 10-11 evidence items per fixture with classification, source type, locator, rule |
| 8 | memory accounting golden passes | LOCAL PASS | `memory_accounting_golden_passes_on_both_fixtures` |
| 9 | CLI analyze works | LOCAL PASS | exit 0 on both fixtures, human and `--json` output |
| 10 | CLI deterministic JSON passes | LOCAL PASS | byte comparison across runs |
| 11 | CLI exit-code tests pass | LOCAL PASS | 5 CLI unit tests; measured 0 / 2 / 3 |
| 12 | SQLite migration smoke passes | LOCAL PASS | 9 storage tests, `schema_migrations`, refusal of a newer schema |
| 13 | SQLite snapshot transaction/roundtrip passes | LOCAL PASS | `a_snapshot_survives_a_full_round_trip_with_identical_facts`, `a_failed_import_leaves_no_build_visible_as_complete` |
| 14 | ts-rs generation works | LOCAL PASS | 10 export tests + `drift/ipc bindings unchanged` |
| 15 | TypeScript strict build passes | LOCAL PASS | `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `useUnknownInCatchVariables`; `tsc --noEmit` clean |
| 16 | minimal Tauri summary builds | LOCAL PASS for build and link; see note | `cargo check -p firmwaresight-desktop --all-targets` clean. In the shipping configuration, `cargo build --release -p firmwaresight-desktop --features custom-protocol` produced a 10,398,208-byte `firmwaresight-desktop.exe` whose embedded asset set contains the built `index-BdwhVhyx.js`. **The windowed app was never launched** |
| 17 | blocking parse not on the WebView event loop | LOCAL PASS | off-thread test + `Session: Send + Sync` assertion |
| 18 | Core/CLI/Desktop parity passes | LOCAL PASS | 7 parity tests |
| 19 | 512 MiB guard works before allocation | LOCAL PASS | accepted at 512 MiB, rejected at +1 KiB with no hash/read time |
| 20 | ~100/256/512 MiB workloads recorded | LOCAL PASS | `P0_PERFORMANCE_REPORT.md` measured table |
| 21 | golden tests pass | LOCAL PASS | 5 CLI golden tests |
| 22 | fmt passes | LOCAL PASS | `cargo fmt --all -- --check` |
| 23 | clippy passes | LOCAL PASS | `cargo clippy --workspace --all-targets --all-features -- -D warnings`, silent |
| 24 | cargo test passes | LOCAL PASS | 102 tests, 0 failures |
| 25 | frontend typecheck/build passes | LOCAL PASS | typecheck, lint, 19 tests, `vite build` |
| 26 | no forbidden dependency or boundary violation | LOCAL PASS for first-party code; **cargo-deny NOT RUN** | the `cargo tree --workspace` graph (345 nodes) contains no `reqwest`/`hyper`/`rustls`/`memmap2`/`sqlx`/`gix`/`rayon`/`wgpu`/`axum`/`tonic`; `tokio` and `anyhow` appear only inside Tauri, with zero first-party references. `deny.toml` is written but never executed |
| 27 | P0 reports complete | LOCAL PASS | All 17 documents the prompt's `P0_TECHNICAL_VALIDATION/` list names are present, plus `P0_EXECUTION_PROVENANCE.md` from the takeover and `P0_DESIGN_CHECKLIST.md` required by AGENTS.md 11 - 19 files, each citing a command that was actually run |
| 28 | known limitations explicit | LOCAL PASS | `P0_KNOWN_LIMITATIONS.md` |

## Item 16, stated precisely

`minimal Tauri summary builds` is met in the narrow sense the phrase allows: the desktop crate
compiles and links, and in the configuration that actually ships - `--features custom-protocol`,
which is what `tauri build` passes - `tauri::generate_context!` resolves and embeds the built
frontend (`dist/`, 231.12 kB JS + 7.62 kB CSS). The embedding is not taken on trust: that binary is
10,398,208 bytes and contains the asset name `index-BdwhVhyx.js`, while the same command without the
feature is 10,330,112 bytes and contains no such string, because tauri's dev-mode codegen embeds
nothing (`tauri-2.12.0/build.rs:253`, `tauri-codegen-2.7.0/src/context.rs:178`).

Timings for the release link were taken against a populated `target/`: 3m 11s for the
`custom-protocol` link, 2m 49s without it, and 5m 46s for the first cold link earlier in this
track. They are single-machine numbers and are quoted as such.

It is **not** met in the sense a reader might assume: nobody has opened the window, so on-screen
layout, WebView2 availability and CSP behaviour in the shipped shell are unverified. That gap is
carried in `P0_KNOWN_LIMITATIONS.md` rather than being closed by wording.

## Not met, in one place

| Item | Status |
| --- | --- |
| CI green | NOT RUN - workflow committed, never executed (no push authorization) |
| cargo-deny licenses/bans/advisories | NOT RUN - tool not installed locally |
| Peak RSS | NOT MEASURED - reason documented in `P0_PERFORMANCE_REPORT.md` |
| Fuzz campaign | NOT RUN - tool policy did not authorize `cargo-fuzz` for P0 |
