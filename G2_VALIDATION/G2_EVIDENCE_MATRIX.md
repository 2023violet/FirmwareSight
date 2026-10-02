---
title: "G2 Evidence Matrix"
doc_id: "FS-G2-MATRIX"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "G2_ENGINEERING_CLOSURE_AUDIT"
owner: "Engineering"
last_updated: "2026-10-01"
---

# G2 — evidence matrix (prompt §9, §11–§15, §27; addendum §7–§8)

Classes: **PROVEN** · **PARTIAL** · **NOT_MEASURED** · **NOT_APPLICABLE** · **BLOCKING_FAIL**. Every
row cites an existing validation document, a command run in this round, or a numbered step of
`G2_END_TO_END_SMOKE_REPORT.md` (written `S<n>`). "This round" means on the audited tree
`e35cfe70aa0e8f273a75ac14b9dc62377483af36`. A PARTIAL or NOT_MEASURED row is not converted by prose.

## 1. The G2 exit authority (`06_DELIVERY/06_STAGE_GATES.md` §G2, after `ADR-0026`)

| Criterion | Class | Evidence |
| --- | --- | --- |
| Start / Project → Analyze → Compare → Release Gate → Release Bundle, locally | PROVEN | CLI chain, five commands exit 0 twice; desktop S1–S33 on the shipping binary |
| Deterministic output correct | PROVEN | §3 of this file |
| Supported-input behaviour reliable | PROVEN | five supported fixtures exit 0 (CLI); S3–S6 |
| Failure recoverable, Unknown never hidden | PROVEN | scenarios A, B, C, U, F; `unknown_state_is_never_rewritten_and_never_counts_as_pass`; the stored-state CHECK in `0003_gate_history.sql` |
| UI credible as a Minimum Credible Product | PROVEN | S1–S43, O1–O4, K, W, S; non-blocking clarity items L19–L21 |
| Release output portable | PROVEN | S34–S41; reader 64/64 and relocated 59/59 |
| Cross-platform CI green | PROVEN for `e35cfe7` | Run `36906482900`, attempt 1, 7/7. The closure commits' own runs are recorded in the closure report |
| Supported fixtures do not crash | PROVEN | malformed set exits 3 with a code, no panic line |
| Real Windows desktop smoke passes | PROVEN | `G2_END_TO_END_SMOKE_REPORT.md`, every step PASS |
| Tests reproducible | PROVEN | clean detached worktree 17/17 at `e35cfe7`, no ignored or untracked dependency |
| User-panel metrics (import activation, TTFV %, completion, comprehension, discovery, return, pay, pilot) | NOT_APPLICABLE to the current G2 | `POST_MVP / NOT CURRENT GATE`; V0 is `0 / 8` and nothing is fabricated |

## 2. P1–P4 stage audit (§11)

| Stage | Acceptance source | Verdict | Current-tree regression evidence | Known gaps | G2 blocking? |
| --- | --- | --- | --- | --- | --- |
| P1 Analyze | US-001, `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:13-21` | PASS / COMPLETE, `P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md:85-88`; CI Run `36556735551` on `e63afaf`, 7/7 | this round: 769 Rust incl. `analyze_queries`, `desktop_parity`, `golden`; UI 155; S3–S10 (unsupported reason, hash, sortable/filterable symbols, bytes/KiB) | dead `Apply filter` click UNRESOLVED (L25); keyboard traversal NOT RUN *then* — run this round (K) | NO |
| P2 Compare | US-002, `:23-31` | PASS / COMPLETE, `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md:128-132`; Run `36648718199` on `4a77ea1`, 7/7 | `diff_schema_contract`, `p2_golden`, `diff_cli`, `compare_ipc`; S11–S20, CLI = desktop byte-for-byte | step 27 PARTIAL (L9); object attribution Unavailable (L8) | NO |
| P3 Release Gate | US-003, `:33-40`; PRD P0-5 | PASS / COMPLETE, `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md:98-101`; Run `36783457030` on `02e8a81` and `36784382005` on `f66a93d`, 7/7 | `gate_history`, `gate_cli`, `release_gate`, `gate_schema_contract`, `config_schema_contract`; S21–S28, A, B, U; one run id from both surfaces | step 30 prior-run re-read PARTIAL — no History surface (L10) | NO |
| P4 Release Bundle | US-004, `:42-49`; PRD P0-6 | PASS / COMPLETE, `P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md:159-165` (67 boxes); Run `36872456446` on `e799f2f`, 7/7 | `release_records`, `bundle_builder`, `bundle_ipc`, `release_cli`, `p4_golden`, `release_schema_contract`, `analysis_schema_contract`; S29–S42, O1–O4, C | Explorer double-click NOT OBSERVED; DPI and second host NOT RUN | NO |
| US-005 CLI | `:51-58` and its v0.4.0 clarification `:62` | covered across P2–P4: no stage pack is dedicated to it | non-interactive chain under `< /dev/null`; `--json` on every verb; exit codes `0/2/3/4/5/6` (`apps/cli/src/main.rs:41-50`) with `exit_code_one_is_never_produced` and `every_aggregate_maps_to_its_own_exit_code_and_none_of_them_is_one`; no desktop process; Git unavailable → UNKNOWN → REVIEW (scenario B) | — | NO |

