# 端口扩展与深度体验·技术可行性简报

| 项 | 内容 |
|---|---|
| 文档性质 | Post-MVP 探索稿（不触发任何 Active Task，不授权实现） |
| 作者 | 鲁班七号（全栈开发） |
| 日期 | 2026-09-27 |
| 输入基线 | FirmwareSight v0.3.0（未改动副本）+ 群内已冻结交付（DESIGN.md / tokens v0.2.0 / ADR-0018 / 代码区规划） |
| 议题来源 | 用户提出：探索更多"端口"建立工作流粘性，深耕窄人群体验 |

---

## 0. 结论速览

| # | 候选端口 | 架构匹配度 | 成本 | 建议阶段 | 插槽类型* |
|---|---|---|---|---|---|
| ① | 输入端口：.axf / ArmClang·IAR MAP / CSV 尺寸 / .su / build log | 高（adapter 插槽现成） | 每格式 S–M | MAP 工具链包 P1–P2；CSV P2；.su 须 ADR；log 不做 | A（.su 为 C） |
| ② | 事件端口：watch 模式 | 高（应用层触发源，Core 零改动） | M | 工程 spike 可 P2–P3；产品决策放 V1 后 | B |
| ③ | CI 端口：GitHub Action / GitLab | 高（前置=headless CLI） | Action S–M、GitLab S | Growth（两处基线口径一致） | C |
| ④ | CLI 管道端口 | 极高（JSON/exit codes 已冻结） | S | P1–P2 | A |
| ⑤ | 编辑器端口 | 低–中（存在"IDE 内置墙"） | 配方 S；扩展 M–L | 配方 P1；扩展 Post-GA 条件触发；LSP 不做 | A（配方）/ D（LSP） |
| ⑥ | Git/provenance 深化 | 高（薄适配器是设计意图） | S | P3（随 Gate 批次）；gix 除外 | A（gix 为 C） |
| ⑦ | 数据端口：toml/policy/schema | 极高（schema_version 即演进插槽） | S–M | P1–P2 | A |

\* 插槽类型定义见 §1。

**一句话结论**：七个端口里有五个踩在架构已预留的插槽上（A 类），真正的 ADR 级高成本项只有三个（.su 域模型扩展、CI 分发通道、gix 替换）；**性价比最高的粘性路径不是加新端口，而是把 ④⑦⑥⑤ 中四件 S 级"插槽内事项"先做掉**——JSON schema 版本化输出、config JSON Schema 发布、git hooks/CLI 管道配方、VS Code task 配方。这些零新概念、零新依赖，但每一样都会在用户的脚本、仓库和日常动作里留下"走不走都得经过 FirmwareSight"的接点。

---

## 1. 评估框架与口径

### 1.1 阶段轴

采用 **FS-DEL-001 路线图**为唯一阶段轴：

```text
P0 Foundation → P1 Analyzer → P2 Compare → P3 Gate → P4 Bundle
→ MVP Candidate → V1 外部验证 → P5 Productization → B1 → RC1 → GA → Growth
```

⚠️ 口径提醒（沿用此前简报结论）：PRD §3 的「P1 能力」清单（CI/headless CLI、policy presets、Keil/ArmClang MAP、richer treemap 等）是**产品层批次**口径，与路线图的 P1 Analyzer 阶段**不同名不同义**。本简报引用 PRD「P1 能力」时一律标注"产品层 P1"。

### 1.2 成本分级

- **S** ≈ ≤1 人周（含 fixture、测试、golden、文档）
- **M** ≈ 1–4 人周
- **L** ≈ >4 人周，或引入新子系统/新代码库

### 1.3 插槽类型（本简报核心分类法）

| 类型 | 含义 |
|---|---|
| **A 已预留插槽** | 不改任何冻结契约即可实现（adapter trait / ReportRenderer / `--json` / config unknown-key 演进） |
| **B 插槽内新功能** | 架构允许，但属新功能：须过 Scope Creep 四问 + 新依赖过 FS-TECH-012/05_DEPENDENCY_POLICY 准入 |
| **C 须 ADR** | 改公共契约、改分发形态、改 Non-goals，或触发既有 ADR 的 revisit trigger |
| **D 不做** | 与 Non-goals / 生命周期 §27 明文冲突 |

### 1.4 治理红线

