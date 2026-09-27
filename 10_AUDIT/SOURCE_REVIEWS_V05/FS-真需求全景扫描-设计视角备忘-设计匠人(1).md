# FirmwareSight 真需求全景扫描 · 设计视角备忘

> 状态：设计视角探索稿（Post-MVP 议题讨论，非 MVP 承诺；本文不触发功能创建，符合 Active Task = NONE 治理边界）
> 依据：DESIGN.md v1.0（冻结，Instrument-grade clarity）· design-tokens v0.2.0（ADR-0018）· 《背景回顾-已覆盖面与本轮扫描边界清单-通用助手》§1–§5 · 《FS-真需求全景扫描-外部信号报告-信息哨兵》S-01–S-12 · 前两轮设计备忘（判据沿用：准1–3 / T1–T11 视觉陷阱 / D1–D4 防 dashboard 化 / N1–N3 通知三律 / B1–B6 行为红线）
> 范围声明：只回答"每个候选在设计上呈现为什么形态、与冻结基调相不相容、怎么防噪、采不采纳"；升格与否、排期与实现成本以综合环节与技术侧结论为准。
> 作者：设计匠人 · 2026-09-27

---

## 0. 判断基准与一句话总立场

沿用前两轮三条准则评审每个候选形态：

| # | 准则 | 内容 |
|---|---|---|
| 准 1 | 判断准则 | 该形态帮用户回答哪个具体问题？答不出就不进产品 |
| 准 2 | 上下文准则 | 每个数字带上下文（相对谁/占比/最大贡献者/是否超预算，至少占一）；证据分级随值而行 |
| 准 3 | 职权准则 | 图形与表格只呈现事实，不做判决——好坏判断只属于 Gate 规则 |

**一句话总立场：本轮 12 条候选没有一条需要新界面——正确形态全部是"现有四动词界面里的新列、新行、新节、新清单条目"，外加能力 banner 的扩位。** 这与信息哨兵 §6"全部可以塞回四动词链内，无需越过任何一条 ADR"的推演在设计侧独立印证：外部真需求的方向是**证据的完整性与可追溯性**，而证据完整性的 UI 落点（能力 banner、证据分级、Unknown 计数、Gate 五态）在冻结基调里已经全部建成——本轮是往既有插座上接读数，不是造新仪表。

反向警报在此同时立下：**"证据完整性"是比"可视化"更容易 dashboard 化的叙事**——合规语境天然诱惑 coverage 分数、风险灯、进度环。§1 逐条给防噪纪律，§3 给横向三条。

---

## 1. 重点候选逐条评审

每条按 ①UX 形态与落点 ②相容性 ③防噪纪律 ④建议 展开。顺序沿信号强度（S-01 强信号居首）。

### 1.1 S-01 栈安全证据：最坏栈 + 函数指针调用链【强｜佐证升格提请】

**设计立场先行：若综合环节裁决升格，设计侧无条件支持——这是外部信号里判断价值最高、设计成本最低的一条。**

- **① UX 形态（落点：Analyze 列 + 行展开 + Gate 规则 + 能力 banner 扩一位，零新增界面）**
  - puncover#87 的用户原声已经把形态说对了："as a column in the symbol list, sortable"——**逐函数最坏栈就是 Symbols 表的一列**（mono 右对齐、tabular-nums、可排序），不是一张新图。判断问题："这个线程的栈配额够不够 / 栈被谁吃了"。
  - 每列带上下文：`worst stack 1,024 B · quota 2,048 B`（配额来自 policy；无配额时只报事实值，不发明"使用率 %"）。
  - **调用链断链处 = 显式 Unknown**（puncover#105 的痛点与 DESIGN.md ⑥ 条款同构）：行展开显示**缩进式调用路径**（表格族，非图型），函数指针/间接调用处一行 `○ callee unknown (indirect call via function pointer)`——不猜、不把断链装成连续。这与"显式 Unknown"哲学互相成就，是本条与 puncover 拉开代差的气质点。
  - **超限与否由 Gate 说话**：`worst stack > thread stack quota` 是天然 deterministic 规则（Zephyr#27189 的 "flag as issues" 与门禁逻辑完全同构，无 AI 红线无虞），落 Gate 四件套（Result / Why / Evidence / Next）。
  - `.su` 未提供时：整列显示 `—` + UNKNOWN + 事实句（"su file not provided; stack bounds unknown"）+ Next（"re-run with -fstack-usage"）——禁 0 冒充；能力 banner 扩 `.su ✓/○` 一位（UI-01 结构可扩）。
