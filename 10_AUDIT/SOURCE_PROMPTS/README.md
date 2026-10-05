---
title: "Execution Prompt Register"
doc_id: "FS-AUDIT-SRC-003"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Audit"
last_updated: "2026-10-04"
---

# Execution Prompt Register

Every authorized execution prompt is preserved here verbatim **where a verbatim copy exists**. Each
SHA-256 below was recomputed from the archived copy in this directory, so the archived bytes match what
was delivered rather than a retranscription. Three P0 rounds were supplied to the executing agent as
text with no source file; each says so in its own entry instead of standing for a hash it cannot have.

## V0 Workflow Prototype Validation

- File: `FirmwareSight_V0_Workflow_Prototype_Validation_PROMPT_v1.1.txt`
- SHA-256: `3431a688c65c08aa6104fe4cb785ef25b6ae9df4e2d3272dddda653704e1df20`
- Authority: execution instruction subordinate to the unique v0.5.0 product baseline, and authoritative for the authorized V0 task.

## V0 Batch A External Validation

- File: `FirmwareSight_V0_Batch_A_External_Validation_PROMPT_v1.0.txt`
- SHA-256: `e62c236b090de9da589bcf570415edd7a0296925910d65437b2105a9612589e7`
- Authority: execution instruction for the Batch A external validation, subordinate to the v0.5.1 baseline. It withheld `v0.5.2` until real Batch A evidence exists, and left P0 unauthorized in that execution.

## P0 Technical Vertical Slice

- File: `FirmwareSight_P0_Technical_Vertical_Slice_EXECUTION_PROMPT_v1.1_ARCHITECT_REVIEWED.txt`
- SHA-256: `60a59196708a53592be4d828a0c1275cf74b3bfad0bdc6455f863ab38d32929b`
- Size: 94802 bytes / 5064 lines
- Authority: execution instruction for the P0 engineering slice against the unique v0.5.1 baseline. Consistent with `09_ADR/ADR-0020-validation-sequence.md`, which already authorizes V0 and P0 as parallel tracks. It overrides only the execution state recorded by the Batch A prompt (`V0` blocking, `P0` not authorized); it does not override any frozen product, architecture, evidence, gate or design baseline, and it does not authorize P1.
- Provenance record: `P0_TECHNICAL_VALIDATION/P0_EXECUTION_PROVENANCE.md`
- Outcome, 2026-09-28: executed to its stop condition as `EXECUTED — CONDITIONAL_PASS (LOCAL)`.
  Baseline stayed `v0.5.1`, `v0.6.0` withheld, G1 not claimed, P1 still unauthorized. The conditions
  were that CI and `cargo deny` had never run, which needed a push this environment was not
  authorized to make.
- Outcome, superseded the same day by measurement: the delivered tree was pushed as `f9b8ccb`, GitHub
  Actions run `36360310447` concluded `failure` with 2 of 6 jobs green, and the status became
  `FAIL — REMOTE CI RUN #1`. `CONDITIONAL_PASS (LOCAL)` is no longer the current label; the
  `LOCAL PASS` / `CI PASS` distinction this pack kept is what made the failure legible rather than
  surprising. Evidence: `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`.
- Final disposition, 2026-09-28: **`P0 = PASS`**, promoted into `FirmwareSight_Project_Baseline_v0.6.0`
  after remote CI Runs #3 and #4 each came back 7/7 green. Nothing in this prompt's scope was renegotiated
  to get there; the two rounds registered below changed how the tree is built and checked, not what the
  slice was asked to produce.

## P0 CI Closure / Cross-Platform Reproducibility Remediation

- File: none. The prompt was supplied inline to the executing agent, so unlike the records above
  there is no stored copy whose SHA-256 can be recorded here. The repository's copy of its terms is
  `.ai/DECISIONS.md` ("P0 CI closure remediation") and
  `P0_TECHNICAL_VALIDATION/P0_CI_REMEDIATION_REPORT.md`.
- Version: v1.0, architect reviewed.
- Authority: local remediation of what Run #1 measured, one `cargo-deny@0.20.2` installation, and one
  real desktop window launch in the shipping configuration. It does not authorize a push, does not
  permit `REMOTE CI PASS` to be written by the executing side, stops before P0 promotion, and does not
  change `ADR-0020`.
- Outcome, superseded by measurement: the owner pushed `ebda52d`, run `36378384225` executed and
  concluded `failure` with **six of seven jobs green**. The first round's five fixes are therefore
  confirmed remotely; the residue is a second job needing the same Linux prerequisites, which is what
  *Remote CI Run #2 Final Drift Closure v1.0*, registered below, closes.
- Outcome, final: the round-2 HEAD `1cd6309` was pushed and Run `36399805005` concluded `success`
  with **7 of 7 jobs green**, including `Generated output drift`. Remote CI is therefore closed by
  measurement; P0 promotion is not, because `P0 = PASS`, `v0.6.0` and a regenerated `SHA256SUMS` are
  the architect's signed act. The track read `CONDITIONAL_PASS` with that condition named until the
  promotion entry at the end of this file.
- Outcome, 2026-09-28: `LOCAL REMEDIATION COMPLETE / READY FOR REMOTE CI RERUN`. All five causes
  reproduced with a command before being changed, a sixth defect found by the authorized window launch
  and fixed with tests first, local gate 14/14 with zero skipped mandatory steps, desktop smoke `PASS`,
  and one architecture conflict (two advisories unreachable inside the frozen Tauri 2.12.0 dependency
  tree) reported rather than resolved. Baseline stays `v0.5.1`, `v0.6.0` is not generated, G1 is not
  claimed, P1 is not started, and the push that closes the round belongs to the owner.
- Final disposition: the round's five fixes plus the macOS job were confirmed remotely by Runs #2, #3 and
  #4, and the storage defect its authorized window launch found became migration `0002` (schema version 2).
  Superseded as a status by *P0 Remote CI Run #2 Final Drift Closure* below.

## P0 Remote CI Run #2 Final Drift Closure

- File: none — **original file unavailable, prompt supplied inline**. No SHA-256 is recorded because no
  archived bytes exist; claiming a "byte-exact copy" of text that arrived in a conversation would be a
  fabricated provenance record. The repository's copy of its terms is `.ai/DECISIONS.md` ("Remote CI Run #2
  and its final drift closure") and
  `P0_TECHNICAL_VALIDATION/P0_CI_RUN_2_CLOSURE_REPORT.md`.
- Version: v1.0, architect reviewed.
- Authority: a workflow-scoped fix for the one job Run #2 left red, plus the governance record of that run.
  Not a push, not `REMOTE CI PASS` from the executing side, not P0 promotion, not `v0.6.0`, not P1, and not
  an architecture redesign. It named the fixes it forbade: deleting the desktop test command, dropping or
  disabling the IPC bindings step, `continue-on-error`, `if: false`, moving the job to Windows, turning the
  failure into a warning, deleting the macOS job or the dependency policy, editing a fixture hash, or
  relabeling P0 as `PASS`.
- Outcome: Run `36399805005` on HEAD `1cd6309` concluded `success`, **7 of 7 jobs green**, with the drift
  job installing the Linux prerequisites and reporting `5/5 steps passed`. The owner pushed that HEAD;
  Run #4 `36402637251` then reproduced the result on the architect-reviewed documentation HEAD.

