---
title: "R7 Design Checklist"
doc_id: "FS-EV-R7-001"
product: "FirmwareSight"
status: "SELF_REVIEW"
owner: "Engineering"
last_updated: "2026-10-17"
---

# R7 design checklist

Template: `templates/DESIGN_CHECKLIST_TEMPLATE.md`.

Scope: the whole redesign as it stands after R1–R7, with R7's own changes (tokens 0.4.1, the `Unknown` vocabulary, report table semantics) read line by line.

Evidence:
- source reading and a grep of `git diff 37ee635` over the UI and report sources;
- 529 component and page tests; the report crate's tests including the table-semantics test;
- the R2–R6 smoke drivers on one build and the A4 printed-page checks in [README.md](README.md).

## Don'ts

| Item | Result | Basis |
|---|---|---|
| No gradient, glass, glow or neon | PASS | R7 adds no `gradient`, `backdrop-filter` or `text-shadow`. |
| No purple; blue only as accent | PASS | No colour was added or moved (ADR-0034 A1). |
| No shadow on panels or tables | PASS | No `box-shadow` was added. Overlays keep `overlay-sm`. |
| No marketing surface | PASS | No page structure changed. |
| State is never colour alone | PASS | Unchanged; the catalog now also keeps the word `Unknown` in Chinese, so the evidence class reads the same in both languages. |
| Dual theme | PASS | Semantic tokens only. Dark was checked natively on every workspace (B09, C09, R12, H11). |
| No invented values | PASS | The eight `z-index` literals and the palette's width sum became tokens; `--check` now refuses a bare `z-index`. |
| UI does not re-implement Core | PASS | No logic changed. The `.symtab` ranking observation is left to Core (plan §4). |
| Banned words | PASS | None added; the existing tests pin them. |

## Do's

| Item | Result | Basis |
|---|---|---|
| One answer first | PASS | Every workspace's lead answer, re-run in the smoke (B03, C01, R02, H05). |
| Unknown: neutral, with a reason; never 0 | PASS | Unchanged; the vocabulary rule now covers `Unknown`. |
| Numbers, hashes and paths in mono | PASS | Unchanged. |
| Tables have semantic headers | PASS | App: DataTable's `caption` and `th scope`. Exports: R7 added `scope` to every header cell and a caption to every table (Q4), held by a test. |
| Esc closes overlays; focus returns | PASS | Inspector, Sheet and palette, re-run in the smoke (B04, R08, H06, H07). Their stacking order is now the `layer` scale. |
| Exports print on A4 | PASS | `@page` A4, repeating headers, rows kept whole, nothing wider than the sheet (print checks). |
| Window minimum 1024×720, checked by a human drag | NOT_VERIFIED | Carried from R1 A10. |
| Screen reader pass (NVDA, Narrator) | NOT_VERIFIED | Needs a person; listed in the plan §3. |
| 125% / 150% display scaling | NOT_VERIFIED | Needs a person; listed in the plan §3. |
