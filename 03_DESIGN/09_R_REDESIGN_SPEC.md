---
title: "R-Series Redesign Spec — Instrument Ledger"
doc_id: "FS-DESIGN-010"
product: "FirmwareSight"
version: "0.6.0"
status: "PROPOSED"
owner: "Design"
last_updated: "2026-10-16"
---

# R 系列重设计规格 —— “Instrument Ledger（仪器账本）”

**状态：** 本文是 R0 交付物，状态 PROPOSED，等待 Owner 视觉评审。
- 治理依据：[ADR-0034](../09_ADR/ADR-0034-instrument-ledger-design-language.md)（PROPOSED）。
- 原型：[prototype/](prototype/README.md)。
- R1 之前，本文只约束设计，不改任何产品代码、token 或 golden。

本文覆盖桌面应用的全部页面和全部导出物：HTML、SVG、PNG、Markdown、CLI 文本。Owner 的 B1 “Quiet Canvas” 方案只作参考，诊断见 §1。

---

## 0. 示例数据

规格、原型与报告样张都使用同一套仓库内的真实数据，没有编造的数字：

| 项 | 值 | 来源 |
|---|---|---|
| 项目 / 策略 | `brake-node`；Flash 预算 512 B、RAM 预算 256 B；增长复核阈值 200 B；策略 sha256 `b87aa057…5e98` | `fixtures/project/p4-release/firmwaresight.toml` |
| 基线构建 | `firmware.elf` `85ae4fc4e2b8`；Flash 132 B · RAM 12 B | `fixtures/elf/m2-objects/base` |
| 目标构建 | `firmware.elf` `3c9daa4f7c09`；Flash 196 B · RAM 44 B；8 节 · 32 符号 · 10 条证据；区域 FLASH 65,536 B、RAM 16,384 B（MAP 声明） | `fixtures/elf/m2-objects/target` |
| 目标构建的对象 | `sensors/imu.o` 80/0 · `main.o` 56/8 · `libdrv.a(spi.o)` 28/32 · `libdrv.a(uart.o)` 12/4（Flash/RAM，B）；未归属 20 B | `fwsight analyze --objects` |
| 对比结果 | Flash +64 B（132→196，+48.5%）· RAM +32 B（12→44）· 精确可比；对象 2 变化 / 1 新增 / 1 未变；`sensors/imu.o` +36 B；`libdrv.a(spi.o)` 新增 28 B；符号 +21 −15 ~6，其中 33 行歧义 | `fwsight diff` |
| Gate | **BLOCK 1**（`release.version_matches_policy`：HEAD 上没有精确版本 tag）· PASS 7 · N/A 2；Flash 196/512（余 316 B）· RAM 44/256（余 212 B） | `fwsight gate`（以 p4 项目策略评估 m2 目标构建） |

---

## 1. 诊断

### 1.1 当前产品（`origin/main` `e4d7f8b`）
1. **页面过长，层层堆叠。**
   - `Release.tsx` 2325 行，11 个区块，编号乱序，配置放在结果之后。
   - `Compare.tsx` 2133 行，Section 和 Symbol 两张变化表近乎复制。
   - Analyze 嵌入 Details 和 Report，实际是三页叠在一起。
2. **同一件事有多种模式。**
   - 视图切换有三种。
   - 筛选表单有三套。
   - 构建选择有两种。
   - 手写 `<table>` 14 张。
   - 单位开关散在 5 处。
3. **证据只在 Analyze › Evidence 标签页可达**，其他页面的结论点不进证据。
4. **页面主体不明确**：过期和保留状态由各页零散的段落承担。

### 1.2 B1 “Quiet Canvas”
1. **六页共用同一模板**，没有一页先给答案。
2. **指标缺少参照**，主次也颠倒：“67 个歧义符号”与 Flash 变化同等字号。
3. **颜色带评判**：增长用琥珀色，Removed 用琥珀、Added 用绿色；新增或删除的节把依据写成 “Unknown”。
4. **没有 Inspector、没有 Help、没有图标体系**，并残留演示外壳。
5. **报告样张有装饰**：8px 顶色条、约 70px 的状态字。

