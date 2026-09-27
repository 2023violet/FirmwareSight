---
title: "FirmwareSight Design Authority"
doc_id: "FS-DESIGN-ROOT"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Design"
last_updated: "2026-09-27"
---

# FirmwareSight DESIGN.md

## 1. 基调定义

**Instrument-grade clarity — engineered, not stylized.**

FirmwareSight 应像一台可靠的工程仪器：精确、克制、可检查、数据密集但不拥挤。视觉必须服务四个信息支柱：

- **See the build**
- **See the change**
- **Verify the release**
- **Keep the evidence**

上游气质参考为 `VoltAgent/awesome-design-md` 的 IBM `DESIGN.md`，但它只是**气质/结构/模式参考**，不是 IBM 官方 Carbon 规范，也不是 FirmwareSight 数值真源。

## 2. 裁决优先级

冲突时固定按以下顺序：

1. 治理红线：合规措辞、ADR、03_DESIGN 反模式
2. 冻结项目资产：`assets/design-tokens.json`、组件铁律、品牌语调
3. 本 `DESIGN.md`
4. 已接受 UI reference screenshots
5. 上游 IBM/Carbon-inspired reference

因此：**基准定气质，tokens 定数值，红线定禁区。**

## 3. 数值真源

所有色值、字号、间距、圆角、行高、动效时长、focus ring、阴影只引用 `assets/design-tokens.json` 的语义角色。

不得从上游 IBM 文档直接复制数值，不得在组件中私造 magic number。

## 4. 布局与密度

- Light theme 是 MVP 唯一主题。
- 左侧 navigation rail 使用布局 token。
- Evidence Inspector 使用自适应宽度 token；默认宽度是基准，不是硬锁。
- 表格是核心工作面：紧凑、可扫描、可排序、可聚焦。
- Dense, not crowded：靠对齐、分组、发丝边框和排版建立层级，不靠卡片阴影。
- 一屏一个主焦点；首屏不堆 KPI。
- 数字、hash、版本、时间戳、路径使用 mono stack。

## 5. 组件铁律

### Buttons
Primary / Secondary / Ghost 三层；一个区域默认最多一个 Primary。

### Status
唯一语义集合：
`PASS / REVIEW / BLOCK / UNKNOWN / N/A`

任何状态 = icon + label (+ optional count)，颜色仅是辅助。

### Tables
表格按 token 行高；长路径可截断但必须可完整查看/复制；数字稳定对齐。

### Diff Row
必须同时呈现 old / new / delta；Added/Removed 使用缺失符号，不得用 `0` 冒充“不存在”。

### Evidence Inspector
所有重要结论一键可追溯 Evidence。显示 classification/source/location/parser/raw value 等真实可得字段。

### Dialog / Toast
Dialog 只处理真正中断流程的决策；Toast 仅处理短暂确认，错误必须在可恢复位置保留。

## 6. 状态与 Unknown

### PASS
规则已确定求值且满足。

### REVIEW
条件已知，需要人工 disposition。接受 Review 生成审计记录，不改写原始 evidence。

### BLOCK
阻断条件失败。

### UNKNOWN
当前证据不足，无法求值。使用中性灰、空心图标、事实句和下一步。Unknown 永不复用 accent 蓝，也永不自动冒充 PASS。

### N/A
规则对当前对象/场景不适用。与 Unknown 不同；手工 N/A 只有规则允许时才能使用，并记录 actor/time/reason。

## 7. 图表

优先顺序：
`table > ranked bars > compact stacked bar > treemap`

Treemap 只用于 size composition。禁止 3D、装饰 donut、无 baseline sparkline、彩虹分类色。单色蓝阶不能作为唯一编码，必须同时依赖标签、位置、长度或数值。

## 8. 动效

只允许 productive motion：面板开合、行展开、状态迁移、拖放反馈。时长与 easing 取自 tokens，并尊重 reduced-motion。

禁止装饰性漂浮、发光、渐变动画、过场表演。

## 9. Do's & Don'ts

### MUST
- Evidence 随时可到达
- 数字带上下文
- Unknown 是一等状态
- 状态 icon + label
- focus-visible 清晰
- 键盘路径与视觉顺序一致
- 错误包含 What happened / Why we know / What to do / Diagnostics ID

### MUST NOT
- 渐变
- 玻璃拟态
- 霓虹/发光
- 紫色系品牌视觉
- 卡片/面板/表格行阴影
- 营销 Hero/CTA/装饰插画模式
- 仅用颜色表达状态
- MVP Dark theme
- UI 复刻 parser/diff/gate 逻辑
- 把 “Can we ship now?” 表述为法律、网络安全或合规认证结论

## Upstream anchor

- Repository: `VoltAgent/awesome-design-md`
- Path: `design-md/ibm/DESIGN.md`
- Pinned blob: `dbef1c5b20357a8953e6832e5d383a866a66f11a`
- Upstream role: temperament/pattern reference only
- Sync: quarterly manual review only; never automatic


## 10. Post-MVP presentation governance

`03_DESIGN/08_POST_MVP_PRESENTATION_RULES.md` freezes presentation rules for future candidates without authorizing the candidates themselves.

Additional invariants:
- chart = table magnifier, never decorative replacement;
- deltas are facts, not green/red judgments;
- History trends use honest axes and stable ordering;
- capability banner uses one evidence source per slot;
- change facts appear before Gate verdicts;
- evidence class stays attached to the value;
- external ports must return users to Analyze / Compare / Gate / Release evidence;
- notifications optimize engineering relevance, not engagement loops.
