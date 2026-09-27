# FirmwareSight UI 基调定案提案

> 提案人：设计匠人 ｜ 日期：2026-09-27 ｜ 状态：待用户拍板后冻结
> 输入：① 本人对 02_BRAND + 03_DESIGN + design-tokens.json 的基线理解；② 通用助手《FS-立项过程与产品演变理解摘要》⑤节（8 条隐含 UI 气质约束）；③ 信息哨兵《FirmwareSight-设计风格调研-awesome-design-md》（74 份风格筛选 + IBM Carbon 首选建议 + 落地五步法）
> 性质：设计治理提案，不创建业务功能，不触碰 Active Task = NONE 约束。

---

## 0. 结论速览

| 项 | 结论 |
|---|---|
| 基准终选 | **采纳 IBM（Carbon）为唯一气质基准**（信息哨兵 A 方案），Vercel / WIRED / Cal.com 降为单点条款参考，不作基准 |
| 总原则 | **基准定气质，tokens 定数值，红线定禁区**——design-tokens.json 仍是唯一数值真源 |
| 关键裁决 | 渐变全禁 ｜ 圆角维持 6/8/10 ｜ accent 维持 #2563EB ｜ 动效 120/150/180ms 三档 ｜ 表格行高 36px 基准 ｜ Unknown 用灰永不用蓝 |
| token 动作 | design-tokens.json 补 10 类缺口 → 升版 v0.2.0，解决 v0.1.0 draft 与 v0.3.0 基线的版本错位 |
| 待拍板 | 4 条（见 ⑥），每条附默认建议 |

---

## ① 基准风格终选与取舍理由

### 终选：IBM（Carbon）单基准 + FirmwareSight Overrides

采纳信息哨兵的 A 方案（纯 Carbon 路线）：以 `design-md/ibm/DESIGN.md`（pin commit）为唯一基准文档，剥离营销面模式、追加项目自有规则后，改造为仓库根目录 `DESIGN.md`。**不采用 B（Vercel 灰阶精密）/ C（Cal.com 温和）混合方案。**

### 取舍理由

1. **哲学同构，红线零冲突**。Carbon 基准原文 "engineered, not stylized" 就是 Instrument-grade clarity 的英文镜像；白底、发丝线、单一 accent、无渐变无玻璃拟态无霓虹——四条硬性排除项全部干净。选它不需要做"剥离手术"来换合规，只需要做"删营销 + 加约束"。
2. **语义状态色体系与证据灯逻辑天然对应**。Carbon 的 success/warning/danger 语义分组，与 FirmwareSight 的 PASS/REVIEW/BLOCK 证据灯是同一套思维；"状态完备性"（hover/pressed/focus/disabled 全标注）恰好是我们 design-tokens.json 最大的缺口方向——基准本身就示范了正确形态。
3. **数据密集是原生形态而非改造**。表格、审计日志、交替行条纹、可排序可聚焦的数据表——固件工作台的主战场（sections/symbols/diff/gate 记录）在 Carbon 体系里是默认场景，不是特例。
4. **开源与可演进路径**。MIT 许可可自由改造；Carbon 有官方 React 组件库，未来若某些组件自研成本过高，存在"逐步替换为真实组件库"的退路（非承诺，仅保留选项）。
5. **为什么不选其他**：
   - **Vercel**：气质对但需要手术（剥离品牌渐变），200 级灰阶的维护成本与"克制"精神相悖；其"发丝线表格 + mono 标签"值得借，但只借条款不借基准。
   - **WIRED**："报纸级密度"是 dense not crowded 的字面来源，但三字体体系不可商用、衬线个性偏编辑部，与"仪器感"有温差——只借它的**审计页密度条款**。
   - **Cal.com**：气质正确（"confidently engineered without trying to impress"）但系统精度（状态语义、token 完备度）明显弱于 Carbon，作备胎合格，作基准浪费。
   - **纯自研**：自研意味着每一个细节（行高、灰阶、焦点、阴影）都要重新争论一遍。Carbon 提供了三十年"仪器感"的行业共识，差异化应该来自**信息架构**（证据链如何呈现），而不是视觉噱头——这正是该仓库 74 个条目证明的规律：最强的品牌都把装饰预算压到最低。