B1 中值得保留的：连续画布、少边框、表格为主、五态字形加文字、浅色导出、为窄屏 reflow 做的努力。

---

## 2. 理念与原则

固件发布是一本账：每个字节有来处，每次变化有原因，每个结论有证据。界面要像精密仪器：读数清楚，刻度诚实，随手能追到出处。

| # | 原则 | 落地方式 |
|---|---|---|
| P1 | 每页先给一个答案 | `LeadAnswer`：一句事实，一个 figure 数字，一个参照 |
| P2 | 任何数字都能追到出处 | `FactValue` 可点击，`Inspector` 显示分类、来源、定位、规则、原始值 |
| P3 | 主体始终可见 | `SubjectBar`：当前构建或构建对、MAP 和证据等级、最新或已保留 |
| P4 | 变化是事实，不做评判 | delta 用中性色加 ▲▼；好坏只由 Gate 五态表达 |
| P5 | 一件事只有一种做法 | 一个 DataTable、一种视图切换（Tabs）、一种通知（Notice）、一种构建选择器（BuildPicker） |

---

## 3. 外壳与信息架构

### 3.1 AppShell

```
┌ TopBar 40 ──────────────────────────────────────────────────────────────────────────┐
│ ■ FirmwareSight   brake-node ▾        本机工作区 · 不会离开这台机器     ⌕ 命令 Ctrl+K │
├ Rail 224 ──────┬ SubjectBar 44 ───────────────────────────────────────────────────────┤
│ ◰ 总览          │ firmware.elf · 3c9daa4f7c09 · MAP ✓ · 证据 observed · 最新     [更换] │
│ ◫ 分析          ├───────────────────────────────────────────────┬────────────────────┤
│ ⇄ 对比          │ 分析                                           │ Inspector 340      │
│ ◇ 发布          │ Flash 已用 196 B（FLASH 区域 65,536 B 的 0.3%） │ .text · Flash 192 B│
│ ◷ 历史          │ ▔▔▔▔ 内存 · 节 · 符号 · 对象 · 证据              │ 分类  observed     │
│                 │ ┌ DataTable ─────────────────────────────┐     │ 来源  ELF + MAP    │
│ ─────────       │ │ …                                       │     │ 定位  section[1]   │
│ ? 帮助          │ └─────────────────────────────────────────┘     │ 规则  map-memory…  │
│ ⚙ 设置          │                                                │ 原始值 0xc0        │
└────────────────┴───────────────────────────────────────────────┴────────────────────┘
```

| 区域 | 规格 |
|---|---|
| TopBar | 高 40。从左到右：品牌、项目切换、本机工作区声明、命令面板入口。单位开关放在命令面板和设置里，页面上不再出现 |
| Rail | 宽 224；窗口小于 1180 时收为 56（只显示图标，悬停或聚焦时出 tooltip）。工作区 5 项，工具区 2 项 |
| SubjectBar | 高 44。单构建模式：名称、短 SHA、MAP 徽章（菜单内可添加、替换、移除）、证据等级、新鲜度。对比模式：基线 ⇄ 目标两个 BuildPicker 加“交换”。整个应用里只有这一处显示“过期 / 已保留”，取代各页零散的 stale 段落 |
| Workspace | 页面标题 20/28 → LeadAnswer → Tabs → 主工作面（DataTable 等） |
| Inspector | 宽 340（可调 320–420）。窗口 ≥1280 时并排固定；<1280 时以覆盖式 Sheet 打开，带 overlay-sm 阴影，这是允许的浮层。Esc 关闭，焦点回到触发元素 |

**窗口尺寸**：`tauri.conf.json` 的最小尺寸改为 token 规定的 1024×720；设计目标仍是 1440×900。

### 3.2 信息架构

| Rail 项 | 回答的问题 | 视图（Tabs） |
|---|---|---|
| 总览 | 这个项目现在能不能发？ | （单视图） |
| 分析 | 这个构建里有什么，空间花在哪？ | 内存 · 节 · 符号 · 对象 · 证据 |
| 对比 | 变了什么，为什么变？ | 概要 · 节 · 符号 · 对象 |
| 发布（原“发布门禁”） | 按项目策略能发吗？证据在哪？ | 结论 · 证据 · 发布包；外加“运行设置”条 |
| 历史（原“发布包与历史”） | 我们当时知道什么？ | 构建 · Gate 运行 · 发布记录 |
| 帮助 / 设置 | 工具区 | — |

