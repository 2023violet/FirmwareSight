---
title: "Product Vision"
doc_id: "FS-PRD-001"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Product"
last_updated: "2026-09-26"
---

# 产品愿景

## Vision

**FirmwareSight 成为小型嵌入式团队发布固件时默认打开的最后一个工程工具。**

用户不应该在 release 前手工拼接：
Git 状态、ELF 信息、MAP、hash、版本号、memory 报告和第三方组件清单。

## 核心承诺

> Know exactly what ships.

FirmwareSight 不尝试“理解一切”，而是让已知事实更清楚、差异更明显、未知项更诚实。

## 产品心智模型

用户面对的不是“扫描器”，而是一个四段式工作台：

1. **Analyze** — 看清 build；
2. **Compare** — 看清变化；
3. **Gate** — 看清风险；
4. **Release** — 固化证据。

## 北极星体验

第一次用户：
1. 打开 FirmwareSight；
2. 拖入 `.elf`；
3. 立即看到 memory 与 identity；
4. 再拖入上一版本；
5. 看到 Diff；
6. 关联 Git 项目；
7. 点击 Prepare Release；
8. 得到可解释的 Gate；
9. 导出 Release Bundle。

不要求用户修改 firmware，不要求迁移构建系统，不要求云账号。
