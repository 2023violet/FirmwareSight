---
title: "Document Index"
doc_id: "FS-ROOT-INDEX"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-10-04"
---


# Document Index

Baseline: `0.6.0` — the P0 Technical Foundation Baseline.

Execution state: `P0: PASS — frozen (remote CI Runs #3, #4, #5 and #6 all 7 of 7 green) · Pre-G1 P1-A0 COMPLETE, including its evidence-identity and persistence correctness closure (Runs #9, #10, #11 all 7 of 7 green) · G1 PASS on the P0 basis under ADR-0026 (2026-09-29) · V0 NON_BLOCKING_USER_FEEDBACK_TRACK 0/8, an honest zero that gates no stage · P1 PASS/COMPLETE — the Analyze verb, evidence in P1_VALIDATION/, remote CI Run #13 `36556735551` on `e63afaf` 7 of 7 green · P2 PASS/COMPLETE — Compare, evidence in P2_VALIDATION/; its mid-round push `c7fc2a3` failed remote Run #17 `36596452341` because `.gitignore` hid half of the P2 fixture pair, `cfee1e5` fixed it, and the head `4a77ea1` is green on Run #18 `36648718199` 7 of 7 · P3 PASS/COMPLETE — Release Gate, authorized 2026-09-29 by execution prompt v1.1 plus ADR-0027 and closed 2026-09-30, evidence in P3_VALIDATION/; 556 Rust tests in 28 executable suites, 135 UI tests, 15/15 gate steps, an 18/18 CLI Gate smoke and all forty §61 desktop steps on the shipping binary — closed as LOCAL PASS while its commits were unpushed, then pushed: Run `36774472141` on `219178af` completed failure with 6 of 7 jobs green, only `Dependency policy` red on a crate yanked on crates.io after the local deny step had passed on a stale index, fixed by a `yoke-derive` patch bump, itself verified by Run `36779321479` at 7 of 7 green; the next push `893a635` went red on `Desktop UI (windows-latest)` alone — a call-count race in `compare.test.tsx` (defect J) that the ubuntu job won on the same commit, fixed with its assertions unchanged and verified by Run `36783457030` at 7 of 7 green, as was the documentation-only successor `f66a93d` on Run `36784382005` — the last run this index names. Run #19 `36665007523` covers P2's closure and not this stage · P4 PASS/COMPLETE — Release Bundle, authorized 2026-09-30 by execution prompt v1.0 (delivered as a file, hashed into `10_AUDIT/SOURCE_PROMPTS/`) and closed 2026-10-01, evidence in P4_VALIDATION/; 769 Rust tests in 41 executable suites, 155 UI tests in 6 files, 15/15 gate steps, a 23-ok CLI smoke and all fifty §59 desktop steps on the shipping binary; the final implementation head `e799f2f` is green on Run `36872456446` at 7 of 7, and the closure heads `157f749` (Run `36879477561`, green on a rerun after the runner's rustup install faulted before any repo step ran) and `94f7bd9` (Run `36881101091`) are green at 7 of 7 · G2 PASS (2026-10-01) — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE; evidence G2_VALIDATION/, product tree `e35cfe7` (Run `36906482900`) and evidence head `f75cbc5` (Run `36948719972`) both 7 of 7 on the first attempt · Post-G2 real-desktop MVP acceptance reached `PASS_WITH_FINDINGS` on
2026-10-02 with three findings and no S0/S1, and its narrow remediation closed the same day: E2E-F001
(S3, a surviving analysis that did not say which selection it described), E2E-F002 (S2, a folder the
engine refuses to replace was offered for replacement) and E2E-F003 (S3, a GNU ld MAP refused as another
linker's output) fixed in `e816dcb` and `971015f`; 770 Rust / 159 UI / `check.py` 15/15, the fix head
green on Run #43 `37100371601` at 7 of 7 on the first attempt, evidence in `POST_G2_E2E_REMEDIATION/`
and the 283-case root outside the repository — G2 unchanged, and that round authorized nothing beyond
itself · P5 IN_PROGRESS (opened 2026-10-03) — Productization, authorized by execution prompt v1.0
delivered as a file, SHA-256 `722125f5aa68e324ba1dea4826f66d8392acab9ad9a015c4919ed5ade471e0ae`, archived
in `10_AUDIT/SOURCE_PROMPTS/`; its §4 audit is written at `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` and
the owner's checkpoint settled version identity (artifacts unify on `0.6.0`), migration `0005` after a
written decision doc, the package matrix (Windows with real install evidence here, macOS/Ubuntu
`CI_BUILD_ONLY`) and the store filename (`firmwaresight-p0.sqlite` stays`). Landed since: `4cc8d93`
governance + audit (Run `37125456689` 7 of 7), `812b472` migration `0005` (Run `37127791999` 7 of 7),
`9e3b1de` the `0.6.0` unification — whose Run `37128593254` **failed 6 of 7** on a pre-existing
`compare.test.tsx` race the commit did not cause — repaired test-only by `20b03e3`, green on Run
`37129900728`, and `0c031cd` the packaging commit — `bundle.active`, the three §41 package jobs,
`P5_VALIDATION/P5_PACKAGING_REPORT.md`, `P5_VALIDATION/P5_CI_AUTHORITY.md` and the gate's
`drift/version identity` step — whose first 10-job run `37133706214` **failed 7 of 10**: seven gate jobs
green, three package jobs skipping their own build because `cargo install tauri-cli` leaves `cargo-tauri`
and the group probed `tauri`, then reporting `4/4 steps passed`. Its successor makes a `SKIP` uncountable as
a pass and fails any skip under `CI`; with the pinned CLI installed here the group built this repository's
first real package and found a second defect — the package step must run from `apps/desktop`, because the
CLI resolves its frontend directory from the process cwd — and head `1055242` then went **10 of 10** on Run
`37138881977`, the first run ever to attach a built package for Windows, Ubuntu and macOS. Reading those
artifact sets back found a third defect in the index itself: a directory `.app` given one checksum line
makes `sha256sum -c` answer `FAILED open or read` on a correct build, so a bundle is now indexed file by file
with the aggregate tree digest kept in metadata and each payload's digest recorded beside its container — and
head `53578e9` went **10 of 10** on Run `37143046338`, whose downloaded darwin set verifies on all five lines
with exit 0. The packaged installer has then been run on this host — §38 A–L and §39 measured with the owner's
store parked, hashed and restored byte-identically, in `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md` — and
**Commit C has since landed the guidance and the history that round found missing**: §13 first-run onboarding,
§14 Help/About with the window title fixed in Rust over a closed page enum (closing L21, which that install
round reproduced on the packaged build), and §15–§18 local History over three new bounded storage **read**
APIs with **no new migration** — `P5_VALIDATION/P5_ONBOARDING_HISTORY_REPORT.md` plus its design checklist.
**§32's L22 item then became a document instead of a claim**: `P5_RELEASE_IDENTITY_ADR_DRAFT.md` costs the three
answers to "what do we hash the release notes as", leaves today's bytes-as-evidence semantics standing, and stops
for the Architect, and its premise is a Rust test rather than an argument — same words in LF and CRLF move both
the run id and the release id while the verdict does not move. Verifying that commit is what surfaced L23's
**sixth** UI-test-race instance: the same `compare.test.tsx` read the ranking's second IPC wave synchronously and
the **local** full gate lost it where CI had not, repaired test-only with a contract test for the pending branch.
The tree is now **813 Rust / 201 UI in 8 files / `check.py` 16 steps / 10 authoritative CI jobs**, and that head
went **10 of 10** on Run `37158606478` at its first attempt; what stays
open is the *acceptance* of those screens in an installed binary (§38 C, §64).
**Commit D then landed the recovery half, and §32's L22 question with an answer.** ADR-0028 decides release
identity as *the exact bytes observed on disk*, keeps `require_clean_git = false` valid with its caveat
written down, recommends `.gitattributes` for the user's repository and does not touch this one; the draft
that asked the question is re-statused `EXECUTION_RECORD` rather than rewritten. Storage now owns its own
health (`PRAGMA integrity_check`, read-only, never a repair) and takes a WAL-safe online-backup snapshot of
any older file-backed store **before** migrating it, with a snapshot that cannot be written stopping the
upgrade; the migration matrix re-runs fresh/v1/v2/v3/v4/v5 against that behaviour. Diagnostics is a
closed 41-key allowlist (counted off `ALLOWED_KEYS: [&str; 41]` in `apps/desktop/src-tauri/tests/diagnostics.rs:47`, not off prose) assembled in Rust, shown on
the page that already exists, exported through a native dialog that opens only when a person names a
folder, and proven by positive-control tests rather than by the
absence of a leak — the exported file on the installed build contains no `/` and no `\` character at all.
Commit D left the tree at **854 Rust / 210 UI in 8 files / `check.py` 16 steps / 10 authoritative CI jobs**, with the
package group **4 of 4** beside it, and §27's focused walk ran against the packaged installer with the
owner's store parked, backed up on a second volume and restored byte-identically:
`P5_VALIDATION/P5_DIAGNOSTICS_RECOVERY_REPORT.md` and its design checklist. That walk earned its keep by
finding two defects no unit test could reach — the startup refusal exited **101 through a Tauri panic**
rather than the typed exit 1, because a `setup` error is panicked by the framework inside the event loop
(`tauri-2.12.0/src/app.rs:1443-1445`), and a file that is not a database was being described as a rolled-back
migration to "schema version 0" — both fixed here, the second with the seventh mutation proof. The remote then
found a third defect the local gate could not: the product head came back **8 of 10** because a Commit D test
asserted two snapshot files differ when the only difference its fixtures offered was a second-granular
timestamp, so the claim was a race with a clock. The repair names the difference instead of timing it, and the
failed head keeps its row in `P5_VALIDATION/P5_CI_AUTHORITY.md`.

**Commit E was the present tree when this line was written on 2026-10-04: 868 Rust / 217 UI in 8 files,
`check.py` 16 steps, the same 10 authoritative
CI jobs, package group 4 of 4 with no `SKIP`.** It ran under its own prompt (SHA-256 `030ca282…ba21148`,
53,915 bytes, 2,459 LF lines, §0–§64) and did three things: it widened the committed cohort with
`fixtures/elf/p5-compat/` — 6 new sets, 31 new files, manifest 25 → 56, all built here by real
`arm-none-eabi-gcc 14.3.1` and `clang 22.1.8` over GNU `ld 2.44.0`, every expected total derived from
`readelf` plus the MAP's own region attributes rather than from the model — which found and fixed a real
undercount (clang's allocated `.ARM.exidx.text.main`, 8 bytes, charged nothing because the parser inferred
"not allocated" from "unrecognised section kind"; `SHF_ALLOC` is now read from the section header); it
rewrote `P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md` from the tree instead of from prose; and it dispositioned
§48's fourteen supportability rows — L14/L16/L19/L20/L24 `CLOSED`, L4/L5/L17/L23 `REDUCED`, L25
`NOT_REPRODUCED`, L8/L12 `CARRIED_FORWARD`, L15 stopped for the Architect in
`P5_COMMIT_E_SCHEMA_DECISION.md`. Evidence: `P5_COMMIT_E_DESIGN.md`,
`P5_COMPATIBILITY_FIXTURE_REPORT.md`, `P5_COMPATIBILITY_MATRIX.md`, `P5_SUPPORTABILITY_REPORT.md`. The remote
read-back is Run `37228929762` at head `59d85c3` — attempt 1, **10 of 10**, every job and step read — with the
E1 head `859648e` carrying no run of its own because one push moved both commits; §7a of the fixture report
holds the clean detached worktree that proves the same bytes again without a compiler on `PATH`. Commit E's
§59 read-back successor `6981625` came back **10 of 10** on Run `37230689636`, attempt 1.
**The tree moved three times after that line, and the present numbers are these.** Commit F1 `0bca373` made
the gate **17 steps** (drift gained `baseline integrity`) and the UI suite **219**; Commit F2 was
documentation-only and held both counts; **Commit F2R1 `fb5f628`** fixed F2's three installed UI findings and
took the UI suite to **225 in the same 8 files** while Rust stayed **868**, and **Commit F2R2** is the
evidence head that installed F2R1's own CI artifact and revalidated them on a real window — it holds **868 /
225** unchanged because §43 treats a moved count on a docs-only head as proof the head is not docs-only. The
authoritative CI set is still the same ten jobs, package group 4 of 4 with no `SKIP`, and `P5 = IN_PROGRESS`
with the product at **MVP CANDIDATE** — F2R ended in a STOP, so **F3 remains Architect-controlled** and no P5
closure sentence exists anywhere in this repository.
**Commit E closure normalization (2026-10-04), documentation only.** Two evidence-contract fixes and one answer,
with zero product source: the matrix status column closed to
`SUPPORTED / SUPPORTED_WITH_LIMITS / CI_BUILD_ONLY / NOT_TESTED / UNSUPPORTED` (38 cells checked programmatically,
0 outside; the drifted words moved to the evidence column and release-readiness states to a separate `state`
column), the A–H disposition column closed to
`PROVED_BY_EXISTING_FIXTURE / PROVED_BY_NEW_FIXTURE / SUPPORTED_WITH_LIMITS / NOT_AVAILABLE` with A, F and H
recorded as **existing** — a new assertion on an old fixture is not a new fixture — and the Architect's answer
to L15 recorded as **Option E**: `SourceType::ElfProgramHeader` / `elf.program-header` preserved as a legacy
`analysis:1` identifier, its accurate meaning documented as **ELF address + flags evidence**
(`P5_COMMIT_E_SCHEMA_DECISION.md` §11, `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §7), with no rename, no migration
0006, no `analysis:2` and no golden byte moved, so L15 is `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`, not
`CLOSED`. `P5_COMMIT_E_CLOSURE_NORMALIZATION.md` is the short record; the prompt is archived with both its
hashes because the delivered CRLF bytes and the stored LF bytes differ only in terminator.
**Still no P5 verdict exists** — the product is
MVP CANDIDATE at `0.6.0`, and no tag, Release, installer publication, signing, updater or licence
choice is authorized · open-source
licence PENDING OWNER CONFIRMATION · pricing and commercial research DEFERRED_POST_MVP · active_task:
P5_PRODUCTIZATION`

## Primary reading path

1. `README.md`
2. `PRODUCT_BASELINE.md`
3. `BASELINE.yaml`
4. `.ai/CURRENT_STATE.md`
5. `.ai/ACTIVE_TASK.md` — `P5_PRODUCTIZATION`; G2 closed `PASS` on 2026-10-01 and P5 opened on
   2026-10-03 under an execution prompt of its own, which is the only way a stage opens here. With a live
   task the pointer still names exactly one stage: nothing in it authorizes V1, and no agent lifts a
   later track off the roadmap
5a. `G2_VALIDATION/G2_EXIT_CHECKLIST.md` and `G2_ENGINEERING_CLOSURE_REPORT.md` — the whole-MVP verdict
5b. `POST_G2_E2E_REMEDIATION/` — the narrow remediation of the post-G2 real-desktop findings: what was
    fixed and why (`REMEDIATION_REPORT.md`), the focused real-desktop re-validation on the shipping
    binary (`FOCUSED_REVALIDATION_REPORT.md`), and the round's exit boxes settled one by one
    (`EXIT_CHECKLIST.md`). It changes no stage status: G2 stays PASS and the product stays MVP CANDIDATE
6. `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`
7. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`
8. `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md`
9. `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` — all five runs, the two failures included
10. `V0_VALIDATION/batch_a/BATCH_A_STATUS.md`
11. `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`
12. `P1_A0_VALIDATION/` — the completed intake slice and its correctness closure
13. `P1_VALIDATION/` — the completed P1 Analyze round: execution report, exit checklist (the US-001
    verdict), design checklist, and the shipped-binary desktop smoke