本简报全部结论为探索性质；Active Task = NONE 不变；任何一条落地前须逐项过 Scope Creep 四问（FS-PRD-004）+ 对应 ADR/依赖准入。

---

## 2. 端口①：输入端口（更多 artifact 格式）

### 2.1 既有插槽

- FS-TECH-017 管线固定为 `Adapter parse → Normalize → Evidence classify → BuildSnapshot`，且给出 `MapAdapter` 接口样例（`detect(text/header) -> confidence` / `parse -> NormalizedMapEvidence`）——**新格式 = 新 adapter + fixtures + 能力报告，管线本体零改动**。
- FS-TECH-004 定义了 Supported 四要件：fixture、parser tests、known limitations、UI capability reporting；MAP 适配器按 toolchain 分离（gnu_ld / armclang / iar / future adapters），**明文禁止"一个大正则通吃"**。
- ADR-0006：Keil/ArmClang/IAR 只有 fixture + adapter tests 齐备后才升 Supported。
- 支持矩阵现状（FS-TECH-004）：Keil `.axf` = "Experimental until fixtures（often ELF, but do not blanket promise）"；ArmClang map = P1；IAR = P1/P2；SREC = P1；COFF/PE 无承诺。

### 2.2 逐格式评估

| 格式 | 技术判断 | 成本 | 证据等级 | 建议阶段 | 类型 |
|---|---|---|---|---|---|
| Keil `.axf` | 绝大多数是 ARM ELF 容器 → 直接走既有 `object` ELF 路径；工作在检测标注、真实 fixture、能力报告措辞（建议输出 "axf (ELF container): Supported/Partial"），不给 blanket promise | S–M | Observed | 与 ArmClang MAP 同批（产品层 P1 已列 Keil/ArmClang）；P1 尝试、P2 转正 | A |
| ArmClang `.map` | MapAdapter 槽位现成；难点在分区语法与 scatter 布局语义差异 | M | Observed | P1–P2（矩阵已承诺） | A |
| IAR `.out/.map` | 同上 | M | Observed | P1/P2（矩阵已承诺） | A |
| CSV/文本尺寸报告（`arm-none-eabi-size`、CSV） | 行格式解析极简；定位为 **Declared 侧输入**——核心价值是"无 ELF 可用时的 diff 基线"（如 CI 机上只有尺寸报告）；**不可覆盖 Observed**（既有证据纪律）；与 ELF 同名 section 冲突时须在 spec 写死合并规则（建议：仅当无 ELF 才作为尺寸来源） | S | Declared/Derived | P2 | A |
| `.su`（GCC `-fstack-usage`） | 行格式解析 S；但引入**新证据维度**（per-function stack usage）→ 触碰 02_DOMAIN_MODEL、存储 schema、diff 引擎、UI，整体 M；且要求用户以 `-fstack-usage` 编译，天然有覆盖缺口 | 解析 S / 整体 M | Declared（依赖编译开关，非二进制观测） | 需求信号待 V1 访谈验证后再立项 | **C（须 ADR：域模型扩展）** |
| `.bss` | **不是独立文件格式**——是 ELF/MAP 内的 section。若指外部尺寸清单，归入上表 CSV 家族；不建议为它新造格式。此项属议题口径澄清 | — | — | — | 澄清项 |
| build log | 工具链/版本/ verbosity 方差极大，维护是无底洞；且 toolchain 信息可从 ELF `.comment` **Observed** 取得，log 只能给 Declared 级别、增益低。不进支持矩阵，仅诊断层"尽力提取" | — | Declared 以下 | 不做 | D（见 §9） |

### 2.3 成本结构提示

每个新格式的真实成本大头**不是 parser**，而是：真实 fixture 获取 + golden 维护 + 能力报告措辞 + 支持矩阵文案。FS-TECH-004 的四要件是成本放大器，也是信任护城河——这与"少量输入→高可信输出"的定位一致。**端口数量 ≠ 粘性；端口可信度 = 粘性**。建议优先级：Keil 包（axf+armclang MAP，直击 Persona A 中最被冷落的 Keil 人群）> IAR > CSV > .su > log。

---

## 3. 端口②：事件端口（watch 模式）

### 3.1 实现路径（Desktop 壳内，Core 零改动）

