---
title: "R3 Design Checklist"
doc_id: "FS-EV-R3-001"
product: "FirmwareSight"
status: "SELF_REVIEW"
owner: "Engineering"
last_updated: "2026-10-16"
---

# R3 design checklist

Template: `templates/DESIGN_CHECKLIST_TEMPLATE.md`. Scope: the surfaces R3 touches.
- Compare (`Compare.tsx`);
- the Analyze Details tables and the region table (`Details.tsx`, `Analyze.tsx`);
- the R1 components as R3 extends them: DataTable server mode, FilterBar apply mode, the shared `ExportMenu`, and BuildPicker naming.

Two kinds of evidence back this checklist:
- source reading, plus a mechanical grep of `git diff 56d5609` over the UI sources;
- 484 component and page tests, and the native smoke in [README.md](README.md).

## Don'ts

| Item | Result | Basis |
|---|---|---|
| No gradient, glass, glow or neon | PASS | The diff adds no `gradient`, `backdrop-filter` or `text-shadow`. |
| No purple; blue only as accent | PASS | No literal colour was added to CSS. |
| No shadow on panels or tables | PASS | The only added `box-shadow` is `overlay-sm` on the shared Export popover, which is an overlay. |
| No marketing surface | PASS | The lead is one sentence, one figure and a reference line on a hairline. |
| State is never colour alone | PASS | Deltas went from green/red `+`/`-` to neutral ▲▼ with spoken words (D-R3-3). The Added, Removed and Changed chips carry their words. Comparability and MAP evidence use `StateBadge`: shape plus label. |
| Dual theme | PASS | Semantic tokens only. Dark was checked natively (smoke C09). |
| No invented values | PASS | The diff adds no px, colour or font-size literal. The popover uses `max-content`, and the hint is capped at `48ch` (ch caps follow the TopBar convention). |
| UI does not re-implement Core | PASS | Sorting, filtering and paging go to the shell. Kinds, deltas, comparability, rankings and ambiguity are shown as returned. The lead's largest source is Core's object ranking, or else Core's section ranking. The UI never decides which sections count. |
| Banned words | PASS | None. |

## Do's

| Item | Result | Basis |
|---|---|---|
| Diff rows: old, new and Δ in one row; absent is never 0 | PASS | `SideValue` and `sizeColumn` on every change table. Tested on sections and symbols, and mutation-proven (absent drawn as 0 fails the test). |
| An unpaired row is never a Changed row | PASS | The row note and the Inspector "Pairing" line are tested. |
| Unknown: neutral, with a reason | PASS | `DeltaCell` gives Unknown plus Core's reason. The Objects view without attribution shows the Unknown notice with Core's reason (D-R3-7). |
| One primary action per state | PASS | Compare has one primary. With fewer than two builds it is "Go to Analyze"; otherwise it is Compare. Export is a secondary menu. |
| Numbers, hashes and addresses in mono | PASS | The `numeric` columns of DataTable, the pickers' values, and the SubjectBar names. |
| Tables have semantic headers; sort state is announced | PASS | `caption`, `th scope="col"`, row headers named by `rowHeader`, and `aria-sort`. Sort buttons carry the column's own name. |
| Tab order matches visual order; Esc closes overlays | PASS | The BuildPicker combobox and listbox; Swap between the two sides. The Inspector returns focus to the row. The Export menu closes on Esc and on focus leaving. Checked natively: C04, C05, C07. |
| Key figures have context | PASS | The lead's reference line gives old → new, %, RAM, comparability and the largest object growth. |
| Errors have their four parts | PASS | ErrorPanel is unchanged. A refused export (6002) is the stale note, not an error, and this is tested. |
| Window minimum 1024×720, checked by a human drag | NOT_VERIFIED | Carried from R1 A10. |
| Screen reader pass | NOT_VERIFIED | Belongs to R7. |
