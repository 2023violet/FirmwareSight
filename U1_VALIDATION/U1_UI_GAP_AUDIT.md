---
title: "U1-A0 UI Gap Audit"
doc_id: "FS-U1-001"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Design"
last_updated: "2026-10-06"
---

# U1-A0 — UI Gap Audit

Track token: this round was delivered as "B1 — UI Productization / Design Convergence". `B1` is already
the canonical identifier for **Private Beta** (`06_DELIVERY/06_STAGE_GATES.md`), and F3 §37 and V1 §40 both
record `B1 = NOT AUTHORIZED`, so the owner registered this track as
**`U1_UI_PRODUCTIZATION_CONVERGENCE`** instead. Section numbers below cite the delivered prompt as
written; `§6` is this audit, `§8.n` the per-page targets.

Authority used, in the order the governance chain fixes (ADR-0018, `AGENTS.md` 11): red lines →
`assets/design-tokens.json` and the frozen component rules → `DESIGN.md` → the accepted seven-screen
reference set in `assets/ui-mockups/`. The mockups are direction authority for **structure, hierarchy and
density**; they do not outrank the token file, and they are frozen assets this round reads and does not
edit. Every one of the seven was opened and compared; the digests are in the run's evidence root.

Every claim in §A–§D carries a measured citation. Where a subagent's inventory and the tree disagreed,
the tree won and is cited: the initial inventory reported "Compare and Release use raw `<th>`", and
`grep -c` over the four table pages says otherwise — Compare uses the shared `SortHeader` 15 times,
Release 25 raw `<th>` and History 22, and both of those are **correct**, because neither page sorts
(`sort` appears once in `Release.tsx` in prose and zero times in `History.tsx`).

## A. Where the current UI already agrees with the references

| Convergence target | Already true, measured |
| --- | --- |
| Instrument-grade, light, hairline-separated calm | `styles/global.css:21-28` canvas `--fs-color-bg-canvas` + UI stack; `App.module.css:19-21` surface + single hairline; **zero `box-shadow` declarations in the whole UI tree** (grep over `apps/desktop/ui/src`) |
| Navigation rail as the spine | `App.tsx:112-121`, rail width is the frozen `--fs-layout-nav-rail` token (`App.module.css:11`) |
| Five states as icon + label, never colour alone | `components/StateBadge.tsx:16-29`; UNKNOWN is a hollow dashed circle in `--fs-color-status-unknown` (`:90-103`), N/A a hollow dash (`:104-109`) — the same glyph language the mockups' `○ UNKNOWN` and `– N/A` chips use |
| Mono for numbers, hashes, versions, timestamps, paths | `global.css:56-61` plus per-page `.mono`; `ArtifactDto.fileName / sha256` and `IdentityDto.snapshotId` are already carried as facts, not inferred |
| Error shape = what happened / why we know / what to do / diagnostics id | `components/ErrorPanel.tsx:22-50` implements exactly the four lines `FS-UI-06-Parse-Failure.png` shows |
| Change kind is tint **plus** a label | `Compare.module.css:260-295` `.kind/.kindAdded/.kindRemoved/.kindChanged`, and diff kinds are deliberately kept out of the gate state set |
| A right-hand evidence column exists | `Details.tsx:676-714` Inspector, with the 320–420 px governance range in `--fs-layout-evidence-inspector-*` |
| Local-first stated in-product | `Help.tsx:230` local-first section; `App.tsx` shell already owns the session facts the mockups' top bar would name |
| Conclusion-before-detail on the diff table | `Compare.tsx` already renders a summary section before the change tables (`:415` report, `:1053`/`:1294` change areas) |

The temperament is therefore not the gap. The gap is **product scaffolding**: the shell furniture, the
per-page header/action grammar, the verdict-first ordering, and the detail panel on three of five pages.

## B. Where the current UI is furthest from the references

