---
title: "Component Rules"
doc_id: "FS-DESIGN-005"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Component Rules

## Button
层级仅三种：
- Primary
- Secondary
- Ghost

Destructive 是语义修饰，不是第四套视觉体系。

一个区域默认最多一个 Primary。

## Status
统一状态：
- PASS
- REVIEW
- BLOCK
- UNKNOWN
- N/A

每个状态由：
icon + label + optional count
组成。

## Table
必须支持：
- keyboard focus；
- column sorting；
- stable numeric alignment；
- mono numerals；
- empty state；
- long path truncation + tooltip/copy。

不要给每行加 card。

## Evidence Inspector
右侧抽屉统一展示：
- conclusion；
- classification；
- source；
- location；
- parser；
- raw value；
- timestamp/hash；
- confidence（若适用）。

## Diff Row
必须同时展示：
old / new / delta。

Added / removed 时不要用 0 冒充不存在。

## Dialog
只有真正需要中断主流程的操作使用。
普通配置优先 inline panel / sheet。

## Toast
只用于短暂确认：
- copied；
- exported；
- saved。

错误不可只用 Toast，必须在可恢复位置保留。
