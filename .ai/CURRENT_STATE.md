---
title: "Current Project State"
doc_id: "FS-AI-002"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-08"
---

# Current State

- Date: 2026-10-08
- Baseline: **v0.6.0 — P0 Technical Foundation Baseline**
- **Active task: `NONE`.** Stage **U1 is `CLOSED` — `PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS`**,
  issued 2026-10-08 by the Architect's own independent visual acceptance record and filed at
  `U1_VALIDATION/U1P_R3_ARCHITECT_FINAL_VERDICT_RECORD.md`; the machine state is `BASELINE.yaml`
  `u1_execution.architect_final_verdict`. The scope sentence is part of the verdict, not commentary: **visual
  readiness for a controlled V1 external-user study**, explicitly *not* GA quality, *not* feature completeness, *not*
  public distribution approval. The acceptance tally stands as the record guards it — **25 items: 23 PASS, 1 FAIL,
  1 NOT_VERIFIED, 0 NOT_CAPTURED, with 2 `MISMATCH_PROVED` flags inside the 23 PASS** — and the verdict's instruction
  is binding on every later document: **"DO NOT rewrite this tally to 25/25 or to 100%."** Deviations A–D stay
  enumerated and open (Analyze's 1024 band track, Compare's section table one scroll down, the Gate picker reset,
  Overview's never-recaptured Previous-analysis box), each with a phrase it now forbids; deviation E, a count defect
  in this file's sibling entry documents, is corrected by this update rather than by a stand-alone CI-loop commit.
  **V1 stays paused** — `IN_PROGRESS` / `RECRUITMENT_READY`, 0 eligible external sessions — because that record's
  closing section is titled `NEXT ACTION RECOMMENDATION, NOT EXECUTED` and it re-froze nothing itself: the newer
  product artifact's bytes were re-downloaded and re-hashed as *evidence about a candidate* only. **Dated later the
  same day, and this is the live build identity:** the owner then issued the separate authority the record asked for,
  and V1's cohort build is now `REFROZEN_TO_U1_CANDIDATE` — artifact `11573661113` of run `37828673549` at product
  head `41bb6a36`, NSIS `3,896,257` bytes / `9a51e86a…c87d93`, unsigned. See the bullet below this one.
  `P5` stays `PASS_COMPLETE`, Productization
  `ENGINEERING_COMPLETE`, `G2 PASS`, product **`MVP_CANDIDATE`** at `0.6.0`. B1 / RC / GA / public release / signing /
  notarization / updater / commercial distribution stay `NOT_AUTHORIZED`; licence `PENDING_OWNER_CONFIRMATION`;
  **L11 stays `CARRIED_FORWARD`** — a visual acceptance of the UI is not a research pass. Counts unmoved: **868 Rust
  across 47 result lines / 291 UI in 9 files**, gate 17 of 17, 30 IPC commands, `assets/design-tokens.json` at
  `94336906…14d4`. What is blocked is blocked on a person: recruitment needs real
  firmware engineers (§47/§48), the cohort bytes need private archival before they expire on 2026-10-22, and any UI
  follow-up needs its own prompt. Read `.ai/ACTIVE_TASK.md`'s top block, then
  stop — AGENTS.md 1 forbids lifting the next track off the roadmap.
- **V1's cohort build was re-frozen on 2026-10-08, under its own owner authority: `COHORT_BUILD =
  REFROZEN_TO_U1_CANDIDATE`.** *FirmwareSight — V1 Cohort Re-freeze / Execution Authorization v1.0* was delivered
  **inline** (no file, so `10_AUDIT/SOURCE_PROMPTS/README.md` records it the way the other inline authorizations are
  and the agent's labelled transcription at `V1_VALIDATION/00_authority/SOURCE_PROMPT_V1_REFREEZE_transcription.md`,
  SHA-256 `88ef07bd…a636a2`, is the transcription's digest and never the authorization's). It grants
  **only** `V1_COHORT_REFREEZE`: the formal research build moves off P5 Commit F3's artifact `11419727517`
  (`08fdfcb` / run `37475580080` / NSIS `3,888,432` B / `182506f2…63d12`) onto the build the Architect's visual
  acceptance was granted over — run `37828673549` (#85, attempt 1, **10 of 10**), head
  `41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177`, artifact `11573661113`, container `5,544,133` B
  (`330e83af…4f3fb4`, **not** the installer size), NSIS `FirmwareSight_0.6.0_x64-setup.exe` delivered as
  `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, **`3,896,257` bytes /
  `9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93`**, CLI companion `1,668,149` B
  (`924f3a71…1d8373`), payload `5026c47b…0ddaae`, version `0.6.0`, **unsigned**, `expired: false`, GitHub expiry
  **`2026-10-22T19:18:25Z`**. Both builds were downloaded and hashed again before either was written into an authority
  document (§二), each verified entry by entry against its own container `SHA256SUMS.txt`, and each **independently
  corroborated by GitHub's published artifact `digest`**, which equals the measured container hash on both sides
  (`sha256:330e83af…` for the live build, `sha256:40cb5c99…` for the superseded one). **Erratum, same day:** earlier
  documents in this repository — U1P-R1, the U1P-R3 verdict record, and this round's own first draft — wrote that
  "GitHub's `sha256_digest` is `null`". The field is named **`digest`** (value `sha256:<hex>`); no field called
  `sha256_digest` exists on these endpoints, and `--jq .sha256_digest` prints `null` for a *missing key*. The
  correction strengthens the freeze rather than weakening it: the checksum now has an authority outside this
  repository's own arithmetic. `F3`'s block in `BASELINE.yaml` is preserved and dated as
  superseded, not rewritten.** What did *not* move: sessions stay **0**, M1–M6 stay `NOT_MEASURED`, the metric
  operations/denominators/thresholds are untouched, U1 keeps its verdict and its guarded 25-item tally with
  deviations A–D open, P5 `PASS_COMPLETE`, product `0.6.0 MVP_CANDIDATE`, G2 `PASS`, B1/RC/GA `NOT_AUTHORIZED`, L11
  `CARRIED_FORWARD`, and the build was not installed, recruited for, or distributed. Why the swap is honest rather
  than convenient: F3→`41bb6a36` is 44 product paths and every one is the presentation layer (42 under
  `apps/desktop/ui/src`, plus `ipc.rs`'s additive `MainWindowPage::Overview` variant and two window titles, plus that
  test); `crates/`, `assets/`, `schemas/`, `fixtures/`, `golden/`, `.github/`, `scripts/`, `09_ADR/` and `templates/`
  are **byte-identical subtrees**, the same **30** commands are registered at both heads, both runs report **868 Rust
  across 47 result lines**, and `Cargo.lock` / `pnpm-lock.yaml` digests match — while the UI suite grew from **225 in
  8 files** to **291 in 9 files**, which is U1's own test contract, not a V1 metric. The docs-only head's own artifact
  `11592690212` (run `37874177649`, #87) is named only to be refused. Record:
  `V1_VALIDATION/V1_COHORT_REFREEZE_RECORD.md` (FS-V1-RF-001); machine state: `BASELINE.yaml`
  `v1_execution.cohort_build_refreeze` with `cohort_build_state: REFROZEN_TO_U1_CANDIDATE`. The re-freeze is commit
  `f633b28`, whose own CI run 37885059634 read back **10 of 10** at 868 Rust / 291 UI and 774 tracked paths; the digest
  erratum above rides in the single docs-only successor that follows it, and a round cannot cite its own hash, so the
  successor's run is the next reader's to record — `BASELINE.yaml`
  `v1_execution.cohort_build_refreeze.predecessor_docs_commit_ci_readback` carries `f633b28`'s figures job by job.
- **Active task as of 2026-10-07, kept as what the first U1 round was authorized to do and where it stopped** (the
  live pointer is the block above, and the dated blocks below carry the track through to its closure):
  `U1_UI_PRODUCTIZATION_CONVERGENCE`. A **UI productization track**, opened 2026-10-07 under
  《FirmwareSight B1 — UI Productization / Design Convergence》 v1.0, delivered **inline** (no file, so
  `10_AUDIT/SOURCE_PROMPTS/` records it the way the three inline P0 precedents are recorded, and the agent's
  labelled transcription is archived at `U1_VALIDATION/00_authority/`). The owner renamed it `U1` because `B1` is
  the canonical Private Beta identifier and stays NOT AUTHORIZED. It converged the desktop React shell toward the
  frozen seven reference screens: one shared design-system layer (`components/Button|Chip|Layout|PageHeader|Panel|
  RankBar|TopBar`), a new Overview page made only of DTOs the shell already returns, the Release verdict moved to
  the top of its page, and the F2R-01 block-table defect closed on Compare and Release, where it was still live.
  Start authority was HEAD = `origin/main` = `f481b78059c14e1c83d3ba18e082004b7de72ee2`, tree clean, run
  `37508243514` 10 of 10. Product state did **not** move: P5 `PASS_COMPLETE`, Productization
  `ENGINEERING_COMPLETE`, G2 `PASS`, **`MVP_CANDIDATE`** at `0.6.0`, and U1 claims no stage — not G3, not B1, not
  RC, not GA. Counts: **868 Rust (unchanged) / 236 UI in 9 files (was 225 in 8)**, 17-step gate shape unchanged
  and run **17 of 17 PASS with no SKIP** at the fully staged closeout,
  no new IPC command (the registry holds 30 at `f481b78` and 30 at `7dc2ca8`; the 27 an earlier draft of this
  line carried was wrong and is corrected in `U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md` §2), and the one
  non-CSS change is the additive `MainWindowPage::Overview` title variant
  declared in `BASELINE.yaml`'s `u1_execution.non_css_change`. Evidence and verdict:
  `U1_VALIDATION/U1_VALIDATION_REPORT.md`.
- **U1's status now: `READY_FOR_ARCHITECT_VISUAL_REVIEW`.** A second U1 unit ran the same day —
  `U1_PUSH_CI_AND_INSTALLED_VISUAL_ACCEPTANCE` (prompt v1.1, Architect authorized) — and it is the round that
  pushed `8efe9c8` + `7dc2ca8`, read remote run `37609108402` (#74, attempt 1) at **10 of 10**, installed the
  Windows artifact that run built (id `11477857379`, installer `372631c3…`, installed executable
  `afdc528b…`), and examined it on a real screen: six primary states at 1440×900, nine responsive captures at
  1024×720 and 1056×799, a functional smoke pass on those same bytes, an eight-dimension convergence matrix, and
  one open **U1-V2** presentation finding that was deliberately not self-waived. No U1-V3 or U1-V4 was observed.
  The owner's store was parked and restored byte-exact; the machine is uninstalled again, as it was. Evidence:
  `U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md`, with the review pack outside Git under
  `%TEMP%\FirmwareSight-U1-Visual-Acceptance-20261007T1037Z\`. That status is **not** visual approval, not
  `PASS_COMPLETE`, not `DESIGN_COMPLETE`, not `MOCKUP_MATCHED`.
- **U1's status now: `READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT`, and `U1-V2-06` is `CLOSED_BY_U1R`.** A third U1
  unit ran the same day — `U1R_STALE_CAPABILITY_CORRECTIVE_AND_FINAL_RECHECK` (prompt v1.0, Architect authorized,
  archived at `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1R_Stale_Capability_Corrective_Final_Recheck_v1.0.txt`,
  delivered bytes 28,659 / SHA-256 `0ff7b3f6…650bb`, 1,302 logical lines) — and it closed the one material
  presentation defect the installed pass had raised: the Analyze page's top capability strip used to borrow the
  previous file's words during a failed or unanalyzed attempt, so a green "ELF supported / MAP provided" row sat
  above a row naming a different file whose MAP was absent. The fix is a derived presentation value in
  `apps/desktop/ui/src/Analyze.tsx` plus one optional `stale` label on `Details.tsx`; last-good retention stays
  exactly where it was, no domain state was added, no Gate verdict was touched, and
  `assets/design-tokens.json` is byte-identical (`94336906…14d4`). Six contract tests guard it
  (**242 UI in 9 files, was 236 in 9**; **868 Rust unchanged**, 47 result lines, 0 failed) and reverting the
  derivation fails T2, T3 and T6 — the mutation was undone by digest, not by hand. Gate **17 of 17** with no SKIP,
  drift 8/8, deny 1/1, core-smoke 3/3, package 4/4, `verify_baseline_artifacts.py` PASS at 755 tracked files /
  753 sum entries. Product commit `4d36f103d2a8fd103e8e3b466e4d2b2010a0a4ef` pushed as a normal fast-forward;
  remote run `37637056980` (#76, attempt 1) **10 of 10**; Windows artifact `11491960105` installed from that
  commit's own bytes (NSIS `41f26a8c…`, installed executable `e62652bb…`) and re-checked on a real screen in the
  success, pending, failed-at-three-sizes and recovered states. The owner's store was parked and restored
  byte-exact and was never opened by this round; the machine is uninstalled again, as it was. Evidence:
  `U1_VALIDATION/U1R_CAPABILITY_STATE_DESIGN.md`, `U1_VALIDATION/U1R_CORRECTIVE_REPORT.md`, and the pack under
  `%TEMP%\FirmwareSight-U1R-Visual-Recheck-20261007T145124Z\`. This is still **not** visual approval, not
  `PASS_COMPLETE`, not `DESIGN_COMPLETE`, not `MOCKUP_MATCHED`: the eight `U1-V1` / `U1-V0` visual items stay
  `OPEN_FOR_ARCHITECT_VISUAL_JUDGEMENT`, and the Architect has not looked at the screenshot pack directly.
- **U1's status now: `REQUIRES_ARCHITECT_POLISH_REVIEW`.** Two more units ran on 2026-10-08, and the line above
  stays where it is because it is what U1R recorded on 2026-10-07. The first is the U1P product round —
  `U1P_FINAL_VISUAL_POLISH_AND_INFORMATION_HIERARCHY_CONVERGENCE` — which recomposed the shell and the five core
  pages onto a decision-first hierarchy without adding a capability: fixed rail with an independently scrolling
  main, the verdict above the metrics on Overview, the analysis result above the dossier on Analyze, a compact
  pair bar above the diff on Compare, the run's own sections above configuration on Release Gate, and one stored
  entity at a time on Bundle & History. **253 UI tests in the same 9 files** (was 242; eleven structural tests
  added, none removed), **868 Rust unchanged**, 30 registered commands unchanged, `assets/design-tokens.json`
  byte-identical, gate 17/17 with drift 8/8 including baseline integrity. Product commit
  `6a071c51c81a00b7a35facaefa5849e9975e715c`, pushed as a normal fast-forward.
  The second unit, `U1P_INSTALLED_ACCEPTANCE_AFTER_CI_RERUN`, took those exact bytes onto a real screen. Run
  `37668291893` (#79) finished **10 of 10 on attempt 2**: attempt 1 left eight jobs green and cancelled
  `Generated output drift` and `Package Ubuntu` at the six-hour ceiling while each was still on the Linux
  prerequisite `apt-get update`, with no repository step started in either — the one class
  `P5_VALIDATION/P5_CI_AUTHORITY.md` lets a single same-SHA rerun answer, and the owner authorized exactly that
  one rerun. In attempt 2 only those two jobs re-executed; the other eight carry attempt 1's window to the
  second and are not described as newly run. There was no attempt 3. Windows artifact `11504871803` (zip
  `57651a3e…c1bb`, NSIS `f69cda6c…`, installed executable `7327d374…`) was installed with real input after the
  owner's store was parked, and sixteen measured captures were taken — P01–P06 at 1440×900, R01–R06 at 1024×720,
  R07–R10 at 1056×799. All six first-viewport contracts came back PASS, the shell is `SHELL_PASS` proved by
  pixel comparison rather than CSS, and the thirteen-step functional smoke found no S0/S1/S2 regression. The
  owner's store came back byte-exact with all four flags YES and `OWNER_STORE_OPENED_BY_U1P = NO`, the fourteen
  unrelated siblings compare identical, and the machine is uninstalled again as it was before.
  Why the status moved down rather than up: one material finding is open. `U1P-V2-01` is that Overview's ship
  verdict does not name the build it judges while the page's own subject line can name a different one — nothing
  stated is false, but the reader cannot check the pairing from that page, and `App.tsx` keeps the gate run and
  the last analysis as independent state. U1P did not create that ownership; it promoted the verdict to the
  dominant element, which is what made the gap material. It is reported, not self-waived, so §12's gate yields
  `REQUIRES_ARCHITECT_POLISH_REVIEW`. Three lesser items go with it: `U1P-V1-01` (a wrapped band leaves an empty
  filled cell at 1024), `U1P-V1-02` (the Compare result does not survive navigation) and note `U1P-V0-01`. No
  `U1P-V3`, no `U1P-V4`, and U1R's truthfulness holds in both failure captures. Evidence:
  `U1_VALIDATION/U1P_VISUAL_POLISH_PLAN.md`, `U1_VALIDATION/U1P_VISUAL_ACCEPTANCE_REPORT.md`, the raw set under
  `%TEMP%\FirmwareSight-U1P-Visual-Acceptance-20261008T001110Z\`, and the uploadable pack
  `C:\Users\16429\Downloads\FirmwareSight_U1P_Final_Visual_Review.zip` (4,727,314 bytes, SHA-256
  `0f59f4ce…3881b`). This is still **not** visual approval: `PASS_COMPLETE`, `VISUAL_ACCEPTED` and
  `MOCKUP_MATCHED` are not states U1P may issue, and no product code was touched after the captures.
- **U1's status now: `READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT`, and `U1P-V2-01` is `CLOSED_BY_U1P_R1`.** A
  sixth U1 unit ran on 2026-10-08 under *FirmwareSight — U1P-R1 Overview Gate Subject Consistency, Execution
  Prompt v1.0* (delivered as a file, 33,484 bytes, SHA-256 `0263e0fe…e8d024`, archived byte-exact). It closed the
  one material finding the round above left open, and it did so by binding a subject rather than by restyling a
  page: `App` now hands `Overview` the selection handle that earned its last-good analysis — the same handle
  `Analyze` had used for its own pending badge since before this unit — and `Overview` asks whether
  `GateRunDto.snapshotId` is the snapshot it is describing. Four states answer NEUTRAL instead of borrowing an
  verdict: a run that judged another build, a selection never analyzed, a stored run with no analysis in the
  session, and a loaded policy whose fingerprint is not the one a run was judged under. Each names both full
  identities as text rather than truncated-with-title, because a `title` attribute is unreachable by a keyboard
  reader. Only the matching state shows the run's own sentence and counts, and it now prints the judged snapshot
  too. Truncated ids, file names, timestamps, commits, byte sizes and the run's baseline were all rejected as
  substitutes, since any of them can agree while the subjects differ.
  Evidence, all of it on this unit's own CI bytes: head `42f75a7`, run `37753793004` (#81) 10/10 with each job's
  own execution window checked, Windows artifact `11539359554` (zip `45922e90…56d8`, NSIS `dd5c8de8…`, installed
  EXE `3bf541bb…`) installed with real mouse input after the owner's store was parked. Five installed scenarios
  — matched, mismatched, unanalyzed selection, matched again after a new run, and a real policy swap that was
  **EXECUTED rather than recorded unsupported** — plus a 1056×799 mismatch capture, a keyboard-only traversal
  that lands a visible focus ring inside the readiness card, and a five-page regression glance in which U1R's
  stale-capability contract still holds in both the pending and the parse-failure states. Fifteen captures
  (sixteen log lines, one of them a superseded mis-click), every one with its client rectangle measured before
  and after the grab. All thirteen acceptance boxes hold.
  The owner's store came back byte-exact with all four flags YES and `OWNER_STORE_OPENED_BY_U1P_R1 = NO`, the
  seventeen application-data entries compare identical, and the machine is uninstalled again as it was before.
  Two things are stated rather than smoothed. The commit message was amended **once before any push** to replace
  a wrong file count, with the tree identical before and after; and one pre-existing Core behaviour was observed
  during the recheck — `capabilities.git` renders `unknown` on Analyze while the Gate reads real git facts for
  the same snapshot, because `Capabilities::with_git()` has no production caller. It is reported for the
  Architect's judgement, not filed as a new finding and not fixed, because Core is outside this unit's boundary.
  `U1P-V1-01`, `U1P-V1-02` and note `U1P-V0-01` stay open exactly as U1P recorded them; this unit neither fixed
  nor waived them. Evidence: `U1_VALIDATION/U1P_R1_SUBJECT_CONSISTENCY_PLAN.md` (written before the code),
  `U1_VALIDATION/U1P_R1_OVERVIEW_SUBJECT_CORRECTIVE_REPORT.md`, the raw set under
  `%TEMP%\FirmwareSight-U1P-R1-Subject-Recheck-20261008T090319Z\`, and the uploadable pack
  `C:\Users\16429\Downloads\FirmwareSight_U1P_R1_Overview_Subject_Review.zip` (1,381,951 bytes, SHA-256
  `342fb89b…99a6`). This is still **not** visual approval: `PASS_COMPLETE`, `VISUAL_ACCEPTED` and
  `MOCKUP_MATCHED` remain states no UI round may issue, and no product source was touched after the captures.
- **`U1P-V2-02` is `CLOSED_BY_U1P_R2`, and U1 stays `READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT`.** A seventh U1 unit ran
  on 2026-10-08 under *FirmwareSight — U1P-R2, Overview Pending-Selection Evidence Scope + Evidence-ID Consistency
  Audit, Architect execution prompt v1.0* (delivered as a file, 21,194 bytes / 449 lines / 448 CRLF pairs, SHA-256
  `3b6108b6…b6221e`; archived byte-exact, stored blob `aea814d6…`, 20,746 LF-normalised bytes, SHA-256
  `fd8d28aa…a35102`). It moved no capability, no verdict rule and no stored fact. The defect was scope, not
  wording: with a selection the user had not analyzed, the capability band and the four figures were rendered from
  the **retained** analysis while the MAP sentence described the **new** selection — `MAP provided` beside
  `Symbol-level analysis depends on it`. `Overview` now derives every current band from
  `selection.selectionId === analyzedSelectionId`; a pending selection is named as `not analyzed yet`, gets no pill
  and no figure, and the retained work sits under `Previous analysis` inside bands literally named
  `Previous input capabilities` and `Previous key figures`, attributed in audible text (`role="note"`) including the
  same-file-name case. Nothing was hidden and no Unknown became a zero.
  Evidence, all of it on this unit's own CI bytes: product commit `2e2e219`, run `37783219031` 10/10 job by job,
  Windows artifact `11554271914` (zip 5,542,981 bytes `94f16ab5…`, NSIS 3,895,064 bytes `fd2d4935…`, installed EXE
  15,367,680 bytes `196036e2…`, `toolchain.git_commit == 2e2e219`) installed with real input after the owner's store
  was parked. Seven installed scenarios — matched, other-build, pending, pending-without-MAP, parse failure after a
  good result, genuine recovery, and a real policy swap — plus three 1024×720 and one 1056×799 captures, seven
  regression glances and a keyboard-only traversal. Twenty-one capture rows, twenty observation rows, two of them
  kept and labelled VOID after a shared-scroll mis-capture. The Overview test file went from 6 failed / 29 passed
  against `60f10a6`'s unmodified implementation to 35 passed after it; 278 UI tests in 9 files (twelve added, none
  deleted), 868 Rust tests unchanged, gate 17 of 17 with no SKIP, `assets/design-tokens.json` byte-identical, the
  `generate_handler!` list still its same 30 entries, name for name, at both heads. `09_acceptance/ACCEPTANCE_GATE.json`: 20 boxes PASS and
  1 MISMATCH_PROVED.
  The owner's store came back byte-exact — `OWNER_STORE_PARKED`, `OWNER_BACKUP_HASH_MATCH`, `ORIGINAL_DB_RESTORED`,
  `ORIGINAL_DB_SHA_MATCH`, `UNRELATED_SIBLING_STORES_UNTOUCHED` all YES, `OWNER_STORE_OPENED_BY_U1P_R2 = NO` — the
  14 unrelated application-data entries compare identical on kind, size and digest, this round's own disposable store
  was hashed then moved by name rather than deleted, and the uninstaller's delete-the-application-data option was
  read as unticked through `BM_GETCHECK` and never activated.
  Three things are stated rather than smoothed. **One:** the round's §7 identity audit found a demonstrable mismatch
  rather than confirming the prompt's premise — the U1P-R1 report's run A and snapshot B had each been transcribed
  one character short (68 and 148 where a SHA-256 makes 69 and 149 the only well-formed lengths), and the prompt's
  own expected strings carry the same short forms, so document-to-document comparison cannot see it. The two lines
  were corrected with their before/after bytes proved against the committed fixtures and the product's own SQLite
  rows, inside this unit's single docs-only successor; the already-hashed U1P-R1 pack was not rewritten or repacked.
  **Two:** the build-time note that predicted `ERR-PARSE-2002` for the junk input was wrong — the installed product
  answers `ERR-FORMAT-0001` — and the original sentence is quoted unmodified beside the correction. **Three:** both
  earlier delivery archives had left `Downloads` by pack time; a read-only check found them in the Recycle Bin
  re-hashed to the exact digests `BASELINE.yaml` records, so the pack's historical images are the delivered bytes and
  not a regeneration, but whether a moved delivery copy is itself a §13 gap is the Architect's call.
  `U1P-V1-01`, `U1P-V1-02` and note `U1P-V0-01` stay open exactly as U1P recorded them, and the Git
  capability/Gate-rule observation U1P-R1 raised stays with the Architect; this unit neither fixed nor waived any of
  them, and it records one limitation plainly — the destination of Overview's pending primary action is held by
  contract test R2-T5 and mutation `M3_pending_cta`, because no press-then-window-title pair was captured on the
  installed build. Evidence: `U1_VALIDATION/U1P_R2_PENDING_SELECTION_CORRECTIVE_REPORT.md`, the raw set under
  `%TEMP%\FirmwareSight-U1P-R2-Pending-Selection-20261008T132730Z\`, and the single shareable pack
  `C:\Users\16429\Downloads\FirmwareSight_U1P_R2_Final_Visual_Review.zip` (4,113,494 bytes, SHA-256
  `d3674569…1bb34c`, 87 entries, CRC clean, 87/87 round trip, 0 missing / 0 invalid manifest entries). It is the
  second build of that pack: the first (`4,111,505` bytes, `d74f6ee3…e212e7`, 86 entries) carried the same evidence
  with a wrong command count in the report inside it, and was moved aside with an explanatory note rather than
  deleted. This is still
  **not** visual approval: `PASS_COMPLETE`, `VISUAL_ACCEPTED` and `MOCKUP_MATCHED` remain states no UI round may
  issue, and no product source changed after the captures.
- **`U1P-V1-01` and `U1P-V1-02` are `CLOSED_BY_U1P_R3`, and U1 stays `READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT`.** An
  eighth U1 unit ran on 2026-10-08 under *FirmwareSight — U1P-R3 Final Narrow Corrective and Visual Closure,
  Execution Prompt v1.0* (delivered as a file, 32,849 bytes / 745 lines / 0 CR, SHA-256 `a185890e…ab4de9`; archived
  byte-exact at `10_AUDIT/SOURCE_PROMPTS/…v1.0.txt`, blob `384e8ca4…`, the same 32,849 bytes — this prompt arrived
  LF-only, so nothing was normalised away). Start authority `f01eec110c76591c741d283e4ee862566e211a2f`, clean tree;
  that head's run `37801519343` (#84) is 10 of 10, read at closeout and recorded in
  `00_authority/START_AUTHORITY_CI.txt`.
  Two presentation defects were closed and no capability was added. **Compare** kept its result in page state while
  `App.tsx` renders one page at a time, so navigating away unmounted the page and its result with it and the screen
  returned showing its action above nothing: the comparison session is now shell state, six values `App` owns and
  hands down as `session` / `onSessionChange`, writes go through functional updaters so two writes in one render
  cannot clobber each other and an obsolete async answer cannot resurrect a stale pair, a first session with two
  candidates and no result says `Ready to compare` and names the next step, and a new process does not resurrect an
  ephemeral `diffId` — in-process only, no SQLite row, no IPC call, no `localStorage`, no schema change.
  **Overview at 1024** painted a fourth, unfilled band track as a grey rectangle because the shared auto-fit grid
  strokes its gap in the border colour: a local `.fillBand` class fills the wrapped line instead, and
  `components/Panel.tsx` / `Panel.module.css` are **not** in the commit — a shared `fill` prop was implemented,
  reviewed and reverted to its HEAD digest, because §10 prefers a local class and "provably inert on the other four
  pages" is a weaker claim than "not touched".
  Thirteen tests came with it (T1–T9 in `compare.test.tsx`, T10–T12 in `overview.test.tsx`); **291 UI tests in the
  same 9 files** (was 278, none deleted), **868 Rust unchanged** across 47 result lines, gate **17 of 17** with no
  SKIP, drift 8/8 including baseline integrity, deny 1/1, `assets/design-tokens.json` byte-identical, the
  `generate_handler!` list still its same 30 named entries. The RED run against the untouched tree reported seven
  failures (T1, T3, T4, T5, T6, T8, T9) and T3b, written after it, is RED in its own separately recorded run, so
  eight of the thirteen are RED on the old bytes and all thirteen pass at the head; T2 and T7 are recorded as locks
  rather than as RED, and so is the Overview trio — re-measured over HEAD copies of `Overview.tsx`,
  `components/Panel.tsx` and `Panel.module.css`, all 38 tests in that file pass against the unchanged product code,
  because jsdom lays nothing out and a DOM test cannot see an empty band track. An earlier draft of the report
  claimed the trio was RED; that claim was wrong and the report says so. Three mutation records: M2 removes the
  `Ready to compare` panel (T1, T9 fail), M3 hard-wires the `stale` rule to false (4 fail — T5 and T6 plus two
  pre-existing U1P tests, which is the proof the rule is load-bearing), and M1 is not a separate edit at all —
  page-local state *is* `f01eec1`, so the RED run is the retention measurement. Product commit
  `41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177` (12 files, +1,532/−58), pushed `f01eec1..41bb6a3` as a normal
  fast-forward under the owner's written authorization for this round; run `37828673549` (#85, attempt 1) **10 of
  10** job by job. Windows artifact `11573661113` (zip 5,544,133 bytes `330e83af…`, NSIS 3,896,257 bytes `9a51e86a…`,
  installed executable 15,368,192 bytes `5c828ecf…`, `toolchain.git_commit == 41bb6a36`) was downloaded from that run
  — never built locally — and installed with real mouse and keyboard input after the owner's store was parked.
  Twenty-six capture rows and twenty-four observation rows: the named matrix **P01–P10 at 1440×900** and **R01–R10 at
  1024×720 / 1056×799** is complete, each with its requested and measured client size and
  `client_inside_work_area: true` — the harness now refuses a grab whose client bottom falls below the work area,
  which is also the correction of a historical image that carried 33 px of taskbar. One grab was discarded for a
  misclick and ships in the pack labelled discarded, because a pack that keeps only flattering frames is not
  evidence.
  `09_acceptance/ACCEPTANCE_GATE.json`: **25 items — 23 PASS, 1 FAIL, 1 NOT_VERIFIED, 0 NOT_CAPTURED**, 2 of the 23
  PASS items flagged `MISMATCH_PROVED` (the stale policy fingerprint Overview names, the refused same-build compare);
  that flag is not a fifth verdict and not extra success, and the README of the pack prints the arithmetic
  `PASS + FAIL + NOT_VERIFIED + NOT_CAPTURED = ITEMS` because the counts are summed from that JSON at packaging time
  and the builder refuses to finish when README and JSON disagree — including a negative control that alters one
  count and requires the checker to reject it. The one FAIL is a finding this round measured and deliberately did
  not fix: **Analyze's metric band leaves the same unfilled track at 1024** (126 wide rows, and the identical 126 on
  the R2-era image, so it predates this commit and sits outside §7's scope and §10's allowlist). The one
  NOT_VERIFIED is honest coverage: no R3 frame re-photographed Overview's `Previous analysis` band, so the O3 leg for
  it stays unverified rather than folded into a PASS. A third observation, recorded not waived: the gate's baseline
  picker resets on navigation. The §8 read-only trace found **no false Gate PASS** — `git.clean` is the gate's true
  statement about a genuinely clean opened project while Analyze's `Git unknown` is about the artifact — so §8
  produced no STOP; the wrong-source sentence it did surface in `Overview.tsx` was repaired inside the authorized
  scope and is quoted off installed bytes.
  Owner data: `OWNER_STORE_PARKED`, `OWNER_BACKUP_HASH_MATCH`, `ORIGINAL_DB_RESTORED`, `ORIGINAL_DB_SHA_MATCH` and
  `UNRELATED_SIBLING_STORES_UNTOUCHED` all YES, `OWNER_STORE_OPENED_BY_R3 = NO`, 12 unrelated digest-bearing
  application-data files identical across the 17-entry before and after inventories, this round's disposable store
  hashed and moved out whole rather than deleted, the uninstaller's delete-the-application-data box read as unticked
  and never clicked, and the machine uninstalled again as it was found.
  Evidence: `U1_VALIDATION/U1P_R3_NARROW_CORRECTIVE_PLAN.md` (written before the code),
  `U1_VALIDATION/U1P_R3_FINAL_NARROW_CORRECTIVE_REPORT.md`, the raw set under
  `%TEMP%\FirmwareSight-U1P-R3-Narrow-Corrective-20261008T190057Z\`, and the uploadable pack
  `C:\Users\16429\Downloads\FirmwareSight_U1P_R3_Final_Visual_Review.zip` (4,224,650 bytes, SHA-256
  `4de07c76…831e52`, 102 members, CRC clean, 102/102 extraction round trip, 104 checks on the pack all passed, 8
  composites labelled DERIVED over unaltered originals, forbidden-content scan clean). It is the **second** build of
  that pack: the first (4,223,656 bytes, `b2e787cf…4b0ab45`) carried the same evidence with the right counts but
  interleaved the negative control's intended rejection among its own checks, and it was moved to
  `08_pack\superseded_first_build\` in the evidence root rather than deleted. The R2 archive stays untouched at its
  own digest, and its stale `MISMATCH_PROVED=0, PASS=0` README line is documented there as historical only.
  Three governance ordinals were repaired by this unit's docs successor, with the dated narrative around each left
  intact: U1P-R1 is the **sixth** U1 unit (it was called the fourth), U1P-R2 the **seventh** (it was called the
  sixth), and this unit the **eighth**. This is still **not** visual approval: `PASS_COMPLETE`, `VISUAL_ACCEPTED`
  and `MOCKUP_MATCHED` remain states no UI round may issue, no product source was touched after the captures, V1
  stays `IN_PROGRESS` / `RECRUITMENT_READY` at 0 eligible sessions with its frozen F3 cohort artifact untouched, and
  B1 stays not authorized.
- **U1 is now CLOSED: `PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS`, on the Architect's authority, and the
  active-task pointer is `NONE`.** The word arrived on 2026-10-08 as an independent visual acceptance record —
  *FirmwareSight | Architect Independent Visual Acceptance Record*, delivered as
  `C:\Users\16429\Downloads\FirmwareSight_U1P_R3_Architect_Final_Verdict_2026-10-08.txt` and archived at
  `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R3_Architect_Final_Verdict_2026-10-08.txt`: **5,159 bytes, 52 logical
  lines, 0 CR**, delivered SHA-256 `8634321c…36c619`, stored blob `f16f0e7ef1279883d62d909d5161d42ab756d74b` with the
  same length and the same digest, `cmp` clean. It is the first document in that directory that closes a track
  instead of opening one, and it is an **authority record, not an execution prompt** — it authorized no work.
  The scope travels with the verdict and is part of it, not commentary: *"visual productization readiness for a
  controlled V1 external-user study, NOT GA quality, feature completeness, or public distribution approval"*, and
  *"an ARCHITECT waiver of specifically enumerated cosmetic/interaction residuals, not a claim that every Agent
  acceptance box passed."* Eight units ran; four closed on `READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT` and none got
  past it, so every `never_self_issued` list is left exactly as written — the word came from outside them, and the
  record itself says *"R3 Agent did not and could not self-authorize this verdict."*
  What the round that filed it did: re-read both cited runs from GitHub's own API rather than copying them
  (`37828673549` #85 at head `41bb6a36` and `37846189269` #86 at head `4d25c85f`, both completed / success, 10 of 10
  authoritative jobs; the run-list endpoint reports `attempt: null` for both, so the verdict's "attempt 1" is filed
  as its claim while the job count and conclusion are the re-read part), re-hashed the delivered pack where it lies
  (still 4,224,650 bytes, `4de07c76…831e52`), and **independently re-verified the artifact bytes the verdict's
  recommendation turns on before writing them anywhere**: artifact `11573661113` re-downloaded from run #85, container
  5,544,133 bytes `330e83af…4f3fb4` CRC-clean and not expired, NSIS `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`
  **3,896,257 bytes `9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93`** — matching the verdict —
  the container's internal `SHA256SUMS.txt` verified entry by entry and `artifact-metadata.json` reporting
  `toolchain.git_commit = 41bb6a36…`; ten checks, all true, nothing installed or launched. One honest subtraction:
  this endpoint returns `sha256_digest: null` for the artifact, as U1P-R1 found for its own, so the second authority
  is the internal manifest plus the metadata commit rather than a self-referential digest.
  *(Dated erratum, later on 2026-10-08: that reading queried a key these endpoints do not have. GitHub publishes
  **`digest`** = `sha256:<hex>`, and for `11573661113` it equals the container hash the re-freeze round measured —
  so a GitHub-side authority does exist and the sentence above understated it. The subtraction was real; its name was
  wrong. See the re-freeze bullet at the top of this file.)*
  What it did **not** do: the record's last section is titled `NEXT ACTION RECOMMENDATION, NOT EXECUTED` and was
  treated as binding. No cohort re-freeze, no protocol amendment, no touch to `V1_VALIDATION/`, no movement of
  `v1_execution.cohort_build_frozen`, no recruitment. **The F3 artifact stays the frozen cohort build in every
  authority document**; re-verifying newer bytes is evidence about a candidate, not a stage transition.
  *(Dated later on 2026-10-08: the separate authority this sentence waited for then arrived, and the build moved —
  see the re-freeze bullet near the top of this file. The sentence above is kept because it is the closure round's
  own boundary, and that round did execute nothing.)*
  The tally is guarded by name — 25 items, **23 PASS / 1 FAIL / 1 NOT_VERIFIED / 0 NOT_CAPTURED**, 2
  `MISMATCH_PROVED` flags inside the 23 PASS — and the verdict says **"DO NOT rewrite this tally to 25/25 or to
  100%."** Four residuals stay enumerated under its own heading `MUST NOT BE SILENTLY CLOSED`, each with a phrase
  it now forbids: **A** Analyze's metric band at 1024×720 (the round's FAIL box; now a U1 visual known issue and a V1
  UX observation prioritised from the first four participant reports or from layout impact, **not** a mandatory
  pre-V1 engineering gate), **B** Compare's Section Changes first data row below the 1440×900 fold (one-scroll-detail
  accepted for V1 only, not literal satisfaction of the stricter aspiration), **C** the Gate baseline picker resetting
  on navigation (a study usability ambiguity; the persisted `GateRunDto` carries its own `baselineSnapshotId`,
  `snapshotId` and `policySha256`, so the reset rewrites no saved record), **D** Overview's Previous-analysis panel,
  never re-captured in R3 (the round's NOT_VERIFIED box; R2's images are contextual evidence, not a substitute).
  **E** was a defect in this file's own entry documents and the verdict routed it rather than excusing it: the
  ordinal/paragraph-count note may be corrected *with the next already authorized governance update, no stand-alone
  CI-loop commit* — which is this update, and it fixed `.ai/ACTIVE_TASK.md`'s "Two self-corrections" to the three the
  R3 report's §5 actually lists, adding the omitted third (an earlier draft of §4 described the mutations and the RED
  count from memory and got them wrong). Nothing was self-waived and no count moved: **868 Rust across 47 result
  lines / 291 UI in 9 files**, 17-step gate, 30 IPC commands, `assets/design-tokens.json` byte-identical at
  `94336906…14d4`, zero product paths in the diff. Evidence: `U1_VALIDATION/U1P_R3_ARCHITECT_FINAL_VERDICT_RECORD.md`
  (FS-U1P-R3-003), machine state at `BASELINE.yaml` `u1_execution.architect_final_verdict`, raw set under
  `%TEMP%\FirmwareSight-U1-Verdict-Record-20261009T013609Z\`.
  `P5` stays `PASS_COMPLETE`, `G2` stays `PASS`, the product stays **`MVP_CANDIDATE`** at `0.6.0`, V1 stays
  `IN_PROGRESS` / `RECRUITMENT_READY` at **0** eligible external sessions, and B1 / RC / GA / public release /
  signing / notarization / updater / commercial distribution stay `NOT_AUTHORIZED` with the licence
  `PENDING_OWNER_CONFIRMATION`. **L11 stays `CARRIED_FORWARD` — a visual acceptance of the UI is not a research pass,
  and only real sessions close it.** What is blocked now is blocked on a person, not a task: the re-freeze needs owner
  authority, recruitment needs real firmware engineers and human moderation (§47/§48), and any UI follow-up — including
  A and C — needs its own prompt and is prioritised by the Architect from participant evidence.
- **V1 is paused, not closed.** `stage_status` stays `IN_PROGRESS`, `research_state` stays `RECRUITMENT_READY`,
  eligible external sessions stay **0**. **Its cohort build was re-frozen on 2026-10-08**: the bytes a formal
  participant must run are now artifact `11573661113` / head `41bb6a36` / NSIS `3,896,257` B /
  `9a51e86a…c87d93` (`v1_execution.cohort_build_refreeze`), and the F3 artifact `11419727517` at
  `182506f2…63d12` is preserved as the superseded first freeze rather than deleted. `v1_execution.paused_for` names
  U1; nothing else in V1's block was rewritten, because a pause
  suspends the turn and is not a verdict, and a build identity is not a verdict either. Resuming needs a real external
  participant, which is operator work (V1 §47, §48) and no agent action. The rest of this section is the record of how
  V1 opened on 2026-10-06.
- **Active task before U1: `V1_OWN_ARTIFACT_EXTERNAL_VALIDATION`.** Stage **V1** opened on 2026-10-06 under
  *FirmwareSight — V1 Own-artifact External Validation, Execution Prompt v1.0 — Architect Authorized*
  (delivered as a file, SHA-256 `48768ff025e3d6351b7b1d8d593ea8bfb18a7930a0a21a101af3093bbec10a0f`, 46,207
  delivered CRLF bytes, 1,697 CRLF pairs, 1,698 logical lines; archived in `10_AUDIT/SOURCE_PROMPTS/` as Git blob
  `d5e8d4575581bd656b568eed97090e7130a51774`, SHA-256 `c62314fee5032ca5ffdbcfe95f51cf3bdb783e7d55a42a165ea526686690c34e`,
  44,510 LF bytes). State: **`V1 = IN_PROGRESS`, `research_state = RECRUITMENT_READY`, eligible external
  sessions = 0.** It is a **research** track, not a product stage: no feature, no schema, no migration, no
  dependency, no cloud, no account, no telemetry, no updater, no signing, no notarization, no licence choice, no
  pricing, no B1, no RC, no GA (§4, §33). The product under test was, as §1 froze it on 2026-10-06, one build — run
  `37475580080`, head
  `08fdfcb`, artifact id `11419727517`, NSIS installer 3,888,432 bytes, SHA-256
  `182506f213383cfe00865f199fcec4fb17079535e8ea370f954fc15097263d12`, **unsigned**, verified against its own
  `SHA256SUMS.txt` at activation and preserved outside Git. That identification is this bullet's 2026-10-06 record;
  the live cohort build is the one named in the bullet above it.
- **What this round did, and the branch it took.** §5 decides it: with no real eligible external participant and
  no session evidence, an agent writes the **Recruitment Ready pack** and stops — it does not simulate a user. So
  `V1_VALIDATION/` now holds §13's sixteen files (plan, metric contract, participant register, five protocol
  documents, sessions register and template, three analysis registers, two deliverables), every register at
  **zero**, the report a **skeleton**, and no fabricated participant, session, quote, timing or completion.
  `P5 = PASS_COMPLETE`, Productization `ENGINEERING_COMPLETE`, G2 `PASS`, product **`MVP_CANDIDATE`**, narrative
  **FirmwareSight Productized MVP Candidate**, `baseline_version = 0.6.0` — all unchanged, and
  `product.status` / `validation.current_gate` deliberately still carry no V1 term: those strings record verdicts,
  and V1 has earned none (the same rule P5 followed while it was running). Counts and gate held exactly at
  **868 Rust / 225 UI in 8 files, 17-step gate** (§45), because a docs-only round that moved either number would
  be a product change wearing a documentation diff.
- **The P5 closure sentence, and what it did and did not open.** P5 ran from 2026-10-03 and closed
  `PASS_COMPLETE` on 2026-10-06 under *FirmwareSight — P5 Commit F3 Final Governance Closure, Execution Prompt
  v1.0 — Architect Authorized* (SHA-256 `860e0097…64514c`, 36,458 delivered CRLF bytes, archived). The closure
  required §6's re-audit first: every P5 exit criterion re-read, and no required engineering item `BLOCKED`
  (`P5_VALIDATION/P5_EXIT_CHECKLIST.md` §7). F3's §37 ended that round at a STOP and wrote **`active_task = NONE`**,
  and its prompt said plainly that nothing next was authorized by the closure sentence itself — V1, B1, private
  beta, RC, GA, signing, notarization, an updater, a licence choice and any new feature each need a new
  architect decision. That is exactly what happened next and what did not: the architect issued the V1 prompt,
  so V1 is open; **no** prompt has authorized B1, beta, RC, GA, signing, notarization, an updater, a licence or a
  feature, and V1's own §40 says even a full V1 pass does not open B1.
- **What P5 opened with**, kept as the record of the round that ran: *FirmwareSight — P5 Productization,
  Execution Prompt v1.0* (file, SHA-256 `722125f5…71e0ae`, 73,722 bytes, 3,442 lines, archived in
  `10_AUDIT/SOURCE_PROMPTS/`), opened on 2026-10-03 from `d83175a` (HEAD = `origin/main`, tree clean, Run
  `37101619245` 7 of 7). Its §4 required a productization audit before any
  product code, and that audit is written: **`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md`** answers §4 A–H
  from commands run on that tree.
  The owner's checkpoint then answered the four questions the audit reserved for the owner — artifact
  versions **unify on `0.6.0`** with the workspace version as the single source, **migration `0005` only
  after a written `P5_MIGRATION_DECISION.md`**, the package matrix is **Windows with real install evidence
  on this host plus macOS/Ubuntu `CI_BUILD_ONLY`**, and the shipped store **keeps the name
  `firmwaresight-p0.sqlite`** with its path made visible rather than moved. One question stayed another
  person's and one has now been answered. **L22 (release identity versus line endings) is decided**: the
  draft that cost the three answers (`P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md`, option C = today's
  bytes-as-evidence semantics left in place, its premise pinned by
  `a_notes_file_that_differs_only_in_line_endings_is_a_different_release` and a mutation proof) went to the
  Architect and came back as **`ADR-0028`, Accepted: release identity uses the exact bytes observed on
  disk`** — `require_clean_git = false` stays valid with its caveat written down, `.gitattributes` is
  recommended for the *user's* repository, and this repository does not edit its own. The draft is
  re-statused `EXECUTION_RECORD`, not rewritten. **§54 still leaves the licence with the owner**, so
  `OPEN_SOURCE_LICENSE_DECISION = PENDING_OWNER_CONFIRMATION` stands in every P5 document. **What §5 forbade at
  the opening** was writing `P5 PASS`, `BETA`, `RC` or `GA` before closure evidence existed; `PASS_COMPLETE`
  came only from F3 §6's re-audit on 2026-10-06, and `BETA`, `RC`, `GA`, a tag, a GitHub Release, a published
  installer, a signature, a notarization, an updater and a licence remain unwritten — §13 of the F3 prompt
  freezes those ten states and `P5_RELEASE_READINESS.md` §5 carries them.
- **What has landed since that checkpoint**, measured rather than summarized: `4cc8d93` opened governance
  and archived the prompt (Run `37125456689`, 7 of 7); `812b472` wrote `P5_MIGRATION_DECISION.md` then
  migration `0005`, which keeps the two numeric `Unknown` reasons the storage write path had been
  discarding (Run `37127791999`, 7 of 7); `9e3b1de` unified every artifact version on `0.6.0` and
  regenerated the seven goldens it moved (Run `37128593254`, **6 of 7 — failed**); `20b03e3` repaired the
  race that run lost, test-only (Run `37129900728`, 7 of 7); and the packaging commit turned `bundle.active`
  on, added the three CI package jobs prompt §41 asks for, wrote `scripts/verify_package_artifacts.py` and
  gave the gate a `drift/version identity` step (Run `37133706214`, **7 of 10 — failed**: the seven gate
  jobs were green and all three package jobs skipped their own build). The tree reached **775 Rust tests /
  160 UI tests in 6 files / `check.py` 16 steps** at that point, and the authoritative CI set is **10 jobs**
  for the length of P5 — recorded in `05_ENGINEERING/06_CI_CD_BASELINE.md` and tracked run by run in
  `P5_VALIDATION/P5_CI_AUTHORITY.md`.
- **The first 10-job run went red because the gate reported a pass it had not earned.** `cargo install
  tauri-cli` leaves a binary named `cargo-tauri`, which is run as `cargo tauri`; the package group probed
  the bare name `tauri`, found nothing on the three runners that had installed the CLI one step earlier,
  printed `SKIPPED: the Tauri CLI is not installed on this machine`, summarised `4/4 steps passed` and
  exited 0 — and each job was then caught red by its own artifact upload. Two rules replace the instance
  fix: the group probes the forms in the order the install methods produce them and runs the one that
  answered, and `check.py` now records a skip as `SKIP`, leaves it out of the passed total, and exits
  non-zero on any skip when `CI` is set. Measured on this host three ways, in `P5_VALIDATION/
  P5_PACKAGING_REPORT.md` §5.
- **Packaging is proved on all three platforms, and the index defect that read-back found is now closed by
  its own read-back.** Run `37138881977` at head `1055242` is the first **10 of 10** green run and the first
  to attach a built package: the three §42-named sets — Windows NSIS, Ubuntu `.deb`, macOS `.app` + `.dmg` —
  each with its own `SHA256SUMS.txt` and `artifact-metadata.json`. They were downloaded with `gh run download`
  and read back, which closed the rows that had only ever been read from upstream source: the macOS runner
  derives an `.icns` from the five committed PNGs, the executable inside `Contents/MacOS` is the Cargo bin name
  (`firmwaresight-desktop`, exactly as the source said), the `.deb` really is unpacked by `dpkg-deb -x`, and
  all three jobs build from `apps/desktop`. That read-back also caught a defect of this repository's own: the
  darwin index gave the `.app` one line naming a directory, so `sha256sum -c` answered
  `FAILED open or read` on a good build. A directory bundle now contributes one line per file, with the
  aggregate tree digest kept in the metadata as identity and each scanned payload's digest recorded — and
  run `37143046338` at head `53578e9`, again **10 of 10**, said it back: five darwin index lines, every one
  `OK`, exit 0, with one flipped byte in a copy of `Contents/Info.plist` making the same index exit 1 and name
  that file. Comparing the two runs, whose heads differ only in `scripts/` and docs, produced a fact nobody
  predicted: the macOS `.app` tree digest is identical across both, while the `.dmg` wrapping it — same
  length — is not. A repeating digest is one pair of runs on one runner image and licenses no reproducibility
  claim; a changing one is not evidence the program changed. The installers for §38 exist on both this host
  and the runner, and the Windows one has now been run here — see the next bullet.
- **The first real install ran on this machine, and it produced three findings that no source read could.**
  Windows Sandbox is absent here (`WindowsSandbox.exe` missing, the feature query needs elevation, which this
  round does not take), so prompt §38's fallback ran: the owner's live store was hashed, parked, and restored
  to three byte-identical digests, and the packaged installer was driven through its own UI. Install, launch
  from the Start Menu shortcut, Analyze of a real fixture, close/reopen, repair, uninstall, reinstall and the
  data-retention check are all transcribed in `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md` with §39's record
  (per-user install path, no UAC prompt, `PATH` entries mentioning FirmwareSight: 0 before and after, no shell
  profile, no TCP endpoint owned by the app, WebView origin `http://tauri.localhost/`). What it found: the
  window title still reads `FirmwareSight - Analyze` while the page is Compare — L21 reproduced on the
  packaged artifact; a returning user sees nothing they did before, because the store kept 1 build, 19
  sections, 42 symbols and 10 evidence rows across a reopen while the UI says "Nothing has been analyzed in
  this session yet"; and the uninstaller never mentions user data, deletes none, and the reinstall reopened
  the same store without re-running the migrations — a retention rule that must now be written where a user
  reads it. §38 item C (first-run onboarding) was not built when that round ran, so §64's full journey stayed
  open until the next bullet, and §65's negative half stays open because this host has the whole toolchain
  installed.
- **Commit C closed §13, §14 and §15–§18 in the code, and closed L21 with them.** First-run guidance is one
  component read by two surfaces (`GettingStarted.tsx`: the seven answers as a list, plus a dismissible panel
  Analyze shows in its empty state), Help is a page whose identity block is asked from the running binary
  rather than kept in the front end, and History is a fourth rail page over three new bounded storage reads —
  `list_history_builds`, `list_history_gate_runs`, `list_history_releases` — with **no migration added**,
  which is what §18 asked for first. The window title the install round found frozen is fixed on the Rust
  side: `set_window_title(MainWindowPage)` composes the text from a closed five-variant enum, because
  granting the WebView `core:window:allow-set-title` would be a capability change (`AGENTS.md` 9) and a
  channel from a file name into a window property; `the_title_fix_took_no_new_capability` reads
  `capabilities/main.json` and asserts the window still holds exactly `["core:default"]`. Two decisions were
  made from evidence rather than habit: a History filter searches **stored identity columns only**, because
  matching `artifacts.path` would turn the search box into a directory oracle (the test that seeds real builds
  into a real temporary directory asserts filtering by that directory's name returns zero rows), and Help
  names the store **file** and not its folder, because the path is a Diagnostics question with its own
  allowlist and Diagnostics is not in this commit. Measured: `python scripts/check.py` **16 of 16**, the tree
  now **812 Rust tests / 200 UI tests in 8 files** (19 storage + 11 desktop + 24 History UI + 16 Help UI),
  and the page reads 100 builds / 100 runs / 50 releases in 467.9µs / 541.5µs / 248.8µs on the release
  profile. Seven mutation proofs are in `P5_VALIDATION/P5_ONBOARDING_HISTORY_REPORT.md` §5 and the design
  review, including the one disclosed non-token measurement, in
  `P5_ONBOARDING_HISTORY_DESIGN_CHECKLIST.md`. **What this does not close:** the page has not been operated
  in an installed binary — §38 C's onboarding and the whole §64 journey still have to be walked against the
  package, which is the remaining install acceptance. The remote said 10 of 10 on the first attempt for
  this head (Run `37154946484` on `e863d0c`), so the new page's UI tests and the rail guards have run on
  both Windows and Ubuntu runners.
- **One gate run went red for a reason that was not in the commit it ran on**, and that is worth knowing
  before the next stage trusts a green. Run `37128593254` failed `Desktop UI (windows-latest)` on a
  pre-existing race in `compare.test.tsx:1094`: the `Section Changes` region mounts before its first page
  does (`Compare.tsx:1001-1004` clears the page, the `<table>` renders only from a resolved one), so a
  synchronous `getAllByRole('columnheader')` after `findByRole('region')` waits for nothing. It reproduced
  on this host without CI (1 of 30 fresh runs) and 3 of 3 under 25 ms of injected mock latency; the repair
  is one awaited query, and a never-resolving query keeps the repaired test red, so the wait is
  load-bearing rather than a widened timeout. `055b54e` had closed the identical shape at this file's pager
  and said the rest was "not converted then" — this is the third such close (`e83950c` was the first). The
  remaining instances are a sweep for P5's validation work, not an excuse to rewrite unrelated tests.
- **The sweep happened, and the class produced one more instance first.** Eight commits after `20b03e3`, on a
  tree whose UI source was identical to a head CI had passed 10 of 10 twice, the **local** full gate went red at
  `frontend/test`: L23's sixth instance, in the same file. `compare.test.tsx` awaited the ranking region and then
  read the *added rows* synchronously, though those come from a second IPC wave that commits on its own
  (`Compare.tsx:217-256`, with its own `Reading the added rows…` branch at `Compare.tsx:876-877`). Three full-suite
  runs on an idle host passed; the gate, loaded by its own compilation, did not — which is why the record says the
  wait is load-bearing rather than the reproduction being the proof. Repaired test-only (one awaited query) plus a
  contract test that holds that wave open; three mutation proofs; 20 clean fresh-process runs of the changed suite;
  `Compare.tsx` untouched. §0a of the audit now carries the bounded sweep of chained IPC waves that §0a had asked
  for, the five synchronous top-level reads that remain after it, and the sweep's own limits written next to it:
  Release's `run`-keyed waves were not walked assertion by assertion, `within(region)` reads inside already-awaited
  regions were not audited, and the repetitions cover the changed suite rather than every suite — so L23 stays
  `SHOULD_CLOSE_P5` and the complete sweep is a named remaining task. Present counts on that tree: `check.py`
  **16 of 16**, **813 Rust / 201 UI in 8 files** — and the remote agreed: Run `37158606478` on `111fe32` came back
  **10 of 10** at its first attempt, with `Desktop UI` green on **both** Windows and Ubuntu runners, which is where
  an awaited second-wave query earns its keep.
- **Commit D landed the recovery half, and the installed walk is what made it honest.** Storage now owns its
  own health: `integrity_check()` asks the engine and reports, and repairs nothing — no `PRAGMA` write, no
  VACUUM, no rebuild, which is what lets the Help screen say a damaged store will not be touched. Any older
  file-backed store is now snapshotted with SQLite's **online backup API** before it is migrated (a plain
  file copy is wrong under WAL and `VACUUM INTO` binds its filename as text, so a non-UTF-8 path fails or
  names the wrong file), the snapshot is opened and health-checked before it is renamed into place, and a
  snapshot that cannot be written stops the upgrade rather than starting it. Diagnostics is a closed
  41-key allowlist assembled in Rust and exported through a native dialog that opens only when a person
  names a folder; the exported file on the packaged build contains **no `/` and no `\` character at all**,
  and the store is named by file, never by directory, on screen and in the file.
- **What that walk found is the reason it was run.** Two defects, neither reachable from a unit test:
  the startup refusal exited **101 through a Tauri panic** instead of the typed exit 1, because a `setup`
  error is panicked by the framework inside its event-loop callback
  (`tauri-2.12.0/src/app.rs:1443-1445`) and the `Error::Setup` arm written for it was unreachable — and a
  file that is not a database was being described as a rolled-back migration to "schema version 0", a step
  that never ran. Both are fixed in this commit, the second with the seventh mutation proof. It also
  falsified two sentences this repository had written for itself: a `setup` failure does *not* return
  before the event loop starts, and a window *is* mapped — measured visible from t=20 ms until the process
  leaves at t=360 ms — so §22's stop is now described as it measures (a window with nothing true to say)
  rather than as it was reasoned (no window at all). And `payload_sha256` never was the digest of the file
  a user launches: the bundler overwrites 3 bytes of it to name the bundle type
  (`tauri-bundler-2.10.1/src/bundle.rs:41-95`), which is why the installed binary reports
  `installChannel: "nsis"`. Present counts: `check.py` **16 of 16**, package group **4 of 4**,
  **854 Rust / 210 UI in 8 files**. The owner's store was parked, backed up on a second volume and restored
  byte-identically at both cycles; `ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`.
- **The remote found a third, and it was in Commit D's own test.** Run `37200245520` on the product head came
  back **8 of 10**: `macOS Core Smoke` and `Rust (ubuntu-latest)` each panicked on one assertion in the
  snapshot-retention test this commit added, which compared two whole snapshot files whose only available
  difference was the **second-granular** `applied_at` default on `schema_migrations` — a clock, not a fact. The
  same test was green on this host and on `Rust (windows-latest)`, so nothing about the product was in
  question and all three package jobs passed on that head. Rejected: a sleep, a retry, and deleting the claim;
  the repair marks each store with a named row and reads that row back out of the standing snapshot, proven
  non-vacuous by an eighth mutation (`backup.rs` keeping a stale snapshot reddens it alone), then 20
  fresh-process runs of the repaired test and a local `16 of 16` + `3 of 3` core-smoke. The failed head is
  listed in `P5_CI_AUTHORITY.md` rather than replaced. Both repair heads have since been read back and both came
  home **10 of 10** on their first attempt — `bccea88` on Run `37202016141`, the record commit `90aa69d` on Run
  `37202301591` — with the two runners that lost the race verified green on the repaired test, and §8 of
  `P5_DIAGNOSTICS_RECOVERY_REPORT.md` is the read-back for the chain.
- **Commit E widened the cohort with bytes a real toolchain produced, and those bytes found a product defect.**
  It ran under its own architect prompt — *FirmwareSight P5 Commit E — Compatibility Fixtures, Support Matrix &
  Supportability Closure, v1.0* (SHA-256 `030ca282…ba21148`, 53,915 bytes, 2,459 lines, archived and registered
  in `10_AUDIT/SOURCE_PROMPTS/README.md` as the current active authority), which is the answer Commit D's §34 STOP
  was waiting for and which authorizes exactly three things: fixtures, the matrix, and the §48 dispositions. Six
  new fixture directories and 31 new files now sit under `fixtures/elf/p5-compat/` (`p5-ram-exec`, `p5-extsram`,
  `p5-dma-region`, `p5-long-preamble`, `p5-no-debug` with two ELFs, `p5-clang-arm`), the manifest went from 25 to
  56 entries, and every file was written by `scripts/gen_p5_compat_fixtures.py` from this host's
  `arm-none-eabi-gcc 14.3.1` and `clang 22.1.8`, both final-linked by GNU `ld 2.44.0.20250616` — no hand-edited
  evidence, and the generator reproduces the committed ELFs and MAPs byte-identically on a second run
  (`identical = 13`, no binary byte moved by the metadata extension). The 14 tests in
  `crates/firmwaresight-artifact/tests/p5_compat_fixtures.rs` re-derive each fixture's expected image and live-RAM
  totals from `readelf` section headers plus the MAP's own region attributes rather than from the model's
  arithmetic, and holding the product to that independent rule is what exposed the undercount: clang emits
  `.ARM.exidx.text.main` with `sh_type` `0x70000001`, the parser mapped an unrecognised kind to "no role" and then
  to "not allocated", so 8 real FLASH bytes entered neither budget while the total still printed `Exact`. It is
  the only committed ELF whose allocated payload sits outside `SHT_PROGBITS` — ten of eleven were already agreeing,
  which is why the cohort could not see it. Fixed the narrow way the owner approved: `map_section_kind` now reads
  `SHF_ALLOC` from the section header, which is what `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §3 and §4 item 3
  already require, with a fixture-wide invariant test and five mutation proofs (A, B, C, E, F).
- **Commit E's other half was the §48 disposition sweep, and it changed no product contract.** L14, L16, L19, L20
  and L24 are `CLOSED` on measurement — L16 by the package group running **4 of 4** with no `SKIP` on this host,
  L19 by each page naming the attribution question it actually answers, L20 by a closed display mapping so Compare
  no longer prints a Core enum word and an `Unrecognized evidence basis` string for anything outside it — while L4,
  L5, L17 and L23 are `REDUCED` with their residue named, L25 is `NOT_REPRODUCED` (30 legitimate Apply
  activations across both tables, 6 more through form submit), L8 and L12 are `CARRIED_FORWARD` with what was
  measured about them, and **L15 was stopped for the Architect** under §42: the prompt asks for a decision about a
  serialized public contract, and the label turns out to be written in three different namespaces (`elf.program-header`
  in the JSON Schema, the same in the report DTO, `ElfProgramHeader` in the stored `evidence.source_type`, plus the
  bundle's `analysis.json`), so `P5_COMMIT_E_SCHEMA_DECISION.md` costs four options, asks four questions and
  STOPS — which is where this bullet stopped, and where the closure normalization round below picks it up.
  L26 is untouched by §1 and stays Commit F's decision. Present counts at the tree this bullet describes:
  `cargo test --workspace` **868 passed / 0 failed across 47 targets**, UI **217 in 8 files**, `check.py`
  **16 of 16**, `--only core-smoke` 3/3, `--only drift` 7/7, `--only deny` 1/1, `--only package` **4 of 4**, and the
  UI reliability campaign **20 runs × 217 tests with 0 failing runs**. The one UI change was test-only
  (`history.test.tsx`'s macrotask sleep replaced by a microtask flush, proven still load-bearing by mutation F).
  `P5_COMPATIBILITY_MATRIX.md` was rewritten from the tree rather than from prose: ten GCC-built ELF files, one
  clang-built, nine MAP files across four distinct region shapes. **Not closed, and not claimed:** no P5 verdict,
  no `SUPPORTED` row that was not measured, no §64 journey, no L26 fix, no general source-control linter.
- **Commit E's remote read-back, and the one thing it does not cover.** The round went up as one push of two
  commits, so GitHub started one run: `37228929762` at head `59d85c3` (E2, whose tree contains E1), attempt 1,
  `completed` / `success`, **10 of 10**, with all ten job names and every step read — the only `skipped` step in
  the run is the conditional Ubuntu apt block on the Windows runner, which is the workflow's design. The E1 head
  `859648e` has no run of its own, and that is recorded as an absence rather than given an invented number.
  A clean detached worktree at the candidate SHA checked the same bytes a second way: 56 manifest paths all
  present, tracked and hash-matching with no generator run, the 14 fixture tests green with
  `arm-none-eabi-gcc`, `clang`, `arm-none-eabi-readelf` and `ld.lld` all absent from `PATH`, and the gate
  **18 of 18** (the same 16 steps plus the two cold-checkout asset steps). Evidence:
  `P5_COMPATIBILITY_FIXTURE_REPORT.md` §7a and `P5_VALIDATION/P5_CI_AUTHORITY.md`.
- **Commit E's evidence vocabulary was then normalized, and its one open question was answered — both in
  documents, with zero product source.** *P5 Commit E Closure Normalization v1.0* (SHA-256 of the delivered
  bytes `9571df2c…075599`, 28,738 bytes CRLF; stored under `10_AUDIT/SOURCE_PROMPTS/` as 27,366 bytes / 1,372
  LF with hash `62040e3d…e6c7af887`, the difference being only the terminator `.gitattributes` requires for
  `*.txt`, and line-by-line identity proven rather than asserted) authorizes four things and nothing else:
  the matrix's **status column** now uses exactly `SUPPORTED / SUPPORTED_WITH_LIMITS / CI_BUILD_ONLY /
  NOT_TESTED / UNSUPPORTED` — a scratch validator over every table's status cell reports **38 cells checked, 0
  violations**, and what the old non-canonical words (`BEST_EFFORT_NO_CLAIM`, `DEFERRED_NO_MVP`, `NOT_CLAIMED`,
  `MEASURED, NOT INFERRED`, `BUILT_AND_VERIFIED_IN_CI`) meant moved into the evidence column unchanged, while
  signing/notarization/update readiness moved out of a status column entirely because `READY_NOT_EXECUTED` is
  not a compatibility claim; the A–H **disposition column** now uses exactly
  `PROVED_BY_EXISTING_FIXTURE / PROVED_BY_NEW_FIXTURE / SUPPORTED_WITH_LIMITS / NOT_AVAILABLE`, with A, F and H
  recorded as **existing** because a new assertion on a pre-existing fixture is not a new fixture; and **L15 is
  decided**. The Architect's answer is **Option E — legacy wire identifier preserved, accurate presentation /
  documentation**: `SourceType::ElfProgramHeader` and the `analysis:1` token `"elf.program-header"` stay, so no
  enum rename, no wire rename, no rewritten history, **no migration 0006, no `analysis:2`**, no golden byte and
  no Bundle change occurs, and what was inaccurate becomes documented — the token is a legacy compatibility
  identifier, and the accurate meaning of `MemoryEvidenceBasis::ElfAddressAndFlags` is **ELF address + flags
  evidence** (`04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §7 now says so next to the precedence rule it names
  loosely; `P5_COMMIT_E_SCHEMA_DECISION.md` is `RESOLVED_BY_ARCHITECT` with its §1–§10 pricing left exactly as
  Commit E wrote it). L15 is therefore `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER` and **not `CLOSED`**: the
  name remains wrong, the risk is now bounded and documented. One visible residue was recorded rather than
  fixed, because that round may not touch code: the Evidence Inspector still rendered the stored token verbatim
  (`Details.tsx:667` ← `details.rs:223` ← `query.rs:503` ← `db.rs:507`), so a user expands such a row and reads
  `ElfProgramHeader`; a display-only caption in L20's closed-table shape was Commit F's to make — **and F1 made
  it**, in the bullet below, which is why that chain now ends at a caption instead of at a raw enum name.
  §18 fixed what could be written about the outcome while that head's own run was still unread: Commit E
  engineering **PASS**, evidence closure **`NORMALIZATION_PENDING_REMOTE_CI`**, `P5 = IN_PROGRESS` *(that head's
  state; F3 closed the stage on 2026-10-06)*, product
  **MVP CANDIDATE**, baseline `0.6.0`, Commit F **`NOT_AUTHORIZED`** — and no `P5 PASS`, `BETA`, `RC` or `GA`
  anywhere. The remote has since answered, and this head is the one allowed to record it: Run
  `37262147348` at `80b47c4` went **9 of 10 on attempt 1** (the single red was `Generated output drift`, killed
  by Ubuntu-runner rustup provisioning before it compiled anything, corroborated and then rerun once) and
  **10 of 10 success on attempt 2**, every job and every step read. **COMMIT_E = FINAL PASS / COMPLETE**, and
  Commit F is authorized by the Architect's next prompt, not by this file deciding it.
- **Commit F is authorized, and its first layer F1 lands on 2026-10-05** under *FirmwareSight — P5 Commit F
  Final Productization Closure, Execution Prompt v1.0 — Architect Reviewed* (delivered 68,315 bytes, SHA-256
  `887f2e9d…962fe9ca`, 3,159 lines; archived byte-identically at
  `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitF_Final_Productization_Closure_v1.0.txt` — this prompt arrived
  LF-only, so unlike Commit E's closure prompt the delivered and stored hashes are the same value and there is
  no transport divergence to explain). F1 is the product/tooling layer and nothing else: ADR-0029, the two
  baseline scripts, one new drift step, L15's display caption and its test, the design record
  `P5_VALIDATION/P5_COMMIT_F_DESIGN.md` (written before the implementation, as §19 requires), and narrow
  documentation. **F2 — the exact CI-built Windows artifact, the owner-store park, the installed migration
  proof and the full §64 journey — is sequenced behind an explicit owner confirmation taken 2026-10-04, not
  behind a silent assumption**; the owner chose *land F1, then confirm*, so the live machine is untouched until
  F1 is green remotely and the owner says go. F3 (`P5 = PASS_COMPLETE`) is not written by this commit and cannot
  be written at all unless §62's ready-for-closure gate passes first.
  Counts at the F1 tree: `cargo test --workspace` **868 passed / 0 failed across 47 targets** (unchanged — F1
  adds no Rust test), UI **219 passed in 8 files** (217 → 219, the two new L15 inspector cases), full gate
  **17 of 17** (16 → 17 and drift 7 → 8 from the new `drift/baseline integrity` step; `--only core-smoke` 3/3,
  `--only deny` 1/1, `--only package` 4/4), `DIRECTORY_TREE.txt` **813 lines** and `SHA256SUMS` **693 entries**
  over 695 tracked paths. Every one of those is the printed value, not the expected one: §22 forbids forcing
  the expected numbers, and the expected direction was "drift likely becomes 8, full gate likely 17".
- **L26 was decided by ADR-0029 and implemented in Commit F1, landing 2026-10-05: the root `SHA256SUMS` is now the
  SHA-256 of canonical Git stage-0 index blob bytes, not of working-directory bytes.** The limitation it closes
  was measured, not remembered: `core.autocrlf` is `true` at both system and global scope here,
  `.gitattributes` declares `text eol=lf`, and the manifest hashed the working copy while taking its path set
  from the index — so the number of entries that disagreed with a clean checkout moved
  **19 → 19 → 14 → 14 → 17 → 13 → 13** across the seven heads from `3400981` to `80b47c4` without anyone deciding
  anything. The "17" quoted by `INDEX.md`, this file, `.ai/HANDOFF.md` and four P5 reports was the value true
  at `859648e`; at `80b47c4` the measured value is 13, and those documents are corrected rather than rewritten,
  because each was accurate about the tree it described. What changed in F1:
  `scripts/generate_baseline_artifacts.py` hashes index blobs fetched through one batched
  `git cat-file --batch` (and refuses to write while Git reports an unstaged semantic change to a tracked
  path, so a forgotten `git add` can no longer certify stale content), `scripts/verify_baseline_artifacts.py`
  re-derives the same content through a different Git path (`:<path>`) with no shared helper, and
  `python scripts/check.py --only drift` now runs the verifier as `drift/baseline integrity` on every
  authoritative CI run — the defect had survived five stages precisely because no machine other than the
  writing host ever contradicted it. **A consequence to know before typing the old command: `sha256sum -c
  SHA256SUMS` against a working tree is no longer a valid check and reports those 13 files as failures on this
  host by design.** The canonical checks are `python scripts/verify_baseline_artifacts.py` and, for an external
  proof, `git archive <commit>` extracted outside the repository followed by `sha256sum -c SHA256SUMS` over the
  extracted tree, which carries canonical blob content. ADR-0028 is untouched: release identity still means the
  exact bytes observed on disk, and three checksum domains — repository baseline, distribution package
  `SHA256SUMS.txt`, Release Bundle `SHA256SUMS` — stay separate on purpose.
- **F1 also closed L15's user-facing residue, presentation only.** The Evidence Inspector no longer prints the
  stored legacy identifier: `apps/desktop/ui/src/Details.tsx` maps both namespaces of it
  (`ElfProgramHeader` from storage, `elf.program-header` from the wire) to **`ELF address + flags evidence`**,
  a value outside the table still renders verbatim, and `details.test.tsx` proves the caption while asserting
  the row object still carries the identifier it was stored under. Nothing was renamed: no enum member, no wire
  token, no schema, no `SCHEMA_VERSION` move, no migration 0006, no `analysis:2`, no golden byte.
  **L15 itself therefore stays `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`**; what moved is the layer beneath it,
  recorded as *user-facing presentation residue: CLOSED*.
- **Commit F2 lands on 2026-10-05 and it is the installed-evidence layer, not a product layer.** Authorized
  by *FirmwareSight — P5 Commit F2 Installed Productization Acceptance, Continuation Prompt v1.0 — Architect
  Authorized*, whose own scope line reads "本 Prompt 只解除 F1 结束后的人工暂停，并授权 F2。不授权 F3。" **F2
  changed no product, tooling, dependency, capability, schema, fixture or golden byte** — §3 and §35 forbid it,
  because any such change would invalidate the F1 installer as the candidate under test. Three product
  findings were recorded and deliberately **not patched** (§27's "do not patch silently"), which means the F1
  artifact is still the artifact that was accepted.
  What it proved: the exact CI-built Windows package — **artifact id `11337963032` of run `37293381181`**,
  3,885,631 bytes, SHA-256 `a1152ef3…3c076b`, installed as downloaded with no local rebuild — was walked
  through the **whole §64 journey in one contiguous pass**: install, first-run onboarding, Analyze ×2,
  Compare, Gate, Bundle plus relocated verification, History, a source-independent reopen after the firmware
  folder was renamed, Diagnostics export and its privacy assertions, close/reopen, repair over a running app,
  uninstall, reinstall, retained-data behaviour, a final uninstall, and the owner's store restored. Every
  action was a real `SendInput` event; SQL ran only read-only, to check what the screen had already shown;
  no WebView2 debugging protocol and no synthetic UIA `Invoke` were used.
  The installed migration is **one path, synthetic v4 → v5, on 34 measured checks**, and the boundary is
  written where it could mislead rather than implied: the owner's real store turned out to be at **schema
  v2**, so the chained v2 → v5 upgrade a returning pre-P5 user would receive has never been executed by an
  installed build. That fact also proves the barrier held — had the installed candidate ever opened the
  owner's store, migrations 0003–0005 would have run and it would be at v5.
  Owner-data discipline was applied **twice**, because §24 needed a second installed cycle the first pass had
  not fully covered: park, hash, verify the copy, move, confirm the live path is clear; on the way back,
  verify the parked copy against the original digests before moving and again after. `d6e41034…` /
  `e3b0c442…` / `fd4c9fda…` both times, `sibling stores lost: 0` both times, and
  **`ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`, `OWNER_STORE_OPENED_BY_F1 = NO` were all
  established before any commit was written**, as §26 requires.
  Counts at the F2 tree, printed rather than expected: `cargo test --workspace` **868 passed / 0 failed
  across 47 targets**, UI **219 passed in 8 files**, full gate **17 of 17**, `--only drift` 8/8, `--only deny`
  1/1, `--only core-smoke` 3/3, `--only package` 4/4. **§37 says to STOP if the Rust or UI count moves on a
  docs-only F2; neither moved**, which is the check that shows the diff really is documentation.
  Verdict, as F2 wrote it (its `P5 = IN_PROGRESS` and `F3 = READY_FOR_ARCHITECT_REVIEW` are that head's state,
  not the present one — see the F3 bullet further down this list): **F2 = COMPLETE, P5 = IN_PROGRESS, F3 = READY_FOR_ARCHITECT_REVIEW.** No `P5 PASS_COMPLETE`, no
  `active_task NONE`, no tag, no GitHub Release, no signature, no notarization, no updater, no licence choice
  — §34's states are recorded in `P5_VALIDATION/P5_RELEASE_READINESS.md` and the F2 prompt returns the round
  to the Architect rather than forwarding it.
- **Commit F3 lands on 2026-10-06 and closes the stage. It is governance, evidence and indexing only: no
  product byte, no install, no owner data.** Authorized by *FirmwareSight — P5 Commit F3 Final Governance
  Closure, Execution Prompt v1.0 — Architect Authorized* (delivered as a file, SHA-256 `860e0097…64514c`,
  36,458 bytes / 1,821 CRLF lines; the stored Git blob is `6cacd10e…d749c66` at 34,637 bytes / 1,821 LF lines,
  and the two differ by exactly the 1,821 carriage returns, with every line proven identical one by one —
  `.gitattributes` was not touched to make the hashes agree). Before writing anything, F3 re-ran §6's exit
  re-audit over the nineteen areas §6 names and found **no required engineering item `BLOCKED`**
  (`P5_EXIT_CHECKLIST.md` §7). It then wrote the canonical state — `P5 = PASS_COMPLETE`, Productization
  `ENGINEERING_COMPLETE`, `active_task = NONE`, product **MVP_CANDIDATE**, narrative **FirmwareSight Productized
  MVP Candidate**, baseline `0.6.0`, G2 `PASS`, P0–P4 untouched — in `BASELINE.yaml` and the entry documents,
  using only the vocabulary that file already had (`stage_status`, `closed_on`, `tests_at_close`,
  `gates_at_close`, `product_state`, `next_stage_after_this_one`), and it invented no machine enum for the
  run it could not yet read.
  **One correction it owed the record:** `P5_EXIT_CHECKLIST.md` asserted that every L1–L26 row was
  accounted for, and counting `P5_KNOWN_LIMITATIONS.md` at `a5ce7c4` found **25 of 26 — L12 had no row**,
  although the fact it covers was carried honestly in two other documents. F3 added the row and corrected the
  assertion instead of deleting the claim, and it left every carried limitation carried: **no limitation
  became `CLOSED` because P5 closed** (§11). L15 keeps its two halves (presentation `CLOSED`, wire
  `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`, no `analysis:2`), L26 is `CLOSED — ADR-0029` kept deliberately
  distinct from `ADR-0028`, L20's sequence (E `CLOSED` on Compare → F2 `REOPENED_BY_F2` on Release → F2R
  `CLOSED_BY_F2R`) is reconciled with dates rather than rewritten, and §12's measured performance truth —
  519,179,252 bytes, ~2,020,073 symbols, warm ~4.73–4.89 s, cold ~68.7 s, ~1,428 MB working set, never "Not
  Responding" — is unchanged, with near-500-MiB UI `MEASURED`, first-use-under-60 s **not proved at this
  workload**, and no claim anywhere that performance was optimized, that a memory target was met or that every large file finishes inside a stated window.
  What F3 did **not** do: install anything (§24 — installed authority stays F2R1's `fb5f628` / run
  `37431977428` / artifact `11397938806`), touch the owner's app-data store, change a test count (§5 requires
  **868 Rust / 225 UI in 8 files** to stand still and they did), sign, notarize, tag, release, publish, enable
  an updater, choose a licence, or write its own CI run. Inside this commit `F3 remote CI =
  PENDING_EXTERNAL_EVIDENCE`, and §22 says the quiet half of the verdict: **final external acceptance requires
  F3 remote CI.** §34 forbids a further commit written only to record it. The record is
  `P5_VALIDATION/P5_FINAL_CLOSURE_REPORT.md`; the lineage with every red and interrupted run intact is
  `P5_EXECUTION_REPORT.md` §7; the row-level CI authority is `P5_CI_AUTHORITY.md`.
  **And it stops there.** V1 `NOT AUTHORIZED BY P5`, B1 / private beta `NOT AUTHORIZED`, no G3, no RC, no GA,
  no new feature. `active_task` is `NONE`, which under `AGENTS.md` 1 means no agent may pick the next track.
- **Commit F2R2 lands on 2026-10-06 and it is the read-back head: F2R1's three findings are now closed on a
  real machine, and this head contains no product code.** The §26 artifact is run `37431977428`'s own Windows
  package — artifact id `11397938806`, installer `FirmwareSight_0.6.0_x64-setup.exe` 3,886,598 B
  `efbc45a3…d5fd4`, installed `firmwaresight-desktop.exe` 15,362,048 B `2cf01a6d…670b`, verified against the
  set's own `SHA256SUMS.txt` before it was run; F1's `11337963032`, F2's binary and any local
  `cargo`/`tauri` build were all excluded by §26 and were not used. The focused revalidation ran with the
  owner's store parked (`OWNER_STORE_PARKED = YES`, `OWNER_BACKUP_HASH_MATCH = YES`) and restored
  byte-exact (`ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`,
  `OWNER_STORE_OPENED_BY_F2R = NO`), against a copy of F2's preserved disposable v5 store. **All three
  findings CLOSED at 1024×720, 1056×799 and 1440×900** (§30–§32): Sections prose wraps at a readable measure
  with the table area owning the scroll, `Details` leads every History row and was closed and reopened with
  `SPACE` under a visible focus ring, and Release renders "ELF address/flags evidence" and **"MAP regions +
  ELF load evidence"** with no raw enum on either that surface or the Analyze weakest-basis line. §33's smoke
  (Analyze, Compare, Gate twice, History, Diagnostics) passed with no crash, no new path leak and no new S0
  or S1. **§34 gives `P5_DESIGN_ACCESSIBILITY = PASS_FOR_FROZEN_DESKTOP_SCOPE` and nothing beyond it** — one
  host at 96 dpi / 100 % scale, with `WCAG_CERTIFICATION = NOT_PERFORMED`,
  `MULTI_DPI_125_150 = NOT_TESTED`, `SECOND_WINDOWS_HOST = NOT_TESTED` unchanged, and one trade recorded
  rather than glossed: at 1440×900 the corrected table now needs a contained scroll of about 1.06× the pane
  where the pre-fix layout fitted the pane by being unreadable. §35 moves **L20 to CLOSED ACROSS VERIFIED
  HUMAN-FACING MEMORY-BASIS SURFACES** (Compare → REOPENED_BY_F2 on Release → CLOSED_BY_F2R) without
  rewriting Commit E's historical sentence, and the §13 re-audit records the two prompt spellings that have
  no code behind them (`ConfiguredRegionAndElfLoad`, `InsufficientEvidence`) as **not found** rather than
  mapping invented semantics.
  What F2R2 writes: `P5_VALIDATION/P5_F2R_UI_CORRECTIVE_REPORT.md` (§37's twenty parts, and it does not
  contain P5's completion token), the §38 addenda to `P5_KNOWN_LIMITATIONS.md` §8, `P5_EXIT_CHECKLIST.md`
  §4–§6, `P5_DESKTOP_ACCEPTANCE_REPORT.md` §9 and `P5_EXECUTION_REPORT.md` §6, and `P5_CI_AUTHORITY.md`'s
  F2 and F2R1 rows — where F2's run #69 is recorded as **three attempts** (7 jobs then 4 jobs cancelled with
  **0 executed steps**, §25's allocation class, and a 10-of-10 third attempt) and `d1dc61c` is recorded as
  having **no run of its own**, because one push of two commits produces one run at the tip. F2R1's own run
  was read job by job and step by step: **10 of 10 on attempt 1**, with `868 Rust / 225 UI` taken out of the
  runner's logs rather than recalled. §42's docs-only rule is proved by §43's count rule: **868 Rust and 225
  UI in 8 files, identical to F2R1** — a documentation head that moved either number would be a product
  change wearing a documentation diff. Harness facts owned rather than buried: one refused installer click
  (`POINT_OWNED_BY_OTHER_WINDOW`, a Chrome window owned the pixel, no input sent), one mis-set installer step
  table, one `FOREGROUND_NOT_ACQUIRED` retry, and a geometry driver that initially matched a window titled
  "Claude" and was fixed with `require_app()` **before** any installed evidence existed.
  `F2R = FINAL PASS / COMPLETE`, `F2 = PASS`, `P5 = IN_PROGRESS`, product **MVP CANDIDATE**. **F3 stays
  Architect-controlled**: §46 ends this round with a STOP, no F3 work was started, no P5 completion sentence,
  no `active_task NONE`, no tag, release, signature, notarization or updater appears anywhere in it.
  *(F2R2's own STOP, left as written — and it was obeyed: the Architect issued F3, and the bullet above is
  that round. `P5 = IN_PROGRESS` and `F3 = READY_FOR_ARCHITECT_REVIEW` were the truth on 2026-10-06 at
  `a5ce7c4` and are not the present answer; the present answer is `P5 = PASS_COMPLETE` with `active_task`
  `NONE`.)*
- **Commit F2R1 lands on 2026-10-06 and it is a narrow UI corrective, not a new layer.** Authorized by
  *FirmwareSight — P5 Commit F2R Installed UI Productization Corrective Candidate, Execution Prompt v1.0 —
  Architect Authorized*, archived with both of its measured digests (delivered `5d8719ec…fb375`, 47,100
  bytes, 2,117 CRLF lines; stored Git blob `6281f5d2…fe4ed`, 44,984 bytes, 2,116 LF lines), it sits between
  F2 and F3 and fixes exactly the three findings F2 recorded against the installed product: **F2R-01 / S2**
  Analyze → Sections collapsed its prose column to one character per line, **F2R-02 / S3** the History row's
  `Details` control fell off the right edge at the default window width, **F2R-03 / S3** Release → Evidence
  basis printed the Core enum `MapRegionAndElfLoad` where a person should read *MAP regions + ELF load
  evidence*. The two table defects share one mechanism and it was measured before anything changed:
  `display: block` on the `<table>` pinned the box to the pane and let its columns be laid out below their
  own minimum, while `overflow-wrap: anywhere` made that prose column's minimum one character, so it took
  the entire deficit. `P5_VALIDATION/P5_F2R_UI_CORRECTIVE_DESIGN.md` §3 keeps the probe numbers and the
  negative control that showed a wrapper alone does not fix it, and §5 why sticky pinning was rejected
  (195 px over the sections header, 163 px over an opened History row). The repair is the one §5 asks for
  first: a `.viewport` wrapper owns `overflow-x: auto`, the table went back to being a table and kept its
  intrinsic minimum, prose cells took a `24ch` measure with `break-word`, an Unknown reason moved onto its
  own line under the word it explains, and the History action moved to the **leading** cell — the placement
  the Evidence table already used, so the precedent is this repo's own rather than a new invention.
  **No design token changed and no pixel constant was invented:** `assets/design-tokens.json` is
  byte-identical, and `ch` is the unit the prose of every page already bounds itself with (60ch, 68ch,
  72ch, 80ch, 88ch). F2R-03 reuses the accepted Compare vocabulary through one new shared
  `evidenceBasis.ts` instead of writing second wording for the same evidence, so Compare, Release and the
  Analyze weakest-basis line now say the same sentence for the same basis; `Analyze.tsx` and its test
  joined the diff for that reason, which §13 authorizes even though §16's path list alone would not.
  Scope checked mechanically, not remembered: the only product paths in the diff are
  `apps/desktop/ui/src/**`, and zero bytes changed under `crates/`, `apps/desktop/src-tauri/`, `scripts/`,
  `fixtures/`, `schemas/`, `migrations/`, `.github/`, `Cargo.toml`, `Cargo.lock`, `pnpm-lock.yaml`,
  `tauri.conf.json`, `deny.toml` or the token file. The serialized contract is untouched — `analysis:1`,
  `diff:1`, `gate-results:1`, `accepted-reviews:1`, `release-manifest:1`, schema 5, migrations 0001–0005,
  ADR-0028, ADR-0029, `elf.program-header` — because this round is display/layout only.
  Counts at this head, printed rather than expected: `cargo test --workspace` **868 passed / 0 failed
  across 47 targets**, unchanged as §21 requires, and UI **219 → 225 passed in the same 8 files** (+6: the
  three contracts and their guards). The authoritative gate is **17 of 17** with drift
  **8 of 8**, deny **1 of 1**, `core-smoke` **3 of 3** and package **4 of 4**, all with no `SKIP`. §20's four mutation proofs each reddened exactly the test that
  covers what they target, and §22's campaign is **20 fresh-process repetitions, 20 green, 0 failing, 225
  tests every time, 7 min 25 s**. One sequencing fact is recorded rather than smoothed over: the first
  §21 logs and the first §22 campaign described a tree that three comment-only lines then moved, so both
  were stopped, kept, and re-run — the numbers above come from runs that prove the UI manifest identical
  before and after.
  **What this commit deliberately does not claim:** §26's exact F2R1 Windows artifact has not been
  downloaded and §30–§33's focused installed revalidation at 1024×720 / 1056×799 / 1440×900 has not been
  run, so **no F2 finding is closed by F2R1** — §36 requires the new installed artifact before any
  disposition moves, and F2R2 records it. `P5 = IN_PROGRESS` and F3 unauthorized were F2R1's true state on 2026-10-06; the head
  above closed the stage.
- The task before it, kept as history: the G2 Product MVP engineering closure audit reached **PASS** on
  2026-10-01: **Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE.** Authorized by *FirmwareSight — G2
  Product MVP Engineering Closure Audit, Execution Prompt v1.0 — Architect Reviewed* (file, SHA-256
  `3c6ab83e…2bac9`, archived) and its inline *Storage Path Semantics Clarification Addendum v1.0*. Product
  tree `e35cfe7` (`055b54e` + the G2-F1 test-only fix) on Run `36906482900`, evidence head `f75cbc5` on Run
  `36948719972`, each 7 of 7 on the first attempt; 769 Rust / 155 UI / `check.py` 15/15 / clean worktree
  17/17; the whole chain on both surfaces with byte-identical parity; evidence in `G2_VALIDATION/`. At the
  time it closed, the next tracks (V1, P5) needed a separate architect decision and nothing there
  authorized one — P5 has since arrived with its own prompt, and V1 still has not.
- Post-G2, and the reason the source above is no longer the whole picture: the product then passed a
  **real-desktop MVP end-to-end acceptance** round on 2026-10-02 — 283 cases, black-box mouse, keyboard
  and native dialogs against the shipping binary — verdict **`PASS_WITH_FINDINGS`**, three findings, no
  S0 and no S1. Its evidence root lives outside the repository, under `%TEMP%`, and the owner's live
  store was restored byte-exact. A follow-on round, *Post-G2 E2E Findings Remediation & Focused
  Re-Validation, Execution Prompt v1.0 — Architect Reviewed*, then fixed all three, narrowly:
  **E2E-F001** (S3) — a surviving analysis did not say which selection it described, so choosing a
  second artifact of the same file name left the first one's figures looking like the new one's;
  **E2E-F002** (S2) — a destination folder that is not a bundle this engine wrote was offered for
  replacement above an engine sentence claiming it already holds one, a decision the engine refuses to
  carry out; **E2E-F003** (S3) — a GNU ld MAP whose banner sat past a 4 KB head window was refused as
  another linker's output. Fixes are `e816dcb` and `971015f`: 770 Rust / 159 UI / `check.py` 15/15,
  fix head green on Run #43 `37100371601` at 7 of 7 on the first attempt, evidence in
  `POST_G2_E2E_REMEDIATION/`. **G2 stayed PASS, the product stayed MVP CANDIDATE, the baseline stayed
  0.6.0, and that round created no P5 authority of its own** — P5 arrived the next day with its own
  prompt, recorded above. What the remediation deliberately did not take on is
  recorded there too: large-file latency and peak RSS stay carried forward with the wording
  `PARTIAL / environment-sensitive` and `MEASURED FOR TESTED WORKLOAD`, and 125/150 % DPI, mouse-wheel,
  the `update_goldens` issue and the licence choice are all still open.
- Previous task, kept as history: `P4_RELEASE_BUNDLE` reached `PASS / COMPLETE` on 2026-10-01 and the
  pointer returned to empty. P4 was opened on 2026-09-30 by *FirmwareSight — P4 Release Bundle MVP
  Implementation, Execution Prompt v1.0 — Architect Reviewed*, delivered as a file (the first stage prompt
  since P0 to arrive that way) and archived with its measured SHA-256
  `1baaec9204a1d2aa5aa53bd735b34d376ee56db7557a04d6abb79265c30840c5` under `10_AUDIT/SOURCE_PROMPTS/`.
  Its acceptance list was the frozen **US-004 Export Bundle** criteria at
  `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:42-49`, plus **PRD P0-6** at `01_PRODUCT/01_PRD_MVP.md:85-95`
  and `01_PRODUCT/01_PRD_MVP.md:126`'s independent-readability requirement. Evidence: `P4_VALIDATION/`.
  The only G2 statement the stage was allowed to write is the one it wrote:
  `READY_FOR_ENGINEERING_GATE_REVIEW` — never `PASS`.
  `P3_RELEASE_GATE`, `P2_COMPARE` and `P1_ANALYZE_DETAILS` closed earlier, on 2026-09-30, 2026-09-29 and
  2026-09-29, in `P3_VALIDATION/`, `P2_VALIDATION/` and `P1_VALIDATION/`.
  **The P4 start fact is `323afad`, one commit ahead of the `ba5e59e` the prompt's §0/§2 names.** §2's
  "if remote is newer, inspect and reconcile before writing" was run: `git diff --name-only ba5e59e
  323afad` moves eleven documentation, governance and integrity files plus two newly tracked
  baseline-artifact scripts, and no product source, fixture, schema or migration, and Run `36810689645`
  on `323afad` is `completed` / `success` / **7 of 7 jobs** (`gh run view 36810689645
  --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs`). Start counts re-measured
  before the first write: one `cargo test --workspace` = **556 passed / 0 failed / 0 ignored**,
  `corepack pnpm test` = **135 passed in 6 files**.
  At P4's closure the G2 engineering closure audit was the next authorizable act and P4's own prompt
  stopped at P4; that audit has since run and closed `PASS` (above).
  Carried forward from P3, still true: `origin/main` sat at
  `219178af195569ec6b13728d84d992ef78df8c04`, where Run `36774472141` concluded `failure` on one job of
  seven — `Dependency policy`, reporting `error[yanked]` for `yoke-derive 0.8.3`, a transitive proc-macro
  that crates.io yanked the same afternoon, after the local `deny` step had passed against an older
  index. Rust on both platforms, Desktop UI on both platforms, the macOS core smoke and the drift check
  passed remotely, and the lockfile moved to `0.8.4` in a successor commit. The prior head was `32b23aa`
  on Run #19 `36665007523`, success, 7 of 7 jobs, following Run #18 `36648718199` on the P2 implementation
  tree `4a77ea1` and the mid-round failure Run #17 on `c7fc2a3` — the fixture-pair story in
  `P2_COMPARE_EXECUTION_REPORT.md` §5.1. Every P3 gate number in this baseline is still a locally measured
  number; the remote result is recorded separately rather than folded into them.

## Product/architecture baseline

The v0.5.0 product, architecture and design decisions remain inherited unchanged. v0.6.0 adds the
validated P0 technical foundation; it does not redefine the product, does not add a fifth product
verb, and does not change MVP scope. Nothing in the frozen baseline was renegotiated to build the
slice, and nothing was renegotiated to promote it.

Four governance changes are in force now, all decided by the architect rather than by the coding side:

- **ADR-0025** supersedes exactly one clause of ADR-0020 (`P1 Product MVP implementation 只有两者都 PASS
  后开始`). After `P0 PASS` the architect may authorize a bounded, reversible Pre-G1 Analyze slice.
  Its Batch A precondition on the *next* slice was superseded the same day by ADR-0026 below; the slice
  it authorized, P1-A0, is complete and its record stands.
- **ADR-0026** (2026-09-29) makes this an **open-source MVP-first delivery**: MVP proceeds
  P1 → P2 → P3 → P4 → G2 on engineering grounds, `G1 = P0 PASS` for this delivery, V0 becomes
  `NON_BLOCKING_USER_FEEDBACK_TRACK`, and pricing / willingness-to-pay / pilot signals leave the gates.
  It supersedes sequencing only: no V0 artifact, P0 evidence, P1-A0 evidence or technical safety boundary
  is relaxed, and it authorizes no cloud, account, telemetry, AI, updater or licensing work.
- **Design tokens `0.2.0 → 0.2.1`**, adding one numeric semantic (`border.width.hairline = 1`) to close
  the documented P0 gap. No color, spacing, radius, typography, layout, motion, shadow or status value
  changed, and the focus ring keeps its own 2px token.
- **ADR-0027** (2026-09-29) is P3 prompt §6 written down as an architecture record rather than
  re-derived in code: a fifth first-party library crate, `firmwaresight-project`, owns
  `firmwaresight.toml`, project-local release evidence, the read-only system Git adapter and the
  deterministic Gate input fingerprints, because CLI and Desktop need one implementation of those
  semantics and Core must stay filesystem-free, Git-process-free and dependency-free. The boundary,
  the prohibited dependencies and the short admission list were specified by the architect in the
  prompt; the coding side's role was to record them, measure the locked versions and licenses, and not
  install anything the requirement does not need. It authorizes no Gate rule semantic outside Core, no
  artifact parsing inside the adapter, and no network.

## P0 — Technical Vertical Slice

Status: **`PASS`** · promoted to v0.6.0 by the architect-signed
*P0 Final Promotion / v0.6.0 Baseline Closure v1.0* · Task at promotion time: **NONE** (recorded as
`active_task: NONE` in `BASELINE.yaml`; the task live on 2026-09-29 was `P1_ANALYZE_DETAILS`, then after it
closed the field read `NONE` for a few hours until the P2 prompt arrived the same day — it read
`P2_COMPARE`, returned to `NONE` when P2 closed, and reads `P3_RELEASE_GATE` now. Each move changes the
pointer and not this frozen P0 verdict)

The chain P0 claimed is proven by executed commands, on this machine and on GitHub's runners:

```text
Real ELF + GNU ld MAP → guarded intake → parse → normalize → evidence
→ memory accounting → BuildSnapshot → deterministic CLI JSON → SQLite → typed IPC → Desktop summary
```

Exactly four Phase-0 library crates, with `firmwaresight-core` declaring zero dependencies. 104 Rust
tests and 19 UI tests as promoted; the tree now runs 142 and 31, and `P1_A0_VALIDATION/` accounts for
every test after those. SQLite schema version 2, including the v1→v2 migration. Deterministic CLI JSON
reproduced byte-identical. Memory accounting reproduced by hand from `readelf` and the MAP region
table. A 512 MiB guard measured from both sides of the boundary. A typed ts-rs IPC boundary whose
generated TypeScript is drift-checked. A real Windows desktop window opened and driven in the
shipping configuration.

### Remote verification

| Run | HEAD | Conclusion | Jobs |
| --- | --- | --- | --- |
| #1 `36360310447` | `f9b8ccb` | `failure` | 2 of 6 green — Rust (win), Rust (ubuntu), Generated output drift, Dependency policy red |
| #2 `36378384225` | `ebda52d` | `failure` | 6 of 7 green — only `Generated output drift` red (Ubuntu provisioning in a second job) |
| #3 `36399805005` | `1cd6309` | **`success`** | 7 of 7 — engineering closure |
| #4 `36402637251` | `5e58f77` | **`success`** | 7 of 7 — pre-promotion revalidation of the governance-only successor |
| #5 `36416146281` | `738ae78` | **`success`** | 7 of 7 — revalidation of the v0.6.0 promotion commit itself |
| #6 `36419864513` | `7d2f38a` | **`success`** | 7 of 7 — revalidation of the baseline consistency-closure commit; the last remote fact of the P0 chain, and still `last_remote_ci` inside that block |

Read with `gh run view 36419864513 --repo 2023violet/FirmwareSight`, not from this file. Run #6's
matrix: `Rust (windows-latest)`, `Rust (ubuntu-latest)`, `Desktop UI (windows-latest)`,
`Desktop UI (ubuntu-latest)`, `Generated output drift`, `Dependency policy`, `macOS Core Smoke` —
all `success`. The consistency-closure commit is on `origin/main`, so the baseline record is green on
its own commit, not only on the commit the architect read.

The two failures are kept as history rather than edited. They are the reason the four successes that
followed mean anything: Run #1 exposed a checkout that could rewrite committed evidence bytes,
a runner with no
Tauri Linux prerequisites, a drift assertion that compared third-party encoder output instead of
pixels, and a supply-chain policy the pinned tool could not parse; Run #2 then exposed that a second
job compiled the desktop crate without the prerequisites its sibling had been given.

### What the two remediation rounds changed

| Cause | Fix | Confirmed |
| --- | --- | --- |
| Checkout changed fixture bytes | `.gitattributes` text policy (`*.ld text eol=lf`, `*.map -text`, binaries unchanged); one manifest value moved to the blob's own hash | Runs #2, #3, #4 |
| Ubuntu runner lacked Tauri prerequisites | the documented apt block, `if: matrix.os == 'ubuntu-latest'`, desktop crate still compiled | Runs #2, #3, #4 |
| Icon check compared encoder bytes | decode and compare pixels; missing ICO frame still fails | Runs #2, #3, #4 |
| `deny.toml` unparseable by 0.20.2 | rewritten to the keys the tool accepts; allow list from `cargo deny list`; bans evaluated over the four shipping targets | Runs #2, #3, #4 |
| No macOS job | `macos-core` running `check.py --only core-smoke` on `main` pushes | Runs #3, #4 |
| `drift` job lacked the same prerequisites | the proven apt block copied into that job — no abstraction for two jobs | Runs #3, #4 |

A sixth defect was found by the authorized desktop launch rather than by CI: `evidence` had a
whole-table primary key while `04_TECH/15` §4 declares `Build 1─N Evidence`, so a second artifact
could never be stored. Migration `0002` keys evidence by `(build_id, id)`; the two failing tests were
written before the fix; the upgrade was replayed against the real user-profile database.

## V0

Status: **`NON_BLOCKING_USER_FEEDBACK_TRACK`** since ADR-0026 (2026-09-29). The sample state is
unchanged and is still an honest zero.

Formal eligible external sessions: `0 / 8 minimum`, `0 / 4–5 Batch A target`.

Completed: clickable prototype, fixture/state machine, protocol/session templates, internal
functional dry run, Batch A takeover, Batch A recruitment-ready package, Batch A activation.

No CI run, no green gate and no promotion signature can supply the missing input, which is real human
participants; the coding side authors none, and none were authored. What changed on 2026-09-29 is the
consequence of that zero: it no longer blocks P1, P2, P3, P4 or G1. The activation round earlier the same
day had already verified the recruitment pack and written nothing else; `ADR-0026` then moved the track
off the critical path, and the price-anchor prompt that would have followed it was
**`WITHDRAWN_BY_ARCHITECT`, never executed**.

The instrument stays frozen and reusable on purpose: `V0_VALIDATION/prototype/` at `v0.1.0` with
`protocol/TASK_SCRIPT.md`, `sessions/TEMPLATE.md` and both registers, so a later feedback round remains
comparable with the protocol that already exists. What is now unverified rather than merely deferred is
the thing V0 existed to check: whether a real firmware engineer distinguishes `Unknown` from `PASS`,
understands why a MAP is requested, or reads a `Review` correctly. That risk is carried forward and
recorded in `ADR-0026`'s Consequences, not argued away here.

## Gates

```text
G0: PASS
P0: PASS — promoted to the v0.6.0 Technical Foundation Baseline; frozen, no further P0 closure prompts
G1: PASS — basis is P0 PASS under ADR-0026 (2026-09-29). Before that date this file read `NOT CLAIMED` against `G1 = V0 PASS + P0 PASS`, and the historical records still say so
V0: NON_BLOCKING_USER_FEEDBACK_TRACK — 0 of 8 eligible external sessions, an honest zero that gates no P-stage and no G1
Pre-G1 (ADR-0025): P1-A0 REAL ARTIFACT INTAKE — COMPLETE, including its evidence-identity and persistence correctness closure
P1: PASS / COMPLETE — the Analyze verb as one product verb: intake, summary, top contributors, bounded Sections / Symbols / Evidence details, the Evidence Inspector, and the US-001 bytes/KiB presentation switch. Evidence: P1_VALIDATION/
P2: PASS / COMPLETE — Compare over persisted snapshots: Core-owned deterministic diff, bounded Compare IPC with a session-local registry, the second desktop page, `fwsight diff`, and portable Diff JSON v1 plus self-contained HTML. Evidence: P2_VALIDATION/. The round's own measurements are LOCAL PASS; the pushed head `4a77ea1` is green on Run #18 `36648718199` (7 of 7), which closes defect E — the fixture half `.gitignore` hid — on the remote too. Open on purpose: desktop smoke step 27 was not observed in the shipped window
P3: PASS / COMPLETE — Release Gate, authorized 2026-09-29 by *P3 Release Gate MVP Implementation, Execution
Prompt v1.1* and by ADR-0027, closed 2026-09-30. Evidence: P3_VALIDATION/. The round's own gate numbers are
LOCALLY measured; what the remote then said is recorded separately in that pack, and P3 is SEALED — no
further P3 documentation-only successor.
P4: PASS / COMPLETE — Release Bundle, opened 2026-09-30 by its own architect prompt (v1.0, delivered as
a file and registered with its SHA-256), closed 2026-10-01. Evidence: P4_VALIDATION/. The round's own
numbers are LOCALLY measured — one `cargo test --workspace` = 769 passed / 0 failed, UI 155 in 6 files,
`check.py` 15/15, the fifty-step shipped-window smoke, the independent reader at 64/64 and 59/59 — and
the remote result is recorded separately: the final implementation head `e799f2f` is green on run
`36872456446`, 7 of 7 jobs. P4 is the last core product-implementation stage of the MVP line.
G2: PASS — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE (2026-10-01). The whole-MVP engineering
closure audit, under its own prompt and storage-path addendum; evidence G2_VALIDATION/; product tree
e35cfe7 and evidence head f75cbc5 each 7 of 7 on the first attempt. Not productization, beta, RC or GA
Open-source licence: OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION — a technical MVP candidate is not a licensed open-source release
Pricing / willingness-to-pay / team-pilot signal: DEFERRED_POST_MVP; the price-anchor prompt was WITHDRAWN_BY_ARCHITECT and never executed
Active task: NONE — G2 closed 2026-10-01; V1 and P5 each need a separate architect decision
Design tokens: v0.2.1
```

`ADR-0026-open-source-mvp-first-delivery.md` is what changed here. It re-bases G1 on `P0 PASS` for the
current open-source MVP and moves V0 off the critical path, superseding the sequencing conclusions of
ADR-0020 and ADR-0025 while leaving their decisions, evidence and safety boundaries in place. G1's
technical content in `06_DELIVERY/06_STAGE_GATES.md` is unchanged and still has to hold; what was removed
is the requirement that a moderated human panel exist before more of the product can be built. That is a
real cost, recorded in the ADR's own Consequences section: whether users distinguish `Unknown` from
`PASS`, or understand why a MAP is asked for, is now unverified rather than deferred.

Full reasoning: `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md`; the promotion act:
`P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`; per-item evidence:
`P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`; run history: `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`;
the two remediation rounds: `P0_CI_REMEDIATION_REPORT.md` and `P0_CI_RUN_2_CLOSURE_REPORT.md`; the
shipped-window result: `P0_DESKTOP_SMOKE_REPORT.md`.

## Known gaps that promotion does not remove

- **Peak RSS: `NOT MEASURED`**, with the reason and the measurements that do exist in
  `P0_PERFORMANCE_REPORT.md`.
- **Fuzz campaign: `NOT RUN`** — `cargo-fuzz` needs a nightly toolchain the tool policy did not
  authorize. The no-panic claim rests on regression tests over malformed fixtures.
- **Two RustSec advisories accepted, not resolved**: `RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and
  `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`, unmaintained) enter through the `gtk-rs 0.18` line
  Tauri 2.12.0 requires and have no compatible upgrade inside it. Disposition: accepted as explicit
  P0 transitive risk, not a promotion blocker, with five revisit triggers. The dependency policy is
  "PASS under a documented policy with two explicitly accepted transitive advisories" — not
  "zero vulnerabilities" and not "security clean".
- **Desktop smoke: one Windows host, one WebView2 version, 100% scaling.** No Windows 11, no other
  DPI, no Linux or macOS window, and the loading state stayed a test-only claim (recorded NOT
  OBSERVED).
- **Two linker layouts tested.** Executable-in-RAM, external SRAM, DMA pools, overlays and
  MPU-aligned sections are unproven; DWARF is recognized but not consumed. **This is the v0.6.0 promotion's
  sentence, and Commit E widened exactly half of it:** `fixtures/elf/p5-compat/` now carries executable-in-RAM,
  external SRAM and a DMA-shaped writable region as real linked evidence across four distinct MAP region shapes
  plus a Clang-built image, so those three are proven rather than unproven. Overlays and MPU-aligned sections
  still are not, and DWARF is still recognized and not consumed — see `P5_COMPATIBILITY_MATRIX.md` and L5's
  `REDUCED` disposition in `P5_SUPPORTABILITY_REPORT.md`.
- **Two stored facts carry no reason.** `sections.file_offset` and `symbols.address` are nullable with
  no companion `*_unknown` column, so a reason cannot survive the write. The read layer types them as a
  plain option and the UI says `Unknown` with no invented explanation. Closing this needs a migration,
  which P1 deliberately did not write; found and reported in
  `P1_VALIDATION/P1_ANALYZE_EXECUTION_REPORT.md` §6.
- **One dead `Apply filter` click is unresolved.** It happened once on an intermediate build and could
  not be reproduced on the shipped binary or in the automated reproducer. The leading hypothesis is the
  WebView2 autofill popup, and `autoComplete="off"` removes that surface, but no cause is established.
  Recorded as unresolved in `P1_ANALYZE_DETAILS_SMOKE_REPORT.md` §6 rather than written up as fixed.
  **Commit E re-ran the question with volume and still found nothing**: 30 legitimate click activations across
  both filter tables and 6 more through form submit, each verified to reach the shell as a request carrying its
  value (`compare.test.tsx`, and L25's `NOT_REPRODUCED` row in `P5_SUPPORTABILITY_REPORT.md`). Keyboard
  activation stays `NOT_VERIFIED_THROUGH_A_KEYPRESS` — a synthetic Enter `keydown` produces no request because
  jsdom does not implement implicit form submission, and a real keypress needs an installed window, which
  §64's journey (Commit F's) will supply.
- **Icon provisioning duplication is a real cost.** Two jobs copy the same apt block; a third job that
  compiles the shell on Linux is the trigger to extract it.
- **Every gate step except one reads the working tree, not the index.** That blind spot is how P2's
  fixture pair survived eight commits with only half of it in the repository: `.gitignore`'s
  `**/target/` matched `fixtures/elf/p2-diff/target/`, the manifest recorded all twelve paths, and the
  hash test happily digested the files the generator had left on disk. CI caught it because CI clones.
  `drift/fixtures tracked` now closes the case for anything a manifest lists; a source file ignored by
  accident is still unchecked, because no manifest names it. Recorded in
  `P2_VALIDATION/P2_COMPARE_EXECUTION_REPORT.md` §5.1.

## Version rule

v0.6.0 exists because P0 reached `PASS` with real engineering evidence measured on three platforms,
signed by the architect. The rule that produced it stays in force for the next promotion: a failed run
must not be relabeled into a PASS baseline, a locally green gate must not be relabeled into a CI PASS,
and a green CI does not equal an architectural promotion. `v0.7.0` and G1 claims will require their own
measured evidence and their own signed prompt.

## Integrity record

`DIRECTORY_TREE.txt` and `SHA256SUMS` are the **v0.6.0** baseline artifacts, regenerated after every change to
the tracked set, in that order: the tree lists the tracked layout rooted
at `FirmwareSight_Project_Baseline_v0.6.0/`, and the manifest carries a lowercase SHA-256, LF line endings,
lexicographic path order and no self-entry for each baseline-controlled tracked file. Both are
produced by `scripts/generate_baseline_artifacts.py` (`tree`, then `sums`) and re-checked by
`scripts/verify_baseline_artifacts.py`, which shares no code and no hashing helper with the generator, because
the manifest cannot certify itself.

**Since ADR-0029 the digests are of canonical Git stage-0 index blob bytes, not of working-directory bytes** —
see the L26 bullet above for why, and `09_ADR/ADR-0029-…md` for the rule and its alternatives. Three
consequences for whoever runs these tools next:

- The order is load-bearing, not stylistic: stage the intended changes, regenerate the tree, **stage the tree**,
  then generate the manifest. `sums` refuses to write while Git reports an unstaged semantic change to a
  tracked path, so the sequence cannot be silently short-cut; a line-ending-only difference is not such a
  change, because the guard asks Git rather than reading bytes.
- `sha256sum -c SHA256SUMS` against a working tree is **not** a valid check any more, and on this host it
  reports 13 files as failures because their checkout representation is CRLF and their blobs are LF. The
  canonical pass is `python scripts/verify_baseline_artifacts.py`, which now also runs as the
  `drift/baseline integrity` step of `python scripts/check.py` on every authoritative CI run. For an external
  proof, `git archive <commit>` extracted outside the repository validates with plain `sha256sum -c`, because
  an archive carries canonical blob content.
- The basename exclusion is unchanged, so the two tracked files named `SHA256SUMS` — the root manifest and
  `golden/reports/p4-release/SHA256SUMS`, a Release Bundle fixture in a different checksum domain — both stay
  out of the root manifest.

This tooling used to live under
gitignored `target/`, which meant a fresh clone could not regenerate either artifact; it is tracked now.
`manifest.txt` is left exactly as the v0.5.1 delivery wrote it - the frozen package list, not
this tree - and `P0_FINAL_PROMOTION_REPORT.md` §10 records the commands and the counts of that promotion,
which describe the tree they measured rather than this one.

**How to read a `file:line` citation in this repository.** A closed validation pack is evidence about the
tree it measured, so its line numbers are as-of-writing, not live: `App.module.css` and `App.tsx` have
both shrunk since P0, and several P0/P1/P2 checklist lines now point past the end of those files. Those
packs are not rewritten to match current code - that would turn a record into a claim. Within a *living*
document, cite the key or the symbol rather than the line number, because a line number goes stale the
moment an adjacent line is added: P3 inserting `crates/firmwaresight-project` into the workspace member
list pushed `license = "Proprietary"` from `Cargo.toml` line 17 to line 18 and silently invalidated every
citation written before it. The current stage's pack is the one place a line pointer is still checkable.

## Next work

**`active_task: U1_UI_PRODUCTIZATION_CONVERGENCE`, and it closed on 2026-10-07.** A new reader inherits: Productization
**ENGINEERING_COMPLETE**, `P5 = PASS_COMPLETE`, product **MVP CANDIDATE** at baseline **`0.6.0`** (narrative:
**FirmwareSight Productized MVP Candidate**), `G2 = PASS`, P0 / P1 / P2 / P3 / P4 still at their own
`PASS` / `PASS_COMPLETE` — nothing earlier was re-statused retroactively — **V1 paused at
`RECRUITMENT_READY` with zero eligible sessions**, and **U1** having converged the desktop UI one round toward
the reference screens without claiming a stage of its own.

**What U1 leaves for the next UI round.** `U1_VALIDATION/U1_VALIDATION_REPORT.md` §6 is the list, and it is
deliberately unpolished: the references' right-hand detail column (no page holds a selection to fill it, so the
components drafted for it were deleted rather than shipped unused), a fourth compact button level for dense table
row actions, the `History` heading against the `Bundle & History` rail and window-title name, the two Release
actions with no command behind them, and a real-desktop before/after screenshot pass this session was not
authorized to run. Nothing on that list is a defect; each is unfinished convergence, and §13 is the reason it is
written down instead of quietly omitted.

**What V1 is waiting on, and who has to move next.** V1 is paused, not closed, and its blocker is not the pause:
it is at `RECRUITMENT_READY` because §5 forbids an agent from being the participant. **Recruitment, consent and
moderation are the human operator's work** (§47, §48): real external firmware/embedded engineers, screened
against §7, running their **own** artifacts on the exact frozen installer. The repository's next V1 write should
be a Batch A evidence commit after four eligible participants exist — not a document written before one session
has happened. `AGENTS.md` 1 still means what it has meant in every round: a pointer names one track and nothing
beyond it, and no agent may lift the next track off the roadmap.

**What still needs a new architect prompt.** B1 / private beta, RC, GA, commercialization, licensing, signing,
notarization, an updater, any new product verb, format, adapter or crate, and every E1/E2/E3/GX candidate —
unchanged from P5's closure list: V1 added nothing to it, and U1 added nothing to it either. V1's own §40 is explicit that **even a full V1 pass
does not open B1**, and §39/§54 forbid this track from self-issuing `V1_PASS_COMPLETE`, `B1_READY` or
`PRIVATE_BETA`. `P5_RELEASE_READINESS.md` §5 still freezes the ten release-readiness states, and
`P5_FINAL_CLOSURE_REPORT.md` §14 still says what `PASS_COMPLETE` does not mean. Start reading the carried
limitations at `P5_VALIDATION/P5_KNOWN_LIMITATIONS.md`: 26 rows, each with its disposition and what it does not
cover — **L11 is the row V1 exists to answer, and it stays `CARRIED_FORWARD` until real people have been
through it.**

**`G2` closed `PASS` on 2026-10-01, under its own architect prompt and addendum.** Audit-first: the gate
ran before anything was written and found G2-F1, a test race fixed test-only in `e35cfe7`; then the whole
MVP was driven through the CLI twice and the shipping window once, the clean tracked tree was proven in a
detached worktree, the failure paths were forced, and the path boundary was proven in 17 checks after the
architect adjudicated `artifacts.path` as expected local-only storage. Per-item verdict:
`G2_VALIDATION/G2_EXIT_CHECKLIST.md`; the canonical known-limitations list:
`G2_VALIDATION/G2_KNOWN_LIMITATIONS.md`.

**`P4 Release Bundle` closed `PASS / COMPLETE` on 2026-10-01, under its own architect prompt.** One
product verb, `Bundle`, over results the earlier stages had already computed and stored: a release
plan — id, version, ten-file list with digests — assembled before any byte is written; a staged copy
of the composed documents and the current artifacts; verification of the staged bytes against their
own `SHA256SUMS` and manifest; publish by rename; replacement only under an explicit confirmation and
only of a destination this engine wrote, with rollback if the swap fails; and a `release_records` row
written after the bytes, never before. Per-item evidence:
`P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md`; what was built, every gate number, the CLI smoke and the
CI runs: `P4_BUNDLE_EXECUTION_REPORT.md`; the shipped-window result over all fifty §59 steps:
`P4_BUNDLE_DESKTOP_SMOKE_REPORT.md`; design review: `P4_BUNDLE_DESIGN_CHECKLIST.md`. The boundaries
held and were not relaxed: Core stayed headless and filesystem-free (`grep -rn "std::fs"
crates/firmwaresight-core/src/` counts zero), assembly lives in the project crate, no sixth crate and
no new third-party dependency entered, no host path crossed IPC or entered a composed document, the
only clock a bundle carries is the moment a named person accepted a review, and the stage stopped at
the Bundle — no History page, no installer, no signing, no updater.

**`P3 Release Gate` closed `PASS / COMPLETE` on 2026-09-30, under its own architect prompt.** One product
verb, `Gate`, over a stored build plus the workspace facts outside the artifact:
`firmwaresight.toml` → project and Git evidence → `GatePolicy` → Core Gate → five factual states →
immutable `GateRun` → immutable review acceptance → `fwsight gate` → the desktop `Release` page. Per-item
evidence: `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md`; what was built, every gate number and the three
defects this round found: `P3_GATE_EXECUTION_REPORT.md`; the shipped-window result over all forty §61
steps: `P3_GATE_DESKTOP_SMOKE_REPORT.md`; design review: `P3_GATE_DESIGN_CHECKLIST.md`. The boundaries
held and were not relaxed: Core stayed headless and dependency-free, the adapter owns the config and the
read-only Git process, no rule is re-evaluated in SQL or React, no host path crossed IPC, the database or
the portable output, and the stage stopped at the Gate.

`P1_ANALYZE_DETAILS` completed on 2026-09-29 and P1 Analyze is `PASS / COMPLETE`:
bounded `Sections`, `Symbols` and `Evidence` queries over the snapshot SQLite already stores, three
use-case IPC commands, top contributors, the Evidence Inspector, and the `bytes / KiB` presentation
switch that `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` US-001 requires. Per-item evidence:
`P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md`; what was built and what was found:
`P1_ANALYZE_EXECUTION_REPORT.md`; the shipped-window result: `P1_ANALYZE_DETAILS_SMOKE_REPORT.md`;
design review: `P1_ANALYZE_DESIGN_CHECKLIST.md`. Earlier slices keep their own packs: `P0_TECHNICAL_VALIDATION/`
and `P1_A0_VALIDATION/`.

**`P2 Compare` closed `PASS / COMPLETE` on 2026-09-29, under its own architect prompt.** The prompt
changed the pointer while the round ran, not the standards: `P2` was `IN_PROGRESS` until `US-002` was
green item by item, and it is now — per-item evidence in `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md`,
what was built and every gate number in `P2_COMPARE_EXECUTION_REPORT.md`, the shipped-window result in
`P2_COMPARE_DESKTOP_SMOKE_REPORT.md`, design review in `P2_COMPARE_DESIGN_CHECKLIST.md`. The boundaries
that held for P1 held for P2 and were not relaxed: no schema migration (`SCHEMA_VERSION` still 2, no
0003), no new dependency (`Cargo.lock` gained no package), no new design token (still v0.2.1) - a
genuinely missing token stops a round rather than being written as a magic number - no whole-table
payload across IPC (default limit 100, hard max 500, enforced in Rust), details always bound to the
last-good handle, `Unknown` never rendered as zero, addresses never unit-converted. `AGENTS.md` 2 / 7 /
11 keep applying in full; `ADR-0026` relaxed research sequencing, not a single technical boundary.

**`P3 Release Gate` closed `PASS / COMPLETE` on 2026-09-30, under its own architect prompt.** The same
prompt line that authorized the work authorized a structural change, so it is stated here rather than left
to be discovered in a diff: P3 added the fifth first-party library crate (`crates/firmwaresight-project`,
ADR-0027), the third migration (`0003_gate_history.sql`, `SCHEMA_VERSION` 2 → 3, additive only), the
first direct dependencies outside the frozen five (`toml`, `regex`, `sha2`, `thiserror`, `serde` — all but
`regex` already present in the graph or admitted by ADR-0027), a third top-level desktop page (`Release`),
two CLI exit codes that were unreachable and are now not (`4` REVIEW / `5` BLOCK — the surface test
asserted `[0, 2, 3, 6]`, and P3 changed it by name rather than by deleting it), and three new portable
contracts (`gate-results`, `accepted-reviews`, `project-config` at v1). It added **no** design token, did
not touch the license metadata, did not create a Bundle table and did not start History. The verdict is
item by item in `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md`; what the round found on the way is in the other
three documents of that pack.

Three product defects were found by P3's own validation and fixed in product code, each with a regression
test: a version pattern quoted inside an evidence locator made a Gate run unpersistable for any project
using the only MVP version source (`ERR-STORAGE-4006`); one build produced two different run ids on the
two surfaces because the stored footprint carried no evidence pointer and Core hashed a `reason` the
database never recorded; and a disabled primary button kept its accent border.

What belongs to the owner and the architect:

1. pushing remains the owner's act. The P2 round pushed twice. The first, mid-round at `c7fc2a3`, produced
   **Run #17 `36596452341`, `failure`, 3 of 7 jobs red** — `Rust (windows-latest)`, `Rust (ubuntu-latest)`
   and `macOS Core Smoke`, all on `committed_fixtures_match_their_recorded_hashes`, for the reason in the
   gap above: `.gitignore`'s `**/target/` had kept the target half of the fixture pair out of the
   repository while every local run digested the bytes the generator had left on disk. That is defect E,
   fixed at `cfee1e5` and re-verified from a clean `git archive` checkout. The owner then pushed the head,
   and **Run #18 `36648718199` on `4a77ea1` concluded `success`, 7 of 7 jobs** — the same seven that
   carried P1's `e63afaf` on Run #13 `36556735551`. `origin/main` is now green again. No commit carries the
   run its own push produced: #18 is recorded by the successor document that reports it, and the run this
   document's own push starts belongs to a later one;
3. the **open-source license decision**, which is not P3's to make. The root `Cargo.toml`'s
   `[workspace.package]` table still reads
   `license = "Proprietary"` and the repository root still has no `LICENSE` file, while the project is
   being delivered as open source under ADR-0026; `AGENTS.md` 9 places a license change in front of a
   human, so the P3 prompt explicitly declines to choose one and the gap is recorded as
   `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` rather than quietly resolved;
4. a **separate architect prompt** for P4 Release Bundle, which P3's closure will make authorizable and
   this file does not authorize. P3's own prompt says STOP AFTER P3, and nothing in P4's scope — a
   bundle directory, `release prepare`, manifest generation, bundle checksums, a History page — exists
   or is implied by it;
5. whether and when to resume the V0 feedback track with real participants - now a choice rather than a
   gate, with the `v0.1.0` instrument still frozen and ready;
6. re-authorizing pricing, paid-tier or pilot work only after MVP, since `ADR-0026` defers it and the
   matching prompt was withdrawn before execution;
7. re-opening the two accepted advisories only on one of their recorded triggers.

Four items the closure round surfaced and left alone, each with its reason in
`P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` §9.7: the golden updater no longer reproduces the
committed goldens' key order; `ElfProgramHeader` names a source the ELF parser never reads; the frozen
release-manifest schema describes one artifact per build; and a release build without
`--features custom-protocol` shows a WebView network error instead of the product.

Do not start P1, Compare, Gate, Bundle, installer, signing, updater, SBOM, cloud, accounts, AI or
telemetry from this file.