```text
[artifacts] 配置路径（建议默认，而非整个 target/）
  → notify crate 监听（inotify / FSEvents / ReadDirectoryChanges）
  → debounce：size/mtime 稳定窗口（约 0.5–2s）
  → 入队既有 import use case（应用层）
  → 完成后发 Tauri 事件 → UI stale/success 态
```

- **并发语义现成**：FS-TECH-013 §5 已写死 "one active import per project" + "SQLite writes serialized" → watch 触发只需合并/去重策略；§4 已有 operation id / progress / cooperative cancellation → 长任务语义现成。
- **解析安全现成**：管线有 "Read immutable bytes + Streaming SHA-256"，快照不可变——文件在分析中途再变更 = hash 变化 → 重新入队即可，数据不会被污染。
- **UI 状态现成**：FS-TECH-014 已把 `stale` 列为每个 feature 必须实现的六态之一——**watch 需要的 UI 状态基线已经写死，这正是插槽存在的直接证据**。
- **资源成本**：OS 事件驱动，空闲近零 CPU、无轮询；内存增量远小于分析本身。
- **主要平台坑**：Windows 链接器写文件的锁占用 → 需 retry/backoff（工程细节，可解）；incremental build 事件风暴 → 靠"只监听配置产物路径 + debounce"规避。
- **纪律**：自动分析 ≠ 自动 Gate/Bundle；通知呈现保持克制（设计侧备忘覆盖，技术侧不做打扰式弹窗）。

### 3.2 成本与分类

- 成本 **M**（含 notify 依赖准入评审、触发/合并语义 spec、跨平台测试）。
- 分类 **B**：架构允许；但 ① 新增常驻依赖 `notify` 须过 FS-TECH-012 准入清单；② 作为新功能过 Scope Creep 四问——问 4（是否延迟 MVP）大概率 = 是 → 默认推迟。
- 建议：工程 spike 可在 P2–P3 间隙做（重点验证 Windows 行为）；**产品正式决策放 V1 外部验证之后**，凭"用户是否真的高频重复分析"信号拍板。CLI 侧 `fwsight watch` 无必要（桌面场景为主）。

---

## 4. 端口③：CI 端口（GitHub Action / GitLab CI）

### 4.1 先说清与"CLI 归属口径分歧"的关系

群内已知的分歧点（专业写手在基线交叉审查中标出）：

| 出处 | 表述 |
|---|---|
| FS-DEL-001 P0 范围 | vertical slice 含 **"CLI JSON"**（CLI 是 MVP 的一部分，已在 P0 证明） |
| FS-PRD-004 In-Scope MVP | "CLI 与 Desktop 共用 Rust Core" |
| FS-PRD-002 §3 | "CI/headless CLI" 列为**产品层 P1 能力** |
| FS-DEL-001 Growth + 生命周期 §26 | **CI Integration（Action/Wrapper）= v1.0 之后** |

综合解读（建议口径）：**MVP 内的 CLI = 核心命令 + 稳定 JSON/exit codes（P0 slice + P4 bundle 已覆盖）；headless 硬化 = MVP 后第一批（产品层 P1）；GitHub Action/GitLab 封装 = Growth。** 三层不矛盾，是三个批次。CI 端口的一切排期以 headless CLI 为前置，此口径建议在 MVP 口径冻结时一并拍板（开放问题 §10-1）。

### 4.2 可行路径

- **GitHub Action**（独立仓库，composite action 基线）：
  1. 按版本 pin 下载对应 runner 平台的 `fwsight` 发布产物（ubuntu/windows/macOS runner 矩阵 → 依赖 `05_ENGINEERING/04_RELEASE_ENGINEERING.md` 的产物矩阵扩展到三平台 CLI 压缩包）；
  2. 执行 `fwsight gate --project . --json`（生命周期 §26 给的入口命令正是它）或 `fwsight diff`；
  3. **渲染与解析分离**：CLI 只出稳定 JSON；PR comment 的 markdown 由 action 侧脚本渲染——CLI 保持纯与确定性。
  4. **exit codes 已编码 Gate 语义**（0 pass / 4 review / 5 block，FS-TECH-008）→ action 逻辑退化为映射：BLOCK=5 → check failure，REVIEW=4 → neutral + annotate，全程无自然语言解析（基线红线"CI 不允许解析自然语言判断结果"）。
  5. 基线对比物：repo 内提交的 baseline 目录或 workflow artifact 传递。
