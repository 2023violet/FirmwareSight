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

> **Current evidence pointer — 2026-10-14 / NEXT-46:** [ELF interoperability foundation](39_ELF_TOOLCHAIN_INTEROPERABILITY.md) separates container recognition, parsed metadata, memory accounting, verified compiler cohorts and MAP support. ELF32/64 and both byte orders have Clang metadata fixtures; Keil/IAR cohorts remain UNVERIFIED and ArmLink/IAR MAP remain NOT_SUPPORTED / FIXTURE_BLOCKED. The older roadmap table below is retained as dated intent, not a current support claim.

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
> - Status as of 2026-10-10: **implemented as byte evidence, and still not a support claim.** C1-U1…U4 landed the
>   attachment data and identity, the portable disclosure, and the surfaces a person uses — `--attach <FILE>` on
>   `fwsight gate` and `fwsight release prepare`, and the Release page's **Release attachments** section over four
>   use-case IPC commands. None of it analyzes a BIN or Intel HEX: `ADR-0030` still excludes record walking,
>   checksum validation, address spans, symbols and provenance, and the measured support state remains
>   `P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md`'s row for these formats, which moves only where a runtime run is
>   recorded there. The earlier line in this bullet — "design approved, not implemented" — described the state
>   before those units ran, and is kept here as what changed rather than deleted.

> **Dated 2026-10-15 (M2, `ADR-0033`, PROPOSED) — the GNU ld `.map` row's "object/archive contribution".**
> On branch `feature/m2-object-attribution` until it merges: the GNU ld MAP's input-section rows (after
> `Linker script and memory map`, wrapped rows joined, `COMMON` included, `*fill*` and linker stubs kept as
> unattributed linker bytes, the discarded block skipped) are stored per build with a normalized key such as
> `libdrv.a(uart.o)` — never a host path — and each row is charged like its output section (ADR-0021), so
> Unknown stays Unknown. The desktop Analyze page lists objects and libraries, Compare lists their changes,
> and `fwsight analyze --objects` / `fwsight diff` print them as text. The capability `Object attribution`
> reads `available` only when such rows exist. **Portable JSON is unchanged** (`analysis:1` / `diff:1` carry no
> per-object rows; `ADR-0033` D7), and no other MAP grammar is read.

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