### 基准的定位（重要）

基准文档是**气质与结构基准**，不是数值真源。三者的关系固定为：

```
治理红线（07_COMPLIANCE 禁词 / 03_DESIGN 反模式黑名单 / ADR 决策）   —— 最高，禁区
项目冻结资产（design-tokens.json / 03_DESIGN 组件铁律 / 02_BRAND 语调）—— 数值与规则真源
基准文档（Carbon 改造版 DESIGN.md）                                  —— 气质、结构、模式参考
```

冲突时按此顺序裁决；基准与项目资产冲突的 7 处，裁决见 ③。

---

## ② 基准风格的采纳 / 剥离 / 改造清单

### ✅ 直接采纳（写进 FS DESIGN.md）

| # | 条目 | 用在哪 |
|---|---|---|
| A1 | "engineered, not stylized" 总纲表述 | 基调定义的直接引用 |
| A2 | 层级靠表面色差 + 1px 发丝线，不靠投影 | 全局布局与面板体系 |
| A3 | 单一 accent 纪律（全系统仅一个品牌色，只标交互意图） | 色彩体系 |
| A4 | 语义状态色分组（success/warning/danger → pass/review/block） | 状态体系 |
| A5 | 状态完备性纪律：交互态（hover/pressed/focus/disabled）必须成对标注 | token 治理与组件评审 |
| A6 | 可见焦点环（高对比 focus ring） | a11y |
| A7 | Productive motion 概念（动效传达状态变化，不做表演） | 动效条款 |
| A8 | 数据表模式：可排序、可聚焦、交替行条纹（用 bg.subtle 实现）、空态必备 | 表格 |
| A9 | Tag 的克制用法（紧凑标签、少量语义色）→ 映射为 FS pill（仅限紧凑状态标签） | 组件 |
| A10 | Do's and Don'ts 护栏格式 | 评审 checklist |

### ❌ 必须剔除（来自基准的营销面 / 冲突面）

| # | 条目 | 剔除理由 |
|---|---|---|
| R1 | hero band / CTA banner / logo marquee / 案例卡片等 ibm.com 营销页模式 | 反营销化红线；工作台没有营销面 |
| R2 | Carbon 全局 0px 直角签名 | 与冻结的 radius 6/8/10 冲突（裁决见 ③-1） |
| R3 | IBM 蓝 #0f62fe 品牌色值 | 与冻结的 accent #2563EB 冲突（③-2） |
| R4 | Carbon "info 蓝"状态位 | 不引入第四状态色相；蓝色只属于 accent，未知用灰（③-4） |
| R5 | 48px 默认表格行高 | 对固件符号级大表过疏，违反 dense（③-3） |
| R6 | Expressive motion（240ms 档及弹性曲线） | 超出 FS 120–180ms 上限（③-5） |
| R7 | Pictograms / 插画体系 | 无插画预算；图标仅用于状态与操作语义 |

### 🔧 改造为项目自有规则（基准让位于冻结资产）

| # | 维度 | Carbon 基准值 | FirmwareSight 裁定值 | 依据 |
|---|---|---|---|---|
| C1 | 圆角 | 0px | **6 / 8 / 10 + pill 999（仅紧凑状态标签）** | design-tokens.json 冻结值 |
| C2 | accent | #0f62fe | **#2563EB**（兼 diff.changed，全家唯一蓝） | tokens 冻结值；两色对白底均 ≈5:1 过 AA，无换值收益 |
| C3 | 行高 | 48px | **默认 36px / 紧凑 28px / 上限 40px** | P5 dense not crowded 的量化落地 |
| C4 | 字体 | IBM Plex Sans/Mono | **MVP 系统栈 + 系统 mono**（Plex 列为 P1 候选，待拍板） | tokens 冻结值；零加载成本、零网络依赖（Local-first） |
| C5 | 动效时长 | 70–240ms | **120 / 150 / 180ms 三档**，统一 ease-out | FS 冻结区间 120–180ms；吸收 productive 分类 |
| C6 | 状态语义 | pass/warn/error/info | **PASS/REVIEW/BLOCK/UNKNOWN/N/A 五态，icon+label+count** | 03_DESIGN 组件铁律（5 态版本为权威，见 ⑤ 治理） |
| C7 | 未知呈现 | 无对应概念 | **Unknown = 中性灰 + 空心图标 + 事实句 + 下一步**，一等状态 | P3 + Explicit Unknown 原则，基准无法提供，属 FS 自有规则 |

