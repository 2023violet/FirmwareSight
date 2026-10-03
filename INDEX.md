---
title: "Document Index"
doc_id: "FS-ROOT-INDEX"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-10-03"
---


# Document Index

Baseline: `0.6.0` — the P0 Technical Foundation Baseline.

Execution state: `P0: PASS — frozen (remote CI Runs #3, #4, #5 and #6 all 7 of 7 green) · Pre-G1 P1-A0 COMPLETE, including its evidence-identity and persistence correctness closure (Runs #9, #10, #11 all 7 of 7 green) · G1 PASS on the P0 basis under ADR-0026 (2026-09-29) · V0 NON_BLOCKING_USER_FEEDBACK_TRACK 0/8, an honest zero that gates no stage · P1 PASS/COMPLETE — the Analyze verb, evidence in P1_VALIDATION/, remote CI Run #13 `36556735551` on `e63afaf` 7 of 7 green · P2 PASS/COMPLETE — Compare, evidence in P2_VALIDATION/; its mid-round push `c7fc2a3` failed remote Run #17 `36596452341` because `.gitignore` hid half of the P2 fixture pair, `cfee1e5` fixed it, and the head `4a77ea1` is green on Run #18 `36648718199` 7 of 7 · P3 PASS/COMPLETE — Release Gate, authorized 2026-09-29 by execution prompt v1.1 plus ADR-0027 and closed 2026-09-30, evidence in P3_VALIDATION/; 556 Rust tests in 28 executable suites, 135 UI tests, 15/15 gate steps, an 18/18 CLI Gate smoke and all forty §61 desktop steps on the shipping binary — closed as LOCAL PASS while its commits were unpushed, then pushed: Run `36774472141` on `219178af` completed failure with 6 of 7 jobs green, only `Dependency policy` red on a crate yanked on crates.io after the local deny step had passed on a stale index, fixed by a `yoke-derive` patch bump, itself verified by Run `36779321479` at 7 of 7 green; the next push `893a635` went red on `Desktop UI (windows-latest)` alone — a call-count race in `compare.test.tsx` (defect J) that the ubuntu job won on the same commit, fixed with its assertions unchanged and verified by Run `36783457030` at 7 of 7 green, as was the documentation-only successor `f66a93d` on Run `36784382005` — the last run this index names. Run #19 `36665007523` covers P2's closure and not this stage · P4 PASS/COMPLETE — Release Bundle, authorized 2026-09-30 by execution prompt v1.0 (delivered as a file, hashed into `10_AUDIT/SOURCE_PROMPTS/`) and closed 2026-10-01, evidence in P4_VALIDATION/; 769 Rust tests in 41 executable suites, 155 UI tests in 6 files, 15/15 gate steps, a 23-ok CLI smoke and all fifty §59 desktop steps on the shipping binary; the final implementation head `e799f2f` is green on Run `36872456446` at 7 of 7, and the closure heads `157f749` (Run `36879477561`, green on a rerun after the runner's rustup install faulted before any repo step ran) and `94f7bd9` (Run `36881101091`) are green at 7 of 7 · G2 PASS (2026-10-01) — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE; evidence G2_VALIDATION/, product tree `e35cfe7` (Run `36906482900`) and evidence head `f75cbc5` (Run `36948719972`) both 7 of 7 on the first attempt · Post-G2 real-desktop MVP acceptance reached `PASS_WITH_FINDINGS` on
2026-10-02 with three findings and no S0/S1, and its narrow remediation closed the same day: E2E-F001
(S3, a surviving analysis that did not say which selection it described), E2E-F002 (S2, a folder the
engine refuses to replace was offered for replacement) and E2E-F003 (S3, a GNU ld MAP refused as another
linker's output) fixed in `e816dcb` and `971015f`; 770 Rust / 159 UI / `check.py` 15/15, the fix head
green on Run #43 `37100371601` at 7 of 7 on the first attempt, evidence in `POST_G2_E2E_REMEDIATION/`
and the 283-case root outside the repository — G2 unchanged, and that round authorized nothing beyond
itself · P5 IN_PROGRESS (opened 2026-10-03) — Productization, authorized by execution prompt v1.0
delivered as a file, SHA-256 `722125f5aa68e324ba1dea4826f66d8392acab9ad9a015c4919ed5ade471e0ae`, archived
in `10_AUDIT/SOURCE_PROMPTS/`; its §4 audit is written at `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` and
the owner's checkpoint settled version identity (artifacts unify on `0.6.0`), migration `0005` after a
written decision doc, the package matrix (Windows with real install evidence here, macOS/Ubuntu
`CI_BUILD_ONLY`) and the store filename (`firmwaresight-p0.sqlite` stays`). Landed since: `4cc8d93`
governance + audit (Run `37125456689` 7 of 7), `812b472` migration `0005` (Run `37127791999` 7 of 7),
`9e3b1de` the `0.6.0` unification — whose Run `37128593254` **failed 6 of 7** on a pre-existing
`compare.test.tsx` race the commit did not cause — repaired test-only by `20b03e3`, green on Run
`37129900728`, and `0c031cd` the packaging commit — `bundle.active`, the three §41 package jobs,
`P5_VALIDATION/P5_PACKAGING_REPORT.md`, `P5_VALIDATION/P5_CI_AUTHORITY.md` and the gate's
`drift/version identity` step — whose first 10-job run `37133706214` **failed 7 of 10**: seven gate jobs
green, three package jobs skipping their own build because `cargo install tauri-cli` leaves `cargo-tauri`
and the group probed `tauri`, then reporting `4/4 steps passed`. Its successor makes a `SKIP` uncountable as
a pass and fails any skip under `CI`, and — with the pinned CLI installed on this host — built this
repository's first real package, a Windows NSIS installer, which surfaced a second defect: the package step
must run from `apps/desktop`, because the CLI resolves its frontend directory from the process cwd
(**775 Rust / 160 UI / `check.py` 16 steps / 10 authoritative CI
jobs**). **Still no P5 verdict exists, and no CI artifact exists yet** — the product is
MVP CANDIDATE at `0.6.0`, and no tag, Release, installer publication, signing, updater or licence
choice is authorized · open-source
licence PENDING OWNER CONFIRMATION · pricing and commercial research DEFERRED_POST_MVP · active_task:
P5_PRODUCTIZATION`

## Primary reading path

1. `README.md`
2. `PRODUCT_BASELINE.md`
3. `BASELINE.yaml`
4. `.ai/CURRENT_STATE.md`
5. `.ai/ACTIVE_TASK.md` — `P5_PRODUCTIZATION`; G2 closed `PASS` on 2026-10-01 and P5 opened on
   2026-10-03 under an execution prompt of its own, which is the only way a stage opens here. With a live
   task the pointer still names exactly one stage: nothing in it authorizes V1, and no agent lifts a
   later track off the roadmap
5a. `G2_VALIDATION/G2_EXIT_CHECKLIST.md` and `G2_ENGINEERING_CLOSURE_REPORT.md` — the whole-MVP verdict
5b. `POST_G2_E2E_REMEDIATION/` — the narrow remediation of the post-G2 real-desktop findings: what was
    fixed and why (`REMEDIATION_REPORT.md`), the focused real-desktop re-validation on the shipping
    binary (`FOCUSED_REVALIDATION_REPORT.md`), and the round's exit boxes settled one by one
    (`EXIT_CHECKLIST.md`). It changes no stage status: G2 stays PASS and the product stays MVP CANDIDATE
6. `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`
7. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`
8. `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md`
9. `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` — all five runs, the two failures included
10. `V0_VALIDATION/batch_a/BATCH_A_STATUS.md`
11. `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`
12. `P1_A0_VALIDATION/` — the completed intake slice and its correctness closure
13. `P1_VALIDATION/` — the completed P1 Analyze round: execution report, exit checklist (the US-001
    verdict), design checklist, and the shipped-binary desktop smoke
