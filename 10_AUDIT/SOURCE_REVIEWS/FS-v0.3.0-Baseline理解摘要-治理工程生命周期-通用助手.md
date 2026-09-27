# FirmwareSight Baseline v0.3.0 理解摘要（治理 / 工程 / 生命周期门禁）

> 阅读范围：README.md、BASELINE.yaml、00_GOVERNANCE/（4 篇）、05_ENGINEERING/（8 篇）、06_DELIVERY/（6 篇）、AGENTS.md、CHANGELOG.md、templates/（4 篇），并核对 INDEX.md 交叉引用。
> 我的专属理解范围：治理规则、工程规范、生命周期与阶段门禁。

---

## 一、核心理念

1. **产品主张**："Know exactly what ships." —— 让嵌入式团队在发布前对"即将发布的东西"形成可复核的工程事实，而不是依赖文件名、记忆和零散脚本。本质是把离散事实收敛成 **Release Evidence Model**。
2. **Evidence before verdict（证据先于结论）**：每个事实必须归类为 `Observed / Derived / Declared / Unknown` 四类；不得把推测写成 Observed；**Unknown 不允许被隐藏**。
3. **Deterministic before intelligent**：核心是确定性引擎，AI 不得进入可信路径（ADR-0005）；价值公式是 Understand → Compare → Verify → Package，而不是 Upload → AI → Magic。
4. **Local-first**：MVP 无网络、无遥测、完全离线可用；核心功能在断网下必须成立。
5. **Product MVP ≠ 技术 Demo**：CLI-only 只能证明技术；MVP 必须是 **Minimum Credible Product UI**（完整、一致、无 Demo 感，但非最终商业视觉），陌生人无需读源码即可走完 Start → Analyze → Compare → Gate → Bundle 全流程。
6. **MVP 的完整性来自完整工作流，而不是功能数量**——一个真实 Release Workflow 完整，胜过二十个半成品。

## 二、关键决策（v0.3.0 冻结）

| 领域 | 决策 |
|---|---|
| 架构 | Core-first, Adapter-driven, Product-specific Shell：headless Rust Core + Ports/Adapters（artifact/object+gimli、storage/rusqlite、provenance/git CLI）+ Tauri 2 壳 |
| 技术栈 | Rust 1.98.1 / Edition 2024 / resolver 3；Tauri 2 + React 19.3 + TS strict + Vite 8 + Node 24 LTS + pnpm 12；tracing、clap、Serde、thiserror |
| 执行模型 | 同步优先；async 只允许在 Application/IO 边缘；domain 签名不得出现 `tokio::*`；阻塞解析/哈希/事务不进 async worker |
| 存储 | SQLite via rusqlite bundled；schema 变更必须走 migration；JSON 仅用于 config/interchange/export/golden fixture |
| 网络与 GPU | MVP 均为 none；启用需 ADR（approved_future: reqwest+rustls / wgpu native island） |
| 生命周期 | Problem Validation ✅ → **Technical Validation ← 当前 NEXT** → Product MVP → External MVP Validation → Productization → Private Beta → RC → GA 1.0 → Growth；阶段编号 P0–P4 / V1 / P5 / B1 / RC1 / GA1 |
| 门禁 | G0（已 PASS）~ G6；当前状态 `PRE_IMPLEMENTATION`，**Active Task = NONE：创建基线 ≠ 授权实现**，下一步若开工只允许创建 P0 Technical Vertical Slice |
| MVP 量化验证 | ≥8 名真实用户用真实 artifact；Activation >80%；TTFV <60s；Analyze→Compare ≥60%；Gate 含义理解 >80%；≥30% 发现真实新信息；≥5/8 愿再次使用；付费信号 ≥3 个明确意愿或 ≥1 个团队 Pilot |

## 三、与我领域相关的执行要点

