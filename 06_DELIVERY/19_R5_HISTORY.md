---
title: "R5 — History with the pinned baseline, Settings, Help and the command palette"
doc_id: "FS-DEL-020"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE"
owner: "Product / Delivery"
last_updated: "2026-10-17"
---

# R5 — History with the pinned baseline, Settings, Help and the command palette

**Authorization.** The Owner set a standing goal in chat on 2026-10-17: carry the plan through to the complete product without stopping for intermediate confirmations ("按照计划一直推进直到完成完整的产品，中间无需我们介入确定，一切由你来定夺").

**Base.** R5 is stacked on the R4 branch (`feature/r4-release`, `2a81d4b`). R4 is complete locally, but its push to `main` was refused by the session's permission guard and is waiting on the Owner (§6).

**Design sources.**
- [03_DESIGN/09](../03_DESIGN/09_R_REDESIGN_SPEC.md) §3.2, §3.3, §6.5 and §6.6.
- [03_DESIGN/08](../03_DESIGN/08_POST_MVP_PRESENTATION_RULES.md) §8 (a pin is a measurement reference).
- [ADR-0034](../09_ADR/ADR-0034-instrument-ledger-design-language.md) D7, which folds D3 into this phase under the Owner's written exception (`.ai/DECISIONS.md` RD decisions).

## 1. What changes for the reader

1. **Pinned baseline (D3).**
   - **History:** the page names the build the project measures against and when it was pinned, with **Unpin** and **Compare with the current build**. Every build row can be pinned. Pinning another build replaces the pin, and unpinning leaves a record.
   - **Compare:** the pin is the default base. The pickers mark the pinned build, the SubjectBar marks it, and **Pin base as baseline** pins the base on screen.
2. **History becomes a workbench.**
   - Its three populations are Tabs (Builds · Gate runs · Release records).
   - Each list is the shared `DataTable` with the shared `FilterBar` in apply mode and the shared pager. A row opens its full record in the Inspector, replacing the old expanding detail rows.
   - The Builds list can start a comparison: **Compare with the current build** on a row, or **Compare the two selected** after ticking two rows. Either one opens Compare and runs it once.
3. **Settings.** Reset asks in a Sheet before it writes. The size unit keeps its own group.
4. **Help.** A keyboard-shortcut table.
5. **Command palette and shortcuts** (spec §3.3).
   - `Ctrl+K` opens a palette (combobox + listbox). It covers:
     - go to any page;
     - choose firmware;
     - compare with the pinned baseline;
     - switch the size unit, the theme and the language.
   - `Ctrl+O` opens the firmware chooser.
   - `/` focuses the filter on screen.
   - `[` and `]` move between the views of the page.
   - `Esc` closes the palette and returns focus.
   - Single-key shortcuts are ignored while typing.
6. **Navigation names (spec §3.2).** "Release Gate" becomes **Release** and "Bundle & History" becomes **History**, in the rail, the page titles and the palette. The window-title words are the Rust enum's and do not change.

## 2. Decisions

| # | Decision |
|---|---|
| D-R5-1 | Migration 0008 adds `baseline_pin_events`, an append-only log per project with `seq` order. Edits and deletes are refused by trigger, a `CHECK` ties `pin`↔build, and only a `COMPLETE` build of the same project can be pinned. The current pin is the newest row when it is a `pin`. `SCHEMA_VERSION` 7 → 8. This is additive; no existing row or table changes. |
| D-R5-2 | Three use-case commands: `get_baseline_pin`, `pin_baseline(snapshotId)` and `unpin_baseline`. No project and no path crosses. They live in `src-tauri/src/baseline.rs`, so `history.rs` stays read-only. |
| D-R5-3 | The pin is held in `App`, read once at start and moved only by a pin or unpin answer. Compare uses it for the default pair (base = pin, target = the build Analyze last proved, or the newest other). A pair the reader already named is never re-pointed. |
| D-R5-4 | Starting a comparison from History or the palette sets the pair in the shared Compare session and asks Compare to run it once (`autoCompare`), the same way a drop asks Analyze to analyze once. |
| D-R5-5 | History's tables do not sort: the shell orders them (newest first) and offers no sort parameter. Filtering and paging stay the shell's. |
| D-R5-6 | The palette runs only commands the shell already offers through existing page actions. It adds no capability and no new IPC. |
| D-R5-7 | No new i18n namespace. New words go into `common`, `history`, `settings`, `help` and `compare`; dead keys are removed in the same change. |

## 3. Accessible-name and structure mapping (as landed)

