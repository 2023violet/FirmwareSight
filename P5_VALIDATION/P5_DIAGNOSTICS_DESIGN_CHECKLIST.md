---
title: "P5 Diagnostics Design Checklist"
doc_id: "FS-P5-DIAGNOSTICS-DESIGN"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — the Diagnostics section on Help: design checklist (prompt §20, AGENTS.md 11)

One surface changed in Commit D: a Diagnostics section added to the page that already exists, plus the
one neutral control that writes the file. `DESIGN.md` and `assets/design-tokens.json` were read before the
first line was written, and neither was edited to make the new section fit. Every box below was settled by
running the command it names against this tree from `apps/desktop/ui`; the counts are output, not intent.

## Frozen assets

- [x] **No new token, no token changed.** `git diff -- assets/design-tokens.json
  apps/desktop/ui/src/styles/tokens.css` prints nothing. `python scripts/check.py --only drift` reports
  `drift/design tokens` PASS, which re-derives `tokens.css` from the frozen file and compares.
- [x] **No magic measurement.** Over `Help.module.css`,
  `grep -nE "#[0-9a-fA-F]{3,8}|rgba?\(|[0-9]+px"` returns **nothing** — no hex, no `rgb()`, no pixel literal
  — and the file carries **37** `var(--…)` uses. The four classes this commit added (`.exports`, `.control`,
  `.status`, and the retained `.health`) take every value from a token; `.control` was copied from the
  neutral control in `Compare.module.css` rather than re-derived, so the one action on a reading page is
  pixel-identical to the neutral action on a workflow page.
- [x] **Banned vocabulary absent.** `grep -rniE
  "gradient|glassmorphism|backdrop-filter|blur|purple|neon|glow|confetti|celebrat|box-shadow|text-shadow"`
  over `Help.tsx` and `Help.module.css` returns nothing. No accent fill anywhere on the page: the only
  `accent` matches in the stylesheet are the two comments that say it uses none.
- [x] **No new page, no new verb, no new colour name.** Diagnostics was placed inside Help rather than as a
  fifth rail entry, and `App.module.css` is untouched.

## The one judgement call worth reading

`DESIGN.md` 5 names five states — PASS / REVIEW / BLOCK / UNKNOWN / N/A — and 9 says colour is an aid, never
the carrier. The first draft of this section rendered store health as a **word painted with the verdict
palette**: `healthy` in `--fs-color-status-pass`, `unhealthy` in `--fs-color-status-block`. That is neither
option cleanly, and the installed-app walk is what made it visible: a value that carries a Gate's colours
on a page with no Gate sits in the worst place between the two readings.

- [x] **Store health is prose, not a badge, and carries no status colour.** `.health` is `font-weight: bold`
  and nothing else; `grep -n "fs-color-status" Help.module.css` now returns exactly one line, and it is
  `.documentFlag` — Commit C's pre-existing "not in this build yet" marker, which uses the *neutral*
  `status-unknown` grey and is unchanged by this commit. The reasoning is written into the stylesheet
  comment rather than left to the diff.
- [x] **The word is always there.** `healthy`, `unhealthy` and `unknown` are rendered as text in every case,
  so a reader who cannot see a colour difference loses nothing, and `help.test.tsx` asserts the words
  themselves (`describes a damaged store in words, and offers no repair`), not a class name. Mutation F in
  `P5_COMMIT_D_DESIGN.md` §12 reddens that test by mistranslating the word — which is the assertion that
  holds, and a colour-only test would not have caught it.
- [x] **`unknown` is not rounded upward.** A health value this build cannot name stays `unknown`; it never
  becomes `healthy` by default, and the row under it says what was not asked and why.

## Typography and the five-state rule

- [x] **Mono for every identifier, version and file name.** `grep -c "styles['mono']" Help.tsx` → **10**:
  version, schema (`v5`), store file name, platform, identifier, binary name, and the diagnostics rows that
  repeat them. The store name is mono in both places it appears, so the page never spells a file name in two
  typefaces.
- [x] **No numeric value is invented by the UI.** Every row is a value the shell answered with; the page
  renders `not reported` until the shell answers, and the test that holds it resolves a deferred promise
  rather than sleeping.
- [x] **No state badge is reused out of its meaning.** `StateBadge` is not imported here, because PASS and
  BLOCK would assert a release decision the Gate was never asked to make.

## Interaction and disclosure

- [x] **One action, disabled while it runs.** The export button is the page's only control; it is `disabled`
  for the duration of the dialog and the write, so a double-click cannot queue a second save.
  `help.test.tsx` asserts the disabled-then-re-enabled sequence on the cancelled path, where the second
  state is the one a user waits for.
- [x] **A pending line is a live region.** Both transient lines carry
  `role="status" aria-live="polite"` (`Help.tsx:384`, `:389`), so the "Waiting for the save dialog…" notice
  and the outcome sentence are announced without stealing focus.
- [x] **Cancellation is reported as a decision, not an error.** `Cancelled.` renders in the same neutral
  sentence slot as `Wrote …`, and no `ErrorPanel` appears for it.
- [x] **A failed export keeps the facts on screen.** The section above the error panel stays rendered;
  `help.test.tsx` asserts the five rows survive an export refusal, because a failed write says nothing about
  what the store already reported.
- [x] **The page does not say where the store lives.** The row is a file name and the note under About says
  so in words; `help.test.tsx` asserts the whole section contains no `/` and no `\`
  (`expect(words).not.toMatch(/[\\/]/)`), which is the design rule and the privacy rule agreeing.

## What this checklist does not cover

The rail, the page rhythm, the document list and the Getting Started panel are Commit C's and were not
restyled here; `git diff --stat HEAD -- apps/desktop/ui/src` shows the three Help files and the IPC pair as
the only UI changes. Responsive behaviour at 125 % and 150 % DPI and keyboard-only traversal are carried
forward in `P5_PRODUCTIZATION_AUDIT.md` §60 and were not re-measured in this commit.
