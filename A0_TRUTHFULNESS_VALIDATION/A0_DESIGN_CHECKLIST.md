---
title: "A0 Design Checklist"
doc_id: "FS-A0-DESIGN-001"
product: "FirmwareSight"
version: "0.6.0"
status: "REPORT"
owner: "Design"
last_updated: "2026-10-09"
---

# A0 — design checklist for the user-visible surfaces the corrective touched

`AGENTS.md` 11 binds every UI change PR to `templates/DESIGN_CHECKLIST_TEMPLATE.md`, and A0-03 changed the text of
one user-visible element while A0-01 changed the *value* one existing element reports. This file answers the template
for exactly that scope. **Nothing here is a re-drawing**: no component, style rule, token, colour, layout, motion or
state was added, removed or altered by this round, so the frozen assets and the token file are untouched
(`assets/design-tokens.json` stays `94336906…14d4`, and no `*.module.css` path is in the diff).

## What actually changed on screen

| Surface | Change | Kind |
| --- | --- | --- |
| Compare → "Top growth and largest additions" hint (`Compare.tsx:950-956`) | one sentence appended to the existing `<p class="hint">` | prose only |
| Analyze → "Object/module attribution" row (`Analyze.tsx:511`) | the same `CapabilityRow`, now fed `unavailable` instead of `available` when a MAP is supplied | value only |
| Release Bundle → §9 limits table (`release_render.rs:766-805`) | a row that was already implemented became reachable | value only |
| ErrorPanel → "What to do" for `ERR-FORMAT-0001` | one sentence replaced by another, same element | prose only |

## Don'ts (一票否决) — none hit

- ⚙ Gradient / glass / neon / glow / purple-blue / shadow: nothing was added. The sentence inherits `styles['hint']`,
  which this round did not touch, and no style rule appears in the diff.
- No marketing surface: no hero, banner, CTA, case card or illustration.
- **State is never colour-alone**: the Analyze row's `unavailable` maps through `capabilityState()`
  (`stateWords.ts:46-61`) to `N/A`, which `StateBadge` renders as an icon **plus** the word — so the corrected value
  arrives with the same icon + label treatment as every other state, not as a colour change.
- No dark theme. No invented values: the addition is text, so there is no number, size, radius, colour or duration to
  come from a token, and no token gap was created.
- **UI does not re-implement parser / diff / gate logic.** The new sentence names two facts the shell already
  delivered — `summary.nonvolatile` (Core's diff) and the ranked rows' runtime-size deltas (Core's diff) — and asserts
  nothing about either. It performs no arithmetic, which is why §五 forbade a "sum reconciliation": computing a
  reconciliation here would have been the UI inventing a Core fact.
- Banned phrasing: the sentence claims no guarantee, no completeness and no intelligence. It states a limit and its
  reason, which is the register `DESIGN.md` asks for.

## Do's — the items this round answers

- **关键数字带上下文（相对谁 / 占比 / 最大贡献者 / 是否超预算，至少其一）** — this is the item A0-03 implements. Before
  it, each ranked list named its own basis but the reader had to guess how it related to the headline; now the hint
  says the headline is the whole image's change and the lists are the top few rows by runtime size.
- **能力 banner 如实反映证据状态，未夸大** — A0-01 is this item, at the source rather than at the paint: the
  declaration itself stopped overstating, so the capability row and the bundle's §9 table both tell the truth instead
  of one of them being edited to look modest.
- **错误四要素齐备（What happened / Why we know / What to do / Diagnostics ID）** — the envelope is unchanged and the
  fourth element (`ERR-FORMAT-0001` and its diagnostics id) is untouched; what changed is that "What to do" now names
  an action the product accepts. The desktop test that pins the exact string
  (`real_artifact_intake.rs:599-605`) is what keeps the four elements coming from one source.
- **状态五态齐备** — five states and their mapping were not modified; `overview.test.tsx`'s new tests assert the
  `PASS` / `N/A` rows stay out of the open-findings list, which is a presentation rule about states, not a new state.
- **推荐下一步始终可见** — Compare's hint sits above both lists and both tables; the refusal's next step sits in the
  error panel's existing "What to do" line.
- **mono 数字 / 行高 / 字号 / 间距 / 动效 / reduced-motion** — no value in any of these categories was introduced or
  changed; the quoted label in the new sentence is a UI term, not a number, so it stays in the prose face like the
  sentence around it rather than being mono-set.
- **表格语义化表头 / accessible name** — untouched. The ranking region keeps its `aria-label`
  ("Top growth and largest additions"), which is the handle the new component test queries by.

## What this checklist does not cover, and why

Pixel placement and reading order. `AGENTS.md` 11's conflict order puts accepted screenshots below the frozen assets and
the token file, and A0's authorization (§五) forbade re-flowing the page; the sentence was placed inside the element
that already carried the neighbouring explanation. Whether the resulting line length reads well at 1024 and 1440 is a
question for an installed capture, and this round installed nothing — see `NOT_RUNTIME_VERIFIED` in
`A0_CORRECTIVE_REPORT.md` §8. The UI-logic boundary is separately proven by tests, not by this file: `pnpm lint`,
`pnpm typecheck` and the 295-test UI suite are green at this head, and the Compare file's 58 tests pass with the new
disclosure asserted inside the ranking region.
