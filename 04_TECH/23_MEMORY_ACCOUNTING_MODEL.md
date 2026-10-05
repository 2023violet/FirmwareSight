---
title: "Firmware Memory Accounting Model"
doc_id: "FS-TECH-024"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-04"
---

# Firmware Memory Accounting Model

## 1. Why this is needed

“FLASH used / RAM used” 在嵌入式产品中不是简单的 section name 求和。

典型 `.data`：
- 初值存于非易失 load image；
- 运行时复制到 RAM；
因此可能同时贡献 **nonvolatile image footprint** 与 **runtime RAM footprint**。

v0.4.0 禁止用“`.data` 只算 RAM”之类简化规则支撑 BLOCK。

## 2. Two primary budgets

### Nonvolatile image footprint
回答：
> 需要写入/存放到目标非易失存储中的 load image 有多大？

### Runtime RAM footprint
回答：
> 程序运行时静态/已分配内存布局占用多少 RAM？

两者分别建模、分别预算。

## 3. Typical semantics

- executable/code/rodata: 通常贡献 nonvolatile footprint
- `.data`: 通常同时贡献 nonvolatile load bytes + runtime RAM
- `.bss`: runtime RAM；通常不贡献 file payload
- debug sections: host/debug metadata，默认排除 device budget
- custom sections: 先看 segment/region evidence；不能仅靠名称猜

这些是规则框架，不是对所有 linker script 的硬编码结论。

## 4. Evidence precedence

从强到弱：

1. explicit project memory-region configuration + ELF program/load evidence
2. supported MAP memory-region evidence
3. ELF segment/section address + flags 的 deterministic mapping
4. section-name heuristic — 只能标记 Derived/low-confidence，不得静默升级成 Observed

## 5. Gate safety

Hard memory BLOCK 需要同时满足：
- 有显式 budget/region 定义；
- 当前 build 的归属证据足够；
- accounting rule 可复现。

否则 memory result 为 UNKNOWN / REVIEW according to policy，不能伪造 PASS/BLOCK。

## 6. UI

Analyze/Compare 必须区分：
- image / nonvolatile footprint
- runtime RAM footprint

若产品简写为 FLASH/RAM，UI 必须可打开 accounting explanation/evidence。

## 7. L15 legacy source identifier (2026-10-04)

This section documents a name. It changes no earlier clause of this document, no evidence class, no
precedence rule, no serialized value and no stored row.

§4 item 3 — "ELF segment/section address + flags 的 deterministic mapping" — is the rule that produces
`MemoryEvidenceBasis::ElfAddressAndFlags`. The evidence label that rule writes down is **not** named after it:
`SourceType::ElfProgramHeader`, whose `analysis:1` wire token is `"elf.program-header"`
(`crates/firmwaresight-report/src/dto.rs:139`, `schemas/analysis.schema.json:616`). The name suggests only an
ELF program header, while §4 item 3's charge reads section **and** segment attributes; a reader is therefore
entitled to find the label loose. It is, and it stays.

The Architect's decision, recorded in `P5_VALIDATION/P5_COMMIT_E_SCHEMA_DECISION.md` §11 (Option E,
`RESOLVED_BY_ARCHITECT`, 2026-10-04), is to **preserve the identifier and document the meaning** rather than
rename it:

- `elf.program-header` is a **legacy `analysis:1` compatibility identifier**. It carries no guarantee that the
  evidence came literally from a `PT_*` program header, and must not be read as one.
- The accurate human-facing description of that basis is **ELF address + flags evidence**. Documentation aimed
  at people should use that phrase; the Compare page's basis caption already does
  (`apps/desktop/ui/src/Compare.tsx:690`).
- Explicitly refused, because each one is a public-contract change: a new enum member, a renamed member, a
  renamed wire value, rewritten `evidence.source_type` history, a `migration 0006`, an `analysis:2`, a moved
  golden byte, or a changed Bundle `analysis.json`. The charge has been arithmetically correct since Commit E
  read `SHF_ALLOC` from the section header; only its label is imprecise, and L15 therefore remains
  `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`, **not** `CLOSED`.
- Open for a later round: the Evidence Inspector renders the stored token verbatim
  (`Details.tsx:667` ← `details.rs:223` ← `query.rs:503` ← `db.rs:507`), so a user sees `ElfProgramHeader`.
  A display-only caption there is a presentation decision; it is named as a Commit F item in
  `P5_VALIDATION/P5_SUPPORTABILITY_REPORT.md` §2's L15 row. §1 and §10 of the closure normalization round that
  wrote this section forbade touching product source, so nothing was changed to make the screen agree.
