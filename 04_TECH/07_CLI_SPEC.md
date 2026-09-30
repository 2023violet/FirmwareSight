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

## P3 落地（2026-09-30）：`fwsight gate`

命令面（P3 prompt §38）：

```bash
fwsight gate --project . --artifact firmware.elf
fwsight gate --project . --artifact firmware.elf --map firmware.map
fwsight gate --project . --artifact firmware.elf --baseline old.elf --baseline-map old.map
fwsight gate --project . --artifact firmware.elf --json
```

- `--artifact` 必填。没有 artifact 就没有可评的构建，而一张“全部 N/A”的表会被读成放行。
- `--project` 默认当前目录，解析到 `<project>/firmwaresight.toml`。读不到或读不懂一律 exit 2：
  FirmwareSight 不用默认值补全一份它读不全的政策。诊断码 `ERR-CONFIG-7001..7006`。
- artifact 与 baseline 在同一进程内 analyze；baseline 的增量复用 P2 的 Core diff，
  所以 Gate 扣的字节数和 Compare 显示的字节数是同一次计算。
- CLI Gate 不建库、不写记录。可复核的 GateRun 由 Desktop Release 页持久化（prompt §38）。
- exit 4 / 5 自 P3 起可达，`1` 仍不在表内。UNKNOWN 不单独定码：它的有效严重度已经进聚合
  （on_unknown 映射 review → 4，映射 block → 5）。
- `--json` 时 stdout 只有一个 gate-results 文档；map/baseline/git 诊断、config warning 与
  operation id 全部走 stderr。
- 文档里没有宿主机路径，也没有墙上时间：run id 是 `gate-<sha256>`，同一输入两次运行字节相同。