### 治理规则（00_GOVERNANCE）
- **文档权威顺序**：已接受 ADR > 当前 PRD/Architecture/Design System > `.ai/DECISIONS.md` > Active Task > 研究资料。研究资料不能覆盖冻结决策；要变更必须写 ADR。
- **文档纪律**：编号 `FS-GOV/PRD/BRAND/DESIGN/TECH/ENG/COMP/RSCH-*` + `ADR-xxxx`；状态仅 DRAFT/REVIEW/BASELINE/SUPERSEDED/ARCHIVED；**禁止删除历史决策文档**，只能 SUPERSEDED 并链接替代文档；基线版本走 SemVer。
- **ADR 触发清单**：产品定位、桌面/前端框架、核心语言、存储格式与迁移、Evidence/Gate 数据模型、默认联网或遥测、SBOM 主版本、工具链承诺、许可证、AI 进可信路径、插件系统、自动更新/签名——以上变更必须 ADR；文案/微调/bug 修复/授权范围内重构不需要。
- **ADR 结构**：Context → Decision → Alternatives → Consequences → Revisit Trigger；Accepted 后不改结论，变更走 supersede。
- **写作规范**：先结论后原因；事实/假设/决策分开；用"必须/应该/可以/不得"表达强度；禁用"智能地/自动地/完美"等不可验证形容词；一切安全合规主张带边界。
- **Agent 行为红线（AGENTS.md）**：接手必读 README → `.ai/*` → `04_TECH/09_TECH_DECISION_MATRIX.md` → 相关 ADR；`ACTIVE_TASK.md = NONE` 时**不得自行创建业务功能**；技术基线不可静默改变（async-first、换框架、换 rusqlite、Core 引入 Tauri/Tokio/SQLite、加 HTTP client/wgpu/telemetry 等均需 ADR）；高风险操作（删除、force push、destructive migration、capability/signing/网络上传/许可证变更、一切不可逆操作）必须人工确认。

### 工程规范（05_ENGINEERING）
- **Repo 结构与预算**：workspace = `crates/`（core、artifact、storage、report）+ `apps/`（cli、desktop）+ `fixtures/` + `schemas/`。**Phase 0 library crate 上限 4 个**，第 5 个必须拿出真实重复证据；依赖方向强制 `core ← artifact/storage/report`，`core/adapters ← apps/*`，**core 依赖集必须最小**；前端 MVP 单 package，不预拆 design system/icons 包。
- **Core-first 硬约束**：core 必须 headless、UI 无关，不得依赖 Tauri/React/SQLite/Tokio，确定性+同步优先，项目自写代码禁 unsafe；**业务事实只从 Core 输出，UI 不得重新实现 parser/diff/gate 逻辑**。
- **编码标准**：rustfmt + `clippy -D warnings`；Result 式错误处理；library 不 `unwrap()` 用户输入；**parsing 代码不 panic**；enum/newtype 表达不可达状态；TS strict、no `any` 无边界、UI state 与 domain state 分离、组件内禁止 parser/业务规则；命名必须用术语表（Glossary）词汇。
- **日志分级**：INFO lifecycle / WARN degraded / ERROR failure / DEBUG parser detail；默认日志不得含源代码或用户完整绝对路径。
- **错误模型**：12 类错误 taxonomy（InputNotFound → InternalBug）；用户消息四要素 = What happened / Why we know / What user can do / Diagnostics ID；InternalBug 对普通用户只给 operation id + copy diagnostics，不裸露 stack trace。
- **测试策略**：金字塔 = Unit（分类/归一化/diff/gate 规则）→ **Fixture integration（真实编译产物是产品基础设施**，每 adapter 至少 valid-minimal / valid-realistic / stripped / malformed / 边缘 endian）→ Golden（versioned，**变更必须人工审查，严禁因测试失败机械批量更新 goldens**）→ Fuzz（ELF/MAP/HEX/manifest 优先）→ UI → E2E（import→compare→gate→export→verify 五步）。MVP fixture 矩阵 9 类（Cortex-M GCC、FreeRTOS-like、LVGL-like、stripped、DWARF、no-symbol、GNU ld MAP、HEX 有效+坏 checksum、malformed ELF header）。每 fixture 配 `fixture.toml` + expected JSON，解析器真值不允许用截图断言。
- **依赖纪律**：新增前七问（现有依赖够不够/是否维护/license 可否商业分发/是否引入 native runtime/bundle 影响/有无更小替代/是否进安全敏感路径）；默认禁止 GPL 进专有分发、unknown license、废弃 parser 进 trusted core、runtime 下载代码、telemetry SDK、AI SDK 进 core；**禁止因"Rust 生态常用"预装 Tokio/reqwest/wgpu/SQLx/Axum/Tonic**；不为一个 Button 引入整套 UI kit；Cargo.lock 与 pnpm lock 必须提交。
- **CI/CD**：`ci.yml`（PR：Rust fmt/lint/test + frontend + schemas + fixtures + 跨平台 smoke）、`security.yml`（cargo-deny、advisory、license policy，依赖变更触发）、`release.yml`（tag/手动保护）。**main 必需检查：Rust、frontend、fixtures/contracts、Windows/Linux/macOS core**。缓存只是优化，绝不能成为正确性依赖。发布通道 dev→beta→stable，stable 需零 P0/P1 + 三平台 packaging/install/upgrade smoke + changelog + checksum + （基础设施就绪后）签名。
- **自身发布工程**：release metadata 必须记录 Git commit / Rust toolchain / Node/pnpm / OS runner / 依赖 lock hash；MVP 不做 auto-update，启用前必须 ADR（签名/回滚/update server/离线环境/企业禁用开关）。
- **Tauri 安全**：web 前端不给通用 shell/fs 权限；commands 必须 use-case oriented（禁止 read arbitrary file）；capability 最小化；MVP 不启用 updater/network plugin。
- **DoD（12 项）**：验收标准齐 + Rust fmt/clippy/test + frontend typecheck/lint/test + 相关 fixture 回归 + **无 parser panic on user input** + 无无关依赖 + docs 与行为一致 + error state 可检视 + privacy/local-first 边界保持。

