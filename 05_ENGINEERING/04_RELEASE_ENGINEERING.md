---
title: "FirmwareSight Application Release Engineering"
doc_id: "FS-ENG-005"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# FirmwareSight App Release

这是 FirmwareSight 自身的软件发布流程，不是客户 firmware release。

## Channels

- dev
- beta
- stable

## Stable requirements

- clean main；
- version tag；
- all tests；
- Windows/macOS/Linux packaging；
- dependency license report；
- changelog；
- installer smoke；
- upgrade smoke；
- checksum；
- signing when infrastructure ready。

## Updates

MVP 初期可不自动更新。
增加 auto-update 前必须 ADR，覆盖：
- signing；
- rollback；
- update server；
- offline environment；
- enterprise disable switch。

## Reproducibility

release metadata 记录：
- Git commit；
- Rust toolchain；
- Node/pnpm；
- OS runner；
- dependency lock hash。
