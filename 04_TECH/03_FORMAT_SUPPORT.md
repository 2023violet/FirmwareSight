---
title: "Artifact and Toolchain Support"
doc_id: "FS-TECH-004"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Format Support Strategy

## Principle

“能打开”不等于“支持”。

一个格式只有满足：
- fixture；
- parser tests；
- known limitations；
- UI capability reporting；
才可列为 Supported。

## MVP Support Matrix

| Input | MVP | Capability |
|---|---|---|
| ELF 32/64 | Yes | sections/symbols/basic metadata |
| GNU ld `.map` | Yes | object/archive contribution where parsable |
| `.bin` | Basic | hash/size/package only |
| Intel HEX | Basic | hash/address span/package only |
| Git repo | Yes | commit/tag/dirty/provenance |
| Keil `.axf` | Experimental until fixtures | often ELF, but do not blanket promise |
| ArmClang map | P1 | adapter |
| IAR `.out/.map` | P1/P2 | adapter |
| COFF/PE | No MVP promise | future |
| SREC | P1 | basic metadata |

## Capability reporting

每次 import 必须给出：

```text
ELF: Supported
Symbols: Available
DWARF: Missing
MAP: Not provided
Object attribution: Partial
Git provenance: Available
Component detection: Not run
```

不能因为缺少某一来源让整个 build “失败”，但必须展示能力缺口。

## MAP adapters

MAP 解析器必须按 toolchain 分离：
- `gnu_ld`
- `armclang`
- `iar`
- future adapters

禁止一个大正则声称“通吃所有 MAP”。

## Corrupt/untrusted input

解析器必须：
- bounds check；
- no panic on user file；
- fuzz target；
- size limits；
- diagnostic error。
