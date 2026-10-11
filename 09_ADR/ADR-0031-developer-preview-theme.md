---
title: "ADR-0031 Developer Preview Theme"
doc_id: "ADR-0031"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-11"
---

# ADR-0031 — System / Light / Dark Ships in the Developer Preview, and the Preference Document Takes a Version

## Status

**Accepted as a design decision. It supersedes exactly one clause of `ADR-0018` and nothing else.**

The clause is inside `ADR-0018` **D3 — FirmwareSight overrides**:

> theme: MVP Light only

Everything else in `ADR-0018` stands unedited: the single upstream temperament anchor (D1), the authority order
(D2), the five-state set (D4), the token file as numeric authority (D5) and the governance artifacts (D6). No
historical sentence of `ADR-0018` is rewritten here, and its own text still says what it said on 2026-09-27; this
record states which of its conclusions no longer governs, and why.

`ADR-0018`'s Revisit trigger anticipated exactly this event — "只有品牌重定位、**MVP Dark theme**、核心 UI 框架变化，
或真实用户研究证明当前密度/可访问性失败时重新评估" — so this is the documented door being used, not a rule being
overrun.

### The Owner's authorization

The Owner decided this on 2026-10-11 in 《FirmwareSight｜架构师接手判定｜2026-10-11》, archived verbatim at
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_B1U1_Review_Decision_And_Next_Prompts_2026-10-11.txt` (1,786 bytes, 17 lines,
0 `CR`, sha256 `52118f91c02d071263615867afee7f1ccda4a6440e39ea1feab342da1f81adb6`):

> 然后 Owner 明确允许以新增、小范围 superseding ADR 覆盖 ADR-0018 的 MVP Light only 约束，授权 NEXT-04 实施
> System/Light/Dark。

The execution prompt that follows this decision,
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_NEXT_04_B1U2_System_Light_Dark_Theme_2026-10-11.txt` (8,193 bytes, 43 lines,
0 `CR`, sha256 `122469aa78adca3b24d1ae8101354ee797fd683f3c0e7772da818e937431dd92`), asks for the supersession to be
taken against "ADR-0018 D4". **That citation is wrong and is corrected here rather than repeated there**: D4 is the
five-state set `PASS / REVIEW / BLOCK / UNKNOWN / N/A`, which this change does not touch. The theme rule lives in D3.
A supersession that named D4 would have left the actual prohibition in place while appearing to remove it.

## Context

Two things made the light-only rule expensive to keep, and one made it newly cheap to lift.

1. **The rule was written against a product with no place to put the choice.** `ADR-0018` froze at v0.4.0, before any
   application preference existed. B1-U1 (closed 2026-10-11, `2c47213`…`2cff702`) built that mechanism and proved it
   end to end in a real window: one document, one read path, one write path, a recovery contract for damaged and
   newer bytes, and a Settings page whose controls all do something. A theme is now a two-line addition to a
   mechanism that already works, not a new surface to invent.
2. **Readability is the stated product temperament, and light-only fails part of it.** `DESIGN.md` §1 fixes the tone
   as "Instrument-grade clarity — engineered, not stylized" and §4 makes the table the primary working surface.
   FirmwareSight is used next to a bench, in a rack aisle and at 2 a.m. during a release freeze. A dense table of
   hex addresses on `#F7F8FA` is the hardest version of that reading task, and the product has no answer for it.
3. **The alternative to lifting the rule quietly was a dead control.** The v0.2 planning document lists `theme`
   among the B-group application preferences. Leaving the rule intact while shipping a Settings page invites a
   disabled or cosmetic switch, which B1-U1 explicitly refused to do
   (`B1_VALIDATION/B1_U1_PREFERENCES_SETTINGS_REPORT.md` §9: "a control appears only when it does something").

## Decision

### D1 — Three values, one default, in the store that already exists

`system | light | dark`, default `system`, stored as one field of the existing `ApplicationPreferences`. No second
preferences mechanism, no per-window theme, no per-page override, no theme editor. The word set is closed and
matched at the command boundary exactly the way `size_unit` and `startup_page` are, so a value the page does not
offer is refused with an `ERR-PREFS` code instead of reaching the document.

### D2 — The document is versioned 1 → 2, and the reader is widened before the writer changes

This is the part that cannot be improvised. `preferences.rs` currently refuses any document whose
`schema_version` is not exactly `SCHEMA_VERSION`:

```rust
Some(seen) if seen > SCHEMA_VERSION => Err(Unreadable::Future { seen }),
// A version below this build's own is not a document this build was given a contract for.
Some(seen) if seen != SCHEMA_VERSION => Err(Unreadable::Damaged),
```

