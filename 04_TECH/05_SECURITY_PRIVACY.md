---
title: "Security and Privacy Model"
doc_id: "FS-TECH-006"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Security"
last_updated: "2026-09-26"
---

# Security & Privacy

## Default posture

- Local-first
- Network off by default for core workflows
- No account required for MVP
- No source code upload
- No artifact upload
- No background telemetry by default

## Tauri boundary

前端只能通过明确 command 调用 Rust。
Capabilities 按 window 最小授权。

文件系统：
- 用户显式选择的 path；
- project workspace；
- export destination。

禁止 frontend 获得任意磁盘访问。

## Untrusted files

ELF/MAP/HEX 均视为不可信输入。

要求：
- parser panic = bug；
- path traversal prevention；
- report HTML escape；
- bundle filename normalization；
- zip/export bomb protections when future archive import arrives。

## Secrets

MVP 不需要 secrets。
未来 vulnerability API token：
- OS credential store；
- 不进 config；
- 不进 logs；
- 不进 reports。

## Network

未来如果增加：
- update check；
- CVE feed；
- licensing；

必须分别列出 endpoint、purpose、data sent、disable switch。

## Telemetry

如未来启用：
- opt-in or explicit setting；
- 不收集文件名、路径、symbol、firmware hash，除非有明确必要和说明；
- crash report 需 scrub path。