Later stages changed earlier semantics only by name and in the open: the P1-A0 correctness closure,
`LayoutSource::as_label()` at P2 closure, `SCHEMA_VERSION` 2→3→4 by additive migrations, and the named
assertion changes `.ai/DECISIONS.md` lists for P2, P3 and P4. No later stage reinterpreted a field of a
published contract (§4).

## 3. Determinism (§25)

| Output | Producers compared | Class |
| --- | --- | --- |
| SnapshotId, base and target | CLI run A, CLI run B, desktop S4/S6 | PROVEN — identical |
| Diff document `diff.json` / HTML | CLI A, CLI B, desktop export S19/S20, bundle copy | PROVEN — byte-identical (`1930cdb6…`, `aa4b08df…`) |
| Gate run id | CLI A, CLI B, desktop S25, the restored-workspace re-run | PROVEN — `gate-e692…` every time; the re-run deduped onto the stored row |
| release id, `release-manifest.json` | CLI A, CLI B, desktop S30/S33, the Replace export O4 | PROVEN — `release-2a7849c0…`, `85e53c18…` |
| `analysis.json`, `gate-results.json`, `accepted-reviews.json`, `release-report.html`, `SHA256SUMS` | CLI A vs B vs desktop | PROVEN — `diff -r` identical |
| destination independence | `out-a/` vs `deeper/out-b/nested/` | PROVEN — nothing moved |
| allowed clocks | SQLite `created_at`; `accepted_at` only when a person accepts | PROVEN — zero acceptances here, and the reader found no wall-clock value in any bundle document |
| committed goldens | `p4_golden`, `p2_golden`, `golden` suites | PROVEN — `the_golden_documents_are_the_bytes_the_shipped_binary_wrote` and siblings pass this round |

## 4. Public contracts (§12)

| Contract | `$id` | `schema_version` | Top level closed | Current outputs validate | Host-path field |
| --- | --- | --- | --- | --- | --- |
| project-config | `urn:firmwaresight:schema:project-config:1` | `schema_version`, required, const 1 | `additionalProperties: false` | `config_schema_contract` | `release.release_notes_path` is `string, minLength 1`; relative-only is enforced by the loader (`an_absolute_or_escaping_notes_path_in_a_config_is_a_hard_error`), not by the schema |
| analysis | `urn:firmwaresight:schema:analysis:1` | `schemaVersion`, required, const 1 | false | reader 64/64; `analysis_schema_contract` | none |
| diff | `urn:firmwaresight:schema:diff:1` | `schemaVersion`, required, const 1 | false | reader; `diff_schema_contract`; `the_portable_diff_validates_against_the_published_schema` | none |
| gate-results | `urn:firmwaresight:schema:gate-results:1` | `schema_version`, required, const 1 | false | reader; `gate_schema_contract` | none |
| accepted-reviews | `urn:firmwaresight:schema:accepted-reviews:1` | `schema_version`, required, const 1 | false | reader; `release_schema_contract` | none |
| release-manifest | `urn:firmwaresight:schema:release-manifest:1` | `schema_version`, required, const 1 | false | reader; `the_manifest_validates_against_the_published_v1_schema` | `files[].path` is `string`; bundle-relative is enforced by the release model (unsafe paths refused, `crates/firmwaresight-report/src/release.rs`) and checked here (path boundary 15) |

