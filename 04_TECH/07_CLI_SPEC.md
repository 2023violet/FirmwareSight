---
title: "CLI Specification"
doc_id: "FS-TECH-008"
product: "FirmwareSight"
version: "0.5.2"
status: "BASELINE"
owner: "Project"
last_updated: "2026-10-12"
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
fwsight gate --project . --artifact firmware.elf --attach image.bin --attach boot.hex
```

- `--attach <FILE>`（C1-U4，`ADR-0030`）可重复，每个文件一次。附件的 kind 由**文件名**声明：`.bin`→`bin`，
  `.hex`→`hex`（大小写不敏感），其余一律 `unknown`；命令行没有、也不新增 `--attach-kind`，因为 §9 冻结的
  载体只有这一个 flag。FirmwareSight 不读文件内容来判定格式，`unknown` 不满足任何 `required` kind。
  读不到的文件按引擎自己的码拒绝（`ERR-BUNDLE-6115`、`6116`、`6117`），退出码 3：这是取输入失败，不是
  判定结果（4/5），也不是写出失败（6）。不带 `--attach` 的调用，stdout 文档与之前逐字节相同。
  `--attach` 的相对路径以**进程当前工作目录**为基准；基准的口径见文末 2026-10-12 澄清。
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
fwsight release prepare --project . --artifact firmware.elf --out dist/release --attach image.bin --attach boot.hex
```

- `--attach <FILE>`（C1-U4）与 `gate` 同一个 flag、同一条声明规则、同一处观测函数
  （`observe_attachment`）：preview 与 publish 各自重新观测一次，所以一个人授权的字节和写出的字节是同一份，
  文件在两次之间变了就拒绝写。这里的拒绝走 §34 既有的 exit 6，并保留引擎自己的码（`6115`…`6118`）；
  两个名字折叠成同一个 bundle 内叶名时是 `ERR-BUNDLE-6109`，说的是选择本身，不是一份没人写过的清单。
  **2026-10-12（`B2-U1R1`）澄清**：上面那句「同一处观测函数」讲的是读数代码只有一份，**不是**说两个动词共用一个
  相对路径基准。`release prepare --attach` 的相对路径以 `--project` 指向的项目根为基准，`gate --attach` 的相对
  路径以进程当前工作目录为基准；同一个相对写法因此可能指向两个不同文件，这是 `N01-F01` 实测到的现状，本轮按
  Owner 的决定保持不变（统一任何一边都是可观察的 CLI 行为变化，需要新的 Owner 决策）。要避开这处歧义就写绝对
  路径——绝对路径两个动词都原样使用，只有基准会不同这一点不适用于它。

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

## 澄清（2026-10-12，`B2-U1R1`）：`--attach` 的相对基准是两个，不是一个

这一节只说明两个动词**现在**各自的基准，以及一处拒绝的措辞换了机制但没换码。它不改变退出码、内层错误码、
Gate identity 或 bundle 字节，也不统一任何一边的解析规则。

| 动词 | 相对路径的基准 | 绝对路径 | 拒绝时的退出码 | 内层码 |
| --- | --- | --- | --- | --- |
| `fwsight gate --attach <FILE>` | 进程当前工作目录 | 原样使用 | 3 | `6115` / `6116` / `6117` |
| `fwsight release prepare --attach <FILE>` | `--project` 指向的项目根 | 原样使用 | 6 | `6115`…`6118` / `6109` |

- 「同一处观测函数」是读数代码只有一份 `observe_attachment`，不是解析入口只有一个：`gate` 直接把调用方给的
  路径交给它，`release prepare` 先经 `resolve_selection` 把相对路径放到项目根上。两条路最终都走同一次
  `stat` + 同一次流式 SHA-256，这就是 §3 里 F01 记录的两个基准并存的原因。
- 两个基准都被测试钉住（`gate_cli.rs` 的 `a_relative_attach_reaches_the_current_directory_and_not_the_project_root`
  与 `release_cli.rs` 的 `a_relative_attach_is_read_against_the_project_directory_and_not_the_current_one`），
  诊断行各自说明自己从哪个目录读起，且都不打印宿主绝对路径。旧行为继续存在；要用一个无歧义的写法就传绝对路径。
- 非法相对分量（`..`、相对路径里的根或盘符前缀）现在在**第一次 `stat` 之前**拒绝，理由是「这个选择放不进项目
  目录」，而不是「碰巧有个文件不存在」。码仍是 `ERR-BUNDLE-6115`，仍走 exit 6，仍不写目的目录，Gate identity
  与 Release ID 不变。旧实现把这类分量映射到项目根下一个「假定不存在」的文件名上，于是当项目里真的存在同名文件时，
  一次 `--attach ../escape.bin` 会以 exit 0 发布那份没人选过的字节（`D-N09-2`，本轮实测复现）。改动之后的输入
  划分是这样的：合法写法一个都没有变得不能用——项目内相对路径、绝对路径，以及一个真叫那个名字、由主人自己选中的
  真实文件，都照旧可用；非法写法照旧被拒，只是多拒掉一种以前**错误放行**的情形，即非法分量恰好撞上同名文件存在。
  没有新增被拒的合法输入，也没有放宽任何拒绝。
- 一处同类问题**没有**在本轮修：配置的 `release.release_notes_path` 里一个「爬出去再回来」的写法
  （如 `docs/../RELEASE_NOTES.md`）会被 `require_project_relative` 放行，却在同一个「假定不存在」的名字上落地，
  于是 Gate 把存在的笔记报成不存在（BLOCK），而那份名字被占用的文件真存在时又把它当笔记读成 PASS。它落在
  `evidence.rs` 的解析函数上，且修它会改动 Gate canonical input 里的笔记事实，属于会移动 run identity 的变更，
  按 `NEXT-10` §40 停止并上报；复现、影响与最小改法记在 `04_TECH/28` §14.4。
