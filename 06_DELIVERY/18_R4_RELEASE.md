---
title: "R4 — Release workbench"
doc_id: "FS-DEL-019"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE"
owner: "Product / Delivery"
last_updated: "2026-10-17"
---

# R4 — Release workbench

The Owner authorized R4 in chat on 2026-10-17 ("执行R4"), after R3 was integrated at `465fe5f` (Actions #125, 10/10). The base is `465fe5f`.

Design sources:
- [03_DESIGN/09](../03_DESIGN/09_R_REDESIGN_SPEC.md) §6.4;
- the accepted [R0 prototype](../03_DESIGN/prototype/README.md);
- the same page anatomy R2 gave Analyze and R3 gave Compare.

Two rules from earlier rounds carry over unchanged:
- **The Gate decides; the page words it.** Every state, count, aggregate, margin and delta is the shell's. An accepted review stays REVIEW, and only the aggregate moves.
- **A verdict is policy readiness, nothing more.** No sentence on the page reaches past the project policy into safety, compliance or certification.

## 1. What changes for the reader

**Release** gets the anatomy of Analyze and Compare.

1. **Header.** The header carries **Export ▾** with PNG, SVG and print-ready HTML. These are the Gate visual exports that used to be their own section. The menu appears once a run exists.
2. **SubjectBar.** It names the build the run on screen judged, by file and short hash, and whether the run is current:
   - `Latest`;
   - "previous result kept" when the setup now names a different build or baseline;
   - "failed" when the newest attempt failed and the last good run is kept.

   The unit switch sits in the bar.
3. **LeadAnswer.** The verdict: the disposition badge, the verdict sentence Core's counts produce, the five-state strip, and the standing note that this is FirmwareSight policy readiness and not a certification. It keeps the verdict focus rule: after a run the reader asked for, focus lands on the verdict.
4. **Run settings.**
   - **With no run**, the settings are the page body: three numbered steps (1 Build and baseline, 2 Project policy, 3 Release attachments) and one **Run Gate** bar after the last step.
   - **With a run**, they collapse to one summary line: build · baseline · policy hash · attachment count, then **Edit settings** and **Run Gate**. **Edit settings** opens the same three steps in place. The steps stay mounted while collapsed, so the attachment set, the readiness check and a policy draft are not lost.
5. **The policy editor moves into a Sheet.** Step 2 shows the loaded config and its readiness. **Edit policy** opens the form in a modal Sheet, with Save and Close in the Sheet. The form, its fields and its "unset means the shell's default" rule are unchanged.
6. **Three Tabs** (Release views):
   - **Findings.** Groups BLOCK → REVIEW → UNKNOWN → PASS → N/A. PASS and N/A start collapsed, with their counts shown. Each row shows rule, state, effective severity, summary and next step.
     - **Show basis** opens the Inspector with the finding's evidence locators and severity.
     - **Accept review** opens a Sheet for name and reason. The recorded acceptance stays inline on the row.
   - **Evidence.** The run record (disposition and as-computed aggregates, run, build, baseline, policy, stored time, workspace, warnings, Read the stored record), then the required-artifact, budget and growth tables on the shared `DataTable`, then the release notes. A budget or growth row opens its full basis in the Inspector.
   - **Bundle.** Prepare → destination → export → verified result, as before. The replace decision moves into a Sheet. The bundle stays disabled with its reason unless the disposition is PASS.
7. **BuildPicker.** Step 1 uses the `BuildPicker` Compare uses. The baseline picker gains a **No baseline** option.

## 2. Decisions

| # | Decision |
|---|---|
| D-R4-1 | No change to Core, IPC, storage, the Gate or the bundle engine. Every request the page sends is the request it sent before R4. |
| D-R4-2 | A new shared `Sheet` component, always modal. It has a dialog role, a focus trap, Esc to close, a scrim click to close, focus returned to the opener, and an optional initial-focus target. It is used for the policy editor, review acceptance and the bundle replace decision. The Inspector stays the evidence surface and is not a Sheet. |
| D-R4-3 | `BuildPicker` gains an optional "none" option (`noneLabel`). The baseline picker uses it. |
| D-R4-4 | `SubjectBuild.map` becomes optional: Release omits it, because a stored candidate does not report its MAP and "MAP unknown" would be a claim the page cannot back. |
| D-R4-5 | `LeadAnswer` gains an optional accessible `label` and a `focusRef`, so the verdict keeps its name ("Gate verdict") and its focus target. |
| D-R4-6 | The Gate export stale guard gets the CI #123 treatment: a layout-effect counter on the run id, and ERR-EXPORT-6002 as the stale note on every path. |
| D-R4-7 | The page seeds its pickers from the run it is given when there is one. Otherwise, returning to Release with a run would show "previous result kept" for a run the reader never moved away from. |
| D-R4-8 | No new i18n namespace (ADR-0032 D4). New words go into `release` and `gate`; dead keys go in the same change. |
| D-R4-10 | The verdict sentence counts the findings whose effective severity is the verdict's word, and how many of those are UNKNOWN. Before this it counted REVIEW- or BLOCK-state rows only, which read "0 findings wait" under a REVIEW disposition carried by unknown evidence (found by the smoke run). The count uses Core's own `effectiveSeverity` field on each row; nothing is re-judged. |
| D-R4-9 | The rail names ("Release Gate", "Bundle & History") stay. The IA rename belongs to R5 with History. |

## 3. Accessible-name and structure mapping (as landed)

| Before | After | Tests changed |
|---|---|---|
| verdict band, region "Gate verdict" | `LeadAnswer` with the same name and focus target, plus the policy-readiness note | none |
| one long page: verdict, findings, readiness, evidence, visual export, bundle, then the three steps and a run bar | SubjectBar "Subject of this page" → verdict → region "Run settings" (one line, **Edit settings** / **Run Gate**) → tablist "Release views" (Findings · Evidence · Bundle) | hierarchy test restated; `openView()` helper |
| steps open in a quieter surface after a run | steps folded into the settings line; **Edit settings** (`aria-expanded`) opens them in place; they stay mounted | "folds the settings into one line" replaces "keeps both configuration sections open" |
| region "Standalone visual report" with three buttons | group "Standalone visual report" with an **Export** disclosure in the header | `exportItem()` helper |
| native `select` "Current build" / "Baseline (optional)", option "No baseline" | `BuildPicker` comboboxes with the same names; "No baseline" is the list's first entry | `pickerValue()` / `choose()` helpers |
| inline policy form under **Edit policy** / **Close policy editor** | dialog "Edit project policy" with Save in its foot; its close button keeps the name "Close policy editor" | none |
| inline acceptance form | dialog "Accept the review of {rule}", focus on the name field | none |
| replace card, group "Replace the existing bundle?" | dialog with the same name, focus on Replace | role `group` → `dialog` |
| evidence locators as a list on each finding ("Evidence for {rule}") | **Show the basis for {rule}** opens them in the Inspector | the finding-row test opens the Inspector |
| all five groups open | PASS and N/A start collapsed: **Show {n} {group} finding(s)** / **Hide the {group} findings** | `expandAll()` before reading PASS rows |
| run record, both aggregates and the evidence tables after the findings | in the Evidence view | those tests call `openView('Evidence')` |
| bundle section after the evidence | the Bundle view (kept mounted while hidden, so a plan survives a view switch; a new run still drops it) | bundle tests call `openView('Bundle')` |
| hand-written tables in `ScrollArea` ("viewport") | `DataTable` ("wrap"); budget and growth rows open the Inspector | layout-contract test |
| growth delta `+512 bytes` | neutral `▲512 bytes`, spoken "increase of 512 bytes"; an unmeasured delta is "Unknown", never "not present" | growth test |
| "previous result" paragraphs | SubjectBar words "Previous run kept · current settings not judged" and "Gate run failed · previous run kept", each with a stale Notice | new tests |
| a record without tables showed nothing | Unknown Notice "Evidence tables not stored" | stored-record test |

Dead keys removed: `release.visualExport.heading`, `release.builds.chooseOption`, `release.attachments.table`, `release.bundle.fileList`, `gate.findings.evidenceList`, `gate.evidence.artifactsTable`, `gate.evidence.budgetsTable`, `gate.evidence.growthTable`.

New tests:
- `release.test.tsx` "R4 Release workbench" (11 tests). Four of them were mutation-checked: each failed with the code it guards broken.
  - D-R4-7 seeding;
  - an unmeasured growth delta as Unknown;
  - an Unknown finding classed Unknown;
  - PASS and N/A collapsed.
- `foundation.test.tsx` "Sheet (R4)" (2 tests) and "BuildPicker none entry (R4)" (1 test).

UI tests: 500/500.

## 4. Verification

- `python scripts/check.py` must pass every step.
- New tests cover:
  - the Sheet (trap, Esc, return focus, initial focus);
  - the settings collapse and its summary;
  - PASS and N/A collapsed with their counts;
  - Show basis opening the Inspector;
  - acceptance in a Sheet;
  - the replace decision in a Sheet;
  - the SubjectBar's retained and failed words;
  - the export stale guard in the same tick.
- Native smoke on the QA-identifier build:
  - analyze a fixture and gate it with and without a baseline;
  - the lead, the three tabs, Show basis, a budget row in the Inspector;
  - the policy Sheet open and Esc;
  - the collapsed settings and Edit settings;
  - export cancel;
  - zh and en, light and dark, 1440 and 1100 px.

  The evidence and the design checklist go in `evidence/R4/`. QA stores are kept and listed, not deleted.
- Then an independent review in its own session and worktree, fixes, an Owner-approved fast-forward push, CI, and cleanup.

## 5. Native smoke (2026-10-17)

The final QA build passed 14 checks; 1 is NOT_RUN (the human drag check). See [evidence/R4](../evidence/R4/README.md) and the [design checklist](../evidence/R4/DESIGN_CHECKLIST.md).

The first run found one defect, now fixed: the verdict sentence's count (D-R4-10). The QA stores both runs created are kept and listed in the evidence README.
