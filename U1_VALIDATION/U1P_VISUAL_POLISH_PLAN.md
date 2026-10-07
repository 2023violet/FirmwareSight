---
title: "U1P Visual Polish Plan"
doc_id: "FS-U1P-001"
product: "FirmwareSight"
unit: "U1P_FINAL_VISUAL_POLISH_AND_INFORMATION_HIERARCHY_CONVERGENCE"
written_before_code: true
start_head: "0cc47058b4f81b2d345e14e2a4730d14d408fc1d"
authority: "assets/ui-mockups (design direction) + DESIGN.md + assets/design-tokens.json + ADR-0018"
---

# U1P — what this round is going to change, and what it is not

Prompt: `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_Final_Visual_Polish_Information_Hierarchy_v1.0.txt`
(delivered 36,140 bytes / 1,606 CRLF pairs / SHA-256 `d92bd88f0742c1738ef004a2964fef7dc785e32ca2c0fe39d4c3df6199bf6d4e`;
stored in Git 34,534 LF bytes / SHA-256 `aad286469d74e03bb5e7755959cb0d808472df13f4ed90039edf4ac86c857eb5`).

The Architect's verdict on U1/U1R is **engineering PASS, state-consistency PASS, visual REQUIRES_POLISH** — the
direction is accepted, the composition is not. This document is the §19 gap-audit addendum: it is written before
any code changes, and every page gets the same eight rows.

## 0. The boundary this round holds itself to

| Allowed | Forbidden |
| --- | --- |
| `apps/desktop/ui/src/**` | `crates/**`, `schemas/**`, `migrations/**`, `fixtures/**` |
| Window-page titles in the smallest existing desktop presentation adapter, only if a title must move | any new IPC command, backend capability or database query |
| `U1_VALIDATION/U1P_*.md` and the governance pointers they require | any new dependency, any design-token **value** |
| — | any invented metric, any fake row, any V1 or B1 movement |

No token value changes are planned, so none is requested. `DESIGN.md` §3 keeps every length, colour and duration
pointing at `assets/design-tokens.json`; the composition devices below are built only from numbers that file
already governs. Where the reference shows something the payload does not carry, §5's instruction applies and the
element is **omitted and recorded as REFERENCE_ONLY** rather than faked.

Two rules from `DESIGN.md` had to be reconciled with this prompt before drawing anything:

- **§4 "一屏一个主焦点；首屏不堆 KPI"** (one primary focus per screen; do not stack KPIs on the first screen)
  against U1P §8.4, which asks for a key-metric row in Overview's first viewport. Resolution: the metric row is
  subordinate to the verdict — one bordered band of label-over-figure cells, no hero treatment, no colour, and it
  sits **below** the ship verdict so the verdict stays the single focus. A compact fact strip is the same device
  the product already uses for a table caption; a KPI wall is not.
- **§10 "change facts appear before Gate verdicts"** against U1P §11, which asks for the verdict first on Release
  Gate. Resolution: §10 orders *change facts before verdicts on a comparison surface*; Release Gate is not a
  comparison surface — its own subject is the verdict — and the verdict strip already sits at the top of that page
  today with U1's acceptance. Compare keeps change facts first. No conflict once the two pages are named apart.

## 1. The measured root cause behind U1-V1-01, found while reading the CSS

`components/Panel.module.css:82` sets

```css
grid-template-columns: repeat(auto-fit, minmax(var(--fs-layout-evidence-inspector-min), minmax(0, 1fr)));
```

`minmax()`'s second argument must be a track breadth — a length, percentage, `flex`, `auto`, or an intrinsic
keyword. A nested `minmax()` is not one, so the whole declaration is invalid and the browser drops it, leaving
`SummaryRow` as a **one-column grid**. Every "row" the product draws through that component is therefore a stack.

That is the mechanism behind the finding the Architect rated REQUIRES_POLISH on Overview, and behind the flat,
vertically wasteful feel on Analyze: the rows were authored and never rendered.

