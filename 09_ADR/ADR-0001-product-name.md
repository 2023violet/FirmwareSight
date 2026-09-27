---
title: "ADR-0001 Product Name"
doc_id: "ADR-0001"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Context
需要冻结项目工作名，并避免上一轮 `FirmwareLens` 的同名冲突。

# Decision
采用 **FirmwareSight**。

CLI：`fwsight`。

# Consequences
- 文档、package、配置使用 FirmwareSight；
- 正式商业发布前做商标/domain clearance；
- 若 clearance 失败，brand 可替换，但内部 domain model 不绑定品牌名。

# Status
Accepted.
