---
title: "Accessibility Baseline"
doc_id: "FS-DESIGN-006"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Accessibility

## 基线目标

桌面 UI 以 WCAG 2.2 AA 的可操作原则作为设计参考，即使产品不是传统 Web 站点。

必须：
- keyboard accessible；
- visible focus；
- 文字/背景对比充足；
- 不只靠颜色编码；
- 表格 header 有语义；
- icon button 有 accessible name；
- dialog focus trap；
- screen-reader 可理解主要状态。

## 键盘

- `Ctrl/Cmd+O`: Open artifact/project
- `Ctrl/Cmd+K`: Command palette（P1）
- `/`: focus filter when table active（可选）
- `Esc`: close transient layer
- Tab order 与视觉顺序一致。

## Reduced Motion
尊重 `prefers-reduced-motion`。

## 字体
不把 11px 以下作为主要工作信息。