The proof is the installed screenshot, not the spec: in `S01_Overview_1440x900.png` the three capability panels
each occupy the full 1,168 px content column at 1440×900, which a valid `minmax(320px, 1fr)` auto-fit row cannot
produce. The fix is to write the track list in the one form that is valid — `minmax(<token>, 1fr)` — and the proof
of the fix is `P01_Overview_1440x900.png` in §28's capture set, measured after the change.

## 2. Shared composition device

One new device, used by four pages, so a "row" means the same thing everywhere: a **band** — a single bordered
container whose cells are separated by internal hairlines instead of each carrying its own box. It is the
reference's own device (FS-UI-01's capability row and metric row are both divided bands) and it is the cheapest
way to remove the stacked-card feel without inventing a new visual language (§15 forbids one).

It is built from existing tokens only: `--fs-color-bg-surface`, `--fs-border-width-hairline`,
`--fs-color-border-default`, `--fs-radius-panel`, `--fs-space-3/4`, and the two layout minima below. Cells that
must stay named regions keep being `<section aria-label>` elements, because `overview.test.tsx:185` — correctly —
requires the capability cells to be reachable as regions.

## 3. Page by page

### 3.1 Global shell (§7) — implementation order step 1

| row | content |
| --- | --- |
| REFERENCE FIRST-VIEWPORT | Rail and top bar are a fixed frame; only the content column scrolls; the rail's own items stay put and a footer line carries the product version. |
| CURRENT FIRST-VIEWPORT | `S01`–`S06`: the rail is `position: sticky` inside a document-height column, so it travels with long page content until its own cell ends (recorded as U1-V1-05). |
| ARCHITECT FINDING | "The shell is coherent, but the page and rail scroll together in places, and the top-level composition feels looser than the references." |
| TARGET COMPOSITION | Viewport-height shell: `.app` owns `height: 100%`, `.shell` gets `min-height: 0`, `.workspace` owns `overflow-y: auto`, the rail owns its own scroll and stops being sticky. `.page` keeps one measure but its gutters tighten so 1440 does not leave a wide unused right field. |
| REAL DATA AVAILABLE | Nothing new is needed; this is layout only. |
| REFERENCE-ONLY ELEMENTS | The reference's rail footer ("FirmwareSight 0.1.0-alpha / All analysis runs on this machine") — the product states the same promise in `TopBar` and Help; duplicating it in the rail would be a second place for a version string to go stale. Not added. |
| IMPLEMENTATION BOUNDARY | `App.tsx`, `App.module.css`, `components/Layout.module.css`. No title, no command, no state change. |
| ACCEPTANCE SCREENSHOT | Every P01–P06 capture shows the rail at the same y while content scrolls; the §21 test asserts the nav is not inside the page-scroll owner. |

No existing test asserts sticky, fixed or overflow behaviour, so this change is unconstrained by the suite; the
rail's five labels, the two `nav` landmarks and the `${label} page` accessible names must not move (pinned in four
test files).

### 3.2 Overview (§8) — step 2

| row | content |
| --- | --- |
| REFERENCE FIRST-VIEWPORT | Header, then a divided capability band (ELF · MAP · Git) across one row, then the dominant "Can we ship now?" panel with its rule lines and next steps, then a divided four-cell metric band. All of it inside one viewport. |
| CURRENT FIRST-VIEWPORT | `S01`: three full-width stacked capability cards consume y≈160–540; the verdict starts at y≈565; "Flash footprint" is only a corner at y≈845. The metrics are below the fold. |
| ARCHITECT FINDING | "The hierarchy is backwards… the Overview's primary value is release/readiness summary." |
| TARGET COMPOSITION | 1 compact header (title, identity meta, actions) · 2 capability band: three cells in one row, each `state glyph + name + chip`, one detail line, one link · 3 the ship verdict, visually the strongest region on the page · 4 metric band under it: **Flash footprint · Runtime RAM · Symbols**, each label + mono figure + evidence note · 5 next steps stay inside the verdict region, beside the verdict they qualify. |
| REAL DATA AVAILABLE | `AnalysisSummaryDto.memory.nonvolatileImageFootprint` and `.runtimeRamFootprint` (both `BudgetDto`: `state`, `classification`, `bytes`, `reason`), `symbolCount`, `capabilities.symbols`, and the whole `GateRunDto` (counts, disposition, findings, run id, timestamp). |
| REFERENCE-ONLY ELEMENTS | **"Largest section"** — `AnalysisSummaryDto` carries `sectionCount` only; the section rows come from a paged query the Overview never issues, and issuing one would be a new database query (§5 forbids). Omitted and recorded. **"Re-run Release Gate"** — the button on the reference executes the gate; Overview can only navigate, which is the disposition already recorded in `U1_UI_GAP_AUDIT.md` §D and unchanged here. |
| IMPLEMENTATION BOUNDARY | `Overview.tsx`, `Overview.module.css`, `components/Panel.tsx`/`.module.css` (the `minmax` fix and the band). No new prop, no new command, no number computed in the UI. |
| ACCEPTANCE SCREENSHOT | `P01_Overview_1440x900.png` shows capabilities + verdict + metrics with no vertical scroll; `R01_Overview_1024x720.png` shows the verdict above the fold with the band allowed to wrap. |