- **GitLab CI**：更薄——CI 模板 + merge request note/artifact，S。
- **安全/治理姿态**：GITHUB_TOKEN 最小权限（PR comment 够用）、action 版本 pin、**无遥测无上传**（local-first 红线延伸到 CI 场景）、确定性不受 CI 环境影响（CLI JSON 本就禁 wall-clock 字段）。

### 4.3 成本、分类、阶段

- 成本：headless 硬化 M（前置）；GitHub Action S–M；GitLab S。
- 分类 **C**：新增分发通道与安全姿态 → 须 ADR，并牵动 release engineering 产物矩阵。
- 阶段：**Growth**（FS-DEL-001 Growth 清单与生命周期 §26 两处口径一致）；若 V1 出现团队付费信号，可提前至 P5 试点。
- 粘性/商业联动：CI 是 Team 层付费锚点（商业模型 Team = 共享 policy / CI / 审计 / 企业打包），与端口⑦的 team policy 咬合——**CI 端口不在粘性starter里，而在变现层里**。

---

## 5. 端口④：CLI 管道端口

### 5.1 已冻结基础（FS-TECH-008）

五命令（analyze / diff / gate / release prepare / project doctor）、`--json` 稳定 schema、stdout=结果 / stderr=诊断、exit codes `0/2/3/4/5/6`（10+ reserved）、确定性（无 wall-clock、project-relative 路径）、默认不覆盖（`--force`）。**这是七个端口里插槽打得最死、天花板最低风险最小的一个。**

### 5.2 低成本补强清单（全部 S、全部 A 类）

1. **JSON 输出带 `schema_version` 字段**，与 `schemas/` 版本化策略对齐（P1）；
2. **全命令 `--json` 覆盖**（diff / gate / release prepare），逐命令 golden——随 P2/P3/P4 各阶段自然发生，不单独立项；
3. **stderr 诊断 ID** 与 FS-TECH-021 观测性规范对齐（P1）；
4. **管道配方文档化**（P1，文档级）：`fwsight analyze --json | jq '...'`、git pre-push hook 跑 `fwsight gate`（与端口⑥联动）、CI job 模板示例、Makefile/npm scripts 集成片段。

### 5.3 为什么这是性价比之王

管道化是"让用户走得慢"的最短路径：**脚本、hooks、Makefile target 一旦写下，迁移成本即刻产生**——零新概念、零新依赖、零 ADR、零契约变更，全在已冻结规格内做增量。建议随 P1 Analyzer 批次顺手收割，不占独立排期。

---

## 6. 端口⑤：编辑器端口（VS Code 扩展 / LSP）

### 6.1 边界先行

- FS-PRD-004 Non-goals **明文**："不做 IDE：不编辑 C/C++ 源码，**不做 LSP**，不做编译器前端"。
- Project Charter 非 "替代 Keil/IAR/VS Code"。
- 结论：**LSP = D 类，不做**（做即等于修改 Non-goals，须 ADR 且方向性冲突）；**VS Code 扩展不被明文禁止，但价值存疑**。

### 6.2 价值判断

- 外部调研（信息哨兵）已指出 **"IDE 内置墙"**：IDE/toolchain 自带 memory/size 视图，FS 在编辑器内的差异化只剩 release 语义（gate/diff/evidence）——而那正是 Desktop 与 CLI 的主场。扩展 M–L 成本换重复价值，性价比低。
- "产品替代焦虑"的解药不是出现在更多容器里，而是在既有容器里**不可撤换**（脚本接点、配置真源、证据包格式）。

### 6.3 低成本替代（S 级，推荐 P1 就做）

1. **firmwaresight.toml 的 JSON Schema 发布**（见端口⑦）→ 用户在 VS Code 装 Even Better TOML 即得补全/校验——**零扩展代码拿到编辑器存在感**；
2. 文档提供 `.vscode/tasks.json` 配方（一键跑 `fwsight analyze` / `fwsight gate`，读 exit code 判定）；
3. report.html / Release Bundle 本就是"编辑器外的深度信息面"。

### 6.4 建议

配方 P1 文档级；扩展 **Post-GA 且仅当 V1/B1 出现真实编辑器内需求信号**再立项（届时仍须 Scope Creep 四问）。

---

## 7. 端口⑥：Git/provenance 端口深化

### 7.1 既有

