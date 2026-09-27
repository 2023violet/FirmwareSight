---
title: "Naming Collision Screen"
doc_id: "FS-RSCH-002"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-26"
---

# Naming Collision Screen

检索日期：2026-09-26

## Screened out

| Name | Reason |
|---|---|
| FirmwareLens | existing firmware/static-analysis project |
| BuildLens | multiple active software/products |
| FirmScope | existing active products + research name |
| ReleaseLedger | active PyPI release tool |
| ReleaseProof | existing Jira release-readiness software |
| FirmwareForge | existing firmware-generation concept/product |
| BuildProof | heavily used by multiple products |
| BinScope | Microsoft binary verification tool |
| FWTrace | NVIDIA firmware trace utility |

## Selected working name

**FirmwareSight**

公开搜索未在本轮结果中发现一个明显的、与本产品同定位的活跃软件品牌，但这不是：
- 商标检索；
- 公司注册检索；
- 域名可用性保证；
- 全球冲突保证。

## Pre-launch gate

公开发布前必须：
1. USPTO / EUIPO / CNIPA 等目标市场检索；
2. GitHub/npm/crates/PyPI/App Stores 搜索；
3. domain/social handle check；
4. 法律顾问或专业商标检索（若进入正式商业化）。


## v0.4 Search Audit Note

v0.3.0 记录了排除结果但没有完整保存当时全部查询字符串，因此不能事后伪造“完整搜索日志”。

v0.4.0 明确把这项标记为 **historical audit limitation**。

GA 前新一轮正式 clearance 必须保存：
- search date
- platform/database
- exact query
- result URL/reference
- reviewer
- legal/trademark conclusion scope

当前 `FirmwareSight` 仍是工作品牌，不等于商标可注册结论。
