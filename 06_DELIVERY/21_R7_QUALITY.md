---
title: "R7 Quality"
doc_id: "FS-DEL-021"
product: "FirmwareSight"
version: "0.6.0"
status: "IMPLEMENTED_LOCAL"
owner: "Engineering"
last_updated: "2026-10-17"
---

# R7: quality pass over the Instrument Ledger redesign

**Authorized by:** the Owner's standing goal in chat on 2026-10-17: carry the plan through to the complete product without intermediate confirmations. ADR-0034 D7 names R7 as the last phase.

**Base:** `feature/r6-reports` (`37ee635`). **Branch:** `feature/r7-quality`.

R7 changes no product behaviour. It closes what the earlier phases left loose, runs every phase's native smoke again on one final build, checks the exports as printed pages, and lists what only a person can still check.

## 1. What changes

| ID | Change | Where |
|---|---|---|
| Q1 | **Tokens 0.4.1** (ADR-0034 amendment A1), additive only. `layout.dialog` 640 replaces the palette's `evidence_inspector.max + nav_rail` sum. A unitless `layer` scale (sticky 1, sticky_above 2, popover 5, inspector 10, sheet 20, palette 30) replaces the eight `z-index` literals R1–R5 used. The scrim stays transparent. | `assets/design-tokens.json`, `scripts/generate_design_tokens.py`, `tokens.css`, eight stylesheets |
| Q2 | `generate_design_tokens.py --check` (the `drift/design tokens` step of `check.py`) now refuses a bare `z-index` number in any UI stylesheet. | `scripts/generate_design_tokens.py` |
| Q3 | The i18n catalog test keeps the evidence class `Unknown` (ADR-0032 D5) as it keeps `Observed`, `Derived` and `Declared`. Five Chinese strings had translated it away ("未知"); they now carry `Unknown`, as the rest of the catalog already did. `UNKNOWN`, the state, satisfies the rule for a group label. | `i18n/catalog.test.ts`, `i18n/zh/compare.ts`, `i18n/zh/history.ts` |
| Q4 | **Report tables are fully semantic.** The A4 check (§2.2) found no `scope` on any header cell of the HTML reports, which the redesign spec §6A.2 requires, and six release-report tables with no caption. Every column header now carries `scope="col"`, every row header `scope="row"`, and every table a caption. A test holds both goldens to it. | `diff_render.rs`, `release_render.rs`, `visual.rs`; `every_report_table_is_named_and_every_header_cell_says_what_it_heads` |
| Q5 | Goldens `p2-diff.html` and the `p4-release` set regenerated on purpose (`update_goldens.py --confirm`): only the `scope` attributes and the new captions differ, and the bundle's digest lines with them. | `golden/reports/*` |

## 2. Verification

### 2.1 Full native regression on the final build

One QA-identifier build of this branch ran the R2–R6 smoke drivers in order: 53 PASS, 3 NOT_RUN (the same human window-drag check), 0 FAIL. The build includes Q1–Q3; it was made before Q4, which changes only the markup of the HTML reports (header `scope` and captions) and is held by the report tests and goldens rather than by the desktop smoke. Evidence: [evidence/R7](../evidence/R7/README.md).

| Driver | Result | Kept QA store |
|---|---|---|
| R2 Analyze and Overview | 10 PASS | `r2-kept-092916` |
| R3 Compare | 11 PASS, 1 NOT_RUN | `r3-kept-093103` |
| R4 Release | 14 PASS, 1 NOT_RUN | `r4-kept-093432` |
| R5 History, palette, Settings, Help | 13 PASS, 1 NOT_RUN | `r5-kept-093555` |
| R6 Markdown export | 5 PASS | `r6-kept-093717` |

The first R4 run on this build failed 9 checks because the driver still clicked "发布门禁 页面", the rail name R5 retired. The driver now clicks "发布 页面"; the product did not change. That run's store is kept as `r4-kept-093233`.

### 2.2 Exports as printed A4 pages