ADR-0017 / FS-TECH-023：安装版 `git` CLI 只读适配器（`--porcelain`、timeout、spawn 直调、路径清洗）；降级态明确（无 Git → provenance Unknown → Gate 规则 Review/N/A）；**gix 替换已写死 revisit trigger 且"须 ADR"**——不主动排期。

### 7.2 深化空间（按成本排序）

| 项 | 说明 | 成本 | 阶段 | 类型 |
|---|---|---|---|---|
| 只读事实小扩容 | `describe --tags --always` 兜底命名、commit date（可复现时间戳候选源）、branch | S | P3（Roadmap P3 本就含 Git provenance，随批落地） | A |
| hooks 配方 | pre-push / pre-tag 跑 `fwsight gate` 的官方配方文档——纯文档，代码零 | S | P1 文档级 | A |
| CI 环境变量 provenance | GITHUB_SHA / GITLAB_CI 等 CI 注入变量作为 **Declared** 证据源——为端口③铺路，证据分级模型现成 | S–M | 与端口③同批（Growth）或先行 | A |
| 非 Git 来源 | source tarball + SHA、构建元数据文件——每源须 fixture + spec + 证据定级 | M | Post-MVP | B |
| gix 替换 | 已由 ADR-0017 写死触发条件 | — | 不排期 | C（已被既有 ADR 覆盖） |

---

## 8. 端口⑦：数据端口（firmwaresight.toml / policy / schema）

### 8.1 既有插槽

FS-TECH-009 的演进机制**已内建**：`schema_version` 门控 + "unknown keys: warning in same major schema" + 类型错误硬失败 + "config error 不允许被默认值悄悄覆盖" + 正式 JSON Schema 在实现阶段冻结。即：**TOML 配置天生按"可加字段"设计**，这是七个端口中最优雅的预留。

### 8.2 演进清单

| 项 | 说明 | 成本 | 阶段 | 类型 |
|---|---|---|---|---|
| config JSON Schema 落地 | `firmwaresight.config.schema.json` 进 `schemas/`、随应用分发、文档页发布；**SchemaStore 提交须稳定公开 URL**——产品仓库当前私有（许可证政策），GA 前先"随应用分发 + 文档页"两步走 | S | P1 | A |
| policy presets | 产品层 P1 已列；presets = 随应用分发的命名 TOML 片段，Gate 引擎本就 deterministic | S | P1 | A |
| `[artifacts.paths]` / `[watch]` 段 | 直接喂端口② watch 与端口③ CI 的路径配置；**先有 config 再有功能**——顺序反了功能就会私造配置 | S–M | P2 | A |
| per-section budgets | Gate 规则 6/7（FLASH/RAM budget）泛化到 section 级——**依赖 EPIC B 的 FLASH/RAM 分类策略先冻结**；gate 语义扩展逐条 spec + golden | M | P2+ | A（gate 语义部分须 spec） |
| release-manifest schema 收紧 | 此前审计发现 `additionalProperties` 偏宽松 + `$id` 为 `.local` 占位——在实现冻结窗口一并处理 | S | 实现冻结时 | A |
| schema_version 迁移机制 | 首次破坏性变更时建立；错误信息须给迁移指引（符合"不被悄悄覆盖"气质） | S–M | 首次 breaking change 时 | A |
| Team policy 免费红利 | firmwaresight.toml 可进 Git = **"共享 policy"的最小实现已经躺在设计里，代码量为零**；与商业模型 Team 层（共享 policy/CI/审计）天然咬合 | 0 | 已存在 | A |

### 8.3 边界

config 唯一不该去的方向：可执行/可编程 policy（表达式 DSL）——破坏 deterministic 与可审计性，列入 §9。

---

## 9. 不值得做的端口（与 Non-goals / 治理明文冲突）

