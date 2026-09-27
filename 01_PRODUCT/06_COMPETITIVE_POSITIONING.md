---
title: "Competitive Positioning"
doc_id: "FS-PRD-007"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Competitive Positioning

## 1. 竞争不是单一产品

FirmwareSight 位于几类工具之间：

| 类别 | 强项 | 缺口 |
|---|---|---|
| ELF/MAP Visualizer | 二进制/内存分析 | 发布流程与 evidence 弱 |
| IDE / Toolchain | 编译调试 | 跨项目 release evidence 弱 |
| SBOM Platform | 供应链与安全 | 对 MCU build artifact 体验偏重 |
| CI/CD | 自动化 | 不理解嵌入式 artifact 语义 |
| Excel/Python scripts | 灵活 | 不一致、不可维护、难交接 |

## 2. Positioning Statement

For small embedded teams that ship firmware but lack a dedicated release engineering stack, FirmwareSight is a local-first release workbench that turns build artifacts into inspectable release evidence. Unlike generic SBOM or CI platforms, it starts from ELF/MAP and works without forcing a new build system or cloud workflow.

## 3. 差异化原则

- Build artifact first；
- local-first；
- deterministic；
- evidence provenance；
- explicit Unknown；
- no workflow migration；
- developer UX；
- compliance-supporting, not compliance theater。

## 4. 竞争边界

不以“支持最多格式”为早期卖点。
第一阶段卖点是：
**少量受支持输入 → 高可信输出。**