---

## ③ 与 Instrument-grade clarity、四大信息支柱、design-tokens.json 的融合方案

### 3.1 冲突裁决表（明确到值）

| # | 冲突点 | 裁决 | 理由 |
|---|---|---|---|
| 1 | **渐变** | **全禁，零容忍**。包括品牌渐变、氛围渐变、图表渐变填充。数据面唯一允许的"多色"是单色蓝阶（见 ⑦ 图表） | 品牌反模式黑名单第一排；Carbon 基准本身无渐变，无执行成本 |
| 2 | **圆角** | 维持 tokens 冻结值 **6/8/10**；pill 半径 999 仅限紧凑状态标签。**拒绝 Carbon 0px** | 6/8/10 已冻结且在"克制"区间内，与仪器感不矛盾；改 0px 属纯审美变更，收益低、波及全组件库（列开放问题 ⑥-1 供用户终审） |
| 3 | **密度** | **表格行高 36px 基准 / 28px 紧凑 / 40px 上限**；单元格横 padding 12（spacing-3）；4px 网格 8 档不变 | "dense not crowded"首次量化成数字；同时否决 Carbon 48px（过疏）与过度压缩（<28px 损可读性） |
| 4 | **状态色** | 四色不变（pass #18794E / review #9A6700 / block #C9372C / unknown #667085，正文场景均 ≥4.5:1）；**Unknown 专用中性灰，永不复用 accent 蓝**；review 在 12px 表格小字场景改用加深变体 #7A5200（新 token） | 蓝 = 交互意图，若兼作"未知"会诱导用户点击/误读为可处理项；4.9:1 的 review 琥珀在 12px 偏紧，加深变体一次解决 |
| 5 | **动效** | **120/150/180ms 三档**（微交互/标准/面板），统一 ease-out；仅限面板开合/行展开/状态迁移/拖放反馈；prefers-reduced-motion 全量尊重 | 吸收 Carbon productive motion 概念，但时长以 FS 冻结区间为准；240ms 表达性动效永不引入 |
| 6 | **阴影** | 仅浮层两级（overlay-sm 菜单与 popover / overlay-lg 对话框与拖拽层），永不用于卡片、面板、行 | 基准与 FS 规则完全一致，补 token 即可 |
| 7 | **accent** | 维持 **#2563EB**，全家唯一蓝：交互意图（链接/主按钮/选中态）+ diff.changed 两个用途，永不作大面积铺色 | 换 #0f62fe 无对比度收益且需全量重验 14 色 + diff 三色复用关系 |

### 3.2 八条隐含气质约束 → 基准条款映射（对通用助手⑤节的逐条回应）