### 生命周期与阶段门禁（06_DELIVERY）
- **门禁模型 G0–G6**：
  - **G0 Problem Baseline**（PASS）：问题/人群/替代方案/边界明确。
  - **G1 Technical Validation**：最小链 `Real ELF Fixture → Parse → Normalize → BuildSnapshot → CLI JSON → Minimal Desktop Summary`；硬门槛 = Core/CLI/Desktop 事实一致、fixture 可重复、坏输入不崩溃、UI 不被阻塞、数据模型能继续支撑 Compare/Gate。UI 深度仅"极简技术 UI"，不进设计系统/账号/云/SBOM/updater。
  - **G2 Product MVP**：五个页面（Start / Analyze / Compare / Release Gate / Release Bundle）全部可用，Gate 每条结果必须带 Result + Why + Evidence + Next；四态 PASS/REVIEW/BLOCK/N/A 确定性可复现（same input+policy → semantically same result）；UI 达 Minimum Credible（1024×720 可用、Windows 125%/150% 不错位、loading/error/empty/unknown 状态完整、键盘可用、100k symbols 不卡死 WebView）；通过外部真实用户验证而非内部自评。
  - **G3 Productization**：兼容矩阵（fixture+regression+known limitations 才准标 Supported）、可靠性（fuzz/恢复/migration/诊断）、完整 design system、onboarding、project/history model、docs、install/sign/update readiness；退出标准 = 陌生用户无需开发团队陪同即可稳定使用。
  - **G4 Private Beta**：20–50 用户、3–5 团队，installer+签名策略+诊断+隐私说明+changelog+支持闭环，开始验证 Free/Pro/Team 付费边界。
  - **G5 RC**：定义是"敢让陌生用户在生产 Release 中依赖它"——零 P0、格式矩阵 green、migration 验证、security/capability review、签名包、安装/升级/恢复、五类文档。
  - **G6 GA 1.0**：明确范围内可发布/可收费/可维护/可升级/可支持；"完整产品"≠"功能最多"。
