---
title: "FirmwareSight 正式产品功能探索：分析结果可视化与文档导出（Post-MVP）"
product: "FirmwareSight"
version: "1.0"
date: "2026-09-27"
authored_by: "专业写手（综合执笔）"
status: "FINAL — 综合交付稿 · 本轮全部结论不纳入 MVP，不改变 Active Task 状态"
inputs:
  - "通用助手《背景回顾：可视化与文档导出的基线事实清单》（60 条事实 + HC/ES 标注）"
  - "信息哨兵《可视化与文档导出·外部调研报告》（27 信源 + 强/弱/无信号分级）"
  - "鲁班七号《FS-可视化与文档导出-技术可行性简报》（技术路径 + S/M/L 分级 + 不值得做清单）"
  - "设计匠人《FS-可视化与文档导出-设计视角备忘》（A1–A8 评审 + 11 陷阱清单 + 仪器打印件主张）"
---

# FirmwareSight 正式产品功能探索：分析结果可视化与文档导出（Post-MVP）

> **阅读前提**：两个议题由用户亲自定位为 Post-MVP 探索。本文不改变治理状态——当前唯一合法下一步仍是 P0 Technical Vertical Slice。本文回答的是"未来要不要做、做成什么、明确不做什么"，不是"现在做"。
>
> **方法声明**：结论综合自四份输入（上方 frontmatter 列出，群内可查），每条结论标注置信度与依据来源。遵循语调三律：说事实不做判决、说未知不填空（所有"未检索到"如实标注，**缺失不等于不存在**）、给下一步。

---

## 1. 议题定义与结论先行

### 1.1 议题原文

用户提出（2026-09-27）："每一次分析完成之后在软件中是否能够直观的可视化的看到结果呢？另外……我们貌似没有能够给用户保存与输出文档之类的，比如输出 md/docx/excel/html 等文档，你说说看加了这个功能之后是否对使用者会更好呢？"——并亲自定位：本轮为正式产品功能探索，不纳入当前 MVP。

### 1.2 结论先行：对使用者是否更好？

**议题① 分析后应用内可视化 —— 会更好，净改善成立。（真需求，高置信度 4/5）**

三条事实链支撑：

1. **需求真实且长期存在**：从 2009 年 Stack Overflow 求助帖、2015 年工程师自造 GUI 工具（embeddedrelated #900，24 条评论），到 STM32CubeIDE 把 Build/Stack Analyzer 做成官方标配、Zephyr 官方收录 puncover——同类工具被独立重造至少 7 次以上，横跨 17 年。（信息哨兵 §1，置信度 4-5 [F]）
2. **但价值在形态，不在"有图"**：主流受欢迎的形态是 **ranked 表格/列表**（Bloaty、Puncover、cargo-bloat 的核心全是"按大小排序、可下钻的列表"），treemap 等图形是补充而非决胜点。工程师要的是"谁吃了我的 Flash"的可执行答案。（信息哨兵 §1.2 [O]）
3. **独立价值必须避开 IDE 内置墙**：STM32CubeIDE/ESP-IDF/Segger 都已免费内置单次查看能力。FirmwareSight 可视化的净改善只能来自 IDE 做不到的事：**跨 build 比较（delta）、Gate 证据总览、发布证据链**——这与基线市场判断"viewer 是入口不是壁垒"一致。（信息哨兵 §1.2；基线 00_MARKET_EVIDENCE）

对使用者的净改善 = **更快的判断**（占用排行 + 预算上下文 + symbol 下钻 + diff 贡献者），前提是不滑向营销 dashboard（见 §6.2 陷阱清单）。

**议题② 文档导出 —— 按格式分层判断，"加了导出"本身不自动等于更好。**