All six are major 1; no v2 file exists. Unknown is a value of the contract, not an absence:
`unknown` / `not-provided` / `unavailable` in analysis, `reason` beside a null in diff, `UNKNOWN` in the
gate state enum. The `schemaVersion` / `schema_version` spelling split follows each contract's family
(P2/P4 camelCase documents vs the P3 snake_case gate family) and is part of the published shape, not a
drift to correct. **Class: PROVEN.**

## 5. Storage (§13)

| Check | Class | Evidence |
| --- | --- | --- |
| `SCHEMA_VERSION = 4`, migrations 0001–0004 | PROVEN | `crates/firmwaresight-storage/src/db.rs:14`; the smoke store's `schema_migrations` lists all four |
| fresh → v4 | PROVEN | `a_fresh_database_migrates_to_the_supported_version` (`tests/storage.rs:73`) |
| v1 → v4, v2 → v4, v3 → v4, data preserved | PROVEN | `release_records.rs:351`, `:383`, `:415` |
| future version fails closed | PROVEN | `an_unknown_newer_schema_is_refused_rather_than_reset` (`storage.rs:303`); CLI `a_schema_version_this_build_does_not_know_exits_two_and_refuses_to_reset` (`apps/cli/tests/gate_cli.rs:151`) |
| failed migration recoverable | PROVEN | `a_failed_0004_leaves_the_v3_database_recoverable` (`release_records.rs:533`); `…gate_migration…recoverable` (`gate_history.rs:314`) |
| foreign keys enabled | PROVEN | set on every open (`db.rs:82-86`); `foreign_keys_are_enforced_on_the_connection` (`storage.rs:127`) |
| GateRun immutable | PROVEN | triggers `gate_runs_are_immutable`, `gate_findings_are_immutable` (`0003_gate_history.sql:108-117`); `a_stored_verdict_cannot_be_rewritten_by_sql` (`gate_history.rs:597`); `the_same_run_id_hiding_a_different_verdict_is_an_invariant_not_an_update` (`:543`) |
| Accepted Review immutable | PROVEN | triggers refusing UPDATE and DELETE (`0003_gate_history.sql:120-129`); `an_acceptance_is_an_audit_record_that_neither_edits_nor_disappears` (`gate_history.rs:894`) |
| Release record immutable | PROVEN | triggers (`0004_release_records.sql:74-83`); `a_stored_record_refuses_update_and_refuses_delete` (`release_records.rs:732`); `one_release_id_with_different_facts_is_an_invariant_not_an_update` (`:693`); O4 wrote no second record |
| no `UPDATE`/`DELETE` on those tables in source | PROVEN | 0 matches under `crates/firmwaresight-storage/src` |
| 0003 and 0004 additive | PROVEN | no `DROP` or `ALTER` in either; 0002's table rebuild is the recorded P0 correction |
| no raw artifact bytes | PROVEN | no `BLOB` column in any migration; 0 BLOB cells in the smoke store |
| path persistence | PROVEN as corrected by the addendum | §7 of this file |

## 6. Architecture and capability surface (§14, §15)

