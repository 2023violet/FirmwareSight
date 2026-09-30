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
`P3: PASS / COMPLETE` (Release Gate, evidence in `P3_VALIDATION/`; the round's gate numbers are LOCAL
PASS — its five commits are not on `origin/main`, which is still `32b23aa` green on Run #19
`36665007523`, so no CI result is claimed for them) ·
`G1: PASS — basis P0 PASS, per ADR-0026 (2026-09-29)` · `active_task: NONE` ·
`V0: NON_BLOCKING_USER_FEEDBACK_TRACK, 0 / 8 honest zero` ·
`P4: NOT STARTED — needs its own architect prompt` ·
`Pricing/commercial research: DEFERRED_POST_MVP`.

ADR-0026 changed **sequencing, not standards**: MVP is built first on engineering grounds, V0 no longer
gates it, and every technical boundary in `AGENTS.md` 2 / 7 / 11 still applies. Item 6 still binds
absolutely — **execute only the task `ACTIVE_TASK.md` names, and stop there.** `P2 Compare` ran because
the architect issued a prompt for it, not because the roadmap listed it next, and its own prompt said
**stop after P2**; `P3 Release Gate` is live for exactly the same reason — prompt v1.1 plus `ADR-0027`,
and nothing more. P3's own §72 says **stop after P3**, which keeps `P4 Release Bundle` unstarted until its
own prompt exists.

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
