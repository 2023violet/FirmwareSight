# 真需求全景扫描·技术可行性简报

| 项 | 内容 |
|---|---|
| 文档性质 | Post-MVP 探索稿（不触发 Active Task，不授权实现） |
| 作者 | 鲁班七号（全栈开发） |
| 日期 | 2026-09-27 |
| 输入基线 | FirmwareSight v0.3.0（未改动副本）；动笔前对 02_DOMAIN_MODEL / 17_RELEASE_PACKAGING / 01_SBOM_STRATEGY / 02_CRA_CONTEXT / release-manifest.schema.json / Cargo.workspace 基线样例做了原文复核 |
| 议题来源 | 第三轮《真需求功能全景扫描》——群管理员指定 5 项重点候选 + 12 条外部信号 |
| 上游输入 | 信息哨兵《FS-真需求全景扫描-外部信号报告》＋ 通用助手《背景回顾-已覆盖面与本轮扫描边界清单》（均群内可查） |

---

## 0. 结论速览

| # | 候选 | 信号 | 架构匹配度 | 插槽类型* | 成本 | 建议阶段 |
|---|---|---|---|---|---|---|
| C1 | 组件版本漂移检测（S-05/H8） | 中 | 高——数据面已排期（ComponentEvidence），Diff 已有 identity_changes 先例 | **B** | **M** | **P2**（门禁规则随 P3） |
| C2 | 栈安全证据 .su（S-01） | 强（佐证升格提请） | 高——adapter 槽位现成，但 per-function 是新证据维度 | **C**（须 ADR） | 解析 S / 整体 M | 升格则 **P2** |
| C3a | SBOM evidence completeness（S-04/H3） | 中 | 高——evidence 模型 + report crate 现成 | **B** | **M**（定义为主，实现 S） | **P2** |
| C3b | evidence retention（S-04/H4） | 中 | 高——P1 History + P4 Bundle 已覆盖约八成 | A/B | **S–M** | **P4 增量** |
| C4 | 发布物符号档案（S-06） | 中 | 极高——selected_artifacts + manifest 插槽现成，零新解析器 | **A** | **S** | **P4** |
| C5 | 组件 ABI 兼容性门禁（S-07） | 中 | 中——gimli 已在栈免依赖 ADR，但深度 DWARF 消费是子系统 | **C**（须 ADR） | 两步走：M → **L** | 第一步可搭 P2/P3；第二步 **Post-GA 凭信号** |

\* 插槽类型沿用二轮简报 §1.3 分类法：A=已预留插槽 / B=插槽内新功能（过 Scope Creep 四问）/ C=须 ADR / D=不做。

**一句话总评**：本轮外部信号指向的"证据完整性与可追溯性"方向，在架构上全部落在既有插槽或其自然延伸上——**没有一个候选需要新 crate、新依赖 ADR 或触碰 Non-goals**。真正的分水岭只有两个：.su 是域模型扩展（C 类），ABI 是子系统级（L）。综合环节若只带一件走，带 C1 组件漂移（P2、B/M，数据前提就是已排期项）；带两件，加 C4 符号档案（P4、A/S、零解析成本）。

**给管理员口径的换算说明**：本文"建议阶段"用路线图真实阶段（P2/P3/P4），**P3（Gate）/P4（Bundle）属 MVP 主路径阶段，不压缩进 Post-GA**；管理员简报口径"P1/P2/Post-GA/Growth"按下表归桶即可——P2 及更早→P1/P2 桶；P3/P4→P2 桶（MVP 内后段）；V1 之后凭信号→Post-GA；Growth 锚点→Growth。

---

## 1. 评估框架与口径

- **阶段轴**：沿用 FS-DEL-001 路线图为唯一阶段轴（P0→P1→P2→P3→P4→V1→P5→B1→RC1→GA→Growth），与二轮简报一致。
- **成本分级**：S ≈ ≤1 人周（含 fixture、测试、golden、文档）；M ≈ 1–4 人周；L ≈ >4 人周或引入新子系统。
- **插槽类型**：A/B/C/D 四类（见 §0 表注），沿用《FS-端口扩展与深度体验-技术可行性简报》§1.3。
- **治理红线**：本文全部结论为探索性质；Active Task = NONE 不变；任何一条落地前须逐项过 Scope Creep 四问（FS-PRD-004）+ 对应 ADR/依赖准入（FS-TECH-012）。

