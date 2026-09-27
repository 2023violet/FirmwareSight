---
title: "UI Reference Screens"
doc_id: "FS-DESIGN-007"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Design"
last_updated: "2026-09-27"
---

# UI Reference Screens

## Accepted set

v0.4.0 接受六张 1440×900 PNG 作为 G1/G2 UI 方向参考。

### 01 Overview
核心：
- capability banner
- “Can we ship now?” readiness summary
- 下一步可见
- 一屏一个主焦点

注意：
“Can we ship now?” 只能表示 **FirmwareSight policy readiness**，不能被解释成法律合规、安全认证或产品适销性的最终结论。

### 02 Analyze
核心：
- Sections + Symbols
- ranked bars
- mono numbers
- Evidence Inspector

Inspector 默认宽度取 token，但必须允许在真实数据下自适应；320–420px 是治理范围，不强锁单一像素。

### 03 Compare
核心：
- old / new / delta
- Added / Removed 不用 0 冒充
- 最大增长贡献者
- 缺 MAP 时明确说明能力降级

### 04 Release Gate
核心：
- BLOCK → REVIEW → UNKNOWN → PASS → N/A
- REVIEW 可显式接受并留下 actor/time
- Bundle preview 与 Gate 状态联动
- BLOCK 存在时不得产出“已通过” Bundle

### 05 Bundle & History
核心：
- 审计密度
- hash / timestamp / ID mono
- bundle 文件可脱离 FirmwareSight 验证
- History 与 Bundle detail 同屏

### 06 Parse Failure
核心：
- What happened
- Why we know
- What to do
- Diagnostics ID
- 不覆盖 last-good artifact
- 明确恢复动作

## Mock data disclaimer

截图中的：
- `relay-controller`
- `v0.3.0-rc2`
- `FirmwareSight 0.1.0-alpha`
- 人名
- 时间戳
- hash
均为视觉示例数据，不是产品协议。

实现时必须清楚区分：
- FirmwareSight App Version
- Project/Artifact Version
- Release Candidate Version

## Missing 07

Unknown Dependency 的图像未随本轮上传，规范见：
`assets/ui-mockups/FS-UI-07-Unknown-Dependency_SPEC.md`


### 07 Unknown Dependency
Core:
- Dependencies is a subview of Analyze, not a new top-level verb;
- Observed / Declared / Unknown are visibly distinct;
- mbedTLS-like detected-but-version-unknown state shows facts, not guesses;
- Declare action records user assertion as Declared;
- Declared never overwrites Observed;
- Unknown has a next step and remains explicit.
