---
title: "P0 Design Review Checklist"
doc_id: "FS-P0-018"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Design / Engineering"
last_updated: "2026-09-28"
---

# P0 Design Review Checklist

`AGENTS.md` 11 requires every UI change to pass `templates/DESIGN_CHECKLIST_TEMPLATE.md`, so this is
that checklist filled for the P0 desktop summary screen. A rule that is never applied to the thing it
governs is decoration.

Authority order where sources disagree (AGENTS.md 11): governance red line > frozen assets, ADR and
tokens > `DESIGN.md` > accepted screenshots > upstream reference.

Method: each ⚙ box below was checked against the tree, and the command that decided it is written
inline so the review can be repeated. Where a box cannot be ticked, it is left unticked with the
reason beside it. The pass produced four findings: a token gap in the design contract itself, no live
region for the screen's only asynchronous action, a `select` with two of four interaction states, and
API enum vocabulary shown to the user verbatim. Two were fixed in the source and are asserted by a
test; two are recorded open, one because closing it means editing a frozen design asset and one
because it is a boundary decision rather than a styling one.

## Don'ts（禁区 · 一票否决）

- [x] ⚙ **No gradient.** `grep -rn "gradient" apps/desktop/ui/src/` returns nothing at all - not in
      the authored sheets and not in the generated `tokens.css`. The only `linear-gradient` text in
      the repository is the prohibition itself, in
      `templates/DESIGN_CHECKLIST_TEMPLATE.md`. `DESIGN.md` 9's MUST NOT list opens with 渐变 at
      line 119.
- [x] ⚙ **No glassmorphism.** No `backdrop-filter` anywhere under `apps/desktop/ui/src`; the rule
      lives in `DESIGN.md` 9's MUST NOT list at line 120 (玻璃拟态) and in `AGENTS.md` 11, and there is
      nothing in the tree for either to catch.
- [x] ⚙ **No neon or glow.** `grep -rn "box-shadow|text-shadow|filter:|drop-shadow" src/` -> no match.
      Two authored comments say so in words: `src/App.module.css:21` ("no card, no shadow") and
      `:137` ("Errors keep their shape without decoration"). The overlay shadows the frozen asset
      defines do reach the generated sheet as real `box-shadow` values
      (`tokens.css:82-83`, `--fs-shadow-overlay-sm` and `--fs-shadow-overlay-lg`) and are referenced by
      no P0 rule - correctly, because P0 renders no floating layer, and `DESIGN.md` 9's MUST NOT names
      卡片/面板/表格行阴影 rather than shadows in general.
- [x] ⚙ **No purple-blue palette.** Zero hardcoded colors in authored sheets -
      `grep -rnoE "#[0-9a-fA-F]{6}" src/ | grep -v tokens.css` -> no match. All colour arrives as a
      `--fs-color-*` variable. In the generated sheet the only blue is the accent
      (`#2563EB`) plus `focus.ring.color`; no `#8…`/`#9…`/`#a…`/`#c…`/`#e…` value exists at all.
- [x] ⚙ **No shadow beyond the overlay pair.** See the first two boxes: no shadow is authored, so no
      panel, card or table row can carry one.
- [x] **No marketing surface.** One `header` - a title and a two-line subhead that states a limit
      ("No diff, no gate, no release action exists in this build"). No hero band, no CTA, no case
      cards, no illustration.
- [x] **State is never colour-only.** `StateBadge` renders icon + label + optional count + note
      (`src/components/StateBadge.tsx:20-28`); every icon is `aria-hidden` with `focusable={false}`,
      so the label is the whole message before colour is considered. Asserted by
      `it('labels evidence quality with a word and an icon, never colour alone')`.
- [x] **No dark theme.** No `prefers-color-scheme`, no theme attribute, no `.dark` rule under `src/`;
      `assets/design-tokens.json` defines a `light` group only. `color.dark.*` exists in the token
      file and is unused - raised as a question at the bottom rather than deleted, since the token
      asset is frozen and not P0's to trim.
