---
title: "R2 — Analyze and Overview on the Instrument Ledger shell"
doc_id: "FS-DEL-017"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE"
owner: "Product / Delivery"
last_updated: "2026-10-16"
---

# R2 — Analyze and Overview

The Owner authorized this phase in chat on 2026-10-16 ("下一步执行 R2：Overview + Analyze 正式重构"). The design sources are [03_DESIGN/09](../03_DESIGN/09_R_REDESIGN_SPEC.md) §6.1–§6.2 and the accepted [R0 prototype](../03_DESIGN/prototype/README.md). The parts come from R1 ([06_DELIVERY/15](15_R1_FOUNDATION.md)).

## 1. What changes for the reader

**Analyze** becomes one page, where it used to be three stacked ones (capability chips, result band and dossier, then a details area with its own tabs):

1. The header carries **Choose firmware**, **Analyze** and **Export ▾** (PNG / SVG / HTML).
2. The **SubjectBar** names the build (file · short SHA), its MAP state and whether the result is current or retained. The MAP actions (add, replace, remove) and the unit switch live there.
3. Notices replace the scattered paragraphs: analyzing (status), the failure (error, with its four answers) and a retained result (stale).
4. The **LeadAnswer** states Flash used as the one figure. The figure is a FactValue that opens the Inspector. The reference line gives RAM, the strength of the load evidence and the number of evidence records. The four-card result band is gone.
5. Five **Tabs**: Memory · Sections · Symbols · Objects · Evidence.
   - **Memory**: the region table, the accounting facts (layout source, dual-accounted sections, excluded metadata) and the largest sections.
   - **Sections / Symbols**: the existing server-paged tables.
   - **Objects**: Objects | Libraries as a segmented control, plus the Unknown notice when there is no MAP.
   - **Evidence**: build identity (file, SHA-256, size, format, parser, entry, build ID), the capability list, the evidence table and the snapshot footer.
6. The **Inspector** (R1) replaces the in-page evidence panel. It docks beside the page from 1280 px and becomes a sheet below that.

**Overview** leads with its answer:

1. The **LeadAnswer** is the readiness answer, in one of five states (spec §6.1):
   - no analysis: the next step;
   - a selection that has not been analyzed;
   - no Gate run: "Release check not run";
   - a current run: its state, its count and its first reason, plus "not a release permit or certification";
   - a run for another build or policy: UNKNOWN, with the retained run kept beneath.
2. The **SubjectBar** shows the same subject rule `pendingSelectionOf` already enforces.
3. Sections follow in this order: memory (region usage), then the latest comparison with neutral deltas (DeltaValue), then the previous analysis under a stale Notice when the selection moved on.
4. The single primary action is the LeadAnswer's own. The header keeps the secondary navigation.

Every honesty rule from U1P-R1/R2 (`readinessScope`, `pendingSelectionOf`, retained-result attribution) is kept unchanged. Only its presentation moves.

## 2. Decisions and deferrals