| 格式 | 判断 | 置信度 | 核心依据（来源见 §4） |
|---|---|---|---|
| JSON/CSV（机器可读） | ✅ 真需求（强） | 4/5 | puncover 官方新增 JSON 报告（CI 场景）；Bloaty `--csv` + diff 官方定位"perfect for CI tests" |
| 自包含 HTML（单文件） | ✅ 真需求（中强） | 4/5 | pytest-html/Lighthouse/coverage.py 自包含报告已成业界惯例；嵌入式 size 域该形态是空位；且"Bundle 脱离 FirmwareSight 独立可读"本就是 P0 承诺 |
| Markdown | ⚠️ 弱需求（场景限定） | 3/5 | CI/PR/wiki 摘要场景惯例，未见作为最终交付物的证据；作为正式交付物证据不足，作为轻量附赠合理 |
| Excel (xlsx) | ❌ 倾向伪需求 | 2/5 | 27 信源检索范围内**无任何同类工具提供原生 xlsx**、无社区直接讨论；CSV 已覆盖"Excel 可打开"路径。**唯一保留项**：若目标客户含国内制造业甲方（Excel 交付文化）需访谈验证 |
| docx | ❌ 倾向伪需求 | 2/5 | 零直接证据；质量/测试类工具行业无先例（无一选 docx 作报告形态）；HTML+打印已覆盖其大部分受众 |
| PDF（原生排版） | ❌ 不做 | — | 非信号问题，是承诺问题：PDF 无法逐字节复现，与产品"所有输出可复现"基因冲突；HTML 打印视图以 S 级成本承接打印需求 |

对使用者的净改善 = **复核者场景被满足**（QA 把可独立阅读的报告发给另一人复核——US-004 定义的导出第一读者），由 HTML/JSON/CSV 承接；xlsx/docx 在获得访谈证据之前，做了大概率不增加使用价值，反而增加依赖与维护面。

**共同限定（商业事实）**：静态 size 可视化工具几乎全部免费开源；付费集中在运行时 trace 可视化（Tracealyzer、SystemView €1,480）；未发现任何"报告/导出"单独收费的先例。可视化与导出的商业角色是**入口与留存**，不是付费墙——与基线商业判断"真正可收费的不是 treemap，而是 release risk reduction / evidence / CI / team policy"一致。（信息哨兵 §4；基线 05_BUSINESS_MODEL §4）

### 1.3 一句话总回答

> 会更好——但只在前四个格子里：**表格化的可视化 + HTML/JSON/CSV 导出**有 17 年需求史与业界惯例背书；**Excel 与 docx 在证据到位之前应留在纸面**。且这一切属于 P1 及以后：MVP 不因此扩大一寸。

---

## 2. 背景回顾摘要（引用，不重提）

完整 60 条事实及逐条【硬约束/可扩展空间】标注见：通用助手《背景回顾-可视化与文档导出的基线事实清单-通用助手.md》（shared/firmwaresight/）。与本文结论直接相关的锚点：

| 主题 | 既有事实（出处见原文件对应编号） | 性质 |
|---|---|---|
| 导出既有承诺 | Release Bundle 7 类内容（含 release-report.html / analysis.json / diff.json）；Diff 可导出 JSON/HTML（US-002）；SBOM 导出 P1/P2；CLI exit code 6 = export error（事实清单 §2.1） | 硬约束 |
| 导出品质基准 | Bundle 脱离 FirmwareSight 独立可读；manifest 全文件 hash；导出前 Preview、不覆盖已有目录、失败不留假完整（§2.2） | 硬约束 |
| 导出生产者 | report crate = 唯一报告适配器（ReportRenderer 端口：versioned JSON + 自包含 HTML）；P0 占位、工程 P4 完整实现；golden 机制已规划（§2.3） | 硬约束 + 扩展空间 |
| 措辞红线 | 每条导出结论必须引用 evidence/policy；禁 guaranteed/compliant/certified 等禁词；语调三律适用于一切导出物（§2.4） | 硬约束 |
| 可视化既有约定 | 图表优先级 **table > ranked bars > compact stacked bar > treemap**（treemap 仅限 size 构成）；chart.seq 五档蓝阶唯一真源；渐变全禁、阴影仅浮层；状态 5 态权威；Unknown 中性灰一等状态；动效 120/150/180ms 三档（§3.1–3.2） | 硬约束 |
| 可视化技术边界 | ADR-0011：MVP no chart framework；无 wgpu，SVG/Canvas/虚拟化阶梯在前、GPU Island 在后且须 ADR；100k symbols 预算；IPC 永不整体传输 symbol 表（§4.1） | 硬约束 |
| 排期锚点 | P1 已列 "richer treemap" + "HTML share report" + History memory trends；工程 P4 是 report crate 完整实现窗口（§2.1.8、§4.2） | 扩展空间 |
| 产品边界 | Non-goals 八项；Scope Creep 四问；四动词纪律；R6 功能堆砌风险；"可收费的不是 treemap"；Growth 功能不得破坏 local-first/deterministic/evidence/unknown 四红线（§5） | 硬约束 |
| 关键空白 | **md/docx/xlsx/pdf 四类格式基线零约定**——无承诺亦无禁令，属纯新增决策，须过依赖七问与（若成公共契约）版本化 schema + ADR（§2.5） | 扩展空间 |

