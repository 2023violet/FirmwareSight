---
title: "FirmwareSight 正式产品功能探索：真需求功能全景扫描（Post-MVP）"
product: "FirmwareSight"
version: "1.0"
date: "2026-09-27"
authored_by: "专业写手（综合执笔）"
status: "FINAL — 综合交付稿 · 本轮全部结论不纳入 MVP，不改变 Active Task 状态"
inputs:
  - "通用助手《背景回顾-已覆盖面与本轮扫描边界清单-通用助手.md》（四类标注 + 30+ 条悬置线索 H1–H10 + 排除口径）"
  - "信息哨兵《FS-真需求全景扫描-外部信号报告-信息哨兵.md》（12 条信号 S-01–S-12 + 强/中/弱分级 + 来源 + 四动词相容性初判）"
  - "鲁班七号《FS-真需求全景扫描-技术可行性简报-鲁班七号.md》（五候选逐一评估 + A–D 插槽分类 + 三梯队排序 + 不值得做 6 条）"
  - "设计匠人《FS-真需求全景扫描-设计视角备忘-设计匠人.md》（12 条候选零新增界面 + 逐条采纳/改造/排除 + 变更呈现三纪律）"
citation_key: "本文引用标记：【①】=通用助手边界清单，【②】=信息哨兵信号报告，【③】=鲁班七号简报，【④】=设计匠人备忘；基线编号（FS-TECH-xxx/ADR-xxxx/FS-COMP-xxx 等）以四份输入所引原文为准。"
---

# FirmwareSight 正式产品功能探索：真需求功能全景扫描（Post-MVP）

> **阅读前提**：本议题由用户定位为 Post-MVP 探索。本文不改变治理状态——Active Task = NONE 不变，当前唯一合法下一步仍是 P0 Technical Vertical Slice。本文回答的是"原规划之外还有哪些有真实需求信号的功能、证据有多强、成本多大、明确不做什么"，不是"现在做"。
>
> **方法声明**：结论综合自四份输入（上方 frontmatter），每条标注来源与证据强度；Reddit 社区证据均为摘要级（信息哨兵环境网络受限，已逐条标注"待证实"）。遵循语调三律：说事实不做判决、说未知不填空（无信号如实标注，缺失不等于不存在）、给下一步。

---

## 1. 议题定义与结论先行

### 1.1 议题原文与口径澄清

用户提出（2026-09-27）：

> "还有什么是真需求的功能吗？比如我们之前讨论的可视化，导出等都是原来项目规划中没有的，我们想要探索与讨论的是这种真需求功能（还是说上一轮的《FirmwareSight 正式产品功能探索》就是已经给出答案了）"

**先回答括号里的问题**：没有。前两轮都是"命题作文"——第一轮只回答用户提出的可视化+文档导出两个功能，第二轮回答端口扩展+深度体验；"原规划之外还有哪些真需求"这个全景问题至今未被系统回答过，本轮即该问题的答案。

**扫描口径**（【①】四类标注）：已覆盖（可视化/导出/端口/体验深化——撞见同类信号只作佐证，不作新发现）、已排期（P0→Growth 有锚项）、禁区（Non-goals/章程/禁跳清单，一律不立项）、本轮候选（30+ 条基线悬置线索）。扫描的合法产出形态 = 新方向信号 + 候选强度验证 + 已排期/已覆盖主题的佐证。

### 1.2 结论先行：三层判定

| 层 | 判定 | 条目 | 一句话依据 |
|---|---|---|---|
| **外部信号强的真需求** | 确认 | **栈安全证据**（S-01，强） | puncover 两个最热 issue + Zephyr 官方 issue 三源同构、高互动【②】§2；注意其性质是第二轮 `.su` 议题（P-10）的"佐证升格提请"，不是全新方向 |
| （强信号·佐证层） | 佐证 | CRA→SBOM（S-02，强） | 三个独立社区求助 + 法规时间线（2026-09 报告义务已启动），佐证基线 SBOM P1 排期并激活 H3/H4/H9【②】§2；属"已排期方向的佐证"，不计入规划外新发现 |
| **有信号待验证** | 候选 | **组件版本漂移检测**（S-05/H8，中） | lockfile 已自发迁入嵌入式构建（ESP-IDF v5 官方引入），缺的是消费端；数据前提就是已排期项，性价比本轮最高【②】§3；【③】§2 |
| | 候选 | **SBOM 证据完备性 / evidence retention**（S-04/H3/H4，中） | "auditor 要求 build-time manifest"直指证据链可信度；CRA 窗口激活【②】§3；约八成已被 P1 History+P4 Bundle 覆盖，增量是口径定义【③】§4 |
| | 候选 | **发布物符号档案**（S-06，中·新方向） | nRF Cloud/Memfault"版本↔符号文件"行业惯例的本地空位；零新解析器【②】§3；【③】§5 |
| | 候选 | **组件 ABI 兼容性门禁**（S-07，中·新方向） | cargo-semver-checks 证明该品类可成为核心门禁；嵌入式侧工具存在但未接入工作流；L 级子系统建议缓行【②】§3；【③】§6 |
| | 搭车 | license 提示（S-03）、工具链 flag 指纹（S-08）（中） | 分别搭 SBOM P1 便车与 Identity 增列，不独立立项【③】§7 |
| **伪需求 / 不升格** | 排除 | changelog 生成（S-10）、OTA 元数据（S-09）、二进制/分区级 diff（S-11）、全量复现指纹、调用图并入 `.su`、AI 判定 | 共 6 条，理由与复核条件见 §6【③】§8 |

