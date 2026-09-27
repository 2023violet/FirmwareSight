---
title: "Performance Budgets"
doc_id: "FS-TECH-007"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Performance Budgets

## User-visible targets

### Startup
- warm start target: < 1.5 s on typical dev PC
- 不在 startup 自动扫描所有项目

### Import
- 100 MB ELF：目标 2 s 内完成基础解析（典型开发机）
- 大任务必须后台线程/async，不阻塞 UI

### Table
- 100k symbols 可过滤/滚动，不冻结 UI
- 必要时启用 virtualization

### Diff
- 100k symbol snapshots 目标 1 s 级完成核心 diff

### Export
- 显示 progress；
- 可取消；
- 失败不留下被误认为完整的 bundle。

## Memory

禁止为每个 symbol 保存重复 path/string。
解析层应考虑 interning/Arc 或 compact representation，但只有 profiler 证明需要时优化。

## Benchmark fixtures

至少：
- tiny Cortex-M ELF；
- medium RTOS firmware；
- large LVGL firmware；
- stripped ELF；
- debug-heavy ELF；
- malformed corpus。


## v0.4 Import Memory Guard

Full-buffer parser default refuses artifacts >512 MiB before allocation.
P0 must benchmark memory peak at ~100/256/512 MiB and record:
- wall time
- peak RSS
- normalized structure count
- failure behavior

mmap remains deferred until profiler/customer evidence justifies it.