**命名约束**：
- 不新增第五个产品动词。
- 不新增 i18n 命名空间（ADR-0032 D4）：Rail 和外壳用语放在 `common`。
- ADR-0032 D5 规定不翻译的词保持原样：五态词、证据分类词、Added/Removed/Changed。

### 3.3 键盘与命令

| 键 | 作用 |
|---|---|
| `Ctrl+K` | 命令面板（combobox + listbox）：跳到某页或某视图、选择固件、附加 MAP、与固定基线对比、切换单位、主题、语言、导出 |
| `Ctrl+O` | 选择固件 |
| `/` | 聚焦当前 DataTable 的筛选框 |
| `[` `]` | 上一个或下一个视图 |
| `I` | 打开或关闭 Inspector（焦点在可追溯数字上时打开该数字） |
| `Esc` | 关闭最上层浮层，焦点回到触发元素 |

每页都有“跳到主内容”链接。切换页面后焦点落在 h1，`document.title` 同步更新为 “页面 · 主体 — FirmwareSight”。

---

## 4. 视觉系统（tokens v0.4.0 草案，正文见 ADR-0034 §D3）

### 4.1 只增不移
- 色板、间距、圆角、密度、动效、焦点环等现有数值都不变。
- **唯一的改值**：浅色 `color.diff.removed` 从 `#C9372C` 改为 `#C7372C`，在 `#FBECEA` 上为 4.56:1，撤销 ADR-0031 D6 的豁免。
- **新增**：
  - `font.size.figure` 28/36（每页只用一次）；`font.weight.regular` 400、`semibold` 600。
  - `layout.topbar` 40、`layout.subject_bar` 44、`layout.rail_collapsed` 56。
  - `layout.breakpoint.rail_collapse` 1180、`layout.breakpoint.inspector_dock` 1280。
  - `icon.size.sm` 16、`icon.size.md` 20、`icon.stroke` 1.5。

### 4.2 字体层级

| 角色 | 规格 | 用途 |
|---|---|---|
| Figure | 28/36 · 600 · tabular · mono | 每页 LeadAnswer 的那一个数字 |
| Page | 20/28 · 600 | h1 |
| Section | 15/22 · 600 | h2 |
| Body | 13/20 · 400 | 正文、表格主列 |
| Meta | 12/18 · 400 | 表头、元数据、脚注 |

只用系统字体：Windows 11 上是 Segoe UI Variable；数字、哈希、路径用 mono，并开启 `tabular-nums`。

### 4.3 颜色用法
| 用途 | 规则 |
|---|---|
| delta | 一律 `text.primary`，加 ▲（增）/ ▼（减）字形，不用状态色。“增长是坏事”这个判断属于 Gate 规则 |
| Diff 三类 | 只在变化视图中出现：`diff.*.bg` 作行底色，并带文字标签 Added / Removed / Changed |
| 预算与区域条 | 填充用 `chart.blue.4`，轨道用 `chart.other`。只有当 Gate 对应规则给出 REVIEW 或 BLOCK 时，条旁才出现状态徽章 |
| Accent | 只用于主按钮、选中状态、焦点环、链接，不用于数据 |
| Unknown | 中性灰、空心虚线图标、一句事实、一个下一步 |

### 4.4 图标
- 仓库内自建 SVG 组件，24 个，viewBox 20，描边 1.5，`currentColor`，不引入图标库。
- 状态图标：
  - PASS：实心圆 + 勾
  - REVIEW：三角 + !
  - BLOCK：八边形 + ×
  - UNKNOWN：虚线空心圆 + ?
  - N/A：圆 + 斜线
- 导航、动作、证据图标见原型的“组件”页。

### 4.5 动效
- 只用三档：Inspector 开合 180、行展开 150、状态迁移 120。
- `prefers-reduced-motion` 下全部取消。
- 不做骨架屏闪烁。

---

## 5. 组件规格（R1 交付）