## P0 Final Promotion / v0.6.0 Baseline Closure

- File: none — **original file unavailable, prompt supplied inline**, so there is no source file to hash.
  The repository's copy of its terms is `.ai/DECISIONS.md` ("P0 Final Promotion Decision — 2026-09-28")
  and `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`.
- Version: v1.0, **architect signed**.
- Authority: exactly one act — promote P0 to `PASS` and freeze `FirmwareSight_Project_Baseline_v0.6.0` as
  the *P0 Technical Foundation Baseline*. It states in its own §1 and §2 that this does not mean V0 passed,
  G1 passed, the MVP is complete, P1 is authorized, Compare/Gate/Bundle exist, a release or installer is
  ready, user value is validated, or the tree is free of vulnerabilities; and it keeps `G1 = V0 PASS + P0
  PASS` unclaimed with `ADR-0020` untouched.
- Scope guards: no production source may move during promotion (`crates/**`, `apps/**`, `fixtures/**`,
  `golden/**`, `migrations/**`, both lockfiles, `deny.toml`, `scripts/check.py`, the workflow, the design
  tokens) — a needed source change would have stopped the round as
  `PROMOTION BLOCKED BY NEW ENGINEERING DEFECT`. No GitHub Release, tag, installer, signing, notarization
  or updater metadata. `V0_VALIDATION/**` not touched. Stale claims are to be audited phrase by phrase and
  classified historical versus current, not globally string-replaced, and version metadata is bumped only in
  current baseline authority documents.
- Outcome, 2026-09-28: executed. `P0 = PASS`, `baseline_version: 0.6.0`, `active_task: NONE`, V0 still
  `0 / 8`, G1 not claimed, P1 not authorized. `DIRECTORY_TREE.txt` and `SHA256SUMS` regenerated as the
  v0.6.0 record.

## Pre-G1 Sequencing Revision + P1-A0 Real Artifact Intake

- File: none — **original file unavailable, prompt supplied inline**, so there is no source file to hash.
  The repository's copy of its terms is `.ai/DECISIONS.md` ("Bounded pre-G1 P1-A0 slice — 2026-09-29"),
  `09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md` and
  `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md`.
- Version: v1.0, architect reviewed.
- Authority: `ADR-0025`, which supersedes exactly one clause of `ADR-0020`, plus one bounded, reversible
  pre-G1 slice — real artifact intake through a native dialog, optional GNU ld MAP, reuse of the already
  validated Analyze summary. `G1 = V0 PASS + P0 PASS` unchanged; P1-A0 is neither a stage gate nor P1
  progress; P1-A1/P2/P3/P4 stay unauthorized.
- Addendum applied first, also supplied inline and also without a source file to hash: the **P1-A0 Design
  Contract Closure Addendum v1.0**, which authorized design tokens `0.2.0 → 0.2.1` (one numeric semantic,
  `border.width.hairline`) and nothing else.
- Outcome, 2026-09-29: executed and pushed by the owner as `2960173`; remote Run #9 `36499759371`
  concluded `success` with 7 of 7 jobs.

## P1-A0 Evidence Identity and Persistence Correctness Closure

- File: none — **prompt supplied inline**, so no SHA-256 is recorded rather than inventing one. The
  repository's copy of its terms is `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` section 9,
  `P1_A0_VALIDATION/P1_A0_CORRECTNESS_SMOKE_REPORT.md` and `.ai/DECISIONS.md`.
- Version: v1.0, architect reviewed.
- Authority: **narrow correctness closure inside the already authorized P1-A0 slice**, explicitly not
  P1-A1 and not a new stage. It authorized fixing two defects the P1-A0 desktop smoke reported — a MAP
  that strengthened evidence in memory but not in the persisted snapshot identity, and evidence items
  claiming MAP provenance their rule did not earn — plus removing `EvidenceSummaryDto.from_map`. No new
  ADR (`00_GOVERNANCE/03_DECISION_POLICY.md` files a bug fix outside the ADR requirement), no
  `NORMALIZATION_VERSION` bump, no migration, `SCHEMA_VERSION` stays 2, and it forbade writing any
  product-scope extension.
- Outcome, 2026-09-29: executed as `ace6fbe`, pushed by the owner, measured `success` with 7 of 7 jobs on
  Run #10 `36515470263`; the governance successor is `3b59585`, green on Run #11 `36516209283`.

## V0 Batch A External Validation Activation and Interim Review

- File: none — **prompt supplied inline**, so there is no source file to hash. This is the successor to
  `FirmwareSight_V0_Batch_A_External_Validation_PROMPT_v1.0.txt`, which authorized the earlier Batch A
  takeover and withheld `v0.5.2`; that file and its SHA-256 above remain the record of the takeover.
- Version: v1.0, architect reviewed.
- Authority: a **research / evidence execution prompt, not a coding prompt**. It activates
  `active_task: V0_BATCH_A_EXTERNAL_VALIDATION`, keeps the frozen clickable prototype `v0.1.0` and
  `V0_VALIDATION/protocol/TASK_SCRIPT.md` as the formal instrument, and authorizes recruitment, screening,
  moderation, session records, metrics at real n/N and a Batch A interim review at >= 4 eligible external
  sessions. It forbids synthetic participants, AI personas and any fabricated quote, outcome, timing,
  willingness-to-pay or count, and forbids writing product code: `crates/**`, `apps/**`, `schemas/**`,
  `migrations/**`, both manifests and lockfiles, `.github/**`, the design tokens, product UI and the
  prototype itself are all out of scope, with no exception. It does not authorize P1-A1 and does not
  declare V0 or G1.
- Outcome, first invocation 2026-09-29: activation, recruitment-pack verification and the stale `v0.5.2`
  wording note only. **No real participant evidence existed, so none was written and no count moved**;
  the round stopped at `BATCH A ACTIVE — WAITING FOR REAL PARTICIPANTS`.

## Withdrawn: V0 Batch A Price Anchor Authorization + Participant Acquisition Pack

- File: none in this directory, and **no entry existed for it before this note** — the prompt was generated
  and then withdrawn before execution, so there is nothing here to hash and nothing here that ever ran.
- Version: v1.0, architect reviewed, **WITHDRAWN_BY_ARCHITECT** on 2026-09-29.
- Reason recorded verbatim from the withdrawing prompt: *open-source MVP-first direction;
  pricing/commercial validation is out of current scope.* See
  `09_ADR/ADR-0026-open-source-mvp-first-delivery.md`.
- Status: **NOT EXECUTED.** No price anchors were authored, no participant acquisition pack was produced,
  `BATCH_A_RESEARCH_PRICE_ANCHORS.md` still states that no concrete anchor is decided, both registers still
  read `NOT_RECRUITED` / `NOT_SCHEDULED`, and the V0 sample remains `0 / 8`. Nothing in this repository may
  read as though any part of it ran.

## Open-Source MVP-First Governance Reset + P1 Analyze Completion

- Supplied inline, architect reviewed, 2026-09-29. No `.txt` file: the prompt text is the owner's message,
  so this registry entry is its only in-repository record.