| # | Decision |
|---|---|
| D-R2-1 | Server-side paging, sort and filter in Details stay exactly as they are (`04_TECH/14` §3). Only the tab list moves to the page level, and Libraries becomes a segment of Objects. |
| D-R2-2 | The server-paged Details tables, their Apply-filter form and the region table (whose per-row section evidence is a disclosure) keep their markup in R2. Moving them onto `DataTable`, `FilterBar` and the Inspector happens in R3, together with Compare's change tables, so the paged mode of `DataTable` is designed once for all of them. |
| D-R2-3 | The unit switch moves from the Details area to the SubjectBar. The global move (TopBar, Settings, palette) is R5. |
| D-R2-4 | No new IPC, no change to Core or storage, no new i18n namespace. New keys go in `analyze` and `common`. |
| D-R2-5 | The per-region object composition bar of the prototype needs per-region object rows that the IPC does not return. It is deferred, and the Memory view shows the largest sections instead. |
| D-R2-6 | Accessible names that change are listed in §3, and the tests change in the same commit. |
| D-R2-7 | A **build line** under the LeadAnswer (file · sha256 · size · format · section and symbol counts) states whose numbers these are. It sits inside the result region, so for a retained result it is named as retained along with everything else. The Evidence view's dossier keeps the full SHA-256 and the parser, entry and build ID. |
| D-R2-8 | The SubjectBar's MAP chip reads `MAP: <file>` / `MAP: Not provided` in both pages, so the file name of an attached MAP stays on screen. |
| D-R2-9 | Overview leads with the first step when nothing is analyzed or chosen; otherwise the ship answer leads. Header actions are secondary, so each page state has exactly one primary action. |
| D-R2-10 | The SubjectBar is 44 px when it fits and wraps to a second row when the page is narrowed (the docked Inspector at 1440 px, or the 1024 px minimum). A scrolling bar hid controls in the native smoke. File and MAP names shorten with an ellipsis, and the full name is in the title. This refines ADR-0034 D2's fixed 44 px; it is a minimum, not a cap. |
| D-R2-11 | The SubjectBar's MAP word is the analysis's own (`capabilities.map`), not the selection's. When the selection gained or lost a MAP after analyzing, the bar says "MAP changed since this analysis · analyze again". A failed attempt has its own word: "Analysis failed · previous result kept" (review S2-1, S3). |

## 3. Accessible-name and structure mapping (as landed)

| Before | After | Tests changed |
|---|---|---|
| Details tablist "Analyze details" (Sections · Symbols · Evidence · Objects · Libraries) | page tablist "Analyze views" (Memory · Sections · Symbols · Objects · Evidence); panels named by their tab; opens on Memory | `details.test.tsx` opens the view first (`openView`) |
| tab "Libraries" | button "Libraries" (`aria-pressed`) in the Objects view's "Object table" group | `details.test.tsx` libraries test |
| group "Input capabilities" (Analyze capability chips) | removed; the SubjectBar region "Subject of this page" names the file, its MAP and its freshness; capability words in Evidence › Capabilities | `intake.test.tsx` U1R T1–T6 rewritten around the SubjectBar |
| regions "Analysis result" / "Last good analysis result" (result band) | region named by the lead sentence ("Flash used"), inside "Analysis summary" / "Last good analysis" | `intake.test.tsx` hierarchy, `details.test.tsx` KiB test |
| region "Evidence detail" (in-page inspector above the table) | complementary "Evidence" (dialog below 1280 px), focus to its close button, Esc back to the row | `details.test.tsx` inspector tests |
| region "Visual report export" with three buttons | group "Visual report export" with an "Export" disclosure holding the three buttons | `intake.test.tsx` export tests (`exportButton`) |
| `MAP: <file>` / `MAP: Not provided` row | same words, in the SubjectBar chip | none |
| Overview: inputs → verdict → figures | verdict (or the first step) → inputs → figures; header meta → SubjectBar | `overview.test.tsx` three order tests, comparison delta words |
| Overview comparison `+120 bytes` | ▲ 120 bytes, spoken "increase of 120 bytes" | `overview.test.tsx` comparison test |

New tests: `intake.test.tsx` "R2 Analyze page" (lead figure → Inspector, view order, export menu, Inspector closes on a new snapshot) and `overview.test.tsx` "R2 Overview lead" (lead order, one primary action, "Not run").

## 4. Verification

- `python scripts/check.py` must pass every step.
- The existing page tests are updated only as §3 lists, and every other assertion is kept.
- New tests cover the views, the Inspector wiring from an evidence row and from the lead figure, the SubjectBar freshness rules and the Overview lead states.
- A native smoke run on the QA-identifier build (CDP driver): choose `fixtures/elf/m2-objects/target` with its MAP, analyze, visit the five views, open the Inspector from the lead figure and from an evidence row, and check Overview's lead. Run it in light and dark, and at 1440 and 1100 px. Pack it as evidence (summary, AC matrix, hashed ZIP).
- Independent review in its own session and worktree, then fixes, then an Owner-approved fast-forward push, CI, and deletion of the worktree.

