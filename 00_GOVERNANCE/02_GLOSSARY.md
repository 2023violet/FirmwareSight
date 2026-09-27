---
title: "Project Glossary"
doc_id: "FS-GOV-003"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# 术语表

| 术语 | 项目定义 |
|---|---|
| Artifact | 一次构建产生的 ELF/MAP/BIN/HEX 等文件 |
| Build | 一组来自同一次编译/链接过程的 Artifact 与元数据 |
| Build Snapshot | FirmwareSight 对一次 Build 的规范化只读快照 |
| Release Candidate | 用户准备发布但尚未确认的 Build |
| Release Gate | 一组确定性的发布前检查 |
| Evidence | 支持某个事实或判断的来源记录 |
| Observed | 从输入直接读取的事实 |
| Derived | 用确定性规则从 Observed 数据推导的事实 |
| Declared | 用户人工声明的信息 |
| Unknown | 当前证据不能支持的状态 |
| Provenance | Artifact 的来源链：Git、工具链、构建时间、哈希等 |
| Build Identity | 能唯一描述一次 Build 的核心元数据 |
| Release Bundle | 可移交、可保存的发布证据包 |
| Component Evidence | 支持“某组件/版本被包含”的证据 |
| SBOM | Software Bill of Materials |
| Gate Finding | 某个 Release Gate 检查的结果 |
| Blocker | 按项目策略阻止发布的结果 |
| Review | 需要人工确认，不自动判定通过/失败 |
| Project Workspace | FirmwareSight 针对一个固件项目保存的本地工作区 |
| Adapter | 针对某类工具链/格式的解析适配器 |
