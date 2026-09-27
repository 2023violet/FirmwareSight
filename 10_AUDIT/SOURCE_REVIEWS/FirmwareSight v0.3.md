# FirmwareSight v0.3.0 基线 · 技术理解摘要

> 阅读范围：README + 04_TECH 全部 23 份 + 09_ADR 全部 17 份 + examples/ 与 schemas/ 基线样例
> 职责范围：技术战略、架构、所有技术决策记录
> 基线状态：PRE_IMPLEMENTATION（技术验证 ← 下一步，当前 Active Task = NONE，**创建 Baseline ≠ 授权实现**）

---

## 1. 技术战略："Core-first, Adapter-driven, Product-specific Shell" 的内涵

这不是一句口号，而是从 2026 Rust 产品普查中提炼、被 ADR-0007 正式化的工程原则，三段各有所指：

- **Core-first（核心优先）**：产品真正的资产是 binary parsing / normalization / diff / evidence / gate / release manifest，这些必须生活在**纯 Rust Core**（领域事实唯一归属地）。Core 禁止依赖 Tauri、Tokio 类型、rusqlite、WebView、UI——换壳、换库、换存储都不许动它。
- **Adapter-driven（适配器驱动）**：外部技术（ELF 解析、SQLite、Git、报告渲染）全部退到 Ports & Adapters 后面（artifact / storage / provenance / report 四类适配器）。解析器可替换、存储可迁移、未来 CI/headless 接入不需要重写核心。代价是接受一层映射代码（ADR-0007 明示接受）。
- **Product-specific Shell（产品专属壳）**：Desktop（Tauri 2 + React）只是壳，不拥有业务真相；CLI（`fwsight`）是**一等公民接口而非包装器**，与 Desktop 共享同一 Core——两条界面入口产出同一套事实。

配套的战略级克制：**依赖即成本**（09 决策矩阵把技术分为 NOW/EDGE/TRIGGERED/DEFER/REJECT 五档，只有 NOW 进入初始 bootstrap）；**不支持跳阶段**（任何新功能先问属于哪个阶段，否则默认不做）。

## 2. 分层架构与数据流

**四层模型 + 依赖方向强制：**

```
Interface 层：React Desktop ｜ fwsight CLI
      ↓
Application 层：use cases / jobs / 取消 / 错误映射（MVP 阶段先驻留在各 binary 内，
      ↓          出现明显重复后才抽 firmwaresight-app crate —— 有意的延迟抽象）
Pure Rust Core：Domain / Normalize / Diff / Gate / Release / Evidence / Policy
      ↓ ports
Adapters：Artifact(object+gimli+MAP 适配器) ｜ Storage(rusqlite) ｜ Provenance(git CLI) ｜ Report(JSON/HTML)
```

**禁止方向**：Core→Tauri/SQLite/Tokio、Parser→UI、UI→SQLite/文件系统直接访问。

**核心数据流**（Import 管道，04_TECH/16）：

```
用户选文件 → stat/size guard → 流式 SHA-256 → magic 格式探测 → 只读字节缓冲
→ Adapter 解析 → Normalize（进入稳定核心模型）→ Evidence 分类 → BuildSnapshot → SQLite 事务落库 → 查询 DTO → Desktop/CLI
```

关键不变量（02 领域模型）：
1. Snapshot 创建后**不可变**；Diff 不改 Snapshot；Gate 对同一 snapshot+policy **可重复求值**；
2. Declared 数据**不能覆盖** Observed 证据，两者并列可见；Unknown 显式存储；
3. 每个导出结论必须引用 evidence 或 policy——这是"evidence-first"产品主张的技术兑现；
4. 失败模型：适配器抛 typed diagnostic，Core **从不编造缺失事实**，Application 映射为稳定错误码，UI 渲染错误+能力状态。

Diff 和 Gate 只操作 normalized snapshot/policy，**从不重新解析原始文件**。

## 3. 已冻结技术栈与明确推迟项

**LOCKED（冻结）**：
- Rust 1.98.1 / Edition 2024 / Cargo resolver 3（rust-toolchain.toml 钉死，发布 CI 禁用 floating stable）
- Tauri 2 stable（禁 Tauri 3 alpha）+ React 19.3 + TypeScript strict + Vite 8 + CSS Modules/variables
- Node 24 LTS + pnpm 12（packageManager 钉 12.7.0，engines >=24 <25）
- Serde（仅用于 versioned 契约）/ thiserror（库层 typed 错误）/ anyhow 仅限 binary 边界
- Clap / tracing+tracing-subscriber / sha2（流式哈希）
- object crate（主 ELF 解析）+ gimli（可选、feature-scoped，仅 DWARF 证据）
- SQLite + rusqlite **bundled**（跨平台一致 SQLite，不赌系统库）
- Git CLI（只读 provenance，不引 gix）
- 初始 workspace 仅 4 个 library crate：core / artifact / storage / report

**条件/触发式（TRIGGERED——预批准但装了才算）**：Reqwest+Rustls（真有 HTTP 时）、wgpu（基准证据 + ADR 后的"Native/GPU Island"叶节点，不得拥有状态/门禁/证据/持久化）。

