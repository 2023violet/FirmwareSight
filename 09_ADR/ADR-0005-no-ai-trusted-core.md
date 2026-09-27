---
title: "ADR-0005 No AI in Trusted Core"
doc_id: "ADR-0005"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Context
Release Gate 和 evidence 必须可复现、可审计。

# Decision
生成式 AI 不参与：
- parsing；
- diff；
- gate；
- hash；
- SBOM事实生成；
- release decision。

未来 AI 只能：
- 解释；
- 搜索；
- 总结；
且输出必须标记 advisory。

# Consequences
产品可信路径更容易测试，也避免网络依赖。
