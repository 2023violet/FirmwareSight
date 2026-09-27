---
title: "FirmwareSight 正式产品功能探索：端口扩展与深度体验（Post-MVP）"
product: "FirmwareSight"
version: "1.0"
date: "2026-09-27"
authored_by: "专业写手（综合执笔）"
status: "FINAL — 综合交付稿 · 本轮全部结论不纳入 MVP，不改变 Active Task 状态"
inputs:
  - "通用助手《背景回顾：端口扩展与深度体验的基线事实清单》（53 条事实 + 硬约束/可扩展空间标注 + Persona 12 阶段触点盘点 + 5 类未触达环节）"
  - "信息哨兵《端口扩展与用户粘性·外部调研报告》（FS-RSCH-012，28 信源，强 10/中 12/弱 6）"
  - "鲁班七号《端口扩展与深度体验·技术可行性简报》（七端口 A–D 插槽分类 + S/M/L 成本 + 九条不值得做清单）"
  - "设计匠人《端口扩展与深度体验·设计视角备忘》（端口准入三问 + W1–W5 体验主张 + 联单纪律 + B1–B6 行为红线）"
citation_key: "本文引用标记：【①】=通用助手清单，【②】=信息哨兵报告，【③】=鲁班七号简报，【④】=设计匠人备忘；基线编号（FS-TECH-xxx/ADR-xxxx 等）以四份输入所引原文为准。"
---

# FirmwareSight 正式产品功能探索：端口扩展与深度体验（Post-MVP）

> **阅读前提**：本议题由用户定位为 Post-MVP 探索。本文不改变治理状态——当前唯一合法下一步仍是 P0 Technical Vertical Slice。本文回答的是"未来哪些端口值得做、按什么顺序、明确不做什么、体验深耕往哪个方向"，不是"现在做"。
>
> **术语口径**（沿用【①】§1）：本文"端口"一律指**产品对外的输入/输出/集成接口**（产品端口）；基线 Ports & Adapters 的架构内部抽象称"架构端口"。两者不得混用。
>
> **方法声明**：结论综合自四份输入（上方 frontmatter），每条标注来源与证据强度。遵循语调三律：说事实不做判决、说未知不填空（所有"无信号"如实标注，缺失不等于不存在）、给下一步。

---

## 1. 议题定义与结论先行

### 1.1 议题原文与用户口径

用户提出（2026-09-27）两件事：①为 FirmwareSight 探索更多"端口"（输入/输出/集成接口），让产品从单一分析工具长成嵌入式工作流枢纽，产生粘性；②人群窄，深耕现有四类 Persona（Firmware Engineer / Lead / QA / Founder）的使用体验。并给出价值判据：

> **"通用的目的不是功能多，而是让用户走得慢；便捷的目的不是好用，而是让用户不想走。"**

### 1.2 金句的产品转译：从口径到可检验判断

金句不是修辞，是验收标准。【④】§0 已完成设计转译，【①】§1 确认其与基线原则同构（"走得慢"≈ Evidence is one click away / 深度＞功能数量；"不想走"≈ Return intent / repeat usage / recurring use 三级回归指标）。综合为五条可检验判断：

| # | 检验 | 内容 | 来源 |
|---|---|---|---|
| J1 | **走慢测试** | 一个端口/功能若只增加"回来看的次数"，不做；若增加"每次停留能完成的判断深度"，做 | 【④】§0 |
| J2 | **留客测试** | 粘性必须由复现力 + 状态延续 + 时间纵深构成；依赖迁移成本或行为回路的粘性，一票否决（见 §6.2 B1–B6） | 【④】§0/④ |
| J3 | **粘性三层排序** | 数据引力 ＞ 流程位 ＞ 协作网络 ＞ 体验优势。体验会磨损，流程位和数据不会；单一功能工具只有最弱一层 | 【②】§2.4 |
| J4 | **端口准入三问** | 它打印什么证据？它在什么阈值下才出声？它把用户带回哪个读数？答不出就不做 | 【④】§0 |
| J5 | **既有度量锚点** | MVP Return intent ≥5/8；V1 repeat usage；B1 real recurring use + support cost；Growth 按真实付费需求逐步增加——金句作价值判据，不替代这些门禁 | 【①】§6.4 |

### 1.3 结论先行：更多端口 + 深耕窄人群，是否让产品更难被替代？

**是——有条件成立。** 外部证据支持"端口=粘性"的基本判断，但不可替代性的本体不是端口数量，而是"一个收敛的证据内核 + 占住的流程位 + 本机时间纵深"。三个限定条件缺一不可。

**支撑证据链（正面）：**