- **② 相容性**
  - 图表优先级链：✅ 第 1 位表格族（表列 + 缩进路径列表），零新图型。
  - 五态与 Unknown：✅ 直接复用——断链 callee=UNKNOWN（空心 ○ + 事实句 + 下一步）、`.su` 缺失=UNKNOWN、超限判定=Gate 五态；不引入新状态词汇。
  - 硬性禁区：✅ 渐变/玻璃拟态/紫蓝/第四色相全部不触碰。
- **③ 防噪纪律**
  - **禁全量调用图可视化**：graph 视图不在四级优先链内，几千节点的"全景调用图"是炫技面——按"拿掉颜色动画、灰度打印还剩什么"口诀判装饰。最坏路径用缩进列表，链上节点可点回 Symbols 行（图即索引的表格版）。
  - 禁整列红染、禁"栈使用率 %"大数字——超没超由 Gate 规则说，表只报事实（准 3）。
  - 列默认显示且可排序（puncover 的教训正是"点进单函数才可见"不成立）；但**不做默认按栈排序**——改变既有默认序是替用户预设关注点，排序是用户动作。
- **④ 建议：采纳（设计侧支持按《边界清单》§1.3 提请升格为 P1/P2 候选排期讨论）。** 设计成本 = 一列 + 行展开 + 一条规则模板 + banner 扩位；puncover 维护真空 + 三源同构的窗口值得抓住。升格与否由综合环节裁决，本文只固定"若升格，形态如上"。

### 1.2 S-05 组件版本漂移检测（H8）【中｜候选验证，须与 P2 严格区分】

- **① UX 形态（落点：Compare 新增 Components 节 + Gate 一条规则，零新增界面）**
  - 与 P2 的区分在设计上很干净：P2 = 同一 artifact 维度的 **size diff**（section/symbol Δ）；S-05 = **组件实体跨 build 的版本对比**（"mbedTLS 3.4.0 → 3.6.1 且没人记录"）。呈现 = Compare 视图新增 **Components 节**：一张表 `component / build A version / build B version / drift kind / evidence grade`——Diff Row 的 old/new 同列语义直接扩列。
  - 关键呈现细节：
    - 版本变更行用 `diff.changed.bg`（蓝，changed 语义）；**升级与降级同色同待遇**——"版本变了"是事实，"变好变坏"不由表格说（Δ 不用红绿的组件版延伸）；
    - **证据等级漂移也是漂移**：`3.4.0 [OBS] → unknown ○ [UNK]` 配事实句（"Observed in build A, not detected in build B"）——证据退化比版本升级更值得看见，这是显式 Unknown 哲学的直接用武之地；
    - 每个版本号必须带分级前缀（`[OBS]/[DRV]/[DEC]/[UNK]`）——没有分级的版本对比会诱读成"确定事实"（准 2）；
    - 默认只列**变化的组件**，未变化折叠为一行计数（`31 components unchanged`）——dense not crowded 的变更版。
  - Gate 落点：`component version changed without declared note → REVIEW`（deterministic 规则模板）。
- **② 相容性**：✅ 表格第 1 位；✅ `diff.changed.bg` token 已有（v0.2.0）；✅ 五态直接适用；✅ 零新增图型——不需要任何"依赖关系图"。
- **③ 防噪纪律**
  - 禁 Dependabot 式呈现：不做彩色 alert 徽章阵列；不做"落后 N 个版本 / 过期 N 天"——后者需要联网 advisory feed（撞 ADR-0013 网络红线），且"过期"是替用户判决（准 3）。
  - 禁升级↑绿 / 降级↓红的箭头色编：方向用文字 + mono 表达（`3.4.0 → 3.6.1`）。
  - 不做"依赖健康面板"（D1 不聚合）：漂移信息只住在 Compare 节与 Gate 结果里，Overview 不吸收。
