---
title: "P1 Analyze Design Review Checklist"
doc_id: "FS-P1-003"
product: "FirmwareSight"
version: "0.1.0"
status: "EXECUTION_RECORD"
owner: "Design / Engineering"
last_updated: "2026-09-29"
---

# P1 Analyze Design Review Checklist

`AGENTS.md` 11 requires every UI change to pass `templates/DESIGN_CHECKLIST_TEMPLATE.md`. This is that
checklist for `P1_ANALYZE_DETAILS`: the detail area (`Details.tsx`, `Details.module.css`), the
bytes/KiB switch, and the changes to `App.tsx` and `format.ts` that the switch required.

Authority order where sources disagree: governance red line > frozen assets, ADR and tokens >
`DESIGN.md` > accepted screenshots > upstream reference.

Read before writing: root `DESIGN.md`, `assets/design-tokens.json` (v0.2.1),
`apps/desktop/ui/src/styles/tokens.css` (generated), `P1_A0_VALIDATION/P1_A0_DESIGN_CHECKLIST.md`, and
the US-001 criteria in `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md`.

**No new design token was required, so the prompt's STOP condition was not reached.**
`assets/design-tokens.json` is unmodified in `git status` and `drift/design tokens` → PASS, which means
`tokens.css` still regenerates identically from the frozen JSON. The detail sheet consumes 23 existing
tokens and invents none.

Method: every ⚙ box below was checked with the command written next to it, run from
`apps/desktop/ui/src`, so the review is repeatable rather than a matter of taste.

## Don'ts（禁区 · 一票否决）

- [x] ⚙ **No gradient, glass, shadow or glow.**
      `grep -rniE "gradient|backdrop-filter|box-shadow|text-shadow|drop-shadow|prefers-color-scheme|\.dark" --include="*.css" .`
      over the authored sheets → **no match at all**. Hierarchy in the detail area comes from hairline
      rules, alignment and `--fs-color-bg-subtle` on the inspector panel, which is the `DESIGN.md` 4
      device ("靠对齐、分组、发丝边框和排版建立层级"). Light theme only, as before.
- [x] ⚙ **No hardcoded colour.** `grep -rnoE "#[0-9a-fA-F]{3,6}\b" --include="*.css" .` → one hit, and
      it is prose: the pre-existing comment in `App.module.css:103` that names `#98A2B3` while
      explaining the disabled-label defect. `Details.module.css` contains zero hex digits.
- [x] ⚙ **No magic numbers.** `grep -noE "[0-9]+(\.[0-9]+)?(px|rem|em|ch|%|ms|s|vh|vw)\b" Details.module.css`
      → **no match**. The complete set of non-token numbers in the sheet is `minmax(0, 1fr)`, `auto` and
      `max-content`: track-sizing keywords, and the `0` in `minmax` is a floor rather than a dimension.
      Spacing, radius, borders, motion, row height and the inspector's min/max widths all resolve to
      `var(--fs-*)`.
- [x] ⚙ **No purple-blue palette, no marketing surface.** The area is a list of contributors, a unit
      group, a tab strip, a filter form, a table and a pager. No hero, no CTA, no illustration, no
      verb-shaped button ("Explore", "Discover"). Copy states limits: *"This counts stored bytes, which
      is not the same question as what a device budget charges."*
- [x] **State is never colour-only.** The detail area adds no new state vocabulary: five-state output
      still comes from `StateBadge` in the summary (icon + label + optional count). Unknown cells use
      `--fs-color-status-unknown` **and** the word `Unknown` **and** the stored reason as text - the
      colour is never the only carrier, and the neutral-grey-plus-fact-sentence rule for Unknown is
      followed rather than reusing the accent.
- [x] ⚙ **No dark theme.** Covered by the grep above: no `prefers-color-scheme`, no `.dark` rule.
- [x] **UI does not re-implement parser, diff or gate logic.** The filter, the sort, the class
      partition and the page window are executed in SQLite; `Details.tsx` never slices or compares a
      row list. The unit switch is the one arithmetic operation the front end performs, and it is a
      presentation division by 1024 on a value the shell already measured - `formatSize` decides no
      fact, and the guard test asserts it issues **zero** commands.
- [x] **Banned phrasing absent.** No "guaranteed", "fully compliant", "zero risk", "intelligent
      detection". The area names what it is: *"Sections of the analyzed artifact, as the parser recorded
      them"*, *"Symbols, ordered and filtered by the shell rather than by this page"*.

## Do's（应做 · 逐项勾选）

### 密度与排版

- [x] **Dense, not crowded.** Rows use `--fs-row-compact` as their minimum height and
      `--fs-space-1`/`--fs-space-2` cell padding, so 11 evidence rows and a 17-row section table read as
      one screen rather than a brochure.
- [x] ⚙ **Mono for everything a machine wrote.** `--fs-font-mono` with
      `font-variant-numeric: tabular-nums` on addresses, offsets, flags, hashes, sizes, source locators,
      rule names and evidence ids, so columns of `0x080000000` align on the digit rather than the glyph.
- [x] **Numbers align.** Contributor byte counts right-align in an `auto` track; the inspector's values
      all start on one edge because the label track is `max-content` and each `div` is
      `display: contents`. Both were real defects found in the window and are described in
      `P1_ANALYZE_EXECUTION_REPORT.md` §5.