| Before | After | Tests changed |
|---|---|---|
| rail "Release Gate" / "Bundle & History"; h1 "Release Gate" / "Bundle & History"; Overview and Help sentences naming them | "Release" / "History" everywhere (zh 发布 / 历史) | renamed throughout the suites |
| History view switch: button group "Stored history" with `aria-pressed` | tablist "Stored history" (Tabs) | `showView()` uses tabs; the hierarchy test reads `aria-selected` |
| hand-written History tables in `ScrollArea` ("viewport") | `DataTable` with `actionFirst` ("wrap") | layout test |
| row button "Details" expanding a detail row | row action "Show the stored record of {name}", first in the row, opening the Inspector | `rowAction()` / `openFirstRow()` |
| filter `textbox` with a "Filtering by …" hint | `FilterBar` in apply mode: a `searchbox` with Apply | `searchbox`; the hint assertion dropped |
| (none) | region "Pinned baseline" (Unpin, Compare with the current build); row buttons "Pin {name} as baseline" and "Compare {name} with the current build"; checkboxes "Select {name} for comparison"; "Compare the two selected (n of 2)" | new R5 tests |
| Compare pickers | the pinned build is marked "Pinned baseline" and is the default base; button "Pin “Old / Base” as the project baseline" | new R5 tests |
| Settings reset: inline group "Reset all four settings to their defaults?" | dialog with the same name (Sheet), focus on "Reset preferences" | `group` → `dialog` |
| (none) | TopBar button "Commands (Ctrl+K)"; dialog "Command palette" with combobox "Search commands" and listbox "Commands" | new R5 tests |
| (none) | Help region "Keyboard shortcuts" | new Help test |

Behaviour that is not a name:
- The History Inspector closes when the view or the unit changes (the R3 rule).
- A comparison started from History runs once. It does not run again on a later visit to Compare; a test that leaves Compare and returns pins this.

New tests:
- Storage: `baseline_pins.rs` (7 tests).
- Shell: `baseline_pin.rs` (2 tests).
- UI:
  - History: 5 pin and compare tests, and 6 palette and shortcut tests;
  - Compare: 2 tests;
  - Help: 1 test.

UI tests: 526/526 after the review fixes (§6). Three tests were mutation-checked: the pinned default base, the auto-compare-once rule, and the typing guard on single-key shortcuts.

The existing migration-list and schema-version assertions now name 0008 / version 8 (storage `compare_candidates`, `map_companion_persistence`, `release_records`, `integrity_and_backup`, `unknown_reasons`, `object_contributions`; shell `real_artifact_intake`, `diagnostics`). Each step-down helper also drops the new table.

## 4. Verification

- `python scripts/check.py` must pass every step.
- Storage, shell and UI tests must cover:
  - the pin log;
  - the commands;
  - History's pin and compare actions;
  - Compare's default base;
  - the palette and its shortcuts;
  - the reset Sheet;
  - the renamed navigation.
- Native smoke on the QA build. The run creates no Owner data, and its QA stores are kept and listed.
- Then an independent review, fixes, and the push request to the Owner.

## 5. Native smoke (2026-10-17)

The final QA build passed 13 checks; 1 is NOT_RUN (the human drag check). See [evidence/R5](../evidence/R5/README.md).

The first run found three visual defects, now fixed: values wrapping mid-value, row buttons repeating the build name, and the palette's width. For the palette width the token file has no dialog size, so it is the evidence-column maximum plus the rail width; recorded for R7's token review.

## 6. Independent review (2026-10-17)

An independent review of `66ec11a` returned **PASS_WITH_FIXES**, with no S1 findings. Its read-only checks were all green and showed no ts-rs binding drift:
- storage and shell tests;
- UI tests 522/522;
- type check and lint.

| Finding | Disposition |
|---|---|
| S2-1: a failed pin read was shown as "No baseline is pinned" | Fixed. The read failure is its own state: History shows an Unknown sentence, the typed error and **Read the pin again**, and the palette names the reason. Tested, including the retry. |
| S2-2: a pin not on the first candidate page was dropped as the default base, and the picker went blank | Fixed. The pin is the default base by id. The pickers also list the pin, and the two sides of a computed comparison, when they are off the loaded page, named by file and digest with a note. "Both chosen" now reads the ids. Tested with an off-page pin. |
| S2-3: `Ctrl+K` / `Ctrl+O` acted over an open modal | Fixed. No shortcut acts while any modal is open. Tested with the reset Sheet open. |
| S3: a late startup read could overwrite a newer pin | Fixed (a version counter). Tested. |
| S3: palette keys were lost after a click inside it; the active row did not scroll | Fixed. Keys are heard on the dialog, a press outside the input keeps focus in it, and the active option scrolls into view. |
| S3: the pin read-then-append was not atomic | Fixed. `BEGIN IMMEDIATE` around each. |
| S3: project scope was enforced only in code | Fixed. A `BEFORE INSERT` trigger in migration 0008 (not yet released). Tested. |
| S3: the pin time was not mono | Fixed. |
| S3: the "compare the two selected" test did not prove the order rule | Fixed. It ticks the older build first. |
| S3: comment rot (App doc comment, renamed-page comments, schema messages) | Fixed. |
| S3: the History privacy note was lost | Restored as an Inspector detail. |

**Found while fixing.** A flake in the full UI run exposed a real race on Release: R4's clearing effect for the policy draft could still be pending when **Edit policy** was pressed, wipe the new draft and close its Sheet. The draft now carries the identity of the project it was seeded from and is derived in render, so no effect is involved.