- Governance half: `ADR-0026-open-source-mvp-first-delivery.md`, G1 re-based on `P0 PASS`, V0 moved to
  `NON_BLOCKING_USER_FEEDBACK_TRACK`, and the withdrawal of the price-anchor prompt below recorded.
  Committed as `872ad7e governance: adopt open-source MVP-first delivery`.
- Engineering half: `P1_ANALYZE_DETAILS` - capabilities 5 to 9 of Analyze.
- Accompanying **P1 Analyze Acceptance Closure v1.0** addendum, also supplied inline and architect
  reviewed, which binds `P1 PASS` to the frozen US-001 criteria including the bytes/KiB presentation
  switch, and fixes the stop condition at the end of P1.
- Outcome, 2026-09-29: executed to its stop condition. `P1 = PASS / COMPLETE`, decided item by item in
  `P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md`, with `P1_ANALYZE_EXECUTION_REPORT.md`,
  `P1_ANALYZE_DESIGN_CHECKLIST.md` and `P1_ANALYZE_DETAILS_SMOKE_REPORT.md` beside it. Measured locally:
  184 Rust tests, 58 UI tests, `python scripts/check.py` 14/14, and a real shipped-binary window run.
  Baseline stays `0.6.0`; no migration, no dependency and no design token was added. `active_task` is now
  `NONE` and `P2 Compare` is the next authorizable stage, unstarted and unauthorized.
- Remote CI for the commits this round produced is recorded by a successor document after the owner
  pushes, never inside their own commit - the same anti-recursion rule used for P0 and P1-A0.

## P2 Compare MVP Implementation

- Prompt: *FirmwareSight — P2 Compare MVP Implementation, Execution Prompt v1.0 — Architect Reviewed.*
  Supplied inline to the execution environment on 2026-09-29, so unlike the V0 and P0 prompts there is no
  stored source file whose SHA-256 this repository can record. The registry convention holds: the fact of
  the prompt is entered, no hash is invented for bytes this repository never received.
- Authority: `ADR-0026` opened P1 → P2 → P3 → P4 on engineering grounds and each stage kept its own
  prompt requirement. This is that P2 prompt; it authorizes no ADR-level change, and none was needed —
  no technology baseline, persistence semantics, capability surface or evidence class moves.
- Scope: `P2_COMPARE` — Core-owned deterministic build diff over persisted snapshots, bounded Compare IPC
  with a session-local diff registry, the second desktop page (Analyze + Compare), `fwsight diff`, and
  portable Diff JSON v1 plus self-contained HTML. Acceptance list: the frozen US-002 criteria in
  `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:23-31` with PRD P0-3 content and its drill-down rule.
- Stop condition, stated by the prompt itself: **stop after P2.** P3 Release Gate, P4 Release Bundle,
  History, Project Settings, installer, signing, updater, cloud, accounts, telemetry, AI, pricing and
  commercial validation are all outside it.
- Start state verified before the first write: `HEAD = origin/main = 7a13660db873439f66eedee850561e8dae1cb3cf`,
  worktree clean, and Run #16 `36576568426` on that HEAD `completed` / `success` / 7 of 7 jobs, read with
  `gh run view 36576568426 --repo 2023violet/FirmwareSight`.
- Outcome: **`P2 = PASS / COMPLETE` (LOCAL PASS), closed 2026-09-29** in `P2_VALIDATION/` — exit checklist
  (US-002 item by item), execution report (every §62 command, the CLI smoke, the cross-renderer equality),
  design checklist, and the shipped-binary desktop smoke. `cargo test --workspace` 184 → 345, UI 58 → 99,
  `python scripts/check.py` 15/15, `SCHEMA_VERSION` still 2 with no `0003`, no new third-party dependency,
  no new design token, and one new public surface: `schemas/diff.schema.json`
  (`urn:firmwaresight:schema:diff:1`), contract-tested and now a compatibility promise.
  Five defects were found by running the product rather than by testing it: three visible only in a real
  window, one only by comparing the CLI export against the desktop export, and one only by reading the
  remote back — `.gitignore`'s `**/target/` had hidden the whole `fixtures/elf/p2-diff/target/` half of the
  fixture pair, so **remote Run #17 `36596452341` on the mid-round push `c7fc2a3` concluded
  `completed/failure`** while every local run passed. `cfee1e5` committed the missing half, un-ignored that
  path and added the `drift fixtures tracked` step; the owner then pushed the head and
  **Run #18 `36648718199` on `4a77ea1` concluded `success`, 7 of 7 jobs**, which closes defect E on the
  remote as well. The round's own verdict stays worded as `LOCAL PASS`, because its gate numbers were
  measured before any push — #18 verifies the tree, it did not produce the verdict. Two things stay
  open on purpose: desktop smoke step 27 was not observable in the shipped window (`PARTIAL`), and object /
  module attribution remains `Unavailable` because no stored fact attributes bytes to a source object.
  As of the entry above ("Open-Source MVP-First Governance Reset + P1 Analyze Completion"), the sentence
  *"P2 Compare is the next authorizable stage, unstarted and unauthorized"* was true at that round's close
  and is superseded by this entry: the prompt arrived the same day, and so did the closure.

## P3 Release Gate MVP Implementation

- Prompt: *FirmwareSight — P3 Release Gate MVP Implementation, Execution Prompt v1.1 — Architect
  Reviewed.* Supplied inline to the execution environment on 2026-09-29, so as with the P1 and P2 prompts
  there is no stored source file whose SHA-256 this repository can record. The registry convention holds:
  the fact of the prompt is entered, no hash is invented for bytes this repository never received.
  Version 1.1 rather than 1.0 because the architect reviewed and re-issued it after measuring the P2
  remote closure: section 0 names Run #18 on the implementation tree and Run #19 on the successor HEAD as
  the start gate, and section 1 adjudicates the five findings P1 and P2 left open as non-blocking rather
  than silently inheriting or silently reopening them.
- Authority: `ADR-0026` put P1 → P2 → P3 → P4 on engineering grounds while keeping the per-stage
  prompt requirement. Unlike P1 and P2, **this stage needs an ADR and has one**: `ADR-0027` authorizes a
  fifth first-party library crate, `crates/firmwaresight-project`, because Gate needs one shared
  implementation of project-config loading, project-relative evidence reads, the read-only system Git
  adapter and deterministic Gate input fingerprints - and `AGENTS.md` 2 / 3 forbid putting any of that in
  Core, Artifact or Storage. The prompt specifies the boundary; the ADR records it with the dependency
  table, the rejected alternatives and the measured locked versions.
- Scope: `P3_RELEASE_GATE` - `firmwaresight.toml` v1 frozen at `schema_version = 1` with `[gate]` and
  `[gate.on_unknown]`, `schemas/project-config.schema.json`, a read-only Git provenance adapter with a
  bounded timeout, Core Gate semantics over the five states frozen by ADR-0023, the ten PRD P0-5 rules
  under stable ids, deterministic `gate-<sha256>` run identity, additive migration `0003_gate_history.sql`
  (`SCHEMA_VERSION` 2 → 3) with immutable GateRun and immutable Review acceptance, portable
  `gate-results` v1 and backward-compatible `accepted-reviews` v1, `fwsight gate`, and the third desktop
  page. Acceptance list: the frozen US-003 criteria in
  `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:33-40`, the PRD P0-5 ten built-in checks at
  `01_PRODUCT/01_PRD_MVP.md:66-77`, and `06_DELIVERY/08_MILESTONE_DELIVERABLE_MATRIX.md:22`'s exit
  evidence - rule matrix plus contract tests.