The verdict must remain **one** region containing the verdict, the open findings and the `DESIGN.md` §9 scope
sentence (`overview.test.tsx:209,276`), and the capability cells must remain regions inside the
`Input capabilities` group.

### 3.3 Analyze (§9) — step 3, highest priority

| row | content |
| --- | --- |
| REFERENCE FIRST-VIEWPORT | Header, capability band, then the analysis itself: Sections/Symbols as the primary object with detail context beside it. |
| CURRENT FIRST-VIEWPORT | `S02`: subhead prose, capability pills, selection row, then the "Artifact" dossier (8 rows) and half of "Memory" before y=900 ends. No table is visible at all. |
| ARCHITECT FINDING | "This still reads like an engineering inspector." |
| TARGET COMPOSITION | Keep the header and the U1R strip exactly. Insert one compact **result band** (artifact · flash · runtime RAM · sections · symbols · load-evidence quality) as a divided strip, then mount **`Details`** — the Sections / Symbols / Evidence tablist that already exists — directly under it, and move the Artifact / Memory / Capabilities / Counts dossier **below** the tables as a secondary region. Shorten the subhead to one sentence. |
| REAL DATA AVAILABLE | Everything the band shows is already in `lastGood`: `artifact.fileName`, both budgets, `sectionCount`, `symbolCount`, `capabilities`, `evidenceSummary`. The tables themselves are `Details`' existing bounded queries. |
| REFERENCE-ONLY ELEMENTS | The reference's right-hand per-row detail column for a selected section: `Details` has an Evidence inspector with real per-row detail, but Sections/Symbols rows carry no stored per-row detail to show beside them, so no fake detail pane is built. The Evidence tab's inspector is the honest equivalent and already exists. |
| IMPLEMENTATION BOUNDARY | `Analyze.tsx`, `Analyze.module.css`, `Details.tsx`/`.module.css` for ordering and secondary styling only. `currentSummary` and the U1R contract do not move. |
| ACCEPTANCE SCREENSHOT | `P02_Analyze_1440x900.png` shows a real analysis table with rows visible in the first viewport. |

Two test couplings are load-bearing here and are handled deliberately rather than by weakening anything:
`intake.test.tsx:324` counts the word "Exact" document-wide, and T1–T6 use document-level `getByText` on
`5,432 bytes` / `12 sections` / `34 symbols`. The band therefore shows **figures without repeating state words**
(the state glyph belongs to the dossier rows, which keep the words), and does not display the artifact's byte size
at all, so those strings stay unique. Where a count assertion genuinely encodes the old composition, it is
rewritten to a scoped assertion over the same fact, and the rewrite is listed in the report.

### 3.4 Compare (§10) — step 4

