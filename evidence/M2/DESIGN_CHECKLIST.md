---
title: "M2 Design Checklist"
doc_id: "FS-EV-M2-001"
product: "FirmwareSight"
status: "SELF_REVIEW"
owner: "Engineering"
last_updated: "2026-10-15"
---

# M2 design checklist (self-review, before the independent review)

Template: `templates/DESIGN_CHECKLIST_TEMPLATE.md`. Scope: the M2 UI surfaces only.
- Details tabs **Objects** and **Libraries** (`apps/desktop/ui/src/Details.tsx`).
- Compare **By object and library** (`Compare.tsx`, `ObjectChanges`).
- Overview **Latest comparison** (`Overview.tsx`, `LatestComparison`).
- The lower-bound wording in `components/MemoryUsage.tsx` (N2).

Evidence class: these results come from source reading and component tests (jsdom). **No native QA has been run on these surfaces**, and no screenshot was taken. Plan section 6 applies.

## Don'ts

| Item | Result | Basis |
|---|---|---|
| No gradient, glass, glow, purple | PASS | The new CSS is `.objectsUnavailable` and `.compareFacts`: flex/grid, gap, margin, colour and font-size tokens only. |
| No shadow on panels or tables | PASS | No `box-shadow` was added. |
| No marketing surface | PASS | — |
| State is never colour alone | PASS | Unavailable attribution uses `StateBadge state="UNKNOWN"`: icon, label and note. |
| Dual theme (ADR-0031) | PASS (source) | Only semantic `--fs-*` variables are used. The contrast script is part of the `check.py` run. |
| No invented values | PASS | Every value is a `var(--fs-…)` token. |
| UI does not re-implement Core | PASS | Charge, ranking, delta, share (basis points, computed in Rust) and the remainder all come over IPC. The UI only formats them. |
| Banned words | PASS | — |

## Do's (the items these surfaces touch)

| Item | Result | Basis |
|---|---|---|
| Unknown: neutral, hollow icon, fact, next step | PASS | Badge, the reason from Rust, and "Analyze the build again with its GNU ld .map file attached…". |
| Absence is never 0 (Diff Row old/new/delta) | PASS | `SideValue` renders "Not present"; there is a test for the Added row. |
| Numbers, keys and hashes in mono | PASS | Table cells and Overview figures use the `mono`/`identity` classes. |
| Key figures have context | PASS | Share of the exact Flash total, the unattributed remainder, and comparability. |
| Tables have semantic headers; sortable columns are buttons | PASS | Reuses `SortHeader`/`Sortable`. |
| Tab order matches visual order; focus ring | NOT_VERIFIED | Needs the native pass. |
| Row height ≤ 40 px at the minimum window; overflow | NOT_VERIFIED | Same `.table` / `ScrollArea` as the existing tables. Not measured. |