- What it deliberately does not resolve: the **open-source license gap**. The repository is delivered as
  open source under ADR-0026 while the root `Cargo.toml`'s `[workspace.package]` table reads
  `license = "Proprietary"` and no root `LICENSE`
  file exists; `AGENTS.md` 9 puts a license change in front of a human, and this prompt declines to choose
  one. Recorded as `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`, non-blocking for engineering,
  and visible in the completion report.
- Stop condition, stated by the prompt itself: **stop after P3.** P4 Release Bundle, `release prepare`,
  a bundle chooser, manifest generation, bundle checksums, a bundle release report, History page, Project
  Wizard, installer, signing, updater, SBOM, CVE, OTA, flashing, HIL, cloud, accounts, telemetry, AI
  judge, pricing and commercial validation are all outside it. P3 may produce Gate JSON and accepted
  review audit records; P4 packages them later.
- Start state verified before the first write: `HEAD = origin/main =
  32b23aa78323d315f6643f04c2343f75576d483f`, worktree clean, and Run #19 `36665007523` on that HEAD
  `completed` / `success` / 7 of 7 jobs, read with
  `gh run view 36665007523 --repo 2023violet/FirmwareSight`. The sealed P2 implementation tree
  `4a77ea1` is green on Run #18 `36648718199`, also 7 of 7. `git diff --name-only 4a77ea1 32b23aa` was
  checked and lists twelve documentation, governance and integrity files - no production source, fixture,
  schema, migration or configuration file - so the newer HEAD is the same tree for engineering purposes.
  Start counts re-measured, not inherited: `cargo test --workspace` **345 passed / 0 failed / 0 ignored**,
  `corepack pnpm test` **99 passed**.
- Outcome: **`PASS / COMPLETE`, closed 2026-09-30.** US-003 and the PRD P0-5 ten checks are settled item
  by item in `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md` — §69's forty boxes, each with the `gh` query,
  command, named test or smoke step that decides it. Measured at closure: `cargo test --workspace` **556
  passed / 0 failed / 0 ignored across 28 executable suites** (from 345 at the start), `corepack pnpm test` **135
  passed in 6 files** (from 99), `python scripts/check.py` **15/15**, an 18/18 CLI Gate smoke, and all
  forty §61 desktop steps walked on the shipping `custom-protocol` binary. The stop condition held: no
  bundle, no History page, no pricing, cloud, account, telemetry, updater, signing, SBOM or AI judge
  exists, and `release` / `watch` / `doctor` remain unregistered.
  **The verdict was `LOCAL PASS` as worded at closure, and the wording has since been overtaken.** The
  five P3 commits were unpushed when §69's boxes were settled, so no CI run covered P3's tree and none
  was claimed. They were pushed afterwards: `origin/main` moved to `219178af195569ec6b13728d84d992ef78df8c04`
  and Run `36774472141` completed `failure` with **6 of 7 jobs green** — Rust on both platforms, Desktop UI
  on both platforms, the macOS core smoke and the drift check all passed, and only `Dependency policy`
  failed, on `error[yanked]` for `yoke-derive 0.8.3`, a transitive proc-macro under `url` that crates.io
  yanked at 2026-09-30T13:19:39Z, after the local deny step had passed against an older index. The fix is
  a patch bump to `0.8.4` in a successor commit, with the gate re-run 15/15 and the per-suite test counts
  unchanged. That closure entry first read **715 passed across 42 suites**; the figure summed one gate log
  in which the `drift` group re-runs the desktop crate after `cargo test --workspace`, so it counted those
  159 tests twice, and it is corrected above rather than replaced quietly. The remote then produced two
  more runs: `36779321479` on `2d1bcea` concluded **success, 7 of 7**, which is the yank fix verified on
  CI's index, and `36779715108` on `893a635` concluded failure on one job, `Desktop UI (windows-latest)`,
  where a `compare.test.tsx` call-count sample lost a race the ubuntu job won on the same commit — defect
  J, fixed by sampling after both change tables resolve, with the delta assertions unchanged and the fix
  proved by mutation rather than by a green run. That fix's own push, Run `36783457030` on `02e8a81`,
  concluded **success with 7 of 7 jobs**, and its documentation-only successor `f66a93d` matched it on Run
  `36784382005` — the last run this pack names, because recording a doc-only successor's own result would
  need another successor. Three product defects were found by running the stage rather than by
  testing it — a version pattern quoted into an evidence locator, which made a Gate run unpersistable for
  any project using the only MVP version source; one build carrying two different run ids across the CLI
  and the desktop; and a disabled primary button keeping its accent border — and each is fixed with a
  named regression test. Two things stay open on purpose: desktop smoke step 30's prior-run re-read was
  never observable through the window. The round's first full gate also came back **13/14** on
  `frontend/test` — a `compare.test.tsx` row query racing the table's own `Loading symbol changes…`
  render, which is a test-side race in P2's file rather than product behaviour; it was diagnosed, fixed
  by awaiting the query with the assertion unchanged, and the gate re-run to 15/15.

## P4 Release Bundle

- File: `FirmwareSight_P4_Release_Bundle_MVP_Implementation_EXECUTION_PROMPT_v1.0_ARCHITECT_REVIEWED.txt`
- SHA-256: `1baaec9204a1d2aa5aa53bd735b34d376ee56db7557a04d6abb79265c30840c5`
  (recomputed from the archived copy in this directory, and equal to the bytes as delivered: the file was
  already LF-only, so no text conversion moved it. This is the first stage since P0 whose prompt arrived as
  a real file rather than inline text, so unlike the P1, P2 and P3 entries this one records a hash instead
  of recording that no hash can be recorded.)
- Authority: `ADR-0026` put P1 → P2 → P3 → P4 on engineering grounds while keeping the per-stage prompt
  requirement. This is that prompt for the last core product-implementation stage of the MVP. **No new ADR
  is required and none was written**: P4 moves no technology baseline, adds no crate, and changes no
  persistence semantic that `04_TECH/15` and `AGENTS.md` 6 did not already anticipate — `release_records`
  is a table §3 of that document has listed since v0.5, and migration `0004` is additive. The one boundary
  it moves with a decision behind it is `SCHEMA_VERSION` 3 → 4, the same way P3 moved 2 → 3.
- Scope: `P4_RELEASE_BUNDLE` — assemble an already-trusted Analyze + Compare + Gate + accepted-Reviews
  result plus the selected firmware artifacts and Release Notes into a directory bundle that is portable,
  independently readable without FirmwareSight, hash-verifiable, host-path-free, and never overwrite-by-
  default. Canonical layout `artifacts/*`, `analysis.json`, `diff.json` when a baseline exists,
  `gate-results.json`, `accepted-reviews.json`, `release-notes.md` when observed, `release-report.html`,
  `SHA256SUMS`, `release-manifest.json`. New: Core `domain/release.rs`, the public
  `urn:firmwaresight:schema:analysis:1` projection, `ReleaseManifestDto` over the existing v1 schema,
  additive `0004_release_records.sql`, `fwsight release prepare`, and a Bundle section under the existing
  Release page. Reused unchanged: `gate-results:1`, `accepted-reviews:1`, `diff:1`,
  `release-manifest:1`, `project-config:1`, and P3's Gate evaluation — §9 requires the bundle to
  *recompute* the Gate and refuse when the run id no longer matches, and §8 requires disposition PASS.