---

## 2. 候选一：组件版本漂移检测（S-05/H8）——本轮性价比之王

### 2.1 架构匹配度：高（B 类：插槽内新功能）

- **数据面已存在**：`ComponentEvidence` 是 02_DOMAIN_MODEL 的一等实体（component_name / version / state / evidence_refs[] / confidence / identifiers），产品层 P1 已排 Component Evidence + CycloneDX SBOM——**漂移检测的数据前提就是基线已排期项，不需要新采集管道**。
- **对比面有先例**：`Diff` 实体已含 `identity_changes` 族（同为跨快照对比一组实体字段），缺的只是 `component_changes` 族。diff 引擎只操作 normalized snapshot、从不重解析原始文件（领域不变量 2）——新对比族是纯 Core 增量，不碰解析层。
- **门禁面现成**："未声明的组件版本升级 → REVIEW"是 deterministic 规则，落 `GatePolicy.rules[]` + thresholds，符合"无 AI 红线"（§3.1-7）。
- **report/schemas 面**：diff 导出加一族 changes 属 additive 演进；发布报告渲染走 report crate 既有 ReportRenderer 端口，零新出口。

### 2.2 与 P2 已排期构建级 diff 的边界（必须写进 feature spec）

P2 已排的 symbol/section diff 回答"**固件本身**变了什么"；H8 回答"**组件版本**漂移了什么、是否被批准"。二者是同一 diff 界面下的并列两族，不重复、不混叠——这正是《边界清单》§4.3 对 H8"须与 P2 严格区分"的技术兑现方式。

### 2.3 数据来源与依赖可得性

| 来源 | 状态 | 证据等级 | 解析成本 |
|---|---|---|---|
| ComponentEvidence（产品层 P1 产出） | 已排期 | Observed/Derived/Declared 混合 | 0（数据已在） |
| ESP-IDF v5 `dependencies.lock` | 官方已引入（外部信号一手） | Declared | S——走 artifact adapter 槽位；结构化文本单文件小体量，无论 YAML/TOML 均为 S 级解析（toml 在栈；若为 YAML 需引入 serde_yaml，S 级依赖准入项） |
| Conan lockfile（JSON） | 生态常见 | Declared | S（serde 已冻结） |
| 无 lockfile 项目（多数） | — | — | 漂移检测仍可跑：基于二进制探测 + 手动声明，version unknown 显式呈现（R3 预期管理已有设计） |

**依赖结论：零～近零新外部依赖。** 建议首批 lockfile 范围收敛到 ESP-IDF 单源，避免重蹈"大正则通吃"覆辙（FS-TECH-004 纪律）。

### 2.4 成本拆解：M

- Core：diff 引擎新增 component_changes 族 + 排序/过滤（主体工作量）；
- 存储/DTO/UI：加列、加视图（S）；
- 门禁规则 1 条 + policy 字段（S，可随 P3）；
- golden + 导出 schema 增量 + 能力报告措辞（S）。

### 2.5 定级与阶段

**B 类；M；P2 落地（对比族），门禁规则随 P3。** 不建议提前到 P1：数据前提（Component Evidence）本身在产品层 P1，先行无米下锅。

---

## 3. 候选二：栈安全证据 .su（S-01）——维持 C 类判定，但升格讨论已被外部信号激活

### 3.1 与二轮简报结论的关系

二轮 §2.2 已判：解析 S、整体 M、**C 类（域模型扩展）**。本轮外部信号（puncover#87/#105 + Zephyr#27189 三源同构、高互动）**不改变技术成本，只改变需求强度置信度**——把"要不要进访谈清单"升级为"值得综合环节裁决升格排期"。技术结论逐字维持，以下补充升格后的落地要点。

### 3.2 架构匹配度：高（C 类：须 ADR）

- **入口插槽现成**：.su 是 GCC `-fstack-usage` 的伴生文本产物，artifact adapter 按 `detect/parse` 范式新增即可（同 MapAdapter 模式，FS-TECH-017），管线本体零改动。
- **但它是新证据维度**：per-function stack usage 要进 `symbols[]`（新字段 + 证据引用）、存储 schema（编号迁移机制现成）、diff（栈增量列）、UI（列 + 排序 + 超限标红）。跨域模型/存储/diff/UI 四层，故维持 C 类——ADR 的实质是"域模型扩展"，不是解析器。
- **证据等级**：Declared（依赖编译开关，非二进制观测）——显式标注，不冒充 Observed（领域不变量 4），证据分级前缀 `[DEC]` 现成。