双排除方向如实记录：团队 policy 共享、多构建趋势追踪经定向检索无新信号，维持排除【②】§4。

### 1.3 总判断：本轮真需求收敛于一个方向

直接回答用户的问题——**除了可视化/导出/端口之外，本轮扫描出的规划外真需求高度收敛于"证据的完整性与可追溯性"**：栈安全、组件版本漂移、发布物符号档案、SBOM 证据完备性、构建指纹。它们不是更多输出格式，也不是更多端口；信息哨兵（§6"全部可以塞回四动词链内，无需越过任何一条 ADR"）与设计侧（§3.1"零新增界面、零新增图型"）从两个视角独立印证了同一结论。

三个结构性事实：

1. **零新增界面**。12 条候选的正确形态全部是四动词界面里的新列/新节/新清单条目 + 能力 banner 扩位——"没有接线问题，只有读数问题"（【④】§3.1）。这是本轮与前两轮最大的形态差异：端口轮解决接线，本轮只补读数。
2. **无一候选触碰架构红线**。不需要新 crate、新依赖 ADR，无一触碰 Non-goals（【③】§0）。分水岭只有两个：`.su` 是域模型扩展（C 类，须 ADR），ABI 是子系统级（L 级）。
3. **真正的全新方向只有两个**：发布物符号档案（S-06）与 ABI 门禁（S-07）（【②】§0）。其余候选要么是基线悬置线索的外部激活（H3/H4/H8/H9），要么是已排期项的佐证（S-01 对二轮 P-10、S-02 对 SBOM P1）。

置信度声明：强/中/弱分级沿【②】口径；Reddit 证据均为摘要级、待证实（【②】§1）；"无信号≠伪需求"继续适用——信号弱的条目进 §5 脑洞区登记，只有证据反向（需求被平台吞掉/撞禁区）的条目才进 §6。

---

## 2. 背景回顾摘要（引用，不重提）

完整四类标注与 30+ 条悬置线索见：通用助手《背景回顾-已覆盖面与本轮扫描边界清单-通用助手.md》（shared/firmwaresight/）。与本文结论直接相关的锚点：

| 主题 | 既有事实 | 性质 |
|---|---|---|
| 边界口径 | 四类标注（已覆盖/已排期/禁区/本轮候选）+ 管理员提示方向×边界标注预判表——本轮 12 条信号的边界标注全部沿此执行 | 方法约束【①】标注规则/§5 |
| 已覆盖面 | 前两轮结论级排除：可视化 V0–V7 阶梯、导出格式结论（JSON/CSV 强、HTML 中强、Excel/docx 挂复核条件、原生 PDF 不做）、端口 A–D 分类、W1–W5、B1–B6 | 排除依据【①】§1 |
| 已排期面 | P0→Growth 全阶段排期（判据"排期有锚"）：SBOM=P1、多 build 历史=P1、构建级 diff=P2、Gate=P3、Bundle=P4、CI=Growth、Team Policy=Growth | 排除依据【①】§2 |
| 禁区面 | Non-goals 8 项 + 章程非目标 + MVP 推迟 15 项 + 禁跳清单 10 项 + ADR 红线（0004 local-first / 0005 无 AI / 0006 MVP 格式 / 0013 network deferred / 0014 GPU deferred）；易踩线预判：OTA 元数据、"AI 写 release notes"、云协作 | 排除依据【①】§3 |
| 悬置线索 | H1–H10（基线有字眼无锚：changelog、SPDX 3.0、completeness、retention、support period、ARM64、updater 时机、组件漂移、license 提示、benchmark）+ Deferred 无锚项 + 前两轮脑洞 B1–B6/X1–X5 | 本轮验证对象【①】§4 |
| 悬置执行件 | 合并 V1 访谈清单（xlsx/docx + `.su` + CI 需求强度）已约定未建；前两轮 17 个待拍板问题仍悬置——本轮结论与之交叉引用、不重复论证 | 流程约束【①】§1.3/§4.5 |

---

## 3. 真需求功能清单

### 3.1 重点候选（C1–C5）

综合技术评估（【③】§0/§2–§6）、外部信号（【②】§0/§2–§4）、设计落位（【④】§1）。插槽类型沿【③】分类法：A=已预留插槽 / B=插槽内新功能（过 Scope Creep 四问）/ C=须 ADR / D=不做。

