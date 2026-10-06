---
title: "P5 Commit F2R design record"
doc_id: "FS-P5-COMMIT-F2R-DESIGN"
product: "FirmwareSight"
version: "1.0"
status: "DESIGN"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 Commit F2R — installed UI productization corrective, design record

Written before any production change, as §14 requires. Nothing here is reverse-engineered from a diff
that had already landed: at the moment this file was created `git status --short` showed only the two
§3 archive files, and no file under `apps/desktop/ui/src/` had been edited.

## 0. Start authority, read before writing

| item | value |
| --- | --- |
| `HEAD` = `origin/main` | `55fad63d2071c5511133ea1cfffc664885b0b47e` — "P5: reconcile the entry documents with the installed round that landed" |
| worktrees | one, `D:/study/Software/FirmwareSight 55fad63 [main]` |
| tree delta at write time | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitF2R_UI_Productization_Corrective_v1.0.txt` (staged, §3) and `10_AUDIT/SOURCE_PROMPTS/README.md` (+47) |
| UI suite at this head, run here | **219 passed / 8 files**, 27.73 s (`pnpm test`) |
| Rust / gate | 868 passed / 0 failed across 47 targets, full gate 17 of 17 — the Architect's §0 figures, re-run at the candidate before commit (§21) |
| F2 CI | run `37367382516`, attempt **3** = 10 of 10; attempts 1 and 2 cancelled 7 and 4 jobs with 0 executed steps. F2 is not rewritten as first-attempt green (§0) |
| evidence root | `%TEMP%\FirmwareSight-P5-F2R-20261005T221112` |

Prompt: *FirmwareSight — P5 Commit F2R Installed UI Productization Corrective Candidate, Execution
Prompt v1.0 — Architect Authorized*, delivered sha256 `5d8719ec…b1fb375` / 47,100 B / 2,117 lines
CRLF, stored blob `5f2bbda77c82fdcf57a6f9dfb48a4e622b28aa3e`, line-by-line identical with only line
terminators differing (`10_AUDIT/SOURCE_PROMPTS/README.md`).

Stage at entry: `P5 = IN_PROGRESS`, `Product = MVP CANDIDATE`, `F1 = FINAL PASS`, `F2 installed
journey = PASS`, `F2 closure = NEEDS_CORRECTIVE_PRODUCT_HEAD`, `F3 = NOT AUTHORIZED` (§0).

## 1. The three findings, and the F2 evidence that found them

| id | severity | finding | F2 evidence |
| --- | --- | --- | --- |
| **F2R-01** | **S2** (layout) | Analyze → Sections collapses its text column until rows are unreadable instead of reflowing | `P5_DESKTOP_ACCEPTANCE_REPORT.md:134`; capture `07_analyze/L07_base_sections_symbols.png` at the default 1056 × 799, where the `.bss` File-offset reason renders **one character per line** and the row is ~470 px tall |
| **F2R-02** | **S3** (layout) | History's row action `Details` is clipped at the default real-window width, across all three table families | `P5_DESKTOP_ACCEPTANCE_REPORT.md:135`; captures `11_history/R00–R12` |
| **F2R-03** | **S3** (presentation) | Release → Evidence basis prints the raw Core enum word `MapRegionAndElfLoad` while every other page prints human text | `P5_DESKTOP_ACCEPTANCE_REPORT.md:133`; `P5_KNOWN_LIMITATIONS.md:124`. This is L20's defect shape on a surface L20 never covered |

F2's own dispositions stand: no fact was wrong in any of the three, nothing was blocked, and F2 could
not patch them because its prompt confined it to evidence (`P5_DESKTOP_ACCEPTANCE_REPORT.md:133`).

## 2. Current code path

| surface | path |
| --- | --- |
| Sections table | `Details.tsx:408` `SectionTable` → `Details.module.css:126` `.table { width:100%; display:block; overflow-x:auto }`, `:142` `.table th, .table td { white-space:nowrap }`, `:157` `.unknown { display:inline-flex }`, `:164` `.note { white-space:normal; overflow-wrap:anywhere }` |
| Symbols / Evidence tables | same `.table` rules; `Details.tsx:639` `td class="mono value"`, `.value` at `:152` also carries `overflow-wrap:anywhere` |
| History rows | `History.tsx:264` builds table (8 columns, action last), `:389` Gate runs (7 columns, action last, `GateRows:531`), `:464` release records (7 columns, action last, `ReleaseRows:607`); `History.module.css:45` `.grid { display:block; overflow-x:auto }`, `:59` `.grid th, .grid td { white-space:nowrap }`, `:155` `.action` |
| Release basis | `Release.tsx:982` `<span>{row.basis ?? 'no basis recorded'}</span>` under the header `Evidence basis` at `:881` |
| accepted caption mapping | `Compare.tsx:685-703` `EVIDENCE_BASIS_CAPTIONS` + `basisCaption()`, used at `Compare.tsx:721` |
| second verbatim site (§13 audit) | `Analyze.tsx:420` and `:428` print `Weakest basis ${basis}` with the wire value untransformed |

## 3. Root cause, measured rather than assumed

`01_layout_probe/probe3.html` re-declares the shipped rules and the shipped markup shapes with real
product strings, and measures them in Blink at the three acceptance sizes. Full numbers and the
method's limits are in `01_layout_probe/MEASUREMENTS.md`; the load-bearing results:

1. `display:block` on the `<table>` forces the table box to the pane width. The table can then be
   laid out **narrower than its own minimum**, and the deficit lands entirely on the one column whose
   minimum is small. `overflow-wrap: anywhere` makes that minimum **one character**, so the prose
   column absorbs everything: measured 18 × 380 px, row 529 px, at both 1024 and 1056. The F2 capture
   shows the same thing at one character per line.
2. Moving `overflow-x: auto` to a wrapper and letting the table stay `display: table` is **necessary
   but not sufficient**: with the shipped `anywhere` still on the prose, the table keeps its intrinsic
   minimum but that minimum still contains a 18 px column (measured: identical 18 × 380 px). The
   second half of the cause is the unbounded minimum, not the scroll axis alone.
3. History's action is clipped for a different reason: the table's rigid minimum (939 px measured) is
   wider than the pane (736 px at the frozen 1024 minimum), and the action is the **last** cell, so it
   sits 195 px beyond the right edge of the scroller. It is not a CSS bug; it is a placement that
   cannot survive a table wider than the window.

## 4. Chosen minimal solution

**F2R-01 / F2R-02 — one mechanism, applied to the two surfaces the findings name.**

1. A semantic table viewport: a wrapper element owns `overflow-x: auto`, and the `<table>` returns to
   `display: table` with `width: 100%`. The table then keeps its intrinsic minimum instead of being
   crushed to the pane, and the table area owns whatever scroll remains — §6's preferred direction.
2. Cells wrap at word boundaries (`white-space: normal`), while `.mono` keeps `white-space: nowrap` so
   addresses, byte figures and ids never split and columns stay aligned. `.value` re-declares
   `white-space: normal` after `.mono`, because the Evidence table's value cell carries both classes.
3. The Unknown reason moves onto its own line inside the cell (`.unknown` becomes a column flex), so
   the column's minimum stops being driven by a whole sentence.
4. The prose keeps a typographic floor: `overflow-wrap: break-word` plus `min-width: 24ch`. `ch` is
   the unit the shipped stylesheets already use to bound prose (`Analyze.module.css:23` 60ch,
   `Compare.module.css:27` 60ch, `Release.module.css:27` 68ch, `:96` 72ch, `:312`/`:319`/`:467` 88ch,
   `History.module.css:27`/`:108` 80ch, `GettingStarted`/`Help` 80ch). It is a measure in the design
   language's own unit, not an invented pixel constant, and no token file is touched.
5. History's row action becomes the **leading** cell of the header and of each row, in all three
   table families. This is not a new invention: the Evidence table already does exactly this and
   already says why — `details.test.tsx:751` "keeps the Inspect action in the first column, where no
   wide value can push it away", with the comment recording that the action was previously "the thing
   hanging off the right edge". `Details` stays a text button with `aria-expanded`; §8's icon-only
   shortcut is not taken. Column counts and every `colSpan` stay as they are.

Measured outcome of this combination (variant `v9`):

| | 1024 (pane 736) | 1056 (pane 768) | 1440 (pane 1152) |
| --- | --- | --- | --- |
| Sections table | 1105 px, area scrolls 369 | scrolls 337 | **1152 px, scrolls 0** |
| Unknown reason | 158 × 40 px, row 97 px | 158 × 40 px | 167 × 40 px |
| Name column / headers | 159 px / 49 px, legible | same | same |
| Builds `Details` | right edge 121, fully visible, hit-test hits the button | 117, visible | 117, visible |
| Symbols, Release budgets | no scroll at all | no scroll | no scroll |
| Evidence table | 879 px, area scrolls 143, digest on one 446 px line | scrolls 111 | scrolls 0 |
| page / shell overflow | **0 px** | 0 px | 0 px |

**F2R-03 — one shared UI-only display helper.** Extract `evidenceBasisCaption(...)` into a small
UI utility module, backed by the map `Compare.tsx` already owns, and use it from `Release.tsx`,
`Compare.tsx` and `Analyze.tsx`. No DTO, schema, wire, storage or Rust value changes; the caption is
never serialized (§11).

## 5. Mechanisms measured and rejected

| rejected | measured reason |
| --- | --- |
| wrapper alone, prose left `anywhere` | the reason column still renders 18 × 380 px; the defect survives behind a scrollbar |
| floor without the own-line reason | table intrinsic 1292 / 1279 px → the table area scrolls **127–140 px at the 1440 design target**, where the shipped build scrolls 0. §30 asks for no regression from target layout |
| `position: sticky; right: 0` action cell | the pinned cell **covers 195 px of stored facts at 1024 and 163 px at 1056**, because it is pulled that far left of its natural position; its separator would also need a shadow, which AGENTS.md §11 forbids on tables. §8: "Do not blindly implement sticky positioning if it introduces overlap. Measure the actual page." Measured: it does |
| letting the mono ids and timestamps break so the table fits | a 64-hex digest or an ISO timestamp split across lines fails DESIGN.md §Tables (long values must stay fully viewable/copyable) and destroys the stable figure alignment the mono columns exist to give |
| dropping the `Details` word for an icon | §8 forbids it: the textual action is part of the accessibility contract |
| reason on its own line with **no** floor | fits the target with 0 scroll but leaves a 71 px × 120 px prose column at 1024 — less extreme, not readable by measure |

## 6. Design-token compliance

No value in `assets/design-tokens.json` changes, and no numeric literal is introduced except the
`24ch` prose measure, expressed in the same unit the shipped prose rules already use. Row density,
hairlines, radii, spacing, motion, fonts and colours keep coming from tokens; the tables stay
`--fs-row-compact`. No gradient, shadow, colour-only state or new accent appears; `Unknown` keeps its
neutral treatment (`Details.module.css:157`). No focus-visible rule is touched. AGENTS.md §11's
prohibition on panel/table shadows is one of the reasons the sticky variant was rejected.

## 7. One trade, recorded rather than hidden

At the frozen 1024 minimum and at 1056 the Sections table is wider than the pane (1105 px against 736
and 768) and therefore scrolls **inside its own table area**. §6 lists that as the accepted outcome
("if horizontal scrolling is needed, the TABLE AREA owns it") and forbids the alternative that
produced the S2 defect. At the 1440 design target the corrected table fits with zero scroll, which is
what §30 asks for; the row is 97 px against the shipped 89 px because the reason now sits on its own
line under `Unknown`. If the Architect would rather trade the readable measure for a zero-scroll
minimum window, the measured knob is the floor: `18ch` gives 250 px of contained scroll with a
119 px prose column, and no floor gives 94 px with a 71 px column. This record recommends `24ch`.

## 8. F2R-03 wording, and the two prompt lines that do not match the code

§11 asks for one shared helper, says "Do not create divergent wording in Release", says "Canonical
current wording must remain aligned with existing accepted Compare vocabulary", and says "Do not
invent semantics". Its mapping table then spells two values that do not exist in the domain
(`ConfiguredRegionAndElfLoad`, `InsufficientEvidence`; the real variants are
`RegionConfigAndElfLoad` and `Insufficient`, `crates/firmwaresight-core/src/domain/memory.rs:165-176`)
and spells one caption with a plus (`ELF address + flags evidence`) where accepted Compare prints a
slash (`ELF address/flags evidence`). Resolution, in the order §11's own priorities give:

- The helper is extracted from Compare's existing map **verbatim**, so Release cannot diverge from
  Compare and no new wording is invented. `RegionConfigAndElfLoad` and `Insufficient` map to the
  existing accepted captions; the two prompt spellings are recorded as not present in the codebase
  rather than added as aliases.
- The `ElfAddressAndFlags` caption stays `ELF address/flags evidence`. Changing it would contradict
  §11's "aligned with existing accepted Compare vocabulary", and the repository has already decided
  this once: `.ai/DECISIONS.md:1760-1762` records "Wording drift was named, not repaired" and rules
  that unifying the two spellings would be a wording change wearing a fix's clothes;
  `P5_COMMIT_E_CLOSURE_NORMALIZATION.md:164` and `P5_COMMIT_F_DESIGN.md:269` both weigh it and leave
  it. The plus spelling remains what `Details.tsx:659` uses for the **different** `SourceType` token,
  which is L15's caption and is not this map; both forms are currently test-pinned
  (`details.test.tsx:700,728` and `compare.test.tsx:744,768`). The residual drift is named here, not
  repaired, and §32 records the string the installed page actually prints.
- Wire/kebab aliases stay supported because Compare already supports them (`Compare.tsx:687`, `:689`,
  `:691`, `:693`, `:695`), which is the condition §11 attaches to keeping them.
- Null keeps Release's existing context-appropriate neutral text `no basis recorded`; an unrecognised
  value prints `Unrecognized evidence basis` (§11).

§13 re-audit, whole UI tree, for the five enum spellings and the wire aliases: the only
human-facing verbatim outputs are `Release.tsx:982` and `Analyze.tsx:420`/`:428`. Compare's line at
`:721` already maps. Everything else is domain code, Rust test data, generated DTO types, CLI goldens
(portable/wire, allowed) or the display maps themselves. `Analyze.tsx` is included in this round
because §13 authorizes including another human-facing verbatim output "only if the same shared
display helper closes it narrowly", and one call to `evidenceBasisCaption(basis)` closes both of its
lines with no semantics change. That adds `Analyze.tsx` and `intake.test.tsx` to §16's expected path
list, which is the smallest set that satisfies §13; no other file outside §16's list is touched.

## 9. Automated-test boundary

jsdom cannot certify clipping, so no geometry is faked (§7, §9). The red evidence for the two layout
findings is `REAL_DESKTOP_F2_EVIDENCE` — the F2 captures named in §1 — and the automated layer
protects the contract that makes the fix possible:

- **structure**: every Details and History table sits inside a viewport element that is not the table
  itself (`table.parentElement` carries the viewport class, the table is still a `<table>` with its
  `columnheader` / `cell` / `row` roles), its ten sections columns are all named, and no branch removes
  a column.
- **action placement**: `Details` is the first cell of each History row and the first `columnheader` of
  each History header row, in all three table families, still a button with correct `aria-expanded`
  transitions.
- **F2R-03**: written first and observed failing at `55fad63` (§15) — Release renders
  `MAP regions + ELF load evidence` and does **not** render `MapRegionAndElfLoad`; an unrecognised
  value renders `Unrecognized evidence basis`; null renders `no basis recorded`; Compare's existing
  caption tests still pass against the shared helper; Analyze's weakest-basis line stops printing a
  wire token.

A CSS-source contract test was written first and then **removed**: this environment cannot read a
stylesheet as text — `?raw` yields `{}` and `?inline` yields `""` while Vitest's default `css: false`
stubs CSS imports, and making it work would mean editing `vite.config.ts` or adding a dev dependency,
which §16 and §18 forbid. The invariant is therefore protected by the rendered tree (which element owns
the scroll axis, which cell leads the row) plus §20's authoritative pre-fix/post-fix screenshot pair.
No screenshot or geometry library is added, no frontend dependency of any kind, and the existing
History/Compare assertions that already prove focus, `aria-expanded` and open/close behaviour are
preserved rather than rewritten. One existing History assertion read a Gate row's cells **by position**;
it now reads the cell under the `Baseline` column by name and separately pins the seven-column count, so
the reorder shows up as a deliberate change rather than as a broken test.

**Validation sequencing, recorded because it nearly produced a false claim.** The first §21 logs
(`2026-10-06T06:43:00Z` and `06:44:03Z`) and the first §22 campaign were taken while three UI files still
carried comment-and-import-order text that moved at `06:47:03Z`. Both were therefore describing a tree
that no longer existed. The campaign was stopped mid-flight and kept as
`05_ui_reliability/CAMPAIGN_superseded_partial.txt`; §21 was re-run at `07:05:37Z → 07:08:56Z` with the
manifest of every `apps/desktop/ui/src` digest captured before and after (`e5713cf5…8b79`, identical),
and §22 was re-run for the full 20 repetitions at `06:48:56Z → 06:56:23Z`. Nothing was discarded: the
superseded artefacts stay in the evidence root, and only the runs that prove tree stability are cited.
A plain `grep` for the vitest summary line returns nothing from that file because vitest writes colour
escapes, so the 20 / 20 count was taken from the same bytes with the escapes stripped, and the strip
counts are recorded beside the claim.

## 10. Real-desktop acceptance and the focused installed plan

Acceptance sizes, per §6, §8 and §30–§32: **1024 × 720** (frozen desktop minimum), **1056 × 799**
(F2's measured default), **1440 × 900** (frozen design target). Where those come from, since no P5
document states a supported range: `assets/design-tokens.json:60-67` freezes `desktop_min 1024×720`
and `design_target 1440×900`, `03_DESIGN/03_VISUAL_SYSTEM.md:77-78` names 1024×720 as the desktop
minimum target, and `06_DELIVERY/01_MVP_EXIT_CRITERIA.md:60` plus `06_DELIVERY/03_QA_CHECKLIST.md:57`
require 1024 × 720 to be usable. F2 measured the outer window at 1056 × 799 for a 1040 × 760 client
(`P5_DESKTOP_ACCEPTANCE_REPORT.md:134`), so each size is set as the outer window and the client width
is recorded with the screenshot.

One contradiction is recorded rather than fixed, because §16 puts `tauri.conf.json` outside this
round: `apps/desktop/src-tauri/tauri.conf.json:16-19` ships width 1040 / height 760 with
**minWidth 720 / minHeight 480**, so the running window can go narrower than the frozen 1024 minimum
the design language is built on. The corrected mechanism degrades monotonically below 1024 (the table
area simply owns more scroll), where the shipped one degraded into a one-character column, but the
window floor itself is not this round's to change.

The installed run is focused, not a §64 rerun (§27): it installs the **new F2R1 CI-built artifact**
built by F2R1's own run — never the F1 artifact `11337963032`, never F2's installed binary, never a
local `cargo`/`tauri` build (§26) — and re-runs only Analyze → Sections, History's three Details
controls and Release's Evidence basis at those three sizes, plus the §33 quick functional smoke
(Analyze, Compare, Gate, History, Diagnostics). No migration proof, no performance workload, no
bundle recreation unless the focused target needs it. Uninstall and cleanup afterwards, and the
owner's store returns byte-identical.

## 11. Owner-store isolation

Before any installed work: `OWNER_STORE_PARKED = YES` and `OWNER_BACKUP_HASH_MATCH = YES` verified
from hashes, and the park barrier re-proved the way F2 proved it. Then, for the focused run:
`ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`, `OWNER_STORE_OPENED_BY_F2R = NO`. The
only evidence copy is never moved. Records reach the disposable store only through the product's own
UI; no arbitrary SQL seeding, no owner data (§28).

## 12. Public-contract impact

None. §17's compatible list is untouched by construction: `analysis:1`, `diff:1`, `gate-results:1`,
`accepted-reviews:1`, `release-manifest:1`, SQLite schema 5, migrations 0001–0005, ADR-0028,
ADR-0029, and the L15 legacy wire token `elf.program-header`. No migration 0006, no `analysis:2`, no
wire rename, no release identity change. §18's dependency and security list is likewise untouched: no
crate, package, capability, permission, network, telemetry, generic filesystem/shell/SQL IPC, updater
or signing change. The round is display and layout only, and the F2R1 diff is confined to
`apps/desktop/ui/src/**`.

## 13. Stop conditions carried into implementation

- A genuinely new **semantic token** becomes necessary → stop and explain before touching a frozen
  asset (§5). Preferred outcome, and the measured outcome here: no token change.
- Any need to change `crates/**`, `src-tauri/**`, schemas, migrations, fixtures, workflows,
  `Cargo.toml`/`Cargo.lock`/`pnpm-lock.yaml`, `tauri.conf.json` or `assets/design-tokens.json` →
  stop and return to the Architect (§16, §17, §18).
- A repository or content CI failure → fix forward in a new product commit; never rerun-until-green.
  A job cancelled with zero executed steps is an allocation failure, not a product failure, and the
  retry policy goes back to the Architect if it becomes ambiguous (§25, §40).
- Desktop harness: no blind screen-coordinate click without confirming target ownership, no
  undocumented Win32 message ids, no clicking while another application owns the foreground, no
  DevTools/CDP product action, no JavaScript `click()`, no direct Tauri IPC, no direct DB mutation as
  a product action. If foreground ownership cannot be obtained, refuse the click. A harness failure
  stays classified as a harness failure unless reproduced independently (§29).
- New S0/S1 or a new S2 during the focused installed run → stop before F3; a new S3 goes to the
  Architect and is not self-waived (§33).
- No `P5 PASS_COMPLETE`, no `Productization ENGINEERING_COMPLETE`, no `active_task NONE`, no tag, no
  GitHub Release, no signing, no notarization, no V1/B1 (§24, §37, §45, §46). F2R ends by returning to
  the Architect; F3 needs a separate authorization.

## 14. What this document does not claim

It does not claim P5 is complete, that F3 is authorized, or that any of the three findings is fixed —
the fixes are specified here and proven later by §20 mutation proofs, §21 validation, §22 reliability
and §30–§33 installed evidence. It does not claim WCAG certification: §34's exit judgment is bounded
to `PASS_FOR_FROZEN_DESKTOP_SCOPE` with `WCAG_CERTIFICATION = NOT_PERFORMED`,
`MULTI_DPI_125_150 = NOT_TESTED` unless actually tested, and `SECOND_WINDOWS_HOST = NOT_TESTED`.
It does not claim an accessibility-role gain from the wrapper: `take_snapshot` flattens table, row and
cell roles for both a plain and a `display: block` table (`01_layout_probe/a11y-control.html`), so the
tool gives no evidence either way and the wrapper is justified by measured layout behaviour only.
