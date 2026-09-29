---
title: "P2 Compare Design Review Checklist"
doc_id: "FS-P2-DESIGN"
product: "FirmwareSight"
version: "1.0"
status: "EXECUTION_RECORD"
stage: "P2_COMPARE"
owner: "Design / Engineering"
last_updated: "2026-09-29"
---

# P2 Compare Design Review Checklist

`AGENTS.md` §11 requires every UI change to pass `templates/DESIGN_CHECKLIST_TEMPLATE.md`; prompt §56
binds P2 to the same authority. This is that checklist filled for the Compare page. Authority order
where sources disagree: governance red line > frozen assets, ADR and tokens > `DESIGN.md` > accepted
screenshots > upstream reference.

Read before writing: root `DESIGN.md`, `assets/design-tokens.json` (v0.2.1),
`apps/desktop/ui/src/styles/tokens.css` (generated), and the earlier passes in
`P0_TECHNICAL_VALIDATION/P0_DESIGN_CHECKLIST.md`, `P1_A0_DESIGN_CHECKLIST.md`.

**No new design token was required, so prompt §56's STOP condition was not reached.**
`git diff 7a13660..HEAD -- assets/design-tokens.json DESIGN.md` is empty: the token file is still
v0.2.1 and `DESIGN.md` is untouched. `drift/design tokens` → PASS, and `tokens.css` was never
hand-edited.

Method: every ⚙ box was checked against the tree with the command written beside it, so the review is
repeatable. Anything the command does not settle is said in prose with its file and line.

## Don'ts（禁区 · 一票否决）