---

## 3. 真实需求提取：可视化功能清单

综合设计侧 A1–A8 评审（设计匠人 §①）、外部需求信号（信息哨兵 §1/§5）、技术成本（鲁班七号 §2）。排序按"对判断的价值 × 证据强度"。

| # | 形态 | 帮助的判断（价值） | 需求信号 | 设计相容 | 技术成本 | 建议阶段 |
|---|---|---|---|---|---|---|
| V0 | 能力 banner / Evidence availability（ELF ✓ · MAP ○ · Git ○） | 这份分析可信到什么程度（**极高**；证据残缺时任何构成图都是误导） | 设计侧主张；Evidence-first 体系的门面 | ✅ 完全符合（Unknown 一等状态） | 已落地（P0 UI 稿 FS-UI-01） | **P0 已落地** |
| V1 | Section/FLASH/RAM 占用 ranked bars + 预算线（表格内嵌） | Flash 装不装得下 / 被谁吃了（**高**，全产品性价比最高的一张图） | 工具演进共识：Bloaty/Puncover/cargo-bloat 主形态 | ✅ 优先级链第 2 位；预算线用 border 不用红染 | **S**（FS-UI-02 已定形态） | **P0 最小版已有 → P1 完整** |
| V2 | Symbol 级 Top-N 下钻表（行展开 + 面板，终点 = symbol + Evidence） | 这 12 KB 具体是哪个函数（**高**；仅 Top-N 形态下高） | 同上；"可下钻列表"是 17 年共识形态 | ✅ 纯 table + bars 组合 | **S–M**（含虚拟化 react-virtual ≈4KB） | **P1** |
| V3 | Compare delta：Diff Row 表格（old/new/Δ 同列）+ Biggest growth contributors 面板 | 这次比上次变了多少、谁贡献的（**极高**，差异化核心；P0 验收已写死禁只给 "+8KB"） | 产品差异化核心；跨 build 比较正是独立工具避开 IDE 内置墙的价值所在 | ✅ Δ 不用红绿、added/removed 禁 0 冒充 | **S–M**（P0 最小版已有 → P1 面板完整） | **P0 + P1** |
| V4 | Compact stacked bar（FLASH/RAM 构成并排） | 单屏看双区构成（中高；A1 的紧凑变体，适合 Overview 与导出报告） | 形态互补信号 | ✅ 优先级链第 3 位；分段 ≤5 + 1px 隔断 + mono 数值 | **S–M** | **P1** |
| V5 | Treemap（size 构成 + 下钻 + **必须与表格双向联动**） | 嵌套空间构成 / 找最大可优化对象（条件性高；做得好是仪器，做得差是营销图） | 少数派但有生态位：走图形路线的工具均未成为头部（信息哨兵 §1.2） | ✅ 三重约束下：仅 size 场景、蓝阶非彩虹、双向联动否则不进产品 | **M**（d3-hierarchy ≈10KB + SVG；leaf ≤500 聚合） | **P1**（PRD 已列 richer treemap） |
| V6 | Gate 结果总览：五态分组清单（BLOCK 在前）+ 计数 pill | 现在有没有 BLOCK / 哪几条要看（**极高**；Gate 是分类事实不是分数） | Gate 是 P0 核心能力，总览即其最优呈现 | ✅ **零新增图型**；明确拒绝 gauge/donut/评分 | 已落地（FS-UI-04） | **P0 已落地** |
| V7 | 跨 build 趋势：带基线的柱列 + 预算线（时间序） | 最近十次 build 内存是不是一直在涨（真实但危险——最易滑向 dashboard） | PRD P1 History "memory trends"；工程师"向他人呈现结果"场景存在（弱信号） | ⚠️ 需一次条款增补：时间序列三纪律（含 0 基线 / 禁重排 / 禁双 Y 轴），建议随下次设计资产升版入册 | **S**（sparkline 版）→ **M–L**（联动图） | **P1**（History 动工时）；重量级联动 Post-GA |

