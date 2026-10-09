---
title: "Stage Gates and Product Maturity Model"
doc_id: "FS-DEL-006"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product / Delivery"
last_updated: "2026-10-07"
---

# Stage Gates

本文件把完整生命周期压缩成可以直接执行的阶段门。

## G0 — Problem Baseline

进入条件：
- 用户群体明确；
- 问题明确；
- 当前替代方案明确；
- 产品边界明确。

FirmwareSight：**PASS**

---

## G1 — Technical Validation

> **Gate basis changed 2026-09-29 by `ADR-0026-open-source-mvp-first-delivery.md`.** G1 was defined here as
> `V0 PASS + P0 PASS` (see the mapping and Pre-G1 sections below). For the current open-source MVP delivery
> it is `P0 PASS`: the technical content of this section — 最小链、必须 清单、极简技术 UI、不进入 清单 —
> is unchanged and still has to hold; only the V0 conjunct was removed. V0 is now
> `NON_BLOCKING_USER_FEEDBACK_TRACK`.

目标：
证明核心技术链成立，而不是验证商业产品。

最小链：

```text
Real ELF Fixture
→ Parse
→ Normalize
→ BuildSnapshot
→ CLI JSON
→ Minimal Desktop Summary
```

必须：
- Core/CLI/Desktop 事实一致；
- 真实 fixture 可重复；
- parser 对坏输入不崩溃；
- UI 不阻塞；
- 数据模型可继续支撑 Compare/Gate。

UI 深度：
**极简技术 UI。**

不进入：
- 最终设计系统；
- 商业包装；
- 账号；
- Cloud；
- SBOM；
- updater。

退出：
G1 PASS 后才能进入 Product MVP。

---

## G2 — Product MVP

目标：
验证真实用户是否能独立完成一个完整 Firmware Release 工作流。

必须闭环：

```text
Start / Project
→ Analyze
→ Compare
→ Release Gate
→ Release Bundle
```

UI 深度：
**Minimum Credible Product UI**

要求：
- 完整；
- 一致；
- 清楚；
- 能让陌生用户操作；
- 没有 Demo 感；
- 但不是最终商业视觉。

关键指标：
- Import activation >80%
- Time to First Value <60s
- Analyze→Compare completion >=60%
- Gate meaning comprehension >80%
- 至少 30% 用户发现真实新信息
- 至少 5/8 用户有再次使用意愿 `POST_MVP / NOT CURRENT GATE`
- 至少 3 个明确付费意愿，或 1 个团队 Pilot `POST_MVP / NOT CURRENT GATE`

前五条依赖真实用户样本，随 V0 反馈轨道一起在 MVP 之后采集，不再是 G2 的前置门槛
（`ADR-0026`，2026-09-29）。当前 MVP 门槛是工程可自证的：本地工作流完整、确定性输出正确、支持的输入行为
可靠、失败可恢复且不掩盖 Unknown、UI 可信、release 输出可移植、跨平台 CI 绿、支持的 fixture 不崩溃、
真实 Windows desktop smoke 通过、测试可复现。

退出：
~~通过真实用户验证，而不是内部自评。~~ 2026-09-29 起修改：G2 的当前退出条件以上是工程验收；真实用户验证
改为 MVP 之后的反馈轨道，属于 `OPTIONAL / POST_MVP`，不阻塞 G2。原文保留在此，因为它记录了本文件此前的
立场。

---

## G3 — Productization

目标：
把“已证明有价值的 MVP”变成可以长期依赖的软件。

核心工作：
- compatibility matrix；
- parser fixture expansion；
- crash/recovery；
- migrations；
- diagnostics；
- complete design system；
- onboarding；
- project/history model；
- documentation；
- install/sign/update readiness。

退出：
陌生用户无需开发团队陪同即可稳定使用。

---

## G4 — Private Beta

目标：
真实项目持续使用。

建议规模：
- 20–50 名用户；
- 3–5 个团队。

必须：
- installers；
- signing/notarization strategy；
- diagnostics；
- privacy；
- changelog；
- update path；
- beta support loop。

开始验证：
Free / Pro / Team 的真实付费边界。 `POST_MVP — 需另行授权，不属于当前 MVP 范围（ADR-0026，2026-09-29）`

---

## G5 — Release Candidate

定义：

> FirmwareSight 已经可以被真实用户用于生产 Release，而不只是“功能齐”。

必须：
- P0 workflow 零阻断 bug；
- supported format matrix green；
- migrations verified；
- security/capability review；
- signed package；
- install/upgrade/recovery；
- Getting Started / Support Matrix / Known Limitations / Privacy。

