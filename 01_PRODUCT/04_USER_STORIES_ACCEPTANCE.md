---
title: "User Stories and Acceptance Criteria"
doc_id: "FS-PRD-005"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# User Stories

## US-001 Analyze ELF
作为 firmware engineer，我可以拖入 ELF 并看到 sections、symbols 和 memory summary。

**验收**
- 不支持的文件显示原因；
- 不因 debug info 缺失而 crash；
- 结果包含 artifact hash；
- symbol list 可排序/筛选；
- 数值单位可在 bytes / KiB 之间切换。

## US-002 Compare Builds
作为 firmware lead，我可以选择两个 snapshots 并看到变化来源。

**验收**
- 明确 old/new；
- 每项 delta 有正负号；
- added/removed/changed 分开；
- 排名不掩盖小但重要变化；
- 可导出 JSON/HTML。

## US-003 Prepare Release
作为 release owner，我可以运行 Gate。

**验收**
- 每个结果显示 rule、state、evidence、remediation；
- Review 与 Block 视觉不同；
- Unknown 不得自动 Pass；
- policy 可保存到 project config。

## US-004 Export Bundle
作为 QA，我可以把证据包发给另一人复核。

**验收**
- bundle 不依赖 FirmwareSight 才能阅读；
- manifest 内含所有文件 hash；
- 报告标注 FirmwareSight version；
- 生成过程不覆盖已有目录，除非用户明确确认。

## US-005 CLI
作为 CI，我可以无 GUI 分析与 gate。

**验收**
- 非交互；
- machine-readable JSON；
- 稳定 exit codes；
- 不依赖桌面进程。

## v0.4.0 Clarification

US-005 的本地 `fwsight` CLI 属于 MVP；云端/团队 CI 集成后置。Git 不可用或证据不足时，不允许 import 失败或默认为 PASS；按 ADR-0023 产生 UNKNOWN，并由 rule policy 映射 effective severity。
