---
title: "AGENTS.md UI Rules 增补节（入仓准备件）"
doc_id: "ADR-0018-ANNEX-AGENTS"
product: "FirmwareSight"
version: "0.3.0"
status: "ACCEPTED"
owner: "Engineering"
last_updated: "2026-09-27"
---

# AGENTS.md 增补节：UI Rules（入仓准备件）

> 起草：通用助手 ｜ 依据：ADR-0018 D1/D5 + 设计匠人《FS-UI基调定案提案》5.1 第 3 条 ｜ 性质：纯文档治理变更，不触碰 Active Task = NONE。
> 本文件**不直接改动基线副本** `/home/z/my-project/firmwaresight/FirmwareSight_Project_Baseline_v0.3.0/AGENTS.md`，入仓时按下述方式合并。

---

## 一、入仓合并方式（建议）

1. **位置**：将下节正文追加为基线 `AGENTS.md` 的 **§11 "UI Rules"**（现 §1–§10 编号与顺序零改动，避免破坏既有文档对节号的交叉引用）。由 ADR-0018 D5 设立并注明出处。
2. **连带改动（唯一一处）**：§1 "接手顺序"清单第 7 条 `与任务相关 ADR / spec` 之前插入一行 `7. 根目录 DESIGN.md（UI 任务必读）+ assets/design-tokens.json`，原第 7 条顺延为第 8 条。若维护者希望节号完全零扰动，也可不改 §1，仅在 §11 正文中声明"接手 UI 任务前必读"（本文正文两种写法均已覆盖）。
3. **版本记号**：入仓时 front matter `last_updated` 更新为入仓日；`CHANGELOG.md` 增补一行（类型：Docs/Governance，关联 ADR-0018）。

---

## 二、增补节正文（以下内容直接并入 AGENTS.md）

```markdown
## 11. UI Rules

由 ADR-0018 设立。对桌面 React 前端、Release Bundle（report.html 等对外产物）与一切面向用户的界面输出生效。接手 UI 任务前必读：根目录 `DESIGN.md` + `assets/design-tokens.json`。

- 所有 UI 遵循根目录 `DESIGN.md`。
- 冲突裁决顺序：**治理红线（07_COMPLIANCE 措辞红线 / 03_DESIGN 反模式黑名单 / ADR 决策）> 冻结资产（design-tokens.json / 03_DESIGN 组件铁律 / 02_BRAND 语调）> 根目录 DESIGN.md**。
- 视觉事项与本文其他节冲突时，以 DESIGN.md 为准；与治理红线冲突时，红线为准。
- 数值纪律：色值、字号、间距、圆角、动效时长只取自 `design-tokens.json`（v0.2.0+），不得私自造色造值。

硬禁区（MUST NOT）：
- 渐变（任何形式，含图表渐变填充）；
- 玻璃拟态（frosted glass / backdrop blur 面板）；
- 霓虹 / 发光效果；
- 紫蓝系配色（紫色系一律不进入 UI；蓝仅 accent #2563EB 及其蓝阶）；
- 卡片 / 面板 / 表格行投影（阴影仅限浮层两级 overlay-sm / overlay-lg）；
- 营销面模式（hero band / CTA banner / 案例卡片 / 装饰插画）;
- 仅用颜色编码状态；
- Dark 主题（MVP 仅 Light）。

状态与数字：
- 状态五态 PASS / REVIEW / BLOCK / UNKNOWN / N/A = icon + label (+ optional count)；Unknown 为一等状态（中性灰 + 空心图标 + 事实句 + 下一步），永不复用 accent 蓝。
- 表格数字、哈希、版本号、时间戳、路径一律 mono；表格行高默认 36px / 紧凑 28px / 上限 40px。

UI 不得复刻 parser / diff / gate 逻辑（重申 §3）。

UI 变更的 PR 必须通过 `templates/DESIGN_CHECKLIST_TEMPLATE.md`（FS-TPL-005）评审；命中 Don'ts 任一项直接打回。
```

---

## 三、与其他文档的关系（入仓核对清单）

| 文档 | 关系 |
|---|---|
| ADR-0018 | 本节的设立依据；裁决顺序与禁区清单的权威出处 |
| 根目录 DESIGN.md（设计侧产出） | 本节指向的视觉权威；冲突时视觉以它为准（红线除外） |
| design-tokens.json v0.2.0（设计侧产出） | 唯一数值真源；本节"数值纪律"指向它 |
| templates/DESIGN_CHECKLIST_TEMPLATE.md（FS-TPL-005） | 本节设定的 PR 评审门禁 |
| ADR-0011（前端基线） | 不变；本节不引入组件库依赖，仅约束自有 UI 的呈现 |
| §3 Core-first 规则 | 不变；"UI 不得复刻 parser/diff/gate 逻辑"为重申 |
