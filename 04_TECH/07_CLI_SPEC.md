---
title: "CLI Specification"
doc_id: "FS-TECH-008"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# CLI Specification

Binary: `fwsight`

## Commands

```bash
fwsight analyze firmware.elf
fwsight analyze firmware.elf --map firmware.map --json
fwsight diff old.elf new.elf
fwsight gate --project .
fwsight release prepare --project . --out dist/release
fwsight project doctor .
```

## Output rules

默认 human-readable。
`--json` 输出稳定 schema。

stdout：
- 正常结果。

stderr：
- diagnostics/progress/errors。

## Exit codes

- `0` success / gate pass
- `2` usage/config error
- `3` parse/import error
- `4` gate review
- `5` gate block
- `6` export error
- `10+` reserved

CI 不允许通过解析自然语言判断结果。

## Determinism

`--json` 不包含无必要的 wall-clock fields。
路径尽量 project-relative。

## Destructive behavior

CLI 默认不覆盖 existing bundle。
需要 `--force`，并在文档中明确。