14. `P2_VALIDATION/` — the completed P2 Compare round: execution report (every gate number and the CLI
    smoke), exit checklist (the US-002 verdict), design checklist, the shipped-binary desktop smoke, and
    defect E — the fixture half `.gitignore` hid, which reddened remote Run #17 and is closed by Run #18
    `36648718199` (success, 7 of 7) on the pushed head `4a77ea1`
15. `P3_VALIDATION/` — the completed P3 Release Gate round: execution report (every §66 gate number and the
    18-step CLI smoke), exit checklist (§69's forty boxes and the US-003 verdict), design checklist, and the
    shipped-binary desktop smoke that walked all forty §61 steps and measured the CLI and the desktop
    agreeing on one run id. Three product defects live there — a version pattern quoted into an evidence
    locator that made a run unpersistable, one build carrying two run ids across the two surfaces, and a
    disabled button keeping its accent border — each with the test that now pins it
16. `P4_VALIDATION/` — Release Bundle: the portable bundle contract, its exit checklist and the desktop smoke
    that produced it
17. `P5_VALIDATION/` — **the live round, and the place a newcomer starts.** `P5_PRODUCTIZATION_AUDIT.md`
    (§4 A–H, plus §G's disposition of every inherited limitation and the new L26),
    `P5_CI_AUTHORITY.md` (the ten authoritative jobs and every run measured against them, failures included),
    `P5_MIGRATION_DECISION.md`, `P5_PACKAGING_REPORT.md`, `P5_INSTALL_RECOVERY_REPORT.md`,
    `P5_ONBOARDING_HISTORY_REPORT.md`, `P5_COMMIT_D_DESIGN.md`, `P5_DIAGNOSTICS_RECOVERY_REPORT.md` (its §8
    is the §32 read-back of the whole Commit D chain), `P5_RELEASE_IDENTITY_ADR_DRAFT.md`, and
    `09_ADR/ADR-0028-release-identity-bytes-as-evidence.md` for the decision that replaced it. Commit E's six:
    `P5_COMMIT_E_DESIGN.md` (fixture design, the five mutation proofs, the reliability campaign),
    `P5_COMPATIBILITY_FIXTURE_REPORT.md` (per-fixture provenance, commands, hashes, and what the product
    answered; §4 the normalized A–H table, §7a the clean detached worktree), `P5_COMPATIBILITY_MATRIX.md`
    (status column closed to the five canonical values, §7b the release-readiness states that are not
    compatibility claims), `P5_SUPPORTABILITY_REPORT.md` (§48's fourteen dispositions),
    `P5_COMMIT_E_SCHEMA_DECISION.md` — the question Commit E stopped on **and the answer**, §1–§10 priced as
    written, §11 recording the Architect's Option E — and `P5_COMMIT_E_CLOSURE_NORMALIZATION.md`, the short
    record of the vocabulary cleanup and the L15 decision. Commit F's design record, written before its code:
    `P5_COMMIT_F_DESIGN.md` — L26's measured history, the index-vs-HEAD-vs-worktree comparison, the generation
    sequence the tools now enforce, the seven cross-checkout and mutation proofs, L15's display-only plan, and
    the F1/F2/F3 boundaries with §64 sequenced behind an explicit owner confirmation. **Commit F2's eight,
    written after the installed round they describe** (2026-10-05): `P5_DESKTOP_ACCEPTANCE_REPORT.md` (the §64
    journey on the exact CI-built F1 installer, its findings classified §27, and the coverage boundaries §31
    forbids rounding off), `P5_MIGRATION_RECOVERY_REPORT.md` (the installed synthetic v4 → v5 proof on 34
    measured checks, and the §4 statement that v1/v2/v3 → v5 through an installed binary has been run by
    nobody), `P5_HISTORY_DIAGNOSTICS_REPORT.md` (History read back after its source project moved, and the
    diagnostics export proved path-free), `P5_SECURITY_SUPPORTABILITY_REVIEW.md`, `P5_KNOWN_LIMITATIONS.md`
    (the file `Help.tsx:50` already pointed at and which did not exist until this round),
    `P5_EXECUTION_REPORT.md`, `P5_EXIT_CHECKLIST.md`, `P5_RELEASE_READINESS.md`. **Commit F2R's two, landing
    2026-10-06:** `P5_F2R_UI_CORRECTIVE_DESIGN.md` (written before the code moved — the measured root cause of
    both table defects, the negative control that a wrapper alone does not fix it, the 195 px / 163 px
    occlusion that is why sticky pinning was rejected, and the design-token compliance argument) and
    `P5_F2R_UI_CORRECTIVE_REPORT.md` (§37's twenty parts: the exact F2R1 CI artifact that was installed, the
    three findings revalidated at 1024×720 / 1056×799 / 1440×900, the owner-store park and byte-exact restore,
    and the bounded `PASS_FOR_FROZEN_DESKTOP_SCOPE` verdict that is not a WCAG claim).

`DIRECTORY_TREE.txt` and `SHA256SUMS` began life as the regenerated **v0.6.0** baseline record and are now
regenerated on **every** commit that changes the tracked set — the tree first, `SHA256SUMS` last, then
`python scripts/verify_baseline_artifacts.py` (**813 lines and 693 entries** at Commit F1: 695 tracked paths,
**822 lines and 702 entries** at Commit F2: 704 — nine documents added, no path removed; **825 lines and 705
entries** at Commit F2R1 `fb5f628`: 707 — three paths added, the archived F2R prompt, its design record and
`apps/desktop/ui/src/evidenceBasis.ts`, with no path removed; **826 lines and 706 entries** at Commit F2R2:
708 — exactly one path added, the documentation file `P5_VALIDATION/P5_F2R_UI_CORRECTIVE_REPORT.md`, nothing
removed, so this head adds no product path at all; of the tracked set the
two files named `SHA256SUMS` — the root manifest and
`golden/reports/p4-release/SHA256SUMS` — are both skipped by basename, so the manifest cannot certify itself).
`P0_FINAL_PROMOTION_REPORT.md` records the original commands. The v0.5.1 manifest these replaced is history,
and the nine-entry drift that `0.6.0` closed is described in `.ai/DECISIONS.md`.

**What those digests are of, since ADR-0029 (Commit F1, 2026-10-05):** the canonical Git **stage-0 index blob
bytes**, not the bytes sitting in the working directory. The rule and its alternatives are in
`09_ADR/ADR-0029-repository-baseline-checksums-use-git-index-blobs.md`; the limitation it closes was L26, and
its measured history is the table in `P5_VALIDATION/P5_COMMIT_F_DESIGN.md` §1 — the count of entries that
disagreed with a clean checkout moved **19 → 19 → 14 → 14 → 17 → 13** across seven heads while every document
quietly kept quoting one of them. Two practical consequences. **`sha256sum -c SHA256SUMS` against a working
tree is no longer a valid check**: with `core.autocrlf=true` here, 13 tracked files are LF in the object store
and CRLF on disk, so those 13 lines now report failures that mean "this is a checkout", not "this is corrupt".
The canonical local pass is the verifier above, which `python scripts/check.py --only drift` also runs as
`drift/baseline integrity` on every authoritative CI run; the canonical external pass is
`git archive <commit>` extracted outside the repository, which carries blob content and does validate with
plain `sha256sum -c`. And **`sums` refuses to write while Git reports an unstaged semantic change to a tracked
path**, so the intended edits have to be staged first — a line-ending-only difference is not one, because the
guard asks Git instead of reading bytes. ADR-0028 still governs release evidence: the exact bytes observed on
disk, in a checksum domain that is deliberately not this one.


## All Markdown documents

| Path | Title |
|---|---|
| `.ai/ACTIVE_TASK.md` | ACTIVE TASK |
| `.ai/CURRENT_STATE.md` | Current State |
| `.ai/DECISIONS.md` | Decisions — v0.6.0 |
| `.ai/HANDOFF.md` | Handoff — FirmwareSight v0.6.0 / P0 closed PASS. No active task. Do not invent one. |
| `.ai/README.md` | AI Entry Point |
| `00_GOVERNANCE/00_DOCUMENT_CONTROL.md` | 文档控制规范 |
| `00_GOVERNANCE/01_PROJECT_CHARTER.md` | 项目章程 |
| `00_GOVERNANCE/02_GLOSSARY.md` | 术语表 |
| `00_GOVERNANCE/03_DECISION_POLICY.md` | 决策与 ADR 规范 |
| `01_PRODUCT/00_PRODUCT_VISION.md` | 产品愿景 |
| `01_PRODUCT/01_PRD_MVP.md` | MVP PRD |
| `01_PRODUCT/02_PERSONAS_JTBD.md` | Personas & JTBD |
| `01_PRODUCT/03_SCOPE_NON_GOALS.md` | Scope / Non-goals |
| `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` | User Stories |
| `01_PRODUCT/05_BUSINESS_MODEL.md` | 商业模式假设 |
| `01_PRODUCT/06_COMPETITIVE_POSITIONING.md` | Competitive Positioning |
| `01_PRODUCT/07_MVP_COHORT_AND_CLI_POLICY.md` | MVP Cohort & CLI Policy |
| `01_PRODUCT/08_PRODUCT_MODULE_MAP.md` | Product Module Map |
| `01_PRODUCT/09_CAPABILITY_PORTFOLIO.md` | Capability Portfolio |
| `02_BRAND/00_NAMING_DECISION.md` | 命名决策 |
| `02_BRAND/01_BRAND_FOUNDATION.md` | Brand Foundation |
| `02_BRAND/02_VOICE_MESSAGING.md` | Voice & Messaging |
| `03_DESIGN/00_DESIGN_PHILOSOPHY.md` | Design Philosophy |
| `03_DESIGN/01_INFORMATION_ARCHITECTURE.md` | Information Architecture |
| `03_DESIGN/02_UX_FLOWS.md` | UX Flows |
| `03_DESIGN/03_VISUAL_SYSTEM.md` | Visual System |
| `03_DESIGN/04_COMPONENT_RULES.md` | Component Rules |
| `03_DESIGN/05_ACCESSIBILITY.md` | Accessibility |
| `03_DESIGN/06_UI_REFERENCE_SCREENS.md` | UI Reference Screens |
| `03_DESIGN/07_DESIGN_TOKEN_VALIDATION.md` | Design Token Validation |
| `03_DESIGN/08_POST_MVP_PRESENTATION_RULES.md` | Post-MVP Presentation Rules |
| `04_TECH/00_TECH_STACK.md` | Technology Stack |
| `04_TECH/01_SYSTEM_ARCHITECTURE.md` | System Architecture |
| `04_TECH/02_DOMAIN_MODEL.md` | Domain Model |
| `04_TECH/03_FORMAT_SUPPORT.md` | Format Support Strategy |
| `04_TECH/04_STORAGE_WORKSPACE.md` | Local Storage |
| `04_TECH/05_SECURITY_PRIVACY.md` | Security & Privacy |
| `04_TECH/06_PERFORMANCE_BUDGETS.md` | Performance Budgets |
| `04_TECH/07_CLI_SPEC.md` | CLI Specification |
| `04_TECH/08_CONFIG_SPEC.md` | `firmwaresight.toml` |
| `04_TECH/09_TECH_DECISION_MATRIX.md` | Technical Decision Matrix |
| `04_TECH/10_TOOLCHAIN_BASELINE.md` | Toolchain Baseline |
| `04_TECH/11_DEPENDENCY_BASELINE.md` | Dependency Baseline |
| `04_TECH/12_ASYNC_EXECUTION_MODEL.md` | Async & Execution Model |
| `04_TECH/13_FRONTEND_ARCHITECTURE.md` | Frontend Architecture |
| `04_TECH/14_IPC_DATA_CONTRACTS.md` | IPC / Data Contracts |
| `04_TECH/15_STORAGE_DATABASE_BASELINE.md` | SQLite Storage Baseline |
| `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md` | Artifact Analysis Pipeline |
| `04_TECH/17_RELEASE_PACKAGING_UPDATE.md` | Build / Package / Sign / Update |
| `04_TECH/18_CI_SUPPLY_CHAIN.md` | CI / Supply Chain |
| `04_TECH/19_NATIVE_GPU_ISLAND_POLICY.md` | Native / GPU Island Policy |
| `04_TECH/20_PLATFORM_SUPPORT.md` | Platform Support Matrix |
| `04_TECH/21_OBSERVABILITY_DIAGNOSTICS.md` | Observability / Diagnostics |
| `04_TECH/22_GIT_PROVENANCE_ADAPTER.md` | Git Provenance Adapter |
| `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` | Firmware Memory Accounting Model |
| `04_TECH/24_BUILD_IDENTITY_EVIDENCE.md` | Build Identity Evidence Model |
| `04_TECH/25_TYPED_IPC_BINDINGS.md` | Typed IPC Bindings |
| `04_TECH/26_PORTABLE_SCHEMA_POLICY.md` | Portable Schema Policy |
| `04_TECH/27_GATE_STATE_SEMANTICS.md` | Release Gate State Semantics |
| `05_ENGINEERING/00_REPO_STRUCTURE.md` | Repository Structure |
| `05_ENGINEERING/01_CODING_STANDARDS.md` | Coding Standards |
| `05_ENGINEERING/02_TEST_STRATEGY.md` | Test Strategy |
| `05_ENGINEERING/03_ERROR_MODEL.md` | Error Model |
| `05_ENGINEERING/04_RELEASE_ENGINEERING.md` | FirmwareSight App Release |
| `05_ENGINEERING/05_DEPENDENCY_POLICY.md` | Dependency Policy |
| `05_ENGINEERING/06_CI_CD_BASELINE.md` | CI/CD Baseline |
| `05_ENGINEERING/07_TEST_FIXTURE_STRATEGY.md` | Artifact Fixture Strategy |
| `05_ENGINEERING/08_CODE_AREA_PLAN.md` | FirmwareSight Code Area Plan |
| `06_DELIVERY/00_ROADMAP.md` | Product Roadmap |
| `06_DELIVERY/01_MVP_EXIT_CRITERIA.md` | MVP Exit Criteria |
| `06_DELIVERY/02_BACKLOG_SEED.md` | Seed Backlog |
| `06_DELIVERY/03_QA_CHECKLIST.md` | QA Checklist |
| `06_DELIVERY/04_EXTERNAL_VALIDATION_PLAN.md` | External Validation Plan |
| `06_DELIVERY/05_MVP_TO_PRODUCT_DEVELOPMENT_LIFECYCLE.md` | FirmwareSight MVP → Complete Product Development Lifecycle |
| `06_DELIVERY/06_STAGE_GATES.md` | Stage Gates |
| `06_DELIVERY/07_POST_MVP_CANDIDATE_ROADMAP.md` | Post-MVP Candidate Roadmap |
| `06_DELIVERY/08_MILESTONE_DELIVERABLE_MATRIX.md` | Milestone and Deliverable Matrix |
| `06_DELIVERY/09_USER_INTERVIEW_QUESTION_BANK.md` | Validation Interview Question Bank |
| `07_COMPLIANCE/00_COMPLIANCE_BOUNDARY.md` | Compliance Boundary |
| `07_COMPLIANCE/01_SBOM_STRATEGY.md` | SBOM Strategy |
| `07_COMPLIANCE/02_CRA_CONTEXT.md` | CRA Context |
| `07_COMPLIANCE/03_LICENSE_POLICY.md` | Licensing Policy |
| `08_RESEARCH/00_MARKET_EVIDENCE.md` | Market Evidence Summary |
| `08_RESEARCH/01_NAMING_SCREEN.md` | Naming Collision Screen |
| `08_RESEARCH/02_TECH_REFERENCES.md` | Technical References |
| `08_RESEARCH/03_RISKS_ASSUMPTIONS.md` | Risks & Assumptions |
| `08_RESEARCH/04_RUST_BASELINE_APPLICATION_TO_FIRMWARESIGHT.md` | Applying the Rust Product Census to FirmwareSight |
| `08_RESEARCH/05_EXTERNAL_TECH_VERIFICATION_2026-09-26.md` | External Technology Verification — 2026-09-26 |
| `08_RESEARCH/06_MARKET_SOURCE_REGISTER.md` | Market Source Register |
| `08_RESEARCH/07_AWESOME_DESIGN_MD_REFERENCE.md` | awesome-design-md Reference |
| `08_RESEARCH/08_COMPETITOR_UPDATE_2026-09-27.md` | Competitor Update — AssureLoop |
| `08_RESEARCH/09_EXPERT_ROUND_MARKET_VERIFICATION_2026-09-27.md` | Expert Round Market Verification — 2026-09-27 |
| `08_RESEARCH/10_POST_MVP_SIGNAL_REGISTER.md` | Post-MVP Signal Register |
| `08_RESEARCH/11_RISK_DEPENDENCY_REGISTER.md` | Risk and Dependency Register |
| `08_RESEARCH/SOURCE_REPORTS/README.md` | Source Reports |
| `09_ADR/ADR-0001-product-name.md` | Context |
| `09_ADR/ADR-0002-desktop-stack.md` | Status |
| `09_ADR/ADR-0003-rust-core.md` | Status |
| `09_ADR/ADR-0004-local-first.md` | Context |
| `09_ADR/ADR-0005-no-ai-trusted-core.md` | Context |
| `09_ADR/ADR-0006-mvp-format-scope.md` | Context |
| `09_ADR/ADR-0007-core-first-adapter-driven.md` | Status |
| `09_ADR/ADR-0008-async-boundary.md` | Status |
| `09_ADR/ADR-0009-storage-rusqlite.md` | Status |
| `09_ADR/ADR-0010-artifact-parser.md` | Status |
| `09_ADR/ADR-0011-frontend-baseline.md` | Status |
| `09_ADR/ADR-0012-tracing.md` | Status |
| `09_ADR/ADR-0013-network-deferred.md` | Status |
| `09_ADR/ADR-0014-gpu-deferred.md` | Status |
| `09_ADR/ADR-0015-release-update-lifecycle.md` | Status |
| `09_ADR/ADR-0016-toolchain-pinning.md` | Status |
| `09_ADR/ADR-0017-git-cli-provenance.md` | Status |
| `09_ADR/ADR-0018-ui-baseline-tokens-governance.md` | ADR-0018 — UI 基准与 Design Tokens 治理 |
| `09_ADR/ADR-0019-typed-ipc-bindings.md` | ADR-0019 — Typed IPC Bindings |
| `09_ADR/ADR-0020-validation-sequence.md` | ADR-0020 — V0 Prototype + P0 Vertical Slice Parallel Validation |
| `09_ADR/ADR-0021-memory-accounting.md` | ADR-0021 — Firmware Memory Accounting |
| `09_ADR/ADR-0022-portable-schema-strictness.md` | ADR-0022 — Portable Schema Strictness |
| `09_ADR/ADR-0023-gate-five-state-semantics.md` | ADR-0023 — Gate Five-State Semantics |
| `09_ADR/ADR-0024-post-mvp-candidate-governance.md` | ADR-0024 — Post-MVP Candidate Governance and Namespace |
| `09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md` | ADR-0025 — Conditional Pre-G1 Analyze Implementation (P1-A0; sequencing partly superseded by ADR-0026) |
| `09_ADR/ADR-0026-open-source-mvp-first-delivery.md` | ADR-0026 — Open-Source MVP-First Delivery (G1 basis, V0 non-blocking) |
| `09_ADR/ADR-0027-project-policy-and-provenance-adapter.md` | ADR-0027 — Project Policy and Provenance Adapter Boundary (P3; authorizes `firmwaresight-project`) |
| `09_ADR/ADR-0028-release-identity-bytes-as-evidence.md` | ADR-0028 — Release Identity Uses the Exact Bytes Observed on Disk (P5; decides L22) |
| `09_ADR/ADR-0029-repository-baseline-checksums-use-git-index-blobs.md` | ADR-0029 — Repository Baseline Checksums Are the Canonical Git Stage-0 Index Blob Bytes (P5 Commit F1; decides L26, distinct from ADR-0028 by evidence domain) |
| `10_AUDIT/00_V0.3_AUDIT_RESOLUTION.md` | v0.3.0 Audit Resolution |
| `10_AUDIT/01_UI_BASELINE_REVIEW.md` | UI Baseline Review |
| `10_AUDIT/02_V0.5_EXPERT_REVIEW_RESOLUTION.md` | v0.5.0 Expert Review Resolution |
| `10_AUDIT/03_ADOPTION_DECISION_REGISTER.md` | v0.5.0 Adoption Decision Register |
| `10_AUDIT/SOURCE_PROMPTS/README.md` | Execution Prompt Register |
| `10_AUDIT/SOURCE_REVIEWS/ADR-0018-ui-baseline-tokens-governance.md` | ADR-0018 — UI 基准与 design tokens 治理 |
| `10_AUDIT/SOURCE_REVIEWS/DESIGN_CHECKLIST_TEMPLATE.md` | FirmwareSight 设计评审 Checklist（Do's & Don'ts） |
| `10_AUDIT/SOURCE_REVIEWS/FS-AGENTS-UI-Rules增补节-通用助手.md` | AGENTS.md 增补节：UI Rules（入仓准备件） |
| `10_AUDIT/SOURCE_REVIEWS/FS-UI基调定案提案-设计匠人.md` | FirmwareSight UI 基调定案提案 |
| `10_AUDIT/SOURCE_REVIEWS/FS-v0.3.0-Baseline理解摘要-治理工程生命周期-通用助手.md` | FirmwareSight Baseline v0.3.0 理解摘要（治理 / 工程 / 生命周期门禁） |
| `10_AUDIT/SOURCE_REVIEWS/FS-立项过程与产品演变理解摘要-通用助手.md` | FirmwareSight 立项过程与产品演变理解摘要 |
| `10_AUDIT/SOURCE_REVIEWS/FirmwareSight v0.3.md` | FirmwareSight v0.3.0 基线 · 技术理解摘要 |
| `10_AUDIT/SOURCE_REVIEWS/FirmwareSight-设计风格调研-awesome-design-md.md` | 情报报告：VoltAgent/awesome-design-md 仓库调研 & FirmwareSight 设计风格选型 |
| `10_AUDIT/SOURCE_REVIEWS/README.md` | Audit Source Reviews |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-PostMVP-开放问题裁决记录-通用助手.md` | FirmwareSight Post-MVP 探索开放问题裁决记录 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-可视化与文档导出-设计视角备忘-设计匠人.md` | FirmwareSight 可视化与文档导出 · 设计视角备忘 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-正式产品功能探索-分析结果可视化与文档导出-专业写手.md` | FirmwareSight 正式产品功能探索：分析结果可视化与文档导出（Post-MVP） |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-正式产品功能探索-真需求功能全景扫描-专业写手.md` | FirmwareSight 正式产品功能探索：真需求功能全景扫描（Post-MVP） |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-正式产品功能探索-端口扩展与深度体验-专业写手.md` | FirmwareSight 正式产品功能探索：端口扩展与深度体验（Post-MVP） |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-外部信号报告-信息哨兵.md` | 真需求全景扫描·外部信号报告 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-技术可行性简报-鲁班七号(1).md` | 真需求全景扫描·技术可行性简报 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-技术可行性简报-鲁班七号.md` | 真需求全景扫描·技术可行性简报 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-设计视角备忘-设计匠人(1).md` | FirmwareSight 真需求全景扫描 · 设计视角备忘 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-设计视角备忘-设计匠人.md` | FirmwareSight 真需求全景扫描 · 设计视角备忘 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-端口扩展与深度体验-技术可行性简报-鲁班七号.md` | 端口扩展与深度体验·技术可行性简报 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-端口扩展与深度体验-设计视角备忘-设计匠人.md` | FirmwareSight 端口扩展与深度体验 · 设计视角备忘 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-端口扩展与用户粘性-外部调研报告-信息哨兵.md` | 端口扩展与用户粘性·外部调研报告 |
| `10_AUDIT/SOURCE_REVIEWS_V05/README.md` | v0.5 Expert Source Reviews |
| `10_AUDIT/SOURCE_REVIEWS_V05/可视化与文档导出·外部调研报告.md` | 《可视化与文档导出·外部调研报告》 |
| `10_AUDIT/SOURCE_REVIEWS_V05/可视化与文档导出·技术可行性简报.md` | 《可视化与文档导出 · 技术可行性简报》 |
| `10_AUDIT/SOURCE_REVIEWS_V05/背景回顾-已覆盖面与本轮扫描边界清单-通用助手.md` | 背景回顾：已覆盖面与本轮扫描边界清单 |
| `10_AUDIT/SOURCE_REVIEWS_V05/背景回顾：可视化与文档导出的基线事实清单.md` | 背景回顾：可视化与文档导出的基线事实清单 |
| `10_AUDIT/SOURCE_REVIEWS_V05/背景回顾：端口扩展与深度体验的基线事实清单.md` | 背景回顾：端口扩展与深度体验的基线事实清单 |
| `AGENTS.md` | AGENTS.md |
| `CHANGELOG.md` | Changelog |
| `DESIGN.md` | FirmwareSight DESIGN.md |
| `PRODUCT_BASELINE.md` | FirmwareSight Product Baseline v0.5.0 |
| `README.md` | FirmwareSight v0.6.0 |
| `P0_TECHNICAL_VALIDATION/README.md` | P0 Technical Validation |
| `P0_TECHNICAL_VALIDATION/P0_ARCHITECTURE_CHECK.md` | P0 Architecture Check |
| `P0_TECHNICAL_VALIDATION/P0_CI_REMEDIATION_REPORT.md` | P0 CI Remediation Report |
| `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` | P0 CI Report |
| `P0_TECHNICAL_VALIDATION/P0_CI_RUN_2_CLOSURE_REPORT.md` | P0 CI Run #2 Closure Report |
| `P0_TECHNICAL_VALIDATION/P0_CLI_PARITY_REPORT.md` | P0 CLI Report and Determinism |
| `P0_TECHNICAL_VALIDATION/P0_DEPENDENCY_REPORT.md` | P0 Dependency Report |
| `P0_TECHNICAL_VALIDATION/P0_DESIGN_CHECKLIST.md` | P0 Design Review Checklist |
| `P0_TECHNICAL_VALIDATION/P0_DESKTOP_SMOKE_REPORT.md` | P0 Desktop Real-Window Smoke Report |
| `P0_TECHNICAL_VALIDATION/P0_EXECUTION_PROVENANCE.md` | P0 Execution Provenance |
| `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md` | P0 Exit Checklist |
| `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md` | P0 Final Promotion Report — v0.6.0 Baseline Closure |
| `P0_TECHNICAL_VALIDATION/P0_FIXTURE_REGISTER.md` | P0 Fixture Register |
| `P0_TECHNICAL_VALIDATION/P0_IMPLEMENTATION_LOG.md` | P0 Implementation Log |
| `P0_TECHNICAL_VALIDATION/P0_IPC_PARITY_REPORT.md` | P0 IPC and Core/CLI/Desktop Parity |
| `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md` | P0 Known Limitations |
| `P0_TECHNICAL_VALIDATION/P0_MEMORY_ACCOUNTING_REPORT.md` | P0 Memory Accounting Report |
| `P0_TECHNICAL_VALIDATION/P0_PARSER_RESULTS.md` | P0 Parser Results |
| `P0_TECHNICAL_VALIDATION/P0_PERFORMANCE_REPORT.md` | P0 Performance Report |
| `P0_TECHNICAL_VALIDATION/P0_PLAN.md` | P0 Plan |
| `P0_TECHNICAL_VALIDATION/P0_SECURITY_INPUT_REPORT.md` | P0 Security and Untrusted Input Report |
| `P0_TECHNICAL_VALIDATION/P0_STORAGE_REPORT.md` | P0 Storage Report |
| `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md` | P0 Technical Validation Report |
| `P1_A0_VALIDATION/P1_A0_CORRECTNESS_SMOKE_REPORT.md` | P1-A0 Correctness Closure Desktop Smoke Report |
| `P1_A0_VALIDATION/P1_A0_DESIGN_CHECKLIST.md` | P1-A0 Design Review Checklist |
| `P1_A0_VALIDATION/P1_A0_DESKTOP_SMOKE_REPORT.md` | P1-A0 Desktop Real-Window Smoke Report |
| `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` | P1-A0 Execution Report |
| `P1_A0_VALIDATION/P1_A0_EXIT_CHECKLIST.md` | P1-A0 Exit Checklist |
| `P1_VALIDATION/P1_ANALYZE_DESIGN_CHECKLIST.md` | P1 Analyze Design Review Checklist |
| `P1_VALIDATION/P1_ANALYZE_DETAILS_SMOKE_REPORT.md` | P1 Analyze Details Desktop Smoke |
| `P1_VALIDATION/P1_ANALYZE_EXECUTION_REPORT.md` | P1 Analyze Execution Report |
| `P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md` | P1 Analyze Exit Checklist |
| `P2_VALIDATION/P2_COMPARE_DESIGN_CHECKLIST.md` | P2 Compare Design Review Checklist |
| `P2_VALIDATION/P2_COMPARE_DESKTOP_SMOKE_REPORT.md` | P2 Compare Desktop Real-Window Smoke Report |
| `P2_VALIDATION/P2_COMPARE_EXECUTION_REPORT.md` | P2 Compare Execution Report (incl. the CLI smoke and defect E) |
| `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md` | P2 Compare Exit Checklist |
| `V0_VALIDATION/README.md` | FirmwareSight V0 Validation Workspace |
| `V0_VALIDATION/V0_EXECUTION_PROVENANCE.md` | V0 Execution Provenance |
| `V0_VALIDATION/V0_PLAN.md` | V0 Plan |
| `V0_VALIDATION/V0_TAKEOVER_REPORT.md` | FirmwareSight V0 接手报告 |
| `V0_VALIDATION/analysis/METRICS.md` | Metrics |
| `V0_VALIDATION/analysis/MISUNDERSTANDING_LOG.md` | Misunderstanding Log |
| `V0_VALIDATION/analysis/PAYMENT_SIGNALS.md` | Payment Signals |
| `V0_VALIDATION/analysis/PROTOTYPE_CHANGE_LOG.md` | Prototype Change Log |
| `V0_VALIDATION/analysis/QUOTES.md` | Quote Register |
| `V0_VALIDATION/analysis/STATE_TRANSITION_FINDINGS.md` | State Transition Findings |
| `V0_VALIDATION/analysis/TOOLCHAIN_REQUESTS.md` | Toolchain Requests |
| `V0_VALIDATION/batch_a/BATCH_A_EVIDENCE_INDEX.md` | Batch A Evidence Index |
| `V0_VALIDATION/batch_a/BATCH_A_EXECUTION_PROVENANCE.md` | Batch A Execution Provenance |
| `V0_VALIDATION/batch_a/BATCH_A_STATUS.md` | Batch A Status |
| `V0_VALIDATION/batch_a/BATCH_A_TAKEOVER_REPORT.md` | FirmwareSight V0 Batch A 接手报告 |
| `V0_VALIDATION/batch_a/V0.5.2_RELEASE_BLOCK.md` | v0.5.2 Release Block |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_EVIDENCE_INTEGRITY_CHECKLIST.md` | Batch A Evidence Integrity Checklist |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_MODERATOR_PACK.md` | Batch A Moderator Pack |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_RECRUITMENT_PLAN.md` | Batch A Recruitment Plan |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_RESEARCH_PRICE_ANCHORS.md` | Batch A Research Price Anchors |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_SCHEDULING_TEMPLATE.md` | Batch A Scheduling Template |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_SESSION_SOURCE_INTAKE.md` | Batch A Session Source Intake |
| `V0_VALIDATION/deliverables/V0_FINDINGS.md` | V0 Findings |
| `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md` | V0 Gate Recommendation |
| `V0_VALIDATION/deliverables/V0_PRODUCT_RISK_UPDATE.md` | Product Risk Update |
| `V0_VALIDATION/deliverables/V0_UI_CHANGE_RECOMMENDATIONS.md` | UI Change Recommendations |
| `V0_VALIDATION/deliverables/V0_VALIDATION_REPORT.md` | FirmwareSight V0 Validation Report |
| `V0_VALIDATION/internal/DESIGN_SCOPE_CHECKLIST.md` | Internal Design / Scope Checklist |
| `V0_VALIDATION/internal/DRY_RUN_REPORT.md` | Internal Dry Run Report |
| `V0_VALIDATION/protocol/CONSENT_PRIVACY.md` | Consent & Privacy |
| `V0_VALIDATION/protocol/INTERVIEW_SCRIPT.md` | Interview / Commercial Discovery |
| `V0_VALIDATION/protocol/MODERATOR_GUIDE.md` | Moderator Guide |
| `V0_VALIDATION/protocol/PARTICIPANT_SCREENING.md` | Participant Screening |
| `V0_VALIDATION/protocol/TASK_SCRIPT.md` | Task Script |
| `V0_VALIDATION/prototype/FIXTURE_NARRATIVE.md` | Fixture Narrative |
| `V0_VALIDATION/prototype/PROTOTYPE_CHANGELOG.md` | Prototype Changelog |
| `V0_VALIDATION/prototype/README.md` | V0 Clickable Prototype |
| `V0_VALIDATION/sessions/README.md` | Session Register |
| `V0_VALIDATION/sessions/TEMPLATE.md` | V0 Session — Participant [ID] |
| `assets/ui-mockups/README.md` | UI Mockup Asset Register |
| `fixtures/malformed/README.md` | Malformed Fixtures |
| `golden/reports/README.md` | Report Goldens |
| `templates/ADR_TEMPLATE.md` | ADR-XXXX — Title |
| `templates/BUG_REPORT_TEMPLATE.md` | Bug |
| `templates/DESIGN_CHECKLIST_TEMPLATE.md` | FirmwareSight 设计评审 Checklist（Do's & Don'ts） |
| `templates/FEATURE_SPEC_TEMPLATE.md` | Feature: <name> |
| `templates/RELEASE_CHECKLIST_TEMPLATE.md` | FirmwareSight App Release Checklist |
