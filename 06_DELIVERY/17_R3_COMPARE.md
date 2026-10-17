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

## 3. Accessible-name mapping

This section is filled in as the work lands. The final table is in the commit.

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
