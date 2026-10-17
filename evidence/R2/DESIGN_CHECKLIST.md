---
title: "R2 Design Checklist"
doc_id: "FS-EV-R2-001"
product: "FirmwareSight"
status: "SELF_REVIEW"
owner: "Engineering"
last_updated: "2026-10-16"
---

# R2 design checklist

Template: `templates/DESIGN_CHECKLIST_TEMPLATE.md`. Scope:
- Analyze (`Analyze.tsx`, `Details.tsx`);
- Overview (`Overview.tsx`);
- the R1 components as these pages use them (SubjectBar, LeadAnswer, FactValue, DeltaValue, Tabs, Notice, Inspector, Icon).

Two kinds of evidence back this checklist:
- source reading, plus a mechanical grep of `git diff be02f5a` over the UI sources;
- 474 component and page tests (jsdom), and the native smoke on the QA build in [README.md](README.md) (10 PASS, 1 NOT_RUN).

The independent review of `19bd5b9` asked for this file (finding S2-4).

## Don'ts

| Item | Result | Basis |
|---|---|---|
| No gradient | PASS | The diff adds no `linear-`, `radial-` or `conic-gradient`. FactValue's half-filled Derived mark is a pseudo-element, not a gradient. |
| No glass, glow or neon | PASS | No `backdrop-filter`, `text-shadow` or glow was added. |
| No purple; blue only as accent | PASS | No literal colour was added to CSS. Every colour is a `var(--fs-color-…)`. |
| No shadow on panels or tables | PASS | The only added `box-shadow` is `--fs-shadow-overlay-sm`, on the Export popover (an overlay). The Inspector sheet uses `overlay-sm` (R1). |
| No marketing surface | PASS | The lead is a sentence, one figure and a reference line, sitting on a hairline. There is no hero, banner or CTA band. |
| State is never colour alone | PASS | Every state uses `StateBadge` or `Icon`: the five shapes plus a label. The SubjectBar freshness chips carry words and the `history` glyph. Deltas carry ▲▼ and spoken words. |
| Dual theme (ADR-0031) | PASS | Semantic tokens only; dark was checked natively (smoke B09). `--contrast`: 0 FAIL, 0 exemptions (tokens 0.4.0). |
| No invented values | PASS | The diff adds no px, colour or font-size literal. The `40ch`/`32ch` name caps follow the existing TopBar `40ch` convention, and the popover uses `max-content`. |
| UI does not re-implement Core | PASS | Every figure, word and row is shown as returned over IPC. `analyzedWithMap` reads Core's capability word and does not compute one. |
| Banned words | PASS | None. |

## Do's

| Item | Result | Basis |
|---|---|---|
| The five states shown correctly | PASS | `StateBadge` for budgets, capabilities and the readiness verdict. "Not run" is an UNKNOWN with the hollow glyph and a word. |
| Interaction states and focus ring | PASS | Buttons, tabs, the Objects/Libraries cut and FactValue use `:focus-visible` from `global.css`. Hover and pressed come from the token roles. |
| Unknown: neutral, hollow, a fact, a next step | PASS | The Objects-unavailable notice (existing) and the "MAP: Not provided" chip. "MAP changed since this analysis · analyze again" names the next step. |
| Errors have their four parts | PASS | ErrorPanel is unchanged: what happened, why we know, what to do, diagnostics ID. |
| Empty state offers a way out | PASS | Analyze shows the empty notice and Getting Started. Overview leads with the first step, and when a file is already chosen the button reads "Analyze selected artifact". |
| One primary action per state | PASS | Header actions are secondary on Overview, and the primary is the lead's own. Tests cover four states; the smoke checked it natively (B08). |
| Numbers, hashes and paths in mono | PASS | FactValue, DeltaValue, the build line hash and size, and the SubjectBar name. |
| Row height and spacing on the token grid | PASS | The tables are unchanged (D-R2-2). The new layout uses `--fs-space-*` gaps only. |
| Motion only from tokens; reduced motion respected | PASS | No new motion apart from the R1 rail width transition, which uses the token and the global reduced-motion rule. |
| Diff rows never use 0 for "absent" | PASS | DeltaValue renders `null` as "— not present". The Overview comparison passes Core's null through. |
| Key figures have context | PASS | The lead reference line gives RAM, the strength of the load evidence and the evidence count. The build line names whose numbers they are. |
| One focus per screen | PASS | One LeadAnswer per page; the result band of four equal KPI cards is gone from Analyze. |
| Accessible names; semantic tables | PASS | The tab panels are named by their tabs and the regions by their sentence. The mapping is in `06_DELIVERY/16` §3. Tables keep `caption` and `th scope`. |
| Tab order matches visual order; Esc closes overlays | PASS | The Inspector returns focus to its trigger; the sheet traps Tab (R1). The Export menu closes on Esc and on focus leaving, and returns focus to its button. Natively checked: B04, B05. |
| The capability banner reflects the evidence | PASS | The SubjectBar's MAP word is the analysis's own (review S2-1). The full capability list is in Evidence › Capabilities. |
| Window minimum 1024×720, checked by a human drag | NOT_VERIFIED | Carried from R1 A10. A programmatic resize bypasses the minimum. |
| Screen reader pass | NOT_VERIFIED | Belongs to R7, where the human runbook covers NVDA and Narrator. |
