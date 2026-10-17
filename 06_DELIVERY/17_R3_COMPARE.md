---
title: "R3 — Compare workbench and the shared tables"
doc_id: "FS-DEL-018"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE"
owner: "Product / Delivery"
last_updated: "2026-10-16"
---

# R3 — Compare workbench and the shared tables

The Owner authorized R3 in chat on 2026-10-16 ("FirmwareSight｜下一步执行 R3：Compare 对比工作台重构"). The base is `56d5609`. The Owner's architecture review of R2 (`R2_ENGINEERING_ACCEPTED / READY_FOR_R3`) set the focus:
- Compare's SubjectBar, LeadAnswer, Tabs, DiffTable and Inspector;
- the table work R2 deferred (D-R2-2);
- two principles that must survive the rework: **absent is not 0**, and **an unreliable pairing is not a change**.

The design sources are [03_DESIGN/09](../03_DESIGN/09_R_REDESIGN_SPEC.md) §6.3 and the accepted [R0 prototype](../03_DESIGN/prototype/README.md).

## 1. What changes for the reader

**Compare** follows the same anatomy as Analyze:
1. The header carries **Export ▾**: JSON, HTML report, PNG, SVG and print-ready HTML.
2. A **pair selector** sits under the header: two BuildPickers (Old / Base and New / Target), **Swap** and **Compare**. Under each picker is the recorded facts line it already shows (size, architecture, both budgets, import time). The pickers name what will be compared next.
3. The **SubjectBar** names what is on screen: the computed pair (file · short SHA for each side) and its freshness. The freshness is `Latest`; or "previous result kept" when the pickers have moved on; or "comparison failed · previous result kept". The unit switch sits in it.
4. The **LeadAnswer** gives the Flash change as one figure: a neutral ▲▼ delta that is a FactValue and opens the Inspector. The reference line gives old → new, the percentage, RAM, comparability and the largest growth source.
5. Four **Tabs**:
   - **Summary**: what moved (counts by change kind, unpaired rows), the memory comparison, the ranking lists, and evidence and comparability.
   - **Sections**, **Symbols** and **Objects** (with Objects | Libraries).

   Picking an entry in a ranking list opens its tab with the row filtered, as it does today, but within the tab.
6. **One `ChangeTable`** replaces three hand-written change tables. It is built on the R1 `DataTable` in server mode: the shell sorts, filters and pages exactly as today. The table shows the change kind, the name and old / new / Δ per measured quantity. A row action opens the Inspector with that row's change detail:
   - kind;
   - each quantity's old, new, Δ and reason;
   - the fields that differ and those it cannot decide;
   - ambiguity;
   - the diff handle.

**Analyze › Details and the region table** move onto the same `DataTable` and `FilterBar`, which closes D-R2-2. Region evidence opens in the Inspector instead of a disclosure.

## 2. Decisions

| # | Decision |
|---|---|
| D-R3-1 | No change to Core, IPC, storage or the diff. Deltas, kinds, comparability, ranking, ambiguity and percentages are all shown as the shell returns them. Server-side sort, filter and paging are unchanged (`04_TECH/14` §3). |
| D-R3-2 | **Absent ≠ 0 and unpaired ≠ changed**, carried as rules of the component: `SideValue` writes "Not present" only for the missing side of an Added or Removed row, and "Unknown" with the reason otherwise. Ambiguous rows keep their note and are never offered as a Changed row. Tests pin both rules on all three tables. |
| D-R3-3 | Deltas are neutral: ▲▼ in primary text through `DeltaValue` (ADR-0034 D1.4). The coloured `deltaPositive` and `deltaNegative` classes are removed. The Added, Removed and Changed **kind** chips keep their diff roles, because they are labelled words. |
| D-R3-4 | `DataTable` gains a server mode: columns flagged sortable without a local `sortValue`, a first sort direction per column, and controlled sort. `FilterBar` gains an apply mode (a form with an Apply button) for filters the shell applies. The pager stays the shared `Pager`. |
| D-R3-5 | The pickers become `BuildPicker`, a select-only combobox, so a build reads as file · hash with its import time. The page keeps the facts line under each picker. |
| D-R3-6 | `ExportMenu` moves from Analyze to `components/ExportMenu.tsx` and is shared by both pages. The export stale guard on Compare gets the CI #123 treatment: ERR-EXPORT-6002 is the stale note on every path, and the counter moves in a layout effect. |
| D-R3-7 | The Objects tab is always present. When object attribution is unavailable it shows the Unknown notice with Core's reason, never an empty table. |
| D-R3-8 | No new i18n namespace (ADR-0032 D4). Dead keys are removed in the same change. |

## 3. Accessible-name and structure mapping (as landed)

| Before | After | Tests changed |
|---|---|---|
| Compare pickers: native `select` named "Old / Base" / "New / Target" | `BuildPicker` comboboxes with the same names (label only; the build is the content), `data-value` = snapshot id, options in a listbox | `compare.test.tsx` `pair()` / `choose()` helpers |
| decorative `→` between the pickers | **Swap** sits between the two sides | the arrow test restated as "Swap between them" |
| one long Compare page (what moved, memory, section table, ranking, symbol table, objects, evidence, export) | SubjectBar "Subject of this page" → lead region "Flash change" → tablist "Compare views" (Summary · Sections · Symbols · Objects); the tables are behind their tabs | `openView()` helper; the hierarchy test restates the R3 order |
| region "Export" with five buttons | group "Export" with an "Export" disclosure in the header | `exportButton()` helper |
| coloured `+256 bytes` / `-68 bytes` | neutral `▲256 bytes` / `▼68 bytes`, spoken "increase of 256 bytes" | memory and lead assertions |
| sort buttons named "Sort by X" (Compare and Details) | the column's own name (`aria-sort` on the header carries the state) | `details.test.tsx`, `compare.test.tsx` |
| filter `textbox` "Filter sections by name" | `searchbox` with the same name, inside a form named "Filter sections" with an Apply button | both suites |
| tables wrapped in `ScrollArea` ("viewport") | the shared `DataTable` wrapper ("wrap") | the layout-contract tests |
| region-table disclosure "Inspect sections" | row action "Show section evidence for {region}" opening the Inspector | `intake.test.tsx` region test |
| (none) | change-row action "Show the change detail for {name}" opening the Inspector | new R3 tests |