- [ ] ⚙ **No invented values - one gap found, and it is in the contract, not in a component.**
      Authored CSS makes 93 `var(--fs-*)` references, and the complete set of literals with units is
      `1px` × 8 (borders), `0ms` × 2 (the reduced-motion reset) and `100%` × 1. The border width has
      no token: the file defines `focus.ring.width = 2` and nothing else that is a width.
      `DESIGN.md` 3 enumerates what must be tokenised -
      "所有色值、字号、间距、圆角、行高、动效时长、focus ring、阴影只引用 `assets/design-tokens.json`
      的语义角色" - and does not list border width, while its next line forbids inventing values
      ("不得在组件中私造 magic number") and `DESIGN.md` 4 makes hairline borders a hierarchy device:
      "靠对齐、分组、发丝边框和排版建立层级，不靠卡片阴影". The design contract therefore requires a
      measurement it never tokenised. The template's rule for that case is a token version bump, and
      a bump to `assets/design-tokens.json` changes a frozen design asset - not P0's to do. Deleting
      the borders would obey the token rule and break `DESIGN.md` 4. The eight `1px` borders ship
      with the gap recorded here, in `P0_IMPLEMENTATION_LOG.md` 19 and `P0_KNOWN_LIMITATIONS.md`.
- [x] **UI does not re-implement parser / diff / gate logic.** `src/format.ts` exports exactly three
      helpers - `formatBytes`, `formatOptional`, `truncateMiddle` - and `src/ipc/bridge.ts` exports
      the two invocations plus `toEnvelope` / `isErrorEnvelope`. No classification, no gate
      evaluation, no size arithmetic beyond byte grouping. Asserted from both sides:
      `it('shows the core facts the same command line prints')` and, in Rust,
      `desktop_and_cli_report_the_same_core_facts_for_the_same_bytes`.
- [x] **No forbidden wording.** `grep -rniE "guaranteed|fully compliant|zero risk|intelligent detection"`
      over `src/` and `index.html` -> no match. Where the UI could overclaim it underclaims:
      "Not admissible for a hard limit".

## Do's（应做 · 逐项勾选）

### 状态与证据

- [x] ⚙ **All five states present and correct.** `StateName = 'PASS' | 'REVIEW' | 'BLOCK' |
      'UNKNOWN' | 'N/A'` is a closed union and `stateClass` is an exhaustive switch with no
      `default`, so a sixth state cannot be added without failing the type check. Each arm returns
      its own glyph. `BLOCK` is reachable in P0 only as the error panel's red left border
      (`src/App.module.css:145`) - there is no gate to produce a BLOCK finding, and none is faked.
- [x] ⚙ **Interaction states complete.** The button has all four: `:hover:not(:disabled)` →
      `accent-hover`, `:active:not(:disabled)` → `accent-pressed`, `:disabled` → the disabled token
      pair, and focus via the global `:focus-visible` ring. The `select` now styles `:hover`,
      `:disabled` (text and surface tokens) and focus, and is disabled while a request is in flight;
      its `:active` state is **N/A**, because a native select's press is the operating system drawing
      its own popup, which no page style can represent - so it is marked N/A rather than invented.
      This box was open when the checklist was first filled: the selector had two of four states and
      stayed usable mid-request, which let a selection change under an in-flight analysis. Both were
      fixed in `App.tsx` / `App.module.css` and are asserted by
      `it('announces the in-flight state and locks both controls until it resolves')`.
- [x] ⚙ **Focus ring not removed or weakened.** `src/styles/global.css:51` is
      `:focus-visible { outline: var(--fs-focus-ring-width) solid var(--fs-focus-ring-color);
      outline-offset: var(--fs-focus-ring-offset); }` - 2px accent, 2px offset, all three values from
      tokens. Nothing in the tree sets `outline: none`.
- [x] **Unknown presented correctly.** Neutral grey (`--fs-color-status-unknown`), hollow dashed glyph
      (`fill="none"`, `strokeDasharray="2 1.6"`), a factual sentence and a next step:
      "Weakest basis … is name- or flag-derived. Supply the linker MAP to strengthen it." Accent blue
      is never reused for Unknown. Unknown is never hidden: `entryPointUnknownReason` and
      `buildIdUnknownReason` render as notes under their values, and Git shows "unknown" as a state
      rather than an error (`it('treats Git as unknown rather than as a failure')`).
- [x] **Error carries its four parts.** `ErrorPanel` is a `role="alert"` region listing What happened,
      Code, Why we know, What to do and Diagnostics ID, with code / details / id in mono. Asserted by
      `it('presents a typed error with what happened, code, what to do and a diagnostics id')`. The
      "what to do" line comes from `ArtifactError::user_message()` in Rust, so CLI and Desktop give
      the same remedy for the same failure.
- [x] **Dialog only for real interruption; toast only for fleeting confirmation.** P0 opens neither.
      There is no `window.confirm`, no `<dialog>`, no modal.
- [x] **Empty state has an exit.** "Nothing has been analyzed in this session yet. Choose a fixture
      and run Analyze." - an instruction, not a grey panel.
