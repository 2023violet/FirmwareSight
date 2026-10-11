---
title: "B1-U1 Design Checklist"
doc_id: "FS-B1-U1-DESIGN-001"
product: "FirmwareSight"
version: "0.6.0"
status: "REPORT"
owner: "Design"
last_updated: "2026-10-11"
---

# B1-U1 — design checklist for the Settings surface

`AGENTS.md` 11 binds every UI change PR to `templates/DESIGN_CHECKLIST_TEMPLATE.md`. This round added one page
(`Settings.tsx` + `Settings.module.css`), one rail entry, and one state owner (`App.tsx`); it did not redraw
anything that already existed. Every ⚙ line below was run as a command against this tree on 2026-10-10, and the
command is printed beside the verdict so the next reader can re-run it rather than trust this file.

## What is on screen

| Surface | Change | Kind |
| --- | --- | --- |
| Rail → Help and about | `Help`, then `Settings` — two native buttons, `aria-current="page"` on the active one | new entry |
| `Settings.tsx` | four sections: General (storage sentence + recovery sentence + local-first + Help access), Appearance (`SizeUnitSwitch`, reused), Start page (two radios), Reset (asked in place) | new page |
| `App.tsx` | the size unit is now stored, so a choice on any page is what every other page shows | state owner |

Nothing else moved. No existing component, style rule, token, colour, layout or state was altered:
`assets/design-tokens.json` is untouched and no pre-existing `*.module.css` is in the diff.

## Don'ts (一票否决) — none hit

- ⚙ No gradient / glass / neon / glow / purple-blue / shadow.
  `grep -Ec -- "gradient|backdrop-filter|box-shadow|outline: ?none|#[0-9a-fA-F]{3,8}|[0-9]px"` over
  `Settings.module.css` and `Settings.tsx` → **0 matches for every pattern**, both files.
- No marketing surface: no hero, no CTA band, no case card, no illustration. The page is one measured column of
  sections in Help's rhythm.
