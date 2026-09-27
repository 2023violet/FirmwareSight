---
title: "Competitor Update 2026-09-27"
doc_id: "FS-RSCH-009"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-27"
---

# Competitor Update — AssureLoop

## Why it matters

2026-09-27 复核发现 `Zoryvix/assureloop` 已进入相邻“embedded release assurance”类别。

Upstream README pinned:
`987007cbe85d6bde97979bb1179ce649b808d2ea`

它自述为 Zephyr-based embedded firmware 的开源 release assurance tooling，覆盖：
- simulator firmware
- SBOM
- release manifest
- signed/verified evidence
- update package
- local OTA lifecycle simulator

## Current differentiation hypothesis

FirmwareSight 当前假设：
- **existing-artifact first**
- 从 ELF/MAP/Git 进入
- 不要求迁移到特定 RTOS/pipeline
- cross-toolchain direction
- Desktop Analyze/Compare/Gate/Bundle workbench
- local-first evidence inspection

AssureLoop 当前更偏：
- Zephyr-first
- simulator/build/sign/update lifecycle
- MCUboot/OTA workflow

## Important caution

以上只是当前公开资料下的定位差异，不是护城河证明。

Action:
- V0/V1 访谈加入“是否已用/会选 AssureLoop/Zephyr-native pipeline”的问题
- 每季度竞争复核
- 不因竞品出现而扩张到 OTA/HIL