**实现铁律（全形态通用，来源：设计备忘 + 技术简报）**：不引图表库（自绘 SVG/CSS + 单点微依赖）；大数据三道闸门 = Core 侧聚合 → cursor 分页 → 前端虚拟化兜底；颜色一律读 tokens v0.2.0 CSS 变量；每根 bar 带 mono 数值标注；UI 不得重算 core 事实。

---

## 4. 导出功能清单

综合技术路径（鲁班七号 §3：一个报告 IR + 多 renderer，全部挂 report crate 既有 ReportRenderer 端口，不新增 crate）、外部信号（信息哨兵 §2/§5）、设计主张（设计匠人 §3："仪器打印件"）。

| 格式 | 使用者价值（谁在什么场景用） | 需求信号 | 实现成本 | 建议阶段 |
|---|---|---|---|---|
| JSON（analysis / diff / release-manifest） | CI 消费、机器可读、版本化公共契约 | **基线已承诺**（Bundle 成员 + CLI）；puncover 官方加 JSON 报告、Bloaty diff for CI 为外部佐证 | 既有 | **P0 已有**（工程 P4 完整） |
| 自包含 HTML（单文件，内联 CSS、零/极小 JS） | **主格式**。QA 发复核者、离线归档、脱离 FirmwareSight 独立阅读 | pytest-html "self-contained report"、Lighthouse、coverage.py 惯例成熟；嵌入式 size 域空位；Bundle 独立可读是既有 P0 要求 | S→M（P0 Bundle 最小版 → P1 增强 + 内联 SVG 图表） | **P0 最小版 → P1 增强**（PRD 已列 HTML share report） |
| CSV（按主题分表：sections/symbols/diff/gate_results） | 给 Excel 的原始表；Persona C（QA/合规）取数 | Bloaty `--csv`；机器可读导出有直接工具先例 | **S**（UTF-8 BOM、added/removed 空单元格禁 0 冒充等裁决进 golden） | **P1** |
| Markdown | 贴 issue/PR/wiki/评审纪要 | CI/PR 摘要惯例（弱信号，置信度 3） | **S**（与 HTML 同一 IR、同构模板） | **P1** |
| XLSX（rust_xlsxwriter，纯 Rust） | Persona C 汇总、Team 审计场景（商业假设存在） | **零直接信号**（倾向伪，见 §6.1 复核条件）；同类工具无一原生提供 | **M**（含确定性 spike：docProps 时间戳 + zip 成员序） | **P1 末–P2 初，且以访谈信号为前置条件** |
| DOCX（docx-rs，纯 Rust） | 无明确定义场景 | 零信号（见 §6.1） | **M–L**（生成易、做出"仪器打印件"级排版体系才是成本） | **P2 可选，仅出现明确付费信号后** |
| PDF（打印视图：HTML + @media print） | 打印/归档 | 无直接证据；由 HTML 打印承接成本最低 | **S**（+0.5–1 人日） | **Post-GA 可选**；原生排版 PDF 永不承诺 |
| CycloneDX SBOM | 合规准备（CycloneDX 1.7 JSON → SPDX 3.0 后续） | 基线既定（PRD P1 + 合规文档） | **S**（同一 IR 通道） | **P1**（PRD 既定，非本轮新增） |