- [x] **Secondary text is secondary.** `--fs-font-size-metadata` with `--fs-line-height-metadata` and
      `--fs-color-text-secondary` for column heads, hints, reasons and the mono diagnostics lines - the
      same three-token pattern the P0 and P1-A0 sheets use.

### 五态与 Unknown

- [x] **Unknown is a first-class state, not a zero.** Every unknown cell renders `Unknown` plus the
      reason the shell sent. In the window: `.bss` file offset, `.debug_info` virtual address (with
      *"this section is host metadata and is not loaded at runtime"*), unnamed symbols, sizeless symbols,
      mapping-symbol kinds. `formatSize(null)` returns the word `Unknown`, so no code path can print
      `0 B` or `0 KiB` for an absent value, and three tests assert the absence of a zero rendering.
- [x] **Unknown never borrows the accent.** It uses `--fs-color-status-unknown`, and the accent stays
      reserved for the active tab underline and the sort arrow.
- [x] **A missing value is not silently repaired.** Where the frozen schema has no reason column
      (`file_offset`, `address`) the UI says `Unknown` with no invented explanation; the reason-loss gap
      is reported in `P1_ANALYZE_EXECUTION_REPORT.md` §6 instead of being papered over.

### US-001 单位切换

- [x] **Default is Bytes.** The `Bytes` radio is checked on first render; asserted by `defaults to bytes`.
- [x] **`1 KiB = 1024 bytes`**, stated in `format.ts` as `BYTES_PER_KIB = 1024` and checked against the
      definition rather than against a snapshot of the output.
- [x] **Applied consistently to exactly the byte quantities US-001 lists**: artifact size, both
      footprints, excluded metadata, section file and memory sizes, contributor sizes, symbol sizes. All
      of them route through the one `formatSize(value, unit)` call, so a cell cannot drift out of the
      switch.
- [x] **Never applied to addresses, file offsets, hashes, counts, ordinals or snapshot ids.** In the
      window, `Entry 0x080000039`, every address and offset, the SHA-256, the snapshot id and the counts
      `17 / 32 / 11` were identical before and after the switch; the Evidence Inspector's raw `Value`
      also stayed verbatim. `leaves addresses, offsets and counts exactly as they were` asserts the
      whole `0x…` set is unchanged.
- [x] **A nonzero count never renders as zero.** Three significant digits
      (`Intl.NumberFormat('en-US', {minimumSignificantDigits: 3, maximumSignificantDigits: 3})`) so 4
      bytes is `0.00391 KiB` and 60 is `0.0586 KiB`. Formatted with an explicit locale, so a build
      server in another locale cannot change a rendered value.
- [x] **Presentation only.** The unit is a prop, not a query dependency: it is absent from the request
      effect's dependency list, and `re-labels every byte figure and calls no command at all` compares
      the command counters before and after. No re-parse, no snapshot, no SQLite write, and
      `drift/goldens unchanged` PASS proves the CLI output did not move.
- [x] **Semantics over widgets.** The switch is a native `role="group"` labelled `Size units` containing
      two `<input type="radio">` elements with real `<label for>` pairs - keyboard-operable, announced,
      and not a styled pair of buttons pretending to be a toggle.

### 组件与交互

- [x] **Tabs use tab semantics**, not buttons styled as tabs: `role="tablist"` with an accessible name,
      `role="tab"`, `aria-selected`, and a `role="tabpanel"` labelled by the active tab.
- [x] **Sortable columns say so.** Each sortable header is a real `<button>` with
      `aria-label="Sort by <label>"` inside a `<th scope="col" aria-sort="…">` that reports
      `ascending` / `descending` / `none`. Choosing the same column twice reverses it, and the reversal
      re-reads from SQLite rather than flipping the rows in memory.
- [x] **The filter is a form.** `<form>` with a `<label for>` naming what is filtered
      ("Filter symbols by name", "Filter evidence by field"), an `Apply filter` submit button, Enter
      submits, and the evidence tab adds a native `<select aria-label="Evidence class">`. The search box
      sets `autoComplete="off"` because form history has nothing useful to offer for a symbol name.
- [x] **The pager reports where the reader is.** `Showing 1 to 11 of 11` in a `role="status"` region,
      with `Previous page` disabled at the first page and `Next page` disabled when the shell reports no
      next offset - disabled by fact, not by guesswork.
- [x] **Errors belong to their area.** A failed detail query renders `role="alert"` inside the detail
      area with message, code, details, remediation and diagnostics id, while the summary the reader
      already trusted stays on screen.
- [x] **No new global surface.** No settings page, no theme control, no command palette: the unit switch
      lives next to the numbers it affects, which is what the addendum's "no global Settings page"
      instruction asked for.

## Known presentation trade-off

`--fs-layout-evidence-inspector-min` (320) is still the inspector's `min-width` and
`--fs-layout-evidence-inspector-max` (420) its `max-width`, both semantically correct uses. It is no
longer used as a contributor-row name column, which was a misuse found in this round.

The sections table has ten columns; at `layout.desktop_min.width = 1024` the evidence table's `Source`
column is wider than the remaining space and the table owns its own horizontal scroll axis. That is a
deliberate device already documented in the sheet's comment. What this round fixed is *which* column
gets clipped: the row action moved to the first column so `Inspect` is reachable without scrolling, and
the full locator is shown unwrapped in the inspector.

A long `Unknown` reason inside a narrow column produces a tall row (see
`P1_ANALYZE_EXECUTION_REPORT.md` §6.5). Keeping the sentence visible was chosen over shortening it,
because a truncated reason is a lost fact.
