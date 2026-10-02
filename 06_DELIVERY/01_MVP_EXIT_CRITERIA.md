---
title: "MVP Exit Criteria v0.3"
doc_id: "FS-DEL-002"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product / QA"
last_updated: "2026-09-29"
---

# MVP Exit Criteria

FirmwareSight 的 MVP 是**Product MVP**，不是技术 Demo。

Technical Vertical Slice 与 MVP 是两个不同 Gate。

## 1. Technical Validation Exit

进入 Product MVP 前：

- real ELF fixture 可重复解析；
- normalized BuildSnapshot 稳定；
- CLI JSON 与 Desktop 核心结果一致；
- parser 不因用户输入 panic；
- Tauri UI 不被解析任务阻塞；
- golden fixture 通过。

---

# Product MVP Exit

以下全部通过才可以进入 External MVP Validation。

## 2. Core workflow

真实用户可以完成：

```text
Start
→ Analyze
→ Compare
→ Gate
→ Release Bundle
```

五个核心界面全部可用。

---

## 3. UI / UX

MVP UI 必须是 **Minimum Credible Product UI**。

不是：
- wireframe；
- debug panel；
- 临时页面。

必须：
- 1024×720 可用；
- Windows 125%/150% scale 不错位；
- typography/spacing/status/button/table 统一；
- loading/error/empty/unknown 状态完整；
- keyboard 基本可用；
- 无明显布局重叠；
- 用户无需阅读源码或 CLI 才能完成流程。

不要求：
- final branding；
- dark theme；
- advanced animation；
- complex data viz。

---

## 4. Product metrics

外部测试建议至少 8 名真实目标用户。

Activation：
- >80% 完成 Import → Analyze

Time to First Value：
- <60 秒

Core Workflow：
- >=60% 完成 Analyze → Compare

Gate comprehension：
- >80% 正确理解 PASS/REVIEW/BLOCK

Discovery：
- >=30% 通过 FirmwareSight 发现以前没有立即注意到的真实信息

Return intent：
- >=5/8 表示下一次 release 愿意再次使用

Commercial signal：
- >=3 个明确付费意愿
或
- >=1 个真实团队愿意 Pilot

---

## 5. Compatibility

MVP 正式声明支持的格式必须：
- 有 fixture；
- 有 parser regression；
- 有 known limitations；
- 有 capability reporting。

未达到以上条件的只能标：
- Experimental
- Partial
- Unsupported

---

## 6. Quality

- zero known data corruption P0；
- malformed supported input 不 crash；
- import transaction 不留下假 Complete；
- export manifest hashes 可验证；
- Gate deterministic；
- same input/policy → semantically same result。

---

## 7. Deferred by design

以下缺失不会阻止 MVP：

- account；
- cloud；
- team；
- AI；
- SBOM；
- CVE；
- updater；
- dark mode；
- plugin；
- HIL；
- flashing；
- OTA。

MVP 的完整性来自：
**完整工作流，而不是功能数量。**


## v0.4.0 Canonical Stage Mapping

权威 stage identifiers：

```text
G0 Problem Baseline        — passed
V0 Workflow Prototype     — NON_BLOCKING_USER_FEEDBACK_TRACK since ADR-0026 (2026-09-29); its 0/8 sample gates no P-stage
P0 Technical Vertical Slice — PASS, frozen at baseline 0.6.0
G1 = P0 PASS               — basis changed 2026-09-29 by ADR-0026; before that date this file read `G1 = V0 PASS + P0 PASS`
P1 Analyzer                — PASS / COMPLETE (2026-09-29); evidence in P1_VALIDATION/
P2 Compare                 — PASS / COMPLETE (2026-09-29); evidence in P2_VALIDATION/
P3 Release Gate            — PASS / COMPLETE (2026-09-30); evidence in P3_VALIDATION/
P4 Release Bundle          — PASS / COMPLETE (2026-10-01); evidence in P4_VALIDATION/
G2 Product MVP Candidate   — PASS (2026-10-01): Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE;
                              evidence in G2_VALIDATION/
V1 Own-artifact External Validation
P5 Productization
B1 Private Beta
RC1 Release Candidate
GA1 General Availability
```

自然语言 `Problem Validation / Technical Validation / Product MVP / Productization / Beta / RC / GA`
只作描述；自动化、任务、交付和 Agent 必须使用上述 identifier。

V1 比例指标必须报告实际 numerator/denominator；Return intent 阈值统一为 `>=62.5% 且至少 5 人`，不再写死成只适用于 n=8 的 `5/8`。
