---
title: "AI Handoff Entry"
doc_id: "FS-AI-001"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-08"
---

# AI Entry Point

Baseline `0.6.0` · `P0: PASS` (frozen) · `P1-A0: COMPLETE` with its correctness closure ·
`P1: PASS / COMPLETE` (Analyze, evidence in `P1_VALIDATION/`) ·
`P2: PASS / COMPLETE` (Compare, evidence in `P2_VALIDATION/`; the round's own gate results are LOCAL PASS,
and the pushed head `4a77ea1` is green on remote Run #18 `36648718199`, 7 of 7 jobs — after Run #17 on the
mid-round commit `c7fc2a3` had gone red and been fixed) ·
`P3: PASS / COMPLETE` and SEALED (Release Gate, evidence in `P3_VALIDATION/`; the round's gate numbers were
measured locally and its closure commits were pushed afterwards — the first of those pushes, Run
`36774472141` on `219178af`, concluded `failure` on 6 of 7 jobs green, the single red being `Dependency
policy` over a crate yanked on crates.io the day of closure and since moved off in `Cargo.lock`; later P3
heads came back 7 of 7, and no further P3 documentation-only successor will be written) ·
`P4: PASS / COMPLETE` (Release Bundle, opened 2026-09-30 and closed 2026-10-01, evidence in
`P4_VALIDATION/`; the final implementation head `e799f2f` green on Run `36872456446` at 7 of 7, and the two
closure heads green at 7 of 7 as read live) ·
`G1: PASS — basis P0 PASS, per ADR-0026 (2026-09-29)` ·
`G2: PASS` — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE (the whole-MVP engineering closure
audit, 2026-10-01, under its own prompt and storage-path addendum; evidence in `G2_VALIDATION/`; product
tree `e35cfe7` and evidence head `f75cbc5` both 7 of 7 on the first attempt) ·
`Post-G2 real-desktop acceptance: PASS_WITH_FINDINGS` and its findings remediation closed the same day
(2026-10-02; three product defects — E2E-F001 S3, E2E-F002 S2, E2E-F003 S3 — fixed narrowly in
`e816dcb` and `971015f`, each with a regression, a mutation proof and a focused real-desktop re-check on
a rebuilt shipping binary; fix head green on Run #43 `37100371601` at 7 of 7; evidence in
`POST_G2_E2E_REMEDIATION/`, the 283-case root outside the repository; **G2 unchanged, still MVP
CANDIDATE**) ·
`P5 productization: PASS_COMPLETE — closed 2026-10-06 (opened 2026-10-03; prompt v1.0 delivered as a file, SHA-256
722125f5…71e0ae, archived; §4 audit written at P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md; Commits A–E have
landed — audit + migration 0005 + version unification, packaging on three runners with the job set grown from
seven to ten, §38/§39 real Windows install, §13–§18 onboarding/Help/History, Commit D's integrity check,
pre-migration backup, 41-key Diagnostics allowlist, typed startup refusal and ADR-0028, and Commit E run under
its own architect prompt (FirmwareSight P5 Commit E v1.0, SHA-256 030ca282…ba21148, 53,915 bytes, 2,459 lines,
archived): the compatibility cohort — 6 new fixture directories and 31 new files under
fixtures/elf/p5-compat/, manifest 25 → 56, built here by arm-none-eabi-gcc 14.3.1 and clang 22.1.8 over GNU
ld 2.44.0 — the undercount those fixtures exposed (clang's allocated .ARM.exidx section charged nothing while
the total printed Exact) and the narrow SHF_ALLOC fix that closed it, P5_COMPATIBILITY_MATRIX.md rewritten from
the tree, and §48's fourteen dispositions: L14/L16/L19/L20/L24 CLOSED, L4/L5/L17/L23 REDUCED, L25
NOT_REPRODUCED, L8/L12 CARRIED_FORWARD, L15 stopped for the Architect — then a documentation-only closure
normalization under its own prompt (FirmwareSight P5 Commit E Closure Normalization v1.0; delivered bytes
SHA-256 9571df2c…075599, 28,738 CRLF; archived LF copy 62040e3d…e6c7af887, 27,366 bytes, 1,373 lines proven
identical one by one): the matrix status column closed to SUPPORTED / SUPPORTED_WITH_LIMITS / CI_BUILD_ONLY /
NOT_TESTED / UNSUPPORTED with release readiness moved to a state column, the A–H dispositions closed to
PROVED_BY_EXISTING_FIXTURE / PROVED_BY_NEW_FIXTURE / SUPPORTED_WITH_LIMITS / NOT_AVAILABLE (A, F and H recorded
as existing, because a new assertion on an old fixture is not a new fixture), and **L15 answered as Option E**
— `SourceType::ElfProgramHeader` and the `analysis:1` token `elf.program-header` preserved as a legacy wire
identifier, its accurate meaning documented as "ELF address + flags evidence" in
P5_COMMIT_E_SCHEMA_DECISION.md §11 and 04_TECH/23 §7, with no enum rename, no wire rename, no migration 0006,
no analysis:2 and no golden byte moved, so L15 is CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER and not CLOSED. Commit D is COMPLETE
as of 2026-10-04. Its §32 read-back successor 956e250, 10 of 10 on Run 37204847474 attempt 1, is
documentation-only, as is the closeout 57a904e after it; the last head that changed a line of Rust before
Commit E was the test-only repair bccea88. Neither 956e250 nor 57a904e is written down here as "current HEAD" — a commit that
asserted that would be falsified by its own existence, so read git rev-parse HEAD for it. Three P5 heads went
red and keep their rows — 9e3b1de, 0c031cd and 3400981 — with
P5_VALIDATION/P5_CI_AUTHORITY.md as the row-level authority. P5 Commit E's engineering candidate 59d85c3 is 10 of
10 on Run 37228929762 attempt 1, its §59 read-back successor 6981625 is 10 of 10 on Run 37230689636 attempt 1,
and the normalization head after them, 80b47c4, is recorded above by its own run: **Run 37262147348 (#67) went
9 of 10 on attempt 1** — the single red being `Generated output drift`, where rustup on the Ubuntu runner
failed to install `clippy-preview-x86_64-unknown-linux-gnu` over a `bin/cargo-clippy` conflict and rolled back
before any crate compiled — **and 10 of 10 on attempt 2**, with only that job re-executed and the other nine
attempt-1 executions carried forward. Two attempts, stated as two attempts; the same runner-provisioning class
P4's 157f749 already records at `BASELINE.yaml:514`. Commit E's §42 STOP has been ANSWERED — Option E.
**Commit F1 landed 2026-10-05** as `0bca373`, **10 of 10 on Run 37293381181 attempt 1**, and it is the product
and tooling layer: ADR-0029, the two baseline scripts, the new drift step, and L15's display caption.
**Commit F2 lands after it** as the installed-evidence layer and nothing else — **no path under `crates/`,
`apps/`, `scripts/`, `fixtures/`, `schemas/`, `migrations/` or `.github/`, and no `Cargo.toml`, `Cargo.lock`,
`package.json`, `pnpm-lock.yaml`, `tauri.conf.json` or `deny.toml`**, because any one of those invalidates the
F1 installer as the candidate under test. F2 ran the whole §64 journey once on the exact CI-built F1 Windows
artifact (artifact id `11337963032` of run `37293381181`, 3,885,631 bytes,
`a1152ef3…3c076b`, installed as downloaded), proved the installed synthetic **v4 → v5** migration on 34
measured checks, and parked and restored the owner's store **twice** with §26's gates satisfied before any
commit. Its verdict is `F2 = COMPLETE`, `P5 = IN_PROGRESS`, `F3 = READY_FOR_ARCHITECT_REVIEW` — and it does
**not** close P5. The stops that remain are F3's to lift: no `P5 PASS_COMPLETE`, no `active_task NONE`, no tag,
no GitHub Release, no signing, notarization, updater or licence choice. **L26 is decided and closed by
ADR-0029** — root `SHA256SUMS` holds canonical Git stage-0 index blob digests, the verifier runs in the
authoritative drift gate, and `sha256sum -c SHA256SUMS` against a working tree is no longer a valid check (read
`09_ADR/ADR-0029-repository-baseline-checksums-use-git-index-blobs.md` before touching either tool). **L15
stays `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`**, with its Evidence Inspector presentation residue CLOSED by
F1's caption and **re-proved in the installed binary by F2**, by analysing the target ELF deliberately without
its MAP. The counts **as Commit F2 left them on 2026-10-05** were 868 Rust / **219 UI** in 8 files with the local gate
at **17 of 17** (drift **8 of 8**, deny 1/1, core-smoke 3/3) and the package group at 4 of 4 with no SKIP —
and F2 was the first head that had to hold those two numbers still, since a docs-only round that moves a count
is a product change wearing a documentation diff.
**Commit F2R landed on 2026-10-06 as the narrow corrective the Architect inserted between F2 and F3**, in two
heads: **F2R1** `fb5f628` fixed F2's three installed findings inside `apps/desktop/ui/src/**` and nowhere else
— the Analyze Sections prose column (S2), History's clipped `Details` (S3), Release printing the raw Core enum
`MapRegionAndElfLoad` (S3) — with **no design token changed and no contract, schema, migration, Rust or
dependency movement**; and **F2R2** is the evidence head that downloaded and installed **F2R1's own** CI
artifact (run `37431977428`, **10 of 10 on attempt 1**; artifact id `11397938806`, installer
`efbc45a3…d5fd4`), revalidated all three on a real window at 1024×720 / 1056×799 / 1440×900, then uninstalled
and restored the owner's store byte-exact. **The present counts are 868 Rust / 225 UI in 8 files** on the same
17-step gate, and F2R2 holds both numbers identical to F2R1 because that equality is its own docs-only proof.
Its verdict is `F2R = FINAL PASS / COMPLETE`, `F2 = PASS`, `P5 = IN_PROGRESS`, product **MVP CANDIDATE**,
`F3 = READY_FOR_ARCHITECT_REVIEW` — **which is not the same as F3 being authorized**: F2R ends with a STOP and
returns to the Architect, L20 is `CLOSED ACROSS VERIFIED HUMAN-FACING MEMORY-BASIS SURFACES`, and the §34
verdict is `PASS_FOR_FROZEN_DESKTOP_SCOPE` on one host at 100 % scale with `WCAG_CERTIFICATION =
NOT_PERFORMED`, `MULTI_DPI_125_150 = NOT_TESTED` and `SECOND_WINDOWS_HOST = NOT_TESTED` all unchanged. The
twenty-part record is `P5_VALIDATION/P5_F2R_UI_CORRECTIVE_REPORT.md`; what was measured before any CSS moved
is `P5_F2R_UI_CORRECTIVE_DESIGN.md`. **Commit F3 then closed the stage on 2026-10-06**, under *FirmwareSight —
P5 Commit F3 Final Governance Closure, Execution Prompt v1.0 — Architect Authorized* (delivered SHA-256
`860e0097…64514c`, 36,458 CRLF bytes; stored blob `6cacd10e…d749c66`, 34,637 LF bytes; every line proven
identical, `.gitattributes` untouched). F3 §6 re-audited all nineteen exit areas, found **no required
engineering item `BLOCKED`**, corrected the one thing the record had got wrong (`P5_KNOWN_LIMITATIONS.md` was
missing its L12 row while the exit checklist asserted the list was complete), and wrote the canonical state:
**`P5 = PASS_COMPLETE`, Productization = `ENGINEERING_COMPLETE`, product `MVP_CANDIDATE`, narrative
"FirmwareSight Productized MVP Candidate", baseline `0.6.0`, `active_task = NONE`** — holding 868 Rust / 225 UI
still, changing no product byte, installing nothing, touching no owner data, and carrying every limitation
forward: L3, L4, L5, L8, L11, L12, L13, L15, L17, L23 and L25 all keep their dispositions, because **no
limitation became CLOSED merely because P5 closed**. `F3 remote CI = PENDING_EXTERNAL_EVIDENCE` inside the
commit: **final external acceptance requires F3 remote CI**, and §34 forbids a further commit written only to
record it. What P5's closure authorizes: **nothing next.** V1, B1, private beta, RC, GA, signing, notarization,
an updater and any licence decision each need a new architect prompt, and `P5_VALIDATION/
P5_FINAL_CLOSURE_REPORT.md` §14 states what `PASS_COMPLETE` does not mean. *(Dated: of that list the architect
has since issued exactly one thing — **V1**, on 2026-10-06, under its own prompt archived with its measured
digest. B1, private beta, RC, GA, signing, notarization, an updater and a licence still need prompts of their
own, and V1's §40 says even a clean V1 pass does not authorize B1.)*
`V1 own-artifact external validation: IN_PROGRESS (opened 2026-10-06 under *FirmwareSight — V1 Own-artifact
External Validation, Execution Prompt v1.0 — Architect Authorized*, delivered file SHA-256 `48768ff0…ec10a0f`,
46,207 CRLF bytes, archived as Git blob `d5e8d457…51774` / SHA-256 `c62314fe…90c34e` / 44,510 LF bytes, every
line proven identical and `.gitattributes` untouched). It is a **research** stage, not a product stage: no
feature, no schema, no migration, no dependency, no cloud, no account, no telemetry, no updater, no signing, no
notarization, no licence, no pricing, no B1, no RC, no GA — and the product code is frozen for the length of a
formal cohort. §1 froze the cohort build to the exact F3 Windows artifact (run `37475580080`, head `08fdfcb`,
artifact id `11419727517`, NSIS installer 3,888,432 bytes, SHA-256 `182506f2…63d12`, **unsigned**, internal
`SHA256SUMS.txt` verified `OK`, preserved outside Git; the 5,536,303-byte figure is the artifact container, not
the installer, and the activation commit's own package is explicitly **not** the cohort build). **Dated 2026-10-08,
and this is the live identity: the owner's inline *V1 Cohort Re-freeze / Execution Authorization v1.0* re-froze the
cohort to the U1-accepted build — run `37828673549`, head `41bb6a36`, artifact id `11573661113`, NSIS 3,896,257 bytes,
SHA-256 `9a51e86a…c87d93`, still **unsigned**, still preserved outside Git, GitHub expiry `2026-10-22T19:18:25Z` — with
F3's freeze preserved and dated superseded rather than rewritten, and with sessions, metrics and thresholds untouched
(`v1_execution.cohort_build_refreeze` / `cohort_build_state: REFROZEN_TO_U1_CANDIDATE`, record
`V1_VALIDATION/V1_COHORT_REFREEZE_RECORD.md`).** **§5 is the
branch this round took: with no real eligible participant there is nothing to measure, so an agent writes the
Recruitment Ready pack and stops rather than inventing a user.** `V1_VALIDATION/` now holds §13's sixteen files —
plan, metric contract with M1–M6 thresholds fixed **before** any data, participant register, five protocol
documents continued from V0's discipline, sessions register and template, three analysis registers, two
deliverables — with every register at **zero**, the report a **skeleton**, and no fabricated participant,
session, quote or timing. `research_state = RECRUITMENT_READY`, **eligible external sessions = 0**,
recommendation `V1_INCOMPLETE_INSUFFICIENT_SAMPLE`, and the next move is **human**: recruit, consent, moderate
(§47/§48). `P5` stays `PASS_COMPLETE`, `G2` stays `PASS`, the product stays `MVP_CANDIDATE` at `0.6.0`, L11 stays
`CARRIED_FORWARD` (the row V1 exists to answer), and the counts held at **868 Rust / 225 UI in 8 files** on the
same 17-step gate because §45 requires them not to move — the figure at the re-frozen cohort build is
**868 Rust / 291 UI in 9 files**, which is U1's own test contract rather than a V1 metric, and
`v1_execution.product_counts_required_unchanged.refrozen_guard` is the live guard)` ·
`active_task: NONE` — **stage U1 is CLOSED as `PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS` on the
Architect's own independent visual acceptance record of 2026-10-08** (`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R3_Architect_Final_Verdict_2026-10-08.txt`,
5,159 bytes / 52 lines / 0 CR / SHA-256 `8634321c…36c619` / blob `f16f0e7e…`, archived and registered as an authority
record rather than an execution prompt; verdict record `U1_VALIDATION/U1P_R3_ARCHITECT_FINAL_VERDICT_RECORD.md`;
machine state `BASELINE.yaml` `u1_execution.architect_final_verdict`). The scope is part of the word: visual readiness
for a controlled V1 study, **not** GA quality, not feature completeness, not distribution approval. The tally is
guarded verbatim — 25 items, 23 PASS / 1 FAIL / 1 NOT_VERIFIED / 0 NOT_CAPTURED, 2 `MISMATCH_PROVED` flags — and
**no later document may restate it as 25/25 or 100%**. Deviations A–D stay open and enumerated; E, a count defect in
these entry documents, was corrected by this update exactly as the record routed it. **V1 is not resumed by this**:
`paused_for` still names U1, 0 eligible external sessions, and the
record's closing section is `NEXT ACTION RECOMMENDATION, NOT EXECUTED` — this round only re-downloaded and re-hashed
the newer artifact's bytes (NSIS 3,896,257 bytes `9a51e86a…`) because the record conditions document changes on that
read. **Dated later on 2026-10-08:** the separate authority that recommendation asked for then arrived, and the cohort
build did move to that re-verified artifact — the sentence in this entry that said "the F3 artifact is still the frozen
cohort build" was true of the recording round and is superseded by the re-freeze entry above it, not by this one.
Counts unmoved at **868 Rust across 47 result lines / 291 UI in 9 files** · *(dated record of where the track
stood two rounds earlier: the UI productization track that ran on 2026-10-07 — the first round closed at first round,
and its continuation round installed the CI-built product and inspected it, so **U1 was
`READY_FOR_ARCHITECT_VISUAL_REVIEW`** with one U1-V2 finding open and unwaived; V1 paused with every state field
intentionally intact)* · open-source licence `PENDING OWNER CONFIRMATION` ·
`V0: NON_BLOCKING_USER_FEEDBACK_TRACK, 0 / 8 honest zero` ·
`Pricing/commercial research: DEFERRED_POST_MVP`.

ADR-0026 changed **sequencing, not standards**: MVP is built first on engineering grounds, V0 no longer
gates it, and every technical boundary in `AGENTS.md` 2 / 7 / 11 still applies. Item 6 still binds
absolutely — **execute only the task `ACTIVE_TASK.md` names, and stop there.** `P2 Compare` ran because
the architect issued a prompt for it, not because the roadmap listed it next, and its own prompt said
**stop after P2**; `P3 Release Gate` ran for exactly the same reason — prompt v1.1 plus `ADR-0027`, and
nothing more — and its prompt said **stop after P3**. `P4 Release Bundle` ran on the same rule and no further: prompt v1.0, delivered as a file and hashed into
`10_AUDIT/SOURCE_PROMPTS/`, with no new ADR because no technology baseline moves — and it closed
`PASS / COMPLETE` on 2026-10-01, returning the pointer to `NONE`. P4's own §78 said **stop after P4**.
The G2 engineering closure audit then ran under its own prompt — audit-first, narrow test-only repair —
and closed `PASS` the same day, returning the pointer to `NONE` again. Its prompt says **stop after G2**:
V1, P5, every History / installer / signing / updater / SBOM / cloud / account / telemetry / AI / pricing
idea and any `v0.7.0` stay unstarted until the architect issues something for them.

The two rounds that ran after G2 obeyed the same rule and did not widen it. The real-desktop acceptance
round was **observe and record only**: no product change, no commit, no fix of what it found. The
remediation round that followed was **fix the three proven defects and stop**: no P5 surface, no parser
performance work, no schema or migration, no new dependency, no licence decision, and the carried-forward
observations still carry forward. Both prompts ended in the same direction this one does — return to the
architect.

V1 arrived the same way and obeys the same rule, with one difference worth naming: it is the first track whose
blocking dependency is a **person** rather than a decision. Its prompt can authorize the protocol, freeze the
build and open the register; it cannot produce a firmware engineer holding their own artifact. §5 anticipates
exactly that and gives the only acceptable answer — build the pack, set `RECRUITMENT_READY`, report **0 eligible
external sessions**, and stop. Later V1 rounds will be judged on whether a real transcript existed before its
session file did.

任何 AI 接手本项目时：

1. 读取根 `README.md`
2. 读取 `.ai/CURRENT_STATE.md`
3. 读取 `.ai/DECISIONS.md`
4. 读取 `.ai/ACTIVE_TASK.md`
5. 读取任务关联 ADR
6. 只执行 Active Task

## 不允许
- 自行从 roadmap 挑任务；
- 因为“更现代”替换技术栈；
- 加云、AI、账号系统；
- 扩大格式支持；
- 修改 baseline decision 而不写 ADR。