**"仪器打印件"视觉主张（导出物设计基准，来源：设计匠人 §③，适用于 HTML/MD 及一切打印派生）**：单栏审计文档 + 标本头（hash/commit/toolchain 铭牌）；证据分级随值而行（Observed/Derived/Declared/Unknown 标注，黑白打印不丢义）；Unknown 独立节可见、可数、可行动，绝不淡化为脚注；与 in-app 同一 token 真源；文末 Reproduction 脚注；节序固定 = 先验明正身 → 再看结论 → 再对证据。

**确定性铁律（全格式，来源：技术简报 §3.3）**：内容 = 快照的纯函数（禁导出时刻 now()）；排序全显式；容器级确定性（zip 成员序/固定 mtime，XLSX/DOCX 需 spike）；每格式 ≥2 套 golden 样张逐字节校验；完全离线、零系统 Office 依赖。

**P1 排期权重建议（技术侧，鲁班七号 §7.1，供未来参考非现期承诺）**：虚拟化表格 + ranked bars → HTML 增强 → CSV/MD → treemap → stacked bar → XLSX（先保判断力，再保可携带性）。

---

## 5. 开放脑洞区（纯探索性质，均无当期证据与排期）

> 本节所有条目**仅为脑洞登记，不构成建议承诺**。任何一条未来推进前，须先获得需求信号，再过 Scope Creep 四问 + ADR。列在此处的目的：防止好想法丢失，同时防止它们未经评审就溜进排期。

| # | 脑洞 | 可能的价值 | 现实约束 / 前置条件 |
|---|---|---|---|
| B1 | 跨 build 趋势深化：多 build 对比缩放联动、>10 万点时序、多图层交互 | Pro 档 History 的纵深 | 自绘无法经济覆盖时才触发"引入图表库" ADR（候选择序：Visx > ECharts 按需）；PRD P1 仅承诺 memory trends 基础版 |
| B2 | CI 深度集成：JSON 报告 → CI 摘要注释 / size budget 检查入 CI | CI 是基线商业模式中 Team 档卖点；Bloaty 官方定位 diff "perfect for CI tests" 为佐证 | CLI 归属口径尚待冻结（群内已知待澄清点）；team 能力前置门槛 = 20 访谈/8 试用/3 付费意愿/1 试点 |
| B3 | 报告订阅 / 定期快照：周期性生成 trend 报告 | 长周期项目的内存回归可见性 | 与 local-first 边界紧张（"订阅"隐含后台机制）；Growth 红线：不得破坏 deterministic/evidence；无任何外部信号 |
| B4 | 团队共享：shared policy / audit history / team export | 商业模式 Team 档已有锚点（05_BUSINESS_MODEL §1） | 必须先过付费验证门槛；"导出云端化"方向被基线 Growth 红线封顶——共享指文件/策略，不是云服务 |
| B5 | 趋势预算告警（事实句式）："FLASH exceeded budget on build #N" | 把趋势数据变成可行动信号 | **设计红线**：只能事实句，不得变"你的版本变差了"式判决（语调三律）；Gate 规则才是判决唯一来源 |
| B6 | 报告模板自定义（用户选节选/品牌头） | 企业场景个性化 | 与"确定性优先于美观"冲突，模板自由度直接攻击 golden 机制；优先级最低 |

---

## 6. 伪需求 / 陷阱排除清单

### 6.1 伪需求初判与复核条件（说未知不填空）

