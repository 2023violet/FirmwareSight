---
title: "Execution Prompt Register"
doc_id: "FS-AUDIT-SRC-003"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Audit"
last_updated: "2026-09-30"
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
  open source under ADR-0026 while `Cargo.toml:17` reads `license = "Proprietary"` and no root `LICENSE`
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
