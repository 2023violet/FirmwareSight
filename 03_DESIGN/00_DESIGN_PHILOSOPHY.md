---
title: "Design Philosophy"
doc_id: "FS-DESIGN-001"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Design"
last_updated: "2026-09-26"
---

# Design Philosophy

## 1. 核心理念：Instrument-grade clarity

FirmwareSight 应该像工程仪器，而不是营销 Dashboard。

### 设计目标
- 快速建立信息层级；
- 让差异和异常显眼；
- 让“证据来源”随时可追溯；
- 数据多但视觉噪声低；
- 用户能够判断，而不是被 UI 替用户判断。

## 2. 五条设计原则

### P1 — Evidence is one click away
任何 Gate、组件、版本结论都应能查看 Evidence。

### P2 — Numbers need context
`+8.4 KiB` 必须同时回答：
- 相对谁；
- 占总量多少；
- 谁贡献最多；
- 是否超过 budget。

### P3 — Unknown is a first-class state
不能把 unknown 隐藏为灰色空白。

### P4 — One primary focus
一个界面只允许一个主要操作焦点。

### P5 — Dense, not crowded
专业工具允许高信息密度，但依靠：
- 分组；
- 对齐；
- spacing；
- typography；
而不是更多卡片、阴影和颜色。

## 3. 视觉反模式

禁止：
- 大面积玻璃拟态；
- 紫蓝渐变做“科技感”；
- 每个数据一张 card；
- 大量圆角胶囊；
- 过度动画；
- 状态完全依赖颜色；
- 为漂亮牺牲表格可读性；
- Dashboard 首屏塞满 KPI。

## 4. 主题策略

MVP 优先 Light Theme。
原因：
- 工程表格/报告可读性；
- 打印/截图；
- 降低双主题 QA 成本。

Dark Theme 在 design tokens 稳定后进入 P1。
