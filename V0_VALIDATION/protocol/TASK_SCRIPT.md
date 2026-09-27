---
title: "V0 Task Script"
doc_id: "FS-V0-012"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-09-27"
---

# Task Script

## T1 — Readiness / STATE A
“这是准备发布的 relay-controller 固件。请告诉我：它现在是否已经可以进入发布流程？”

## T2 — Obtain symbol evidence
“现在你想进一步查看函数/符号级占用。请尝试获得这些信息。”

After MAP is explicitly added:
“请告诉我这版固件主要 Flash 占用来自哪里。”

## T3 — Compare
“和 v0.2.1 相比，这一版为什么大了？”

## T4 — OTA Review
“有一个新 OTA section。请确认 FirmwareSight 对它是什么态度。”

## T5 — Accept Review
“如果你确认这个 OTA section 是合理变化，请处理这个 Review。”

## T6 — Bundle eligibility
“现在能不能 Build Bundle？”

## T7 — Unknown dependency
“FirmwareSight 发现了 mbedTLS。请告诉我它是什么版本。”

## T8 — Declare
“假设你确认项目使用的是 mbedTLS 3.5.2，请把这个信息记录下来。”

## T9 — Parse failure
“现在你准备替换当前 ELF，但选中的文件是错误的。请处理这个问题。”

## T10 — Historical evidence
“三个月后你要证明某个版本当时为什么被允许发布。你会去哪里找？”

## Never add hints to the task wording.
Any hint is a Moderator Intervention and must be logged.