| row | content |
| --- | --- |
| REFERENCE FIRST-VIEWPORT | A `BASE → TARGET` bar across the top, then the delta summary and the section diff. |
| CURRENT FIRST-VIEWPORT | `S03`: two labelled selector columns with five metadata lines each, Swap/Compare, size units, and only then Memory deltas; the section-change table is far below. |
| ARCHITECT FINDING | "That makes Compare feel like a setup form." |
| TARGET COMPOSITION | One compact pair bar: `BASE → TARGET` with each side's identity inline (file · snapshot · size) and the Swap/Compare actions at its end. When a result exists it becomes the first content region: delta summary strip, then the section-change table, with the growth contributors beside it; deep evidence moves below. Before any comparison the bar plus the honest "nothing to compare yet" state is the whole screen. |
| REAL DATA AVAILABLE | `CompareSummaryDto` already carries the memory deltas, comparability, per-side evidence and the change counts; `SectionChangePageDto` / `SymbolChangePageDto` carry the tables. Fixtures in `compare.test.tsx:177-311` already hold real change rows, so no new data is needed. |
| REFERENCE-ONLY ELEMENTS | **"Export diff"** — not implemented, and §10I forbids inventing it. The reference's per-file growth chart stays as the existing ranked lists (DESIGN.md §7: table first). |
| IMPLEMENTATION BOUNDARY | `Compare.tsx`, `Compare.module.css` only. Selection semantics, the "delta = target − base" sentence, the "Not present" rule and the evidence-basis caption stay word-for-word. |
| ACCEPTANCE SCREENSHOT | `P03_Compare_1440x900.png` shows pair identity + delta summary + section diff in one viewport. |

`compare.test.tsx:600-604` reaches the side facts through `getByText('Old / Base').closest('div')`; that locator is
the dossier shape being removed, so those assertions move to the new bar's accessible structure while still
asserting the same four facts.

### 3.5 Release Gate (§11) — step 5