| 隐含约束 | 融合后落点 |
|---|---|
| 1 反魔法、反 AI 化 | 基调引用 "engineered, not stylized"；无 AI 装饰、无魔法按钮；ADR-0005 已禁 AI 入可信路径 |
| 2 反营销化、合规谦逊 | R1 全部剔除营销面模式；界面文案守语调三律与合规禁词（无 guaranteed/compliant） |
| 3 证据优先、不确定性可视化 | 状态五态 icon+label；Unknown 一等状态（C7）；置信度以文本+-ranked 呈现（"来自 version.h 推断"），不做伪装精确的进度条 |
| 4 工程报表气质而非仪表盘炫技 | 图表优先级链表格优先；treemap 限定 size 构成 + 单色蓝阶——"好看"被约束在不撒谎的范围内；获客张力由 G1 极简技术 UI / G2 Minimum Credible UI 的分层化解，基调不变 |
| 5 克制专业、量化不夸口 | P2 数字必须带上下文写进图表条款；动效只做功能不做表演 |
| 6 零门槛进入 | 首流 A 保持：无账号墙，能力 banner（ELF ✓ · MAP 未提供 · Git 未链接）+ 推荐下一步；空态必须给行动出口 |
| 7 面向发布决策而非文件查看 | Overview 回答"现在能发吗"；导航四动词叙事不变；基准的表格/审计模式全部服务于该叙事 |
| 8 清单式错误、非弹窗恐吓 | 错误四要素（What happened / Why we know / What to do / Diagnostics ID）驻留页面可恢复位置；Dialog 仅真正中断，Toast 仅瞬时确认 |

### 3.3 四大信息支柱 → 视觉决策映射

| 支柱 | 主战场 | 基调落点 |
|---|---|---|
| See the build | Overview / Analyze | 表格优先 + ranked bars；所有数字 mono；每个数字带上下文（占比/最大贡献者/是否超预算）；能力 banner 即证据状态栏 |
| See the change | Compare | Diff Row 必须 old/new/delta 同列，added/removed 禁 0 冒充；diff 三色复用状态四色（无新色相）；浅底色变体补 token 后用于行级高亮；最大增长贡献者可点击下钻 |
| Verify the release | Release Gate | 五态分组呈现（Block 组在前）；每条规则 Result+Why+Evidence+Next 四件套；Review 必须显式接受；Block 与 Review 的视觉区分靠图标+文字而非仅颜色 |
| Keep the evidence | Bundle / History | 借 WIRED 审计密度条款：发丝线分隔、署名行式元数据（谁/何时/哪个 hash）；哈希、时间戳、签名一律 mono；Bundle 脱离 FirmwareSight 独立可读 |

### 3.4 与 design-tokens.json 的融合关系

- **tokens = 唯一数值真源**：色值、字号、间距、圆角只存在于 JSON；DESIGN.md 只引用语义名，不另立数值（避免双真源漂移）。
- **14 个语义角色全部保留**，不动一个色值（14 色对白/浅底作正文均 ≥4.5:1，最紧 review ≈4.9:1 已由 #7A5200 变体兜底）。
- **缺口按 ⑤ 清单补齐后升版 v0.2.0**，同时解决"v0.1.0 draft 却被 v0.3.0 基线指定为权威"的版本错位（治理动作，见 ⑥-4）。

---

## ④ UI 基调最终表述

### 一句话基调定义

> **FirmwareSight 的界面是一台安静的测量仪器：白底与发丝线之上只陈列证据——每个数字带上下文，每个状态带图标与出处，每个未知都坦率可见；装饰预算为零，判断永远留给工程师。**

### 关键视觉决策清单

**色彩**
- MVP 仅 Light 主题（Dark = P1，届时全量重验对比度）
- 表面层级：canvas #F7F8FA ＜ surface #FFFFFF ／ subtle #F1F3F5——层级靠色差与 1px 边框，不靠阴影
- 文字：primary #171A1F / secondary #5D6673；边框 #D9DEE5
- 唯一品牌色：accent 蓝 #2563EB（交互意图 + diff.changed；永不铺大面积）
- 状态四色：pass #18794E / review #9A6700（小字变体 #7A5200）/ block #C9372C / unknown #667085
- 渐变 0 容忍；treemap/堆叠条仅用锚定 accent 的单色蓝阶

**字体**
- 系统栈（Windows 首要平台自然落到 Segoe UI）+ 系统 mono；Plex 列 P1 候选
- 哈希、版本号、路径、寄存器地址、时间戳、**所有数字**一律 mono
- 字阶：Page 20/28 · Section 15/22 · Body 13/20 · 表格 12–13/18 · 元数据 12/18；全 UI ≥12px