- Acceptance list: the frozen **US-004 Export Bundle** criteria in
  `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:42-49` (readable without FirmwareSight, manifest carries every
  file hash, report names the FirmwareSight version, no existing directory overwritten without explicit
  confirmation), plus **PRD P0-6 Release Bundle** at `01_PRODUCT/01_PRD_MVP.md:85-95` for the file set, and
  `01_PRODUCT/01_PRD_MVP.md:126`'s "Release Bundle 可在另一台机器上独立阅读".
- What it deliberately does not do: **declare G2.** The only permitted end state is
  `G2 = READY_FOR_ENGINEERING_GATE_REVIEW`; the architect performs the whole-MVP G2 closure audit in a
  later round. It also leaves the open-source license gap exactly where P3 left it (§76 forbids touching
  `license = "Proprietary"` or adding a `LICENSE`), and it claims no signing: §62 forbids the words
  *trusted*, *authentic*, *signed* and *tamper-proof*, because a SHA-256 proves byte consistency against an
  included manifest and authenticates nobody.
- Stop condition, stated by the prompt itself: **stop after P4.** History page, Project Wizard, installer,
  signing, notarization, updater, SBOM, CVE, OTA, flashing, HIL, cloud, accounts, telemetry, AI, pricing and
  commercial work are all outside it, as are `v0.7.0`, P5, V1, B1, RC1 and GA1.
- Start state verified before the first write: the prompt's §0/§2 anchor is
  `ba5e59e79208969a4687db11e69fdef2ababd8ab`, green on Run #25 `36785425648` (completed / success / 7 of 7).
  `origin/main` had moved one commit further when this round started: `HEAD = origin/main =
  323afad155348bd5b721fa2b7e2ed2388bdc2691`, worktree clean. `git diff --name-only ba5e59e 323afad` lists
  eleven documentation, governance and integrity files plus the two newly tracked baseline-artifact
  scripts, and **no** production source, fixture, schema, migration or configuration file — so the newer
  HEAD is the same tree for engineering purposes and is the start fact this round records, the same way P3
  recorded Run #19 on its successor HEAD. Run `36810689645` on `323afad` is `completed` / `success` / 7 of 7,
  read with `gh run view 36810689645 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs`.
  Start counts re-measured, not inherited: `cargo test --workspace` **556 passed / 0 failed / 0 ignored**,
  `corepack pnpm test` **135 passed in 6 files** — which matches the prompt's §0 figures, and confirms the
  retired `715` was not reintroduced.
- Closure: **executed and closed `PASS / COMPLETE` on 2026-10-01.** The item-by-item US-004 verdict is
  `P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md` (sixty-one boxes), the shipped-window result over all fifty
  §59 steps is `P4_BUNDLE_DESKTOP_SMOKE_REPORT.md`, and `G2` was written only as
  `READY_FOR_ENGINEERING_GATE_REVIEW`. Closing counts on the final tree: `cargo test --workspace`
  **769 passed / 0 failed** in 41 executable suites, `corepack pnpm test` **155 passed in 6 files**; the
  final implementation head `e799f2f` is green on Run `36872456446` at 7 of 7 jobs, and `active_task`
  returned to `NONE` with baseline still `0.6.0`.

## P4 Governance Baseline Consistency Closure

- Title: *FirmwareSight — P4 Governance Baseline Consistency Closure, Execution Prompt v1.0 — Architect
  Reviewed*. **Supplied inline**, so no source file exists in this repository to hash and none is
  reconstructed here.
- Scope: one governance defect — `BASELINE.yaml` kept P4's running-state live fields (`product.status`,
  `active_task`, `p4_execution.stage_status` / `g2_status`) after every other surface had recorded P4
  closed. No product code, schema, dependency or design change.
- Outcome, 2026-10-01: `fe420d1` changed `BASELINE.yaml` and `SHA256SUMS` only. Run `36892307828`
  attempt 1 failed on one job — a test-side race in `compare.test.tsx` that this commit did not touch —
  and attempt 2 on the same SHA was 7 of 7. That race was closed by the next round.

## Pre-G2 Compare UI Test Reliability Closure

- File: `FirmwareSight_Pre_G2_Compare_UI_Test_Reliability_Closure_EXECUTION_PROMPT_v1.0_ARCHITECT_REVIEWED.txt`
- SHA-256: `dee5b8e4417b96cf5b4089b8d609042151ed6be310fa86f42c72353abf45af9e` — 18,306 bytes, 817 lines,
  LF only. Archived byte-for-byte from the delivered file during the G2 round (the G2 prompt §5 asked for
  it); `cmp` against the delivered copy is clean.
- Scope: one test — `compare.test.tsx` "pages with the offset the shell reported…", which read the pager
  synchronously after the region mounted and before its first page arrived.
- Outcome, 2026-10-01: `055b54e` (test only, plus `SHA256SUMS`). Root cause reproduced with a 25 ms mock
  latency, the repaired assertion proved by two mutations, target 30/30, file 10/10, UI 155, Rust 769,
  `check.py` 15/15. Run `36899128645`: attempt 1 7 of 7, and two further Windows UI executions green on the
  same SHA. `TEST_RELIABILITY_CLOSURE = PASS`.

## G2 Product MVP Engineering Closure Audit

- File: `FirmwareSight_G2_Product_MVP_Engineering_Closure_Audit_EXECUTION_PROMPT_v1.0_ARCHITECT_REVIEWED.txt`
- SHA-256: `3c6ab83e11ce6a4c91a609dd0a6bc318e700dc0cbbccd3d4493eaf65c3e2bac9` — 44,539 bytes, 2,105
  lines, LF only, archived byte-for-byte; `cmp` against the delivered copy is clean.
- Authority: `06_DELIVERY/06_STAGE_GATES.md` §G2 as amended by `ADR-0026`. The whole-MVP closure audit
  that P4's own prompt forbade that round to perform or claim. Audit-first; narrow remediation only inside
  existing MVP contracts; no new feature, verb, schema major, migration, dependency, crate, token,
  licence choice, network or P5 behaviour.
- Start: `055b54e`, green on Run `36899128645` attempt 1.
- Outcome: see the addendum entry below and `G2_VALIDATION/`.

## G2 Storage Path Semantics Clarification Addendum

- Title: *FirmwareSight — G2 Storage Path Semantics Clarification Addendum, Addendum v1.0 — Architect
  Reviewed*, applying to the G2 prompt above (`3c6ab83e…`). **Supplied inline**, so no source file exists
  here to hash and none is reconstructed.
- Scope: it overrides two things only. (1) The G2 prompt's §13 / §34 "no host path persisted" is
  corrected to the boundary the product always had: SQLite's local-only `artifacts.path` is allowed
  (`04_TECH/15` §7, P1-A0); IPC, UI, Gate locators, `release_records`, the five portable documents and the
  bundle carry no host path. (2) The audit head moves to `e35cfe7`, the G2-F1 test-only fix. It authorizes
  no migration, no schema change and no storage redesign.