1. **There is no Overview page.** The rail is exactly Analyze / Compare / Release / History + Help
   (`App.tsx:43-62`, `type Page = 'analyze' | 'compare' | 'release' | 'history' | 'help'`), while six of
   the seven mockups carry an `Overview` entry and `FS-UI-01-Overview.png` is a whole page. This is the
   single largest difference between "engineering-complete" and "product".
2. **No product bar.** Every mockup has a top bar: brand, `project · version` chip in mono, and the
   sentence "Local workspace — nothing leaves this machine". The shell has a rail brand and nothing else
   (`App.tsx:112-121`). The local-first promise is real today but lives one click away in Help.
3. **One primitive, five declarations.** Measured by `grep -l` over the CSS modules: `.page` in 6 files,
   `.header` in 5, `.mono` in 7, `.caption` in 4, `.primary` and `.stale` in 3, `.empty` in 3. There is no
   shared `PageHeader`, `Panel`, `ActionBar`, `Chip`, `EmptyState` or `SummaryCard` — `components/` holds
   four files (`StateBadge`, `Table`, `ErrorPanel`, `SizeUnitSwitch`). Pages therefore drift by
   construction, which is what §7.1 exists to stop.
4. **The primary action is not where the eye starts.** `FS-UI-01/02/03/04/05` all put one primary plus one
   secondary plus a link at the top-right of the page. `Analyze.tsx:144` puts its controls in a
   `section.controls` below the prose subhead; `Release.tsx` spreads actions across six numbered sections
   (`:320 :395 :510 :625 :827 :1147`).
5. **Release buries its verdict.** `Release.tsx:309` opens `div.page` (the only page that is not `main`)
   and reaches "3 · FirmwareSight policy readiness" at line 510, after policy and builds. The mockup's
   first object is a verdict band carrying `✓4 ⚠2 ×1 ○1 –1`, which is exactly `GateCountsDto`
   (`pass/review/block/unknown/notApplicable`) — the data is already in the DTO and already unused for a
   headline.
6. **No detail panel on Compare, Release or History.** `FS-UI-03` pairs the diff table with "Biggest growth
   contributors"; `FS-UI-04` pairs the rule list with "Bundle preview"; `FS-UI-05` pairs the history list
   with a bundle detail card. Today those three pages are single-column stacks.
7. **Ranked bars are absent although the tokens for them are frozen and unused.** `--fs-color-chart-blue-1`
   through `-5` and `--fs-color-chart-other` have zero references in the tree, and `DESIGN.md` §7 ranks
   `table > ranked bars` — the mockups' inline size bars are inside the system already.
8. **Chips have no pill.** `--fs-radius-pill: 999px` is unused; `StateBadge` renders inline text with no
   surface, so the mockups' chip vocabulary has no component to land in.
9. **A known layout defect class is still live on two pages.** `Compare.module.css:411-417` and
   `Release.module.css:431-437` set `display: block; overflow-x: auto` **on the `<table>` itself** — the
   mechanism Commit F2R-01 removed from Details, whose CSS comment now states the rule
   (`Details.module.css:126` "The scroll axis belongs to a wrapper, not to the table"; same sentence at
   `History.module.css:44`). At the frozen 1024×720 minimum this is the same S2/S3 class that produced
   F2's three findings.
10. **A third error shape exists.** `Details.tsx:717-728` defines a local `DetailError` instead of using
    `ErrorPanel`, so the product has two error marks and one prose standard.
11. **Empty states are per-page prose.** `.empty` in three CSS modules with different markup and no shared
    component; `FS-UI-06` shows what a first-class "nothing yet" state looks like.
12. **History has no selection concept.** `History.tsx:238/369/442` are three stacked tables with filters
    and pagers, no selected row, no detail area — while `FS-UI-05` is built entirely around
    list-plus-selected-detail.

## C. Which references can be implemented for real, on existing data

