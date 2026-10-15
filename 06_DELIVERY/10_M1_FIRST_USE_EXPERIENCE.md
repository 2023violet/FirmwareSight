---
title: "M1 First-Use Experience"
doc_id: "FS-DEL-010"
product: "FirmwareSight"
version: "0.6.0"
status: "DELIVERED_PENDING_REVIEW"
owner: "Engineering"
last_updated: "2026-10-14"
---

# M1 — First-Use Experience

Authorization: the Owner approved M1 in chat on 2026-10-14, covering copy and jargon, drag-and-drop intake, the Overview memory bars, the Release step order, table overflow, and native QA for AC24/AC26/AC20. Base: `cee270c`. Branch: `feature/m1-first-use`.

**Goal:** a firmware engineer who has never seen FirmwareSight completes Analyze → Compare in 5 minutes without opening Help, and the three deferred native MUST items are observed on a real Windows desktop.

**Out of scope:** new formats (Keil/IAR), object attribution, signing, any Core parser, diff or Gate logic change, schema or migration changes, new runtime dependencies, visual restyling (tokens stay as they are).

## Tasks and acceptance criteria

### T1 — Copy cut and jargon replacement (en + zh-CN)

- Each page intro shrinks to one sentence: ≤ 120 characters in English, ≤ 50 Chinese characters. Longer explanations move to Help or an inline "Why?" disclosure, which stays collapsed by default.
- Getting Started becomes three numbered actions (choose ELF → optionally add MAP → Analyze), replacing the current seven-fact essay.
- Primary surfaces use plain terms, and the precise term stays visible in Details and the Evidence Inspector. Examples: `weakest basis …` → "Evidence: MAP + ELF"; "Pair comparability" → "Comparison quality"; "Admissible for a hard limit" → "Can enforce a hard budget".
- zh-CN is rewritten as natural engineering Chinese rather than literal translation.
- The five Gate states and the four evidence classes are not renamed (ADR-0032 D5). No banned words appear.
- **Accept:**
  - catalog parity (`pnpm typecheck`);
  - a measured before/after table of page-intro character counts;
  - the existing UI tests updated with no lost assertions on behaviour;
  - Help still carries the long explanations.

### T2 — Drag-and-drop intake

- The Rust shell handles the window `DragDrop` event. One ELF dropped stages it as the artifact, exactly as the Choose dialog does. A `.map` dropped while an artifact is selected attaches it. ELF + MAP dropped together does both.
- Mixed or unsupported drops are refused, with a typed, localized message and no staging. This covers directories, more than two files, two ELFs, or a MAP with no artifact.
- No full path ever crosses IPC or appears in the UI. The WebView receives only the existing `SelectionDto` through an app event.
- A drop anywhere in the window opens Analyze. The window shows a drop-target highlight using existing tokens and productive motion, and respects reduced motion.
- **Accept:** Rust unit tests for drop classification and refusal; UI tests for the event-driven state; native observation in T7.

### T3 — Overview memory usage bars

- Rust adds a usage figure to `MemoryRegionDto`: used/capacity as a decimal string with one fractional digit, or `null` with a reason. The UI does not compute percentages.
- Overview shows one bar per region: name, kind, `used / capacity`, the percentage, the risk word, and an icon. Unknown capacity or usage is shown in neutral grey with a hollow icon and a one-line next step. Colour is never the only signal.
- Overview also shows the latest Compare delta for the session, when one exists.
- **Accept:**
  - Rust tests for exact rounding, zero capacity, unknown inputs and >100 %;
  - regenerated ts-rs bindings (`drift/ipc bindings unchanged`);
  - UI tests;
  - token-only CSS;
  - contrast check with no new FAIL.

### T4 — Release page step order

- Inputs first, numbered from 1: **1 Build and baseline**, **2 Project policy** (with readiness), **3 Attachments**, then the Run Gate button. Results follow: Verdict, Findings, Policy readiness, Evidence, then Share and Bundle.
- After a run, focus moves to the verdict heading, so keyboard order matches visual order.
- **Accept:** updated `release.test.tsx` covering order and focus; no behaviour change in Gate, attachments or Bundle.

### T5 — Table overflow

- Wide tables scroll horizontally inside their own container, so the page never scrolls sideways. Headers are never clipped. The first column may be sticky.
- Check Compare sections and symbols, Analyze sections and symbols, and Gate evidence tables at 1024×720 and 1440×900.
- **Accept:** native screenshots at both sizes; no new magic numbers.

### T6 — Native dialog localization (prerequisite for AC20)

- The UI passes localized, length-bounded dialog strings with each command that opens a native dialog: picker titles, Save titles, and the replace confirmation title, message and buttons. Rust displays them and never chooses a language, which keeps ADR-0032's rule that Rust is independent of the UI language.
- **Accept:** Rust tests for bounds and fallback; native observation in both languages in T7.

### T7 — Native QA: NEXT44-AC24, NEXT44-AC26, NEXT45-AC20

- Run the 14 planned cases from the NEXT-47 deferred register on this Windows host.
- **Isolation:** a QA build with identifier `com.firmwaresight.desktop.qa`, so it gets its own app data and config directories. The Owner's store is hashed before and after and is never opened. Inputs are repository fixtures only. Outputs and sentinels go to a scratch directory outside the repo.
- **Driver:** WebView2 CDP for in-page actions, and Win32 for real native dialogs. Windows UI Automation is used to read the accessible name, role and state. The operator is recorded as "Claude (automated driver) on the Owner's Windows host". The step log, screenshots, hashes and decoded outputs are kept as evidence.
- **Accept:**
  - every applicable step is PASS, or a defect is filed, fixed and re-observed;
  - the register is updated with real observations;
  - closure of AC24/AC26/AC20 still requires an independent sign-off, run as a separate review.

### T8 — Integration

- `python scripts/check.py` 20/20, 0 SKIP.
- `templates/DESIGN_CHECKLIST_TEMPLATE.md` filled in for this round.
- `.ai/NOW.md` and `CURRENT_STATE.md` updated.
- One independent review of the final SHA.
- The push to `main` happens only after the Owner approves it.

## Done when

All of T1–T8 are accepted, the gate is green, the review has no S0/S1 findings, and the Owner has approved the push.

## Outcome

Delivered on `feature/m1-first-use`; see [evidence/M1](../evidence/M1/README.md). Accepted: T1, T2 (native drag-and-drop NOT_RUN), T4, T5 (1440×900 not captured), T6. T3 partly: the memory bars shipped, but **the latest Compare delta on Overview was not built** and is carried to a later round. T7: the independent review rated AC24, AC26 and AC20 PASS_WITH_LIMITS, and they remain open; the gaps are listed in the evidence.
