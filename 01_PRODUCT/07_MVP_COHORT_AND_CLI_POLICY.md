---
title: "MVP Cohort and CLI Policy"
doc_id: "FS-PRD-008"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product"
last_updated: "2026-09-27"
---

# MVP Cohort & CLI Policy

## 1. Early-adopter cohort

FirmwareSight 的长期目标用户覆盖 GCC/Clang、Keil/ArmClang、IAR 等生态。

但 **MVP Supported cohort** 明确收窄为：
- ELF 32/64 可由当前 object adapter 可靠解析；
- GNU ld MAP（有 fixture + golden test）；
- BIN/Intel HEX 仅基础 metadata/hash；
- Git provenance optional。

Keil/IAR 用户仍属于 discovery cohort，但在 adapter fixture/回归完成前不得宣传 Supported。

## 2. CLI scope resolution

v0.3.0 中存在真实口径冲突：技术路线要求 `fwsight` 从 P0 起成为一等接口，而商业文档又把 CI/headless CLI 写为 P1/Pro。

v0.4.0 冻结：

### Foundation CLI — MVP included
随着相应能力落地提供：
- `fwsight analyze`
- `fwsight diff`
- `fwsight gate`
- `fwsight release prepare`
- stable JSON/exit codes

它用于：
- Core parity
- local automation
- regression
- 可复现验证

### Commercial CI integration — post validation
以下不属于 MVP 承诺：
- hosted CI integration
- shared team policy
- signed organization policy
- GitHub/GitLab/Jenkins packaged integrations
- remote audit/history

因此：
**CLI ≠ 自动等于 Pro；Team CI 产品化是后续商业能力。**

## 3. Product entitlement rule

在 V0/V1/Beta 验证期，完整核心工作流 Analyze→Compare→Gate→Bundle 对测试用户开放。

不得在核心价值尚未验证前，用付费墙隐藏 Gate/Bundle 并据此判断需求失败。