| Reference | Real backing that already exists |
| --- | --- |
| `FS-UI-01` Overview: ELF / MAP / Git cards | `CapabilitiesDto = { elf, sections, symbols, debugInfo, map, objectAttribution, git }` — capability states are already reported by Core and already rendered as words on Analyze |
| Overview "Can we ship now?" band with counts | `GateRunDto.counts` (`GateCountsDto`) plus `overallEffectiveSeverity` and `dispositionEffectiveSeverity`, both derived by Core; `AGENTS.md` 3 forbids the UI recomputing them, so the band displays and never decides |
| Overview summary cards (flash / largest section / symbols / last gate) | `MemorySummaryDto.nonvolatileImageFootprint` (ADR-0021 budget), `AnalysisSummaryDto.sectionCount / symbolCount`, `ArtifactDto.byteSize / sha256`, `GateRunDto.createdAt / runId`; `list_history_gate_runs` already returns stored runs |
| Next-step links on the Overview band | Every target is an existing page or an existing command (`attach_map`, `run_release_gate`, `select_artifact`) |
| `FS-UI-02` Analyze chips + sections/symbols + right detail | `Analyze.tsx:281-381` summary, `Details.tsx` tabs/tables/Inspector already exist; the change is composition, not data |
| `FS-UI-02` inline size bars | `SectionRowDto` / `SymbolRowDto` sizes plus the unused chart tokens |
| `FS-UI-03` summary line + contributors panel | `Compare.tsx:854-883` already computes a two-column ranking area from `ContributorDto`; it is placed mid-page rather than as the paired detail |
| `FS-UI-04` grouped rule list | `GateRunDto.findings` is documented as "Findings in canonical rule order; the screen groups them by state" — the grouping the mockup shows is the shape Core intends |
| `FS-UI-05` selected row + detail card | `HistoryGateRunRowDto` / `HistoryReleaseRowDto` / `CandidatePageDto` carry the fields the detail card shows; `BundlePreviewDto` supplies real bundle contents **only after** `prepare_release_bundle`, which is a Release-page action |
| `FS-UI-06` parse-failure card | `ErrorEnvelopeDto { code, message, details, remediation, operationId }` maps onto the mockup's four labelled rows one for one |

## D. Which references are visual pattern only, and why

| Reference element | Disposition | Reason |
| --- | --- | --- |
| `FS-UI-07` Dependencies tab, dependency table, "Declare component version" dialog, "Re-scan" | **`NOT_IMPLEMENTED_IN_U1`** | There is no dependency detection, no declaration command and no rescan command: 27 Tauri commands are registered in `ipc/bridge.ts:50-77` and none of them is dependency-related; a grep for `dependenc / declare_version / rescan` across `crates/`, `src-tauri/src` and `ui/src` returns only prose about dependency-free crates. Shipping the mockup's table would mean inventing its rows, which §2C and §14.2 forbid, and the storage to keep a declaration would be a schema change (§14.1) |
| Git card action "Link repository…" | `DESIGN_REFERENCE_ONLY` | `GateGitDto` and `CapabilitiesDto.git` report git facts; no command links a repository. The card will state the real capability and offer no action |
| "Mark N/A" on an UNKNOWN gate rule | `DESIGN_REFERENCE_ONLY` | `accept_review` exists for REVIEW dispositions; there is no manual N/A command, and `DESIGN.md` §6 allows manual N/A only where a rule permits it. The UNKNOWN row keeps its real next step (attach the MAP) |
| "New bundle…" and "Open bundle folder" on the History header | `DESIGN_REFERENCE_ONLY` | `prepare_release_bundle` / `choose_bundle_destination` / `export_release_bundle` belong to the Release flow, which owns policy, gate run and confirmation. Duplicating them on History would be a second copy of a destructive-adjacent flow for presentation's sake |
| "Recent activity" rail item | `VISUAL_PATTERN_ADOPTED` as a group label only | The History page *is* recent activity; a second entry pointing at the same data with a different name would be a fake surface |
| Theme-toggle glyph in the mockups' top bar | **`TREATMENT_NOT_ADOPTED`** | `DESIGN.md` §4 and §9: light theme is the only MVP theme |
| Card elevation in the mockups | **`TREATMENT_NOT_ADOPTED`** | `DESIGN.md` §9 forbids card/panel/table-row shadows and the tree contains zero `box-shadow`. Hierarchy is built from hairline, `bg-surface` vs `bg-subtle`, alignment and type — which is what the mockups mostly do anyway; their faint elevation is the one thing not portable |
| Preselected "Compare with…" baseline from Overview | `VISUAL_PATTERN_ADOPTED` (navigation only) | The button can navigate to Compare; carrying a preselected baseline across pages would be a new cross-page contract, and Compare's own pickers are the authority for that choice |
| Mockup data (`relay-controller`, `v0.3.0-rc2`, `FirmwareSight 0.1.0-alpha`, person names, timestamps, hashes) | `DESIGN_REFERENCE_ONLY`, never copied | `03_DESIGN/06_UI_REFERENCE_SCREENS.md` "Mock data disclaimer": those values are visual examples, not protocol. Real screens show `AppIdentityDto`, `ArtifactDto`, `ProjectContextDto` and `GateRunDto` facts |