1. **枢纽沉淀实锤**：PlatformIO（多平台构建→VS Code→注册表，变现走 Premium Support $249/月）、Memfault（设备 SDK 端口→云观察，2025 被 Nordic 收购）、SonarQube（分析→Quality Gate，进入合并决策链）、Codecov（coverage→PR comment→status check，2022 入 Sentry、2026 入 Harness）——集成位本身能被反复并购定价。【②】§2.2，强
2. **本域商业验证**：MemBrowse 以"本地 CLI 分析 → CI 上传 → PR comment → 预算门禁 → 云端历史"端口链，一年内完成商业化（Free/$99/$249 按 tracked targets 计费，客户墙含 Flipper Devices、wolfSSL、Apache NuttX、RT-Thread、TinyUSB）。【②】§1.2，强
3. **结构性空位**：嵌入式 size 域"官方 CLI + 官方 Action"的组合尚无开源先例（现有集成全靠社区 Action 或商业闭源补位）；自包含 HTML 证据报告依旧无人做——云产品的商业模式决定了它不会做。【②】§1.1/§7
4. **差异化本体**：单 build 分析可被替代（Bloaty/puncover 免费开源 + IDE 内置墙：STM32CubeIDE/ESP-IDF/Segger 均免费内置单次查看）；带证据链的时间纵深不可替代——数据在本机、口径由 FirmwareSight 统一、每个历史点可复现、基准 pin 即有对照系。【④】W3；【②】§1.1

**反面实锤（不做的代价）：**

- Bower 官网至今挂着官方弃用劝退声明——单一功能工具被工作流枢纽吸收的官方盖章案例；`arm-none-eabi-size` 单条命令正被分析平台吸收（Memfault 教学文→MemBrowse 产品化）。停在"单次分析器"，就是下一个被吸收对象。【②】§2.1
- 用户亲手点名的端口缺口："分析工具存在，但没接入开发者工作流"（r/cpp 高赞）。【②】§3.2 S3，中

**三个限定条件（成立的代价）：**

1. **端口数量≠粘性，端口可信度=粘性**（【③】§2.3）：每个新格式的真实成本大头是 fixture+golden+能力报告措辞，不是 parser；Supported 四门槛是成本放大器，也是信任护城河。广撒网式加端口反而稀释信任。
2. **性价比最高的粘性路径不是加新端口**：七端口中有五个踩在架构已预留插槽上（A 类），但真正应该先做的是四件 S 级插槽内事项——JSON schema 版本化输出、config JSON Schema 发布、git hooks/CLI 管道配方、VS Code task 配方。零新概念、零新依赖，每一样都在用户的脚本、仓库和日常动作里留下接点。【③】§0
3. **云生态位必须避开**：MemBrowse 已占"云 portal（历史/趋势/对比/门禁）"生态位并有一年客户积累，正面仰攻云订阅是弱势开局。错位=本地优先（数据不出内网）+ 桌面工作台 + 自包含 HTML 证据——这恰是云订阅模式做不了的形态。【②】§6.2

**对"深耕窄人群"的判断**：四类 Persona 共性是人少、多板多项目并行、发布低频高风险（【④】②）。【①】§6 的 12 阶段触点盘点显示未触达环节高度集中：五类未触达中，两条已有基线排期（History=P1、CI=Growth），两条是纯空白（watch/自动发现、命令面板），两条是红线区（IDE 编码侧、发布后 fleet）。**深耕的真实含义=把既有排期兑现 + 在空白区克制补齐 + 红线区绝不伸手**，不是新开功能面（详见 §4）。

### 1.4 一句话总回答

> 成立——但让产品更难被替代的，不是更多出口，而是**同一证据内核派生的全部出口 + 必经之路上的流程位 + 本机带证据链的时间纵深**。金句的可检验形态即 §1.2 五条检验；本轮全部候选仅入"Post-MVP 候选池"不排期，MVP 不因此扩大一寸。

---

## 2. 背景回顾摘要（引用，不重提）

完整 53 条事实及逐条【硬约束/可扩展空间】标注见：通用助手《背景回顾-端口扩展与深度体验的基线事实清单-通用助手.md》（shared/firmwaresight/）。与本文结论直接相关的锚点：

