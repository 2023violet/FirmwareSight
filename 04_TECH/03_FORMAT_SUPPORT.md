---
title: "Artifact and Toolchain Support"
doc_id: "FS-TECH-004"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-10-09"
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

> **Dated 2026-10-09 (`ADR-0030`) — reads this table against the product.** The two `Basic` rows above are this
> document's 2026-09-26 intent, kept as written. What is true and what is decided now:
>
> - `.bin` and Intel HEX are **not** analysis inputs. ELF remains the only analyzed artifact and GNU ld MAP the only
>   auxiliary one, so no row of `analysis.json` or Compare carries a BIN/HEX section or symbol table.
> - The Intel HEX capability above, **"address span", is withdrawn rather than deferred.** No record walker, per-record
>   checksum validation or addressing handling exists anywhere in the workspace, and `ADR-0030` excludes it. An
>   attached `.hex` file reports bytes, size, SHA-256 and a declared kind only.
> - The `package` half of both rows is the surviving promise: BIN and Intel HEX become **release attachments** — raw
>   bytes, `byte_size`, SHA-256, declared kind — bound into the Gate required-artifact verdict and into the release
>   identity, and verified into the bundle.
> - Status as of this date: **design approved, not implemented.** `ADR-0030` authorizes no code. The specification is
>   `04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md`, and the measured state stays
>   `P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md`'s `UNSUPPORTED`.

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