- Effect: G2-F2 adjudicated `EXPECTED_LOCAL_PRIVATE_PERSISTENCE`, non-blocking; the 17-point boundary proof
  is `G2_VALIDATION/G2_EVIDENCE_MATRIX.md` §7.
- Outcome, 2026-10-01: `G2 LOCAL PASS / READY FOR REMOTE CLOSURE` in the evidence commit `f75cbc5`, whose
  own Run `36948719972` was 7 of 7 on its first attempt (the product tree `e35cfe7` likewise, Run
  `36906482900`). The one successor commit the prompt allows records that and closes **G2 = PASS —
  Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE**, with `active_task` back to `NONE`. The
  successor's own run is external evidence and is not recorded in a further commit.

## Post-G2 Real Desktop MVP End-to-End Acceptance v1.1

- File: `FirmwareSight_PostG2_E2E_Acceptance_v1.1.txt`
- SHA-256: `8867ff7ab10c440847ee20437d60dcc5d5023b972f8831316970d590f2653e5d` (42,378 bytes)
- Authority: an observation round on a tree already marked G2 PASS. It authorized black-box real-desktop
  acceptance only - TEST FIRST / OBSERVE / RECORD / DO NOT FIX - and its section 18 forbade a code
  change, a commit, a push, P5, and fixing what it found. It capped the verdict at
  `PASS_WITH_FINDINGS` while an open finding stood, required the owner's live store to be restored
  byte-exact, and named 125/150 % DPI, mouse-wheel, cold-disk latency and peak RSS as things to record
  rather than fix.
- Outcome, 2026-10-02: 283 cases, `REAL_DESKTOP_E2E = PASS_WITH_FINDINGS`, three findings
  (`E2E-F001` S3, `E2E-F002` S2, `E2E-F003` to be judged from evidence), no S0 or S1,
  `ACTUAL_EXPECTATION_FAILURE_COUNT = 6`, `ORIGINAL_DB_RESTORED = YES` and
  `ORIGINAL_DB_SHA_MATCH = YES`. Its recommendation was narrow post-G2 remediation, and the round made no
  commit - which is why this archive holds its prompt but no `POST_G2_E2E/` evidence directory: the
  evidence root lives under `%TEMP%`, outside the repository, by that prompt's own instruction.

## Post-G2 E2E Findings Remediation and Focused Re-Validation v1.0

- File: `FirmwareSight_PostG2_E2E_Findings_Remediation_v1.0.txt`
- SHA-256: `7e98cfbfbda6e57db37088ec68ac41c068c3a14458abe15e8f4c4f3f4c8a45d1` (31,908 bytes)
- Authority: fix the evidence-backed product defects the acceptance round observed, prove each with a
  targeted regression and a focused real-desktop re-check, and stop. It made the evidence files - not
  chat memory - the primary input, required a Takeover Report before any write, put `E2E-F003` behind an
  eight-condition include/exclude gate with "do not invent a fix", forbade performance work, schema,
  migration, dependency, architecture and P5 scope, and reserved any bundle-engine semantics change for
  Architect review. It listed 125/150 % DPI, mouse-wheel infrastructure, the E2E harness refactor, the
  `update_goldens` issue and licence selection as out of scope, and required remote CI to be green at 7
  of 7 on the first attempt for both the fix head and the closure successor.
- Outcome, 2026-10-02: `POST_G2_E2E_FINDINGS_REMEDIATION = PASS`. `E2E-F001` and `E2E-F002` fixed in
  `e816dcb`, `E2E-F003` included on the gate's evidence and fixed in `971015f`; 770 Rust / 159 UI /
  `check.py` 15/15; fix head Run #43 `37100371601` completed / success / 7 of 7 on attempt 1. Evidence in
  `POST_G2_E2E_REMEDIATION/`. G2 stays PASS, the product stays MVP CANDIDATE, the baseline stays 0.6.0,
  and the prompt's last instruction - do not start P5, return to the Architect, stop - is what the
  closure commit did.

## P5 Productization to Productized MVP v1.0

- File: `FirmwareSight_P5_Productization_v1.0.txt`
- SHA-256: `722125f5aa68e324ba1dea4826f66d8392acab9ad9a015c4919ed5ade471e0ae` (73,722 bytes, 3,442 lines,
  86 numbered sections) — recomputed from the archived copy in this directory and byte-compared against
  the file the owner supplied, so the registered hash is the hash of the stored bytes, not of a recollection.
- Authority: turn the G2-passed MVP Candidate into a **Productized MVP** a stranger engineer can install,
  understand, use (Analyze / Compare / Gate / Bundle), inspect in local History and Diagnostics, recover
  from, and uninstall or reinstall — without the dev team present. Its discipline is AUDIT FIRST / NO
  FEATURE SPRAWL / LOCAL-FIRST / FAIL CLOSED / PRESERVE UNKNOWN / PACKAGE WHAT WAS PROVEN, and §4 forbids
  any product code before `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` exists. It opens with a hard git
  preflight and a STOP if the remote moved, forbids `reset --hard` / `clean -fd` / `stash` / `restore .` /
  rebase of published history / force push, and requires a fetch plus a re-read of HEAD and `origin/main`
  after every commit. It caps the verdict: **P5 PASS is not BETA, RC or GA**, and §5 forbids writing any
  of those words before closure evidence exists.
- Explicit boundaries this round must not cross: no new network capability, telemetry, analytics SDK,
  updater or update endpoint; native dialogs stay Rust-side; no generic shell, `read_file` or SQL IPC;
  CSP stays restrictive; Git access stays read-only; no signing or notarization execution (§11 fixes the
  status at `READY_NOT_EXECUTED`, §12 at `UPDATE_READY_MANUAL`, and both forbid a fake certificate or a
  committed private key); no tag, GitHub Release or published installer (§72). §54 is absolute about the
  licence: the **P5 agent must not choose** MIT / Apache-2.0 / GPL / AGPL / MPL, so
  `OPEN_SOURCE_LICENSE_DECISION = PENDING_OWNER_CONFIRMATION` carries through every P5 document. Diagnostics
  is allowlist-only and must never contain an absolute source path, the project root, firmware bytes, MAP
  contents, Release Notes, a Git remote, the user name, the home directory, a token dump, the environment,
  the database path or the bundle destination — and §46/§66 require *positive-control* tests for that, not
  merely the absence of a leak. Security wording is fixed: never "security clean"; the permitted sentence
  is "dependency policy passes with documented accepted risks".
- Deliverables it requires: `P5_VALIDATION/` with 12 named documents (the audit, execution report, exit
  checklist, packaging, install-recovery, migration-recovery, compatibility matrix, history-diagnostics,
  desktop-acceptance, security-supportability, known-limitations and CI-authority records), 19 end-user
  documents, the §26 fixture expansion inside the existing GCC/Clang ELF + GNU ld MAP cohort, and the §61
  six-commit split — "do not make one giant P5 commit".