### 3.3 数据来源可得性

- 标准编译开关（GCC/Clang），Zephyr/ESP-IDF 可启用，**非默认全开 → 天然覆盖缺口**——用既有 capability banner 模式呈现（"stack 证据未提供"），与"MAP 未提供"完全同构，设计侧零新概念。
- 纯文本行解析，零新依赖；fixture 易造（本地 GCC 产出即可），符合 fixture-driven expansion 机制。

### 3.4 范围纪律（建议写进 ADR 的边界）

- 本候选范围 = "逐函数最坏栈列 + 排序 + 超限对照 policy 阈值标红"。与 Zephyr#27189 的"报告 + 超限 flag"诉求同构，天然 deterministic，落 `GatePolicy.thresholds`（用户声明线程栈配额为 Declared 输入）。
- puncover#105 的"函数指针调用链断链 / 未知被调者标注"是**另一个需求（L 级，静态调用图重建）**，不并入本次升格裁决——防止一个 ADR 背两个功能。

### 3.5 定级与阶段

**C 类（须 ADR：域模型扩展）；解析 S / 整体 M；若升格 → P2（P1 symbols/evidence 稳定后）。** V1 访谈问题保留，但口径从"验证要不要"改为"验证优先级与场景强度"。

---

## 4. 候选三：CRA 窗口的 SBOM completeness / evidence retention（S-04，H3/H4）——八成已被基线覆盖，增量是"定义"而非"代码"

### 4.1 H3：SBOM evidence completeness model

- **基线原文钥匙（FS-COMP-002 §5，2026-09-27 复核）**：产品未来可以给 `SBOM evidence coverage`，"但不能给虚假的 100% complete，除非定义并验证 completeness model"——**基线把门槛放在'模型定义与验证'上，不是放在代码上**。
- 技术形态：coverage 报告 = ComponentEvidence（state、version 有无、证据源五级）之上的派生统计，由 report crate 渲染——数据模型已排期（产品层 P1），实现是口径与渲染问题，代码本身 S。
- 成本大头：completeness 模型的**定义与验证**（口径文档 + fixture 语料验证"模型不自欺"），M。
- **定级：B 类；M；P2**（产品层 P1 数据落地后）。CRA 时间窗是叙事不是排期驱动。

### 4.2 H4：evidence retention

- 基线原文（FS-COMP-003）："可以支持 evidence retention"，无排期锚（《边界清单》§4.3 H4）。
- **技术盘点：retention 的约八成已被已排期项覆盖**——P1 多 build 历史（SQLite 不可变快照）+ P4 Release Bundle（自包含、SHA256SUMS、脱离 FirmwareSight 可独立阅读）+ CLI 确定性 JSON 输出。
- 真实增量只有三件：①retention 口径定义（留什么、留多久、"可交审计的归档"= Bundle + analysis JSON 组合的一页说明）；②config 增 `[retention]` 段（数据端口 schema_version 演进插槽现成，FS-TECH-009）；③Bundle manifest 明示 evidence 文件集（schema `additionalProperties: true` 已预留）。
- **定级：A/B 类；S–M；P4 增量为主，不独立立项。**

### 4.3 边界提醒

- **CRA 是叙事不是排期驱动**：FS-COMP-003 Design consequence 明文"CRA 不是核心产品存在的唯一理由"——completeness/retention 的工程价值独立成立（交接、审计、回归基准），措辞不靠法规续命。
- **R4 红线**：只出工程证据，不出"合规/完备"结论；措辞按 07_COMPLIANCE 白名单（Evidence available/missing、Review required）。

---

## 5. 候选四：发布物符号档案（S-06）——零新解析器的 A 类项

### 5.1 架构匹配度：极高（A 类：插槽现成）

- `ReleaseBundle` 域实体已含 **`selected_artifacts`**——"把哪些产物装进 bundle"本来就是设计内决策，符号档案只是它的一个具体选法。
- release-manifest.schema.json 的 `artifacts[]` 已要求 path/sha256/size 且 `additionalProperties: true`——加 role/kind 标注（如 `role: "symbols"`）是**同 major additive 演进**，schema_version=1 不动。
- "符号文件"就是用户已导入分析的 ELF 本体（+ 可选 MAP）：**零新解析器、零新依赖**。nRF Cloud/Memfault"版本↔符号文件"惯例的本地形态 = bundle 带上 ELF + hash 映射，与 SHA256SUMS/release-manifest 天然衔接，不涉任何云上传（不触 ADR-0004）。

