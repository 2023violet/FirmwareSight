---
title: "AI Handoff Entry"
doc_id: "FS-AI-001"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-30"
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
`G1: PASS — basis P0 PASS, per ADR-0026 (2026-09-29)` · `G2: READY_FOR_ENGINEERING_GATE_REVIEW` ·
`active_task: NONE` ·
`V0: NON_BLOCKING_USER_FEEDBACK_TRACK, 0 / 8 honest zero` ·
`Pricing/commercial research: DEFERRED_POST_MVP`.

ADR-0026 changed **sequencing, not standards**: MVP is built first on engineering grounds, V0 no longer
gates it, and every technical boundary in `AGENTS.md` 2 / 7 / 11 still applies. Item 6 still binds
absolutely — **execute only the task `ACTIVE_TASK.md` names, and stop there.** `P2 Compare` ran because
the architect issued a prompt for it, not because the roadmap listed it next, and its own prompt said
**stop after P2**; `P3 Release Gate` ran for exactly the same reason — prompt v1.1 plus `ADR-0027`, and
nothing more — and its prompt said **stop after P3**. `P4 Release Bundle` ran on the same rule and no further: prompt v1.0, delivered as a file and hashed into
`10_AUDIT/SOURCE_PROMPTS/`, with no new ADR because no technology baseline moves — and it closed
`PASS / COMPLETE` on 2026-10-01, returning the pointer to `NONE`. P4's own §78 said **stop after P4**,
which keeps the G2 closure audit, every History / installer / signing / updater / SBOM / cloud / account /
telemetry / AI / pricing idea and any `v0.7.0` unstarted until the architect issues something for them.

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
