---
title: "Personas and Jobs To Be Done"
doc_id: "FS-PRD-003"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Personas & JTBD

## Persona A — Firmware Engineer

**环境**
- STM32 / ESP32 / Nordic / NXP / Renesas 等 MCU；
- GCC/Clang、Keil、IAR 等；
- 发布流程往往是“build + copy files + tag”。

**JTBD**
> 当我要交付一个 firmware build 时，我想快速确认内存、版本、二进制和 Git 状态是一致的，这样我不会因为手工遗漏把错误版本交出去。

## Persona B — Firmware Lead

**JTBD**
> 当团队准备 release 时，我想看到一份可复核的差异与证据，而不是让开发者口头告诉我“应该没问题”。

关注：
- regression；
- build provenance；
- release repeatability；
- team process。

## Persona C — QA / Quality / Compliance Support

**JTBD**
> 当客户或内部流程要求解释一个 firmware release 时，我希望拿到结构化证据，而不是临时向研发收集截图和 Excel。

## Persona D — Small Hardware Founder / CTO

**JTBD**
> 当团队很小、没有专职 DevOps/安全工程师时，我希望用低成本方式把 release 做得专业，而不是购买重型企业平台。

## 不优先用户

- 需要完整 HIL 自动化的大型汽车测试团队；
- 只做 Linux distro/容器的软件供应链团队；
- 需要二进制逆向安全分析的安全实验室；
- 需要设备 fleet OTA 的 IoT 平台团队。

## v0.4.0 Clarification

Persona 描述长期市场，不等于 MVP compatibility promise。早期试用优先招募能提供受支持 GCC/Clang ELF + GNU ld MAP 的工程师；Keil/IAR 用户用于需求发现和 fixture 获取，直到其 adapter 达 Supported。