| 主题 | 既有事实（出处见原文件对应节号） | 性质 |
|---|---|---|
| 定位与流程 | 两议题=Post-MVP 探索；ACTIVE_TASK=NONE；任何具体化须过开发纪律三问 + Scope Creep 四问 + ADR | 硬约束 |
| 架构插槽 | 四架构端口就位（Artifact/Storage/Provenance/Report）；MapAdapter 有 `future adapters` 明文插槽、禁大正则通吃；ReportRenderer 唯一报告生产者；"future CI/headless 不需要重写"是架构明文预留意图（§2） | 硬约束 + 扩展空间 |
| 入口边界 | IPC command 必须=用户 use case（禁通用 fs/shell/sql command）；CLI=与 Desktop 平级的第二入口，六命令+稳定 `--json`+exit codes 0/2/3/4/5/6；CI 不允许解析自然语言；FS-ENG-007 是自身研发 CI，≠用户 CI 集成，不得混引（§2.6–2.7） | 硬约束 |
| 配置即代码 | `firmwaresight.toml` 可入 Git + team gate policy = 现成的唯一团队协作原语；unknown keys=warning、type 错=hard error（§2.8） | 硬约束 + 扩展空间 |
| 本地优先红线 | ADR-0004 local-first；ADR-0013 MVP 无 HTTP/TLS（Reqwest+Rustls 是唯一批准的未来路线）；ADR-0005 AI 不进可信核心（§2.10） | 硬约束 |
| 排期锚点 | watch、命令面板：基线零出现（无承诺无禁令）；CI 集成=产品层 P1 headless→§26（v1.0 后）→Growth；History=P1；SBOM=P1；policy 层层递进（§3） | 硬约束（排期）+ 扩展空间 |
| 产品纪律 | 四动词纪律（新端口必须挂靠，不能成第五动词）；Non-goals 八项；三张推迟/禁止清单的差异化语义；Growth 四不破坏（local-first/deterministic/evidence/unknown）（§4） | 硬约束 |
| 商业分层 | 粘性押 Pro（unlimited history + CLI），集成押 Team（CI/shared policy/audit）；"真正可收费的不是 treemap，而是 repeatability/team policy/CI integration/compliance"；不按使用量收费；写 Team 能力前须过付费验证门槛（20 访谈/8 试用/3 付费意愿/1 试点）（§5） | 硬约束（假设+流程） |
| Persona 触点 | 12 阶段×四 Persona 盘点（推导分析，非基线原文）；五类未触达：构建侧/流水线侧/协作侧/记忆侧/外部世界侧（§6） | 口径声明 |
| 外部证据 | MemBrowse 全案（形态/定价/客户墙/隐私口径）；粘性三层来源；收费先例集中在流程位不在导出物；Codecov 供应链泄露反例（信息哨兵报告全文） | 调研结论 |
| 技术评估 | 七端口 A–D 插槽分类与 S/M/L 成本；九条不值得做清单（鲁班七号简报全文） | 技术结论 |
| 设计主张 | 端口准入三问；零新增一级导航；通知三律 N1–N3；联单纪律四条；B1–B6 行为红线（设计匠人备忘全文） | 设计结论 |

---

## 3. 端口扩展清单

综合技术评估（【③】七端口×A–D 分类）、外部信号（【②】§3/§6.1）、设计落位（【④】§1.5）。**A 类=已预留插槽（不改冻结契约）**；**B 类=插槽内新功能（须过 Scope Creep 四问+依赖准入）**；**C 类=须 ADR**；D 类进 §6 排除清单。粘性价值按 §1.2-J3 三层标注。

### 3.1 A/B 组：插槽内低成本项（优先池）

