---
title: "AI Handoff Entry"
doc_id: "FS-AI-001"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-04"
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
`P5 productization: IN_PROGRESS (opened 2026-10-03; prompt v1.0 delivered as a file, SHA-256
722125f5…71e0ae, archived; §4 audit written at P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md; Commits A–D have
landed — audit + migration 0005 + version unification, packaging on three runners with the job set grown from
seven to ten, §38/§39 real Windows install, §13–§18 onboarding/Help/History, and Commit D's integrity check,
pre-migration backup, 41-key Diagnostics allowlist, typed startup refusal and ADR-0028. Commit D is COMPLETE
as of 2026-10-04. Its §32 read-back successor 956e250, 10 of 10 on Run 37204847474 attempt 1, is
documentation-only, as is the closeout 57a904e after it; the last head that changed a line of Rust was the
test-only repair bccea88. Neither 956e250 nor 57a904e is written down here as "current HEAD" — a commit that
asserted that would be falsified by its own existence, so read git rev-parse HEAD for it. Three P5 heads went
red and keep their rows — 9e3b1de, 0c031cd and 3400981 — with
P5_VALIDATION/P5_CI_AUTHORITY.md as the row-level authority. §34's STOP is in force: no Commit E, no fixture
expansion, no P5 closure, no V1. New limitation recorded, deliberately unfixed: L26, SHA256SUMS verifies only
on the host that wrote it. No P5 verdict exists and the product stays MVP CANDIDATE at 0.6.0)` ·
`active_task: P5_PRODUCTIZATION` · open-source licence `PENDING OWNER CONFIRMATION` ·
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