| # | 候选 | 需求信号 | 使用者价值（回答什么问题） | 技术成本 | 类型 | 设计相容性 | 建议阶段 | 与前三轮的去重关系 |
|---|---|---|---|---|---|---|---|---|
| C1 | **组件版本漂移检测**（S-05/H8） | 中：ESP-IDF v5 官方引入 dependencies.lock；Renovate 宣称覆盖 C/C++ 但不贴合本地工作流；消费端缺位【②】§3 | "两次发布之间，组件版本漂移了什么、是否被批准"——Lead/QA 的复核证据，防"没人记录的升级" | **M**（diff 引擎加 component_changes 族为主体；lockfile 解析 S） | **B** | ✅ Compare 新增 Components 节 + Gate 一条规则；零新界面【④】§1.2 | **P2**（门禁规则随 P3） | 数据前提=产品层 P1 Component Evidence（已排期，不重复立项）；与 P2 已排期构建级 diff 同界面并列两族、严格区分 |
| C2 | **栈安全证据 `.su`**（S-01） | **强（佐证升格提请）**：puncover#87/#105（另有 #27/#99 同向）+ Zephyr#27189 三源同构；puncover 维护真空【②】§2 | "这个线程的栈配额够不够、栈被谁吃了"——栈溢出是固件最隐蔽故障类；门禁可拦超限 | 解析 **S** / 整体 **M**（跨域模型/存储/diff/UI 四层） | **C（须 ADR）** | ✅ Symbols 表增 worst stack 列 + 缩进调用路径（断链=显式 Unknown）+ banner 扩位；禁全量调用图【④】§1.1 | 升格则 **P2**（P1 symbols/evidence 稳定后） | 与二轮 P-10 同源——本轮为外部佐证激活的升格提请，非新发现；V1 访谈口径从"要不要"改为"优先级与场景强度" |
| C3a | **SBOM evidence completeness**（S-04/H3） | 中："auditor is a stickler for build-time manifests"；CRA 2027-12 全面适用倒逼【②】§3 | "这份 SBOM 的证据可信吗、缺口在哪"——五级证据分级计数，可复核、不伪装精确 | **M**（大头在 completeness 模型定义与验证，实现 S） | **B** | ✅ 证据分级计数行；禁 coverage 分数/donut/进度环【④】§1.3 | **P2**（产品层 P1 数据落地后） | SBOM 本体 P1 已排期（本轮不重复论证）；H3 为基线悬置（FS-COMP-002 §5"未来可以给"）的外部激活 |
| C3b | **evidence retention**（S-04/H4） | 中：社区痛点在"生成的 SBOM 可信吗、能存多久"【②】§2/§3 | "这个 build 当时的证据还在吗、归档里带了什么"——审计/交接/回归基准 | **S–M**（口径定义 + config 段 + manifest 明示） | **A/B** | ✅ History 行内 evidence-integrity 复验 + Bundle 归档 + Settings 单行；不新增 retention 管理界面【④】§1.3 | **P4 增量**，不独立立项 | 约八成已被 P1 History（SQLite 不可变快照）+ P4 Bundle（自包含+SHA256SUMS）覆盖——增量是"口径"不是"代码" |
| C4 | **发布物符号档案**（S-06） | 中（新方向）：nRF Cloud/Memfault"版本↔符号文件"为运维必需品；本地优先侧无对应物【②】§3 | "v1.2.3 发布时的 ELF 在哪、hash 是多少"——crash post-mortem 的前提 | **S**（零新解析器、零新依赖） | **A** | ✅ Bundle 清单条目 + manifest role 字段；不做符号管理页【④】§1.4 | **P4**（挂 V1 访谈验证 crash post-mortem 场景） | Bundle P4 已排期——符号档案只是 `selected_artifacts` 槽位的一个具体选法；与二轮 X4 相邻不同（X4=交付通道侧，本条=归档内容侧） |
| C5 | **组件 ABI 兼容性门禁**（S-07） | 中（新方向）：cargo-semver-checks 被评"single most valuable check"；libabigail 存在但未接入工作流【②】§3 | "组件升级是否悄悄破坏了 ABI"——把链接期/运行期才暴露的事故前移到发布前 | 两步走：第一步 **M**（DWARF 辅助符号变更明细）/ 第二步 **L**（布局 diff+门禁，子系统） | **C（须 ADR）** | ✅ 类型/签名变更字段表 + Gate 规则；禁内存布局画图、禁 breaking 彩徽章、禁兼容性评分【④】§1.5 | 第一步可搭 P2/P3；第二步 **Post-GA 凭 V1/B1 事故信号** | 与 P2 symbol diff 严格区分：symtab 级抓不到"同名函数签名变了/结构体布局变了"——这正是 ABI 门禁的边际价值 |

### 3.2 次级搭车项与佐证项

| # | 项 | 判定 | 成本 | 阶段 | 去重关系 |
|---|---|---|---|---|---|
| S-03/H9 | **license 字段透传 + 风险提示** | 搭 SBOM P1 便车：CycloneDX licenses 字段透传 + "用户配置 license 黑名单命中 → REVIEW" deterministic 规则；守 R4（提示≠法律结论，禁 risk 红黄绿灯） | S–M | 搭 P1 SBOM / P2 | SBOM P1 已排期，license 是其自然扩列【③】§7；【④】§2 |
| S-08 | **工具链 flag 指纹** | Identity 已含 compiler/linker/target/build_id（域模型实锤）——增量仅 DWARF producer / `-grecord-gcc-switches` flag 摘要作 Observed 增列；全量"复现指纹"不做（§6-4） | S | 随 P1/P2 | 二轮 P-8 非 Git provenance 的外部佐证与具体扩展点【③】§7；【②】§3 |
| S-02 | CRA→SBOM | **P1 排期佐证**，非新功能：CRA 时间窗使本地 SBOM 生成从 nice-to-have 变为 deadline 驱动采购；V1 访谈与 Beta 定价宜挂靠此窗（§8-Q9） | — | P1（既有排期） | 已排期方向的佐证【②】§2 |
| S-12 | 解析适配需求池 | 非新功能——R2 fixture-driven expansion 既有机制承接（overlay/debuglink 等按 fixture 准入） | — | 既有排期域 | R2 佐证池【②】§4；【③】§7 |

### 3.3 梯队汇总（沿【③】§9，候选≠承诺）