| # | 端口 | 挂靠动词 | 粘性价值（层级） | 需求信号（来源+强度） | 成本 | 建议阶段 | 类型 | 设计相容性 |
|---|---|---|---|---|---|---|---|---|
| P-1 | **CLI 管道深化**：全命令 `--json` 覆盖 + 输出带 `schema_version` + stderr 诊断 ID + 管道配方文档（jq / git hooks / Makefile / CI job 模板） | Analyze→Gate | 流程位起步——脚本与 hook 一旦写下，迁移成本即刻产生 | "CI 不解析自然语言"是行业公理；CLI+稳定 JSON 是一切集成的地基【②§1.3，强】 | S | P1–P2 随批，不单独立项 | A | ✅ 版本化 stdout schema；UI 不复刻 gate 逻辑【④§1.5】 |
| P-2 | **数据端口**：config JSON Schema 发布（随应用分发+文档页）+ policy presets | Gate/Release | 协作网络最小形态——"共享 policy"的最小实现已躺在设计里，代码量为零；配置真源留在用户仓库 | 基线已锚（产品层 P1 policy presets）；FS-TECH-009 演进机制内建【①§3.1；③§8】 | S | P1 | A | ✅ Settings 单一入口；schema 版本化 |
| P-3 | **工具链输入扩张**：Keil `.axf`（S–M）→ ArmClang MAP（M）→ IAR MAP（M）→ CSV 尺寸报告（S；Declared 侧输入，无 ELF 时作 diff 基线，不覆盖 Observed） | Analyze | 窄人群深耕：直击 Persona A 中被冷落的 Keil/IAR 用户；Narrow compatibility before false compatibility | 支持矩阵已承诺（产品层 P1 Keil/ArmClang、IAR P1/P2）【①§2.9】；MemBrowse 已支持 IAR ICF/Segger 布局——"慢一步就成它支持我们不支持"【②§6.1#6，强】 | 每格式 S–M；**成本大头=fixture+golden+能力报告措辞，非 parser** | P1 启动 Keil 包→P2 IAR/CSV | A | ✅ 能力 banner 如实标注证据等级；禁把"支持格式数量"当营销点【④§1.5】；节奏被 fixture 门禁锁死【①§2.9】 |
| P-4 | **编辑器低成本存在**：toml JSON Schema（VS Code 装 Even Better TOML 即得补全/校验）+ `.vscode/tasks.json` 配方 | Analyze/Gate | 编辑器内"零跳转"存在感的最低成本形态；不做扩展也能拿到 | IDE 内置墙下差异化只剩 release 语义，配方足够【②§1.1；③§6.2】 | S（文档级） | P1 | A | ✅ 读数切片进编辑器，不是第二 UI |
| P-5 | **Git/provenance 小扩容**：`describe --tags --always` 兜底、commit date、branch（只读事实扩容）+ pre-push/pre-tag 跑 gate 的 hooks 官方配方（纯文档） | Analyze（build identity） | git 动作是必经之路，hooks 配方把 gate 嵌进既有习惯 | git describe 注入版本串是嵌入式惯例【②§3.1】；Roadmap P3 本就含 provenance【③§7.2】 | S | hooks 配方 P1 文档级；事实扩容 P3 随批 | A | ✅ specimen header 增字段，不改结构 |
| P-6 | **配置段前置**：`[artifacts.paths]`/`[watch]` 段 + per-section budgets（依赖 EPIC B 分类策略先冻结）+ release-manifest schema 收紧 | Analyze/Gate | "先有 config 再有功能"——为 watch/CI 铺路径配置，防功能私造配置 | FS-TECH-009 演进机制内建【③§8.2】 | S–M | P2 | A | ✅ Settings 单一入口；gate 语义扩展逐条 spec+golden |
| P-7 | **watch 模式**（本地目录监听→debounce→入队既有 import use case；Core 零改动） | Analyze→Compare/History | 数据引力——给时间纵深自动喂料：替用户完成"每次 build 后跑一遍"，不是实时性表演【④§1.4】 | 基线零出现零禁令【①§3.1】；Persona A 全部触点以"主动打开+手动拖入"为前提【①§6.2】；外部惯例 jest --watch/cargo watch，无强呼喊【②§1.3，中】 | M（notify 依赖准入 + 触发/合并语义 spec + 跨平台测试；Windows 链接器锁占用可解） | 工程 spike 可 P2–P3 间隙（重点验 Windows 行为）；**产品决策放 V1 外部验证后**，凭"用户是否真的高频重复分析"信号拍板【③§3.2】 | B | ✅ 三档 Off/Armed/Triggered（间隔默认 60s=下限、可见可调）；触发输出=History 追加同形态记录，非弹窗；通知三律 N1–N3；⚠️ 依赖 History(P1) 排期 |
| P-8 | **非 Git provenance 来源**（source tarball+SHA、构建元数据文件） | Analyze | 补 evidence 覆盖面（非 Git 工作流团队） | 无当期信号（说未知不填空） | M（每源 fixture+spec+证据定级） | Post-MVP 凭信号 | B | ✅ 走归一化管道+能力 banner |

### 3.2 C 组：须 ADR 高成本项（后置池）

| # | 端口 | 挂靠动词 | 粘性价值（层级） | 需求信号（来源+强度） | 成本 | 建议阶段 | 类型 | 设计相容性 |
|---|---|---|---|---|---|---|---|---|
| P-9 | **CI 端口**：headless CLI 硬化（前置）→ GitHub Action（薄壳包 CLI + Job Summary / PR comment）+ GitLab CI 模板；gate→exit code→status check 纯映射（BLOCK=5→failure、REVIEW=4→neutral+annotate，全程无自然语言解析） | Gate→Release | **流程位最强层**——从"看"变"拦"，进入合并决策链；Team 层商业锚点（CI 与 shared policy 咬合） | **全议题最强信号格**：Memfault 实战文（强）+ MemBrowse 一年客户墙（强）+ cargo-bloat-action / size-limit 产品样本（强）+ "分析工具存在但没接入工作流"（r/cpp，中）【②§3.2 S1–S4】 | headless 硬化 M（前置）+ Action S–M + GitLab S | headless=产品层 P1；Action/GitLab=Growth（FS-DEL-001 与生命周期 §26 两处口径一致）；若 V1 出现团队付费信号，可提前至 P5 试点【③§4.3】 | C（新增分发通道与安全姿态→须 ADR，牵动 release engineering 产物矩阵） | ✅ CI tty 打印件（摘要≤一屏，`[OBS]/[DRV]/[DEC]/[UNK]` 前缀）；PR 评论五件套（铭牌头/verdict/分级计数/Top findings≤3/折叠打印件）；联单纪律四条；bot 只报读数不判刑；渲染与解析分离（CLI 保持纯与确定性）；无遥测无上传 |
| P-10 | **`.su` 输入**（GCC `-fstack-usage`，per-function stack usage） | Analyze | 新证据维度 | **零当期信号**；且要求用户以特定开关编译，天然覆盖缺口【③§2.2】 | 解析 S / 整体 M（域模型+存储 schema+diff+UI 连锁） | 列入 V1 访谈验证问题清单，需求强度验证后再立项 | C（域模型扩展→须 ADR） | ✅ 新证据类型走归一化管道；能力 banner 如实标注 |
| P-11 | **VS Code 扩展**（非配方形态） | Compare/Gate 的读数引用 | 存疑——IDE 内置墙下差异化只剩 release 语义，M–L 成本换重复价值，性价比低 | 无当期信号【③§6.2】 | M–L | Post-GA，且仅当 V1/B1 出现真实编辑器内需求信号再立项；届时仍须 Scope Creep 四问 | C/ADR 级评估 | ✅ 打印件切片进编辑器，不得长出分析界面（Non-goals 不做 IDE）；LSP=D 类不做 |

