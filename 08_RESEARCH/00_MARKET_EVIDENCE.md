---
title: "Market Evidence Summary"
doc_id: "FS-RSCH-001"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-26"
---

# Market Evidence Summary

## Evidence pattern

公开社区中反复出现以下行为：

- firmware 团队用 Git tag + CI + 手工 metadata 管理发布；
- 工程师询问如何保存 release/build manifest 与 device state traceability；
- MCU 老项目做 SBOM 时需要手工追 FreeRTOS、STM32Cube、mbedTLS 等；
- 有人计划根据 object files 自己写 Python 判断真正进入 firmware 的组件；
- ELF/MAP visualizer 持续有人重复开发，说明 build-size 可视化是真需求，但单独不足以形成强商业壁垒。

## Product inference

机会不是“再做一个 ELF viewer”，而是把 viewer 变成进入：

`Analyze → Compare → Gate → Release Evidence`

的免费入口。

## Regulatory tailwind

欧盟 CRA：
- reporting obligations: 2026-09-11
- full application: 2027-12-11

这提升了 traceability/component evidence 的紧迫度，但不应成为唯一定位。

## Source classes

### Primary / authoritative
- European Commission CRA pages
- EUR-Lex
- Tauri official docs
- CycloneDX / SPDX official specs

### User evidence
- Reddit embedded discussions
- GitHub issues/projects
- V2EX/Bilibili community examples

社区证据用于发现痛点，不等同于市场规模统计。

## Selected research URLs

- https://www.reddit.com/r/embedded/
- https://digital-strategy.ec.europa.eu/en/policies/cyber-resilience-act
- https://digital-strategy.ec.europa.eu/en/policies/cra-reporting
- https://cyclonedx.org/specification/overview/
- https://spdx.dev/use/specifications/
