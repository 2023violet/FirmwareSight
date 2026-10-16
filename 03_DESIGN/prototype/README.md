# R0 prototype — Instrument Ledger

`instrument-ledger.html` is the interactive R0 prototype for [ADR-0034](../../09_ADR/ADR-0034-instrument-ledger-design-language.md) and [09_R_REDESIGN_SPEC](../09_R_REDESIGN_SPEC.md). It is a design study, not product code. Nothing in it ships, and no product file, token value or golden changed with it.

- **Open:** double-click the file (it is self-contained, loads nothing from the network), or use the Owner's private Artifact copy.
- **Data:** real values from `fixtures/elf/m2-objects` (base → target) and the `fixtures/project/p4-release` policy, as produced by `fwsight` at `e4d7f8b`. The Gate verdict is the p4 policy evaluated on the m2 target. Import times in History are examples.
- **Tokens:** every colour, size, radius and motion value is from `assets/design-tokens.json` v0.3.0, plus the v0.4.0 additions proposed in ADR-0034 D3 (figure 28/36, weights, shell sizes, breakpoints, icon sizes, light `diff.removed` `#C7372C`).
- **Review controls** above the window (theme, language, scenario) are not part of the product. Scenarios: normal, empty, parse error, stale, no MAP, Gate not run.
- **Covered:** Overview, Analyze (Memory · Sections · Symbols · Objects · Evidence), Compare (Summary · Sections · Symbols · Objects), Release (setup steps, Findings · Evidence · Bundle), History with the D3 baseline pin, Settings, Help, the Inspector, the `Ctrl K` command palette, and an export-samples page (HTML report, SVG 1600×900 summary, Markdown, CLI text).
- **Keyboard:** `Ctrl K` palette · `/` filter · `[` `]` views · `I` Inspector · `Esc` close.
- **Self-check (2026-10-16):** 276 renders (2 languages × 6 scenarios × every page and view) with no script error and no `undefined`/`NaN` text; English pages carry no CJK except the language endonym and a bilingual type sample; no horizontal overflow at 400 px.

Known limits: the prototype's buttons that would open files, write data or export show a toast instead; the A4 report is shown on screen, not printed; PNG output is represented by its SVG source.