## E. Convergence priority

1. Shared layer: `PageHeader` (title + metadata + action bar), `Panel`, `Chip`, `StatusStrip`,
   `EmptyState`, `SummaryCard`, `RankBar`, `DetailPanel`, `ActionBar` buttons — all values from tokens,
   the two unused families (`radius-pill`, `chart-blue-*`) put to work.
2. Shell: product bar with the real project/version chip and the local-first sentence; rail grouping that
   matches the mockups' two groups.
3. `FS-UI-01` Overview as a fifth page, composed only from `AnalysisSummaryDto`, `GateRunDto` and the
   history reads, with honest "nothing analyzed yet in this session" states.
4. `FS-UI-02` Analyze: chip row, sections/symbols cards with rank bars, Inspector as the paired detail,
   `ErrorPanel` everywhere (retire `DetailError`), `.viewport` already correct here.
5. `FS-UI-04` Release Gate: `main.page`, verdict band with `GateCountsDto`, findings grouped
   BLOCK → REVIEW → UNKNOWN → PASS → N/A, bundle preview as the paired detail.
6. `FS-UI-03` Compare: summary band with gate chips, contributors as a right detail panel, and the
   `.viewport` wrapper replacing `display: block` on its tables.
7. `FS-UI-05` Bundle & History: header metadata + action bar, selectable rows with a right detail card,
   shared empty/error/loading states, `.viewport` on the Release tables too.

## F. Explicitly not doing in this round

- No new Tauri command, no Core change, no storage, schema or migration change, no wire or golden change,
  no `analysis:2`, no contract-version bump. `urn:firmwaresight:schema:diff:1` and the four other public
  compatibility tokens stay exactly where they are.
- No edit to `assets/design-tokens.json`, to the generated `styles/tokens.css`
  (its header names `scripts/generate_design_tokens.py` as the only writer), or to the seven frozen
  mockups. A value with no token is a reason not to draw the thing, or a decision for the owner — not a
  magic number.
- No dependency detection, no version declaration, no repository linking, no rescan, no "recent activity"
  data source, no bundle actions on History — see §D.
- No gate-rule semantics, severity aggregation, evidence classification or release-identity change: the UI
  displays what Core decided, in the order Core decided it (`AGENTS.md` 3, 8).
- No dark theme, no gradient, no glow, no card shadow, no marketing hero, no colour-only status.
- No V1, B1, RC or GA state. V1 keeps `IN_PROGRESS / RECRUITMENT_READY` with 0 sessions and gains a
  `paused_for` field; `B1 Private Beta` remains unauthorized and untouched.
- No claim that the result equals the mockups. The permitted sentence is the delivered §15 one: the UI has
  converged from an engineering-complete MVP candidate toward the intended desktop product interface.
