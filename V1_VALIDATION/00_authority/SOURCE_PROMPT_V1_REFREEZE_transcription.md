---
title: "V1 Cohort Re-freeze — Agent Transcription of the Delivered Authorization, Not a Delivered File"
doc_id: "FS-V1-RF-000A"
product: "FirmwareSight"
version: "0.6.0"
status: "TRANSCRIPTION"
authority: "transcribes the owner's inline authorization of 2026-10-08 for unit V1_COHORT_REFREEZE"
---

# V1 Cohort Re-freeze — Execution Authorization v1.0 (transcription)

**What this file is, and what it is not.** The authorization below was delivered **inline as chat message text**,
not as a file, so there is no owner-side byte stream to hash. `10_AUDIT/SOURCE_PROMPTS/README.md` records the round
with `File: none (delivered inline)`, the way the three inline P0 precedents and the inline U1 prompt are recorded.
This is an agent transcription made after receipt. **Its SHA-256 is the digest of this transcription and must never
be cited as the digest of the delivered authorization.** Where transcription and the delivered message differ, the
delivered message governs. The one deliberate difference is this front-matter block and this paragraph: they are
added for provenance and are not part of what was delivered.

Transcription method: section titles, section order, numeric identifiers, hashes, file paths and the stop conditions
were copied literally; line wrapping was not reconstructed; no clause was added, removed or softened.

---

# FirmwareSight — V1 Cohort Re-freeze
## Execution Authorization v1.0

### 任务性质

本轮仅授权执行 `V1_COHORT_REFREEZE`。

目标：在保留所有历史证据、产品功能及测试契约不变的前提下，将 V1 正式研究构建从旧 F3 安装包重新冻结到经 U1
视觉验收的候选版本，为后续真实用户验证建立统一、可追溯的版本权威。

本轮不授权开始参与者会话，不授权公开分发，不授权 B1、RC 或 GA。

### 一、必须先完成只读接手

按 AGENTS.md §1 读取：

1. README.md
2. .ai/README.md
3. .ai/CURRENT_STATE.md
4. .ai/DECISIONS.md
5. .ai/ACTIVE_TASK.md
6. 04_TECH/09_TECH_DECISION_MATRIX.md

然后读取：

- V1_VALIDATION/ 的研究协议与版本身份
- U1_VALIDATION/U1P_R3_ARCHITECT_FINAL_VERDICT_RECORD.md
- P5_VALIDATION/P5_RELEASE_READINESS.md
- BASELINE.yaml
- 相关 ADR

确认 HEAD、origin/main、工作树状态及当前远程 CI。

如发现已有未经提交的修改，不得清理、覆盖、stash、reset 或删除。

### 二、重新核对新旧研究构建

原冻结版本：

- Artifact ID: 11419727517
- NSIS SHA-256: 182506f213383cfe00865f199fcec4fb17079535e8ea370f954fc15097263d12

拟冻结版本：

- Product commit: 41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177
- Product CI: 37828673549
- Artifact ID: 11573661113
- NSIS SHA-256: 9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93

必须验证下载字节、内部 SHA256SUMS、产品版本、产品提交身份和 CI 结果。

不得将重新构建的二进制、仅文档更新后的 HEAD 或其他产物自动当作冻结研究版本。

### 三、允许的工作

- 建立明确的 V1 Re-freeze 授权记录。
- 更新 V1 研究协议中受此次换版影响的构建身份和对应说明。
- 更新 BASELINE.yaml 及相关 AI handoff、研究状态记录。
- 保存原 F3 冻结版本的不可变历史。
- 标明换版理由、影响范围、旧版与新版差异。
- 保留 U1 视觉验收 25 项原始计数及偏差 A–D。
- 使用项目规定的 staged-index 方式更新目录索引与校验清单。
- 完成授权范围内的本地检查及远程 CI 验证。

### 四、禁止事项

- 不修改 apps/、crates/、schemas/、migrations/、设计 tokens 或产品业务代码。
- 不增加依赖或功能。
- 不修改 M1–M6 的指标、分母定义或验收门槛。
- 不删除或重写任何历史研究证据。
- 不虚构参与者、录音、访谈或测试结果。
- 不默认开启招募、测试会话、安装或外部分发。
- 不修改签名、更新、安全权限或许可证。
- 不进入 B1、RC、GA。
- 不为使 CI 变绿而弱化测试。

### 五、验收标准

完成时必须证明：

1. 新版研究构建的字节身份可独立核验。
2. 新旧版本的历史记录完整，来源关系明确。
3. 所有引用正式研究构建的当前权威文件一致。
4. 产品源码与业务行为未被本轮修改。
5. V1 参与者人数仍为 0，M1–M6 仍为 NOT_MEASURED。
6. U1 保持 PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS。
7. P5 保持 PASS_COMPLETE，产品保持 0.6.0 MVP_CANDIDATE。
8. 本地验证和远程 CI 的实际结果分别报告，不能用未运行代替 PASS。
9. 研究协议明确使用同一冻结构建，并禁止未经授权的版本替换。
10. 已获批准的范围完成后停止，ACTIVE_TASK 回到 NONE。

### 六、最终交付

输出：

- Re-freeze 执行报告
- 新旧版本身份对照及完整哈希
- 修改文件清单和差异摘要
- 本地检查结果
- 远程 CI Run ID 及逐 Job 结果
- 尚未完成的人工操作
- 下一步建议（仅建议，不执行）

最终状态应为：

`V1 = IN_PROGRESS / RECRUITMENT_READY`

`COHORT_BUILD = REFROZEN_TO_U1_CANDIDATE`

`ELIGIBLE_EXTERNAL_SESSIONS = 0`

`B1 = NOT_AUTHORIZED`

`ACTIVE_TASK = NONE`

不要自行宣称 V1 PASS，也不要直接启动参与者测试。

**本轮完成后停止，等待项目所有者下一次明确授权。**
