---
title: "Decision and ADR Policy"
doc_id: "FS-GOV-004"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# 决策与 ADR 规范

## 必须写 ADR 的变化

- 产品名称或一级定位；
- 桌面框架、前端框架、核心语言；
- 存储格式或数据迁移策略；
- Artifact / Evidence / Gate 核心数据模型；
- 默认联网或遥测；
- SBOM 标准主版本；
- 支持工具链的承诺；
- 商业许可证；
- AI 进入可信路径；
- 插件系统；
- 自动更新与代码签名策略。

## 不需要 ADR

- 纯文案优化；
- 不改变语义的 UI 微调；
- Bug 修复；
- 已授权范围内的内部重构；
- 测试补充。

## ADR 结构

Context → Decision → Alternatives → Consequences → Revisit Trigger。

ADR 一旦 Accepted 不修改结论；若变更，新增 ADR supersede 旧 ADR。