- **验证失败分类处理**：技术失败→修 Parser/Adapter；UX 失败→重排信息架构；Workflow 失败（用户发布前根本不会打开它）→重新定位使用节点，属严重问题；Value 失败（"IDE 都能看到"）→不能靠 UI 美化解决；Payment 失败→重估 Pro 边界/买家/定价。
- **防跑偏三问（最重要纪律）**：任何新功能先问——①属于哪个阶段？②当前阶段是否必须？③没有它是否无法验证本阶段假设？任一不满足 → **默认不做**。
- **MVP 明确 Deferred**：account/cloud/team/AI/SBOM/CVE/updater/dark mode/plugin/HIL/flashing/OTA；Post-GA 才考虑 Component Evidence/CycloneDX-SPDX/CVE/CI 集成/Team Policy，且任何 Growth 功能不得反向破坏 local-first、deterministic core、evidence provenance、explicit unknown。
- **模板已就绪**：ADR（五段式）、Feature Spec（问题→用户→目标→非目标→流程→域变更→UI/错误态→安全→性能→验收→测试→发布→开放问题）、Release Checklist（15 项）、Bug Report（环境/复现/最小 fixture/诊断 ID/数据安全影响）。

## 四、发现的疑点 / 风险

1. **macOS CI 要求存在两处口径差**：`FS-ENG-003 测试策略`写 PR 仅 Windows+Linux required、macOS 可放 release/nightly（考虑 runner 成本）；`FS-ENG-007 CI/CD Baseline`写 main 必需检查含 macOS core。可解读为"PR 不跑、main push 跑"，但基线未明确 macOS 检查挂在哪个事件上，建议澄清统一。
2. **06_DELIVERY 编号缺 04**（00/01/02/03/05/06）。已核对 INDEX.md 同样无 04，判断为从未分配而非违规删除，但按文档控制规范建议补一条占位/说明，避免后续 Agent 以为漏读。
3. **FS-DEL-005（生命周期主文档）内部仍引用 v0.2.0 基线**（"与 FirmwareSight_Project_Baseline_v0.2.0 配套使用"、"截至 v0.2.0"），而它已是 v0.3.0 权威文档；实质内容与 v0.3 冻结结论一致，属版本引用未同步的文档卫生问题。同理 `05_ENGINEERING/00` 标题仍写 "Repository Structure v0.2"。
4. **阶段命名存在两套并行体系**：BASELINE/ROADMAP/STAGE_GATES 用 P0-P4/V1/P5/B1/RC1/GA1 + G0-G6；FS-DEL-005 第 2 节另用 Problem→Technical→Product MVP→Commercial 四验证表述。可互相映射，但建议明确"以 BASELINE.yaml + FS-DEL-006 Stage Gates 为准"，防止后续 Agent 混用。
5. **指标基数小**：验证指标以 8 人为基数（如 Return intent 5/8），而招募范围是 8–15 人；若实际 12–15 人，阈值是按比例还是固定 5/8 未定义。属小风险，建议在 V1 启动前定死换算规则。
6. **git CLI 依赖提醒**：provenance 走 Git CLI（ADR-0017），错误模型已有 `GitUnavailable/GitStateConflict` 覆盖，设计完整；但意味着无 git 环境下 Identity 部分字段必然 Unknown——符合"explicit unknown"理念，实现 P0 时记得按降级路径处理而非报错中断。

## 五、结论

v0.3.0 是一份"纯规则冻结、零实现代码"的基线：治理（ADR 优先级链）、工程（Core-first 红线 + fixture 即产品基础设施 + golden 人工审查）、生命周期（G0-G6 门禁 + 防跑偏三问）三者自洽，且 README/BASELINE.yaml/AGENTS/CHANGELOG 与各规范交叉引用基本一致（仅上述 6 处文档卫生/口径类小瑕疵）。当前全局处于 `PRE_IMPLEMENTATION`，**下一步唯一合法动作是等待授权创建 P0 — Technical Vertical Slice（Real ELF fixture → Rust normalized BuildSnapshot → fwsight CLI JSON → Tauri Minimal Summary）**，任何提前实现业务功能的行为都违反 AGENTS.md。