| Compare export group "Export" (its button also "Export") | group "Comparison export" with the button "Export" (review S3) | export test |
| zh `region.showEvidence` "查看 {name} 的 Section 证据" | "查看 {name} 的节证据" | none |

Behaviour changes that are not names:
- A change table's sort, filter, kind and page reset when its view is re-opened, and when a new comparison lands, because each table mounts with its view. Before R3 the tables stayed mounted and kept their sort across comparisons.
- A ranking pick is applied once (review S2-1).
- The Inspector closes when the unit changes, and when the rows of the table it came from change (filter, sort, page, Objects | Libraries).

New tests:
- `compare.test.tsx` "R3 Compare workbench" (7 tests): absent ≠ 0 on the section and symbol tables, unpaired ≠ changed, a ranking pick opening its view filtered, the change-row Inspector, the lead figure Inspector, the SubjectBar's moved and failed words, and the ERR-EXPORT-6002 rule. Two of them are mutation-proven: absent drawn as 0, and 6002 shown as an error.
- `foundation.test.tsx`: DataTable server mode, FilterBar apply mode.

## 4. Verification

- `python scripts/check.py` must pass every step.
- The existing tests are updated only as §3 lists.
- New tests cover:
  - absent ≠ 0 and unpaired ≠ changed on every ChangeTable;
  - a ranking pick opening its tab, filtered;
  - the Inspector from a change row and from the lead figure;
  - SubjectBar freshness for a moved pair and for a failed attempt;
  - swap;
  - the export stale guard in the same tick;
  - the server mode of DataTable.
- A native smoke run on the QA-identifier build covers:
  - analyze the m2-objects base and target, then compare them;
  - the lead, the four tabs, a ranking pick and the Inspector from a change row;
  - swap, then a stale pair;
  - export cancel;
  - Analyze Details through the new tables;
  - zh and en, light and dark, 1440 and 1100 px.

  The run's evidence (summary, AC matrix, hashed ZIP) and the design checklist go in `evidence/R3/`.
- Independent review in its own session and worktree, then fixes, then an Owner-approved fast-forward push, CI, and deletion of the worktree. QA stores the smoke creates are kept and listed, not deleted.

## 5. Native smoke (2026-10-17)

The final QA build passed 11 checks; 1 is NOT_RUN (the human drag check). See [evidence/R3](../evidence/R3/README.md) and the [design checklist](../evidence/R3/DESIGN_CHECKLIST.md).

The run surfaced two defects, both fixed:
- **The lead's largest source.** It now names the largest object growth when the MAP attributes one, and otherwise falls back to Core's section ranking.
- **Cells wrapping mid-value.** Figures no longer break across lines, and names break only between words.

It also surfaced one Core observation, left unchanged: `.symtab` leads Core's runtime-size growth ranking. The QA stores the runs created are kept and listed in the evidence README.

## 6. Independent review (2026-10-17)

An independent review of `8f932d4` ran in its own session and worktree. Verdict: **PASS_WITH_FIXES**, with no S1 findings. `check.py` passed 20/20 and the UI tests passed 484/484.

| Finding | Disposition |
|---|---|
| S2-1: a ranking pick was re-applied on every return to its view, and it pulled focus out of the tab list | Fixed. The table reports the pick consumed and Compare forgets it. A test covers pick, clear, leave and return: the filter is null and focus stays on the tab. The test is mutation-proven. |
| S2-2: the region Inspector said "N bytes" and Observed for a partial region | Fixed. It uses the table's wording ("At least N bytes"), and the class is Derived (Unknown when there is no evidence). A test was added and is mutation-proven. |
| S3: the Inspector outlived a unit change, an Objects ↔ Libraries switch, and filter, sort or page changes | Fixed (`onReadingChange`, and `unit` added to the reset). |
| S3: tables forget their state on a tab switch; sort resets on a new comparison | Accepted, and recorded in §3. A table mounts with its view, so its reading starts clean. The one unfiltered request before a pick's filtered one is discarded by the request counter. |
| S3: `latestQuery` was assigned during render | Fixed. The callback refs are kept current in a layout effect. |
| S3: the layout-contract test covered only two tables | Fixed. The Objects table is covered too, using an attributed fixture. |
| S3: duplicate name "Export" | Fixed: the group is named "Comparison export". |
| S3: hard-coded "Unknown"; "—" in the lead's raw value | "Unknown" stays untranslated on purpose. It is the fourth evidence class, next to Observed, Derived and Declared, which ADR-0032 D5 keeps as Core's words, and Compare hard-coded it before R3 too. The catalog test's kept-vocabulary list does not yet name it. Adding it there, or moving the word into the catalog, is a follow-up for R7's i18n pass, not an R3 change. The raw value now uses "Not present" for an absent side. |
| S3: commit `f8ed1f0` ("wip … Not for push") would enter main | Left for the Owner to decide at push time (see the push request). The branch is local and unpushed. |
| S3: the zh label change was not in §3 | Listed. |