### 5.2 成本拆解：S

- bundle 组装策略（显式勾选 ELF/MAP；默认**不**整目录吸入）+ 体积提示；
- manifest role 字段 + SHA256SUMS 联动 + golden；
- 文档一页（"crash post-mortem 时按版本找回 ELF/hash"场景说明）。

### 5.3 定级与阶段

**A 类；S；P4。** V1 访谈若出现 crash post-mortem / 云端符号上传场景，可前置预览；未来若与云平台对接，本档案即"上传物"——但对接本身属 ADR-0004 域，不在本候选内。

---

## 6. 候选五：组件 ABI 兼容性门禁（S-07）——可行但是 L 级子系统，建议缓行

### 6.1 架构匹配度：中（C 类）

- **依赖面好消息**：gimli 已在冻结栈内（feature-scoped，ADR-0010"仅 DWARF 证据"）——**不需要新依赖 ADR**。
- **功能面真成本**：DWARF 深度类型消费（struct 布局 / 函数签名 / 调用约定）是自研子系统——GCC/Clang/ArmClang 的 DWARF v4/v5 方差、C++ vtable 与名称修饰、模板实例化。libabigail（abidiff）的体量是该复杂度的直接证据。且类型布局/签名属**新证据维度** → 域模型 + diff + gate + 报告全链扩展，与 .su 同理须 ADR，但工作量高一个量级。
- **与 P2 symbol diff 的关系**：symtab 级 diff 能抓 added/removed/changed（按名/大小），抓不到"同名函数签名变了 / 结构体布局变了"——这正是 ABI 门禁的边际价值，也是它必须吃 DWARF 的原因。

### 6.2 两步走路径（给综合环节的降险选项）

1. **第一步（可搭车 P2/P3，M）**：DWARF 辅助的符号变更明细——用既有 gimli 管道给 `symbol_changes` 补"签名变化"标注，不建完整 ABI 判定；
2. **第二步（独立立项，L）**：结构体布局 diff + "公共 ABI 破坏未声明 → REVIEW/BLOCK" 门禁规则，须 ADR + 显式 revisit trigger。

### 6.3 定级与阶段

**C 类；M（第一步）/ L（第二步）；第二步 Post-GA，触发条件 = V1/B1 出现组件升级 ABI 事故信号 + P2 diff 基建已落地。** cargo-semver-checks 证明该品类成立（"single most valuable check"），但"证明成立"≠"现在做"——它排在所有插槽内事项之后。

---

## 7. 次级候选速评（信号报告已激活的搭车项）

| 候选 | 判定 | 类型 | 成本 | 阶段 |
|---|---|---|---|---|
| S-03/H9 license 字段透传 + 风险提示 | CycloneDX licenses 字段透传（数据来自 manifest/Declared；二进制侧 license 天然不可观测，预期管理 SBOM 策略已埋）；deterministic 规则"用户配置 license 黑名单命中 → REVIEW"；守 R4：提示≠法律结论 | B | S–M | 搭 P1 SBOM / P2 |
| S-08 工具链 flag 指纹 | Identity 已含 compiler/linker/target/build_id（域模型实锤）——增量仅 DWARF producer / `-grecord-gcc-switches` 的 flag 摘要作 Observed 增列 | A | S | 随 P1/P2 |
| S-12 解析适配需求池 | 非新功能——R2 fixture-driven expansion 既有机制承接（overlay/debuglink 等按 fixture 准入） | — | — | 既有排期域 |

---

## 8. 不值得做项及理由

