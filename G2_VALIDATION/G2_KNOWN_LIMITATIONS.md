---
title: "G2 Known Limitations"
doc_id: "FS-G2-LIMITS"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "G2_ENGINEERING_CLOSURE_AUDIT"
owner: "Engineering"
last_updated: "2026-10-01"
---

# G2 — canonical known limitations (prompt §28)

One list for the whole MVP, as it stands on the audited tree `e35cfe7`. Each row says what is true, where
the evidence is, what it costs, whether it blocks G2, and who owns it afterwards. Nothing inherited was
dropped because G2 is being judged; the rows marked *new* were found by this round.

"Post-MVP owner" names a stage identifier from `06_DELIVERY/06_STAGE_GATES.md`. It is where the item
would be picked up, not a commitment that it will be: every later track needs its own architect prompt.

| # | Limitation | Status on `e35cfe7` | Evidence | Impact | Blocks G2? | Post-MVP owner |
| --- | --- | --- | --- | --- | --- | --- |
| L1 | Peak RSS | NOT MEASURED | `P0_TECHNICAL_VALIDATION/P0_PERFORMANCE_REPORT.md` "Peak RSS: NOT MEASURED", with its reason | no memory ceiling is claimed for any workload | NO — §27 adjudication: the 512 MiB guard is enforced (`ERR-GUARD-0001`, re-proved this round with `--max-bytes 1024`) and no hang or crash was observed | P5 |
| L2 | 500 MB working set in the *window* | NOT MEASURED | P0 measured the CLI pipeline at 512 MiB in 2,794.7 ms; no desktop run at that size exists | UI responsiveness at that size is unverified | NO — same §27 adjudication | P5 |
| L3 | Fuzzing | NOT RUN | `cargo-fuzz` needs a nightly toolchain the tool policy never authorized | the no-panic claim rests on regression fixtures (`fixtures/malformed/*`, all exit 3 with a code this round) | NO | P5 |
| L4 | Desktop smoke scope | one Windows host, one WebView2 (`Edg/154.0.4258.48`), 100% DPI, 1040 × 760 viewport | `G2_END_TO_END_SMOKE_REPORT.md` "Run under test" | no macOS or Linux window, no other DPI, no second host | NO — the Linux and macOS code compiles and tests green on CI | P5 |
| L5 | Linker layout coverage | two layouts (`p0-dual-region`, the P2 pair) plus ELF-only | `fixtures/manifest.json`; `.ai/CURRENT_STATE.md` known gaps | executable-in-RAM, external SRAM, DMA pools, overlays and MPU-aligned sections unproven; DWARF recognized, not consumed | NO — the Supported cohort is GCC/Clang ELF + GNU ld MAP | P5 |
| L6 | `sections.file_offset` reason | nullable `INTEGER` with no `*_unknown` column | `crates/firmwaresight-storage/migrations/0001_initial.sql` (`file_offset INTEGER,` beside `load_unknown`, `mem_unknown`) | the window says `Unknown` with no reason (step 7: `.bss`) | NO — Unknown is still never zero | P5 (needs a migration) |
| L7 | `symbols.address` reason | nullable `INTEGER` with no `*_unknown` column | same file (`address INTEGER,` beside `name_unknown`, `size_unknown`) | same as L6 | NO | P5 (needs a migration) |
| L8 | Object / module attribution | Unavailable by evidence | Compare step 18; `diff.json` `objectChanges.available: false` with its reason | no object or module delta, and no Object tab | NO — PRD P0-3 makes it conditional on evidence | P5 |
| L9 | P2 smoke step 27, same-pair lock in the window | NOT VERIFIED then; the native `<select>` popup was again not mouse-driven here | `P2_VALIDATION/P2_COMPARE_DESKTOP_SMOKE_REPORT.md`; G2 step 24 used the keyboard path | covered by `compare.test.tsx` and the IPC tests | NO | — |
| L10 | No History surface; a prior Gate run or release record cannot be reopened in the window | P3 step 30 PARTIAL; P4 "Not observed" | `P3_VALIDATION/P3_GATE_DESKTOP_SMOKE_REPORT.md`; `P4_VALIDATION/P4_BUNDLE_DESKTOP_SMOKE_REPORT.md` | records persist (read back from the store this round) but are not browsable | NO — History is G6 scope, outside the MVP verbs | G3/P5 |
| L11 | User comprehension | UNTESTED with real users; V0 `0 / 8` | `09_ADR/ADR-0026-open-source-mvp-first-delivery.md` Consequences | whether people read `Unknown`, `Review` and a MAP request correctly is unverified | NO — user-panel metrics are POST_MVP for G2 | V1 |
| L12 | RustSec advisories | `RUSTSEC-2024-0429` (glib 0.18.5, unsound) and `RUSTSEC-2024-0370` (proc-macro-error 1.0.4, unmaintained) accepted in `deny.toml` with reasons | `deny.toml` ignore entries; `cargo deny` this round: `advisories ok, bans ok, licenses ok, sources ok` | "dependency policy passes with documented accepted risks" — never "security clean" | NO | P5 security review, or a recorded revisit trigger |
| L13 | Open-source license | `license = "Proprietary"` in the root `Cargo.toml`, no root `LICENSE` | `Cargo.toml` `[workspace.package]` | the public repository carries no open-source licence grant | NO for the technical gate — but Technical MVP Candidate ≠ open-source licensing completed | owner decision (`AGENTS.md` 9) |
| L14 | `scripts/update_goldens.py` key order (P1-A0 §9.7) | the source now serializes with `sort_keys=True` (`scripts/update_goldens.py:250`, last touched in `ee27416`), which addresses the recorded mismatch; **not re-verified by a dry run** this round, see L24 | `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` §9.7 | the goldens themselves are guarded by the test suite (`cli_analyze_json_matches_the_committed_golden`, `the_golden_documents_are_the_bytes_the_shipped_binary_wrote`, …), not by the updater | NO | P5 |
| L15 | `ElfProgramHeader` source label | still the enum word used for the ELF-only memory basis (`crates/firmwaresight-artifact/src/pipeline.rs:286`) | P1-A0 §9.7 | a label may name a source the parser does not read | NO | P5 |
| L16 | `custom-protocol` build requirement | still required: `apps/desktop/src-tauri/Cargo.toml` `[features] custom-protocol = ["tauri/custom-protocol"]` | G2 shipping build used it; page URL `http://tauri.localhost/` | a release build without it shows a WebView network error | NO — documented, and the gate's `clippy --all-features` builds it | P5 packaging |
| L17 | CI provisioning duplication; index blind spot | two jobs copy one apt block; only `drift/fixtures tracked` reads the index | `.ai/CURRENT_STATE.md` known gaps | maintenance cost; an accidentally ignored source file no manifest names is unseen | NO | P5 |
| L18 | Local database retains source artifact location as local-only implementation metadata | `artifacts.path` holds the location the user chose, as `04_TECH/15` §7 and P1-A0 designed it | G2 smoke "Database handling"; `G2_EVIDENCE_MATRIX.md` G2-F2 and the 17-point path boundary | a privacy and storage fact, not a defect: the column never reaches SnapshotId, IPC, UI, Gate locators, `release_records`, portable documents or a bundle | NO — adjudicated by the G2 Storage Path Semantics Clarification Addendum v1.0 | — |
| L19 *new* | "Object attribution" means two things | Analyze capabilities say `available` when a MAP is supplied; Compare and `diff:1` say `unavailable` | smoke steps 4 / 18; CLI `analyze` `capabilities.objectAttribution` vs `diff` `objectChanges.available` | the same words, different scopes, on two pages of one product | NO — no fact is wrong, the label is ambiguous | P5 |
| L20 *new* | Compare prints a Core enum word | "weakest basis MapRegionAndElfLoad" where Analyze prints `map-memory-configuration+elf-load` | smoke step 13–14 evidence block | an inherited P1 open design item, still current | NO | P5 |
| L21 *new* | Window title fixed | `tauri.conf.json` `"title": "FirmwareSight - Analyze"` on every page | smoke: title stayed "FirmwareSight - Analyze" on Release | cosmetic | NO | P5 |
| L22 *new* | Line endings move a release's identity | with the account's `core.autocrlf=true`, a checkout rewrote LF notes as CRLF; Release Notes digest is part of the Gate run id, and a CRLF copy produced a different run id with `git.clean` BLOCK | G2 smoke "Failure and recovery" note | two checkouts of one commit can disagree about a release id; the Gate fails closed rather than passing | NO — fail-closed, and the bundle ships the bytes it hashed | P5 |
| L24 *new* | `scripts/update_goldens.py` cannot rerun on Windows over its own leftover scratch | the dry run (`python scripts/update_goldens.py`, no `--confirm`) exited 1 in `release_subject()`: `shutil.rmtree(target/update_goldens-p4)` raised `PermissionError [WinError 5]` on a read-only `.git/objects` file left by an earlier run; no tracked file changed | G2 closure report §8 (G2-F3) | a developer refreshing goldens on Windows must clear that gitignored folder first; CI and the gate do not run this script | NO — tooling, outside the shipped product and outside the gate | P5 |
| L25 | one dead `Apply filter` click (P1) | UNRESOLVED, never reproduced on a shipped binary | `P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md` "Not measured, not run"; `P1_ANALYZE_DETAILS_SMOKE_REPORT.md` §6 | not seen again: this round's filter-bearing pages loaded and filtered through the shell, and `autoComplete="off"` removed the leading suspect | NO | P5 |
| L23 *new* | UI test races in the same shape | G2-F1 is the fourth instance of a test reading asynchronously delivered state synchronously (after P3's defects I and J and the pagination race `055b54e` closed) | `e35cfe7` | another may exist that no run has lost yet | NO — every observed instance is fixed and proved by mutation | P5 |