Both report goldens were opened in a local headless Edge, under print media at A4 width, with no network:

| Check | `p2-diff.html` | `release-report.html` |
|---|---|---|
| `@page` | `size: a4; margin: 16mm` | same |
| Content wider than the sheet | none (779 px of 794) | none |
| Table headers that repeat on every page | 3 of 3 | 8 of 8 |
| Body rows kept whole across a page break | 96 of 96 | 84 of 84 |
| Captions / tables | 3 / 3 | 15 / 15 |
| Scoped header cells | 21 of 21 | 74 of 74 |
| Scripts, external URLs | 0, 0 | 0, 0 |

Printed to PDF, the comparison report is 16 A4 pages and the release report 11. The visual summary embedded in the release report prints at about half its screen size; it is a summary, and the tables after it carry the same figures in full.

### 2.3 Gates

- `check.py` passes every step (§4).
- 529 UI tests; Rust tests across the workspace; the token drift check including Q2.

## 3. What only a person can check (NOT_RUN)

Automation cannot settle these. Each needs one person at the machine; none blocks the redesign from being used, and none is claimed as passed anywhere.

| Item | Why automation cannot | Since |
|---|---|---|
| Drag the window smaller than 1024×720 and see it stop | A programmatic resize bypasses the window minimum; only a real drag exercises it. The value is tied to the token file by a unit test. | R1 A10 |
| NVDA and Narrator through Analyze, Compare, Release and History | No screen reader can be driven from this session. Landmarks, names, captions, `scope` and live regions are checked by tests and the accessibility tree. | ADR-0034 §7 |
| 125% and 150% Windows display scaling | Changing system DPI is a system setting this session may not change. | ADR-0034 §7 |
| A keyboard-only pass by a person | Every shortcut and focus return is driven by the smoke, but not by a person who has never seen the app. | R5 |
| Drag-and-drop from Explorer | Automation cannot synthesize an OS drag. | M1 |

The D2C human native-QA runbook ([evidence/D2C](../evidence/D2C/HUMAN_NATIVE_QA_RUNBOOK.md)) is the format for these. The accepted-screenshot register (`06_UI_REFERENCE_SCREENS.md`) is replaced once a person has looked at the final build (ADR-0034 Consequences); the R7 smoke screenshots in the evidence ZIP are the candidates.

## 4. Observations carried forward

- **`.symtab` leads the Compare section ranking on the fixture.** The section ranking sorts by the size change of every section the Core diff reports, including non-loaded ones such as `.symtab`. On `m2-objects` that puts `.symtab` first, above the sections that cost Flash. This is Core's ranking, and the UI must not re-rank it (AGENTS.md §3). Whether the ranking should weigh loaded sections first is a Core question for a later round; it is recorded here, not changed.
- The Gate Markdown's accepted-review path is covered by a shell test, not by the native smoke, because accepting a review needs a policy that produces a REVIEW on the fixture.

## 5. Independent review and dispositions

The review of `6ed6652` returned PASS_WITH_FIXES with no S1 or S2 finding. It checked the token values, every `z-index`, every report `<th>` scope and caption placement, and the evidence counts by hand.

| ID | Finding | Disposition |
|---|---|---|
| S3-1 | The bare `z-index` check was case-sensitive, matched inside comments, and let `calc(5 + 1)` through. | Fixed: case-insensitive, comments stripped first, and anything but `var(--fs-layer-*)` or `auto` is refused. Checked by planting `Z-Index: calc(5 + 1)` and reverting it. |
| S3-2 | The z-index scan runs only after `tokens.css` matches. | Kept: a stale `tokens.css` already fails the step; the scan reports on the next run. |
| S3-3 | The visual print HTML's tables had no test of their own. | Fixed: `the_print_summary_tables_are_named_and_their_headers_scoped`. |
| S3-4 | The golden test counts `<th>`/`<th ` and does not check `row` against `col`. | Kept: the renderers emit no other spelling, and the reviewer checked every scope by hand. |
