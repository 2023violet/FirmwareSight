---
title: "MVP Cohort and CLI Policy"
doc_id: "FS-PRD-008"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product"
last_updated: "2026-10-09"
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

> **Dated 2026-10-09（`ADR-0030`）。** 上面第三条在本轮的口径由该 ADR 限定：BIN/Intel HEX 不在 Supported cohort 的
> 分析能力里，也不提供 metadata；它们属于 **Release 侧附属字节证据**（字节、byte size、SHA-256、声明 kind），参与
> Gate 的 required-artifact 判定与身份绑定。截至该日期此能力**已批准设计、尚未实现**，所以对用户仍不得宣传 Supported；
> `P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md` 的 `UNSUPPORTED` 行是当前的真实状态。规格见
> `04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md`。

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