**优先级梯队汇总**（沿【③】§11，候选≠承诺）：

```text
第一梯队（P1，S 级，插槽内）：P-1、P-2、P-4、P-5（配方部分）+ P-3 Keil 包启动（视 fixture 获取）
第二梯队（P2–P3，随阶段自然发生）：P-3 IAR/CSV、P-6、P-7 watch spike（P2–P3 间隙）
第三梯队（Post-MVP / Growth，须 ADR 或凭 V1 信号）：P-9（headless→Action/GitLab）、
  P-7 产品化（V1 信号后）、P-10（V1 访谈后）、P-11（Post-GA 条件触发）、P-8
不做：§6.1 全表
```

**实现铁律（全端口通用，来源：③①④）**：新输入格式=新 adapter+fixture 门禁，走归一化管道，禁大正则通吃；新输出格式挂 ReportRenderer；新桌面能力以 use-case command 过 IPC 边界；CI 侧走 CLI 出口、不经桌面 IPC；渲染与解析分离；无遥测无上传；新依赖过 FS-TECH-012/FS-ENG-006 准入；一切 Growth 增量过"四不破坏"试金石（local-first / deterministic core / evidence provenance / explicit unknown）；端口必须挂靠四动词，挂不上动词的端口默认是 Scope Creep。

---

## 4. 窄人群深度体验提升清单

### 4.1 四类 Persona：未触达环节 × 体验主张对照

未触达环节引自【①】§6.1–6.3（推导盘点，非基线原文）；体验主张引自【④】§②（W1–W5 全部通过走慢/留客双测试与哲学相容性检查）。

| Persona | JTBD 要点 | 未触达环节（①） | 对应体验主张（④ + 本轮端口） | 建议落点 |
|---|---|---|---|---|
| **A · Firmware Engineer**（STM32/ESP32/Nordic…；GCC/Keil/IAR） | 交付 build 时快速确认内存/版本/二进制/Git 一致 | ①日常流（IDE→编译→烧录→测试）零存在感：所有触点以"主动打开+手动拖入"为前提；②Keil/IAR 窗口内仅 Experimental/P1 承诺；③习惯回路（watch/自动发现）空白 | W1 项目记忆（多板并行，重开接得上）+ W2 Recent builds（日常入口，一行一 build）+ P-7 watch（自动喂料）+ P-3 工具链扩张（Keil 包直击被冷落人群） | watch spike P2–P3 / 产品决策 V1 后；Keil 包 P1–P2；History P1（基线既定） |
| **B · Firmware Lead** | 要可复核的差异与证据，不是开发者口头"应该没问题" | ①"复核"动作发生在聊天工具/邮件/会议，产品无协作面；②regression 长期趋势依赖 History(P1) | W3 时间纵深+基准 pin（复核有固定对照系）+ W5 复现与交接（随时向第三方证明仪器给出过什么）+ P-9 PR 联单（verdict 送达评审现场） | History+基准 pin P1；PR 联单随 CI 端口 Growth；协作流转见 §5-X3 |
| **C · QA / Quality / Compliance** | 拿结构化证据，不临时向研发收集截图和 Excel | ①取数导出未实现（上一轮探索已论证归 P1）；②audit history 在 Team 层；③SBOM P1/P2 | W5 复现与交接（证据包替代截图+Excel）+ 打印件/联单纪律（证据分级随值而行、Unknown 独立节、报告自解释带图例） | 导出清单沿《FS-正式产品功能探索-分析结果可视化与文档导出》交付（P1 CSV/MD/HTML 增强）；SBOM 基线既定 P1 |
| **D · Small Hardware Founder / CTO** | 无专职 DevOps 也低成本专业发布 | **最痛的恰是 CI 集成空缺与 Growth 排期的时机错配**——没有 DevOps 意味着更依赖自动化，而 CI Integration 锚在 v1.0 后【①§6.2，本轮记录的关键事实】 | 四动词闭环+CLI 已给专业感；W5（走得完整：不需要 DevOps 也走完整发布流程）；对 D 的粘性不能建立在频率上（D 是低频用户），只能建立在"每次来都接得上、走得完整"【④②总论】 | CI 端口 Growth；建议把"D 类用户对 CI/自动化门禁的需求强度与付费意愿"列入 V1 访谈优先项（见 §8-Q10） |

### 4.2 共性主张：工作台记得活儿

深度体验的本质一句话：**工作台记得活儿**（【④】②总论）。W1–W5 五项主张按价值排序：

