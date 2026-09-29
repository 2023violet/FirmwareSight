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

Baseline `0.6.0` · `P0: PASS` (frozen) · `active_task: P1A0_REAL_ARTIFACT_INTAKE`, **complete** with its
correctness closure · `V0: 0 / 8` · `G1: NOT CLAIMED` · `P1: NOT PASS / NOT CLOSED` ·
`P1-A1 / P2 / P3 / P4: NOT AUTHORIZED`. That task exists because ADR-0025 authorized exactly one bounded
pre-G1 slice, and it is finished; item 6 still governs — there is no active task to execute, so infer
none from the roadmap and wait for a new prompt.

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