- [x] ⚙ **No gradient.** `grep -rniE "gradient|backdrop-filter|box-shadow|text-shadow|drop-shadow|\bdark\b|prefers-color-scheme" apps/desktop/ui/src --include="*.css"` (excluding the generated `tokens.css`) → **no match at all**. Compare builds all of its hierarchy from hairlines, `bg.subtle` and alignment — the `DESIGN.md` §4 device — including the dense section/symbol tables, where a card wall or a shadow would have been the easy wrong answer.
- [x] ⚙ **No glassmorphism.** Same grep, `backdrop-filter` → no match.
- [x] ⚙ **No neon or glow.** Same grep, all three shadow properties → no match. Nothing in the new sheets is authored with a shadow, so nothing can exceed the `overlay-sm` / `overlay-lg` pair.
- [x] ⚙ **No purple-blue palette, no hard-coded colour.** `grep -rnE "#[0-9a-fA-F]{3,8}\b" --include="*.css"` over the authored sheets → **one hit and it is prose**: the pre-existing comment in `Analyze.module.css:108` that names `#98A2B3` while explaining the P0 invisible-label defect. Every rendered colour arrives as a `--fs-color-*` variable, including the three diff-vocabulary chips (`--fs-color-diff-added`, `-removed`, `-changed`).
- [x] ⚙ **No shadow beyond the overlay pair.** Nothing is authored.
- [x] **No marketing surface.** The page opens with a title and a two-line subhead that states a limit ("reads the facts stored by Analyze; it never re-reads the files"). No hero, no CTA, no illustration, no case card. The export row is two buttons and a note.
- [x] **State is never colour-only.** The five Gate/evidence states still go through `StateBadge` (icon + label + optional count + note). The change kinds are the *diff* vocabulary and are words with counts: `Compare.tsx:754` records the rule in the code ("Added, Removed and Changed are not Gate states and borrow no Gate icon"), and `Kind` renders the word — colour only reinforces it. The two in-flight announcements use `role="status"`, not a spinner's colour.
- [x] **No dark theme.** `prefers-color-scheme` and `.dark` → no match; MVP is light only.
- [x] ⚙ **No invented values.** The complete set of numeric literals with units in the nine authored sheets is `max-width: 60ch` twice (`Analyze.module.css:23`, and Compare's subhead reusing the same established measure at `Compare.module.css:27`) and `transition-duration: 0ms` / `animation-duration: 0ms` in the reduced-motion reset. **Zero `px` literals.** Token use: 394 `var(--fs-*)` references over 51 distinct tokens, and a `comm` against the definitions in the generated `tokens.css` shows **no reference to an undefined token**.
- [x] **UI does not re-implement parser / diff / gate logic.** Compare renders DTOs. The sign of a delta, what counts as `Added`, whether a pair is comparable and how many rows are unpaired all arrive computed; the page contains no arithmetic on sizes beyond the display-unit division the prompt assigns to it, and `format.ts` only picks a unit and a locale-independent string.
- [x] **Banned phrasing absent.** No "guaranteed", "fully compliant", "zero risk", "intelligent detection". Where the product cannot say more it says less: `Object attribution — unavailable` with the reason, and `Not present` / `Unknown` instead of a reassuring zero.

## Do's（应做 · 逐项勾选）

### 状态与证据

- [x] ⚙ **Five states present and correct.** `StateBadge` remains the only renderer of PASS / REVIEW / BLOCK / UNKNOWN / N/A, each as icon + label (+ count). Compare uses it for the evidence rows and for the memory budgets; the change-kind chips deliberately do not use it, because those are not gate states.
- [x] ⚙ **Interaction states complete.** Focus is global (`styles/global.css:51`), the pickers and buttons carry hover and pressed rules in `Compare.module.css`, and the disabled Compare action is a real `disabled` attribute plus the `.disabled` rule — defect C's screen state was reached and read in the live window, not assumed.
- [x] ⚙ **Focus ring intact.** `:focus-visible { outline: var(--fs-focus-ring-width) solid var(--fs-focus-ring-color); outline-offset: var(--fs-focus-ring-offset) }` — token-defined 2px accent + 2px offset; no rule removes or weakens it anywhere in the sheet.
- [x] **Unknown is honest.** Unpaired and absent sides render as `Not present` with `Delta Unknown` and a sentence for the reason ("present only in the base build: a removal…"); the ambiguous-count warning is a neutral note naming `SYMBOL-AMBIGUOUS` and its number. Neutral grey, hollow icon, fact sentence, next step — no accent reused, nothing hidden, no zero substituted.
- [x] **Errors carry the four elements.** `ErrorPanel` still shows what happened, why we know, what to do, and the diagnostics id. The failed-comparison path was exercised live and produced defect C — the note that says which pair a standing report describes.
- [x] **Dialogs only for real interruption.** The only dialogs the page can open are the OS save dialog and the OS `Replace existing file?` guard (prompt §36). Cancel is a plain note — `Export cancelled. No file was written.` — with no alert role and no red.
- [x] **Empty state has an exit.** `Nothing to compare yet` (`Compare.tsx:325`) carries a button that goes to Analyze; the empty screen is an instruction, not a blank.
- [x] **Next step always stated.** The rail, the pickers and the Compare action say what is available; the stale-report note names the pair on screen and ends with "press Compare to move to it".
- [ ] **Capability banner** — not a new element here: unchanged from the P1-A0 pass, and still honest. Compare's equivalent is the evidence block, checked below.

### 数字与密度

- [x] ⚙ **Mono for numbers.** `--fs-font-mono` plus `font-variant-numeric: tabular-nums` in the generated base; `.mono` and `.monoSmall` (`Compare.module.css:218,223`) and the table figure cell (`:378`) carry it. Hashes, addresses, sizes, deltas and the footer's snapshot ids are all mono in both the app and the exported HTML.
- [x] ⚙ **Row height ≤ 40 px.** No row sets a pixel height; rows are `var(--fs-space-1) var(--fs-space-3)` padding on the metadata line height — the same table component P0/G1 passed with, verified against a real 18-section and 76-symbol render in the smoke.
- [x] ⚙ **Spacing on the 4-px grid, radius in the frozen set.** Every spacing and radius value in the new sheet is a `--fs-space-*` / `--fs-radius-*` reference; the `.kind` chip uses `--fs-radius-control`.
- [x] ⚙ **Font size ≥ 12 px.** Only `--fs-font-size-*` tokens are referenced; no sub-12px token exists in v0.2.1, so no rule can reach one.
- [x] ⚙ **Motion within the frozen budget.** `transition: … var(--fs-motion-duration-micro) var(--fs-motion-ease)` — three uses in Compare, all border/background/color on hover and selection, which is the allowed class.
- [x] ⚙ **Reduced motion respected.** `styles/global.css:66-69` zeroes transition and animation durations under `prefers-reduced-motion: reduce`; Compare adds no animation that could escape it.
- [x] **Diff row has old / new / delta in one column.** The summary reads `Old 256 bytes → New 376 bytes  Delta +120 bytes`, and the tables keep the three figures side by side per row. Observed live, including `Delta file 0` next to `Delta RAM +221` on the same row — the two are not collapsed.
- [x] ⚙ **Added / Removed never wear a fabricated zero.** An absent side is `Not present` and its delta is `Unknown`; the additions list is captioned "absence, never 0" and lists rows whose old size does not exist. Same rule in the CLI human report and in the exported HTML.
- [x] **Key numbers carry context.** Every budget figure names its evidence basis and comparability; growth rows say which rows they were ranked from; the pair comparability states that the weaker side caps it.
- [x] **One focus per screen.** The page's focus is the comparison result; counts, contributors, tables and exports are separated by hairline sections rather than by a KPI wall.
- [x] **Review amber in 12px table text uses the deepened variant** — not exercised by P2: Compare shows no REVIEW cell. `status.review.strong` remains the rule for whoever adds one.

### 图表与可访问性

- [x] **Charts: none added.** Priority `table > ranked bars > compact stacked bar > treemap` is respected by using the top of that list; the top-growth list is a ranked text list, not a bar. No 3D, no donut, no sparkline, no rainbow.
- [x] ⚙ **Icon buttons named, tables semantic.** Every control is a `<button>` with a text label or an `aria-label`; the section and symbol tables are real `<table>` elements with `<caption>`/`<th scope>` and a `Showing 1 to 18 of 18 sections` status line.
- [x] **Keyboard order = visual order.** The page is one linear flow — pickers, actions, summary, contributors, tables, exports — with no portal or trap; the drill-down writes into the filter field the same tab order already reaches.
- [x] **Evidence status reflected, not inflated.** `Base evidence` and `Target evidence` each state `Linker MAP used`, the layout label, the weakest basis (`MapRegionAndElfLoad`) and whether a footprint row was recorded; the missing-MAP degradation was read in the live window, and object attribution is reported unavailable with its reason.

## Prompt §56 reference, item by item

| Required | Where it is |
| --- | --- |
| hairline borders | `--fs-border-width-hairline` throughout the new sheet; no other border width is referenced |
| dense engineering tables | `components/table.module.css` shared with Analyze: metadata size, 1+space padding, mono figures, pager |
| mono numbers | `.mono`, `.monoSmall`, table figure cell — all `--fs-font-mono` |
| existing typography | only `--fs-font-size-*` / `--fs-line-height-*` tokens |
| existing state/evidence vocabulary | `StateBadge` for states, diff tokens for change kinds, no new vocabulary invented |
| old / new / delta | summary and both tables |
| Added / Removed explicit | counts row (`Added 1 / Removed 1 / Changed 16`), filters, and the two addition lists captioned "absence, never 0" |
| top growth contributors | `Top section growth (Changed rows)` and `Top symbol growth`, clicking one filters the table |
| MAP degradation visible | evidence block per side plus pair comparability, live in the smoke |

## Defects this review re-checked after they were fixed

Three of the four desktop-smoke defects were design defects, so they are recorded against the
checklist they violated, not only against the CSS that fixed them:

| | Violated | Cause | Fix | Re-verified |
| --- | --- | --- | --- | --- |
| A | "State is never colour-only / icon + label is the state" | `overflow-wrap: anywhere` on `.value` is inherited and collapsed the label's min-content width to one character, so the long note squeezed `unavailable` into `unav`/`ailabl`/`e` | `StateBadge .label { flex: none }` | Run 2, live window: `— unavailable` on one line |
| B | "one focus per screen" / rail reachability | `.rail` was both `position: sticky` and `align-self: stretch`, so its box was page-height and could not stick | `.railSticky` inner group carries the sticky; the rail box keeps the full-height surface and hairline | Run 2, live window at 1,000 px scroll |
| C | "next step always stated" | the standing-report note existed only on a failed attempt and named builds by file alone — meaningless for two builds of one artifact | `buildLabel` (file + 12-hex prefix, as the pickers use) and `stale` extended to "not the selected pair" | Run 2, live window after Swap |

All three values stayed inside the token vocabulary; none of the fixes needed a new token, a shadow,
or a colour that was not already defined.