```text
第一梯队（插槽内，随既有阶段走）：
  C1 组件漂移          P2    B/M      数据前提已排期，只差对比视图 + 一条门禁规则
  C4 符号档案          P4    A/S      selected_artifacts 插槽现成，零解析成本
  C3b retention 增量    P4    A–B/S–M  八成已被 P1 History + P4 Bundle 覆盖
  S-08 flag 指纹       P1–P2 A/S      Identity 增列，随批搭车

第二梯队（B 类 M 级，P2 批次）：
  C3a completeness     P2    B/M      难在模型定义与验证，不在代码

第三梯队（须 ADR，凭信号/裁决）：
  C2 .su 栈证据        P2    C/M      升格裁决待 §8-Q1；范围纪律见 §4.2
  C5 ABI 门禁第二步     Post-GA C/L    两步走；第二步凭 V1/B1 事故信号

不做：§6 全表。
```

【③】§0 的取舍建议原样保留：**综合环节若只带一件走，带 C1 组件漂移；带两件，加 C4 符号档案。**

---

## 4. 五项重点候选深度评审

### 4.1 C1 组件版本漂移检测——本轮性价比之王

- **需求信号与置信度**（【②】§3，中）：ESP-IDF v5 component manager 官方引入 `dependencies.lock`——lockfile 模式已自发迁入嵌入式构建侧；Renovate 宣称覆盖 C/C++ 但社区认知不贴合本地优先固件工作流。缺的是**消费端**：没有工具回答"两次发布间组件版本漂移了什么、是否被批准"。（另有 r/embedded 版本联动讨论，摘要级/待证实。）
- **使用者价值**：对比视图回答"mbedTLS 从 3.4.0 变 3.6.1 且没人记录了吗"；门禁规则把"未经声明的版本变更"拦在发布前。直接服务 Persona B（可复核 diff）与 Persona C（结构化证据）。
- **技术评估**（【③】§2，B 类/M）：`ComponentEvidence` 是域模型一等实体且产品层 P1 已排期——**数据前提就是已排期项，不需要新采集管道**；`Diff` 已有 `identity_changes` 先例，缺的只是 `component_changes` 对比族（diff 引擎只操作 normalized snapshot、从不重解析原始文件，纯 Core 增量）；lockfile 解析 ESP-IDF 单源起步（S 级；若为 YAML 需引入 serde_yaml，S 级依赖准入项）；无 lockfile 项目仍可跑（基于二进制探测 + 手动声明，version unknown 显式呈现）。**零～近零新外部依赖。**
- **设计形态与防噪**（【④】§1.2）：Compare 新增 Components 节（component / build A version / build B version / drift kind / evidence grade）；版本变更行用 `diff.changed.bg` 蓝底，升级与降级同色同待遇（"版本变了"是事实，"变好变坏"不由表格说）；**证据等级漂移也是漂移**（`3.4.0 [OBS] → unknown ○ [UNK]` 配事实句——证据退化比版本升级更值得看见）；每个版本号带 `[OBS]/[DRV]/[DEC]/[UNK]` 分级前缀；默认只列变化组件、未变折叠为一行计数（`31 components unchanged`）。禁 Dependabot 彩色徽章阵列、禁"落后 N 个版本/过期 N 天"（后者需联网 advisory feed，撞 ADR-0013）、禁红绿箭头、不做依赖健康面板。
- **去重与边界**：与 P2 已排期构建级 diff 的边界必须写进 feature spec——P2 回答"固件本身变了什么"，本条回答"组件版本漂移了什么、是否被批准"，同一 diff 界面下并列两族，不重复、不混叠。
- **spec 级口径提醒（综合发现）**：门禁规则模板两份输入措辞略有差异（【③】"未声明的版本升级"、【④】"changed without declared note"）。建议按【④】"变更"口径（事实对称，升降级同待遇），阈值可配；立项时在 feature spec 中冻结。
- **建议阶段**：P2 落地（对比族），门禁规则随 P3。不建议提前到 P1——数据前提本身在产品层 P1，先行无米下锅（【③】§2.5）。

### 4.2 C2 栈安全证据 `.su`——唯一强信号，升格裁决待拍板

- **需求信号与置信度**（【②】§2，强）：三个独立来源同构——puncover#87（puncover 最热开放 issue，用户原声："I frequently use Puncover to find the worst case stack usage… exceptionally useful if I could see this information as a column in the symbol list"，来自 Palmsens 固件工程师）、puncover#105（Zephyr 开发者 + Prusa 用户：函数指针/回调使静态调用图断链，"tripped us a number of times"）、Zephyr#27189（官方仓库："Outliers that exceed the default privilege stack size should be flagged as issues"）。puncover 作者已声明"currently not spending any time on this project"——维护真空期是窗口。
- **性质判定**：栈用量在第二轮 P-10 已探讨、挂 V1 访谈。本轮证据强度（三个独立仓库高互动 issue + "线程定容"具体工程场景）显著超过普通访谈线索，按【①】§1.3"强冲突提请"条款作**佐证升格提请**——升格与否由用户裁决（§8-Q1），本文不改第二轮结论。
- **技术评估**（【③】§3，C 类/解析 S 整体 M）：`.su` 是 GCC `-fstack-usage` 的伴生文本产物，adapter 按 detect/parse 范式新增（同 MapAdapter 模式），管线本体零改动；但 per-function stack usage 要进 `symbols[]`（新字段）+ 存储 schema + diff（栈增量列）+ UI（列+排序+超限标红）——跨四层，故须 ADR，**ADR 的实质是"域模型扩展"，不是解析器**。证据等级 Declared（依赖编译开关，非二进制观测），不冒充 Observed（领域不变量 4）。
- **范围纪律（建议写进 ADR）**：本候选范围 = 逐函数最坏栈列 + 排序 + 超限对照 policy 阈值标红（与 Zephyr#27189"报告 + 超限 flag"诉求同构，天然 deterministic；线程栈配额为用户声明的 Declared 输入）。puncover#105 的"调用图断链/未知被调者标注"是另一个需求（L 级静态调用图重建），不并入——防止一个 ADR 背两个功能（【③】§3.4；§6-5）。
- **设计形态与防噪**（【④】§1.1）：Symbols 表增 worst stack 列（mono 右对齐、tabular-nums、可排序——puncover 的教训正是"点进单函数才可见"不成立）；每列带配额上下文（`worst stack 1,024 B · quota 2,048 B`；无配额时只报事实值，不发明"使用率 %"）；行展开缩进式调用路径，函数指针断链处一行 `○ callee unknown (indirect call via function pointer)`——不猜、不把断链装成连续；超限与否由 Gate 说话（落 Gate 四件套 Result/Why/Evidence/Next），禁整列红染；不做默认按栈排序（排序是用户动作）；`.su` 未提供时整列 `—` + UNKNOWN + 事实句（"su file not provided; stack bounds unknown"）+ Next（"re-run with -fstack-usage"），禁 0 冒充；能力 banner 扩 `.su ✓/○` 一位。禁全量调用图可视化（graph 视图不在图表优先级链内）。
- **建议阶段**：若升格 → P2（P1 symbols/evidence 稳定后）；V1 访谈问题保留，口径从"验证要不要"改为"验证优先级与场景强度"。