- **④ 建议：采纳。** 信息哨兵判"性价比最高"，设计侧从形态成本独立验证：数据模型在 P1 Component Evidence 已有，呈现 = Compare 一张表 + 一条规则。若排期要挤，它是本轮候选里最值得先做的一条。

### 1.3 S-04 SBOM evidence completeness / retention（H3/H4）【中｜CRA 窗口激活】

- **① UX 形态（落点：Analyze 组件表扩列 + History/Bundle 既有结构 + Settings 单行，零新增界面）**
  - **H3（completeness）的正确形态是"证据分级计数行"，不是完备性分数**：组件表/摘要行给 `24 observed · 5 inferred (from version.h) · 2 declared · 1 unknown` 五级计数——可数、可复核、不伪装精确。做"SBOM completeness 82%"就是 T3 健康分换皮，直接排除。
  - **H4（retention）用户要判断的只有两件事**："这个 build 当时的证据还在吗 / 归档里带了什么"。前者 = History 行内的 evidence-integrity 指示（对 Bundle hash 做复验，复验结果用既有五态或 meta 文字呈现，不发明新色）；后者 = Bundle 清单本身（SBOM / analysis / manifest 已在 P4 清单结构内）。**不新增 retention 管理界面**；保留策略（保留多久 / 自动清理）落 Project Settings 单行配置——Settings 单一入口纪律沿用。
- **② 相容性**：✅ 计数行 = 文字（meta 字号），非图型；✅ History 行结构已有（W2/W3 已立）；⚠️ 不建议为"各证据等级组件数"上 ranked bars——五级证据源是**分类不是排序**，纯计数行比图形更克制也更诚实。
- **③ 防噪纪律**
  - 禁 coverage donut / 进度环 / 百分比大数字（T2/T3 点名）。
  - 禁"合规进度"叙事：界面文案只说事实（`CRA reporting period: evidence retained per build`）；**compliant / audit-ready 是禁词**（07_COMPLIANCE）；R4 边界：工程证据 ≠ 法律结论，界面永不出现"合规评分"。
  - Unknown 计数在摘要行与报告中永不淡化为脚注或折叠（前序备忘 3.2-3 已立；H3 计数行正是它的数据来源之一）。
- **④ 建议：改造采纳。** CRA 时间窗是真推力，但合规焦虑恰恰最容易催生假精确——设计侧把 H3 压到"计数行 + 分级列"、把 H4 压到"History 复验 + Bundle 归档 + Settings 单行"，宁窄勿宽。H3 的证据分级计数比 H2（SPDX 导出）更优先的判断，设计侧附议。

### 1.4 S-06 发布物符号档案（version → symbol file 归档）【中｜新方向】

- **① UX 形态（落点：Release/Bundle 清单条目 + manifest 字段，零新增界面）**
  - Bundle 清单增可选条目：`symbols: firmware.elf · SHA-256 3f2a…`（mono hash 惯例已有）；release-manifest 增 version→hash 映射字段。判断问题："v1.2.3 发布时的符号文件在哪、hash 是多少"——**Keep the evidence 支柱的直接延伸**，署名行式元数据（哪个版本 / 何时 / 哪个 hash）。
  - **不做"符号档案管理页"**：版本↔文件映射若独立成界面就是冗余——History 已按版本组织一切（W3），Bundle 已是归档物本身。信息哨兵"零新增解析器、纯 Bundle 扩列"的判断，在设计侧对应"零新增界面、纯清单条目"。