| row | content |
| --- | --- |
| REFERENCE FIRST-VIEWPORT | Verdict dominant, rule findings immediately under it, configuration visually secondary. |
| CURRENT FIRST-VIEWPORT | `S04`: verdict strip, then "1 · Project policy" and "2 · Build and baseline" consume the rest of the viewport; "3 · policy readiness" and "4 · Findings by state" begin at the very bottom edge. |
| ARCHITECT FINDING | "The user should not have to scroll past configuration to understand why the Gate passed/blocked/reviewed." |
| TARGET COMPOSITION | Re-sequence the page so the reading order is verdict → rule-state breakdown (BLOCK / REVIEW / UNKNOWN / PASS / N/A from the run's own findings) → **Gate inputs** (policy and build/baseline condensed into one secondary band, still reachable, nothing hidden) → evidence → bundle. The numeric prefixes are re-sequenced with the sections so they still describe the order the page is read. |
| REAL DATA AVAILABLE | `GateRunDto.findings`, `.counts`, `.dispositionEffectiveSeverity`, `.computedSeverity`, plus the existing project/build/baseline state. Nothing new is fetched. |
| REFERENCE-ONLY ELEMENTS | The reference's bundle-preview side panel: the product has a real bundle preview only after a bundle is prepared, and §11E forbids inventing one — the existing "6/… Release Bundle" region stays where the real data is. |
| IMPLEMENTATION BOUNDARY | `Release.tsx`, `Release.module.css`. **Every rule semantic stays exactly**: severity words, the disposition-vs-computed pair, the accept-review flow, the overwrite confirmation, the non-compliance disclaimer. |
| ACCEPTANCE SCREENSHOT | `P04_ReleaseGate_1440x900.png` shows verdict plus at least one real rule finding line without scrolling. |

`release.test.tsx:795-804` pins the h3 order (findings groups first, then evidence tables) — that order is the
same one this change asks for, so it survives; only `:1181`'s literal `6 · Release Bundle` heading text moves with
the re-sequenced numbering.

### 3.6 Bundle & History (§12) — step 6

| row | content |
| --- | --- |
| REFERENCE FIRST-VIEWPORT | One focused history list with a detail column beside it. |
| CURRENT FIRST-VIEWPORT | `S05`: heading "History" (while the rail and window title say "Bundle & History"), then three stacked tables — Builds, Gate runs, Release records — each with its own filter row. |
| ARCHITECT FINDING | "Technically honest, but visually still feels like storage inspection." |
| TARGET COMPOSITION | Heading becomes **Bundle & History**, matching the rail label and the Rust page title already sent to `set_window_title`. An accessible segmented control chooses **one active entity list at a time**; the selected row's real detail moves into a detail region beside the list at 1440 and stacks under it at 1024. Filters stay available but read as controls of the active list, not as three competing bands. |
| REAL DATA AVAILABLE | The three existing bounded read commands and their rows; the detail content is the set of fields the expanded detail row already renders today (identity, timestamp, artifact, verdict/disposition, stored evidence) — nothing new is queried. |
| REFERENCE-ONLY ELEMENTS | The reference's cross-entity trend column and any bundle action: no command exists, so none is drawn. Entity distinctions (Build / Gate run / Release) stay separate; §12B forbids merging them and the segmented control keeps them apart by name. |
| IMPLEMENTATION BOUNDARY | `History.tsx`, `History.module.css`. The rail label and the `MainWindowPage` wire value `'History'` do not change — only the page's own visible heading. No write action, no new link element, no export button. |
| ACCEPTANCE SCREENSHOT | `P05_BundleHistory_1440x900.png` shows one list plus its selected detail; `R05` shows the same at 1024×720. |

This is the page with the heaviest test coupling (`history.test.tsx` asserts three simultaneous regions, per-table
row counts and exactly three textboxes). The composition change is what §12 asks for, so those assertions are
rewritten to the new shape — one active region per view, the same row counts inside it, and the same forbidden-action
scan — rather than the page being held to the old shape to keep the suite quiet.

### 3.7 Parse Failure (§13) — step 7, composition only

| row | content |
| --- | --- |
| REFERENCE FIRST-VIEWPORT | A failure card: what happened, why we know, what to do, diagnostics, one clear recovery action, and a quiet line saying what was not overwritten. |
| CURRENT FIRST-VIEWPORT | `U1R_S02`: correct semantics, then the retained Artifact dossier begins immediately below and the page runs far past the reference's length. |
| ARCHITECT FINDING | "The current failure layout is clear, but the retained Artifact dossier begins immediately below, making the page much longer than the reference." |
| TARGET COMPOSITION | Keep the failure panel's five parts and the U1R strip words exactly. Make the recovery action the visually prominent move beside the failure. Render the retained result as an explicitly secondary region — its attribution sentence stays always-visible, its bulk reads as continuation rather than as the page's subject. |
| REAL DATA AVAILABLE | The error envelope and the retained `lastGood` summary — both already on screen. |
| REFERENCE-ONLY ELEMENTS | None new. |
| IMPLEMENTATION BOUNDARY | `Analyze.tsx`, `Analyze.module.css`, `ErrorPanel` styling only. **No stale capability chip may return** (§13E), and the "Previous result retained below" / "Previous analysis of …" sentences are not reworded. |
| ACCEPTANCE SCREENSHOT | `P06_ParseFailure_1440x900.png` shows failure + diagnosis + next action + previous-result status without scrolling. |

## 4. What this round is explicitly not doing

- Not changing which page the session opens on. `App.tsx:104-112` lands a first run on Analyze because Overview
  summarizes an analysis and a gate run it does not yet have. U1P §6 asks what the **first viewport** answers, not
  which page loads first, and moving the landing page would trade a real first-run problem for a cosmetic match.
  Recorded as a deliberate non-change.
- Not touching `FS-UI-07` (§14): no dependency detection, no declare-version, no re-scan, no storage, no IPC, no
  fake screenshot, no fake rows.
- Not restyling: no gradient, no glass, no shadow on panels or rows, no rounded consumer card, no new theme, no
  motion decoration, no smaller body text. Density comes from composition (§16).
- Not resuming V1, re-freezing V1's cohort artifact, or moving any stage state (§34, §35).
- Not self-passing §29. If any required first-viewport condition fails in the installed capture, the result is
  `U1P-V2` and `REQUIRES_ARCHITECT_POLISH_REVIEW` naming the page.
