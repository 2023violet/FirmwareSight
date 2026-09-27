---
title: "ADR-0018 UI Baseline and Design Tokens Governance"
doc_id: "ADR-0018"
product: "FirmwareSight"
version: "0.3.0"
status: "ACCEPTED"
owner: "Architecture"
last_updated: "2026-09-27"
---

# ADR-0018 — UI 基准与 design tokens 治理

> 入仓准备件（通用助手起草，2026-09-27）。依据：设计匠人《FS-UI基调定案提案》5.1/5.3 节 + 基线 ADR_TEMPLATE 结构。入仓位置：`09_ADR/ADR-0018-ui-baseline-tokens-governance.md`；入仓时 front matter `status` 由 ACCEPTED 改为 BASELINE，并同步 `INDEX.md` / `manifest.txt`。

## Status

Accepted（2026-09-27，用户确认 UI 基调提案 4 个开放问题全部按默认建议定案）。待入仓。

## Context

1. **缺气质基准**。v0.3.0 基线已有 03_DESIGN 六篇与 `assets/design-tokens.json`，但没有一份统一的"气质基准"文档——AI 编码代理与人工贡献者实现 UI 时没有共同风格参照，视觉漂移不可避免。
2. **版本错位 + token 缺口**。`design-tokens.json` 自标 version 0.1.0 / status `draft-design-tokens`，却被 03_DESIGN/03_VISUAL_SYSTEM.md 指定为"权威草案"（v0.3.0 基线内）。同时该文件缺 10 类必要 token（accent 交互态、焦点环、review 小字变体、diff 浅底、浮层阴影、pill 半径、表格行高三档、动效三档、图表蓝阶、disabled 文字色）——组件库落地时必然出现"私自造色"。
3. **状态语义矛盾**。02_BRAND/02_VOICE_MESSAGING.md 定义 4 个状态，03_DESIGN/04_COMPONENT_RULES.md 定义 5 个（多 N/A），需裁定权威版本。
4. **提案已定案**。设计侧《FS-UI基调定案提案》经信息哨兵 74 份风格逐份取证、立项记录 8 条隐含 UI 气质约束交叉论证；用户已确认 4 个开放问题全部按默认建议定案。

## Decision

### D1 气质基准：IBM Carbon 单基准 + FirmwareSight Overrides

采纳 IBM Carbon（源自 awesome-design-md `design-md/ibm/DESIGN.md`，pin commit hash 记录于 DESIGN.md 页脚）为**唯一气质基准**，剥离营销面模式、叠加项目自有约束后改造为仓库根目录 `DESIGN.md`。三者分工固定：

```
基准（根目录 DESIGN.md）定气质 —— 结构与模式参考
tokens（design-tokens.json）定数值 —— 唯一数值真源
红线（07_COMPLIANCE 措辞红线 / 03_DESIGN 反模式黑名单 / ADR 决策）定禁区
```

**冲突裁决顺序：治理红线 > 项目冻结资产（design-tokens.json / 03_DESIGN 组件铁律 / 02_BRAND 语调）> DESIGN.md**。AGENTS.md 增补 UI Rules 节固化此顺序；视觉事项与 AGENTS.md 其他节冲突时以 DESIGN.md 为准，与治理红线冲突时红线为准。

### D2 关键裁决值（基准与冻结资产冲突处，基准让位）

| 维度 | 裁决 |
|---|---|
| 渐变 | 全禁，零容忍（含图表渐变填充） |
| 圆角 | 维持 6/8/10 + pill 999（仅紧凑状态标签）；拒绝 Carbon 0px 直角 |
| accent | 维持 #2563EB（兼 diff.changed，全家唯一蓝）；拒绝 IBM #0f62fe；不引入 Carbon info 蓝状态位 |
| 表格行高 | 默认 36px / 紧凑 28px / 上限 40px；拒绝 Carbon 48px |
| 字体 | MVP 系统栈 + 系统 mono（Windows 首要平台自然落到 Segoe UI）；IBM Plex 列 P1 候选 |
| 动效 | 120/150/180ms 三档，统一 ease-out，仅限功能性动效；剔除 Expressive motion |
| 灰阶 | 维持现有 5 阶；不引入多级灰阶体系 |
| 阴影 | 仅浮层两级（overlay-sm / overlay-lg）；卡片、面板、表格行永不投影 |
| Unknown | 中性灰 + 空心图标 + 事实句 + 下一步，一等状态；永不复用 accent 蓝 |
| 主题 | MVP 仅 Light；Dark 为 P1，届时全量重验对比度 |