### 4.3 C3a/C3b SBOM evidence completeness 与 evidence retention——八成已被覆盖，增量是"定义"不是"代码"

- **需求信号与置信度**（【②】§2/§3，中；CRA 时间线为一手事实）："auditor is a stickler for build-time manifests"（r/embedded，摘要级）直指完备性与留存诉求；CRA 2026-09 报告义务已启动、2027-12 全面适用，市场监督期要求技术文档可追溯留存。
- **使用者价值**：H3 回答"这份 SBOM 的证据可信吗、缺口在哪"（五级证据源分级计数，可复核、不伪装精确）；H4 回答"这个 build 当时的证据还在吗、归档里带了什么"（审计/交接/回归基准）。
- **技术评估**（【③】§4）：H3——基线原文（FS-COMP-002 §5，2026-09-27 复核）把门槛放在"定义并验证 completeness model"，不是代码；coverage 报告是 ComponentEvidence（state、version 有无、证据源五级）之上的派生统计，由 report crate 渲染——实现 S、定义与验证 M。H4——**约八成已被已排期项覆盖**（P1 多 build 历史 SQLite 不可变快照 + P4 Release Bundle 自包含/SHA256SUMS/脱离 FS 可独立阅读 + CLI 确定性 JSON）；真实增量只有三件：retention 口径定义（留什么、留多久、"可交审计的归档"= Bundle + analysis JSON 组合的一页说明）、config 增 `[retention]` 段（schema_version 演进插槽现成）、Bundle manifest 明示 evidence 文件集（`additionalProperties: true` 已预留）。
- **设计形态与防噪**（【④】§1.3）：H3 = 证据分级计数行（`24 observed · 5 inferred (from version.h) · 2 declared · 1 unknown`），**禁 coverage 分数/donut/进度环**（"SBOM completeness 82%"就是健康分换皮）；不建议为各等级组件数上 ranked bars（五级证据源是分类不是排序，纯计数行更克制也更诚实）。H4 = History 行内 evidence-integrity 复验（对 Bundle hash，复验结果用既有五态或 meta 文字呈现）+ Bundle 清单本身 + Project Settings 单行（保留策略）；**不新增 retention 管理界面**。禁词：compliant / audit-ready / 合规评分（R4 红线；07_COMPLIANCE 白名单——Evidence available/missing、Review required）。Unknown 计数在摘要行与报告中永不淡化为脚注或折叠。
- **去重与边界**：SBOM 本体 P1 已排期（本轮不重复论证）；CRA 是叙事不是排期驱动（FS-COMP-003 明文"CRA 不是核心产品存在的唯一理由"）——completeness/retention 的工程价值独立成立（交接、审计、回归基准），措辞不靠法规续命。
- **建议阶段**：C3a = P2（B/M，产品层 P1 数据落地后）；C3b = P4 增量（A–B/S–M），不独立立项。

### 4.4 C4 发布物符号档案——零解析成本的新方向

- **需求信号与置信度**（【②】§3，中，一手二源）：nRF Cloud 官方文档明确要求按固件版本上传 Symbol files 以解码设备数据（2026-08 更新）；Memfault SDK 的 crash 事后解析依赖与固件版本精确对应的符号文件。两大固件云平台都把"版本↔符号文件"当运维必需品，**本地优先侧没有对应物**——团队事后想知道"v1.2.3 发布时的 ELF 在哪、hash 是多少"只能翻 CI 产物或个人机器。
- **使用者价值**：crash post-mortem 的前提条件；"Keep the evidence"支柱的直接延伸。
- **技术评估**（【③】§5，A 类/S）：`ReleaseBundle` 域实体已含 `selected_artifacts`——"把哪些产物装进 bundle"本来就是设计内决策，符号档案只是它的一个具体选法；"符号文件"就是用户已导入分析的 ELF 本体（+可选 MAP），**零新解析器、零新依赖**；release-manifest 的 `artifacts[]` 加 role/kind 标注（如 `role: "symbols"`）是同 major additive 演进，schema_version=1 不动。
- **设计形态与防噪**（【④】§1.4）：Bundle 清单增可选条目（`symbols: firmware.elf · SHA-256 3f2a…`，mono hash 惯例已有）+ manifest 增 version→hash 映射；**不做"符号档案管理页"**（History 已按版本组织一切，Bundle 已是归档物本身——版本↔文件映射独立成界面即冗余）；防"云符号库上传"形态诱惑——上传撞 ADR-0004/0013，本地归档形态才合法，界面不出现 Sync/Synced 类过程态词汇；Bundle"脱离 FirmwareSight 独立可读"的 P0 承诺不受影响。
- **去重与边界**：与二轮 X4（Bundle 下游交付 local-first 扩展）相邻不同——X4 是交付通道侧（归档目录/Git 工作区路径约定），本条是归档内容侧；二者可独立推进、互不阻塞。
- **建议阶段**：P4 并车项；挂 V1 访谈验证 crash post-mortem 场景是否真实存在（默认内容策略见 §8-Q3）。

