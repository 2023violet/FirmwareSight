---
title: "ADR-0006 MVP Format Scope"
doc_id: "ADR-0006"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Context
嵌入式 toolchain 格式碎片化，过早宣称 Keil/IAR 全支持会产生大量兼容债务。

# Decision
MVP 正式支持：
- ELF 32/64 基础分析；
- GNU ld MAP；
- BIN/Intel HEX 基础 metadata；
- Git provenance。

Keil/ArmClang/IAR 只有在 fixture + adapter tests 完成后升级为 Supported。

# Consequences
范围小，但可信。