- **② 相容性**：✅ 纯清单条目（UI-04 右栏结构已落）；零图型、零新色、零导航变化；Bundle"脱离 FirmwareSight 独立可读"的 P0 承诺不受影响（多一个自描述文件）。
- **③ 防噪纪律**：几乎无 dashboard 风险；唯一要防的是**云符号库上传**的形态诱惑（nRF Cloud / Memfault 模式）——上传撞 ADR-0004（local-first）与 ADR-0013（network client deferred），本地归档形态才合法；界面不出现 "Sync / Synced" 类过程态词汇。
- **④ 建议：采纳（P4 并车项，挂 V1 访谈验证 crash post-mortem 场景是否真实存在）。** 设计成本 ≈ 0，是本轮新方向里形态最干净的一条。

### 1.5 S-07 组件 ABI 兼容性检查【中｜新方向】

- **① UX 形态（落点：Compare 组件节扩展表 + Gate 规则，零新增界面；排期属 P3+）**
  - 形态本质 = S-05 的深度版：两 build 间的**类型/签名变更表**（`type·symbol / A layout / B layout / change kind`），字段级行呈现 `field / offset A / offset B / size`，偏移量 mono 列；Diff Row 的 old/new 同列语义直接适用。
  - Gate 落点：`public header ABI break without declaration → REVIEW/BLOCK`（deterministic，规则模板与栈规则同族）。
  - 变更分类用**文字标签**（removed / layout-changed / signature-changed），不用彩色徽章。
- **② 相容性**：✅ 表格族、✅ Gate 四件套、✅ 证据分级随行（DWARF 实测 = Observed，头文件声明 = Declared，与 ⑥ 三词一致）。
- **③ 防噪纪律**
  - **禁内存布局画图**：struct 字节布局图示看起来"直观"，但它是优先级链外的新图型，且信息密度低于字段表（黑白打印更是灾难）。
  - 禁 cargo-semver-checks 式 breaking/major/minor 彩色徽章阵列——"BREAKING"红徽章 = 预支判决，破坏性认定只属于 Gate 规则（准 3）。
  - 禁"兼容性评分"（T4 同族）。
- **④ 建议：改造采纳（P3+ 候选）。** 方向真（cargo-semver-checks 已验证该品类可成为开发者离不开的门禁），但 DWARF 深解析成本高、呈现纪律必须**预先**立下（本节三条即预案）。建议综合环节排在 S-05 验证之后再议。

---

## 2. 其余信号快判（S-02 / S-03 / S-08 / S-09 / S-10 / S-11 / S-12）

| # | 信号 | 形态与落点（①） | 相容性（②） | 防噪要点（③） | 建议（④） |
|---|---|---|---|---|---|
| S-02 | CRA→SBOM（P1 佐证） | SBOM 是**输出物不是浏览对象**：CycloneDX JSON 作为 Bundle 清单条目；组件呈现继续住在 Analyze 组件表。**不做 SBOM 树状浏览界面** | ✅ | 机器可读条目不做 UI 树状渲染；禁"SBOM 覆盖率"叙事 | 采纳（既有 P1 排期，设计侧无保留） |
| S-03 | license 风险提示（H9） | 组件表增 license 列（文字 + 分级随行，如 `license: GPL-2.0 (Inferred from source header)`）；Gate 规则模板（如"GPL 组件进入闭源固件 → REVIEW"） | ✅ 规则 deterministic 可行 | **R4 红线**：只陈述 `license: X (来源)`，禁 risk 红黄绿灯、禁 "compliance risk" 措辞——提示 ≠ 法律结论 | 改造采纳（搭 SBOM P1 便车，预留 license 维度即可，不独立立项） |
| S-08 | 构建/工具链指纹 | specimen header / Identity 区增字段（toolchain 版本、flags、源 revision、环境指纹）——前序备忘 3.2-1 结构已预留 | ✅ 零新面 | 无仪表盘风险；字段多按 meta 折行，不做成"指纹卡片" | 采纳（P-8 保留产出的外部佐证，Identity 增列 + manifest 同步） |
| S-09 | OTA 元数据 | 若 V1 访谈强现：Bundle 可选节（字段照 MCUboot header 清单），纯清单条目 | ✅ 窄但合规 | 《边界清单》§3.8 高危擦边：过 Scope Creep 四问再议 | **维持不做**（设计侧同意默认立场） |
| S-10 | changelog 生成（H1） | 若未来与 S-05/S-06 联动："汇编草稿"形态 = Release 视图内草稿文本块，每句后跟 build-id / evidence 指针 | ✅ | 外部弱信号：平台已吞掉通用需求；禁 AI 润色（ADR-0005） | **维持"如有"口径**（不升格，与信息哨兵一致） |
| S-11 | 二进制/分区级 diff | 字节级 diff 的自然呈现（hex view / 热力图）全部出界：hex view 是逆向工具的视觉语言，热力图不在图表优先级链 | ❌ 近禁区 | Non-goals #5：防产品叙事滑向逆向工具——设计侧补充排除理由：**该方向的呈现形态本身就出界** | **排除**（附议信息哨兵"不立项"） |
| S-12 | 解析适配池 | 每新增格式的唯一 UI 义务 = 能力 banner 如实标注证据等级（前序备忘 1.5 已立） | ✅ | 禁把"支持格式数"当营销点（支持矩阵说话，R2） | 佐证池，知悉 |

