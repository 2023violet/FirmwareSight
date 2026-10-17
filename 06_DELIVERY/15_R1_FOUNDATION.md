---
title: "R1 — Foundation Layer (Instrument Ledger)"
doc_id: "FS-DEL-016"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE"
owner: "Product / Delivery"
last_updated: "2026-10-16"
---

# R1 — Foundation Layer

The Owner authorized this in chat on 2026-10-16. The scope has four parts: ADR-0034 moves to BASELINE; tokens 0.4.0 land together with the light `diff.removed` change, with goldens regenerated on purpose; and the shell and core components are implemented. The phase map is in [ADR-0034](../09_ADR/ADR-0034-instrument-ledger-design-language.md) D7. The design source is [03_DESIGN/09](../03_DESIGN/09_R_REDESIGN_SPEC.md) and the [R0 prototype](../03_DESIGN/prototype/README.md).

## 1. Principle of this phase

R1 builds the parts. Pages move onto them in R2–R5. R1 changes **no page behaviour and no accessible name**. The existing page tests (`*.test.tsx`, `test/order.ts`) must pass unmodified, apart from those listed in §5. The visible changes in R1 are:

- the shell frame: a 40 px product bar and a rail with icons that collapses below 1180 px;
- semibold (600) replacing `bold` (700);
- the state icons redrawn as five distinct shapes;
- the light removed-row red.

## 2. Work items

| # | Item | Files | Done when |
|---|---|---|---|
| F1 | ADR-0034 → BASELINE. ADR-0031 D6 notes that the exemption is retired. DECISIONS index. | `09_ADR/ADR-0034…`, `ADR-0031…`, `.ai/DECISIONS.md` | Status lines agree |
| F2 | Tokens 0.4.0. New keys: `typography.size.figure` 28, `line_height.figure` 36, `weight.regular` 400, `weight.semibold` 600, `layout.topbar` 40, `layout.subject_bar` 44, `layout.rail_collapsed` 56, `layout.breakpoint.rail_collapse` 1180, `layout.breakpoint.inspector_dock` 1280, `icon.size.sm` 16, `icon.size.md` 20, `icon.stroke` 1.5. Light `diff.removed` → `#C7372C`. Changelog entry. | `assets/design-tokens.json` | JSON valid; no other value moves |
| F3 | Generator: emits weights, figure size and the `icon` group. `CONTRAST_EXEMPT` becomes empty. `--contrast` shows 36/36 ok. | `scripts/generate_design_tokens.py`, `styles/tokens.css` | `--check` passes; 0 FAIL, 0 EXEMPT |
| F4 | Exports follow the token. The embedded styles in `diff_render` / `release_render` use `#C7372C` for `diff.removed`; `status.block` stays `#C9372C`. Goldens regenerated with `update_goldens.py --confirm`, and the diff is reviewed to be exactly that one byte change per file. | `crates/firmwaresight-report/src/{diff_render,release_render}.rs`, `golden/reports/p2-diff.html`, `golden/reports/p4-release/release-report.html` | Report tests and `drift/goldens unchanged` pass |
| F5 | Window minimum size 1024×720, from the token. | `apps/desktop/src-tauri/tauri.conf.json` | Package check passes |
| F6 | `global.css`: token weights instead of `bold`; the figure class. | `styles/global.css` | No literal `bold` left in CSS modules |
| F7 | `Icon`: one in-repo SVG set (viewBox 20, stroke `--fs-icon-stroke`, `currentColor`, 16/20 px). Nav, state, action and object icons. `StateBadge`/`StateIcon` draw from it: PASS filled circle and check, REVIEW triangle and bar, BLOCK octagon and ×, UNKNOWN dashed hollow circle and ?, N/A circle and slash. | `components/Icon.tsx` (+ test) | Every name renders; the state shapes are distinct |
| F8 | Shell: `TopBar` at 40 px with a slot for the command entry (wired in R5). `Rail` with icons, a collapse toggle that is remembered for the session, and auto-collapse below 1180 px; accessible names unchanged. `App.tsx` uses the new rail. | `components/Rail.tsx`, `TopBar.*`, `App.tsx`, `App.module.css` | Existing tests pass; the rail test covers collapse and `aria-current` |
| F9 | `SubjectBar`, built and tested, mounted in R2. It shows a single build or a pair, MAP state, latest/retained, and an actions slot. | `components/SubjectBar.tsx` | Test |
| F10 | `Inspector`: docks at ≥1280 px and becomes an `aria-modal` sheet below that. It has a focus trap; Esc closes it and returns focus to the trigger. It holds the class, source, locator, rule, raw value and a copy action. | `components/Inspector.tsx` | Test covers focus, Esc and both modes |
| F11 | `FactValue` (a number with an evidence-class mark; it opens the Inspector), `DeltaValue` (neutral ▲▼; an absent side reads "— not present", never 0), `LeadAnswer` (a sentence, a figure and a reference line). | `components/{FactValue,DeltaValue,LeadAnswer}.tsx` | Tests, including absent ≠ 0 |
| F12 | `Tabs` (WAI-ARIA tabs, arrow keys and roving tabindex), `Notice` (info, stale, unknown and error; for errors: what happened, why we know, what to do and diagnostics ID), `DataTable` (column definitions, caption, `th scope`, sort, sticky header, row action, density) with `FilterBar`, and `BuildPicker` (a listbox of builds supplied by the caller). | `components/*` | Tests per component |
| F13 | i18n: new `common.*` keys, in both catalogs, for the component chrome: rail collapse, inspector labels, notice headings, delta "not present", tab and filter labels. No new namespace (ADR-0032 D4). | `i18n/{en,zh}/common.ts` | Catalog test passes |
| F14 | `DESIGN.md` §4–§5 and `03_DESIGN/03_VISUAL_SYSTEM.md` record the token additions, the icon rule, the five principles, neutral deltas and the new components. | docs | Docs match behaviour |

