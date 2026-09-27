---
title: "Visual Design System"
doc_id: "FS-DESIGN-004"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Design"
last_updated: "2026-09-26"
---

# Visual System

## 1. Design tokens

### Spacing
4px base grid.

Scale:
`4 / 8 / 12 / 16 / 24 / 32 / 48 / 64`

禁止随意出现 13px、19px 等无系统间距。

### Radius
- control: 6px
- panel: 8px
- modal: 10px
- pill: only for compact status/tag

### Border
以 1px neutral border 为主。
Shadow 只用于：
- floating menu；
- dialog；
- drag layer。

### Typography

UI stack:
`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial, sans-serif`

Mono:
`ui-monospace, "SFMono-Regular", Consolas, "Liberation Mono", monospace`

建议层级：
- Page title: 20/28 semibold
- Section title: 15/22 semibold
- Body: 13/20 regular
- Table: 12–13/18
- Metadata: 12/18
- Numeric mono: 12–13/18

## 2. Color roles

颜色使用“角色”而不是组件私有颜色：

- `bg.canvas`
- `bg.surface`
- `bg.subtle`
- `text.primary`
- `text.secondary`
- `border.default`
- `accent.primary`
- `status.pass`
- `status.review`
- `status.block`
- `status.unknown`
- `diff.added`
- `diff.removed`
- `diff.changed`

色值以 `assets/design-tokens.json` 为权威草案。

状态不能仅靠颜色，必须同时有 icon + text。

## 3. Layout

Desktop minimum target: `1024x720`  
Primary design target: `1280x800` / `1440x900`

建议：
- navigation rail: 220–240px；
- content max 不强制居中；
- evidence inspector: 320–380px optional；
- tables 优先使用可用宽度。

## 4. Data visualization

优先：
1. table；
2. ranked bars；
3. compact stacked bar；
4. treemap（只用于 size composition）。

禁止：
- 3D chart；
- donut 滥用；
- 没有 baseline 的 sparkline；
- 为装饰添加图表。

## 5. Motion

动画 120–180ms；
只解释：
- panel open；
- row expand；
- state transition；
- drag/drop feedback。

数据刷新不得用大幅 skeleton 闪烁掩盖真实状态。
