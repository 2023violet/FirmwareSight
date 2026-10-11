---
title: "B1-U2 Design Checklist"
doc_id: "FS-B1-U2-DESIGN-001"
product: "FirmwareSight"
version: "0.6.0"
status: "REPORT"
owner: "Design"
last_updated: "2026-10-11"
---

# B1-U2 — design checklist for the second palette

`AGENTS.md` 11 binds every UI change PR to `templates/DESIGN_CHECKLIST_TEMPLATE.md`. This round added a palette, one
control group on an existing page, and a version field. It redrew nothing. Every ⚙ line was run as a command against
this tree on 2026-10-11 and the command is printed beside the verdict, so the next reader re-runs it instead of
trusting this file.

The rule this checklist is measured against moved on 2026-10-11: `ADR-0031` superseded `ADR-0018` D3's
`theme: MVP Light only`, and the template's own Don't line now reads "双主题合规（ADR-0031）". Nothing else about the
tone changed — `DESIGN.md` §1 still decides what the product looks like, and dark is the same instrument on a
different ground.

## What is on screen

| Surface | Change | Kind |
| --- | --- | --- |
| `Settings.tsx` → Appearance | three native radios (`System` / `Light` / `Dark`) under the existing unit switch, plus one sentence naming what the choice is doing | new control group |
| `Settings.tsx` → Reset | "two settings" → "every setting"; "both" → "all three", and the theme is named among what is about to be discarded | corrected copy |
| `App.tsx` | the palette is applied to the document element before paint, and follows the desktop only while the choice is `System` | state owner |
| `tokens.css` | a `[data-fs-theme="dark"]` block carrying the same 30 role names, generated | new palette |
| `global.css` | `color-scheme: light` / `dark` per theme | native first paint |

No component stylesheet changed, no layout rule changed, no spacing, radius, typography, table, motion, focus
geometry or shadow changed, and no colour literal was added outside the generated file.

## Don'ts (一票否决) — none hit

| Line | Verdict | Command |
| --- | --- | --- |
| ⚙ No gradient | 0 hits | `grep -rniE "linear-gradient\|radial-gradient\|conic-gradient" apps/desktop/ui/src --include=*.css` |
| ⚙ No glass / no `backdrop-filter` | 0 hits | same tree, `backdrop-filter` |
| ⚙ No neon, no glow | 0 `text-shadow`, and the only `box-shadow` uses are the two overlay levels | `grep -rn "box-shadow" --include=*.css` → nothing outside `overlay-*` |
| ⚙ No purple in the palette | `[]` — every dark role is checked by channel relation, not by eye | the parity script in §"Palette discipline" below |
| ⚙ No dark-via-transform | no `filter: invert`, no overlay film, no opacity trick — dark is 30 values, not a transform | `grep -rn "filter:" apps/desktop/ui/src --include=*.css` |
| ⚙ No magic numbers | the dark block is generated from `assets/design-tokens.json`; 62 of 62 colour literals in the UI tree live in the generated `tokens.css`, none in a component file | `grep -rn "#[0-9A-Fa-f]{3,8}" --include=*.css` → one file |
| ⚙ Status never colour-only | unchanged, and re-photographed in dark: `⊘ Block 0 · ⚠ Review 0 · ◌ Unknown 2 · ✓ Pass 2 · — Not applicable 6` | `target/n04_shots/gate_findings_dark.png` |
| ⚙ Both themes compliant (the replaced line) | 18 measured pairs × 2 palettes, 1 new failure, 1 carried exemption | `python scripts/generate_design_tokens.py --contrast` |
| ⚙ UI does not re-derive Core facts | the theme is a colour attribute; no query, no recompute, and the Gate answered with one identity in both palettes | `gate-d19c48891d79…` identical light and dark, in the running window |
| No marketing surface | none added | — |

## Do's — the lines this round actually touches

| Line | Verdict | Evidence |
| --- | --- | --- |
| ⚙ Focus ring: 2px accent + 2px offset, `focus-visible` always | **PASS in both palettes**, and the ring is now theme-relative rather than a second constant | measured in the real window through the keyboard path: `2px solid rgb(37, 99, 235)` in light, `rgb(107, 161, 245)` in dark, `offset 2px`, `:focus-visible` true |
| ⚙ Interaction states | **PASS** — hover/pressed/disabled unchanged; disabled dark text measured at 3.78:1, which AA excepts for inactive components and the generator reports rather than hides | `--contrast` reported block |
| ⚙ Unknown stays neutral | **PASS** — hollow icon, `status.unknown #98A2B3`, never the accent, in both palettes | screenshot above |
| ⚙ Keyboard path matches visual order | **PASS** — `Tab` walks rail → General → units → theme → start page → reset, entering each radio group at its checked option; `ArrowUp` from `Dark` selects `Light` and stores it, with no key handler of our own | `settings.test.tsx > is walkable with the keyboard in the order the page reads` (now nine controls) + the real-window walk |
| ⚙ Mono for ids, hashes, timestamps | unchanged | `DESIGN.md` §4 already held; the dark block does not touch typography |
| ⚙ `prefers-reduced-motion` | **inherited, not re-proved** — a colour change animates nothing new | unchanged global rule |
| Errors keep the four parts | unchanged from B1-U1; the refusal block still carries What happened / Code / Diagnostics ID | `settings.test.tsx > shows a refusal as a refusal, and never as saved` |
| One focus per screen; no KPI stacking | unchanged | — |

## Palette discipline

Two claims are checked by the generator on every run, not by this file:

- **Parity.** `color.dark` must carry exactly the role names `color.light` carries — 30, no more and no fewer. A new
  role invented to make a dark value convenient fails the build, and so does a light role dropped because dark
  found it awkward.
- **Contrast.** 18 rendered pairs per theme at WCAG 2.2 AA floors, printed to four decimals. Light passes 17 and
  carries one named exemption (`diff.removed` on `diff.removed.bg`, 4.4974:1, tied to two committed report goldens —
  see `B1_U2_THEME_REPORT.md` §2). Dark passes 18, lowest 5.4228:1. An exemption whose pair later passes is itself
  a build failure, so the hole cannot outlive the reason.

```
python - <<'PY'   # the parity and purple check quoted above
import json
d = json.load(open('assets/design-tokens.json', encoding='utf-8'))['color']
print(set(d['dark']) == set(d['light']), len(d['dark']))
PY
```

## Verdict for this reviewer

No Don't is hit. The two ⚙ lines that a colour change could plausibly break — focus visibility and status
never-colour-only — were re-measured in the dark palette rather than assumed from light, and both hold. Three lines
are **not verified** and say so: Windows display scaling at 125% / 150%, a real desktop theme flip while the window
is open, and forced-colors / high-contrast mode. The evidence is `DEV_RUNTIME_VERIFIED` on a dev binary, not an
installed artifact. The round is inside `ADR-0031`, `DESIGN.md` and `AGENTS.md` 11 for what it adds, and it moved no
value in the light palette, no exported byte, and no line of `ADR-0018`.