| 组件 | 职责 | 关键 a11y |
|---|---|---|
| `AppShell` / `Rail` / `TopBar` / `SubjectBar` | 外壳与主体 | nav 地标；`aria-current="page"`；SubjectBar 是 `region`，名称为“当前主体” |
| `Inspector` / `Sheet` | 证据详情、编辑面板 | 覆盖式时 `role=dialog aria-modal`、有焦点陷阱；并排时为 `complementary` |
| `DataTable` | 列定义、排序、筛选、分页、sticky 表头、复制单元格、行 → Inspector、28/36 两档密度、空态和错误态 | `caption`；`th scope`；`aria-sort`；行操作用原生 button |
| `FilterBar` | 字面子串筛选、分类、变化类型 | 表单有名称；`/` 聚焦 |
| `Tabs` | 唯一的视图切换方式 | tablist / tab / tabpanel；方向键切换 |
| `FactValue` | 数字 + 单位 + 证据分类点（observed 实心、derived 半实心、declared 描边、unknown 虚线）+ 可追溯 | button，名称为“{值}，{分类}，查看证据” |
| `DeltaValue` | 旧 → 新、▲▼delta、缺失记号“—（不存在）” | 朗读为“从 X 到 Y，增加 Z” |
| `LeadAnswer` | 一句事实 + figure + 参照 + 至多一个主按钮 | h1 之后的第一个段落，同时是 `aria-describedby` 的来源 |
| `Notice` | info / stale / unknown / error 四种；error 写明 发生了什么 / 依据 / 下一步 / 诊断 ID | error 用 `role=alert`，其余用 `role=status` |
| `BuildPicker` | 选择已存的构建：搜索、最近、固定基线置顶 | combobox |
| `CommandPalette` | 全局命令 | combobox + listbox，`aria-activedescendant` |
| `StateBadge` / `Icon` | 五态 = 图标 + 文字（+ 计数） | 图标 `aria-hidden`，文字可读 |
| 保留并重构 | `Button`、`RankBar`、`MemoryUsage`、`Pager`、`SortHeader` | — |

---

## 6. 页面

线框中的数字均来自 §0。

### 6.1 总览 —— “这个项目现在能不能发？”
```
总览
BLOCK  1 项阻止发布                                                       [前往发布 →]
       release.version_matches_policy：HEAD 上没有符合策略的精确版本 tag。这不是发布许可，也不是认证。
──────────────────────────────────────────────────────────────────────────────────────
当前构建  firmware.elf · 3c9daa4f7c09 · MAP ✓ · observed            Gate 运行 gate-42b84f…
内存余量（策略预算）  Flash  196 / 512 B  ▕██████░░░░░░░░░░▏ 余 316 B
                      RAM     44 / 256 B  ▕███░░░░░░░░░░░░░▏ 余 212 B
自固定基线 85ae4fc4e2b8 以来   Flash ▲64 B · RAM ▲32 B · 最大来源 sensors/imu.o ▲36 B   [查看对比]
```
- LeadAnswer 的五种状态：
  1. 未分析：“选择固件开始”，加三步编号说明和拖放区。
  2. 已分析但未跑 Gate：“尚未运行发布检查”。
  3. Gate PASS：附“不是发布许可”。
  4. Gate REVIEW 或 BLOCK：给出计数和第一条原因。
  5. Gate 结果过期：由 SubjectBar 标注。
- 预算条与区域条在视觉上严格分开：“策略预算”和“物理容量”是两种参照，标签必须写明。

### 6.2 分析
```
分析
Flash 已用 196 B（策略预算 512 B 的 38% · FLASH 区域 65,536 B 的 0.3%）· 最大对象 sensors/imu.o 80 B
内存 · 节 · 符号 · 对象 · 证据
─ 内存 ────────────────────────────────────────────────────────────────────────────────
FLASH  0x08000000–0x08010000  65,536 B（declared）
  构成 ▕imu 80▏main 56▏spi 28▏uart 12▏未归属 20▏…  （单一蓝阶，表格为准）
  名称            Flash      RAM     份额     证据
  sensors/imu.o    80 B      0 B    40.8%    ● observed
  …
RAM    0x20000000–0x20004000  16,384 B（declared）
```
- **对象视图**：对象和库用分段控件切换。没有 MAP 时显示 Unknown Notice：“这个构建没有 GNU ld MAP，只有 MAP 能说明每个字节来自哪个目标文件。附加 MAP →”
- **证据视图**：DataTable 列出 分类 · 字段 · 值 · 来源 · 规则；点行打开 Inspector。
- **输入**：主按钮“选择固件”（`Ctrl+O`，支持拖放）；MAP 操作在 SubjectBar 的徽章菜单里。
- **分析中**：顶部进度条加取消。
- **失败**：error Notice，同时保留上一次结果，SubjectBar 显示“已保留上一次结果”。

