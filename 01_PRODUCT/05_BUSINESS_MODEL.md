---
title: "Business Model Hypothesis"
doc_id: "FS-PRD-006"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# 商业模式假设

> 本文件是验证假设，不是最终定价。

## 1. Product-led Growth 结构

### Free
目的：让工程师主动安装。

建议包含：
- 单 build ELF/MAP Analyzer；
- 基础 memory；
- symbol explorer；
- 单次 build diff；
- 本地使用。

### Pro — Individual
价值来源：
- unlimited project history；
- release gates；
- release bundle；
- CLI；
- policy templates；
- richer diff；
- component evidence / SBOM。

### Team / Business
价值来源：
- shared policy；
- CI；
- signed/controlled policy；
- team export；
- audit history；
- support；
- enterprise packaging。

## 2. 不采用的收费逻辑

不按：
- 文件大小；
- 单次扫描；
- symbol 数量；
- 每个 MCU 型号

收费。这些会惩罚正常使用。

## 3. 付费验证门槛

在写 Team 能力前至少取得：
- 20 个目标用户访谈；
- 8 个实际 artifact 试用；
- 3 个明确付费意愿；
- 1 个真实团队愿意以自己的 release 流程试点。

## 4. 商业判断

真正可收费的不是 treemap，而是：
- release risk reduction；
- repeatability；
- evidence package；
- team policy；
- CI integration；
- compliance preparation。

## 5. 定价暂不冻结

不要在 MVP 前锁死人民币/美元价格。先验证 willingness-to-pay，再根据个人与企业价值分层。

## v0.4.0 Validation Policy

商业分层是待验证假设，不是 MVP 功能墙。V0/V1/Beta 阶段测试用户必须能体验 Analyze→Compare→Gate→Bundle 全闭环。

付费验证分层：
- V0：最少 8，目标 12–15 名 workflow prototype 会话；
- V1：最少 8 名真实用户用自己的 artifact，且出现 ≥3 个明确价格锚定付费意愿或 ≥1 个团队 Pilot；
- 在投入 Team 产品前，累计目标约 ≥20 次目标用户对话、≥8 真实 artifact、≥1 team pilot。
