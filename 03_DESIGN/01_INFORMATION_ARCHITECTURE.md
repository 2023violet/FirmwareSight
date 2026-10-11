---
title: "Information Architecture"
doc_id: "FS-DESIGN-002"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-10-10"
---

# Information Architecture

## 一级导航

1. **Overview**
2. **Analyze**
3. **Compare**
4. **Release**
5. **History**
6. **Project Settings**

MVP 若功能尚未实现，不显示空壳导航。

## Overview

目标：回答“这个项目现在准备好了吗？”

内容：
- Current candidate；
- identity summary；
- FLASH/RAM；
- latest comparison；
- gate status；
- unresolved Review/Block。

## Analyze

子区域：
- Summary
- Sections
- Symbols
- Objects
- Evidence

## Compare

- Build A / Build B selector；
- summary deltas；
- section diff；
- symbol diff；
- contributor diff。

## Release

- policy；
- gate results；
- release notes；
- bundle preview；
- export。

## History

P1：
- snapshots timeline；
- release history；
- memory trends。

## Settings

只放项目级设置：
- budgets；
- artifact requirements；
- version rules；
- adapters；
- privacy/network。

不要把 app-level preferences 和 project policy 混在一起。

## 修正案：Settings 承载两种状态，而不是一种（2026-10-10，B1-U1）

上面 `## Settings` 一节做了两件事：把本页限定为"只放项目级设置"，并要求不要把 app-level preferences 和
project policy 混在一起。第二个要求原样保留，B1-U1 正是按它实现的；被改变的是第一个，因为《FirmwareSight｜
完整产品能力、基础体验与专业工作流决策 v0.2》§3 把状态分成三种，而 `firmwaresight.toml` 无法回答"这台机器上的
这个窗口怎么显示数字、下一次打开落在哪一页"——那不是项目的决定，是人和这台机器的决定。

- **Application Preferences**：本机 `settings.json`，v0.6.0 只有 size unit 与起始页两项。存储与校验在
  `apps/desktop/src-tauri/src/preferences.rs`；WebView 只能通过三条 use-case 命令读写它，拿不到路径，也拿不到
  通用文件权限（`AGENTS.md` 7）。
- **Project Policy**：`firmwaresight.toml` 的 budgets、artifact requirements、version rules、adapters、privacy。
  仍然只在 Release Gate 页面出现；Settings 页不写它，也不新建第二个策略文件。
- **Session State**：当前页、当前筛选、已暂存的选择。关窗即失效，不落盘。

边界是可测的，不是一句承诺：偏好不进入 CLI 机读输出、Gate canonical input 与 run id、policy 指纹、release id、
可移植 manifest/JSON、SnapshotId 或 Bundle。守住它的是
`apps/desktop/src-tauri/tests/release_gate.rs::a_stored_preference_moves_no_gate_identity_no_verdict_and_no_policy_fingerprint`
——同一条会话在保存偏好前后各跑一次 Gate，除了写入时刻之外所有字节相同，而 run id 是 canonical input 的摘要，
所以偏好一旦靠近判据就会显形。

本节不改写任何 Accepted ADR，也不删改上面那一节的原文；项目级设置那段仍是历史事实，此处只记录边界从哪里开始变化。
