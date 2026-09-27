---
title: "ADR-0004 Local-first"
doc_id: "ADR-0004"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Security"
last_updated: "2026-09-26"
---

# Context
Firmware artifacts、symbol names、项目路径可能敏感，目标用户也常在离线研发环境。

# Decision
MVP local-first/offline-first。
核心能力不依赖登录或服务器。

# Consequences
- 更容易取得工程团队信任；
- 云协作延后；
- licensing/update 需考虑离线环境。
