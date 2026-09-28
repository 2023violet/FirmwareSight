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

Status of the track changed three times after this list was first written. Run #1 failed four jobs;
the remediation fixed them and Run #2 measured six of seven green; the second round added the one
missing provisioning step, and Run #3 (`36399805005`, head `1cd6309`) concluded **`success`, 7 of 7
jobs green**. P0 is therefore `CONDITIONAL_PASS`, the condition being the architect's promotion
sign-off rather than any untested check. The counts below are this tree's, and where an item is
confirmed by a runner rather than only by this machine, the evidence column says which.
`P0_CI_REPORT.md` keeps all three runs; `P0_CI_REMEDIATION_REPORT.md` and
`P0_CI_RUN_2_CLOSURE_REPORT.md` explain the two rounds.

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
| 12 | SQLite migration smoke passes | LOCAL PASS + REMOTE PASS | 11 storage tests, all green in Run #2 on Windows and Ubuntu, two ordered migrations, `schema_migrations`, refusal of a newer schema, and an in-place upgrade of a hand-built version-1 database |
| 13 | SQLite snapshot transaction/roundtrip passes | LOCAL PASS | `a_snapshot_survives_a_full_round_trip_with_identical_facts`, `a_failed_import_leaves_no_build_visible_as_complete` |
| 14 | ts-rs generation works | LOCAL PASS | 10 export tests + `drift/ipc bindings unchanged` |
| 15 | TypeScript strict build passes | LOCAL PASS | `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `useUnknownInCatchVariables`; `tsc --noEmit` clean |
| 16 | minimal Tauri summary builds | LOCAL PASS, and the window was opened | `cargo check -p firmwaresight-desktop --all-targets` clean. In the shipping configuration, `cargo build --release -p firmwaresight-desktop --features custom-protocol` produced a 10,400,768-byte `firmwaresight-desktop.exe` containing the current built asset names. The window was launched on a real display and both fixtures analyzed - see `P0_DESKTOP_SMOKE_REPORT.md`, which also records the storage defect that launch found |
| 17 | blocking parse not on the WebView event loop | LOCAL PASS | off-thread test + `Session: Send + Sync` assertion |
| 18 | Core/CLI/Desktop parity passes | LOCAL PASS | 7 parity tests |
| 19 | 512 MiB guard works before allocation | LOCAL PASS | accepted at 512 MiB, rejected at +1 KiB with no hash/read time |
| 20 | ~100/256/512 MiB workloads recorded | LOCAL PASS | `P0_PERFORMANCE_REPORT.md` measured table |
| 21 | golden tests pass | LOCAL PASS | 5 CLI golden tests |
| 22 | fmt passes | LOCAL PASS | `cargo fmt --all -- --check` |
| 23 | clippy passes | LOCAL PASS + REMOTE PASS | `cargo clippy --workspace --all-targets --all-features -- -D warnings`, silent here; Run #2's `Rust (windows-latest)` and `Rust (ubuntu-latest)` both passed 5/5 steps |
| 24 | cargo test passes | LOCAL PASS + REMOTE PASS | 104 tests, 0 failures here; Run #2's two Rust job logs each sum their `test result: ok.` lines to 104 |
| 25 | frontend typecheck/build passes | LOCAL PASS + REMOTE PASS | typecheck, lint, 19 tests, `vite build`; Run #2's `Desktop UI (ubuntu-latest)` logs `Tests 19 passed (19)` |
| 26 | no forbidden dependency or boundary violation | LOCAL PASS + REMOTE PASS | `cargo deny check licenses bans sources advisories` → exit 0, `advisories ok, bans ok, licenses ok, sources ok` with `cargo-deny 0.20.2` installed. The `[bans] deny` list is evaluated over the four shipping targets, so `reqwest`/`hyper` (an Android/iOS-only dependency of `tauri 2.12.0`) stay banned rather than exempted; `anyhow` is banned with `wrappers` limited to the Tauri tree, so a first-party use still fails. The same command passed remotely: Run #2's `Dependency policy` job is green. Two informational advisories are recorded with reasons and now carry the architect's acceptance - see `P0_CI_REMEDIATION_REPORT.md` and `P0_DEPENDENCY_REPORT.md` |
| 27 | P0 reports complete | LOCAL PASS | All 17 documents the prompt's `P0_TECHNICAL_VALIDATION/` list names are present, plus `P0_EXECUTION_PROVENANCE.md` from the takeover, `P0_DESIGN_CHECKLIST.md` required by AGENTS.md 11, and the three the CI track adds: `P0_CI_REMEDIATION_REPORT.md`, `P0_DESKTOP_SMOKE_REPORT.md` and `P0_CI_RUN_2_CLOSURE_REPORT.md` - 22 files, each citing a command or a run that actually happened |
| 28 | known limitations explicit | LOCAL PASS | `P0_KNOWN_LIMITATIONS.md` |

## Item 16, stated precisely

`minimal Tauri summary builds` is met in the narrow sense the phrase allows: the desktop crate
compiles and links, and in the configuration that actually ships - `--features custom-protocol`,
which is what `tauri build` passes - `tauri::generate_context!` resolves and embeds the built
frontend (`dist/`, 231.19 kB JS + 7.62 kB CSS). The embedding is not taken on trust: the binary
contains the current asset names, while the same command without the feature is 68 kB smaller and
contains no such string, because tauri's dev-mode codegen embeds nothing
(`tauri-2.12.0/build.rs:253`, `tauri-codegen-2.7.0/src/context.rs:178`).

Timings for the release link were taken against a populated `target/`: 6m 32s for the first
`custom-protocol` link in this round, 2m 51s for the rebuild after the storage fix, and 5m 46s for
the first cold link earlier in this track. They are single-machine numbers and are quoted as such.

The gap this item used to carry - "nobody has opened the window" - is closed: `P0_DESKTOP_SMOKE_REPORT.md`
records two launches in a real session, the on-screen facts compared field by field against the CLI,
the WebView2 runtime version, the CSP that governed them, a clean close, and the defect the second
launch was built to find. One sub-item stayed not-observed (the loading state, too fast to capture by
hand) and is reported that way rather than as a pass. It is carried forward rather than repeated in
round 2, because that round changes no runtime source - one workflow step.

"Compiles and links" is no longer a single-machine claim. Run #2 built `firmwaresight-desktop` on
`ubuntu-latest` as part of `clippy --workspace --all-targets --all-features` and passed, after the apt
step installed the GTK stack; the same crate's 17 tests are inside the 104 both Rust jobs report. The
`drift` job's failure is the same compile on a job that never got those libraries, which is the whole
of what remains open.

## Not met, and where each one stands

| Item | Status |
| --- | --- |
| CI green | **MET** — Run #3 on `1cd6309`: `success`, 7 of 7 jobs. Runs #1 and #2 stay in the record as the two failures that got there |
| Generated output drift on Ubuntu | **MET REMOTELY** — the drift job now runs `Install Linux prerequisites for the Tauri shell` (`Setting up libwebkit2gtk-4.1-dev` in its log) and reports `5/5 steps passed`. It was NOT LOCALLY EXECUTED before Run #3; Run #3 is that execution |
| P0 promotion | NOT CLAIMED — `P0 = PASS`, `v0.6.0` and a regenerated `SHA256SUMS` are the architect's signed act. This pack says `CONDITIONAL_PASS` with the condition named |
| cargo-deny licenses/bans/advisories | MET locally and remotely — exit 0 here, and Run #2's `Dependency policy` job green |
| Peak RSS | NOT MEASURED - reason documented in `P0_PERFORMANCE_REPORT.md`; a measurement gap, not a promotion blocker |
| Fuzz campaign | NOT RUN - tool policy did not authorize `cargo-fuzz` for P0. Unchanged by Run #3: a green CI is not a fuzzer |
| `v0.6.0` | NOT GENERATED — the all-green remote run now exists (Run #3), but the promotion act that would justify the version belongs to the architect's next prompt, so nothing was generated here |