---

## 3. 横向结论（给综合环节的三条设计侧总判断）

**3.1 零新增界面、零新增图型。** 12 条候选全部落在四动词界面内：Analyze 增列（S-01/03/12）、Compare 增节（S-05/07；S-11 为排除项）、Gate 增规则模板（S-01/03/05/07）、Release/Bundle 增条目（S-02/06/09）、History 增指示（S-04）、能力 banner 扩位（S-01/12）。冻结的导航结构与图表优先级链**无一需要松动**——这是本轮与"端口扩展"轮最大的不同：没有接线问题，只有读数问题。

**3.2 "证据完整性"叙事的统一 UI 落点 = 能力 banner + 证据分级 + Unknown 计数 + Gate 五态。** 所有合规向候选（S-02/03/04/08）共用这四个既有件，**不新设"合规面板"**。配套一条扩位纪律建议入册：**能力 banner 一源一位**——每个证据源一个槽位（✓/○ 如实标注），禁止膨胀成图例墙；banner 是"证据在不在"的单一真相，不是功能 bragging 面。

**3.3 所有"变更"类呈现共用三纪律**（建议随下一次设计资产升版写入 DESIGN.md ⑤/⑥ 增补，本轮触发源：S-05/S-07）：
1. **变更表在前、判定在 Gate**——表格只报事实（changed 蓝底 + mono 值），禁红绿箭头、禁 breaking 徽章、禁风险灯；
2. **证据分级随值而行**——版本号/签名/布局每个值带 `[OBS]/[DRV]/[DEC]/[UNK]`，无分级的对比不合法；
3. **未变即折叠**——变更列表默认只列 delta，未变项一行计数（dense not crowded 的变更版）。

---

## 4. 留给用户/管理员拍板的三件事

1. **能力 banner 扩位制**：接受"一源一位"扩位纪律写入 DESIGN.md（本轮触发源：`.su` 位、SBOM/license 维度、retention 复验位）。我的建议：接受。
2. **变更三纪律入册**：§3.3 三条随下一次设计资产升版增补进 DESIGN.md ⑤ Diff Row 条款与 ⑥ 状态条款。我的建议：接受（S-05/S-07 动工前完成即可，属文档级变更）。
3. **S-01 升格形态预认**：若综合环节裁决将 `.su` 从"访谈验证"升格为 P1/P2 候选，设计侧按 §1.1 形态直接执行（Symbols 列 + 缩进调用路径 + Gate 规则模板 + banner 扩位），无需再过一轮设计评审。我的建议：预认，省一轮往返。

---

*FirmwareSight 真需求全景扫描·设计视角备忘 · v1.0 · 2026-09-27 · 设计匠人*
*本文为探索稿：不修改任何基线文档；若采纳 §4 裁决，随下一次设计资产升版正式入册。*