**明确推迟（DEFER）/ 拒绝（REJECT）**：
- DEFER：SQLx、Axum/Tonic、Redux/Zustand、Tailwind、全量 UI kit、图表库、云 DB
- REJECT（main shell）：egui/iced/Slint（不是"差"，是收益未证明）、GPUI/Floem（pre-1.0 生态风险）、Electron（无理由分发 Chromium）、Telemetry SDK（MVP 隐私立场）
- MVP 无网络（本地优先，缩小攻击/依赖面）、无 GPU（100k 符号表先靠索引/分页/虚拟化解决）

**明确的平台态度**：Windows Tier 1（嵌入式工具链客户群 Windows 居多，要"excellent on Windows"）；macOS/Linux Tier 2；Win/Linux ARM64 推迟；Mobile 出局。

## 4. 关键 ADR 决策及理由（17 份速览）

| ADR | 决策 | 核心理由 |
|---|---|---|
| 0001 | 定名 FirmwareSight，CLI `fwsight` | 避开 FirmwareLens 撞名；GA 前须做商标/domain clearance；domain model 不绑定品牌名 |
| 0002 | Tauri 2 + React 19 + TS strict + Vite 8 | 数据密集型工具（表格/过滤/diff/报告）非 GPU 编辑器；React 胜在生态/测试/人才与 AI 语料；壳迁移不得要求重写 Core |
| 0003 | 纯 Rust Core + 共享 CLI | CLI 是真实产品接口；解析/存储可在 ports 后替换；单测不需要 WebView/数据库 |
| 0004 | Local-first / offline-first | 固件产物、符号名、路径敏感，目标用户常在离线研发环境；换工程团队信任 |
| 0005 | 可信核心禁入 AI | Gate/evidence 必须可复现可审计；未来 AI 只做解释/搜索/总结且标 advisory |
| 0006 | MVP 格式范围收敛 | ELF 32/64 + GNU ld MAP + BIN/HEX 基础元数据 + Git provenance；Keil/ArmClang/IAR 须 fixture+测试后才升 Supported。范围小但可信 |
| 0007 | 正式化 Core-first/Adapter-driven | 领域事实只在 Core；接口层永不持有业务真相；适配器类型不得泄入 Core 公共契约 |
| 0008 | 异步边界 | Async 是执行关注点非领域属性：Core 同步、CLI 默认同步、Desktop 只在 application 边缘用 Tauri/Tokio；Tokio 类型绝不跨入 Core API |
| 0009 | SQLite via rusqlite+bundled | 本地同步桌面场景，拒绝 SQLx（async/多库无需求）；bundled 消除系统 SQLite 差异 |
| 0010 | object 主解析 + gimli 可选 + 自有 MAP 适配器 | 缺 DWARF/MAP 只降级能力，绝不否定合法 ELF 导入 |
| 0011 | 前端基线极简 | 无 Redux/全 UI kit/图表框架；产品自持设计语言，库只在证明的复杂点引入 |
| 0012 | tracing 结构化日志 | 解析不可信产物需要可复现字段化诊断；日志默认不含源路径/符号/固件内容 |
| 0013 | 网络客户端推迟 | MVP 离线成立；预批准 Reqwest+Rustls，触发条件=签名更新/CVE feed/licensing |
| 0014 | GPU 推迟 + Native Island 政策 | 先走 IPC 瘦身→SQLite 分页→虚拟化→Canvas/SVG 升级阶梯；Island 只能是叶适配器 |
| 0015 | 交付链路从第一天设计 Build→…→Signed Update | 但 MVP 关闭 auto-update；启用需签名钥/托管/回滚/离线开关齐备——避免末期架构意外，也不提前上风险 |
| 0016 | 工具链钉死 | 可复现性 > 追新；升级=独立 PR+全量 CI；major 依赖升级须 ADR |
| 0017 | Git CLI 做 provenance | 所需事实很少（commit/tag/dirty/branch）；直接 spawn、无 shell、porcelain 输出、超时、typed 错误；无 Git 时降级 Unknown 而非失败 |

**跨 ADR 的共同模式**：每一个"推迟/拒绝"都写了**触发条件**（revisit trigger）——决策是可逆的、有证据门槛的，不是一刀切。

## 5. IPC 与数据契约要点

