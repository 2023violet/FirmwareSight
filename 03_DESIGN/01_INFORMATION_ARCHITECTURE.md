---
title: "Information Architecture"
doc_id: "FS-DESIGN-002"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
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
