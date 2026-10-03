---
title: "Post-G2 E2E Remediation Exit Checklist"
doc_id: "FS-POSTG2-EXIT"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "POST_G2_E2E_FINDINGS_REMEDIATION"
owner: "Engineering"
last_updated: "2026-10-02"
---

# Exit criteria — settled box by box

Each box is settled by something that ran, cited to the file or command that shows it. `R§n` is a
section of `REMEDIATION_REPORT.md`, `V§n` a section of `FOCUSED_REVALIDATION_REPORT.md`, and an
external name is a file in `%TEMP%\FirmwareSight-PostG2-E2E-Remediation-20261002`.

## E2E-F001

- [x] **Reproduced from evidence** — the round's own record: `13_findings/E2E-F001.md`,
      `02_analyze/E2E-F001-pending-selection-no-banner.png`, 3/3 in that round. Read, not remembered.
- [x] **Regression red before fix** — the two new tests were written first and failed: without the
      identity in `stale` they end with `Unable to find role="region" and name "Last good analysis"`.
      Re-proved against the final code as a mutation (R§Validation).
- [x] **Fixed** — `e816dcb`: `analyzedSelectionId` owned by `App` beside `lastGood`,
      `stale = error !== null || pendingSelection`.
- [x] **Same-filename pending selection truthful** — V§E2E-F001, and `r3_B_pending.png`: "…a different
      file with the same name, and it has not been analyzed yet" over A's unchanged figures.
- [x] **Success clears the marker** — card pixels 38,462 → 0 after analyzing B, and the region label
      returns to `Analysis summary` (pinned in `intake.test.tsx`).
- [x] **Failure-path previous-analysis semantics preserved** — run D in all three sessions keeps the
      last-good report (4,240–4,249 ink) with its own sentence; the two pre-existing failure tests still
      pass unchanged.
- [x] **Focused desktop 3/3** — three fresh sessions, all PASS.

## E2E-F002

- [x] **Reproduced from evidence** — `13_findings/E2E-F002.md` and the Tier A screenshot of the card
      over a foreign folder.
- [x] **Regression red before fix** — the two new tests failed first; the mutation proof additionally
      shows that the brief's single-line mutation is not sufficient to make them red, and names the
      guard that is (R§Validation).
- [x] **Foreign destination never gets a false recognized-bundle claim** — Export is disabled before
      the engine can answer 6106, and the status line says the folder "is not a bundle this engine
      wrote". Verified on screen, `r_case3_after.png`.
- [x] **Foreign destination never gets a Replace action** — `replace_offered false` and
      `press_offered_replace false`, three of three, with the confirmation card detected as a new
      accent control rather than by reading text.
- [x] **Canary survives 3/3** — `DO_NOT_DELETE.txt` sha `9197d76cae958857` identical before and after,
      directory listing unchanged, three of three. No foreign directory was deleted: the round's E2E-S0
      condition never triggered.
- [x] **Recognizable bundle Replace still works** — case 2, three of three: first press answers 6106
      with the card, second press writes 10 files.
- [x] **Cancel preserves the tree** — `tree_after_cancel == tree_before == 3f2b489eb871ed08`, three of
      three.
- [x] **New destination export works** — case 1 by the journey (V§Happy path, 10 files, 59/59
      portability checks) and again as case 3's recovery step, three of three.
- [x] **Backend `ERR-BUNDLE-6107` safety remains** — `crates/firmwaresight-project/src/bundle.rs` and
      its tests are in neither commit's file list; the retargeted UI test walks the real
      6106 → 6107 sequence instead of mocking a state the engine cannot return first.

## E2E-F003

- [x] **Exact original definition read** — `13_findings/E2E-F003.md`, including the three measured
      banner offsets and the positive control.
- [x] **Included or excluded with an evidence-based reason** — included: all eight gate conditions held
      (genuine defect, reproducible, narrow, no schema major change, no migration 0005, no new
      dependency, no architecture change, no P5). The decision and the reasoning are in
      `TAKEOVER_REPORT.md`, made with the owner's explicit choice on record.
- [x] **Its own regression / revalidation passed** — `p0_acceptance.rs` +1 test with a 500,000-byte
      preamble; target test 30 runs ×2 series, whole file 10 runs; desktop 5/5 including the m10
      positive control rendering byte-identically to the old binary.

## Regression

- [x] **Target tests repeated** — 30 fresh processes each for F001 and F002, 30 for F003, zero flakes
      (`REPEATS.log`, two series).
- [x] **Full related files repeated** — `intake.test.tsx` 30 ×2, `release.test.tsx` 30 ×2,
      `p0_acceptance.rs` 10.
- [x] **UI suite green** — 159 passed in 6 files, twice (before and after the copy change).
- [x] **Rust green** — `cargo test --workspace` 770 passed / 0 failed; clippy `-D warnings` clean; fmt
      clean.
- [x] **`check.py` 15/15** — `gate_check_py.txt` and `gate_check_py_final.txt`, the second run on the
      committed state after the last source edit.
- [x] **Shipping build green** — exit 0, `ca2a4cdb…`, 14,994,944 bytes, and the embedded asset
      `index-ChIB_w0m.js` is checked to be the one the final sources produced.

## Focused desktop

- [x] **Analyze → Compare → Gate → Bundle smoke green** — V§Happy path, plus 59/59 independent bundle
      portability checks.
- [x] **Original DB restored byte-exact** — `ORIGINAL_DB_RESTORED = YES`,
      `ORIGINAL_DB_SHA_MATCH = YES`; the round's own store was retired, not deleted.
- [x] **No S0** — none observed.
- [x] **No S1** — none observed.

## Remote

- [x] **Product fix head 7/7** — Run #43 `37100371601`, attempt 1, completed / success, 7 of 7 jobs,
      on head `971015f4d106f2e77efea52fe225a20ed471a84d`. No rerun.
- [ ] **Closure successor 7/7** — this commit is that successor. Its run cannot be recorded inside
      itself, and the brief forbids a further commit merely to record its own run, so the result is
      reported to the Architect in the round's final message rather than written back here.

## Scope

- [x] **No P5** — no History, installer, signing, updater, SBOM, CVE, cloud, auth, telemetry or AI
      surface appears in either commit.
- [x] **No performance refactor** — `map.rs` changes one predicate on text already in memory; no
      streaming, no mmap, no parser redesign, no memory optimization, no guard change.
- [x] **No schema / migration** — `drift/ipc bindings unchanged`, `drift/goldens unchanged`, no
      migration file touched, `schema_version` unchanged.
- [x] **No new dependency** — `pnpm install --frozen-lockfile` reports the lockfile up to date, and
      `deny/cargo-deny` passes.
- [x] **No license decision** — licence remains **PENDING OWNER CONFIRMATION**; no licence file was
      touched.
- [x] **No baseline change** — G2 stays PASS, product stays MVP CANDIDATE, `active_task` stays NONE,
      `baseline_version` stays 0.6.0. `BASELINE.yaml` is deliberately not edited: this round changes no
      stage status, and its `post_mvp_candidates` block is a registry pointer, not a place for
      ad-hoc items.

## Carried forward

Excluded by the brief and untouched, recorded so the next round starts from the same list: near-500 MiB
first-use latency and peak RSS (worded in `REMEDIATION_REPORT.md` as `PARTIAL / environment-sensitive`
and `MEASURED FOR TESTED WORKLOAD`), 125/150 % DPI coverage, mouse-wheel observation, the
`update_goldens` cleanup issue, the E2E harness refactor, and the design-system boundary that this round
did not cross.