### 4.5 C5 组件 ABI 兼容性门禁——方向真、成本高、缓行

- **需求信号与置信度**（【②】§3，中）：cargo-semver-checks 被 corrode.dev Rust 工具索引评价为 "The single most valuable check"（2026-09）并进入 Rust 官方项目目标（2025-11）——"API 破坏性变更检测作为发布门禁"是 Web/后端已验证的真需求模式；嵌入式 C 侧对应工具 libabigail（abidiff）存在但"工具存在、没接入工作流"（摘要级）。
- **使用者价值**：vendor HAL/中间件升级时，"结构体布局变了/函数签名变了导致 ABI 破坏"目前靠编译侥幸发现——链接期才暴露，运行期更糟。本候选把该类事故前移到发布前。
- **技术评估**（【③】§6，C 类，两步走）：好消息是 gimli 已在冻结栈内（feature-scoped，ADR-0010），**不需要新依赖 ADR**；真成本是 DWARF 深度类型消费（struct 布局/函数签名/调用约定；GCC/Clang/ArmClang 的 DWARF v4/v5 方差、C++ vtable 与名称修饰、模板实例化）——自研子系统，libabigail 的体量是该复杂度的直接证据；且类型布局/签名属新证据维度 → 域模型 + diff + gate + 报告全链扩展。两步走降险：第一步（M，可搭 P2/P3）= DWARF 辅助的符号变更明细，用既有 gimli 管道给 `symbol_changes` 补"签名变化"标注，不建完整 ABI 判定；第二步（L，独立立项）= 结构体布局 diff + "公共 ABI 破坏未声明 → REVIEW/BLOCK" 门禁规则，须 ADR + 显式 revisit trigger。
- **设计形态与防噪**（【④】§1.5）：类型/签名变更字段表（`type·symbol / A layout / B layout / change kind`；字段级 `field / offset A / offset B / size`，偏移量 mono 列）；变更分类用文字标签（removed / layout-changed / signature-changed），不用彩色徽章；证据分级随行（DWARF 实测 = Observed，头文件声明 = Declared）。禁内存布局画图（优先级链外新图型且黑白打印灾难）、禁 cargo-semver-checks 式 breaking/major/minor 彩色徽章阵列（"BREAKING"红徽章=预支判决，破坏性认定只属于 Gate 规则）、禁兼容性评分。
- **去重与边界**：与 P2 symbol diff 的关系=增量价值所在——symtab 级 diff 抓 added/removed/changed（按名/大小），抓不到"同名函数签名变了/结构体布局变了"；这正是 ABI 门禁必须吃 DWARF 的原因，也是它与 C1 共存于 Compare 的理由（一表两族、深度不同层）。
- **建议阶段**：第一步搭 P2/P3；第二步 Post-GA，触发条件 = V1/B1 出现组件升级 ABI 事故信号 + P2 diff 基建已落地。cargo-semver-checks 证明品类成立，但**"证明成立"≠"现在做"**——排在所有插槽内事项之后。

---

## 5. 开放脑洞区（纯探索性质）

> 本节所有条目仅为脑洞登记，不构成建议承诺。分两组：本轮新登记（基线悬置线索中信号弱/零者）与前两轮继承（状态不变，防丢失再登记）。任何一条未来推进前，须先获得需求信号，再过端口准入三问 + Scope Creep 四问 + ADR。

**5.1 本轮新登记（基线悬置线索中信号弱/零者）**

| # | 脑洞 | 来源 | 信号状态 | 现实约束 / 前置条件 |
|---|---|---|---|---|
| N1 | SPDX 3.0 导出（H2） | 【①】§4.3 | 零独立信号 | CycloneDX 1.7 已排 P1；三方输入附议"H3 优先于 H2"（§8-Q6）——SPDX 继续悬置 |
| N2 | support period 跟踪（H5） | 【①】§4.3 | 零信号（CRA 强化需求列表字眼） | 基线未排期；须先有 SBOM P1 落地与真实合规场景信号 |
| N3 | Windows ARM64 / Linux ARM64（H6） | 【①】§4.3 | 零信号 | FS-TECH-020 deferred future；平台扩张受分发与 QA 成本约束 |
| N4 | updater plugin 启用机制细节（H7） | 【①】§4.3 | 零信号 | 功能本身有 v1.0 锚，悬置仅机制细节 |
| N5 | 性能 benchmark（H10） | 【①】§4.3 | 零信号 | EPIC H 其余项已归 P5，benchmark 无阶段锚；何时做、对谁做未定义 |
| N6 | 本地 CVE 对照（离线 advisory 数据形态） | 【①】§4.1/§4.2 | 零独立信号 | vulnerability intelligence（联网 CVE feed）有 Growth 锚且本轮维持排除；"本地离线"形态是悬置变体，联网部分触 ADR-0013 触发条件，须专项评估 |
| N7 | account/cloud/team workspace 细化形态、plugin 极简形态 | 【①】§4.1 | 零信号 | 无排期锚点；plugin marketplace 为禁区，"极简插件形态"未经探讨；云端形态触 ADR-0004 |

