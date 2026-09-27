---
title: "Coding Agent Operating Rules"
doc_id: "FS-GOV-AGENTS"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-26"
---

# AGENTS.md

对所有 AI Coding Agent、自动化编程工具和人工贡献者生效。

## 1. 接手顺序

开始任何任务前必须读取：

1. `README.md`
2. `.ai/README.md`
3. `.ai/CURRENT_STATE.md`
4. `.ai/DECISIONS.md`
5. `.ai/ACTIVE_TASK.md`
6. `04_TECH/09_TECH_DECISION_MATRIX.md`
7. 与任务相关 ADR / spec

若 `ACTIVE_TASK.md` 为 `NONE`，不得自行创建业务功能。

## 2. 技术基线不可静默改变

未经 ADR 不得：
- 把 Rust Core 改为 async-first；
- 把 Tauri 2 换成其他桌面框架；
- 把 React 换成其他前端；
- 把 rusqlite 换成 SQLx；
- 在 Core 引入 Tauri/Tokio/SQLite；
- 引入 HTTP Client；
- 引入 wgpu；
- 加 server/cloud/auth/telemetry；
- 改 artifact evidence 分类；
- 改持久化 schema 语义；
- 改 updater/signing 模型。

## 3. Core-first 规则

`firmwaresight-core`：
- MUST be headless；
- MUST be UI independent；
- MUST NOT depend on Tauri；
- MUST NOT depend on React/JS；
- MUST NOT depend on SQLite；
- MUST NOT expose Tokio types；
- SHOULD remain deterministic and mostly synchronous；
- SHOULD forbid project-authored unsafe code。

业务事实从 Core 输出；UI 不得重新实现 diff/gate/parser 逻辑。

## 4. Dependency discipline

新增依赖必须说明：
- 当前哪一项用户需求需要它；
- 标准库/现有依赖为什么不够；
- license；
- maintenance；
- binary/build impact；
- 是否进入 trusted core。

禁止因为“Rust 生态常用”而预装：
- Tokio everywhere；
- Reqwest/Rustls；
- wgpu；
- SQLx；
- Axum；
- Tonic。

## 5. Async rules

- CLI MVP default synchronous。
- Desktop long-running local work runs off UI thread。
- Blocking parser/hash/SQLite transaction 不直接放 async executor worker。
- 若使用 Tokio，只允许 Application/IO edge。
- Domain signature 不出现 `tokio::*`。

## 6. Storage rules

- SQLite via `rusqlite + bundled`。
- 所有 schema change 走 migration。
- 结构化历史数据不得长期散落 JSON 文件。
- JSON 仅做 config/interchange/export/golden fixture。
- Release artifact 原始文件不默认复制进 database。

## 7. Tauri security rules

- Web frontend 不获得通用 shell 权限。
- Web frontend 不获得通用 filesystem 权限。
- Tauri commands 必须是 use-case oriented，而不是“read arbitrary file”。
- capability 最小化。
- MVP 不启用 updater/network plugin。
- updater 只有在 signing/key-management ADR 完整后启用。

## 8. Evidence First

每个事实属于：
- Observed
- Derived
- Declared
- Unknown

不得把推测写成 Observed。

## 9. 高风险操作

必须人工确认：
- 删除文件/数据/分支；
- force push/history rewrite；
- schema destructive migration；
- security/capability change；
- signing/update key change；
- default network upload；
- license change；
- 不可逆操作。

## 10. Definition of Done

- acceptance criteria complete；
- Rust fmt/clippy/test；
- frontend typecheck/lint/test；
- relevant fixture regression；
- no parser panic on user input；
- no unrelated dependency；
- docs match behavior；
- error state inspectable；
- privacy/local-first boundary preserved。


## 11. UI Rules

由 ADR-0018 设立。对桌面 React UI、Release Bundle HTML 等用户可见输出生效。

- UI 任务必读：根 `DESIGN.md` + `assets/design-tokens.json`。
- 冲突裁决：治理红线 > 冻结资产/ADR/tokens > DESIGN.md > accepted screenshots > upstream reference。
- 数值必须来自 token；禁止 magic colors/spacing/radius/motion。
- MUST NOT：渐变、玻璃拟态、霓虹/发光、紫色系、面板/表格阴影、营销 Hero/CTA、纯颜色状态、MVP Dark theme。
- 五态：PASS / REVIEW / BLOCK / UNKNOWN / N/A = icon + label (+ optional count)。
- Unknown：中性灰、空心图标、事实句、下一步，不复用 accent。
- 数字/hash/version/timestamp/path 使用 mono。
- UI 不得复刻 parser/diff/gate 逻辑。
- UI PR 必须通过 `templates/DESIGN_CHECKLIST_TEMPLATE.md`。
