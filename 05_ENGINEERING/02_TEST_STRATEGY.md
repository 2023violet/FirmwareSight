---
title: "Test Strategy"
doc_id: "FS-ENG-003"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Test Strategy

## Test pyramid

### Unit
- section classification；
- symbol normalization；
- diff；
- gate rules；
- config validation；
- filename/path normalization。

### Fixture integration
真实编译 artifact 是核心。

每个 supported adapter 至少：
- valid minimal；
- valid realistic；
- stripped；
- malformed；
- edge endian/bitness where applicable。

### Golden tests
对 snapshot/diff/report JSON 做 versioned golden test。

Golden 变化必须人工审查，不能机械 update。

### Fuzz
优先：
- ELF parser boundary；
- MAP line parser；
- HEX parser；
- manifest import。

### UI
- component tests for state rendering；
- keyboard/focus；
- evidence inspector；
- table filtering。

### E2E
最少：
1. import fixture；
2. compare；
3. run gate；
4. export bundle；
5. verify output files。

## Cross-platform

PR:
- Windows required
- Linux required
- macOS 可在 release/nightly（取决于 runner 成本）

正式 release 三平台验证后再宣称支持。


## v0.4 CI event authority

跨平台“在哪个事件必须跑”的最终口径以 `05_ENGINEERING/06_CI_CD_BASELINE.md` 的 Event Matrix 为准：
- PR: Windows/Linux required, macOS optional/cost-controlled
- main: macOS core required
- release: all declared release platforms required