| 项 | 初判 | 置信度 | 判断依据 | **复核条件（满足才重启）** |
|---|---|---|---|---|
| Excel (xlsx) | 倾向伪需求 | 2/5 | ① 27 信源内无任何同类工具原生 xlsx；② 社区无"Excel 汇总固件体积"直接讨论；③ CSV 已覆盖"Excel 可打开"路径 | ① 用户访谈（建议 5–8 名目标用户，信息哨兵建议；可并入 MVP 阶段付费意愿访谈）中"你拿到分析结果交给谁、用什么格式"出现 xlsx 明确信号；② 或目标客户画像确认含国内制造业甲方（Excel 交付文化）——此时按 P1 末–P2 初排入，并先做确定性 spike |
| docx | 倾向伪需求 | 2/5 | ① 零直接证据；② 质量/测试类工具行业无 docx 报告先例；③ HTML + 系统打印已覆盖"要 Word 的人"的大部分真实路径（浏览器/Word 均可另存） | 出现明确付费信号（与付费验证门槛同一套口径）后，先花 ≤1 人日 spike docx-rs 表格样式能力，再决定 P2 是否排入 |
| 原生排版 PDF | 不做 | — | 非信号缺失，是**承诺不可能**：字体/分页点随环境漂移，做不到逐字节复现，与"所有输出可复现"基因冲突；typst/Chromium 路线另爆安装包体积 | 无。Post-GA 仅存"HTML 打印视图"一条低成本口子（S 级，非逐字节确定，明确不承诺归档用途——归档用自包含 HTML） |
| 用户原话中的"excel/docx"枚举 | **注意**：格式枚举 ≠ 格式需求 | — | 用户原话列举了 md/docx/excel/html，这是对"输出文档"的能力枚举，不是四个已验证的格式需求。真实诉求大概率是"能把结果拿给别人看 / 存档"——该诉求由 HTML/CSV/MD 以更低成本满足 | 与用户确认具体场景（谁看、什么场合、现在手工怎么做）——正是访谈问题清单的第一条 |

**一个诚实的标注**：信息哨兵明确报告，"给老板/客户/审核方看固件体积报告"的**直接原帖未检索到**（Reddit 反爬加剧缺口），"Excel 汇总"同样未检索到。**缺失 ≠ 不存在**——这正是本节所有"初判伪"都必须挂复核条件、而非直接判死的原因。

### 6.2 营销 dashboard 陷阱（点名排除，来源：设计匠人 §②，11 条全录于原文件）

高危七项速览（完整版含"为什么诱人→为什么错→替代"三段式，见设计备忘原文）：

| 陷阱 | 替代 |
|---|---|
| KPI 大数字卡阵列 | 摘要行：mono 数字 + 占比 + 环比上下文 |
| Donut/环形占比 | ranked bars + 占比列 |
| Gauge/进度环、"Release readiness 82%" | Gate 五态分组 + 计数 pill（分类事实，不是分数） |
| 健康分/单一风险分 | 永不出现——多维证据不可压缩为一个数字而不撒谎 |
| 彩虹分类色（含彩虹 treemap） | chart.seq 五档蓝阶 + chart.rest；状态永远 icon+label |
| 伪 sparkline（无基线） | 真趋势 = 带预算线的柱列（V7） |
| 数据大屏 / 表演性动效 | 仪器工作台；动效仅 120/150/180ms 功能性三档 |

**鉴别口诀**：这张图拿掉颜色、拿掉动画、灰度打印之后，还剩下什么判断价值？剩下的部分才值得做。

### 6.3 技术负面清单（明确不值得做，来源：鲁班七号 §5）

原生排版 PDF（printpdf/typst/Chromium 任一路线）｜PPTX｜RTF/ODF｜带公式/透视表的"活" XLSX｜图表 PNG 光栅化导出｜前端本地生成导出文件（违反"UI 不复刻核心逻辑"红线，作为反模式写明）｜自研二进制报告格式。

---

## 7. 与 MVP 的关系

