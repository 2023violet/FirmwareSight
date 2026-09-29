---
title: "AI Handoff Entry"
doc_id: "FS-AI-001"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-29"
---

# AI Entry Point

Baseline `0.6.0` · `P0: PASS` (frozen) · `P1-A0: COMPLETE` with its correctness closure ·
`P1: PASS / COMPLETE` (Analyze, evidence in `P1_VALIDATION/`) ·
`G1: PASS — basis P0 PASS, per ADR-0026 (2026-09-29)` · `active_task: P2_COMPARE` ·
`V0: NON_BLOCKING_USER_FEEDBACK_TRACK, 0 / 8 honest zero` ·
`P2: IN_PROGRESS` (Compare, its own architect prompt arrived 2026-09-29) ·
`P3 / P4: NOT STARTED — each needs its own architect prompt` ·
`Pricing/commercial research: DEFERRED_POST_MVP`.

ADR-0026 changed **sequencing, not standards**: MVP is built first on engineering grounds, V0 no longer
gates it, and every technical boundary in `AGENTS.md` 2 / 7 / 11 still applies. Item 6 still binds
absolutely — **execute only the task `ACTIVE_TASK.md` names, and stop there.** `P2 Compare` is live because
the architect issued a prompt for it, not because the roadmap listed it next; the same rule keeps
`P3 Release Gate` unstarted until its own prompt exists.

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