14. `P2_VALIDATION/` — the completed P2 Compare round: execution report (every gate number and the CLI
    smoke), exit checklist (the US-002 verdict), design checklist, the shipped-binary desktop smoke, and
    defect E — the fixture half `.gitignore` hid, which reddened remote Run #17 and is closed by Run #18
    `36648718199` (success, 7 of 7) on the pushed head `4a77ea1`
15. `P3_VALIDATION/` — the completed P3 Release Gate round: execution report (every §66 gate number and the
    18-step CLI smoke), exit checklist (§69's forty boxes and the US-003 verdict), design checklist, and the
    shipped-binary desktop smoke that walked all forty §61 steps and measured the CLI and the desktop
    agreeing on one run id. Three product defects live there — a version pattern quoted into an evidence
    locator that made a run unpersistable, one build carrying two run ids across the two surfaces, and a
    disabled button keeping its accent border — each with the test that now pins it

`DIRECTORY_TREE.txt` and `SHA256SUMS` are the regenerated **v0.6.0** baseline record: the tree lists
this repository's tracked layout and `SHA256SUMS` covers the baseline-controlled files in it. They were
produced by the promotion round, in that order, and verified with `sha256sum -c` plus an independent
checker; `P0_FINAL_PROMOTION_REPORT.md` records the commands. The v0.5.1 manifest these replaced is
history, and the nine-entry drift that `0.6.0` closed is described in `.ai/DECISIONS.md`.


## All Markdown documents

| Path | Title |
|---|---|
| `.ai/ACTIVE_TASK.md` | ACTIVE TASK |
| `.ai/CURRENT_STATE.md` | Current State |
| `.ai/DECISIONS.md` | Decisions — v0.6.0 |
| `.ai/HANDOFF.md` | Handoff — FirmwareSight v0.6.0 / P0 closed PASS. No active task. Do not invent one. |
| `.ai/README.md` | AI Entry Point |
| `00_GOVERNANCE/00_DOCUMENT_CONTROL.md` | 文档控制规范 |
| `00_GOVERNANCE/01_PROJECT_CHARTER.md` | 项目章程 |
| `00_GOVERNANCE/02_GLOSSARY.md` | 术语表 |
| `00_GOVERNANCE/03_DECISION_POLICY.md` | 决策与 ADR 规范 |
| `01_PRODUCT/00_PRODUCT_VISION.md` | 产品愿景 |
| `01_PRODUCT/01_PRD_MVP.md` | MVP PRD |
| `01_PRODUCT/02_PERSONAS_JTBD.md` | Personas & JTBD |
| `01_PRODUCT/03_SCOPE_NON_GOALS.md` | Scope / Non-goals |
| `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` | User Stories |
| `01_PRODUCT/05_BUSINESS_MODEL.md` | 商业模式假设 |
| `01_PRODUCT/06_COMPETITIVE_POSITIONING.md` | Competitive Positioning |
| `01_PRODUCT/07_MVP_COHORT_AND_CLI_POLICY.md` | MVP Cohort & CLI Policy |
| `01_PRODUCT/08_PRODUCT_MODULE_MAP.md` | Product Module Map |
| `01_PRODUCT/09_CAPABILITY_PORTFOLIO.md` | Capability Portfolio |
| `02_BRAND/00_NAMING_DECISION.md` | 命名决策 |
| `02_BRAND/01_BRAND_FOUNDATION.md` | Brand Foundation |
| `02_BRAND/02_VOICE_MESSAGING.md` | Voice & Messaging |
| `03_DESIGN/00_DESIGN_PHILOSOPHY.md` | Design Philosophy |
| `03_DESIGN/01_INFORMATION_ARCHITECTURE.md` | Information Architecture |
| `03_DESIGN/02_UX_FLOWS.md` | UX Flows |
| `03_DESIGN/03_VISUAL_SYSTEM.md` | Visual System |
| `03_DESIGN/04_COMPONENT_RULES.md` | Component Rules |
| `03_DESIGN/05_ACCESSIBILITY.md` | Accessibility |
| `03_DESIGN/06_UI_REFERENCE_SCREENS.md` | UI Reference Screens |
| `03_DESIGN/07_DESIGN_TOKEN_VALIDATION.md` | Design Token Validation |
| `03_DESIGN/08_POST_MVP_PRESENTATION_RULES.md` | Post-MVP Presentation Rules |
| `04_TECH/00_TECH_STACK.md` | Technology Stack |
| `04_TECH/01_SYSTEM_ARCHITECTURE.md` | System Architecture |
| `04_TECH/02_DOMAIN_MODEL.md` | Domain Model |
| `04_TECH/03_FORMAT_SUPPORT.md` | Format Support Strategy |
| `04_TECH/04_STORAGE_WORKSPACE.md` | Local Storage |
| `04_TECH/05_SECURITY_PRIVACY.md` | Security & Privacy |
| `04_TECH/06_PERFORMANCE_BUDGETS.md` | Performance Budgets |
| `04_TECH/07_CLI_SPEC.md` | CLI Specification |
| `04_TECH/08_CONFIG_SPEC.md` | `firmwaresight.toml` |
| `04_TECH/09_TECH_DECISION_MATRIX.md` | Technical Decision Matrix |
| `04_TECH/10_TOOLCHAIN_BASELINE.md` | Toolchain Baseline |
| `04_TECH/11_DEPENDENCY_BASELINE.md` | Dependency Baseline |
| `04_TECH/12_ASYNC_EXECUTION_MODEL.md` | Async & Execution Model |
| `04_TECH/13_FRONTEND_ARCHITECTURE.md` | Frontend Architecture |
| `04_TECH/14_IPC_DATA_CONTRACTS.md` | IPC / Data Contracts |
| `04_TECH/15_STORAGE_DATABASE_BASELINE.md` | SQLite Storage Baseline |
| `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md` | Artifact Analysis Pipeline |
| `04_TECH/17_RELEASE_PACKAGING_UPDATE.md` | Build / Package / Sign / Update |
| `04_TECH/18_CI_SUPPLY_CHAIN.md` | CI / Supply Chain |
| `04_TECH/19_NATIVE_GPU_ISLAND_POLICY.md` | Native / GPU Island Policy |
| `04_TECH/20_PLATFORM_SUPPORT.md` | Platform Support Matrix |
| `04_TECH/21_OBSERVABILITY_DIAGNOSTICS.md` | Observability / Diagnostics |
| `04_TECH/22_GIT_PROVENANCE_ADAPTER.md` | Git Provenance Adapter |
| `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` | Firmware Memory Accounting Model |
| `04_TECH/24_BUILD_IDENTITY_EVIDENCE.md` | Build Identity Evidence Model |
| `04_TECH/25_TYPED_IPC_BINDINGS.md` | Typed IPC Bindings |
| `04_TECH/26_PORTABLE_SCHEMA_POLICY.md` | Portable Schema Policy |
| `04_TECH/27_GATE_STATE_SEMANTICS.md` | Release Gate State Semantics |
| `05_ENGINEERING/00_REPO_STRUCTURE.md` | Repository Structure |
| `05_ENGINEERING/01_CODING_STANDARDS.md` | Coding Standards |
| `05_ENGINEERING/02_TEST_STRATEGY.md` | Test Strategy |
| `05_ENGINEERING/03_ERROR_MODEL.md` | Error Model |
| `05_ENGINEERING/04_RELEASE_ENGINEERING.md` | FirmwareSight App Release |
| `05_ENGINEERING/05_DEPENDENCY_POLICY.md` | Dependency Policy |
| `05_ENGINEERING/06_CI_CD_BASELINE.md` | CI/CD Baseline |
| `05_ENGINEERING/07_TEST_FIXTURE_STRATEGY.md` | Artifact Fixture Strategy |
| `05_ENGINEERING/08_CODE_AREA_PLAN.md` | FirmwareSight Code Area Plan |
| `06_DELIVERY/00_ROADMAP.md` | Product Roadmap |
| `06_DELIVERY/01_MVP_EXIT_CRITERIA.md` | MVP Exit Criteria |
| `06_DELIVERY/02_BACKLOG_SEED.md` | Seed Backlog |
| `06_DELIVERY/03_QA_CHECKLIST.md` | QA Checklist |
| `06_DELIVERY/04_EXTERNAL_VALIDATION_PLAN.md` | External Validation Plan |
| `06_DELIVERY/05_MVP_TO_PRODUCT_DEVELOPMENT_LIFECYCLE.md` | FirmwareSight MVP → Complete Product Development Lifecycle |
| `06_DELIVERY/06_STAGE_GATES.md` | Stage Gates |
| `06_DELIVERY/07_POST_MVP_CANDIDATE_ROADMAP.md` | Post-MVP Candidate Roadmap |
| `06_DELIVERY/08_MILESTONE_DELIVERABLE_MATRIX.md` | Milestone and Deliverable Matrix |
| `06_DELIVERY/09_USER_INTERVIEW_QUESTION_BANK.md` | Validation Interview Question Bank |
| `07_COMPLIANCE/00_COMPLIANCE_BOUNDARY.md` | Compliance Boundary |
| `07_COMPLIANCE/01_SBOM_STRATEGY.md` | SBOM Strategy |
| `07_COMPLIANCE/02_CRA_CONTEXT.md` | CRA Context |
| `07_COMPLIANCE/03_LICENSE_POLICY.md` | Licensing Policy |
| `08_RESEARCH/00_MARKET_EVIDENCE.md` | Market Evidence Summary |
| `08_RESEARCH/01_NAMING_SCREEN.md` | Naming Collision Screen |
| `08_RESEARCH/02_TECH_REFERENCES.md` | Technical References |
| `08_RESEARCH/03_RISKS_ASSUMPTIONS.md` | Risks & Assumptions |
| `08_RESEARCH/04_RUST_BASELINE_APPLICATION_TO_FIRMWARESIGHT.md` | Applying the Rust Product Census to FirmwareSight |
| `08_RESEARCH/05_EXTERNAL_TECH_VERIFICATION_2026-09-26.md` | External Technology Verification — 2026-09-26 |
| `08_RESEARCH/06_MARKET_SOURCE_REGISTER.md` | Market Source Register |
| `08_RESEARCH/07_AWESOME_DESIGN_MD_REFERENCE.md` | awesome-design-md Reference |
| `08_RESEARCH/08_COMPETITOR_UPDATE_2026-09-27.md` | Competitor Update — AssureLoop |
| `08_RESEARCH/09_EXPERT_ROUND_MARKET_VERIFICATION_2026-09-27.md` | Expert Round Market Verification — 2026-09-27 |
| `08_RESEARCH/10_POST_MVP_SIGNAL_REGISTER.md` | Post-MVP Signal Register |
| `08_RESEARCH/11_RISK_DEPENDENCY_REGISTER.md` | Risk and Dependency Register |
| `08_RESEARCH/SOURCE_REPORTS/README.md` | Source Reports |
| `09_ADR/ADR-0001-product-name.md` | Context |
| `09_ADR/ADR-0002-desktop-stack.md` | Status |
| `09_ADR/ADR-0003-rust-core.md` | Status |
| `09_ADR/ADR-0004-local-first.md` | Context |
| `09_ADR/ADR-0005-no-ai-trusted-core.md` | Context |
| `09_ADR/ADR-0006-mvp-format-scope.md` | Context |
| `09_ADR/ADR-0007-core-first-adapter-driven.md` | Status |
| `09_ADR/ADR-0008-async-boundary.md` | Status |
| `09_ADR/ADR-0009-storage-rusqlite.md` | Status |
| `09_ADR/ADR-0010-artifact-parser.md` | Status |
| `09_ADR/ADR-0011-frontend-baseline.md` | Status |
| `09_ADR/ADR-0012-tracing.md` | Status |
| `09_ADR/ADR-0013-network-deferred.md` | Status |
| `09_ADR/ADR-0014-gpu-deferred.md` | Status |
| `09_ADR/ADR-0015-release-update-lifecycle.md` | Status |
| `09_ADR/ADR-0016-toolchain-pinning.md` | Status |
| `09_ADR/ADR-0017-git-cli-provenance.md` | Status |
| `09_ADR/ADR-0018-ui-baseline-tokens-governance.md` | ADR-0018 — UI 基准与 Design Tokens 治理 |
| `09_ADR/ADR-0019-typed-ipc-bindings.md` | ADR-0019 — Typed IPC Bindings |
| `09_ADR/ADR-0020-validation-sequence.md` | ADR-0020 — V0 Prototype + P0 Vertical Slice Parallel Validation |
| `09_ADR/ADR-0021-memory-accounting.md` | ADR-0021 — Firmware Memory Accounting |
| `09_ADR/ADR-0022-portable-schema-strictness.md` | ADR-0022 — Portable Schema Strictness |
| `09_ADR/ADR-0023-gate-five-state-semantics.md` | ADR-0023 — Gate Five-State Semantics |
| `09_ADR/ADR-0024-post-mvp-candidate-governance.md` | ADR-0024 — Post-MVP Candidate Governance and Namespace |
| `09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md` | ADR-0025 — Conditional Pre-G1 Analyze Implementation (P1-A0; sequencing partly superseded by ADR-0026) |
| `09_ADR/ADR-0026-open-source-mvp-first-delivery.md` | ADR-0026 — Open-Source MVP-First Delivery (G1 basis, V0 non-blocking) |
| `09_ADR/ADR-0027-project-policy-and-provenance-adapter.md` | ADR-0027 — Project Policy and Provenance Adapter Boundary (P3; authorizes `firmwaresight-project`) |
| `10_AUDIT/00_V0.3_AUDIT_RESOLUTION.md` | v0.3.0 Audit Resolution |
| `10_AUDIT/01_UI_BASELINE_REVIEW.md` | UI Baseline Review |
| `10_AUDIT/02_V0.5_EXPERT_REVIEW_RESOLUTION.md` | v0.5.0 Expert Review Resolution |
| `10_AUDIT/03_ADOPTION_DECISION_REGISTER.md` | v0.5.0 Adoption Decision Register |
| `10_AUDIT/SOURCE_PROMPTS/README.md` | Execution Prompt Register |
| `10_AUDIT/SOURCE_REVIEWS/ADR-0018-ui-baseline-tokens-governance.md` | ADR-0018 — UI 基准与 design tokens 治理 |
| `10_AUDIT/SOURCE_REVIEWS/DESIGN_CHECKLIST_TEMPLATE.md` | FirmwareSight 设计评审 Checklist（Do's & Don'ts） |
| `10_AUDIT/SOURCE_REVIEWS/FS-AGENTS-UI-Rules增补节-通用助手.md` | AGENTS.md 增补节：UI Rules（入仓准备件） |
| `10_AUDIT/SOURCE_REVIEWS/FS-UI基调定案提案-设计匠人.md` | FirmwareSight UI 基调定案提案 |
| `10_AUDIT/SOURCE_REVIEWS/FS-v0.3.0-Baseline理解摘要-治理工程生命周期-通用助手.md` | FirmwareSight Baseline v0.3.0 理解摘要（治理 / 工程 / 生命周期门禁） |
| `10_AUDIT/SOURCE_REVIEWS/FS-立项过程与产品演变理解摘要-通用助手.md` | FirmwareSight 立项过程与产品演变理解摘要 |
| `10_AUDIT/SOURCE_REVIEWS/FirmwareSight v0.3.md` | FirmwareSight v0.3.0 基线 · 技术理解摘要 |
| `10_AUDIT/SOURCE_REVIEWS/FirmwareSight-设计风格调研-awesome-design-md.md` | 情报报告：VoltAgent/awesome-design-md 仓库调研 & FirmwareSight 设计风格选型 |
| `10_AUDIT/SOURCE_REVIEWS/README.md` | Audit Source Reviews |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-PostMVP-开放问题裁决记录-通用助手.md` | FirmwareSight Post-MVP 探索开放问题裁决记录 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-可视化与文档导出-设计视角备忘-设计匠人.md` | FirmwareSight 可视化与文档导出 · 设计视角备忘 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-正式产品功能探索-分析结果可视化与文档导出-专业写手.md` | FirmwareSight 正式产品功能探索：分析结果可视化与文档导出（Post-MVP） |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-正式产品功能探索-真需求功能全景扫描-专业写手.md` | FirmwareSight 正式产品功能探索：真需求功能全景扫描（Post-MVP） |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-正式产品功能探索-端口扩展与深度体验-专业写手.md` | FirmwareSight 正式产品功能探索：端口扩展与深度体验（Post-MVP） |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-外部信号报告-信息哨兵.md` | 真需求全景扫描·外部信号报告 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-技术可行性简报-鲁班七号(1).md` | 真需求全景扫描·技术可行性简报 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-技术可行性简报-鲁班七号.md` | 真需求全景扫描·技术可行性简报 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-设计视角备忘-设计匠人(1).md` | FirmwareSight 真需求全景扫描 · 设计视角备忘 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-真需求全景扫描-设计视角备忘-设计匠人.md` | FirmwareSight 真需求全景扫描 · 设计视角备忘 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-端口扩展与深度体验-技术可行性简报-鲁班七号.md` | 端口扩展与深度体验·技术可行性简报 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-端口扩展与深度体验-设计视角备忘-设计匠人.md` | FirmwareSight 端口扩展与深度体验 · 设计视角备忘 |
| `10_AUDIT/SOURCE_REVIEWS_V05/FS-端口扩展与用户粘性-外部调研报告-信息哨兵.md` | 端口扩展与用户粘性·外部调研报告 |
| `10_AUDIT/SOURCE_REVIEWS_V05/README.md` | v0.5 Expert Source Reviews |
| `10_AUDIT/SOURCE_REVIEWS_V05/可视化与文档导出·外部调研报告.md` | 《可视化与文档导出·外部调研报告》 |
| `10_AUDIT/SOURCE_REVIEWS_V05/可视化与文档导出·技术可行性简报.md` | 《可视化与文档导出 · 技术可行性简报》 |
| `10_AUDIT/SOURCE_REVIEWS_V05/背景回顾-已覆盖面与本轮扫描边界清单-通用助手.md` | 背景回顾：已覆盖面与本轮扫描边界清单 |
| `10_AUDIT/SOURCE_REVIEWS_V05/背景回顾：可视化与文档导出的基线事实清单.md` | 背景回顾：可视化与文档导出的基线事实清单 |
| `10_AUDIT/SOURCE_REVIEWS_V05/背景回顾：端口扩展与深度体验的基线事实清单.md` | 背景回顾：端口扩展与深度体验的基线事实清单 |
| `AGENTS.md` | AGENTS.md |
| `CHANGELOG.md` | Changelog |
| `DESIGN.md` | FirmwareSight DESIGN.md |
| `PRODUCT_BASELINE.md` | FirmwareSight Product Baseline v0.5.0 |
| `README.md` | FirmwareSight v0.6.0 |
| `P0_TECHNICAL_VALIDATION/README.md` | P0 Technical Validation |
| `P0_TECHNICAL_VALIDATION/P0_ARCHITECTURE_CHECK.md` | P0 Architecture Check |
| `P0_TECHNICAL_VALIDATION/P0_CI_REMEDIATION_REPORT.md` | P0 CI Remediation Report |
| `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` | P0 CI Report |
| `P0_TECHNICAL_VALIDATION/P0_CI_RUN_2_CLOSURE_REPORT.md` | P0 CI Run #2 Closure Report |
| `P0_TECHNICAL_VALIDATION/P0_CLI_PARITY_REPORT.md` | P0 CLI Report and Determinism |
| `P0_TECHNICAL_VALIDATION/P0_DEPENDENCY_REPORT.md` | P0 Dependency Report |
| `P0_TECHNICAL_VALIDATION/P0_DESIGN_CHECKLIST.md` | P0 Design Review Checklist |
| `P0_TECHNICAL_VALIDATION/P0_DESKTOP_SMOKE_REPORT.md` | P0 Desktop Real-Window Smoke Report |
| `P0_TECHNICAL_VALIDATION/P0_EXECUTION_PROVENANCE.md` | P0 Execution Provenance |
| `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md` | P0 Exit Checklist |
| `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md` | P0 Final Promotion Report — v0.6.0 Baseline Closure |
| `P0_TECHNICAL_VALIDATION/P0_FIXTURE_REGISTER.md` | P0 Fixture Register |
| `P0_TECHNICAL_VALIDATION/P0_IMPLEMENTATION_LOG.md` | P0 Implementation Log |
| `P0_TECHNICAL_VALIDATION/P0_IPC_PARITY_REPORT.md` | P0 IPC and Core/CLI/Desktop Parity |
| `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md` | P0 Known Limitations |
| `P0_TECHNICAL_VALIDATION/P0_MEMORY_ACCOUNTING_REPORT.md` | P0 Memory Accounting Report |
| `P0_TECHNICAL_VALIDATION/P0_PARSER_RESULTS.md` | P0 Parser Results |
| `P0_TECHNICAL_VALIDATION/P0_PERFORMANCE_REPORT.md` | P0 Performance Report |
| `P0_TECHNICAL_VALIDATION/P0_PLAN.md` | P0 Plan |
| `P0_TECHNICAL_VALIDATION/P0_SECURITY_INPUT_REPORT.md` | P0 Security and Untrusted Input Report |
| `P0_TECHNICAL_VALIDATION/P0_STORAGE_REPORT.md` | P0 Storage Report |
| `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md` | P0 Technical Validation Report |
| `P1_A0_VALIDATION/P1_A0_CORRECTNESS_SMOKE_REPORT.md` | P1-A0 Correctness Closure Desktop Smoke Report |
| `P1_A0_VALIDATION/P1_A0_DESIGN_CHECKLIST.md` | P1-A0 Design Review Checklist |
| `P1_A0_VALIDATION/P1_A0_DESKTOP_SMOKE_REPORT.md` | P1-A0 Desktop Real-Window Smoke Report |
| `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` | P1-A0 Execution Report |
| `P1_A0_VALIDATION/P1_A0_EXIT_CHECKLIST.md` | P1-A0 Exit Checklist |
| `P1_VALIDATION/P1_ANALYZE_DESIGN_CHECKLIST.md` | P1 Analyze Design Review Checklist |
| `P1_VALIDATION/P1_ANALYZE_DETAILS_SMOKE_REPORT.md` | P1 Analyze Details Desktop Smoke |
| `P1_VALIDATION/P1_ANALYZE_EXECUTION_REPORT.md` | P1 Analyze Execution Report |
| `P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md` | P1 Analyze Exit Checklist |
| `P2_VALIDATION/P2_COMPARE_DESIGN_CHECKLIST.md` | P2 Compare Design Review Checklist |
| `P2_VALIDATION/P2_COMPARE_DESKTOP_SMOKE_REPORT.md` | P2 Compare Desktop Real-Window Smoke Report |
| `P2_VALIDATION/P2_COMPARE_EXECUTION_REPORT.md` | P2 Compare Execution Report (incl. the CLI smoke and defect E) |
| `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md` | P2 Compare Exit Checklist |
| `V0_VALIDATION/README.md` | FirmwareSight V0 Validation Workspace |
| `V0_VALIDATION/V0_EXECUTION_PROVENANCE.md` | V0 Execution Provenance |
| `V0_VALIDATION/V0_PLAN.md` | V0 Plan |
| `V0_VALIDATION/V0_TAKEOVER_REPORT.md` | FirmwareSight V0 接手报告 |
| `V0_VALIDATION/analysis/METRICS.md` | Metrics |
| `V0_VALIDATION/analysis/MISUNDERSTANDING_LOG.md` | Misunderstanding Log |
| `V0_VALIDATION/analysis/PAYMENT_SIGNALS.md` | Payment Signals |
| `V0_VALIDATION/analysis/PROTOTYPE_CHANGE_LOG.md` | Prototype Change Log |
| `V0_VALIDATION/analysis/QUOTES.md` | Quote Register |
| `V0_VALIDATION/analysis/STATE_TRANSITION_FINDINGS.md` | State Transition Findings |
| `V0_VALIDATION/analysis/TOOLCHAIN_REQUESTS.md` | Toolchain Requests |
| `V0_VALIDATION/batch_a/BATCH_A_EVIDENCE_INDEX.md` | Batch A Evidence Index |
| `V0_VALIDATION/batch_a/BATCH_A_EXECUTION_PROVENANCE.md` | Batch A Execution Provenance |
| `V0_VALIDATION/batch_a/BATCH_A_STATUS.md` | Batch A Status |
| `V0_VALIDATION/batch_a/BATCH_A_TAKEOVER_REPORT.md` | FirmwareSight V0 Batch A 接手报告 |
| `V0_VALIDATION/batch_a/V0.5.2_RELEASE_BLOCK.md` | v0.5.2 Release Block |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_EVIDENCE_INTEGRITY_CHECKLIST.md` | Batch A Evidence Integrity Checklist |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_MODERATOR_PACK.md` | Batch A Moderator Pack |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_RECRUITMENT_PLAN.md` | Batch A Recruitment Plan |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_RESEARCH_PRICE_ANCHORS.md` | Batch A Research Price Anchors |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_SCHEDULING_TEMPLATE.md` | Batch A Scheduling Template |
| `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_SESSION_SOURCE_INTAKE.md` | Batch A Session Source Intake |
| `V0_VALIDATION/deliverables/V0_FINDINGS.md` | V0 Findings |
| `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md` | V0 Gate Recommendation |
| `V0_VALIDATION/deliverables/V0_PRODUCT_RISK_UPDATE.md` | Product Risk Update |
| `V0_VALIDATION/deliverables/V0_UI_CHANGE_RECOMMENDATIONS.md` | UI Change Recommendations |
| `V0_VALIDATION/deliverables/V0_VALIDATION_REPORT.md` | FirmwareSight V0 Validation Report |
| `V0_VALIDATION/internal/DESIGN_SCOPE_CHECKLIST.md` | Internal Design / Scope Checklist |
| `V0_VALIDATION/internal/DRY_RUN_REPORT.md` | Internal Dry Run Report |
| `V0_VALIDATION/protocol/CONSENT_PRIVACY.md` | Consent & Privacy |
| `V0_VALIDATION/protocol/INTERVIEW_SCRIPT.md` | Interview / Commercial Discovery |
| `V0_VALIDATION/protocol/MODERATOR_GUIDE.md` | Moderator Guide |
| `V0_VALIDATION/protocol/PARTICIPANT_SCREENING.md` | Participant Screening |
| `V0_VALIDATION/protocol/TASK_SCRIPT.md` | Task Script |
| `V0_VALIDATION/prototype/FIXTURE_NARRATIVE.md` | Fixture Narrative |
| `V0_VALIDATION/prototype/PROTOTYPE_CHANGELOG.md` | Prototype Changelog |
| `V0_VALIDATION/prototype/README.md` | V0 Clickable Prototype |
| `V0_VALIDATION/sessions/README.md` | Session Register |
| `V0_VALIDATION/sessions/TEMPLATE.md` | V0 Session — Participant [ID] |
| `assets/ui-mockups/README.md` | UI Mockup Asset Register |
| `fixtures/malformed/README.md` | Malformed Fixtures |
| `golden/reports/README.md` | Report Goldens |
| `templates/ADR_TEMPLATE.md` | ADR-XXXX — Title |
| `templates/BUG_REPORT_TEMPLATE.md` | Bug |
| `templates/DESIGN_CHECKLIST_TEMPLATE.md` | FirmwareSight 设计评审 Checklist（Do's & Don'ts） |
| `templates/FEATURE_SPEC_TEMPLATE.md` | Feature: <name> |
| `templates/RELEASE_CHECKLIST_TEMPLATE.md` | FirmwareSight App Release Checklist |