- [x] **Next step always visible.** Each Unknown and Partial carries its remedy, and the strings come
      from `apps/desktop/src-tauri/src/service.rs`, so a next step cannot drift from the fact that
      produced it. The UI invents none of them.
- [x] ⚙ **Live region for asynchronous state.** The in-flight line
      (`src/App.tsx:134-141`) is `role="status" aria-live="polite"`, so a screen reader hears the
      request start and, when the element unmounts on completion, that it ended. The error panel is
      separately live through `role="alert"`. This box was the finding that mattered most in this
      review: the layout made "Analyzing…" obvious to a sighted user and invisible to everyone else,
      and nothing in the type system, the parity tests or the gate could see it. Asserted by
      `it('announces the in-flight state and locks both controls until it resolves')`, which holds the
      IPC promise open, reads the status role and both controls' `disabled`, then resolves it and
      requires the status region to be gone.
- [ ] **Terminology not shown raw.** Capability badges render Core's serialized enum words verbatim:
      "supported", "available", "not-provided", "unavailable", "unknown"
      (`crates/firmwaresight-core/src/domain/capability.rs`), and `layoutSource` renders as "map".
      Left unticked because it is a genuine tension rather than a clean violation: `AGENTS.md` 11 and
      the parity tests forbid the UI from deriving or renaming facts, and the CLI human reader prints
      the identical strings - `fwsight analyze fixtures/elf/p0-basic/firmware.elf` prints
      "MAP not-provided" and "Git unknown" exactly as the desktop badge does. A product-copy layer
      would look better and would be the first place CLI and Desktop could disagree. The resolution is
      a boundary decision (copy owned by Core, by the shell projection, or by the view), so it is
      recorded here as an open question instead of being dressed up as compliant.

### 数字与密度

- [x] ⚙ **Mono numerals.** `sha256`, parser id, build id, entry point, byte counts, section and symbol
      counts, error codes and operation ids all render through `mono` / `monoSmall`, which resolve to
      `--fs-font-mono` with `font-variant-numeric: tabular-nums` (`src/styles/global.css:57-61`).
      `SHA-256` and the byte figures are asserted as rendered text in `App.test.tsx`.
- [x] ⚙ **Row height within limits.** The rule's numbers exist as tokens -
      `table.row.compact = 28`, `default = 36`, `max = 40` - and P0 authors no height at all: every
      row is one line of `--fs-font-size-metadata` (12px) inside
      `--fs-line-height-metadata` (18px) plus token padding, which cannot exceed 40px. Asserted as
      "no authored row height" rather than "measured 34px", because the jsdom render is not a real
      display - see the last section.
- [x] ⚙ **Spacing on the 4px grid; radii within the set.** 23 spacing and 3 radius references, all
      `var(--fs-space-*)` / `var(--fs-radius-*)`. The tokens used are space 1, 2, 3, 4, 6, 8 →
      4 / 8 / 12 / 16 / 24 / 32px, and radius `control` (6) and `panel` (8) - two of the four legal
      values. The only non-token length in the layout is the `1px` border gap above.
- [x] ⚙ **Type ≥ 12px.** The generated sheet emits 12 / 13 / 15 / 20px
      (`metadata`, `body`, `section`, `page`) and those four are exactly what the UI references.
      `DESIGN.md` 3's "不得小于 12px" holds by construction: no other size exists to reach.
- [x] ⚙ **Motion inside the token set and inside its permitted uses.** One authored transition:
      `src/App.module.css:61`, `transition: background
      var(--fs-motion-duration-micro) var(--fs-motion-ease)` - 120ms,
      `cubic-bezier(0, 0, 0.2, 1)` (ease-out), on a control's background as it is pressed or
      disabled. That is state feedback, one of the four allowed uses. No animation, no easing curve,
      no other duration appears.
- [x] ⚙ **`prefers-reduced-motion` respected.** `src/styles/global.css:63-69` sets
      `transition-duration: 0ms; animation-duration: 0ms` for `*` and both pseudo-elements inside the
      media query. `0ms` is not a token value; neutralising motion entirely is the requirement, and no
      token encodes "none".
- [x] **Diff row: old / new / delta.** **N/A by scope.** P0 has no compare surface, so no added or
      removed row exists to get wrong. Recorded as N/A rather than checked; the six
      `--fs-color-diff-*` variables that `tokens.css:26-31` emits are referenced by no authored rule
      for exactly that reason.
