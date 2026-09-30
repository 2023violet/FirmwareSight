---
title: "P3 Release Gate Design Review Checklist"
doc_id: "FS-P3-DESIGN"
product: "FirmwareSight"
version: "1.0"
status: "EXECUTION_RECORD"
stage: "P3_RELEASE_GATE"
owner: "Design / Engineering"
last_updated: "2026-09-30"
---

# P3 Release Gate Design Review Checklist

`AGENTS.md` §11 requires every UI change to pass `templates/DESIGN_CHECKLIST_TEMPLATE.md`; prompt §63
binds P3 to the same authority. This is that checklist filled for the Release Gate page
(`apps/desktop/ui/src/Release.tsx`, `Release.module.css`) and the rail change in `App.tsx`.

Authority order where sources disagree: governance red line > frozen assets, ADR and tokens >
`DESIGN.md` > accepted screenshots > upstream reference.

**No new design token was required, so prompt §63's stop condition was not reached.**
`git diff 32b23aa..HEAD -- assets/design-tokens.json DESIGN.md` is empty, `drift/design tokens` →
PASS, and `tokens.css` was never hand-edited. Every value below resolves to an existing token.

Method: each ⚙ box was checked with the command written beside it, so the review is repeatable.
Anything a command does not settle is said in prose with its file and line.

## Don'ts (one hit fails the review)

- [x] ⚙ **No gradient.** `grep -rniE "gradient|backdrop-filter|box-shadow|text-shadow|drop-shadow|prefers-color-scheme|\.dark" src --include="*.css" --exclude=tokens.css` → **no match at all**. The page builds hierarchy from hairlines, `bg.subtle` and alignment.
- [x] ⚙ **No glassmorphism.** Same grep, `backdrop-filter` → no match.
- [x] ⚙ **No neon or glow.** Same grep, all three shadow properties → no match. The acceptance record block is a hairline box on `bg.subtle`, not a raised card.
- [x] ⚙ **No purple, no hard-coded colour.** `grep -rnE "#[0-9a-fA-F]{3,8}\b" src --include="*.css" --exclude=tokens.css` → **one hit and it is prose**: the pre-existing comment in `Analyze.module.css:108`. `grep -nE "style=\{\{|#[0-9a-fA-F]{3,8}" src/Release.tsx` → no match: no inline style, no literal colour. 131 declarations in `Release.module.css` all read `var(--fs-*)`.
- [x] ⚙ **No shadow beyond the overlay pair.** Nothing is authored.
- [x] **No marketing surface.** The page opens with `Release Gate` and a subhead that states a limit: *"The answer below is that policy's verdict and nothing more: no claim on this page reaches beyond the rules the project itself wrote."* No hero, no CTA, no illustration.
- [x] **State is never colour-only.** All five states go through `StateBadge` (icon + label + optional count) — `Release.tsx:500-513` renders the aggregate and the five counts. Each finding row repeats `icon + REVIEW + effective severity: REVIEW` as words. The `N/A` badge uses a dash glyph, not a grey dot alone.
- [x] **No dark theme.** `prefers-color-scheme` → no match.
- [x] **No invented numbers.** `grep -nE "[0-9]+px" src/Release.module.css` → **no match**: spacing comes from `--fs-space-{1,2,3,4,6,8}`, radii from `--fs-radius-control` / `--fs-radius-panel`, type from `--fs-font-size-body` / `--fs-font-size-metadata`.
- [x] **The UI re-implements no parser, diff or Gate logic.** `Release.tsx` renders fields of `GateRunDto` and never computes a state: `acceptable` is Rust's (`state == REVIEW && acceptance.is_none()` in `release.rs::finding_dto`), the aggregate is Core's, the growth deltas are P2's diff loaded through `load_diff_input` + `compare()`. `release.test.tsx` asserts the page shows what it is handed, including the `policy_source` of a default-policy run.
- [x] **No banned phrasing.** No "guaranteed", "fully compliant", "zero risk", "intelligent detection". The page says `policy readiness`, `as computed`, `not re-observed`.

## Do's — states and evidence