- **Command 即用例**：`open_project / import_artifact / get_analysis_summary / query_symbols / compare_builds / run_release_gate / prepare_release_bundle`；禁止 `read_file / execute_shell / run_sql / get_any_path` 这类原语暴露——前端拿不到任意磁盘/SQL 能力（Tauri capability 按 window 最小授权，样例仅 `core:default`）。
- **DTO 纪律**：Serde 可序列化、有界、含稳定 ID；绝不外泄 rusqlite/Tauri/object crate 类型；IPC 不出现 `any`。
- **大数据分页**：100k 符号表**默认永不整体过 IPC**——请求 filter/sort/cursor/limit（有上限），响应 rows+next_cursor+total_or_estimate+capabilities。
- **错误信封**：`code / message / operation_id / details? / remediation?`；堆栈永不进 UI。稳定错误码 + contract tests + golden fixtures，禁止字段的静默重释。
- **版本化分层**：Desktop 前后端同车发布 → IPC 本身 MVP 不做公共 semver；但**可移植 schema（如 release manifest）是 versioned public contract**（schemas/release-manifest.schema.json，schema_version=1：release/build/artifacts 必填，sha256 正则校验）。
- **类型生成**：Rust 是 IPC DTO 语义 source of truth；Phase 0 做小 spike 后选维护中的 Rust→TS 生成器；**选型前禁止手维护 100+ 重复类型模型**。
- **CLI 契约**：`--json` 输出稳定 schema、默认 human-readable；stdout=结果 / stderr=诊断；exit code 0/2/3/4/5/6（10+ 保留），CI 禁止解析自然语言判结果；默认不覆盖 bundle，需 `--force`；`--json` 无多余 wall-clock 字段、路径尽量 project-relative。
- **存储事务**：导入走单事务（build 置 IMPORTING → 插 artifacts/sections/批量 symbols/evidence → COMPLETE），失败则绝不呈现"半导入"为有效；SQLite 配置 foreign_keys=ON、WAL、busy timeout、显式迁移版本；每张 schema 变更=编号迁移+老库升级测试+备份策略，禁止静默重置。
- **SQLite 的定位**：只是本地结构化索引/查询/历史载体，**不是唯一真相**；可移植交换格式是 versioned JSON/HTML/Release Bundle。

## 6. 发现的疑点 / 风险

1. **Rust→TS 类型生成器选型悬空（P0 必须闭环）**：14 号文档明确"v0.2 policy：Phase 0 spike 后选 generator，此前禁止手维护重复模型"。这意味着 UI 开工前有一项硬前置任务；若 spike 拖延，前端极易滑向手写重复类型（被明令禁止），需要作为 P0 检查项盯住。
2. **release-manifest.schema.json 偏宽松**：几乎所有对象 `additionalProperties: true`，且 `$id` 用 `schemas.firmwaresight.local` 占位域名。作为"versioned public contract"，宽松策略利于演进但弱约束；建议实现阶段明确"哪些字段允许 additional、何时收紧 major"，并落实真实 schema 托管。
3. **大文件内存峰值策略**：MVP 无 mmap、一次性 immutable byte buffer 交给解析器；导入上限"数百 MB"（非 multi-GB），size guard 可配置。与"100MB ELF 2 秒基础解析"预算匹配，但峰值内存 ≈ 文件大小+解析结构，P0 需真实 fixture 验证 guard 与流式哈希的组合行为。
4. **MAP/工具链格式是最大兼容性长尾**：GNU ld map 无正式规范、随 binutils 漂移；Keil `.axf` "often ELF but no blanket promise"。文档已把 fixture（含 malformed corpus）作为 Supported 的准入门槛——fixture 库建设实际是解析可信度的生命线，P0 第一步就是 Real ELF fixture。
5. **文档标题残留 v0.2**：00_TECH_STACK / 01_SYSTEM_ARCHITECTURE 的 H1 仍写 "v0.2"，14 号文档写 "v0.2 policy"，但 front-matter 均为 version 0.3.0。属小瑕疵，引用时以 front-matter 为准；建议下次 baseline 统一清理。
6. **Updater 私钥是单点高危资产**（文档自认）：丢失即已装客户端无法更新。MVP 关闭 updater 正确；启用前六项触发条件（密钥托管/回滚/离线开关等）缺一不可，需运营侧提前建流程。
7. **平台 WebView 碎片化**：Tauri 用系统 WebView（Win=WebView2 / macOS=WKWebView / Linux=WebKitGTK），文档已要求避免实验性 API、逐平台测 CSS/键盘/文件对话框；Linux WebKitGTK 依赖预期需文档化。这是壳选型的固有代价（ADR-0002 negative 项），测试矩阵要当真。
8. **商标/domain clearance 未完成**（ADR-0001 后果项）：GA 前必做；若失败品牌可换，domain model 已解耦，风险可控但别拖到发布前。
9. **小观察**：README 第 3 节 ASCII 架构图把 adapters 混排且未画 Report Adapter（01 号文档是四类齐全）；阅读时以 01_SYSTEM_ARCHITECTURE 为准，避免以简图建立心智模型。

---

**一句话总结**：这是一份"克制到骨子里"的技术基线——用纯 Rust Core 锁住领域真相，用 Ports & Adapters 隔离一切易变技术，用 NOW/EDGE/TRIGGERED/DEFER/REJECT 五档决策矩阵管理依赖成本，用不可变 Snapshot + 四级证据分类 + 可重复 Gate 兑现"Know exactly what ships"；所有推迟项都带触发条件，下一步唯一合法动作是 P0 Technical Vertical Slice（真实 ELF → BuildSnapshot → fwsight CLI JSON → Tauri Minimal Summary）。
