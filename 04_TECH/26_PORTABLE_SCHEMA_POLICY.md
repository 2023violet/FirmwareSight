---
title: "Portable Schema Policy"
doc_id: "FS-TECH-027"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-10"
---

# Portable Schema Policy

## Scope

以下是对外可持久/可移交 contract：
- release manifest
- gate results
- accepted reviews
- CLI machine-readable export（对应版本化 schema）

它们比同车发布的 Desktop IPC 更严格。

## Rules

- stable object 默认 `additionalProperties: false`
- 显式 extension point 使用 `extensions`
- `$id` 使用稳定 URN，避免伪域名
- schema_version required
- semantics 改变需要 schema major
- additive optional field 仍需 contract tests
- unknown input schema major → hard error，不静默重释

## v0.4 schemas

- `urn:firmwaresight:schema:release-manifest:1`
- `urn:firmwaresight:schema:gate-results:1`
- `urn:firmwaresight:schema:accepted-reviews:1`

## Hosting

真正公开 schema URL/域名在品牌/domain clearance 后确定；URN 在此之前保持稳定。

## P3 落地（2026-09-30）：Gate 文档与 project 配置契约

- `gate-results:1` 保持 v1，顶层属性表一个不动：`policy_sha256`、
  `project_config_schema_version`、`baseline_snapshot_id`、`overall_effective_severity` 一律走
  `extensions`。五态词表未变，effective_severity 仍是独立字段（ADR-0023）。
- `accepted-reviews:1` 只加一件事：**可选** 属性 `original_state`（`enum ["REVIEW"]`），required
  列表不变。契约测试两头都跑 —— P3 之前写下的最小 v1 记录仍然通过，P3 自己产出的每条记录都带
  `original_state = "REVIEW"`。
- 新增 `urn:firmwaresight:schema:project-config:1`（`schemas/project-config.schema.json`）：
  `firmwaresight.toml` 自此是公开契约，语义见 `04_TECH/08`。
- 三份契约共用 `firmwaresight_report::schema_check`，draft/2020-12 的无依赖子集；未实现的关键词
  返回失败而不是放行。`format: date-time` 按存储实际写入的 UTC 形状断言，而不是当注解忽略。

## P4 落地（2026-09-30）：Analysis v1 与 bundle 契约集合

P4 之后的公开契约集合是六个，全部 major = 1：

| contract | 状态 | 命名风格 |
| --- | --- | --- |
| `urn:firmwaresight:schema:analysis:1` | **新增** | camelCase |
| `urn:firmwaresight:schema:diff:1` | 既有（P2） | camelCase |
| `urn:firmwaresight:schema:gate-results:1` | 既有（P3） | snake_case |
| `urn:firmwaresight:schema:accepted-reviews:1` | 既有（P3） | snake_case |
| `urn:firmwaresight:schema:release-manifest:1` | 既有，**未改** | snake_case |
| `urn:firmwaresight:schema:project-config:1` | 既有（P3） | snake_case |

- 新增 `analysis:1`（`schemas/analysis.schema.json`）是 Release Bundle 里 `analysis.json` 的形状。
  它**不是** CLI/Desktop 用的 `firmwaresight.analyze/p0-internal-1` 换个名字：那个 envelope 与桌面
  IPC 同车发布，按 `04_TECH/14` §4 不需要公开语义版本；bundle 会被没有这个仓库的人读到，所以内部
  形状不得冒充持久契约。被复用的是 **projection**（memory block 与 evidence 行与 CLI 同一函数），
  新增的是完整 section/symbol 表。
- `analysis:1` 的三条硬规则：只写 bundle 内文件名（不写 host path）、不写生成时刻、absence 保持
  explicit（每个 `Fact` 是 value 加 `*UnknownReason`，未知不写成 0 或空串）。地址是 hex 文本且永不
  做单位换算。
- `release-manifest:1` **一个字段都没动**，也没升 major。v1 的 `release/build/generated_by/files[]`
  属性表是 closed 的，但顶层保留了 `extensions: {additionalProperties: true}`，P4 需要的
  `gate_run_id`、`policy_sha256`、`integrity_model`、schema majors 一律走 `extensions`——与 P3 对
  `gate-results:1` 的处理同一手法。自行造 v2 会让已经按 v1 校验的读者失效，而保留位已经够用。