1. **W3 时间纵深+基准 pin（价值：极高——对"产品单一易被替代"的直接回答）**：History 不是日志列表，是记录纸（chart recorder）——每个 build 是一条可回放读数，任意两条可设为 A/B，基准 pin 作长期对照。用得越久历史越值钱，这是**诚实的"越用越有用"**，与行为钩子有本质区别（区别即 §6.2 红线）。
2. **W1 项目记忆 / W2 Recent builds（价值：高）**：记住上次所在视图与选中项、基准选择；History 顶部短列表只做时间序，不做活动流、不做智能排序。
3. **W4 工作台常驻感（价值：高）**：常驻感≠常驻存在——不默认自启、不做红点徽标、无"你 3 天没来了"。下一步=证据工作的下一步，不是回访的下一步。
4. **W5 复现与交接（价值：高）**：粘性的正当来源=复现力。每份读数一键拿到"何输入、何版本、何命令可复现"。

**记忆侧的既有地基**：SQLite 初始表已含 `gate_runs / gate_findings / release_records`——History 与审计的数据地基已预留（【①】§2.5，【②】§6.1#5：数据已在，缺呈现，M 级）。History=P1 基线既定，是"让用户不想走"的最大既有钩子（【①】§6.1#9）。

### 4.3 验收口径（既有度量锚点，【①】§6.4）

MVP：Return intent ≥5/8、Discovery ≥30%、Analyze→Compare 完成 ≥60%。V1：repeat usage。B1：real recurring use + support cost。Growth：按真实付费需求逐步增加。本节全部主张的验收回到这些锚点，不新增指标体系。

---

## 5. 开放脑洞区（纯探索性质）

> 本节所有条目**仅为脑洞登记，不构成建议承诺**。与上轮交付同规则：防止好想法丢失，同时防止它们未经评审就溜进排期。任何一条未来推进前，须先获得需求信号，再过端口准入三问（【④】§0）+ Scope Creep 四问 + ADR。

| # | 脑洞 | 可能的价值 | 现实约束 / 前置条件 |
|---|---|---|---|
| X1 | 命令面板（command palette） | 基线零出现零禁令的纯空白【①§3.1】；键盘流 power user 的单次停留深度 | 与"一屏一主焦点"存在张力，须过设计评审；当前无任何用户信号 |
| X2 | Agent 端口（Claude Code plugin 形态） | 2026 新变量：MemBrowse 已支持 AI agent 自动接线 CI【②§1.2】 | 仅观察不跟进，直至真实用户信号；涉及分发形态=ADR 级 |
| X3 | 决策协作流转（谁审、谁批、在哪讨论） | Gate 的 Review 显式接受目前是单机动作；协作面仅在 Post-GA 清单出现一次【①§6.1#6】 | Post-GA 锚点已存在；须先过付费验证门槛（20 访谈/8 试用/3 付费意愿/1 试点）【①§5.2】；云协作形态受 ADR-0004 约束 |
| X4 | Bundle 下游交付的 local-first 友好扩展（用户指定归档目录 / Git 工作区约定路径） | US-004 交付通道（文件系统目录）的语义内便利化；不联网不触红线 | 邮件/IM/工单等联网回写通道触 ADR-0004/0013，明确不做【①§6.1#7】；仅限本地文件系统语义 |
| X5 | 托盘常驻（OS 习惯可选项） | 【④W4】边界允许：可选项、无红点徽标、无角标计数 | 不默认自启；优先级最低；须先有真实需求信号 |

---

## 6. 伪需求 / 陷阱排除清单

### 6.1 九条不值得做的端口（D 类，【③】§9 全录）

| # | 端口 | 依据 | 替代答案 | 复核条件（如有） |
|---|---|---|---|---|
| 1 | Plugin System / 扩展运行时 | 生命周期 §27 明文"不要直接跳到"；plugin marketplace 在"不自动扩展"名单【①§4.2】 | 把"插件需求"导向已冻结的**类型化端口**：CLI JSON、TOML、schemas、ReportRenderer、adapter fixture 规范——端口够用，不需要运行时 | 非永久禁止：Growth 之后 + ADR + 用户裁决才可重启【①§4.2】 |
| 2 | LSP / C/C++ 语言特性 | Non-goals 明文"不做 LSP" | config JSON Schema + tasks.json 配方（P-4） | 做即修改 Non-goals，方向性冲突，无技术层复核路径 |
| 3 | 构建系统集成（CMake/Make/PlatformIO 插件、构建注入） | §27 "Build System" | watch 模式（观察产物，不接管构建）（P-7） | — |
| 4 | 通用 MAP / build log "大正则"解析器 | FS-TECH-004 明文禁止"一个大正则通吃"；log 证据等级低、维护无底洞 | 按 toolchain 的 MapAdapter；`.comment` Observed 取 toolchain | — |
| 5 | Flasher / OTA / 设备管理端口 | Non-goals + 章程非目标【①§6.1#12】 | — | — |
| 6 | 云端口（webhook、在线协作、云 dashboard） | ADR-0004 local-first + ADR-0013 network deferred；网络面仅预留 update metadata / CVE feed / license API | CI 在用户侧 runner 跑本地 CLI（P-9），不上传 | Reqwest+Rustls 是唯一批准的未来路线，触发条件=签名更新/CVE feed/licensing/optional cloud【①§2.10】 |
| 7 | 逆向 / 漏洞分析导入端口 | Non-goals "不做 Firmware Reverse Engineering Suite" | — | — |
| 8 | 可编程 policy DSL | 破坏 deterministic 与可审计性（治理气质冲突，无明文，存档） | 声明式 TOML + presets | — |
| 9 | AI 判断端口 | ADR-0005 no-AI in trusted core | 若未来做 AI，只能解释/搜索/总结且必须标 advisory | — |

