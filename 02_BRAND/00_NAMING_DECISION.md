---
title: "Naming Decision"
doc_id: "FS-BRAND-001"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Brand"
last_updated: "2026-09-26"
---

# 命名决策

## 1. 正式工作名

# FirmwareSight

Descriptor: **Embedded Firmware Release Workbench**

Tagline: **Know exactly what ships.**

中文主张：**每一次固件发布，都知道自己到底发布了什么。**

CLI：`fwsight`

默认配置文件：`firmwaresight.toml`

工作区目录：`.firmwaresight/`

## 2. 为什么选择 FirmwareSight

### Firmware
直接限定产品领域，不浪费早期市场教育成本。

### Sight
表达“看见、识别、理解”，对应产品核心：
- see the build；
- see the diff；
- see the evidence；
- see what is unknown。

它不会暗示：
- 自动修复；
- 自动安全认证；
- AI 魔法。

## 3. 被淘汰名称

### FirmwareLens
弃用：公开网络已存在同名 firmware static analysis 项目。

### BuildLens
弃用：已有多个活跃 BuildLens 产品/项目。

### FirmScope
弃用：已有产品，且 FIRMSCOPE 也曾用于安全研究系统。

### ReleaseLedger
弃用：已有活跃 PyPI 项目。

### ReleaseProof
弃用：已有 release-readiness 产品。

### FirmwareForge
弃用：已有相关产品/概念使用。

## 4. 命名风险

本次仅做公开网络级别的初步碰撞筛查。

在以下动作前必须重新做专业检索：
- 注册公司/商标；
- 购买品牌域名；
- App Store / Microsoft Store 发布；
- 大规模公开营销。

不得把“搜索没有发现明显同名产品”写成“商标可注册”。

## 5. 命名语法

正确：
- FirmwareSight
- FirmwareSight CLI
- FirmwareSight Release Gate

避免：
- Firmware Sight
- Firmware-Sight
- FWSight（除 CLI / 内部缩写）