- [x] ⚙ **Five states present and correct.** Observed live in smoke Run 3: PASS (steps 11, 13–17, 19, 20), REVIEW (step 22), BLOCK (steps 29, 33), UNKNOWN (step 35), N/A (step 12). Each with icon + label; the counts row adds the number.
- [x] ⚙ **Interaction states complete.** hover / pressed / focus / disabled exist for `.primary`, `.control`, `.select`, `.link`. This stage also **fixed a defect in them**: a disabled primary button kept its accent border (defect E), so `:disabled` now gives up `border-color` in both `Release.module.css` and `Compare.module.css` — re-seen on `Record acceptance` at smoke step 23.
- [x] ⚙ **Focus ring.** `:focus-visible` on `.control`, `.primary`, `.link`, `.select` with the token ring (2px accent + offset); nothing removes or flattens it.
- [x] **Unknown is honest.** Neutral grey, hollow glyph, a factual sentence naming what is missing, and a next step: `Workspace dirty state is unavailable: this directory is not a git repository…` / `Next step Run the Gate inside a Git working tree the release owner can consult, or set release.require_clean_git = false…`. It borrows no accent, is never hidden, and never reads as Observed.
- [x] **Errors carry the four parts.** Every refusal is a typed envelope: what happened, why we know, what to do, and an operation id — e.g. `ERR-DIFF-5001` for the same build on both sides, `ERR-STORAGE-4005` for an unknown snapshot, `ERR-CONFIG-7007` for a save that would drop unknown keys. Paths are redacted by `hidden_of()` before an envelope leaves Rust.
- [x] **Dialogs only for real interruptions.** The two native dialogs (open config, save config) are the ones that must be. The acceptance form is **inline** under its own finding, so no modal steals focus and no focus trap is needed; `Cancel` discards the draft.
- [x] **Empty states give an exit.** No config: `No project policy is loaded. A run judges FirmwareSight's stated default policy, and the run below shows which values those were.` No baseline: `No build is chosen. With growth thresholds configured and no baseline, the growth rule answers Unknown and the run still happens.`
- [x] **Next step always visible.** Every finding carries Core's remediation line, and the notes block says what it does not do: `There is no editor here: FirmwareSight reads the notes file, it does not write one.`

## Do's — numbers and density

- [x] ⚙ **Mono for numbers, hashes, ids, timestamps, paths.** `Release.module.css` uses `var(--fs-font-mono)` for the run id, snapshot ids, SHAs, byte figures, the config file name and the notes path.
- [x] ⚙ **Row height ≤ 40 px.** Tables use the shared token row rhythm; the budgets, artifacts and growth tables render one figure per cell with no wrapped rows (smoke steps 14–18).
- [x] ⚙ **Spacing on the 4 px grid, radii in {6, 8, 10, pill}.** Only `--fs-space-*` and the two radius tokens are used; no literal.
- [x] ⚙ **Type ≥ 12 px.** Only `--fs-font-size-body` and `--fs-font-size-metadata`.
- [x] ⚙ **Motion within the token set.** Two transitions only (`border-color`, `background`) at `--fs-motion-duration-micro` with `--fs-motion-ease`, on control feedback — nothing decorative.
- [x] ⚙ **`prefers-reduced-motion` honoured** by the existing global rule (`styles/global.css:63-69`), which zeroes transition and animation durations.
- [x] **Old / new / delta in one column, no fabricated zero.** The growth table shows `old 256 bytes · new 376 bytes · delta +120 bytes · review threshold 100 bytes`; a run without a baseline shows `Unknown` with no number rather than `0` (smoke steps 9, 18).
- [x] **Key numbers carry context.** Each budget row states actual, limit, headroom, state and the evidence basis (`MapRegionAndElfLoad · complete attribution`) — the figure never stands alone.
- [x] **One focus per screen.** The page reads policy → build → verdict → findings → judged-on; the disposition is the single accent object.
- [x] **Amber for 12 px review text uses the strong variant** where it is small (`--fs-color-status-review-strong`).

## Do's — tables and accessibility

- [x] **No chart where a table belongs.** P3 adds no chart: three tables, ranked by nothing more than the canonical rule order.
- [x] ⚙ **Icon buttons named, tables semantic.** 3 `<table>` each with a `<caption>` and `<th scope="col">` (23 header cells); 10 `aria-label`s; every input has a `<label htmlFor>` (`fs-gate-current`, `fs-gate-baseline`, `fs-actor-*`, `fs-reason-*`).
- [x] **Tab order is reading order.** The rail, the policy row, the two pickers, Run Gate, the unit radios, then findings in rule order with each REVIEW row's `Accept review` adjacent to its own record.
- [x] **Live regions are announcements, not alarms.** Four `role="status"` surfaces (config loaded, stale run, acceptance recorded, refusal) — no alert role, no red flash for a neutral message.
- [x] **Capability banner reflects evidence, not aspiration.** The build block states `nonvolatile 376 bytes (exact) · runtime 76 bytes (exact)` because the footprint really is exact; a partial total would say so, and `evidence.unknown_review` reports `0 snapshot evidence item(s) are Unknown, below the review threshold of 1.`

## Reference and layout

Prompt §63 fixes the reading order for this page: **BLOCK → REVIEW → UNKNOWN → PASS → N/A**, which is
`GateRuleId`-independent and is what `Release.tsx` groups by; the acceptance record shows actor and
time (`By lin.we · When 2026-09-30T18:41:31Z · Accepted state REVIEW`). Bundle preview was **not**
implemented (§64): the rail lists three pages, and `intake.test.tsx` asserts Bundle, History,
Settings, SBOM, Pricing and Cloud are absent.

One density defect is carried forward unfixed: the `Memory budgets` table is wider than the content
column once `Evidence basis` is shown, so the report area scrolls horizontally (smoke steps 16–17,
38). Nothing is clipped without a scroll and no rule text is lost; fixing it needs a token-level
decision about column priority, which is why it was not patched with a magic width.