**明确：本轮全部结论不纳入 MVP。**（用户亲自定位 Post-MVP；基线一致——MVP Exit Criteria 明确不要求 complex data viz。）

边界划分：

1. **MVP 已含的最小形态不因本轮而扩大**：表格 + 最小 ranked bars（P0 验收口径：section 级占用 + 最大贡献者）、Diff Row（old/new/Δ 同列）、Gate 五态分组、Release Bundle（含 release-report.html / analysis.json / diff.json）。这些是既有承诺，不是本轮新增。
2. **本轮探索的一切增量**（V4/V5/V7 完整版、HTML 增强、CSV/MD/XLSX、脑洞区全部）默认推迟，进入 P1/P2/Post-GA 候选池，**候选 ≠ 承诺**。
3. **未来引入的治理路径**（不可绕过）：
   - 每一项先过 **Scope Creep 四问**（示例，以 XLSX 为例：①是否直接提升四动词？——间接，服务于 Release 证据交付；②是否需要新增协议/硬件/运行时？——否；③是否可在现有 fixture 中验证？——是；④是否延迟 MVP？——若现在做，是 → **默认推迟**，与本文结论一致）；
   - 依赖过 **FS-ENG-006 依赖七问** + license 商业分发审查；
   - 新格式若成公共契约 → **版本化 schema + ADR**；若引图表库 → **实现 ADR**（含触发条件）；
   - 设计条款增补（A7 时间序列三纪律、A5 Δ 配色补记）随下一次设计资产升版入册。
4. **R6 提醒**：可视化与导出必须始终挂在四动词（Analyze→Compare→Gate→Release）的工作流价值上——是"看清 build / 看清变化 / 看清风险 / 固化证据"的延伸，不是独立 feature 岛。本轮全部清单项均已按此原则挂靠（V1–V5 挂 Analyze/Compare，V6 挂 Gate，V0/V7 挂证据与 History，导出物挂 Release 证据链）。

---

## 8. 待用户拍板的开放问题（每条附建议）

| # | 问题 | 建议 |
|---|---|---|
| Q1 | 本轮探索结论如何处置？ | 建议接受：本文标记为 FINAL 探索交付，清单进入"Post-MVP 候选池"（非承诺）；不改任何基线文档；当前唯一合法下一步仍是 P0 Technical Vertical Slice |
| Q2 | 若未来进入 P1，可视化与导出的优先序？ | 建议接受技术侧原则："先保判断力，再保可携带性"（虚拟化表格+ranked bars → HTML 增强 → CSV/MD → treemap → stacked bar → XLSX）；具体排期待 P1 规划时结合当时信号再定，现在不冻结 |
| Q3 | Excel/docx 的验证路径？ | 建议把"你拿到分析结果后会交给谁、用什么格式打开/转发"列入 MVP 阶段用户访谈问题清单（与付费意愿访谈合并进行，5–8 人起）；xlsx/docx 维持"不做"直至信号出现 |
| Q4 | PDF 的产品口径？ | 建议接受："官方不提供 PDF 导出，提供 HTML 打印视图（Ctrl+P）；归档用途请使用自包含 HTML"——避免支持成本与确定性承诺陷阱 |
| Q5 | 两项设计条款增补是否批准？ | 建议批准：A7 时间序列三纪律（含 0 基线/禁重排/禁双 Y 轴）增补入 DESIGN.md ⑦（History 动工前完成）；A5"Δ 不用红绿"补记入 DESIGN.md ⑤——均为文档级变更，随下次设计资产升版入册 |
| Q6 | Release Bundle 清单是否为 xlsx 等扩容？ | 建议维持锁定：xlsx/docx 不进 Bundle（体积与受众均不合理），需要的人走应用内交互导出；未来若要扩 Bundle，按基线纪律走 ADR（release-manifest 是 versioned public contract） |

---

*综合执笔：专业写手 · 2026-09-27 · 本文为探索交付稿，不修改任何基线文档，不触发功能创建；所有事实以四份输入所引基线原文为准。*