---

## G6 — GA 1.0

FirmwareSight v1.0 完整性定义：

```text
Analyze
Compare
Release Gate
Release Bundle
History
CLI
Installer
Signing
Update
Documentation
Commercial Distribution
```

“完整产品”不等于“功能最多”。

GA 的意义是：

> 在明确范围内，FirmwareSight 已经可发布、可收费、可维护、可长期依赖。

---

## Post-GA

之后再逐步验证：

- Component Evidence；
- SBOM；
- vulnerability intelligence；
- CI integrations；
- team policy；
- collaboration。

不自动扩展到：
- IDE；
- flashing；
- HIL；
- OTA；
- device fleet；
- cloud dashboard；
- plugin marketplace；
- AI judge。


## v0.4.0 Canonical Stage Mapping

权威 stage identifiers：

```text
G0 Problem Baseline        — passed
V0 Workflow Prototype     — NON_BLOCKING_USER_FEEDBACK_TRACK since 2026-09-29 (ADR-0026); sample 0/8 does not gate any P-stage
P0 Technical Vertical Slice — PASS, frozen at baseline 0.6.0
G1 = P0 PASS               — since ADR-0026 (2026-09-29); was `G1 = V0 PASS + P0 PASS` before that date
P1 Analyzer                — PASS / COMPLETE (2026-09-29); evidence in P1_VALIDATION/, verdict item by item in P1_ANALYZE_EXIT_CHECKLIST.md
P2 Compare                 — PASS / COMPLETE (2026-09-29); evidence in P2_VALIDATION/
P3 Release Gate            — PASS / COMPLETE (2026-09-30); evidence in P3_VALIDATION/
P4 Release Bundle          — PASS / COMPLETE (2026-10-01); evidence in P4_VALIDATION/; the last core
                              product-implementation stage of the ADR-0026 line
G2 Product MVP Candidate   — PASS (2026-10-01): Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE, on
                              the engineering exit above; evidence in G2_VALIDATION/. Not G3 Productization,
                              Private Beta, RC or GA
V1 Own-artifact External Validation — IN_PROGRESS / RECRUITMENT_READY (opened 2026-10-06 under its own architect
                              prompt, delivered as a file and archived with its SHA-256; PAUSED for U1 from
                              2026-10-07 and not resumed by U1's closure; cohort build RE-FROZEN 2026-10-08 to the
                              U1-accepted artifact 11573661113 at head 41bb6a36 under the owner's inline re-freeze
                              authorization, with F3's first freeze preserved and dated superseded). A research track, not a
                              product stage: it measures whether real external firmware engineers, on their own
                              artifacts and the frozen build, can use Analyze / Compare / Gate, learn something
                              true, and come back. Eligible external sessions 0 of a minimum 8; M1–M6 thresholds
                              fixed before any data and unmoved by the re-freeze; protocol pack in V1_VALIDATION/. It authorizes no feature,
                              schema, migration, dependency, cloud, account, telemetry, updater, signing,
                              notarization, licence, pricing, B1, RC or GA — and §40 says even a full pass does
                              not open B1. This line was blank before that round opened it, which is why this file
                              is one of the paths §43 allowed a docs-only activation to touch.
P5 Productization            — PASS_COMPLETE (opened 2026-10-03 under execution prompt v1.0, archived with its
                              SHA-256; closed 2026-10-06 by Commit F3 after §6's exit re-audit found no required
                              engineering item BLOCKED). Productization is ENGINEERING_COMPLETE and the product
                              narrative is "FirmwareSight Productized MVP Candidate"; `active_task` returned to
                              NONE, which authorizes no next track. §4 audit remains written at
                              P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md, the verdict and its boundaries at
                              P5_VALIDATION/P5_FINAL_CLOSURE_REPORT.md, and the carried limitations at
                              P5_VALIDATION/P5_KNOWN_LIMITATIONS.md. This line is not G3, B1, RC1 or GA1, and
                              closing P5 did not make it one
U1 UI Productization Convergence — CLOSED 2026-10-08 by the Architect: PASS_COMPLETE /
                              VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS. Opened 2026-10-07; prompt delivered inline,
                              and the owner registered the track here as `U1` because the prompt's own title says
                              `B1`, `B1` below is Private Beta, and both F3 §37 and V1 §40 record that identifier as
                              NOT AUTHORIZED. A UI productization track, not a product stage and not a gate: it
                              converged the desktop React shell toward the frozen seven-screen reference set in
                              `assets/ui-mockups/` and changed no Core semantics, evidence class, schema, migration,
                              storage contract, wire format, release identity rule, ADR-0028 or ADR-0029 conclusion.
                              Eight units ran on 2026-10-07 and 2026-10-08; each closed at an Architect-review word
                              and none self-issued this verdict — the record says so itself ("R3 Agent did not and
                              could not self-authorize this verdict"). The closing word arrived as an independent
                              visual acceptance record, archived at
                              `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R3_Architect_Final_Verdict_2026-10-08.txt`
                              (5,159 bytes, 52 lines, 0 CR, SHA-256 `8634321c…36c619`, blob `f16f0e7e…`) and
                              recorded at `U1_VALIDATION/U1P_R3_ARCHITECT_FINAL_VERDICT_RECORD.md`; it is an
                              authority record, not an execution prompt, and it authorizes no work.
                              Its scope is part of the word: visual readiness for a controlled V1 external-user
                              study, NOT GA quality, NOT feature completeness, NOT public distribution approval —
                              an Architect waiver of enumerated cosmetic/interaction residuals, not a claim that
                              every acceptance box passed. The tally stays guarded: 25 items, 23 PASS / 1 FAIL /
                              1 NOT_VERIFIED / 0 NOT_CAPTURED, 2 `MISMATCH_PROVED` flags, and the record forbids
                              restating it as 25/25 or 100 %. Four residuals stay enumerated and open (Analyze's
                              1024 metric-band track, Compare's section table one scroll below the 1440×900 fold,
                              the Gate baseline picker resetting on navigation, Overview's never-recaptured
                              Previous-analysis box). It authorizes no feature, dependency, cloud, account,
                              telemetry, updater, signing, notarization, licence, pricing, B1, RC or GA, and it does
                              **not** resume V1: V1 stays `IN_PROGRESS / RECRUITMENT_READY` with its 0 eligible
                              sessions, `paused_for` this track, and the record's
                              closing section reads `NEXT ACTION RECOMMENDATION, NOT EXECUTED` — the cohort
                              re-freeze onto the product artifact needed a separate owner authority before
                              participant #1. That authority arrived later on 2026-10-08 and is recorded on the V1
                              row above, not here: this verdict moved no build and started nobody. Recruitment,
                              consent and moderation still need their own owner authorization. `active_task` is `NONE`. Evidence and the governance documents are in
                              `U1_VALIDATION/`; machine state at `BASELINE.yaml`
                              `u1_execution.architect_final_verdict`. This line is not G3, B1, RC1 or GA1, and
                              closing U1 did not make it one
B1 Private Beta
RC1 Release Candidate
GA1 General Availability
```

