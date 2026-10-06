---
title: "P5 Onboarding, Help and History Design Checklist"
doc_id: "FS-P5-ONBOARDING-HISTORY-DESIGN"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — onboarding, Help and History: design checklist (prompt §13, §14, §16, AGENTS.md 11)

Every box below was settled by running the grep or the command it names against the tree this commit
closes, from `apps/desktop/ui`. Three new surfaces were reviewed — the first-use panel, the Help page and
the History tables — and all three are *evidence* surfaces: dense lists, mono identifiers, factual states,
no primary action. `DESIGN.md` and `assets/design-tokens.json` were read before the first line was
written, and neither was edited to make the new surfaces fit.

## Frozen assets

- [x] **No new design token, and no token changed.** `git diff -- assets/design-tokens.json
  apps/desktop/ui/src/styles/tokens.css` prints nothing, and `assets/design-tokens.json` still reads
  `"version": "0.2.1"` at line 5. `python scripts/check.py --only drift` reports
  `drift/design tokens` PASS, which re-derives `tokens.css` from the frozen file and compares.
- [x] **No magic measurement in the three new stylesheets.** Over `History.module.css`,
  `Help.module.css` and `GettingStarted.module.css`, a grep for
  `#[0-9a-f]{3,8}|rgba?\(|[0-9]+px` returns three lines, all of them the same construct — see the
  disclosure below. Every other value is a token: 61, 24 and 25 `var(--…)` uses respectively, and no
  `@media` rule was added.
- [x] **Banned vocabulary absent.** `grep -rniE
  "gradient|glassmorphism|backdrop-filter|blur|purple|neon|glow|confetti|celebrat|box-shadow|text-shadow"`
  over `History.tsx`, `Help.tsx`, `GettingStarted.tsx`, `App.tsx` and their three modules returns
  nothing. No panel, table or row casts a shadow; nothing celebrates; the only `accent` matches in the
  new CSS are the comments that say the surfaces use none — a read-only viewer has no primary action to
  accent, which each file's header comment states.
- [x] **Dark theme not introduced, brand vocabulary unchanged.** No new colour is named anywhere in the
  commit; the rail, the hairlines and the row rhythm are the ones P1–P4 established, and `App.module.css`
  is untouched — the Help entry reuses `navItem`/`navItemActive`.

### One disclosed measurement

`.srOnly` in `History.module.css` (lines 199-209) is the standard clip technique for a column header that
must be announced but not painted: `width: 1px; height: 1px; margin: -1px; clip: rect(0 0 0 0)`. Those
`1px`s are not a design value — the element is never visible at any size — and `styles/global.css` holds
no visually-hidden utility to reuse. Binding an accessibility technique to a spacing token would make a
screen-reader-only cell depend on the type scale for no benefit, so the literal stays, in the one rule
that is not about appearance. This is the only non-token measurement in the commit.

## The five states and the mono rule

- [x] **States are icon + label (+ count), never colour alone.** History draws every state through the
  frozen `StateBadge`: the disposition as one badge, the findings as five (`Pass n`, `Review n`,
  `Block n`, `Unknown n`, `Not applicable n`), and each recorded budget as its own. The icons are the
  hollow/solid SVG set `StateBadge` already ships; `history.test.tsx` asserts the label text is present
  *and* that the row contains an `svg`, so a state cannot become a swatch.
- [x] **A count of zero stays on the screen.** *shows every state count, including the ones that are
  zero* asserts `Block0` renders, because a run with no blocking finding is a fact about that run.
- [x] **UNKNOWN keeps its neutral grey and hollow icon, and gets a fact sentence.** `Budget` maps a state
  that is neither `exact` nor `partial` to UNKNOWN with the word `Unknown`, and an unrecorded byte count
  renders `Unknown` rather than a number — *leaves an unrecorded budget unknown through a unit change*
  asserts no `0 bytes`/`0 KiB` appears for it before or after the switch. A stored disposition word the
  reader cannot map becomes `Unknown disposition`, not a verdict; the mutation that promoted it to PASS
  was caught (report §5, proof 3).