| # | 项 | 理由 | 依据 |
|---|---|---|---|
| 1 | **S-10 changelog/release notes 生成升格** | 外部信号弱（GitHub 平台内置已吞掉通用需求）；嵌入式差异化仅在"从构建元数据**汇编**发布说明"，那是 C1/C4 落地后 report crate 的 S 级副产品，不是独立立项理由。**维持基线"如有"口径**——与外部报告"已建议不升格"结论一致 | 边界清单 §4.3 H1 + 信号报告 S-10 |
| 2 | **S-11 二进制/分区级 diff** | 紧邻 Non-goals #5（逆向套件），一步即触禁区；符号级 diff（P2 已排）已覆盖发布场景的决策需求，字节级增量有限；防产品叙事滑向 binwalk 替代品 | FS-PRD-004 §3.1-5 + 信号报告 §6 风险 3 |
| 3 | **S-09 OTA 发布物元数据** | OTA 在推迟清单与禁跳清单两处禁区；极窄候选须过 Scope Creep 四问，默认不做。若 V1 访谈 OTA 团队频现，作 P4 可选字段再议（MCUboot header 字段清单是现成参考） | §3.8 + 信号报告 S-09 |
| 4 | **全量"构建环境复现指纹"（S-08 完整形态）** | 容器/环境哈希超出本地可观测范围，只能 Declared 手工喂——与二轮 build log 同判例：维护无底洞、增益低。§7 的 S 级 flag 摘要已拿走值得拿的部分 | 二轮简报 §2.2 build log 判例 |
| 5 | **.su 升格时并入"调用图/未知被调者"** | 静态调用图重建是 L 级独立需求，与 .su 列展示是两个 ADR 的事；并入会让升格裁决背上不必要重量 | 本文 §3.4 |
| 6 | **AI 参与任何判定/生成** | ADR-0005 可信核心禁入 AI；本轮信号无任何支持重开的新证据 | ADR-0005 |

---

## 9. 汇总与给综合环节的排序建议

```text
第一梯队（插槽内直接做，随既有阶段走）：
  C1 组件漂移        P2   B/M   数据前提已排期，只差对比视图 + 一条门禁规则
  C4 符号档案        P4   A/S   selected_artifacts 插槽现成，零解析成本
  C3b retention 增量  P4   A–B/S–M  八成已被 P1 History + P4 Bundle 覆盖
  §7 flag 指纹       P1–P2 A/S   Identity 增列，随批搭车

第二梯队（B 类 M 级，P2 批次）：
  C3a completeness   P2   B/M   难在模型定义与验证，不在代码

第三梯队（须 ADR，凭信号/裁决）：
  C2 .su 栈证据      P2   C/M   升格裁决留给综合环节；范围纪律见 §3.4
  C5 ABI 门禁第二步   Post-GA C/L 两步走；第二步凭 V1/B1 事故信号

不做：§8 全表。
```

**须拍板的三个问题（各附建议）**：

1. **.su 是否升格**——建议：升格进 P2 候选池，ADR 范围限定为 §3.4（列 + 排序 + 超限标红），调用图明确排除；
2. **lockfile 首批范围**——建议：ESP-IDF `dependencies.lock` 单源起步，其余 Declared 手动声明兜底；
3. **符号档案默认内容策略**——建议：显式勾选（不整目录吸入）+ manifest `role` 字段 + 体积提示。

---

## 10. 引用与治理声明

- **基线引用（2026-09-27 原文复核）**：04_TECH/02_DOMAIN_MODEL（`Diff.identity_changes`、`ComponentEvidence`、`ReleaseBundle.selected_artifacts`）、04_TECH/17_RELEASE_PACKAGING_UPDATE、schemas/release-manifest.schema.json（artifacts 必填三项 + additionalProperties:true）、07_COMPLIANCE/01_SBOM_STRATEGY §5、07_COMPLIANCE/02_CRA_CONTEXT、examples/Cargo.workspace.baseline.toml；FS-TECH-004/008/009/012/017 与 ADR-0004/0005/0006/0007/0010/0013（承二轮简报引用口径，未复读）。
- **群内输入**：信息哨兵《FS-真需求全景扫描-外部信号报告》（信号强度/需求真伪口径）；通用助手《背景回顾-已覆盖面与本轮扫描边界清单》（边界标注与悬置线索出处）；鲁班七号《FS-端口扩展与深度体验-技术可行性简报》（A–D 分类法与 .su 判定出处）。
- **口径分工**：信号强度与需求真伪属信息哨兵调研口径，本文只做技术成本/插槽/阶段判断；两口径冲突时以基线原文为准。
- 本简报为 Post-MVP 探索稿：未创建 Active Task、未修改基线、未授权任何实现；所有"建议阶段"均须立项时逐项过 Scope Creep 四问 + 对应 ADR/依赖准入。

---

*整理：鲁班七号 · 2026-09-27 · 供第三轮综合成文使用；本文不修改任何基线文档。*
