---
title: "External Validation Plan"
doc_id: "FS-DEL-005"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Product / Research"
last_updated: "2026-09-27"
---

# External Validation Plan

## Why v0.4 changes the sequence

立项记录曾强调“先做 5–7 个核心界面原型，再写正式代码”；v0.3.0 又把 P0 Technical Vertical Slice 放在真人工作流验证之前。

两者各自有合理性：
- 纯原型可快速验证信息架构/价值；
- 技术 slice 可防止验证一个实际无法可靠解析的产品承诺。

v0.4.0 统一为 **V0 + P0 并行**，两者都通过后才进入 Product MVP implementation。

## Track V0 — Workflow Prototype Validation

输入：
- 已接受 6 张高保真 reference
- Unknown dependency 规范
- 可点击 prototype（实现工具不限）

目标：
验证用户是否理解：
`Analyze → Compare → Gate → Bundle`

不验证：
- parser 性能
-真实 artifact 兼容
- SQLite
- Tauri implementation

招募：
- minimum 8
- target 12–15
- 优先受支持 GCC/Clang/GNU ld cohort
- 可包含 Keil/IAR discovery participants，但不得误称当前 Supported

记录：
- task success
- hesitation/confusion
- Gate state comprehension
- “Can we ship now?” 是否被误解成安全/法律认证
- payment question / price anchor
- requested toolchain

## Track P0 — Technical Vertical Slice

真实 ELF fixture：
`parse → normalize → memory → BuildSnapshot → fwsight JSON → minimal Tauri summary`

Exit:
- deterministic facts
- Core/CLI/Desktop parity
- no parser panic
- memory accounting model proven on fixtures
- typed IPC path proven
- large-file guard benchmark

## Gate into P1

只有：
- V0 PASS
- P0 PASS
两者同时满足，才进入 P1 Analyzer / Product MVP implementation。

## V1 — Own-artifact MVP Validation

P1–P4 MVP Candidate 后：
- minimum 8 real target users with own artifacts
- report numerator/denominator for all ratios
- return-intent threshold: >=62.5% AND minimum 5 users
- ≥3 price-anchored willingness signals OR ≥1 team pilot

## Commercial validation before Team investment

累计目标：
- ~20 target-user conversations
- ≥8 own artifacts
- ≥1 team pilot

这些是 go/no-go evidence，不是市场规模统计。