- [x] **Numbers, hashes, ids, versions and timestamps are mono.** `styles['mono']` appears 25 times in
  `History.tsx` and 7 times in `Help.tsx`: snapshot ids, both digests, stored times, run/release ids, the
  release version, and every byte figure. Tabular figures come from the frozen `.mono` rule; no new face
  is declared.

## Empty-state and error behaviour (AGENTS.md 10, DESIGN.md 5)

- [x] **The first-use panel is guidance, not a gate.** It is a `<section aria-label="Getting started">`
  inside `<main>`, below the controls that answer it — not a `<dialog>`, not an overlay, and nothing on
  Analyze waits for it. `help.test.tsx` focuses *Choose firmware artifact* with the panel open, asserts
  `panel.closest('dialog')` is null and that no `alertdialog` exists, and then checks that hiding the
  panel leaves the empty state and every control exactly where they were. Its copy says so in the first
  sentence: hiding it "does not hide anything else".
- [x] **Every failure is inspectable.** All four new error surfaces — three History tables and the Help
  identity block — render the shared `ErrorPanel` with *What happened / Code / Why we know / What to do /
  Diagnostics ID*, and each one is scoped to its own region: *keeps two healthy tables on screen when the
  third cannot be read* and *keeps a failed identity read inside that one section*.
- [x] **An unknown says what is unknown and what to do next.** An empty store, a filter that matched
  nothing and a row whose budget was never recorded each get their own sentence on the screen, with the
  next action named ("Analyze an artifact and it appears here"), and none of them reuses the accent to
  imply a result.
- [x] **No loading spinner, no marketing hero, no CTA.** The in-flight state is a `role="status"` text
  line per table, the same idiom Compare and Details already use.

## Typography, density and layout

- [x] **One language with the existing tables.** `History.module.css` reuses the hairline-over-card
  pattern, the frozen compact row height, `var(--fs-space-*)` padding and
  `var(--fs-layout-design-target-width)`: the page is a third instance of the table the reader already
  met on Compare, not a new visual system.
- [x] **A shortened value is a convenience, not the only copy.** Ids and digests render through
  `truncateMiddle(…, 8)` in the row and in full inside the opened detail row, with the unit unchanged and
  no second encoding. `Help.tsx` truncates nothing: a version and an identifier are short by nature.
- [x] **Responsive rules unchanged.** No new breakpoint was introduced; the rail is the existing sticky
  column and the workspace keeps its own scroll, so History's three tables behave the way Compare's two
  already do at the widths §38 measured.

## Navigation and scope

- [x] **Four workflow pages, and Help outside that sequence.** The rail lists Analyze / Compare / Release
  / History — §15's canonical set and nothing more — and Help sits under its own
    `aria-label="Help and about"` navigation so it cannot read as a fifth stage. Three guard tests pin it
    (`intake`, `compare`, `release`), and the word-ban lists keep `Settings`, `Pricing`, `Cloud` and
    `SBOM` out of the product's own words while `History` is removed from them because it is now true.
- [x] **No Bundle navigation verb.** `release.test.tsx` still asserts the rail contains no
    `/bundle/i` button and that the bundle heading's `closest('nav')` is null: a bundle remains a section
    of Release, which is the §58 rule this commit did not need to touch.
- [x] **The UI re-implements nothing.** History renders rows it is handed; the page, the ceiling, the
    filter's column set, the sort and the disposition all live in `firmwaresight-storage` (`AGENTS.md` 3).
    The only computation in `History.tsx` is `formatSize`, which is the shared presentation helper and is
    driven entirely by the unit the shell already holds.
- [x] **No path is painted, and none is asked for.** `history.test.tsx` asserts the rendered page matches
    no drive-letter, UNC or `/Users|/home|/Volumes|/mnt` pattern even with a row opened, and
    `intake`/`help` keep the whole-document `<a>` count at zero.
- [x] **DESIGN_CHECKLIST_TEMPLATE items covered.** The boxes above are the template's items, in the
    template's order, with the commands that settled them named.