| Check | Class | Evidence |
| --- | --- | --- |
| exactly five first-party library crates | PROVEN | workspace members: core, artifact, project, storage, report + `apps/cli` + `apps/desktop/src-tauri` |
| Core headless, synchronous, zero-dependency | PROVEN | `crates/firmwaresight-core/Cargo.toml` `[dependencies]` empty; 0 hits in its `src` for `tauri`, `rusqlite`, `tokio`, `async fn`, `std::fs`, `std::process`, `std::net`, `thread::spawn` |
| no project-authored `unsafe` | PROVEN | 0 `unsafe` blocks in first-party `src`; `#![forbid(unsafe_code)]` in all five crates and the desktop shell (the CLI binary has 0 `unsafe` without the attribute) |
| Project owns config, Git evidence and bundle staging | PROVEN | `firmwaresight_project::bundle::{prepare, BundlePlan::publish, verify_bundle}`; desktop (`src-tauri/src/bundle.rs:104`, `:211`) and CLI (`apps/cli/src/main.rs:502`, `:510`) call the same engine. `P4_BUNDLE_EXECUTION_REPORT.md` §1's sentence that "the engine in `apps/desktop/src-tauri` and the CLI own staging" describes the callers, not where the code lives |
| Storage owns SQLite | PROVEN | `rusqlite` is a dependency of `firmwaresight-storage` only |
| Report owns portable projections and rendering | PROVEN | its only `std::fs` is inside a `#[test]` (`schemas.rs:85-95`) |
| Git adapter read-only and bounded | PROVEN | `crates/firmwaresight-project/src/git.rs`: `rev-parse --show-toplevel`, `rev-parse HEAD`, `tag --points-at HEAD`, `status --porcelain`; `GIT_COMMAND_TIMEOUT` 5 s, output cap, no shell |
| WebView capability minimal | PROVEN | `capabilities/main.json` grants `core:default` only; one plugin, `tauri_plugin_dialog`, initialised in Rust (`lib.rs:915`) and not granted to the WebView |
| use-case IPC only | PROVEN | 23 registered commands; every parameter is a typed request DTO, an opaque id or token, or a bool — none is a path, SQL, shell or Git argument |
| no network, cloud, account, telemetry | PROVEN | 0 `TcpStream`/`UdpSocket`/`std::net` in first-party source; 0 `fetch`/`XMLHttpRequest`/`WebSocket` in UI source; UI runtime dependencies are `@tauri-apps/api`, `react`, `react-dom`. `reqwest`, `hyper` and `tauri-plugin-fs` appear in `Cargo.lock` as optional Tauri edges, and `cargo tree -p firmwaresight-desktop --features custom-protocol -e normal` lists no `reqwest`, `hyper` or `rustls` for Windows, Linux or macOS; `tauri-plugin-fs` is compiled in under the dialog plugin with no WebView permission |
| CSP | PROVEN | `default-src 'self'; script-src 'self'; … object-src 'none'; frame-ancestors 'none'`; no updater, `bundle.active: false` |
| dependency policy | PROVEN with accepted risks | `cargo deny`: `advisories ok, bans ok, licenses ok, sources ok`; `RUSTSEC-2024-0429` and `RUSTSEC-2024-0370` ignored with reasons in `deny.toml:48-63`. Never "security clean" |

## 7. Path and privacy boundary (addendum §3, §4, §8)

Run over this round's outputs by a scanner whose probes are the machine's own path vocabulary (the
scratch root, the account directory, `AppData`) plus drive-letter, POSIX-absolute and UNC shapes. Probe 1
is the positive control: the same scanner must *see* the path where it is allowed to be.

