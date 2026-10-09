---
title: "MVP Product Requirements"
doc_id: "FS-PRD-002"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product"
last_updated: "2026-10-09"
---

# MVP PRD

## 1. MVP 目标

证明 FirmwareSight 能以低迁移成本提升嵌入式固件发布的确定性。

## 2. MVP P0 能力

### P0-1 Artifact Import
输入：
- ELF；
- GNU ld MAP；
- BIN / HEX（仅基础 metadata/hash）；
- 可选 Git repository path。

必须：
- 文件不可识别时明确失败；
- 原文件默认只读；
- 记录 SHA-256；
- 不静默修改输入。

> **Dated 2026-10-09（`ADR-0030`，限定 P0-1 的第三条输入）。** BIN / HEX 不是 Analyze 的输入，也不产出 metadata：
> 架构、entry point、section、symbol 与 Intel HEX 地址跨度一律不声称。它们以 **Release 附属字节证据**进入产品——
> 原始字节、byte size、SHA-256、声明的 kind——并且可以满足 `[artifacts] required` 里的 `bin` / `hex`。上面的
> "记录 SHA-256" 与"原文件默认只读"两条继续适用于附属文件，且实现方式就是流式哈希。
> 这一天 C1 只是**已批准的设计**：功能未实现（`04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md`），本条不是能力声明。

### P0-2 Build Analysis
ELF 最少展示：
- target architecture；
- endian / bitness；
- entry point；
- sections；
- FLASH/RAM 分类后的占用；
- symbols；
- top contributors；
- debug info presence；
- build-id（如存在）。

### P0-3 Build Diff
支持两个 Build Snapshot：
- total FLASH delta；
- total RAM delta；
- section delta；
- symbol delta；
- object/module delta（证据足够时）；
- added / removed / changed symbols。

不得只显示“+8 KB”；必须能继续 drill down。

### P0-4 Build Identity
最少：
- Artifact hash；
- artifact filename；
- Git commit；
- Git tag（如存在）；
- dirty/clean；
- compiler/toolchain（能观测时）；
- project version（声明或解析）；
- timestamp 的来源与可信类型。

### P0-5 Release Gate
MVP 内建检查：
1. Git tree 是否 clean；
2. 当前 commit 是否与 release metadata 匹配；
3. tag/version 是否一致；
4. required artifacts 是否存在；
5. hashes 是否生成；
6. FLASH budget；
7. RAM budget；
8. 与 baseline 相比是否出现超阈值增长；
9. 必填 Release Notes 是否存在；
10. Unknown evidence 是否达到需要人工 Review 的条件。

Gate 结果只有：
- PASS
- REVIEW
- BLOCK
- NOT_APPLICABLE

### P0-6 Release Bundle
生成目录：
- selected firmware artifacts；
- `SHA256SUMS`；
- `release-manifest.json`；
- `release-report.html`；
- `release-notes.md`（如有）；
- `analysis.json`；
- `diff.json`（如有 baseline）。

MVP 不自动签名，不上传服务器。

## 3. P1 能力

- Component Evidence；
- CycloneDX SBOM；
- 多 build 历史；
- CI/headless CLI；
- policy presets；
- Keil/ArmClang MAP adapter；
- richer treemap；
- HTML share report。

## 4. 非功能需求

- 离线可用；
- 输入不默认上传；
- 所有输出可复现；
- 同一输入 + 同一版本程序应产生语义等价结果；
- 对未知格式 fail closed；
- UI 不掩盖 Unknown；
- 支持 Windows 作为首要平台；
- Linux/macOS 保持同代码基线。

## 5. MVP 成功指标

- First useful analysis < 60s；
- 500 MB 以下 artifact 工作集不导致 UI 卡死；
- 支持 fixture 的解析成功率 100%；
- 受支持格式不允许 crash；
- Diff 结果可由测试 fixture 精确验证；
- Release Bundle 可在另一台机器上独立阅读。

## v0.4.0 Clarification

- Gate finding state 统一为 5 态：PASS / REVIEW / BLOCK / UNKNOWN / N/A；详见 ADR-0023。
- Foundation CLI 属于 MVP 技术/产品表面；“CI integration”指后续团队化产品，不再把 CLI 整体推迟。
- MVP Supported cohort 先以 GCC/Clang ELF + GNU ld MAP 证据链为主；Keil/IAR 仅在 fixture+adapter 通过后升 Supported。
- Memory budget 必须使用 `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` 的双口径模型，不得把 `.data` 简化成只占 RAM 或只占 FLASH。