自然语言 `Problem Validation / Technical Validation / Product MVP / Productization / Beta / RC / GA`
只作描述；自动化、任务、交付和 Agent 必须使用上述 identifier。

### Pre-G1 条件实现（ADR-0025，2026-09-28 登记；sequencing 已被 ADR-0026 取代）

历史登记原文：`G1 = V0 PASS + P0 PASS` 不变，且 G1 当前为 **NOT CLAIMED**（V0 为 `0 / 8` eligible
external sessions）。`ADR-0025` 在 `P0 PASS` 之后登记一个受限、可回退的 **Pre-G1 Analyze 切片**，
identifier 为 **`P1-A0`**，用途是 Real Artifact Intake + Analyze Summary。它的边界当时是文档级硬规则：

- `P1-A0` 不是 stage gate，不等于 `P1 PASS`，也不关闭 `P1`；
- ~~`P1-A1` 及之后任何切片，前置条件是 V0 Batch A `>=4` eligible external sessions 加 interim architect
  review，并且仍需 architect 另行签发 prompt；~~ **2026-09-29 起该前置条件失效（`ADR-0026`）**，逐阶段
  仍需 architect 另行签发 prompt 这一点不变；
- ~~`P2` / `P3` / `P4` 保持 NOT AUTHORIZED；~~ 现为 P1 完成后可依次成为下一授权阶段，仍需各自 prompt；
- 每个额外切片都需要新的 architect prompt；本文件不授权任何后续工作；
- Pre-G1 切片是 development slice，不创建新 baseline：`baseline_version` 保持 `0.6.0`。

保留这条记录的理由：P1-A0 与其 correctness closure 正是在这套边界下实现并验证的，
`P1_A0_VALIDATION/**` 的证据按这些条款解读。删除它们会让已交付的切片失去其授权上下文。

V1 比例指标必须报告实际 numerator/denominator；Return intent 阈值统一为 `>=62.5% 且至少 5 人`，不再写死成只适用于 n=8 的 `5/8`。