### 6.2 B1–B6 行为红线（一票否决，【④】§④ 全录）

总判据：**仪器的可信来自它对注意力 indifferent——不渴求被使用，才值得被信赖。** 视觉层不装（上轮 11 陷阱），行为层不缠（本节）。

| # | 反模式 | 替代（走慢测试的胜利） |
|---|---|---|
| B1 | 游戏化积分 / 徽章 / 等级 | W3 时间纵深：用得越久历史越值钱，这是不需要奖励的留存 |
| B2 | streak / 连续使用天数 | 无替代——仪器应当随时可用、不在意你何时来 |
| B3 | 通知轰炸 / 每日每周 digest | 阈值事件通知（出厂仅订阅 BLOCK），其余静默入 History；digest 通常需要服务端，与 local-first 冲突 |
| B4 | 引导成瘾 onboarding / FOMO（解锁倒计时、空态诱导） | 空态给行动出口 + 能力 banner 如实呈现（事实陈述，不是任务清单） |
| B5 | 假社交 / 排行榜 / 团队活跃度 | Team 层粘性 = 共享 policy 与 Bundle 互认（可复核性），不是活跃度 |
| B6 | **数据锁定式粘性**（导出格式私有化、历史不可迁移） | 全格式导出 + 无锁历史（W5）；离开成本=用户愿意放弃的记忆与纵深，不是带不走的人质；Bundle 可独立阅读是 P0 承诺 |

### 6.3 云生态位冲突与端口类陷阱（【②】§6.2 + 综合）

| 陷阱 | 内容 | 复核条件 / 替代 |
|---|---|---|
| 云 portal 跟随者 | MemBrowse 已占"云端历史+PR diff+门禁"生态位并有一年客户积累；正面仰攻云订阅是弱势开局 | 无复核路径——除非用户主动推翻 ADR-0004。替代=错位：本地优先（数据不出内网）+ 自包含 HTML 证据（可离线归档），这是云订阅模式做不了的形态 |
| GitHub-only 绑定 | 嵌入式公司内网 CI 常见 GitLab/Jenkins；先做官方 Action 再做 CLI 会挡住一批用户 | 替代=CLI 优先级 + "CI 不解析自然语言"纪律保证 CI 无关性；Action/GitLab 模板都是薄壳 |
| 端口=供应链攻击面 | Codecov 2021 Bash uploader 泄露波及 GoDaddy/Atlassian/P&G【②§2.2，强】 | 官方 Action/上传通道必须最小权限、无遥测无上传；local-first 本身是对该风险的差异化回答（设计警戒线，也是卖点） |
| 端口=长期维护承诺 | Action inputs 演进、GitLab 模板腐化、JSON schema 坏契约——每个端口都是新支持面 | 替代=一个 CLI 出口 + 薄壳集成，而非 N 个胖客户端；`--json` schema 一旦发布即公共契约，严格执行版本化 |
| 集成剧场 | demo 好看但非必经的集成不产生粘性（如"又一个导出格式"） | 验收回金句与基线指标：Return intent / repeat usage / real recurring use |
| "先占位后补质量"逻辑 | CubeMX 案例（"锁定不靠质量靠流程位"）的正确读法：FirmwareSight 是信任型窄工具，锁定必须同时经得起 Supported 四门槛——竞争压力不构成放宽 fixture 门禁的理由【综合判断，依据②§1.1+①§2.9】 | 无 |

---

## 7. 与 MVP 的关系

**明确：本轮全部结论不纳入 MVP。**（用户亲自定位 Post-MVP；Active Task=NONE 不变；本文不触发功能创建、不修改任何基线文档。）

边界划分：