### D3 状态语义权威版本

组件 **5 态（PASS / REVIEW / BLOCK / UNKNOWN / N/A），呈现 = icon + label + optional count**（03_DESIGN/04_COMPONENT_RULES.md）为权威。02_BRAND/02_VOICE_MESSAGING.md 的 4 态表述自本 ADR 起被覆盖；N/A 的语义与视觉（规则因证据缺失或场景不适用而不评估）在 DESIGN.md 状态条款中补充说明。

### D4 design tokens 升版 v0.1.0 → v0.2.0（版本错位豁免）

`design-tokens.json` 自标 v0.1.0 `draft-design-tokens` 却被 v0.3.0 基线指定为"权威草案"的版本错位，经本 ADR 一次性记录并豁免：**升版至 v0.2.0，status 改为 baseline-authoritative**，同时补齐 10 类缺口——accent.hover/.pressed、focus.ring（2px + offset 2px，仅 focus-visible）、status.review.strong #7A5200、diff.added/removed/changed.bg、shadow.overlay-sm/lg、radius.pill、table.row default/compact/max、motion.duration micro/standard/panel + ease、chart.seq.blue 五档单色蓝阶、text.disabled。**14 个既有语义角色色值全部保留不动**。升版文件由设计侧另行产出（放 shared/ 待入仓，不覆盖基线副本），入仓前须过机器对比度验证。

### D5 配套治理产物

1. 根目录 `DESIGN.md`（九节结构，只引用 token 语义名，不另立数值）——设计侧产出；
2. `AGENTS.md` 增补 UI Rules 节（权威声明 + 裁决顺序 + 禁区清单）——本文配套件；
3. `templates/DESIGN_CHECKLIST_TEMPLATE.md`（FS-TPL-005，Do's & Don'ts，可机械核验）——本文配套件。

上游锚定：pin commit hash 记录于 DESIGN.md 页脚；**每季度人工 diff 一次，不自动同步**；关键值与 Carbon 官方文档（carbondesignsystem.com）复核一次后即可脱离上游独立演进。

## Alternatives

- **Vercel 基准**：气质相符但需手术式剥离品牌渐变，200 级灰阶维护成本与克制精神相悖 → 不采纳；仅借"层级靠表面色差、不靠阴影"单点条款。
- **WIRED 基准**：审计页密度优秀，但三字体体系不可商用、衬线个性与仪器感有温差 → 不采纳；仅借审计密度条款。
- **Cal.com 基准**：气质正确，但状态语义与 token 完备度系统性弱于 Carbon → 不采纳。
- **纯自研基准**：行高、灰阶、焦点、阴影每个细节都要重新争论，放弃行业共识 → 不采纳。
- **tokens 不升版、仅加 BASELINE 注记**：消除错位表述最快，但 10 类缺口仍在、私自造色动因不除 → 用户否决，选择 ADR 一次定案。

## Consequences

**正面**
- AI 编码代理与人类评审共用同一视觉语言与裁决顺序；UI 评审有 checklist 可依（FS-TPL-005）。
- tokens 单一数值真源 + 缺口补齐，消除私自造色动因；5 态权威消除文档矛盾。
- 本基调 + tokens v0.2.0 直接作为 G1"极简技术 UI"输入，G2"Minimum Credible UI"验收共用同一 checklist——两阶段一套基调，不返工。
- 全程为文档治理变更，不创建业务功能，不违反 Active Task = NONE。

**负面 / 代价**
- 基准源自对 ibm.com 的逆向提取（非 IBM 官方发布物），关键 token 落地前须与 Carbon 官方文档复核一次。
- 新增 token 值（#7A5200、diff 浅底草案等）须过机器对比度验证后方可定稿。
- DESIGN.md 与 design-tokens.json 双文件需同步维护；DESIGN.md 严禁写入数值，漂移风险由 checklist 与评审拦截。
- ADR-0011"no full UI kit"不变：本 ADR 只引入气质基准文档，不引入任何组件库依赖。

## Revisit trigger

- **Dark 主题启动（P1）**：全量重验 14 色 + 新增色对比度，token 扩展须增补本 ADR 或新 ADR。
- **IBM Plex 引入（P1）**：字体打包与 fallback 策略须新 ADR。
- **季度上游 diff** 发现 awesome-design-md 关键语义变更：人工评估是否跟进，不自动同步。
- **未来引入真实组件库**（如 Carbon React）替换自研组件：新 ADR 重议基准形态与依赖纪律。