- 自引用规则（bundle 的 `SHA256SUMS` 覆盖除自身与 manifest 外的每个文件；manifest 覆盖除自身外的
  每个文件、包含 `SHA256SUMS`）写在 `extensions.integrity_model` 里，也写在
  `release-report.html` 里。没有任何文件携带自身摘要，也不伪造空摘要。注意 repository 根目录的
  `SHA256SUMS` 与 bundle 的 `SHA256SUMS` 是两个不同物件。
- `schema_check` 的覆盖不变：`analysis:1` 只用已实现的关键词与三条白名单 pattern 之一，
  `release-manifest:1` 只用 `^[0-9a-f]{64}$`；`tests/analysis_schema_contract.rs` 与
  `tests/release_schema_contract.rs` 各自断言"schema 声明的 pattern 全都在被求值的集合里"。
- 现有 majors 一个都没升（prompt §66：没有证据不得升）。

## C1 落地（2026-10-10）：附件字节的两处扩展披露，major 仍为 1

`ADR-0030` 选的 C1 路线只加披露，不加字段语义，所以这一节的结论与 P3、P4 同一手法：事实进
`extensions`，闭合的属性表一个都不动。

- `gate-results:1` 新增两把 `extensions` 键，**成对出现、否则整体缺席**：
  `canonical_input_label = "firmwaresight-gate-input/2"` 与
  `attachments: [{kind, sha256, byte_size}]`（按 §5 的 `(kind, digest)` 顺序、同一 `(kind, digest)`
  只出现一次）。没有附件的 run 一个键都不写，因此文档字节与 P4 当年写的完全一致——
  `apps/cli/tests/p4_golden.rs` 的逐字节比对是这条结论的独立证据，不是本节的自我声明。
- 成对规则**写在读者侧，不写在 schema 里**，这是设计后果而不是偷懒：`gate-results:1` 与
  `release-manifest:1` 的 `extensions` 都是 `additionalProperties: true` 的保留位，v1 故意不收紧它，
  这样事实才能不加 major 就进来；要在 schema 里表达“有 label 必须有行”，就得把一个开放对象改成
  受限对象，那是语义变更。于是 `verify_bundle` 承担这条规则：半边披露、`/1` 或未知语法标签、
  乱序或重复的行、与 manifest 披露集合不一致、`run_id` 与 `extensions.gate_run_id` 不相等，
  一律确定性拒绝（`ERR-BUNDLE-6109`，与既有读回拒绝同一个码，本层没有新分配错误码）。
- `release-manifest:1` 的 `extensions.attachments` 行由 `C1-U2` 落地，字段是
  `{path, kind, sha256, size, kind_basis, provenance}`，`provenance` 恒为 `"unknown"`。
  **`path` 是本契约的规范名（2026-10-10 定案）**：`04_TECH/28` §7.5 与 §12 早先草图写的 `file`
  作废，理由是 `files[]` 条目与验证器已经用 bundle-relative `path`，两个名字并存只会漂移。
  这条定案记录在 `04_TECH/28` §14.2，不靠删除旧文本来“假装没发生过”。
- `analysis:1` 一个字段都没加，也没有 `extensions` 保留位——附件是被哈希的文件，不是被分析的文件
  （`ADR-0030` D-1），所以它在这里连一个可以放伪分析行的位置都没有。`diff:1`、
  `accepted-reviews:1`、`project-config:1` 同样未变。
- 旧文档可读性这次是被测量过的，不是被断言的：`golden/reports/p4-release/` 里五个 JSON 文档
  逐个用今天的 `schemas/` 校验通过，目录本身由测试枚举（不是把文件名抄进测试），且断言它们
  都不含 `attachments` / `canonical_input_label`；把该 bundle 移到别处、补上它自己索引里已经写明的
  两个 committed fixture 工件后，当前验证器重新推导出的发布 id 等于它自己声称的
  `release-e400a8ac…`。
- 六个 major 全部保持 1；本节的“加披露”没有引入任何必填根字段。