## 3. Not in R1

- Moving any page onto the new components, the nav renames ("Release", "History"), the command palette, and D3. These belong to R2–R5.
- Report layout redesign and the Markdown export. These belong to R6.

## 4. Verification

- `python scripts/check.py` must pass every step. That covers token drift, goldens, IPC bindings, Vitest, lint, typecheck, clippy and tests.
- `generate_design_tokens.py --contrast` must show 0 FAIL and 0 EXEMPT in both themes.
- The golden diff is reviewed by hand: one line per file, `#C9372C`→`#C7372C` on `--fs-color-diff-removed`.
- A desktop smoke check of the new rail at 1440 and 1100 px wide, in light and dark (`cargo tauri build` with the QA identifier, using the CDP driver).
- Independent review, then fixes, then an Owner-approved fast-forward push, CI, and deletion of the worktree.

## 5. Test changes allowed in R1

- The state-icon tests may change their path assertions, but not the label or the count.
- The token-file tests in the report crate may change their version string.
- Any other test edit is a scope violation and must be listed in the review.

## 6. Review and smoke (2026-10-16)

An independent review of `ecfc26f` ran in its own session and worktree. Verdict: **PASS_WITH_FIXES**, with no S1 findings. The dispositions:

| Finding | Disposition |
|---|---|
| S2-1: the Inspector sheet's focus trap could be escaped when focus fell to `<body>` | Fixed. Document-level `focusin` and Esc listeners run while the sheet is open, and the panel takes `tabIndex=-1`. Test added. |
| S2-2: focus was lost when the window crossed 1280 px while open, or when the page cleared the evidence | Fixed. The outer element is now the same in both modes (`display: contents` when docked), and focus returns in an effect cleanup. Two tests added. |
| S2-3: no record of the desktop smoke run; the default 1040 window opened with the rail collapsed | Fixed. The default window width is now 1280, so the rail opens expanded. The native smoke ran on the fixed build: 9 PASS, 1 NOT_RUN (the minimum-size clamp needs a human drag). See [evidence/R1](../evidence/R1/README.md). |
| S3: breakpoints and the window minimum were copied by hand | Range queries `(width < 1180px)` / `(width >= 1280px)`. `tokenDrift.test.ts` ties them and `tauri.conf.json` to the token file, and a mutation proved the test fails on drift. |
| S3: the rail toggle announced its state twice | It now has one fixed name, "Page names", with `aria-pressed`. |
| S3: state glyphs used a 1.8 stroke; duplicate glyphs | The token stroke is used. `evidence` gains a plus sign; `swap` is now vertical. |
| S3: FactValue used `aria-pressed`, and its selection was shown by colour only | It now uses `aria-current`, plus an accent underline on `bg.subtle`. |
| S3: LeadAnswer was named twice; DataTable used a "Sort by" name and a duplicate group name | Fixed: `aria-labelledby`, the header text as the button name, and no group wrapper name. |
| S3: FilterBar comment vs `useId` | Fixed. The input carries `data-fs-filter`, which R2's `/` key uses. |
| S3: BuildPicker highlight reset, no scroll into view, focusable list | All three fixed. |
| S3: Tabs `aria-controls` pointed at unrendered panels | It is now set only on the selected tab. |
| S3: Notice heading level was hard-coded | A `level` prop (2–4), default 3. |
| S3: Gate glyphs used for non-Gate facts | The stale Notice and the retained chip use the `history` glyph, and the MAP chip uses `file`. The error Notice keeps the BLOCK shape because it blocks the action. The Owner may revisit this in R2. |
| S3: Inspector shadow | `overlay-sm`, per spec 09. |
| S3: TopBar had no command slot | A `command` slot was added; R5 fills it. |
| S3: stale weight comment; `th`/`strong` at 700 | The comment was removed. `strong`, `b` and `th` use the semibold token. |
| S3: ADR-0034 said R1 revises all of DESIGN.md | The ADR's Consequences now say which sections R1 revised and which phases revise the rest. |

Only `components/foundation.test.tsx` (new in R1) and the new `components/tokenDrift.test.ts` changed. Existing page tests are untouched (§5).