- Status at archive time, 2026-10-03: §5 governance is opened by this commit (`active_task:
  P5_PRODUCTIZATION`, stage `P5`, state `IN_PROGRESS`, `baseline_version` unchanged at `0.6.0`), the audit
  is written in `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md`, and the owner's checkpoint answered four of its
  questions: artifacts unify on **0.6.0** with the workspace version as the single source; migration
  **0005** follows a written `P5_MIGRATION_DECISION.md` (and never before it); the package matrix is
  **Windows with real install evidence on this host, macOS and Ubuntu `CI_BUILD_ONLY`**; and the shipped
  store keeps its existing name `firmwaresight-p0.sqlite` instead of a data move, with its path made
  visible instead. Two are not this round's to decide: §32 sends the release-identity-versus-line-endings
  question (L22) to the Architect as `P5_RELEASE_IDENTITY_ADR_DRAFT.md` with a STOP, and §54 leaves the
  licence with the owner. No P5 verdict exists yet, and none is written here.

## P5 Commit D Diagnostics and Recovery v1.0 — SUPERSEDED, HISTORICAL

- File: `FirmwareSight_P5_CommitD_Diagnostics_Recovery_v1.0.txt`
- SHA-256: `5ec32b8b3d88dd512a9fb4b6aeab56bcf0f5f40f12559c8b671801acd8a8105f` (35,119 bytes, 1,355 lines) —
  recomputed from the archived copy in this directory.
- Authority: the **first** Commit D round. It authorized the preflight, the authority read of the storage
  core, the IPC surface and the audit's section D, the dependency baseline, and the same five workstreams
  the continuation later restated: sanitized Diagnostics, storage-owned integrity health, a consistent
  pre-migration backup, migration recovery evidence, and logical store identity.
- Disposition, corrected on 2026-10-04. The first round archived this file *as the authority of Commit D*,
  and that was wrong: the continuation prompt superseded it as the authority for the remaining work before
  the storage half was finished. It stays here because its bytes are the record of what the first round was
  told — not because it governs anything now. `P5_VALIDATION/P5_COMMIT_D_DESIGN.md` §0 carries the
  correction, and the design decisions that survived from this prompt are the ones the continuation restated.

## P5 Commit D Continuation v1.2 — COMPLETE, HISTORICAL

- File: `FirmwareSight_P5_CommitD_Continue_v1.2.txt`
- SHA-256: `600d70a358778280b667d1c423b7309079f3a5b575ad0c4b919b75de8c274033` (31,588 bytes, 1,472 LF
  endings) — recomputed from the archived copy, `cmp`-verified byte-identical to the file the owner
  delivered on 2026-10-04.
- Authority: continue the in-flight Commit D without redoing its valid work; correct the D-0 authority
  evidence first; decide L22 as `ADR-0028` (Accepted) and propagate that decision; then implement the five
  workstreams §4 names under the §7 test-first gate, §23's mutation proofs, §24's dependency and capability
  STOP rules, §26's "never touch the owner's live store while developing" and §27's focused installed-app
  walk, and stop at §34. Its §0 re-anchors the round at `2cdfced` with Run `37159810291` as the
  authoritative 10-of-10 CI, and its baseline numbers (Rust 813, UI 201, local gate 16/16) are the ones this
  round was measured against.
- The v1.1 question, stated rather than papered over. §1 of this prompt names
  `FirmwareSight_P5_CommitD_Continue_v1.1.txt` — sha256 `f38ee5f2c1d640058f5ed84e5ef00f78577bd6c0d68536aca5e9ff4c7e8ab045`,
  27,834 bytes, 1,014 lines — as the canonical active authority and instructs the executing agent to locate
  those exact bytes, verify the hash, and STOP with `AUTHORITY_PROMPT_BYTES_UNAVAILABLE` if they cannot be
  recovered. They cannot be recovered on this machine: no copy exists in the owner's downloads, in this
  repository, or in the temp tree, and the bytes were never delivered to any round here. The prompt's own
  fallback was therefore taken — no reconstruction from memory, no byte-exact claim, no claim that v1.1 was
  archived — the measurement and the stop were reported, and the owner chose to archive **v1.2** as this
  round's authority. What v1.1 said is not nothing: v1.2's §1 restates its content and §2 lists the work it
  had already validly produced, which is what made the continuation safe to resume without a redo.

## P5 Commit E Compatibility Fixtures, Support Matrix & Supportability Closure v1.0 — ENGINEERING COMPLETE, ACTIVE AUTHORITY NOW SUPERSEDED BY THE NORMALIZATION PROMPT BELOW

- File: `FirmwareSight_P5_CommitE_Compatibility_Supportability_v1.0.txt`
- SHA-256: `030ca28233b964958d6aea8e59a312c66ceb4d117273cba9e667950b4ba21148` (53,915 bytes, 2,459 LF
  endings, 65 numbered sections) — recomputed from the archived copy in this directory and `cmp`-verified
  byte-identical to the file the owner delivered on 2026-10-04.
- Authority: the canonical unit `P5_COMMIT_E_COMPATIBILITY_SUPPORTABILITY` and nothing else. It names what it
  is **not** — not a Commit D extension, not Commit F, not P5 closure, not V1, not B1/RC/GA, not a format
  expansion, not a parser rewrite — and its discipline is REAL FIXTURES / NO HAND-EDITED EVIDENCE / CLAIM
  ONLY WHAT WAS PROVEN / NO FORMAT SCOPE EXPANSION / NO P5 CLOSURE / NO L26 FIX / NO FULL §64 JOURNEY. Its
  work is to strengthen evidence **inside** the frozen GCC/Clang ELF + GNU ld MAP cohort (§6 forbids Keil,
  IAR, TI COFF, HEX, BIN, UF2, Mach-O, PE and universal MAP shapes, and forbids a DWARF semantic-analysis
  feature), produce `P5_COMPATIBILITY_MATRIX.md`, `P5_COMPATIBILITY_FIXTURE_REPORT.md` and
  `P5_SUPPORTABILITY_REPORT.md`, and disposition §13's eight layout cases A–H with four words only.
- The start-authority delta, measured rather than smoothed. §0 and §2 fix the start at
  `origin/main = 956e250`, Run `37204847474`, 10 of 10, and §2 requires a STOP plus a classification if the
  remote moved. It had moved two heads: `57a904e` and `bfbc1aa`, ten files, 98 insertions and 30 deletions,
  every one of them `.ai/`, `P5_VALIDATION/`, `README.md`, `INDEX.md` or `SHA256SUMS` — **no** product
  source, fixture, schema, migration, script, configuration or workflow path. The behaviour tree Commit E
  builds on is therefore the one §0 describes, and `bfbc1aa` — green 10 of 10 on Run `37214675036`, first
  attempt, all ten jobs read individually — is the HEAD the round actually starts from. Nothing was rewound;
  no published history was rewritten.
- What it reserves for Commit F (§1, §63): the full §64 installed journey (a Commit E packaged check is
  allowed but must be labelled `FOCUSED_COMMIT_E_PACKAGE_CHECK`, never an installed-journey pass), the L26
  `SHA256SUMS` semantic decision — explicitly not to be fixed here, not by blob bytes, not by normalizing
  line endings, not by wiring the verifier into CI, and not by editing `ADR-0028` — and the two consolidation
  documents `P5_MIGRATION_RECOVERY_REPORT.md` and `P5_HISTORY_DIAGNOSTICS_REPORT.md`.
