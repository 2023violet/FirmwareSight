---
title: "ADR-0006 MVP Format Scope"
doc_id: "ADR-0006"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-09"
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

# Dated note — 2026-10-09

`ADR-0030` 对本记录作**限定性 supersession**：上面 Decision 列表里的 "BIN/Intel HEX 基础 metadata" 一句被限定并改道，
其余三条（ELF 32/64 基础分析、GNU ld MAP、Git provenance）与 Keil/ArmClang/IAR 的 fixture 前置条件原样有效。

改道的内容：BIN 与 Intel HEX 不再是 Analyze 的输入，也不携带任何 metadata（没有 architecture、entry point、section、
symbol，也没有 Intel HEX 地址跨度）。它们以 **Release 附属字节证据**进入产品——原始字节、byte size、SHA-256 与声明的
kind——并进入 Gate 的 required-artifact 判定与身份绑定。

上面三段文字是本记录当时的决定，未作修改，也不会被覆写。**这条 note 是本文件唯一的变化**（含把
`last_updated` 改为 `2026-10-09`）。截至该日期 C1 只是已批准的设计：功能未实现，能力矩阵仍应读作
`UNSUPPORTED / NOT_IMPLEMENTED`。规格见 `04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md`。
