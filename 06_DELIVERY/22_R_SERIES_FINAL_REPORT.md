---
title: "R-Series Final Report"
doc_id: "FS-DEL-022"
product: "FirmwareSight"
version: "0.6.0"
status: "FINAL_REPORT"
owner: "Engineering"
last_updated: "2026-10-17"
---

# R 系列重设计（Instrument Ledger）最终报告

**授权：** Owner 于 2026-10-16 批准 R0 原型与 R1；2026-10-17 在对话中给出常设目标“按照计划一直推进直到完成完整的产品，中间无需我们介入确定，一切由你来定夺”。治理：`ADR-0034`（含修订 A1）。

## 1. 结论

R0–R7 全部完成，并已通过普通快进推送到 `main`。

| 阶段 | 内容 | 计划文档 | 证据 |
|---|---|---|---|
| R0 | 规格、ADR-0034、可交互原型 | [03_DESIGN/09](../03_DESIGN/09_R_REDESIGN_SPEC.md) | `03_DESIGN/prototype/` |
| R1 | tokens 0.4.0、图标、外壳与共享组件 | [15](15_R1_FOUNDATION.md) | [evidence/R1](../evidence/R1/README.md) |
| R2 | 分析与总览 | [16](16_R2_ANALYZE_OVERVIEW.md) | [evidence/R2](../evidence/R2/README.md) |
| R3 | 对比 | [17](17_R3_COMPARE.md) | [evidence/R3](../evidence/R3/README.md) |
| R4 | 发布工作台 | [18](18_R4_RELEASE.md) | [evidence/R4](../evidence/R4/README.md) |
| R5 | 历史（固定基线 D3）、设置、帮助、命令面板 | [19](19_R5_HISTORY.md) | [evidence/R5](../evidence/R5/README.md) |
| R6 | 报告体系：CLI 文本、HTML、SVG/PNG、新增 Markdown 导出 | [20](20_R6_REPORTS.md) | [evidence/R6](../evidence/R6/README.md) |
| R7 | 质量：tokens 0.4.1、报告表格语义、全量原生回归、A4 打印检查、真实输入收尾 | [21](21_R7_QUALITY.md) | [evidence/R7](../evidence/R7/README.md)、[evidence/R7C](../evidence/R7C/README.md) |

**集成记录（Observed）：**
- R0–R3：`main` 于 `465fe5f`（Actions #125，10/10）。
- R4–R7：`465fe5f..e1faeff`，run `39086793226`，10/10 jobs。
- 收尾（跳过链接、焦点进标题、真实输入检查）：`e1faeff..af3c0d0`，run `39091381963`，10/10 jobs。
- 每个阶段：计划 → 实现 → 测试 → 原生 smoke → 独立评审 → 修复 → `check.py` 18/18 → 提交。各阶段评审处置记录在各自计划文档末节。

## 2. 验证概况

- **本地门禁：** 每次提交前 `check.py` 18/18（fmt、clippy、Rust 测试、前端类型检查/lint/测试/构建、tokens/图标/基线/IPC/fixture/版本/golden 漂移、cargo-deny）。
- **UI 测试：** 532 项全部通过。
- **原生 QA（R7C，收尾版本）：** 71 项，66 PASS、0 FAIL、2 NOT_RUN、3 SUPERSEDED（旧的“需人工拖拽”行，已由真实拖拽取代）。
  - 真实鼠标拖拽：窗口停在 1024×720（关闭 R1 A10）。
  - 真实按键：纯键盘流程。
  - 真实资源管理器拖放：文件被暂存并分析，源文件不变。
  - 内容缩放 1.25 / 1.5：七个页面无溢出。
  - 无障碍树审计：可操作元素全部有名称，landmark 齐全，表格都有标题。
- **导出：** 两份 HTML 报告 golden 按 A4 打印媒体检查通过：表头跨页重复、行不断开、不超宽、无脚本、无外链，`th scope` 与 caption 齐全。

## 3. 不由本会话完成的事项（及原因）

| 事项 | 状态 | 为什么不能由本会话完成 | 已做的部分 |
|---|---|---|---|
| 用 NVDA / 讲述人实际收听 | NOT_RUN | 本机未安装 NVDA，安装需要下载并运行外部程序，必须 Owner 同意；讲述人只有语音输出，没有可读取的记录，无法验证“读出了什么”。 | 读屏软件读取的无障碍树已审计通过（R7C SCALE-S03）。 |
| 真实 125% / 150% 系统缩放 | NOT_RUN | 修改系统显示设置属于本会话禁止的系统设置修改。 | 内容缩放 1.25 / 1.5 在最小窗口下通过，比真实场景更窄（R7C S01、S02）。 |
| 删除已合并的本地分支 `feature/r4-release`…`feature/r7-quality`、`docs/r-closeout`、`docs/r-final-report` | 保留 | AGENTS.md §9：删除分支必须人工确认，总体授权不能替代。 | 分支都已包含在 `main` 中，worktree 已全部清理。 |
| 删除 smoke 保留的 QA 数据目录 | 保留 | “可以重新生成”不等于“可以未经确认删除”。 | 清单见各 `evidence/R*/README.md`。 |

## 4. 留给后续的观察

- **`.symtab` 排在对比页节排行首位：** 排行来自 Core 的 diff，包含不加载的节。是否让“加载进芯片的节”优先，是 Core 层面的决定；按 AGENTS.md §3，UI 不重新排序。
- **截图登记表** `03_DESIGN/06_UI_REFERENCE_SCREENS.md` 要等有人看过最终版本后再替换；候选截图在 R7、R7C 的证据 ZIP 中。

## 5. 未触碰的本地状态

- Owner 原工作目录 `D:\study\Software\FirmwareSight` 仍在 `e34f9e6`，九个已暂存文件原样保留。
- Owner 的 `%APPDATA%\com.firmwaresight.desktop` 只做过哈希：每轮前后都是 23 个文件、哈希一致。
- `.ai/ACTIVE_TASK.md` 为 `NONE`：下一项业务功能要等 Owner 命名后再开始（AGENTS.md §1）。