### 6.3 对比
```
基线 firmware.elf · 85ae4fc4e2b8   ⇄   目标 firmware.elf · 3c9daa4f7c09        [固定基线]
对比
Flash ▲64 B（132 → 196 B，+48.5%）· 最大来源 sensors/imu.o ▲36 B · 可比性 exact
概要 · 节 · 符号 · 对象                                                         [导出 ▾]
─ 变化来源 ─────────────────────────────────── ─ 证据与可比性 ──────────────────────────
对象 | 库 | 节                                   ● 两侧均有 MAP，精确可比
sensors/imu.o     44 → 80 B     ▲36            ○ 33 行符号名重复，未配对，不视为变化
libdrv.a(spi.o)    — → 28 B     Added           （Notice · info）
main.o            52 → 56 B     ▲4
```
- 节、符号、对象三个视图共用 DiffTable，列为 变化 · 名称 · 旧 · 新 · Δ；Flash/RAM 两组。
- 缺失一侧写“—”，悬停或读屏时说明“不存在于基线”；歧义行带注释，永不配对。
- 导出菜单：JSON · HTML · 视觉报告（SVG/PNG/打印 HTML）· Markdown（新）。

### 6.4 发布
```
发布
BLOCK  1 项阻止发布 · PASS 7 · N/A 2          这是项目策略的判定，不是认证或发布许可。
运行设置  构建 3c9daa4f7c09 · 基线 85ae4fc4e2b8 · 策略 b87aa057… · 附件 0          [编辑]
结论 · 证据 · 发布包
─ BLOCK ───────────────────────────────────────────────────────────────────────────────
⊗ release.version_matches_policy   HEAD 上没有符合 `[version] pattern` 的精确 tag。
   下一步：为本次发布的 HEAD 打上符合 pattern 的 tag，然后重新运行 Gate。     [查看依据]
─ PASS（7） ───────────────────────────────────────────────────────── 已折叠 · 展开
─ N/A（2） ────────────────────────────────────────────────────────── 已折叠 · 展开
```
- **没有运行时**：主体是三步纵向步进，底部固定“运行 Gate”主按钮：
  1. 构建与基线（BuildPicker）
  2. 项目策略：摘要 + 在 Sheet 中编辑（取代 255 行的嵌套表单）
  3. 附件
- **有运行后**：设置折叠为一行摘要。结论按 BLOCK → REVIEW → UNKNOWN → PASS → N/A 分组。REVIEW 行“接受”时在 Sheet 里填理由，并记录操作人和时间；接受不会改写证据。
- **发布包视图**：存在 BLOCK 时禁止生成，并说明原因；有替换风险时用 Sheet 确认。

### 6.5 历史（含 D3）
```
历史
固定基线：firmware.elf · 85ae4fc4e2b8（2026-10-16 固定）                       [取消固定]
构建 · Gate 运行 · 发布记录
 ☐  构建                       导入时间        Flash   RAM   MAP   操作
 📌 firmware.elf 85ae4fc4e2b8  …              132 B   12 B   ✓    [与当前对比]
 ☐  firmware.elf 3c9daa4f7c09  …              196 B   44 B   ✓    [固定为基线] [对比]
                                                        选中 2 项 → [对比所选]
```
- **固定基线**是测量参照，不是收藏（`08_POST_MVP_PRESENTATION_RULES.md` §8）：
  - 每个项目同一时刻只有一个。
  - 存储在 migration 0008 `baseline_pins`，可撤销，撤销本身也留记录。
  - Compare 默认把它作为基线。