- [x] **Key numbers carry context.** Each budget row shows the state word, the byte figure beside it
      (`BudgetRow` deliberately keeps the number outside the badge: a bare number would be a number
      with no evidence label), the count of unattributed sections when the basis is partial, and the
      accounting rule and layout source as notes beside them.
      **Partial**: "largest contributor" is absent, because the P0 payload carries section counts and
      a dual-accounted list rather than a ranked breakdown, and ranking it in the view would be the UI
      deriving a Core fact.
- [x] **One focus per screen; no KPI wall.** A fixture selector, one pressable primary action
      (Analyze), then read-only sections. `DESIGN.md` 5's "one Primary per region" holds - there is
      one Primary in the document.
- [x] **Review amber uses the deepened variant at small sizes.** `StateBadge.module.css:21` and `:25`
      both resolve to `--fs-color-status-review-strong`; the lighter `status.review` tone is never
      applied to 12px text.

### 图表与可访问性

- [x] **Chart priority order.** **N/A**: P0 renders no chart. `grep -rnoE "<canvas|role=\"img\"" src/`
      -> no match; the only `<svg>` elements are the five badge glyphs.
- [x] **Chart anti-patterns zero.** Same reason - no 3D, no donut, no sparkline, no rainbow ramp. The
      rule becomes testable when the first chart arrives, and `DESIGN.md` 7 already fixes the
      colour rule (单色蓝阶).
- [x] ⚙ **Icon buttons named; tables semantically headed.** There is no icon-only control - the single
      button's accessible name is its text, "Analyze". There is no `<table>`: the layout is term/value
      rows, so no header association can be missing. Badge icons are `aria-hidden` because their label
      sits beside them. The `select` is named by wrapping `<label>`
      (`src/App.tsx:102-121`), which is an implicit association and does not need an `id`.
- [x] **Tab order = visual order; dialog trap; Esc.** DOM order is paint order (header, selector,
      button, then report) and `grep -rn "tabIndex" src/` -> no match, so nothing is reordered.
      **N/A** for the trap and Esc rules: there is no dialog, popover or overlay to trap or dismiss.
- [x] **The capability banner tells the truth.** Seven rows - ELF, Sections, Symbols, Debug info, MAP,
      Object attribution, Git - each from `capabilities.*` as Core produced it
      (`build_capabilities` in `crates/firmwaresight-artifact/src/pipeline.rs`). They stay negative
      where the evidence is negative: on the committed p0-basic golden the banner reads
      `map: not-provided`, `objectAttribution: unavailable`, `git: unknown` while `elf: supported` and
      `sections: available`, and `capabilityState()` maps a string to a badge and nothing else, so the
      UI can promote nothing. The hard assertion lives in Rust:
      `the_desktop_projection_keeps_the_same_evidence_claims_as_the_cli`.

## Score

Don'ts: 10 of 11 boxes ticked. The one open box is the border-width token gap - closing it means
changing a frozen design asset, which P0 is not authorized to do. Nothing in the list is *violated* in
the sense the template means by 一票否决: every value in the layout is token-derived except a border
width the design contract requires and never tokenised.

Do's: 24 of 25 boxes ticked, two of those being N/A by scope (diff rows, charts) and recorded as N/A
rather than quietly passed. Two findings this review produced were fixed rather than filed: the missing
live region and the selector's incomplete interaction states, both asserted by
`it('announces the in-flight state and locks both controls until it resolves')`. One stays open -
capability badges showing Core's enum words - because the fix is a boundary decision about who owns
user-facing wording, and picking a side without review is exactly the kind of change this checklist
exists to catch rather than hide.

The open item is therefore a question for the architecture owner, not a task: a copy layer in the view
would look better and would be the first place the CLI and the Desktop could start disagreeing about
the same fact, which `AGENTS.md` 11 and the parity tests forbid.

## What this review cannot claim

One pass by the person who wrote the code, on one machine, against the authored style sheets and the
markup as rendered under jsdom. It is not a screen-reader audit, not a keyboard walk-through in a real
WebView - the window has never been opened, which `P0_KNOWN_LIMITATIONS.md` records - and not a review
by someone other than the implementer. The template allows manual review at this stage and asks for the
⚙ items to be scripted before G2; each command above is written so it can become a CI step as-is.

Two questions this pass raises for the design owner rather than engineering:

1. Do the hollow Unknown glyphs still read as Unknown at 12px on a real display, or does a dashed 12px
   circle just look like noise? Only a launched window can say.
2. `color.dark.*` and the `shadow.overlay-*` pair are defined in the frozen token file and referenced
   by nothing. Is the dark group reserved for the post-MVP theme, or is dead weight acceptable in a
   frozen asset?