**密度**
- 4px 网格八档（4–64）；表格行高默认 36 / 紧凑 28 / 上限 40；单元格横 padding 12
- nav rail 220–240px；Evidence Inspector 320–380px（P0 真实数据验证后定稿）；表格优先吃满宽度
- 一屏一个主焦点；KPI 不塞首屏

**边框与阴影**
- 1px 边框为默认分隔语言；圆角 6/8/10，pill 999 仅限紧凑状态标签
- 阴影仅两级且仅浮层（overlay-sm / overlay-lg）；卡片与面板永不投影
- 焦点环 2px accent + 2px offset，focus-visible 必现

**状态与 Unknown**
- 五态 = icon + label (+ count)：✓ PASS ／ ⚠ REVIEW ／ ✕ BLOCK ／ ○ UNKNOWN ／ – N/A；状态永不只靠颜色
- Unknown = 中性灰 + 空心图标 + 事实句（"Detected / version unknown"）+ 下一步动作；永不蓝色、永不隐藏、永不冒充 Observed；用户 Declare 后明确标 Declared
- 错误四要素驻留可恢复位置；Dialog 仅真正中断，Toast 仅瞬时确认

**图表**
- 优先级：table ＞ ranked bars ＞ compact stacked bar ＞ treemap（仅 size 构成）
- 禁 3D、禁滥用 donut、禁无基线 sparkline、禁彩虹分类色
- 每个数字必须带上下文：相对谁 / 占比 / 最大贡献者 / 是否超预算

**动效**
- 120 / 150 / 180ms 三档，统一 ease-out；仅限面板开合 / 行展开 / 状态迁移 / 拖放反馈
- Productive motion：动效传达状态变化，不做引人注意的表演；prefers-reduced-motion 全量尊重

---

## ⑤ 落地建议

### 5.1 基准 md 的治理入仓方案

1. **产出《FirmwareSight DESIGN.md》置于仓库根目录**（遵循 awesome-design-md 的"AGENTS.md 管怎么构建，DESIGN.md 管长什么样"消费约定，AI 编码代理与人类评审共用）。结构九节：
   ① 基调定义（一句话 + 四支柱 + 八气质约束映射）→ ② 裁决优先级（红线＞tokens/冻结资产＞本文）→ ③ 数值真源声明（只引用 design-tokens.json 语义名）→ ④ 布局与密度 → ⑤ 组件铁律（沿 03_DESIGN：按钮三级/五态/表格/Diff Row/Dialog/Toast）→ ⑥ 状态与 Unknown 条款 → ⑦ 图表条款 → ⑧ 动效条款 → ⑨ Do's & Don'ts checklist + 上游差异声明。
2. **上游锚定**：记录 awesome-design-md 的 pin commit hash 于 DESIGN.md 页脚；上游活跃更新（2026-09 仍在推送），**每季度 diff 一次，不自动同步**；关键值与 Carbon 官方文档（carbondesignsystem.com）复核一次后即可脱离上游独立演进。
3. **AGENTS.md 增补 UI Rules 节**：声明"所有 UI 遵循根目录 DESIGN.md；DESIGN.md 与 AGENTS.md 冲突时视觉以 DESIGN.md 为准；与治理红线冲突时红线为准；禁止渐变/玻璃拟态/霓虹/紫蓝系"。
4. **评审流程**：Do's & Don'ts 转为 PR 设计 checklist（可机械核验项：无渐变、交互态齐备、焦点环、状态 icon+label、mono 数字、行高 ≤40px）。
5. **与门禁衔接**：本基调 + tokens v0.2.0 直接作为 G1"极简技术 UI"的输入；G2"Minimum Credible UI"验收时用同一 checklist 评审——两个阶段共用一套基调，避免返工。

### 5.2 需补齐的 token 缺口清单（design-tokens.json v0.1.0 draft → v0.2.0）