- ⚙ Status is never colour-only, and this page states no status at all: there is no PASS/REVIEW/BLOCK badge on it,
  because a settings page that reports a verdict would be inventing one. Refusals arrive through `ErrorPanel`, which
  carries icon + label + code + what-to-do (`AGENTS.md` 11's five-state rule is satisfied by not claiming a state).
- ⚙ No Dark theme. `theme` in `BASELINE.yaml` stays `light_only_mvp`; the page renders no theme control at all, and a
  test asserts the words `Theme`, `Dark`, `Light`, `Language`, `Font` and `Density` never appear on it
  (`settings.test.tsx > offers no control for a setting this build does not have`).
- ⚙ No invented values. Every length, colour, radius, border and motion in the new CSS is a `var(--fs-*)` reference,
  and each name was checked against `apps/desktop/ui/src/styles/tokens.css`:

  ```
  grep -o "var(--fs-[a-z0-9-]*)" Settings.module.css | sort -u   # then: is each name present in tokens.css
  ```

  Two tokens this round first reached for and which do **not** exist — `--fs-space-5` and `--fs-font-weight-strong` —
  were found by that check and replaced (`--fs-space-6`, and the keyword `bold`, which is what `global.css:42`
  already does for `h1` because the frozen token file defines no weight scale). No value was created in place of a
  missing token.
- UI re-implements no parser/diff/gate logic. The page never computes a size: it sends two words to the shell and
  renders what came back. `format.ts` remains the only place a unit is applied, and it was not modified.
- No forbidden register: the page says "Local first: … no account, no upload and no service that receives anything",
  which is the capability surface this build actually has (`core:default`, no network plugin, no updater) and repeats
  the sentence Help already carries. It claims no "zero risk", "fully compliant" or "guaranteed".

## Do's — item by item

### 状态与证据

- ⚙ Five states: **N/A by design, and stated** — the page presents no finding, so it shows no state badge. The five
  states it could report are Gate facts, and a preference is not one.
- ⚙ Interaction states: the new `.control` rule defines `hover`, `disabled` and inherits the global `focus-visible`;
  the radios are native inputs, whose states the browser owns. Copied verbatim from `Help.module.css:126-149`, which
  is the approved treatment for a page whose only action is neutral.
- ⚙ Focus ring: not removed or weakened anywhere — the grep for `outline: none` returns 0, and
  `global.css:51-54` (`--fs-focus-ring-width` + `--fs-focus-ring-color`, offset) applies to the new controls.
- Unknown presentation: **N/A** — nothing on this page is Unknown. The four *recovery* sentences (`missing`,
  `corrupted`, `futureVersion`, `nowhereToStore`) are factual statements in secondary text, not a reused status colour.
- ⚙ Error four elements: a failed save renders through `ErrorPanel` — what happened, the code (`ERR-PREFS-8002`),
  why we know (the operating-system sentence, with directories reduced out by `startup::without_directories`), and
  what to do (`remediation()`), and it stays where it appeared. Asserted by
  `settings.test.tsx > shows a refusal as a refusal, and never as saved`, which also fails if the word "saved" appears.
- Dialogs: none. Reset asks its question *in place*, in the same inline-confirm shape Release uses for a replace
  (`Release.tsx:1439-1460`), so the page never hides the section it is standing in.
- ⚙ Empty state: a first start is the empty state, and it is answered — the page names the defaults as defaults, and
  the General section states where a document will be written once one exists.
- Next step always visible: each refusal's `remediation()` is the next step; the start-page section says in its own
  sentence that the choice applies at the next start rather than pretending it moved the reader there.

### 数字与密度

- ⚙ Mono for identifiers: the one machine string on the page, `settings.json`, renders in `--fs-font-mono`
  (`settings.test.tsx` also asserts no `/` or `\` anywhere in the page's text — a name, never a location).
- ⚙ Row height, spacing on the 4px grid, radius ∈ {6, 8, 10}: inherited from the token values referenced
  (`--fs-space-*` are 4/8/12/16/24/32/48/64; `--fs-radius-control` is the frozen control radius). No table row exists
  on this page, so the ≤40px row rule is **N/A**.
- ⚙ Font size ≥12px: only `--fs-font-size-section` and `--fs-font-size-metadata` are used, both token values.
- ⚙ Motion: one transition, `border-color` over `--fs-motion-duration-micro` = **120ms** with `--fs-motion-ease`,
  on a hover of the neutral control. That is inside the allowed {120, 150, 180}ms set and it is a control affordance,
  not a panel animation.
- ⚙ `prefers-reduced-motion`: respected globally at `global.css:63`; the new rule declares no animation that could
  escape it. Not re-verified per-component in this round — recorded as a limit of this checklist, not as a pass of a
  new test.
- Diff row three columns: **N/A** — no diff is shown here.
- Numbers with context: **N/A** — the page shows no measurement.
- One focus per screen: the page has no accent-filled primary button. Its two verbs (Open Help and About, Reset
  application preferences) are neutral controls, matching Help's single-action treatment.
- Review amber at 12px: **N/A** — no amber appears.

### 图表与可访问性

- Chart rules: **N/A** — no chart, no bar, no treemap.
- ⚙ Icon buttons carry accessible names: **N/A** — every button on the page is a text button. The rail entry carries
  `aria-label="Settings page"` and `aria-current="page"`, which is the existing convention (`App.tsx:99-100`).
- ⚙ Semantic table headers: **N/A** — no table.
- ⚙ Tab order = visual order: asserted in `settings.test.tsx > is walkable with the keyboard in the order the page
  reads`, which walks `main button, main input` in DOM order and requires the sequence
  `Open Help and About → Bytes → KiB → Analyze → Bundle & History → Reset application preferences`, all native
  elements, none a `div role="button"`. The reset confirmation moves focus to the decision it just asked for
  (`autoFocus`, asserted by the reset test's `document.activeElement` check).
  **Not verified on the real window**: the focus *ring's* rendering, Esc-to-dismiss (not implemented — the confirm is
  answered by its two buttons), and layout at 1024×760/200% zoom. The OS-level automation path available on this host
  timed out on every capture this round, so those three stay open rather than being claimed from jsdom.
- Capability banner truthfulness: the page carries no ✓/· capability strip. It points at Help for the version and the
  diagnostics rather than restating them, so two surfaces cannot disagree about one fact (`support::product_facts`).

## Verdict for this reviewer

No Don't is hit, and every ⚙ line that can be checked on this host was checked by a command printed above. Two
lines are **N/A** because the page shows no state, no table and no chart; one line (reduced-motion) is inherited
rather than newly proven; and the real-window keyboard/zoom/layout walkthrough is **NOT VERIFIED** with the reason
stated. The page is therefore inside `ADR-0018` and `AGENTS.md` 11 for what it adds, and adds nothing to what was
already frozen.

## 修正案：本清单的 No Dark theme 行已被 ADR-0031 定向取代（2026-10-11，B1-U2）

上方 "⚙ No Dark theme" 一行写于 2026-10-11 凌晨，当时它是正确的：`ADR-0018` D3 仍规定 MVP 只有浅色，所以 B1-U1 的
页面一个主题控件都不渲染，并用一条测试断言 `Theme`/`Dark`/`Light` 三个词不出现。

`ADR-0031`（2026-10-11）只取代 D3 的那一句 `theme: MVP Light only`，取代范围是 Developer Preview 的桌面外壳。因此：

- 该测试中禁止的三个主题词由 B1-U2 收窄为仍然不存在的设置（语言、字号、密度），而不是删除整条测试。断言"没有控件
  通向没有功能的开关"这一条规则不变。
- 本清单其余结论（token 唯一数值真源、无阴影/渐变、状态非纯颜色、Unknown 中性）在两种主题下继续生效，B1-U2 需要
  在暗色下重新证明一遍，而不是继承。
- 导出面不变：Release Bundle HTML、portable report、CLI JSON 仍为浅色，主题不进入任何字节产物、hash、runId 或判定。

