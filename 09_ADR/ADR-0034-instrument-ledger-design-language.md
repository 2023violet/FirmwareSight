---
title: "ADR-0034 Instrument Ledger Design Language (R-Series Redesign)"
doc_id: "ADR-0034"
product: "FirmwareSight"
version: "0.6.0"
status: "PROPOSED"
owner: "Architecture / Design"
last_updated: "2026-10-16"
---

# ADR-0034 — Instrument Ledger: One Design Language for the App and Every Export

## Status

**PROPOSED** (R0, 2026-10-16). It becomes BASELINE when the Owner accepts the R0 prototype. Until then no token value, product file or golden changes. It extends ADR-0018 and amends ADR-0031 D6 (see D3).

## Context

On 2026-10-16 the Owner asked for the next round to return to product building with a complete UI/UX redesign. The scope covers the desktop app and every export: HTML, SVG, PNG, Markdown and CLI text. The Owner's own B1 "Quiet Canvas" study is treated as reference only. The diagnosis is in `03_DESIGN/09_R_REDESIGN_SPEC.md` §1.

In short, the shipped UI:
- stacks long pages, solves one job in several ways, and makes evidence reachable from one tab only;
- reports staleness in scattered paragraphs.

The B1 study repeats one template on every page, colours change as good or bad, and has no Inspector.

The Owner decided four things on 2026-10-16:
1. a targeted token bump under this ADR;
2. spec and prototype first;
3. D3 folded into the History redesign;
4. light `diff.removed` → `#C7372C`.

## Decision

**D1 — Five principles.** They bind every R-phase:
1. One answer per page (`LeadAnswer`).
2. Every number traces to its evidence (`FactValue` → `Inspector`).
3. The subject is always visible (`SubjectBar`).
4. Change is a fact, not a judgement: deltas are neutral with ▲▼, and only Gate states carry status colour.
5. One way per job (`DataTable`, `Tabs`, `Notice`, `BuildPicker`).

**D2 — Shell and information architecture.**
- The shell is TopBar 40 / Rail 224 (collapsing to 56) / SubjectBar 44 / Workspace / Inspector 340 (320–420).
- The Inspector docks at ≥1280 px and becomes a Sheet below that.
- Rail items are Overview · Analyze · Compare · Release · History, then Help · Settings. "Release Gate" is renamed "Release" and "Bundle & History" is renamed "History".
- Views use one Tabs pattern. No fifth product verb and no new i18n namespace (ADR-0032 D4).
- The window minimum becomes the token minimum, 1024×720.

**D3 — Tokens v0.4.0.** Additive, with one value change:
- New:
  - `font.size.figure` 28 / line-height 36; `font.weight.regular` 400 / `semibold` 600;
  - `layout.topbar` 40, `layout.subject_bar` 44, `layout.rail_collapsed` 56;
  - `layout.breakpoint.rail_collapse` 1180, `layout.breakpoint.inspector_dock` 1280;
  - `icon.size.sm` 16, `icon.size.md` 20, `icon.stroke` 1.5.
- Changed: light `color.diff.removed` `#C9372C` → `#C7372C` (4.56:1 on `#FBECEA`). This retires ADR-0031 D6's only contrast exemption. `CONTRAST_EXEMPT` becomes empty, and the goldens that print the colour are regenerated on purpose.
- `font.size.figure` may appear once per page, in the `LeadAnswer`, and once per report, in the lead line.
- Palette, spacing, radius, density, motion, focus and shadow values do not move. The no-gradient, no-glass and no-panel-shadow rules stand.

**D4 — Icons.** One in-repo SVG set (about 24 icons; viewBox 20; stroke 1.5; `currentColor`), shipped as React components and inlined into exports. No icon library (ADR-0011). Each of the five states has a distinct shape, so colour is never the only carrier.

**D5 — One report grammar.** All exports follow the same order:
1. masthead;
2. lead line, the same sentence as the app;
3. facts, tables first;
4. evidence appendix;
5. limitations;
6. footer with full hashes and generator version.

Exports stay light-only and deterministic and load nothing from the network. HTML is semantic and A4-printable, and SVG carries `<title>`/`<desc>`. The JSON schemas, `SHA256SUMS`, the manifest and bundle membership do not change.

**D6 — Markdown export.** A new human-readable Markdown report for Analyze, Compare and Gate, in GFM with no HTML. It is exported from the desktop menu and the CLI `--markdown <FILE>`, and never overwrites an existing file. It is not part of the release bundle (ADR-0028). This promotes candidate E1-B by Owner exception. It is recorded with D3 of the route.

**D7 — Phases.** R0 spec and prototype → R1 foundation → R2 Analyze and Overview → R3 Compare → R4 Release → R5 History (with D3), Settings, Help and the command palette → R6 reports (after R1, in parallel) → R7 quality. Each phase needs independent review, an Owner-approved fast-forward push, and worktree cleanup.

## Consequences

- Tests that pin accessible names change in R2–R5. Each phase carries a name-mapping table and updates the tests in the same commit.
- `DESIGN.md` §4–§10 and `03_DESIGN/00–06` are revised in R1 to match this ADR. The accepted screenshot register (`06_UI_REFERENCE_SCREENS.md`) is replaced after R7.
- Golden HTML, SVG, PNG and CLI outputs change in R6, plus the `diff.removed` change in R1. Both are regenerated deliberately with `scripts/update_goldens.py --confirm` and reviewed diff by diff.
- D2C's open human items (native QA, Pilot, V1 re-freeze) are not closed by this work. The native-QA runbook gains the new screens in R7.

## Alternatives rejected

- **Adopt B1 as the redesign.** It is clean, but it keeps one template on every page, uses evaluative colour, and has no Inspector.
- **No token change.** The lead number would stay at 20 px, beside 20 px titles, so the hierarchy stays flat.
- **New visual language (bundled font, new palette).** That is a larger change than the problems require. It would mean a new contrast and golden programme, and ADR-0018's temperament is not what is wrong.
- **A UI kit or icon library.** Rejected by ADR-0011; the needed set is small.