- Where it must stop mid-round: §28 sends L8 back to the Architect if closing it needs a new MAP
  grammar/parser subsystem; §32 stops the L15 subproblem if the change would move serialized evidence
  semantics; §42 stops any fix that needs a new portable field, a renamed serialized enum, a schema version
  or a release-identity change, and asks for `P5_COMMIT_E_SCHEMA_DECISION.md` in its place; §46 stops before
  any new product dependency or capability. §9 is absolute about provenance — a fixture is genuine
  compiler/linker output, and hand-edited ELF bytes, hex-patched headers, a hand-written MAP, padding bytes
  dressed up as layout, or a GCC ELF relabelled as Clang are all forbidden. §45 forbids regenerating
  existing goldens as a routine step, and §50/§53 forbid counting repetition campaigns into the direct test
  totals. §22 closes with three honest Clang outcomes and no fourth: blanket `SUPPORTED` without a fixture is
  not one of them.
- Its own outcome, recorded before the next entry is read: engineering `PASS` at candidate `59d85c3` on Run
  `37228929762` (attempt 1, 10 of 10, every job and step read), read back by `6981625` on Run `37230689636`
  (attempt 1, 10 of 10). Two evidence-contract mismatches survived that — the matrix status column and the A–H
  disposition column had drifted outside the vocabularies this prompt itself required — and §42's STOP was
  answered by the Architect rather than by this round. Both are what the entry below exists to settle.

## P5 Commit E Closure Normalization v1.0 (Evidence Vocabulary + L15 Architect Decision) — CURRENT ACTIVE AUTHORITY

- File: `FirmwareSight_P5_CommitE_Closure_Normalization_v1.0.txt`
- Two hashes, both measured, because the delivered bytes and the stored bytes differ in one respect only.
  Delivered to the agent on 2026-10-04 as **28,738 bytes with 1,372 CRLF terminators** and a final unterminated
  line: SHA-256 `9571df2cc08107ea134ed89acc4a984254803b40e451c57c6e9c3ca480075599`. The copy in this directory
  is the same content normalized to **27,366 bytes, 1,372 LF, 1,373 content lines**: SHA-256
  `62040e3d02fffe0f7682909828e4a2d6a91d81b09a937c0bd0b4765e6c7af887`. Verified with `cp` + `cmp` at archive time
  and then split line by line against the delivered file (1,373 lines each, content `identical: True`), so
  nothing was reconstructed and no line was edited — the only difference is the terminator, which
  `.gitattributes`' `*.txt text eol=lf` rule requires for every text file in this repository (that rule was
  **not** modified to preserve CRLF: §1 does not allow a line-ending or integrity-policy change, and
  `AGENTS.md` 9 puts such a change in front of a human). Consequence, stated rather than hidden: the delivered
  CRLF hash does **not** reproduce from a checkout of this directory, and this is the first archived prompt here
  whose delivered bytes needed normalization — the other thirteen arrived LF.
- Authority: a **documentation-only** closure normalization of Commit E and nothing else. §1 lists what it may
  touch (`P5_VALIDATION/*.md`, `.ai/*.md`, `04_TECH/*.md` only where L15 legacy semantics need documenting,
  `README.md`, `INDEX.md`, `10_AUDIT/SOURCE_PROMPTS/*`, `DIRECTORY_TREE.txt` because archiving adds a path, and
  `SHA256SUMS` last) and what must stay empty: `crates/**`, `apps/**`, `scripts/**`, `fixtures/**`,
  `schemas/**`, `golden/**`, `.github/**`, `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`,
  `tauri.conf.json`, `deny.toml`, `migrations/**`, any generated TS binding, any ELF/MAP/binary fixture. If a
  forbidden path appears the instruction is STOP, not "fix it while here" — and §19 fixes the expected direct
  counts at **Rust 868 / UI 217**, so a test-count move in this round is itself the alarm.
- Its four jobs: normalize the matrix status column to
  `SUPPORTED / SUPPORTED_WITH_LIMITS / CI_BUILD_ONLY / NOT_TESTED / UNSUPPORTED` with nuance confined to the
  evidence column (§5, §6, §23A); normalize the A–H dispositions to
  `PROVED_BY_EXISTING_FIXTURE / PROVED_BY_NEW_FIXTURE / SUPPORTED_WITH_LIMITS / NOT_AVAILABLE` with the
  architect's own mapping — A, F and H **existing**, B, C, D, E and G **new**, because H's multiple `PT_LOAD`
  and F's `-g` builds were already in the cohort and a new assertion is not a new fixture (§7, §23B); record
  the Architect's **L15 Option E** (§8–§12); and reconcile the governance files (§17).
- The L15 answer, in one line: `SourceType::ElfProgramHeader` and the wire token `elf.program-header` are
  **kept** as legacy `analysis:1` identifiers, with `analysis:1` / `diff:1` / `gate-results:1` /
  `accepted-reviews:1` / `release-manifest:1` / the SQLite schema / `SCHEMA_VERSION` / goldens / fixture bytes /
  release identity / `ADR-0028` all unchanged — **no migration 0006, no `analysis:2`, no wire rename** — and the
  accurate human meaning, "ELF address + flags evidence", documented instead. L15 therefore becomes
  `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`, and §11 forbids marking it `CLOSED`: the identifier stays
  technically inaccurate; the risk is now bounded and written down rather than removed.
- Hard stops that survive this round: §14 keeps **L26** undecided (no blob-byte sums, no canonical checkout
  bytes, no baseline verifier wired into CI, no `ADR-0028` edit for repository baseline checksums), §15 forbids
  the §64 installed journey and any owner-store park/restore or package installation, §16 keeps Commit F's
  consolidation documents unwritten, and §18 caps the outcome vocabulary: before this head's CI,
  `Commit E evidence closure = NORMALIZATION_PENDING_REMOTE_CI`; after 10 of 10, `COMMIT_E = FINAL PASS /
  COMPLETE` with `P5 = IN_PROGRESS` and `Commit F = NOT_AUTHORIZED`. §27 then forbids a further commit merely to
  record that run: it is external evidence, reported back in chat.
- Where the verdict is written: `P5_VALIDATION/P5_COMMIT_E_CLOSURE_NORMALIZATION.md` (§24), whose Remote CI
  section says `PENDING` because the document is committed before the run exists.

## Supersession note on the V0 Batch A activation entry

The section "V0 Batch A External Validation Activation and Interim Review" above records a round that
**was** executed on 2026-09-29 and whose outcome stands: activation happened, the recruitment pack was
verified, no participant evidence was written, and the round stopped at
`BATCH A ACTIVE — WAITING FOR REAL PARTICIPANTS`. Its factual content is unchanged.

What no longer stands is its sequencing authority. `ADR-0026` moved V0 to
`NON_BLOCKING_USER_FEEDBACK_TRACK`, so `active_task` has since passed from `V0_BATCH_A_EXTERNAL_VALIDATION`
to `P1_ANALYZE_DETAILS` - which completed the same day, leaving `active_task: NONE` - and no Batch A
session count gates any engineering stage any more. The
recruitment-ready pack, the frozen `v0.1.0` prototype and the protocol files stay in place for a later,
non-blocking feedback round.