**5.2 前两轮继承（状态不变）**

| # | 脑洞 | 来源轮次 | 状态 |
|---|---|---|---|
| B1 | 跨 build 趋势深化（图表库 ADR 触发条件） | 第一轮 | 悬置——V7 有阶梯位、无实施锚 |
| B3 | 报告订阅/定期快照 | 第一轮 | 悬置（本地自动化形态未探） |
| B5 | 趋势预算告警（事实句式） | 第一轮 | 悬置 |
| B6 | 报告模板自定义 | 第一轮 | 悬置（与 report crate 模板机制关系未探） |
| X1 | 命令面板 | 第二轮 | 悬置（与"一屏一主焦点"张力待设计评审） |
| X2 | Agent 端口（Claude Code plugin 形态） | 第二轮 | 悬置——ADR-0005 边界初判成立（Agent 是"使用产品的人"），信号待验证 |
| X3 | 决策协作流转（REVIEW 指派/闭环） | 第二轮 | 悬置——须过 ADR-0004 本地优先边界 |
| X4 | Bundle 下游交付 local-first 扩展 | 第二轮 | 悬置——与 C4 相邻不同（通道侧 vs 内容侧），互不阻塞 |
| X5 | 托盘常驻/后台守护 | 第二轮 | 悬置（常驻≠监听重分析，与 watch 排除项不同） |

（B2/B4 已随前两轮结论归入 CI/团队共享排期域，不再重复登记。）

---

## 6. 伪需求 / 不升格清单

6 条不值得做项全录（【③】§8），每条附理由与复核条件：

| # | 项 | 理由 | 复核条件 |
|---|---|---|---|
| 1 | **changelog/release notes 生成升格**（S-10/H1） | 外部信号弱：GitHub 平台内置 Release notes 自动生成已吞掉通用需求，嵌入式侧多轮定向检索无独立强信号——这是"内生理想"而非"外部拉力"，不因基线有字眼而自动升格。差异化仅在"从构建元数据（组件漂移、门禁结论、符号档案）汇编发布说明"，那是 C1/C4 落地后 report crate 的 S 级副产品，不是独立立项理由。**维持基线"如有"口径** | V1 访谈出现高频手工维护 release notes 痛点 **且** C1/C4 已落地 → 作 S 级副产品再议（非独立立项）；禁 AI 润色（ADR-0005） |
| 2 | **二进制/分区级 diff**（S-11） | 紧邻 Non-goals #5（不做逆向套件），一步即触禁区；符号级 diff（P2 已排）覆盖约九成发布决策需求，字节级增量有限；设计侧补充排除理由：hex view（逆向工具视觉语言）/热力图（优先级链外图型）的呈现形态本身就出界 | 无常规复核路径。若出现"自家两版发布的字节级完整性自证"明确需求，先过 Scope Creep 四问并证明符号级 diff 不够用，再议 |
| 3 | **OTA 发布物元数据**（S-09） | OTA 在推迟清单与禁跳清单两处禁区；极窄候选须过 Scope Creep 四问，默认不做。MCUboot header 字段清单（version/hash/signer/anti-rollback）可留作未来参考 | V1 访谈 OTA 团队频现 → 作 P4 可选字段再议 |
| 4 | **全量构建环境复现指纹**（S-08 完整形态） | 容器/环境哈希超出本地可观测范围，只能 Declared 手工喂——与二轮 build log 同判例：维护无底洞、增益低；S 级 flag 摘要已拿走值得拿的部分 | 仅当出现可本地观测的新证据源再评估 |
| 5 | **`.su` 升格时并入调用图/未知被调者标注** | 静态调用图重建是 L 级独立需求，与 `.su` 列展示是两个 ADR 的事；并入会让升格裁决背上不必要重量 | `.su` 升格落地且 V1 访谈证实函数指针断链痛点高频 → 独立 ADR 立项 |
| 6 | **AI 参与任何判定/生成** | ADR-0005 可信核心禁入 AI；本轮信号无任何支持重开的新证据 | 重开须用户级裁决 + ADR-0005 修订 |

另两条管理员提示方向经扫描**无新信号**，维持排除（如实记录，非新增判定）：团队 policy 共享（Growth 排期佐证不足）、多构建趋势追踪（P1+V7 双排除，检索所得均为既有佐证）（【②】§4）。

---

## 7. 与 MVP 的关系

**明确：本轮全部结论不纳入 MVP。**（用户亲自定位 Post-MVP；Active Task = NONE 不变；本文不触发功能创建、不修改任何基线文档。）