## 5. Native smoke (2026-10-16)

The native smoke ran on the final QA build: 10 PASS, 1 NOT_RUN (the human drag of the window minimum, carried from R1). See [evidence/R2](../evidence/R2/README.md).

The run exposed four layout defects, and all four were fixed before the recorded run:
- inline icons inside buttons;
- the lead reading sentence-first;
- the SubjectBar wrapping beside the Inspector;
- the full-height sticky docked Inspector.

## 6. Independent review (2026-10-16)

An independent review of `19bd5b9` ran in its own session and worktree. Verdict: **PASS_WITH_FIXES**, with no S1 findings. `check.py` passed 20/20 and the UI tests passed 469/469.

| Finding | Disposition |
|---|---|
| S2-1: the MAP chip took the selection's MAP while the hash and "Latest" came from the analysis | Fixed (D-R2-11) in both pages, using shared helpers `analyzedWithMap` and `mapChangedSince` in `stateWords.ts`. Test added: analyze without a MAP, add one, and the bar keeps "MAP: Not provided" and says the MAP changed. The U1R tests now start from a selection whose MAP matches its analysis. |
| S2-2: the Inspector stayed open when the result became retained | Fixed. It closes on a new snapshot, when the result becomes retained, and on a view switch. A test was added; it is mutation-proven, and it waits for the effect instead of racing it. |
| S2-3: on Overview, a file that was chosen but not analyzed got "Run it on Release Gate" as its primary | Fixed. Overview leads with the intake whenever nothing is analyzed, and the button reads "Analyze selected artifact". Tests added for this state and for one primary action in the pending and not-current states. |
| S2-4: no design checklist | Added: [evidence/R2/DESIGN_CHECKLIST.md](../evidence/R2/DESIGN_CHECKLIST.md). |
| S3: SubjectBar height, stale comment, long names | D-R2-10. The comment is fixed, and names shorten with an ellipsis. |
| S3: the failure was worded as "not analyzed" | Fixed: the `failed` freshness word. |
| S3: notices used h3 under the h1 | Fixed: stale notices use `level={2}`. |
| S3: the Memory panel contained a "Memory" region | The section is renamed "Memory accounting". The Inspector and the Evidence tab both stay named "Evidence": they have different roles (complementary/dialog versus tab), and renaming one would make it say less. |
| S3: Export menu focus, outside close, width token | Focus returns to the Export button after a choice. Focus leaving the menu closes it. The width is `max-content`, not the rail token. |
| S3: the Inspector showed the raw value twice for evidence rows | The raw block shows only when the raw value differs from the shown one. |
| S3: the Inspector stayed open across a view switch | It now closes (S2-2). |
| S3: about 15 i18n keys were unused | 35 keys were removed from both catalogs. The dynamically built `page.*`, `drop.refused.*` and `evidenceClass.*` keys were kept. |
| S3: the retained lead was named "Flash used" | It is now "Flash used · previous analysis". |
| S3: decision order; stale Overview comment | Both fixed. |

## 7. CI #123 on `4963c51` (2026-10-16)

`Desktop UI (windows-latest)` failed one test: *keeps the stale-export note visible when the new selection hides the export region*. The other nine jobs passed.

**Cause.** This was a product race, not only a test race. Analyze decided whether a Save-dialog answer was stale by a request counter, and the counter was bumped in a passive effect. R2 rendered more per selection change, which widened the window. In that window a late ERR-EXPORT-6002 was taken as the current subject's answer and shown as an error rather than as the stale note.

**Fix.** Two changes:
- ERR-EXPORT-6002 is the stale note on both paths, because Rust is the authority that the subject changed.
- The counter moves in a layout effect, in the same commit as the new subject.

**Proof.** A new deterministic test lands the answer in the same tick as the new subject. It failed before the fix and passes after it (mutation proof). A green rerun alone was not taken as a fix. The full suite ran three times locally with 475/475.
