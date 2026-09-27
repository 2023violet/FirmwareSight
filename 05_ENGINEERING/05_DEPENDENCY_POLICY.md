---
title: "Dependency Policy"
doc_id: "FS-ENG-006"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Dependency Policy

## Rule

新增依赖不是“顺手操作”。

新增前回答：
1. 标准库/现有依赖是否够用？
2. 依赖是否维护？
3. license 是否允许商业分发？
4. 是否引入 native runtime？
5. 是否显著增加 bundle？
6. 是否能被更小依赖替代？
7. 是否进入安全敏感路径？

## Disallowed by default

- GPL dependency 进入 proprietary distribution（除非明确法律评估）；
- unknown/custom license；
- abandoned parser in trusted core；
- runtime download code；
- telemetry SDK；
- AI SDK for core。

## Frontend

避免因为一个 Button/Tooltip 引入完整 UI kit。
设计系统由项目掌控。

## Lockfiles

- Cargo.lock committed
- pnpm lock committed