| # | 缺口 | 建议值（定稿前过对比度验证） |
|---|---|---|
| 1 | accent 交互态 | accent.hover #1D4ED8 / accent.pressed #1E40AF / accent.disabled 沿用灰禁用样式 |
| 2 | 焦点 | focus.ring = #2563EB，2px，offset 2px，仅 focus-visible |
| 3 | review 小字变体 | status.review.strong = #7A5200（12px 表格文字用，≈6.9:1） |
| 4 | diff 浅底色 | diff.added.bg / removed.bg / changed.bg（草案 #E7F3EC / #FBECEA / #E9F0FD，与前景字成对验证） |
| 5 | 浮层阴影 | shadow.overlay-sm（菜单/popover）/ shadow.overlay-lg（对话框/拖拽层），炭黑低透明度双层 |
| 6 | pill | radius.pill = 999px + pill 高度档（限紧凑状态标签） |
| 7 | 表格密度 | table.row.default = 36 / table.row.compact = 28 / table.row.max = 40 |
| 8 | 动效 | motion.duration.micro/standard/panel = 120/150/180ms + motion.ease = 标准 ease-out 曲线 |
| 9 | 图表蓝阶 | chart.seq.blue 五档（#DCE7FB→#1D4ED8 锚定 accent，相邻档 ≥3:1 区分）+ 中性"其余"色 #F1F3F5 |
| 10 | 元数据修正 | meta.version → 0.2.0、status → baseline-authoritative；text.disabled = #98A2B3 |

> Dark 主题全套 token 不在本缺口内（P1 另行全量重验）。

### 5.3 治理动作清单（按依赖序）

1. 用户拍板 ⑥ 的 4 个问题 → 2. 新增一份 ADR（建议 ADR-0018：UI 基准与 design tokens 治理——采纳 Carbon 气质基准、tokens 为数值真源、5 态版本为权威、版本错位豁免注记）→ 3. design-tokens.json 按上表升版 v0.2.0 → 4. 产出根目录 DESIGN.md + AGENTS.md 增补 → 5. checklist 进 templates。全程为文档变更，不违反 Active Task = NONE。

---

## ⑥ 需要用户拍板的开放问题（附默认建议）

| # | 问题 | 选项 | 我的默认建议 |
|---|---|---|---|
| 1 | **圆角签名**：维持冻结的 6/8/10，还是硬化为 Carbon 式 0–2px？ | 6/8/10（温和仪器）vs 0–2px（硬核仪器，更贴 Carbon 原味） | **维持 6/8/10**。已冻结、在克制区间内；0px 是纯审美变更，波及全组件，不值得 |
| 2 | **品牌字体**：MVP 用系统栈，还是本地打包 IBM Plex Sans/Mono？ | 系统栈（零成本）vs Plex（更强的仪器个性，包体 +几百 KB，需本地化打包） | **MVP 系统栈，Plex 列 P1 候选**。G2 验证期品牌字体不构成付费理由；若 GA 前想强化品牌再打包 |
| 3 | **灰阶策略**：维持现有 5 阶灰，还是引入 Vercel 式多级灰阶？ | 现有 5 阶 vs 200 级灰阶体系 | **维持 5 阶**。只借鉴 Vercel"层级靠表面色差不靠阴影"的原则，不搬体系——维护成本与克制精神相悖 |
| 4 | **tokens 升版走法**：补缺口升 v0.2.0 走 ADR-0018 一次定案，还是暂维持 v0.1.0 仅加 BASELINE 注记？ | ADR 定案 vs 仅注记 | **ADR-0018 一次定案**。版本错位 + 缺口 + 5 态权威三件事一个 ADR 收干净，避免下次再翻旧账 |

---

## 附：本提案的置信度与边界

- 三份输入均为一手材料（本人基线精读 / 摘要经原文件交叉核对 / 信息哨兵逐份取证 74 份 md），无二手转述链。
- design-tokens.json 的 14 色对比度为本人在浅底场景的复算值；#7A5200、diff 浅底草案等新增建议值在 token 升版时须再过一轮机器验证。
- Carbon 基准源自对 ibm.com 的逆向提取（非 IBM 官方发布物），关键 token 落地前与 Carbon 官方文档复核一次（已列入 5.1 第 2 条）。
