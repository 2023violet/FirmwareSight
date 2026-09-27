---
title: "Core UX Flows"
doc_id: "FS-DESIGN-003"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# UX Flows

## Flow A — First analysis

Launch
→ `Open Project` / `Analyze Artifact`
→ Drop ELF
→ Parse
→ Summary
→ Show capability banner:
`ELF symbols available · MAP not provided · Git not linked`
→ Suggested next action: `Add MAP` or `Compare build`

### 原则
不要首次打开就要求创建账号或填写完整项目配置。

## Flow B — Compare

Analyze current
→ `Compare`
→ choose previous snapshot / import artifact
→ normalize
→ diff summary
→ click top growth contributor
→ symbol/object detail
→ evidence inspector。

## Flow C — Release Gate

Select current build
→ Release
→ Validate project config
→ Run gate
→ grouped results:
Block / Review / Pass
→ user resolves or explicitly accepts Review
→ `Prepare Bundle`
→ destination chooser
→ preview file list
→ export.

## Flow D — Unknown dependency

Component panel
→ `mbedTLS`
→ State: `Detected / version unknown`
→ Evidence:
`Middlewares/Third_Party/mbedTLS/...`
→ action:
`Declare version`
→ mark as Declared, never Observed.

## Flow E — Parse failure

Drop file
→ parser cannot identify
→ show:
- filename；
- detected bytes/magic；
- attempted adapter；
- reason；
- supported formats；
- `Copy diagnostics`。

不允许只显示 “Import failed”.
