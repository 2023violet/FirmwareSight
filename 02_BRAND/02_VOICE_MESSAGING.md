---
title: "Voice and Messaging"
doc_id: "FS-BRAND-003"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Brand"
last_updated: "2026-09-26"
---

# Voice & Messaging

## 1. 文案原则

### 说事实，不做判决
好：
> Git tree has 2 modified files.

差：
> Your release is unsafe.

### 说未知，不填空
好：
> FreeRTOS detected; version could not be verified.

差：
> FreeRTOS v10.x

### 给下一步
> Version tag does not match manifest. Update the manifest or select the intended tag.

## 2. 状态文案

PASS：
> Version and Git tag match.

REVIEW：
> Toolchain version could not be confirmed from the artifact.

BLOCK：
> Required `firmware.bin` is missing.

UNKNOWN：
> No evidence available.

## 3. 禁用词

避免：
- guaranteed
- fully compliant
- secure by default（除非严格限定）
- perfect
- intelligent detection（若实际是规则）
- zero risk

## 4. 首页首屏建议

Headline:
**Know exactly what ships.**

Subhead:
Analyze firmware builds, compare what changed, verify release rules, and export evidence — locally.

Primary CTA:
**Analyze a build**

Secondary:
**Compare releases**


## v0.4.0 Canonical Gate Vocabulary

ADR-0023 supersedes any earlier four-state table.

Canonical finding states:
- PASS
- REVIEW
- BLOCK
- UNKNOWN
- N/A

Voice examples:

PASS:
> Version and Git tag match.

REVIEW:
> Flash usage is within the hard budget but exceeds the project review threshold.

BLOCK:
> Required firmware artifact is missing.

UNKNOWN:
> Git provenance is unavailable, so this rule cannot be evaluated.

N/A:
> SBOM input is not configured for this project and the rule is not applicable.

Unknown 与 N/A 不得互换；Unknown 表示证据不足，N/A 表示规则本来不适用。