| # | Check | Class | Evidence |
| --- | --- | --- | --- |
| 1 | `artifacts.path` holds the local source path | PROVEN (allowed) | 4 of 4 rows, e.g. `%LOCALAPPDATA%\Temp\fs_g2\subject\source\base\firmware.elf` |
| 2 | `SelectionDto` has no source path | PROVEN | fields `selectionId, fileName, mapFileName, mapAttached` |
| 3 | `ArtifactDto` has no source path | PROVEN | `fileName` and facts only |
| 4 | Analyze summary has no source path | PROVEN | `AnalysisSummaryDto` fields; the window's text at S3, S5, S9: 0 hits |
| 5 | Diff JSON / HTML | PROVEN | CLI and desktop exports: 0 hits |
| 6 | Gate portable JSON | PROVEN | PASS, dirty and no-Git documents: 0 hits |
| 7 | Gate evidence locators | PROVEN | 135 cells (portable refs + every stored `gate_finding_evidence` row): 0 hits; locators refuse `\` (`a_host_path_can_never_be_stored_as_an_evidence_locator`, `gate_history.rs:664`) |
| 8 | Bundle preview DTO | PROVEN | `BundlePreviewDto` fields carry ids, a version and `bundleFolderName`; `BundleFileRowDto.path` is bundle-relative by contract; window text at S30, S33: 0 hits |
| 9–14 | `analysis.json`, `diff.json`, `gate-results.json`, `accepted-reviews.json`, `release-manifest.json`, `release-report.html` | PROVEN | desktop, CLI and relocated copies: 0 hits each |
| 15 | `SHA256SUMS` paths bundle-relative | PROVEN | 8 names, none absolute, none with `\` or `..` |
| 16 | `release_records` holds no path | PROVEN | columns `id, build_id, baseline_build_id, gate_run_id, release_version, manifest_sha256, created_at`; 0 hits in its cells; `the_release_table_has_no_column_that_could_hold_a_path_or_a_blob` (`release_records.rs:632`) |
| 17 | relocated bundle verifies without SQLite, source or project | PROVEN | S39–S41: 59/59 and 64/64 |
| — | no new path persistence | PROVEN | across migrations 0001–0004 the only path-like column is `artifacts.path` (`0001_initial.sql:34`) |
| §5 | immutable artifacts' own bytes | NOT_APPLICABLE (by the addendum) | each P2 fixture ELF carries its compiler's build directory (`D:\…\fixtures\elf\p2-diff\<side>`) in its DWARF. The bundle ships those bytes unchanged and hash-verified; no composed document copies the string out (rows 9–14) |

## 8. Findings of this round

| Finding | Observation | Classification | G2 blocking | Status |
| --- | --- | --- | --- | --- |
| **G2-F1** | `release.test.tsx` "puts focus on the decision it just asked for" read `document.activeElement` synchronously after `findByRole`; failed once in a full local run on `055b54e` and once in 30 single-file runs | test-harness race; production behaviour correct (O2 observed the focus in the window) | was YES until fixed | **CLOSED** in `e35cfe7`: `await waitFor(...)`, reproduced by a deferred-focus mutation, guarded by a no-focus mutation, 30/30, 30/30, 15/15; Run `36906482900` attempt 1, 7/7 |
| **G2-F2** | SQLite `artifacts.path` stores the local source path | **EXPECTED_LOCAL_PRIVATE_PERSISTENCE** | NO | Authority: `04_TECH/15_STORAGE_DATABASE_BASELINE.md` §7; `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md`. Portable leakage NONE observed; IPC/UI leakage NONE observed; release-record leakage NONE; bundle-generated-document leakage NONE. Architect adjudication: the G2 Storage Path Semantics Clarification Addendum v1.0 corrects the original prompt's over-broad "no host path persisted" wording |
| **G2-F3** | `scripts/update_goldens.py` dry run exits 1 on Windows: `shutil.rmtree` cannot remove a read-only `.git/objects` file in its own leftover `target/update_goldens-p4` | tooling defect, outside the shipped product and the gate | NO | recorded as L24; not fixed in this round, because no G2 requirement depends on the updater |
| clarity | object attribution "available" (Analyze) vs "unavailable" (Compare); a Core enum word in Compare; fixed window title; CRLF moving a release id | presentation and environment | NO | L19–L22 |

## 9. PRD MVP success metrics (§27, `01_PRODUCT/01_PRD_MVP.md:119-126`)

| Metric | Class | Evidence |
| --- | --- | --- |
| First useful analysis < 60 s | PROVEN on the supported fixtures | CLI analyze 42–52 ms each; desktop Analyze click → report 366 ms and 352 ms (S3, S5); P0 measured the 512 MiB synthetic pipeline at 2,794.7 ms. A user-panel *time to first value* is POST_MVP and not claimed |
| 500 MB working set does not freeze the UI | **NOT_MEASURED** | no desktop run at that size; peak RSS NOT MEASURED (L1, L2). Per §27's adjudication not a G2 blocker: the 512 MiB guard is enforced (`ERR-GUARD-0001`, re-proved this round) and no hang or crash was observed on the G2 fixtures. Not claimed as proven |
| supported fixture parse success 100% | PROVEN | 5 of 5 supported inputs exit 0 (CLI); desktop S3–S6 |
| supported formats never crash | PROVEN within the fixture set | no panic on any supported or malformed input; fuzzing NOT RUN (L3) |
| Diff exactly verifiable by fixture | PROVEN | `the_portable_diff_document_matches_the_committed_golden`, `the_html_export_matches_the_committed_golden_byte_for_byte`; CLI = desktop export byte-for-byte |
| Release Bundle readable on another machine | PROVEN as relocation | S39–S41: another directory root with source, project, store and app all unavailable. A second physical machine was not used (L4) |
