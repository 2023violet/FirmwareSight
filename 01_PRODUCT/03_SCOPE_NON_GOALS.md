---
title: "Scope and Non-Goals"
doc_id: "FS-PRD-004"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Scope / Non-goals

## In Scope — MVP

- 静态读取构建产物；
- memory / symbols / sections；
- build identity；
- build comparison；
- release rules；
- release evidence package；
- 本地 project workspace；
- CLI 与 Desktop 共用 Rust Core。

## Explicitly Out of Scope

### 不做 IDE
不编辑 C/C++ 源码，不做 LSP，不做编译器前端。

### 不做 Debugger
不实现 JTAG/SWD/GDB server。

### 不做 Flasher
MVP 不烧录 MCU。

### 不做 HIL
不控制示波器、电子负载、DAQ。

### 不做 Firmware Reverse Engineering Suite
不做 binwalk 替代品，不做漏洞利用分析。

### 不做 Cloud SBOM Platform
MVP 不要求账号/云端。

### 不做“AI 判断能否发布”
Release Gate 必须是 deterministic policy。

### 不承诺所有 ELF 都等价
ELF 是容器格式，不同工具链携带的信息不同。界面必须显示 evidence availability。

## Scope Creep 判定

出现“顺便支持……”时必须回答：
1. 是否直接提升 Analyze/Compare/Gate/Release？
2. 是否需要新增协议/硬件/运行时？
3. 是否可在现有 fixture 中验证？
4. 是否延迟 MVP？

若 1=否 或 4=是，默认推迟。