- Gate 运行和发布记录：DataTable，点行打开 Inspector 查看详情。

### 6.6 设置 · 帮助
- **设置**：使用统一外壳，分组为：外观（主题、语言）· 数据显示（单位）· 启动页 · 重置（Sheet 确认）。
- **帮助**：关于 · 可读取的格式 · 本机优先说明 · 快捷键表 · 诊断导出。

---

## 7. 状态矩阵（每页必须设计并测试）

| 状态 | 表现 |
|---|---|
| 空 | LeadAnswer 写明下一步，只有一个主按钮，带拖放区（总览、分析） |
| 加载 | 局部进度，旧内容保持可见并标注；不用骨架屏 |
| 错误 | error Notice（四要素）；保留上一次结果，SubjectBar 显示“已保留” |
| 过期 | SubjectBar 显示新鲜度“已保留上一次结果 · 当前选择未分析”，并给出操作 |
| Unknown | unknown Notice 或单元格“Unknown + 原因”，永远不写 0 |
| 不存在 | “—（不存在）”，永远不写 0 |
| 歧义 | 行注释“名称重复，未配对” |
| 窄窗 | 宽 ≥1280 时三栏；1180–1280 时 Inspector 改为 Sheet；1024–1180 时 Rail 收起 |

---

## 8. 导出与报告体系（“Ledger · Report edition”）

### 8.1 统一的报告语法
所有导出物都按同一顺序组织：
1. 刊头：产品 · 报告类型 · 主体身份
2. 结论行（与应用内同一句）
3. 事实区（表格为主）
4. 证据附录
5. 限制说明（写成事实句）
6. 页脚（完整哈希、生成器版本）

导出恒为浅色、确定性生成（同一输入同一字节），不联网。

### 8.2 各格式规格

| 格式 | 规格 |
|---|---|
| HTML（Compare 详细、Release 报告、视觉打印版共用一个模板体系） | 单文件内联 CSS，无 JS；地标、`caption`、`th scope`；状态用内联 SVG + 文字；A4 `@page`，表头跨页重复；数字用 mono、tabular |
| SVG（1600×900） | 网格：刊头带 72 → 结论 96（figure 28）→ 主图（排行条或预算条，单一蓝阶）→ 关键表格 → 脚注；正文 ≥12px；带 `<title>`、`<desc>`、`role="img"`；去掉装饰顶条和超大状态字 |
| PNG | 由同一份 SVG 栅格化；页脚注明字体依赖系统，这是已知的可移植性限制 |
| Markdown（新增） | 面向 PR、Issue、评审记录。GFM 表格，不含 HTML；状态写 `**BLOCK**` 这类纯文字；结构与报告语法一致；入口为桌面“导出 ▸ Markdown”和 CLI `--markdown <FILE>`；不进发布包 |
| CLI 文本 | 80 列；区块标题加 `─` 线；数字列右对齐；状态写 `[BLOCK]`；N/A 规则不再显示成 `[PASS]`（当前 gate 输出的一个瑕疵）；保留终端转义 |

### 8.3 不改动的部分
所有 JSON schema、`SHA256SUMS` 与 manifest 格式、发布包内容、`release-notes.md` 的正文（那是用户自己写的）。

---

## 9. 无障碍目标（WCAG 2.2 AA）
- 两种主题下，生成器检查的颜色对都要 0 FAIL、0 豁免。
- 键盘能完成全部流程，焦点可见，浮层关闭后焦点归还。
- 状态永远是图标 + 文字；适配 `forced-colors` 与 `prefers-reduced-motion`。
- 导出 HTML 有语义结构，SVG 有文本替代。
- **R7 由人类实测**：NVDA、Narrator，125%/150% DPI。

---

## 10. 阶段

R0（本文、ADR-0034、原型与样张）→ R1 基础层 → R2 分析与总览 → R3 对比 → R4 发布 → R5 历史（含 D3）、设置、帮助、命令面板 → R6 报告体系（R1 之后可与 R2–R5 并行）→ R7 质量。

每个阶段都要经过：独立评审、Owner 批准 fast-forward 推送、CI 通过、清理 worktree。
