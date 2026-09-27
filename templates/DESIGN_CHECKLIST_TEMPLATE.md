---
title: "Design Review Checklist Template"
doc_id: "FS-TPL-005"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Design"
last_updated: "2026-09-27"
---

# FirmwareSight 设计评审 Checklist（Do's & Don'ts）

> v0.4.0 baseline checklist；适用于 UI 变更 PR、V0/G1 技术 UI 与 G2 Product MVP 设计评审。
> 依据：ADR-0018 ｜ 根目录 `DESIGN.md` ｜ `assets/design-tokens.json` v0.2.0+
> 用途：UI 变更 PR 评审 ／ G1"极简技术 UI"检查 ／ G2"Minimum Credible UI"验收，三处共用同一份。
> 使用规则：**Don'ts 一票否决，任一命中直接打回，不接受"下版再改"**；Do's 逐项勾选；标注 ⚙ 的项可机械核验（脚本/CI 可查，方法见文末）。

## Don'ts（禁区 · 一票否决）

- [ ] ⚙ 无渐变：样式与图表均无 `linear-gradient` / `radial-gradient` / `conic-gradient`
- [ ] ⚙ 无玻璃拟态：无 `backdrop-filter` / frosted glass 面板
- [ ] ⚙ 无霓虹 / 发光：无 neon 色值、无发光 box-shadow / 荧光 text-shadow
- [ ] ⚙ 无紫蓝系：紫色系色值一律不出现；蓝仅 accent #2563EB 及其蓝阶（含 diff.changed）
- [ ] ⚙ 无投影越界：卡片 / 面板 / 表格行无 box-shadow（阴影仅 overlay-sm / overlay-lg 两级浮层）
- [ ] 无营销面：无 hero band / CTA banner / 案例卡片 / 装饰插画
- [ ] 状态非纯颜色编码：任何状态均有 icon + label，颜色只作辅助
- [ ] 无 Dark 主题实现（MVP 仅 Light）
- [ ] 无数值私造：样式值均来自 design-tokens.json，无魔法数（token 缺口须走升版，不许就地造值）
- [ ] UI 未复刻 parser / diff / gate 逻辑（业务事实只来自 Core）
- [ ] 禁词合规：无 guaranteed / fully compliant / zero risk / intelligent detection（若实为规则）等语调禁词

## Do's（应做 · 逐项勾选）

### 状态与证据

- [ ] ⚙ 状态五态齐备且呈现正确：PASS / REVIEW / BLOCK / UNKNOWN / N/A，每态 = icon + label (+ optional count)
- [ ] ⚙ 交互态齐备：可交互元素具备 hover / pressed / focus / disabled（确实不适用的角色须显式标注 N/A，不得缺失）
- [ ] ⚙ 焦点环：2px accent + 2px offset，`focus-visible` 必现，不被移除或弱化
- [ ] Unknown 呈现合规：中性灰 + 空心图标 + 事实句（如 "Detected / version unknown"）+ 下一步动作；未用蓝、未隐藏、未冒充 Observed
- [ ] 错误四要素齐备：What happened / Why we know / What to do / Diagnostics ID，驻留可恢复位置
- [ ] Dialog 仅用于真正中断；Toast 仅瞬时确认；无弹窗恐吓式报错
- [ ] 空态给出行动出口（不是灰白空白）
- [ ] 推荐下一步始终可见（说事实、给下一步，不替用户判断）

### 数字与密度

- [ ] ⚙ mono 数字：表格数字 / 哈希 / 版本号 / 时间戳 / 路径一律 mono 栈
- [ ] ⚙ 行高 ≤40px：表格默认 36px / 紧凑 28px，无超限行
- [ ] ⚙ 间距落在 4px 网格八档（4–64）；圆角 ∈ {6, 8, 10, pill(999，仅紧凑状态标签)}
- [ ] ⚙ 字号 ≥12px（无 11px 及以下）
- [ ] ⚙ 动效时长 ∈ {120, 150, 180}ms + ease-out，且仅限面板开合 / 行展开 / 状态迁移 / 拖放反馈
- [ ] ⚙ 尊重 `prefers-reduced-motion`
- [ ] Diff Row 三列齐备：old / new / delta 同列；added / removed 未用 0 冒充
- [ ] 关键数字带上下文：相对谁 / 占比 / 最大贡献者 / 是否超预算（至少其一）
- [ ] 一屏一个主焦点；KPI 不塞满首屏
- [ ] review 琥珀用于 12px 表格小字时使用加深变体 status.review.strong

### 图表与可访问性

- [ ] 图表优先级合规：table > ranked bars > compact stacked bar > treemap（仅 size 构成）
- [ ] 图表反模式为零：无 3D、无滥用 donut、无无基线 sparkline、无彩虹分类色；多色仅单色蓝阶
- [ ] ⚙ icon button 有 accessible name；表格语义化表头
- [ ] Tab 顺序 = 视觉顺序；dialog 有 focus trap；Esc 关浮层
- [ ] 能力 banner（如 ELF ✓ · MAP 未提供 · Git 未链接）如实反映证据状态，未夸大

## 机械核验方法（⚙ 项的脚本化建议）

| 检查 | 建议手段 |
|---|---|
| 无渐变 / 无 backdrop-filter / 无发光 | CI 对 UI 源码 grep 关键 CSS 属性，命中即 fail |
| 紫蓝系 / 非法色值 | lint 校验色值仅来自 design-tokens.json 白名单（或 CSS var 引用），硬编码色值即 fail |
| 交互态 / 焦点环 | 组件约定 + 样式审计；e2e 键盘走查抽查 |
| 五态呈现 | 组件 props 断言（status 组件必须传 icon+label） |
| mono 数字 / 行高 / 字号 / 圆角 / 动效时长 | 样式规则 lint（对 tokens 值做断言） |
| reduced-motion | e2e 模拟偏好后断言动画关闭 |

> CI 起步阶段允许人工评审；G2 前应将 ⚙ 项逐步脚本化；脚本化本身不改变本模板的判定标准。
