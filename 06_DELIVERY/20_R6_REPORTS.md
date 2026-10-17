---
title: "R6 — Reports: the report grammar in every export"
doc_id: "FS-DEL-021"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE"
owner: "Product / Delivery"
last_updated: "2026-10-17"
---

# R6 — Reports: the report grammar in every export

**Authorization.** The Owner's standing goal of 2026-10-17 (see [19_R5](19_R5_HISTORY.md)).

**Base.** R6 is stacked on `feature/r5-history` (`29b13bb`).

**Design sources.** [03_DESIGN/09](../03_DESIGN/09_R_REDESIGN_SPEC.md) §8 and [ADR-0034](../09_ADR/ADR-0034-instrument-ledger-design-language.md) D6.

## 1. What changes

Every export reads in one order (§8.1):
1. a masthead naming the product, the report and its subject;
2. the answer;
3. the facts;
4. the evidence;
5. the limits, written as facts;
6. a footer naming the generator.

It is staged in three parts.

- **R6a — CLI text and Markdown.**
  - `analyze`, `diff` and `gate` print in the terminal edition of the grammar:
    - a `FirmwareSight · <report>` masthead ruled to 80 columns;
    - the answer as the first line;
    - section headings underlined with `─`;
    - prose wrapped at 80 columns, with identifiers never broken;
    - states as bracketed words (`[BLOCK]`).
  - The gate's answer counts the findings whose effective severity carries the verdict, the same count the desktop uses since R4.
  - An `N/A` finding no longer shows its schema-filler `PASS` beside it (spec §8.2's named defect).
  - A new `--markdown <FILE>` on all three commands writes a GFM reading copy:
    - no HTML;
    - deterministic output;
    - absent and unknown written as words;
    - text from a file escaped;
    - never replaces an existing file (exit 6, `ERR-EXPORT-6001`);
    - never part of a release bundle.
- **R6b — desktop Markdown.** Analyze, Compare and Release gain "Markdown" in their Export menus, written through the shell's own Save dialog, like the other exports.
- **R6c — HTML, SVG and PNG.** The detailed comparison HTML, the release report HTML and the visual summaries move to the same grammar. The goldens are regenerated on purpose and each difference is reviewed.

## 2. Decisions

| # | Decision |
|---|---|
| D-R6-1 | No JSON schema, `SHA256SUMS`, manifest or bundle content changes (spec §8.3). `release-notes.md` is the user's and is copied as is. |
| D-R6-2 | One shared terminal module (`report/src/text.rs`) and one Markdown module (`report/src/markdown.rs`). The answer sentence and the state order are reused from the gate renderer, not re-derived. |
| D-R6-3 | Markdown escapes only what can open markup mid-line or in a cell (`\` `` ` `` `*` `_` `[` `]` `<` `>` `|` `~` `&`), so `firmware.elf` stays readable; quotes and apostrophes print as written. A control, bidi or zero-width character is written as its `\u{…}` escape in text and in code spans, so a name can neither hide one nor decode into one (`&` is escaped so `&#x202E;` stays literal). Identifiers are code spans. Known limit: a GFM renderer may still autolink a bare `www.` or `https://` string in free text; nothing is fetched, and identifiers in code spans are not linked. |
| D-R6-4 | The CLI writes the Markdown file before printing, like the HTML export, so a run that could not write it has not announced success. |

## 3. Verification

- `check.py` must pass every step.
- Report-crate tests:
  - the gate text: masthead, verdict, group order, no severity on N/A, 80 columns for prose;
  - Markdown: escaping, report order, no HTML, N/A without severity, determinism.
- CLI tests (`markdown_cli.rs`): analysis and comparison Markdown, determinism, and refusing to replace an existing file.
- Desktop: export tests for the new menu item and a native smoke run.
- Then an independent review.

## 4. Landed so far

- **R6a** (`8d3c002`, `bb53f5d`) covers:
  - `text.rs`;
  - the gate, analysis and comparison text;
  - `markdown.rs`;
  - the `--markdown` flag on all three commands.
- **R6b** covers the desktop side:
  - `export_analysis_markdown`, `export_compare_markdown` and `export_gate_markdown` (use-case commands that write through the shell's Save dialog, with the same stale guard and replace confirmation as the other exports);
  - an "Export Markdown" item on each page's Export menu.

  `markdown::gate` takes the portable gate-results projection, so the CLI's fresh run and the desktop's stored run write the same document. The verdict sentence is one function over state and severity words (`gate_render::sentence_from_words`).
- Tests:
  - report crate: escaping, Markdown order, N/A without severity, determinism, the gate text;
  - CLI: `markdown_cli.rs`, 3 tests;
  - shell: `markdown_export.rs`, 2 tests;
  - UI: 3 export tests (529/529).
- **R6c** covers:
  - **The detailed comparison HTML:**
    - a masthead naming the report and the pair (`<title>` "FirmwareSight comparison report");
    - the answer line before any table;
    - deltas as neutral ▲▼ text with spoken words (the coloured negative and the CSS "+" are gone);
    - a Limits section before the footer.
  - **The release report HTML:** the same masthead and an answer line with the disposition and the five counts, plus a "not a certification" sentence.
  - **The SVG/PNG summary:**
    - `<desc>` repeats the subject, the figures and the first limit in words;
    - a "FirmwareSight · <kind> summary" masthead ruled with the border token;
    - a footer naming the generator and the PNG font dependency;
    - N/A rows no longer carry "severity PASS".
  - **Goldens:** `p2-diff.html`, the `p4-release` report, and its `SHA256SUMS` and manifest were regenerated on purpose with `update_goldens.py --confirm` and reviewed. The bundle's file set and manifest format are unchanged; only the report file inside it, and therefore its digest, changed.

## 5. Independent review and dispositions

The review returned PASS_WITH_FIXES. Every finding was fixed on this branch:

| ID | Finding | Disposition |
|---|---|---|
| S2-1 | Markdown escaping printed quotes and backslashes wrongly, and a hidden character could pass through. | `escape` keeps quotes, doubles a backslash once, and writes control/bidi/zero-width characters as `\u{…}`; `code` does the same inside spans. Unit tests cover quotes, `\`, `&#x202E;`, U+202E and U+200B. |
| S2-2 | The SVG masthead hairline at y=160 ran through the metric labels. | The rule sits at y=80, between the title and the subject (`MASTHEAD_RULE_Y`). `no_rule_runs_through_a_line_of_text` reads every rule and text box back from the SVG; it fails at the old y=160. |
| S2-3 | The desktop Gate Markdown ignored the stored disposition and the accepted reviews, so a cleared run still read "needs a decision". | `markdown::gate_with_record` takes the disposition and the acceptances; the answer states the disposition and the as-computed verdict, and an "Accepted reviews" table records rule, actor, time and reason. The finding still reads REVIEW. Shell test: `the_gate_markdown_reports_the_disposition_and_the_acceptance_as_an_audit_record`. |
| S3-1 | `&` was not escaped. | Escaped (D-R6-3). |
| S3-2 | GFM autolinks were not documented. | Recorded as a known limit in D-R6-3. |
| S3-3 | No CLI test for `gate --markdown`. | `gate_markdown_writes_the_report_grammar_and_never_replaces_a_file` (`gate_cli.rs`). |
| S3-4 | `diff --markdown X --html X` let the HTML silently overwrite the Markdown. | Each command refuses two exports naming one file before any work (exit 6, `ERR-EXPORT-6001`); `two_exports_naming_one_file_are_refused_before_anything_is_written`. Paths are compared as given, not canonicalized. |
| S3-5 | The Analyze export group was still named "Visual report export" though it now holds Markdown. | Renamed "Report export" / "报告导出" (and the error labels on Analyze and Release); UI tests updated. |
| S3-6 | The gate text printed summaries, remediations and refs without the terminal escaping the other reports use. | They go through `terminal_safe`. |