1. **阶段标注口径**：本文"建议阶段"（P2/P4 等）是路线图落位——仅当候选未来被采纳并通过治理后才生效；它不改变"MVP 范围一寸不扩"的结论。基线 PRD/Scope 冻结的 MVP 范围不受本文影响。
2. **本轮探索的一切增量**（C1–C5、S-03/S-08 搭车、N1–N7 脑洞）默认进入"Post-MVP 候选池"，**候选≠承诺**。至此三轮探索共累积三个候选池（可视化与导出、端口与体验、证据完整性），全部处于待拍板/待信号状态。
3. **未来引入的治理路径（不可绕过）**：
   - 每项先过 **Scope Creep 四问**（FS-PRD-004）——注意对"新证据维度"类候选（C2/C5）杀伤力最大（命中"新增数据维度/模型扩展"），默认推迟是常态、突破是例外；
   - **C 类项走 ADR**：C2（域模型扩展）、C5 第二步（子系统）；新依赖（如 serde_yaml）过 FS-TECH-012/FS-ENG-006 准入；
   - **设计条款先行**：能力 banner "一源一位"扩位纪律 + 变更呈现三纪律（§8-Q4/Q5），S-05/S-07/C2 动工前完成；
   - **R4 措辞红线**：合规向候选（C3a/C3b/S-03）只出工程证据不出合规结论，措辞守 07_COMPLIANCE 白名单；
   - Growth 项终极试金石 = 四不破坏（local-first / deterministic core / evidence provenance / explicit unknown）；任何候选必须挂靠四动词，不能成为第五动词。
4. **合法推进路径只有一条**：探索与提案先行，实现让位于阶段门。当前唯一合法下一步仍是 P0 Technical Vertical Slice。

---

## 8. 待用户拍板的开放问题（四份输入去重汇总，每条附建议）

| # | 问题 | 来源 | 建议 |
|---|---|---|---|
| Q1 | **`.su` 栈证据是否升格**为 P2 候选排期（三份输入指向同一裁决）：升格则 ADR 范围限定为"最坏栈列 + 排序 + 超限标红"，调用图明确排除；设计形态预认（Symbols 列 + 缩进调用路径 + banner 扩位，不再过一轮设计评审）；V1 访谈口径从"要不要"改为"优先级与场景强度" | ③§9-1；④§4-3；②§2 | **建议升格**——三源同构强信号 + puncover 维护真空窗口 + 技术/设计成本双低（C 类里最轻的 ADR）。若求稳，可降档为"升格进 P2 候选池，ADR 在 P1 symbols/evidence 落地后再提" |
| Q2 | **lockfile 首批范围**：是否收敛到 ESP-IDF `dependencies.lock` 单源起步，其余场景 Declared 手动声明兜底 | ③§9-2 | 接受——避免重蹈"大正则通吃"覆辙（FS-TECH-004 纪律）；Conan 等其余 lockfile 凭 fixture 信号逐个准入 |
| Q3 | **符号档案默认内容策略**：显式勾选（不整目录吸入）+ manifest role 字段 + 体积提示；挂 V1 访谈验证 crash post-mortem 场景 | ③§9-3；④§1.4 | 接受 |
| Q4 | **能力 banner "一源一位"扩位纪律**写入 DESIGN.md（本轮触发源：`.su` 位、SBOM/license 维度、retention 复验位；banner 是"证据在不在"的单一真相，不是功能 bragging 面） | ④§3.2/§4-1 | 接受（文档级变更，随下一次设计资产升版入册） |
| Q5 | **变更呈现三纪律**（变更表在前、判定在 Gate / 证据分级随值而行 / 未变即折叠）随下一次设计资产升版增补进 DESIGN.md ⑤/⑥ 条款 | ④§3.3/§4-2 | 接受（S-05/S-07/C2 动工前完成即可） |
| Q6 | **H3 与 H2 的优先序确认**：SBOM evidence completeness 计数行（H3）优先于 SPDX 3.0 导出（H2）；SPDX 继续悬置 | ②§3；④§1.3 附议 | 确认 H3 优先——外部信号显示痛点在"SBOM 可信吗"而非"多一种格式" |
| Q7 | **次级搭车项采纳**：license 维度随 SBOM P1 预留（含 R4 措辞纪律）+ 工具链 flag 摘要随 P1/P2 Identity 增列——均不独立立项 | ③§7 | 采纳搭车口径；两条排期随宿主阶段走，不新增排期评审 |
| Q8 | **Reddit 摘要级证据是否补查原帖**（S-02/S-05/S-08 引用的社区帖为搜索引擎摘要级） | ②§1/§6 | 不主动补查（网络环境受限、时效有限）；引用任何 Reddit 条目时保留"待证实"标记；S-02 的一手部分（CRA 法规时间线）不受影响。临近立项时定向核验 |
| Q9 | **CRA 时间窗挂靠**：V1 访谈问题清单与 B1 定价实验是否显式挂靠 CRA 窗口（2026-09 报告义务已启动、2027-12 全面适用） | ②§6 机遇 2 | 挂靠——访谈清单加入 CRA 驱动场景问题，定价实验观察 deadline 驱动采购信号；同时守 R4：产品与营销措辞不承诺合规结论 |
| Q10 | **三轮候选池归并**：三轮探索已累积三个候选池（可视化与导出 / 端口与体验 / 证据完整性），是否授权归并为一份《Post-MVP 候选池总册》（统一编号、统一状态、标注依赖关系），供 V1 外部验证后按信号统一筛选 | 综合环节提出 | 建议归并——防散落、防重复论证；三轮结论多处互相咬合（如 C1 依赖 P1 Component Evidence、C4 与 X4 相邻），总册可把依赖关系显式化；不改变任何单条"候选≠承诺"状态 |

---

*综合执笔：专业写手 · 2026-09-27 · 本文为探索交付稿，不修改任何基线文档，不触发功能创建；所有事实以四份输入所引基线原文为准。口径分工：信号强度与需求真伪以信息哨兵调研口径为准，技术成本/插槽/阶段以鲁班七号评估口径为准，呈现形态与防噪以设计匠人备忘口径为准，边界标注以通用助手清单口径为准；口径冲突时以基线原文为准。*
