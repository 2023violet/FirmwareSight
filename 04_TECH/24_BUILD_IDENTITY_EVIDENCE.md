---
title: "Build Identity Evidence Model"
doc_id: "FS-TECH-025"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-04"
---

# Build Identity Evidence Model

## Principle

“Git 当前是 commit X” 不等于“这个 ELF 一定由 commit X 构建”。

FirmwareSight 必须把 artifact 身份与 workspace provenance 分开。

## Fields

### Artifact identity — Observed
- SHA-256
- byte size
- parsed architecture
- build-id if present in artifact

### Project/release version
允许来源：
- embedded/version section: Observed
- exact Git tag related to linked workspace: Derived context
- user declaration: Declared
- unavailable: Unknown

不同来源不得互相覆盖；UI 可显示 conflict。

### Git provenance
- repo root
- HEAD commit
- exact tag
- dirty state
是 workspace evidence，而不是任意 binary 的绝对 build proof。

### Time
必须拆分：
- `artifact_mtime`: filesystem observation
- `imported_at`: FirmwareSight observation
- `build_time`: 只有 artifact/toolchain evidence 或用户声明时才存在

禁止把 mtime/import time 叫 build time。

## Gate implications

- missing Git never aborts artifact analysis；
- Git-dependent rule → UNKNOWN if unavailable；
- policy `on_unknown` 决定 effective severity；
- version mismatch BLOCK 只有在被比较的 evidence sources 明确定义后成立。

## Release Notes identity：字节即证据（ADR-0028）

`observe_release_notes` 对文件原始字节做 SHA-256；路径上没有 decode、没有 text mode、也没有 newline 处理。
`ADR-0028` 把这一点定为规则，而不是巧合：

- LF 与 CRLF 是两份不同的 evidence；同一篇 notes 的两种行尾是两个不同的 release。
- Release Notes digest 进入 Gate canonical input，所以 Gate run id 与 release id 可能随 checkout 的行尾移动，
  而 verdict 不变 —— identity 与 correctness 是两个问题，把它们混在一起才会让 normalization 变成一个
  presentation fix 溜进来。
- bundle 发布它 hashing 的那份字节，`verify_bundle` 仍然可以从 bundle 自身字节重算 release id。
- FirmwareSight 不 normalize、不改写、不替换 Release Notes，也不写 `.gitattributes`；被分析的项目目录不属于
  这个产品（`AGENTS.md` §7）。

`release.require_clean_git = false` 仍然是有效的项目显式 policy 选项，ADR-0028 明确保留它，不强制为 `true`。
它的代价必须与开关一起被读到：`false` 时，工作区字节差异（行尾在内）可以改变 Release Notes digest、Gate run id
与 release id，而 `git.clean` 不会 BLOCK，屏幕上也没有一句话说明为什么。推荐的项目做法是在**用户自己的仓库**
用 `.gitattributes` 固定 byte-sensitive 的 release 文件；那是给项目仓库的建议，不是产品行为。Diagnostics 在报告
这个 flag 的同时报告这句话。

契约测试：`crates/firmwaresight-project/tests/bundle_builder.rs` 的
`a_notes_file_that_differs_only_in_line_endings_is_a_different_release`。它是 ADR-0028 的可执行表述，不可删除，
也不可弱化；改它等于改这条 ADR。

## Release attachments: 字节是 Observed，来源永远是 Unknown（C1-U2，2026-10-09）

`ADR-0030` 采纳 Option C1，`04_TECH/28` 冻结其设计，`C1-U1` 存储其事实，`C1-U2` 让它进入 Gate verdict 与 Release
Bundle。按本文档的原则（“artifact 身份与 workspace provenance 分开”），一个被 attach 的 `.bin` / Intel HEX 文件拆成
三个互不冒充的事实：

- **Observed**：文件在选定那一刻的原始字节、`byte_size`、SHA-256。`observe_attachment` 只做 stat、regular-file 检查、
  拒绝零字节、流式 SHA-256、取长度；bundle 写出的就是这批字节，`verify_bundle` 能从 bundle 自身重算 release id。
- **Declared**：它的 kind（`bin` / `hex`）。kind 由 release owner 声明，`kind_basis` 因此永久是 `declared`，
  **绝不**从扩展名、magic bytes 或路径推导；声明成 `hex` 而字节不是 Intel HEX 的文件照原样运送，并照原样披露。
- **Unknown**：它与分析中的 build 的关系。这是本文档 `Gate implications` 的一条具体化 —— `ADR-0030` D-7 规定
  attachment 的 provenance 永久为 `Unknown`，且**不计入** `unknown.count`，所以一个附加文件既不会伪造 PASS，
  也不会把 Unknown 聚合规则从 snapshot 那条路挪过来改变 verdict。

locator 因此是 `attachment:<kind>:<sha256>`，与 `artifact:<kind>:<sha256>` 可区分：读者看到的是“这批字节被绑进来了”，
不是“这个文件是这个 build 产出的”。`evidence_refs` 里两种 scheme 并存时，limitation 句子跟着 count 一起写进 finding，
不藏在别处。

身份影响（`ADR-0028` 的“字节即证据”在这里仍然成立，且没有新增例外）：attached 字节进入 Gate canonical input 的
`attachments[…]` block（仅当集合非空，所以 pre-C1 的 run id 一个都没动），release id 用
`artifact=<kind>:<sha256>:<size>:<file_name>` 的既有文法带上它。两个身份因此各自只回答一件事：run id 回答“这批字节”，
所以 attach 一个文件移动它而 snapshot id 不动（`attaching_a_file_moves_the_run_id_and_leaves_the_snapshot_alone`），
同一个字节改一次也必然移动它（bundle 自己重算时同一个字节改一次会破坏 release id 的复核，
`tampering_with_a_shipped_attachment_breaks_the_bundle_it_sits_in`）；release id 回答“这批字节叫什么名字”，
所以只改文件名会移动 release id 而不移动 run id
（`a_renamed_attachment_keeps_the_run_id_and_moves_the_release_id`）。`SnapshotId::compose`、ELF parser、
`build_snapshot` 与 analysis/diff identity 都不在这条链上，`C1-U2` 也没有触碰它们。

契约测试：`crates/firmwaresight-project/tests/bundle_builder.rs` 的
`attaching_a_file_moves_the_run_id_and_leaves_the_snapshot_alone`、
`a_renamed_attachment_keeps_the_run_id_and_moves_the_release_id`、
`the_same_bytes_declared_as_a_different_kind_move_the_run_id` 与
`a_declared_hex_whose_bytes_are_not_hex_is_shipped_and_disclosed_as_declared`。