1. **MVP 已含的最小形态不因本轮扩大**：CLI 核心命令+稳定 JSON/exit codes（P0 slice 已证明）、四动词工作台、能力 banner、Release Bundle——这些是既有承诺，不是本轮新增。CLI 批次口径（MVP=核心命令 / headless 硬化=产品层 P1 / Action 封装=Growth，【③】§4.1）待 §8-Q1 拍板。
2. **本轮探索的一切增量**（P-1 至 P-11、X1–X5）默认进入"Post-MVP 候选池"，**候选≠承诺**。
3. **未来引入的治理路径（不可绕过）**：
   - 每项先过 **Scope Creep 四问**——注意问②（"是否需要新增协议/硬件/运行时？"）对端口议题杀伤力最大：多数集成接口天然命中，故默认推迟是常态、突破是例外（【①】§4.4）；若 1=否 或 4=是，默认推迟；
   - **开发纪律三问**（属于哪个阶段 / 当前阶段是否需要 / 没有它是否无法验证本阶段假设）——默认不做；
   - 技术基线变更走 **ADR**（P-9 分发通道、P-10 域模型、P-11、任何云面）；新依赖过 FS-TECH-012 / FS-ENG-006 准入清单；
   - 设计条款先行：端口准入三问 + 通知三律 + B1–B6（待 §8-Q8 入册）；
   - Growth 项终极试金石=**四不破坏**：local-first / deterministic core / evidence provenance / explicit unknown（【①】§3.2）；
   - 任何端口必须挂靠四动词，不能成为第五动词（【①】§4.3；【④】§0）。
4. **合法推进路径只有一条**：探索与提案先行，实现让位于阶段门。当前唯一合法下一步仍是 P0 Technical Vertical Slice（与上一轮探索交付一致）。

---

## 8. 待用户拍板的开放问题（四份输入去重汇总，每条附建议）

| # | 问题 | 来源 | 建议 |
|---|---|---|---|
| Q1 | **CLI 三层口径冻结**：MVP=核心命令+稳定 JSON/exit codes；headless 硬化=产品层 P1；Action/GitLab 封装=Growth——是否接受？ | ③§10-1 | 接受，并在 MVP 口径冻结文档中记为一句话（三层是三个批次，不矛盾；影响 P-1/P-9 排期） |
| Q2 | **watch 排期**：接受"工程 spike 可早（P2–P3 间隙）、产品决策放 V1 后"，还是直接列入 MVP 后第一批？ | ③§10-2 | 前者——凭 V1"用户是否真的高频重复分析"信号拍板，避免为想象中的习惯回路预付 M 级成本 |
| Q3 | **watch 设计契约**：默认 Off；Armed 间隔默认 60s=下限、可见可调；触发=History 追加同形态记录+单条系统通知（出厂仅订阅 BLOCK） | ④⑥-1 | 照此采纳，随 History(P1) 动工前写入 DESIGN.md；与 Q2 互不阻塞（契约先定，排期凭信号） |
| Q4 | **Keil/.axf 批次 fixture 来源**：无内部渠道时是否公开征集真实 Keil/ArmClang 工程产物（涉及商业/法务口径）？ | ③§10-3 | 先盘点内部渠道（目标用户访谈对象中的 Keil 用户大概率有真实产物），无渠道再议公开征集；公开征集前过法务口径 |
| Q5 | **config JSON Schema 公开渠道**：私有仓库期间先"随应用分发+文档页"、GA 后再提交 SchemaStore，两步走是否接受？ | ③§10-4 | 接受（SchemaStore 提交须稳定公开 URL，私有仓库期间客观不满足） |
| Q6 | **`.su`（stack usage）是否列入 V1 访谈验证问题清单**？ | ③§10-5 | 列入——与上一轮 xlsx/docx 验证问题合并为同一份访谈清单，一次访谈双份收益 |
| Q7 | **证据分级前缀缩写制**：CI/纯文本用 `[OBS]/[DRV]/[DEC]/[UNK]`，报告场景用全称+图例 | ④⑥-2 | 采纳；同时把"组件版本三词（Inferred）≈ Derived"的对照写进 DESIGN.md ⑥ 备注，消除两套词汇的误读空间 |
| Q8 | **设计条款入册**：端口准入三问 + 通知三律 + B1–B6 行为红线，作为 DESIGN.md ⑨ Do's & Don'ts 行为层增补（或独立 ⑩ 外部端口条款） | ④⑥-4 | 采纳——文档级变更，不触发功能创建，随下一次设计资产升版入册 |
| Q9 | **基准 pin 落位**：pin 动作放 History 行内（仪器概念"固定对照点"，非社交化"收藏"）；pin 后 Compare 的 A 侧默认选中该基准 | ④⑥-3 | 采纳 |
| Q10 | **Persona D 的 CI 时机错配应对**：D 最痛的 CI 集成锚在 Growth，是否需要提前验证？ | ①§6.2（本轮综合提出） | 把"D 类用户对 CI/自动化门禁的需求强度与付费意愿"列为 V1 访谈优先问题；若信号强，CI 试点可按 ③§4.3 提前至 P5——是否提前届时再裁决，现在不冻结 |
| Q11 | **外部调研残留缺口是否补查**：MapView 售价未取到、MemBrowse 免费档细节（需注册探明）、u/OldTap7 原帖永久链接待补 | ②§7 | 不主动补查（注册探明成本高、时效有限；客户墙为商业自述口径，已标注置信度）；仅在 CI 端口临近 Growth 立项时定向补查 MemBrowse 档位明细 |

---

*综合执笔：专业写手 · 2026-09-27 · 本文为探索交付稿，不修改任何基线文档，不触发功能创建；所有事实以四份输入所引基线原文为准。*
