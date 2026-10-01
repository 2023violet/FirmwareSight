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

## P4 落地（2026-10-01）：`fwsight release prepare`

命令面（P4 prompt §33）：

```bash
fwsight release prepare --project . --artifact firmware.elf --out dist/release
fwsight release prepare --project . --artifact firmware.elf --map firmware.map --out dist/release
fwsight release prepare --project . --artifact firmware.elf --baseline old.elf --baseline-map old.map --out dist/release
fwsight release prepare --project . --artifact firmware.elf --out dist/release --force --json
```

- 原 spec 示例 `release prepare --project . --out dist/release` 不指定 artifact：`firmwaresight.toml`
  描述的是政策，不是"哪一次构建在发布"。P4 用显式 `--artifact` 补全这处欠定，属于实现澄清，
  不是新增产品动词（prompt §33）。`release` 命令族仍只有 `prepare` 一个动词。
- `--artifact`、`--out` 必填；`--project` 默认当前目录。缺任一项都是 exit 2，不写任何目录。
- Gate 由 bundle 引擎用 P3 的同一套逻辑重算（`build_context` + `evaluate`），CLI 不实现第二套判定；
  §8 的"只有 PASS 才能打包"在 Core 里只有一处表述，引擎与 release model 调用同一函数。
- CLI 没有 Review 历史，也不交互式接受 Review：聚合为 REVIEW → exit 4，BLOCK → exit 5，
  两者都不写 bundle（prompt §33）。接受 Review 仍是 Desktop Release 页的审计动作。
- exit 码（§34）：`0` bundle 写成；`2` usage/config/版本解析失败；`3` artifact/MAP 解析导入失败；
  `4` Gate REVIEW；`5` Gate BLOCK；`6` bundle/source/destination/自校验失败。仍不定义 `1`。
- `--json` 成功时 stdout 只有一个文档，且与 bundle 内 `release-manifest.json` 字节相同；
  失败时 stdout 只有一个 error envelope（`ERR-BUNDLE-61xx` 码 + remediation），诊断全部走 stderr。
  机读输出里没有宿主机路径，`SHA256SUMS`/manifest 的排除规则由 bundle 自身表达。
- `--force` 只是替换授权，且只替换可识别的 FirmwareSight bundle（§30/§31/§32）；
  陌生目录即使带 `--force` 也拒绝（`ERR-BUNDLE-6107`），不删除任何用户数据。
- 发布走 sibling 暂存目录：写 payload → 逐项校验 → `SHA256SUMS` → 最后写 manifest → rename 到位，
  替换成功才丢弃旧 bundle；失败不留半份 bundle，也不覆盖别人的目录。
- CLI release 不建库、不写 release record（没有可绑定的 stored run）；可复核的发布记录由 Desktop 持久化。