| # | 端口 | 依据 | 替代答案 |
|---|---|---|---|
| 1 | **Plugin System / 扩展运行时** | 生命周期 §27 明文"不要直接跳到"清单 | 把"插件需求"导向已冻结的**类型化端口**：CLI JSON、TOML、schemas、report IR（ReportRenderer）、adapter fixture 规范——端口够用，不需要运行时 |
| 2 | **LSP / C/C++ 语言特性** | FS-PRD-004 明文"不做 LSP" | config JSON Schema + tasks.json 配方（§6.3） |
| 3 | **构建系统集成**（CMake/Make/PlatformIO 插件、构建注入） | §27 "Build System" | watch 模式（观察产物，不接管构建） |
| 4 | **通用 MAP / build log "大正则"解析器** | FS-TECH-004 明文禁止"一个大正则通吃"；log 证据等级低、维护无底洞 | 按 toolchain 的 MapAdapter；`.comment` Observed 取 toolchain |
| 5 | **Flasher / OTA / 设备管理端口** | Non-goals + §27 | — |
| 6 | **云端口**（webhook、在线协作、云 dashboard） | ADR-0004 local-first + ADR-0013 network deferred（网络面仅预留 update metadata / CVE feed / license API） | CI 在用户侧 runner 跑本地 CLI（§4），不上传 |
| 7 | **逆向 / 漏洞分析导入端口** | Non-goals "不做 Firmware Reverse Engineering Suite" | — |
| 8 | **可编程 policy DSL** | 破坏 deterministic/可审计（治理气质冲突，无明文，存档） | 声明式 TOML + presets |
| 9 | **AI 判断端口** | ADR-0005 no-AI trusted core | — |

---

## 10. 开放问题（须拍板）

1. **CLI 口径冻结**（影响端口③/④排期）：是否接受 §4.1 的三层口径（MVP=核心命令+稳定 JSON；headless 硬化=产品层 P1；Action 封装=Growth）？建议：接受，并在 MVP 口径冻结文档中记为一句话。
2. **watch 模式**：接受"工程 spike 可早、产品决策放 V1 后"，还是直接列入 MVP 后第一批？建议：前者。
3. **Keil/.axf 批次 fixture 来源**：需要真实 Keil/ArmClang 工程产物；无内部渠道时是否公开征集 fixtures（涉及商业/法务口径，需用户拍板）？
4. **config JSON Schema 公开渠道**：私有仓库期间先"随应用分发 + 文档页"、GA 后再提交 SchemaStore，两步走是否接受？建议：接受。
5. **`.su`（stack usage）**：是否列入 V1 用户访谈验证问题清单（需求强度/付费意愿）？建议：列入。

---

## 11. 汇总：优先级建议

```text
第一梯队（P1，S 级，插槽内直接做）：
  ④ CLI --json 全覆盖 + schema_version + 诊断 ID
  ⑦ config JSON Schema + policy presets
  ⑥⑧ git hooks / tasks.json / jq 管道配方（文档级）
  ① Keil 包启动（axf+armclang MAP adapter，视 fixture 获取）

第二梯队（P2–P3，M 级，随阶段自然发生）：
  ① IAR MAP、CSV 尺寸报告
  ② watch spike（P2–P3 间隙，Windows 行为验证）
  ⑦ [artifacts.paths]/[watch] 配置段、per-section budgets

第三梯队（Post-MVP / Growth，须 ADR 或凭 V1 信号）：
  ③ headless 硬化（产品层 P1）→ GitHub Action / GitLab（Growth）
  ② watch 产品化（V1 信号后）
  ① .su 域模型扩展（V1 访谈后）
  ⑤ VS Code 扩展（Post-GA 条件触发）
  ⑥ 非 Git 来源、gix（触发条件制）

不做：§9 全表。
```

---

## 12. 引用与治理声明

- 基线引用：FS-TECH-004 / 008 / 009 / 013 / 014 / 017 / 023；FS-PRD-002 / 004；FS-DEL-001 / 003；06_DELIVERY/05_MVP_TO_PRODUCT_DEVELOPMENT_LIFECYCLE §26–§27；05_ENGINEERING/06_CI_CD_BASELINE（FS 自身 CI，非用户 CI 端口）；ADR-0004 / 0005 / 0006 / 0007 / 0013 / 0017。
- 群内交付引用：FS-ENG-009 代码区规划（apps/cli 壳、report crate ReportRenderer 端口）；ADR-0018 与 DESIGN.md（UI 红线，设计侧备忘另发）；信息哨兵外部调研（IDE 内置墙）。
- 外部组件（notify 等）版本未锁定，实施前按 FS-TECH-012 依赖准入清单重新核验。
- 本简报为 Post-MVP 探索稿：未创建 Active Task、未修改基线、未授权任何实现；所有"建议阶段"均须在立项时逐项过 Scope Creep 四问 + 对应 ADR/依赖准入。