Raising `SCHEMA_VERSION` alone would make every real user's existing file read as **Damaged** on the first launch
after the upgrade: defaults, a "could not be read" sentence, and — on the next save — the person's own working
preferences demoted to `settings.json.unreadable`. Non-destructive, but a data-loss experience wearing a recovery
costume. So the contract is fixed in this order:

- **Accept both.** The v2 reader accepts `schema_version` 1 and 2. A v1 document is read as a complete v1 document;
  a missing `theme` resolves to `system`. Nothing is rewritten during a read, so opening the app never mutates the
  file — the invariant B1-U1 proved twice per cold start.
- **Write v2 only when something is stored.** The first save after an upgrade produces `schema_version: 2` with all
  three fields present. Until then the file stays exactly as the previous build left it.
- **The old build already behaves.** A v1 build meeting a v2 document takes the `Future` path: defaults, the
  "written by a newer version" sentence, and a save refused with `ERR-PREFS-8004` that leaves the newer bytes
  untouched. That path is implemented and was driven in a real window during `NEXT-03`, so downgrading is safe
  without new code.
- **Anything above 2 stays future.** `seen > SCHEMA_VERSION` keeps meaning "do not touch", which is what protects a
  v3 document from a v2 build.

### D3 — The document key is flat: `theme`

The v0.2 planning document names preferences by group, so the field is written `appearance.theme` there and
`display.size_unit` for the unit. B1-U1 already settled how that notation reaches disk: v0.2's `display.size_unit`
is the flat JSON key `size_unit`, because the document is one flat object and a nested group for a single field
would be a shape with no second instance. `theme` follows the same rule. `ADR-0031` records the mapping so nobody
later "fixes" the file into a nested `appearance` object and breaks the reader.

### D4 — `system` means the platform's own answer, read through the WebView

`prefers-color-scheme` is a standard CSS media query, available in the WebView2/WebKit/WKWebView that Tauri already
embeds. It needs no Tauri plugin, no new capability, no permission prompt and no network, which keeps
`AGENTS.md` §7 and the local-first boundary untouched.

- The OS is consulted only while the stored value is `system`. A person who picks `light` or `dark` stops following
  the OS until they pick `system` again.
- A change while in `system` re-themes the live window; the listener is registered with the subscription and torn
  down with it.
- Where no OS answer is available, the resolved theme is **light**, and the page says so in the same voice the
  missing-config-folder case already uses. Light is the fallback because it is the theme every existing screenshot,
  every accepted reference and every current token value describes.
- The resolved theme is applied before first paint, including the native window background, so a dark user does not
  see a white flash on the way to dark. Where that cannot be proven on a platform, the flash is reported as
  unverified rather than assumed away.

### D5 — Tokens stay the only numeric authority, and dark becomes a sibling of light

`assets/design-tokens.json` holds `color.light` with 30 semantic roles. This ADR authorizes one sibling,
`color.dark`, carrying **the same 30 role names** — no new role is invented to make a dark value convenient, and no
light role is dropped. Rules:

- The existing light values keep their exact numbers. Adding dark must not move a light token by one hex digit; the
  light regression is part of the acceptance, not an afterthought.
- `scripts/generate_design_tokens.py` hard-codes `THEME = "light"` and emits one `:root` block. It becomes a
  per-theme emitter: `:root` for light, a theme attribute selector for dark, both generated, neither hand-edited.
  `tokens.css` remains generated output and stays in the drift check.
- Exactly one token reference in the file is theme-qualified today — `focus.ring.color` is
  `{color.light.accent.primary}`. It becomes theme-relative, or every dark theme inherits a light-mode focus ring
  and the accessibility story is inconsistent at the one place `DESIGN.md` makes focus a first-class requirement.
- No global CSS filter, no `invert`, no overlay film, no gradient, no shadow on panels or table rows. Dark is a set
  of values, not a transform.
- The five semantic states stay five, and `UNKNOWN` keeps its neutral hollow treatment in both themes. A dark
  surface is not a licence to make "we do not know" look like "this passed".

### D6 — Contrast becomes measurable in both themes, with one named exception

`DESIGN.md` has always required a clear `focus-visible` ring and forbidden colour-only status. From this ADR, both
themes must hold WCAG 2.2 AA on the roles that carry meaning: 4.5:1 for body text, 3:1 for large text and for the
non-text visual information that carries state, focus or selection. The numbers come from a script that reads the
token file, not from looking at a screenshot; where a ratio is only achievable by changing a value, the change goes
into the token file.

Eighteen rendered pairs are measured per theme. Seventeen pass in both palettes. The eighteenth is carried, and
carried out loud:

> **light · `diff.removed` on `diff.removed.bg` = 4.4974:1** against a 4.5 floor.

It fails by 0.0026, and it is not a new failure — the pair shipped in `B1-U1`'s predecessor and nobody had measured
it. The value that clears it is `#FBECEA` → `#FCEEEC`, and `#FBECEA` is already printed into
`golden/reports/p2-diff.html` and `golden/reports/p4-release/release-report.html`: the same byte belongs to two
committed report artefacts, and D7 (and `NEXT-04` §D) puts those outside this round. `crates/firmwaresight-report`'s
own contract test, `html_export_colours_come_from_the_frozen_token_set`, is what proved that on the first attempt —
it failed the gate when this round tried to move the token, which is the check working rather than a rule being
worked around.

So the floor stays at 4.5, the exception is enumerated in `scripts/generate_design_tokens.py` with its measurement
and its reason, and the generator makes the *exception* itself a checked fact: if the pair ever passes, the stale
exemption fails the build. Clearing it is a separate decision about two goldens, not a detail of a theme unit. The
chip that carries the sub-floor pair reads "Removed" in words beside its colour, which is what `DESIGN.md` §5
requires of a state in any case.

### D7 — Desktop chrome only. Every exported artefact stays light

The Release Bundle HTML, the portable report, CLI JSON, goldens, hashes, `runId`, `releaseId`, Gate verdicts,
evidence classification and stored rows are unaffected by a theme, and a theme change must not cause a re-query or a
re-run. This is the B1-U1 isolation proof extended: the same fixture, analysed once, produces byte-identical output
under all three settings. A person's reading comfort is not a fact about their firmware.

### D8 — Scope: the Developer Preview, not a brand change

This ADR changes which themes the application renders. It does not change the temperament, the type stack, the
layout, the density, the chart rules or the component iron laws; §1 of `DESIGN.md` still decides what the product
looks like, and dark is that same instrument drawn on a different ground. The 7 accepted UI reference screenshots
remain light, and they remain the reference for layout and density — they stop being the reference for ground and
ink colour, which is the only thing this ADR moves.

## Alternatives

- **Keep light-only until GA.** Rejected: it leaves the product unusable in the lighting its own use case describes,
  and it makes the Settings page's future theme control a lie until GA.
- **Add the field without a version bump (`schema_version` stays 1).** Rejected on the evidence: the document is
  `deny_unknown_fields`, so an older build would read a newer file as *corrupted* — "the stored settings could not
  be read" — instead of the accurate "written by a newer version", and would then be free to overwrite it. The
  version exists to make that distinction; using it is the cheap option.
- **Migrate on read: rewrite a v1 file as v2 at startup.** Rejected: B1-U1 proved twice per cold start that opening
  the app writes nothing, and a silent write at launch would break that invariant, write to disk during a read-only
  inspection, and create a new failure mode (a launch that can fail) to solve a non-problem.
- **A `data-theme` attribute plus a hand-maintained second CSS file.** Rejected: two places for one number is the
  drift `ADR-0018` D5 was written to prevent.
- **A theme plugin from the UI ecosystem.** Rejected: `AGENTS.md` §4, and 30 roles do not need a dependency.
- **Per-page or per-document theme.** Rejected: presentation is one decision per window, and two answers to one
  question is the same defect B1-U1 avoided by putting the unit switch in the shell.

## Consequences

Positive:

- The product is readable where firmware is actually flashed, which is the use case the temperament describes.
- The preference mechanism earns its second field, proving the B1-U1 design was a mechanism and not a one-off page.
- Contrast stops being an opinion: the token file becomes testable arithmetic in two themes.
- The versioning contract is exercised for real, before a third preference forces it.

Costs:

- Every component with a hard-coded light value has to be found and re-pointed at a role. That is the bulk of the
  work and it is deliberately not delegated to a global transform.
- Screenshots, docs and the accepted reference set now describe one of two themes; the light set stays authoritative
  for layout only.
- The verification matrix doubles for the visual cases: each state, each surface, two themes, two window sizes.
- `ADR-0018` is no longer the whole answer about theme, which means anyone reading only it will be wrong. `AGENTS.md`
  §11, `DESIGN.md` §4/§9 and `BASELINE.yaml` `design.theme` are updated in the same breath as this ADR for that
  reason.

## Revisit trigger

Reopen when the product leaves the Developer Preview — at which point the Owner decides whether dark ships in a
released build — or when real user evidence shows a dark instrument table failing on legibility, or when a system
theme API becomes available that can distinguish high-contrast and reduced-transparency intent from colour scheme
alone, which would make "system means light or dark" too coarse a reading.
