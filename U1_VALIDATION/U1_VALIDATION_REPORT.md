---
title: "U1 Validation Report"
doc_id: "FS-U1-003"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Design"
last_updated: "2026-10-07"
---

# U1 — UI Productization / Design Convergence: Validation Report

Delivered §12 deliverable D, and the round's stop condition (§16 step 8): this document ends the round. Track
token `U1_UI_PRODUCTIZATION_CONVERGENCE`; the delivered prompt calls itself `B1`, which the owner re-registered
because `B1` is Private Beta (`06_DELIVERY/06_STAGE_GATES.md`) and both F3 §37 and V1 §40 record it as NOT
AUTHORIZED. Companions: `U1_UI_GAP_AUDIT.md` (FS-U1-001, §6) and `U1_DESIGN_CONVERGENCE_PLAN.md` (FS-U1-002, §12 B).

Every number below was printed by the command named beside it, run in this session on 2026-10-07. Nothing here
is recalled from an earlier round's report, and nothing here claims a run that did not happen.

## 1. Start authority (§2A)

Measured before the first file was written and re-measured at close:

```text
git rev-parse HEAD            f481b78059c14e1c83d3ba18e082004b7de72ee2
git rev-parse origin/main     f481b78059c14e1c83d3ba18e082004b7de72ee2
git ls-remote origin main     f481b78059c14e1c83d3ba18e082004b7de72ee2
git status --short            (empty at preflight)
git worktree list             one worktree, [main]
gh run list                   37508243514 #73 completed success, head f481b78
gh run view 37508243514       10 jobs
```

HEAD equals origin/main, the tree was clean, there was one worktree and no remote movement to classify, and the
run standing on that head is 10 of 10. The full record is `00_authority/PREFLIGHT_U1.txt`.

The delivered §0 said the incoming state is `active_task = NONE`. It was not: V1 had activated the pointer at
`f481b78`. That difference was reported rather than smoothed over, and the owner resolved it — pause V1 with its
state intact and activate U1. Both resolutions are in `.ai/DECISIONS.md` and `BASELINE.yaml`.

## 2. What converged, page by page (§8, §11 per-page report)

The reference screens are `assets/ui-mockups/FS-UI-01..07`; the seven were opened and compared in §B of the gap
audit. Dispositions below use the labels the audit fixes.

**Shell.** The product bar is new (`components/TopBar.tsx`): brand, the loaded project, and the local-first
sentence that previously lived one click away in `Help.tsx`. The rail gained `Overview` as its first entry, so
its five entries are the five built stages, and the brand moved out of the rail into the bar. The bar prints no
artifact name and no application version, both recorded in §D of the audit: the first would duplicate the page
that owns the file name, the second would cost a second `get_app_identity` read every session.

**Overview (FS-UI-01).** A new page, built only from DTOs the shell already returns — the last analysis, the
latest gate run, the loaded project, `CapabilitiesDto`, `GateCountsDto`. It adds no command, computes no
severity (`stateWords.ts` maps Core's words to the five states and nothing else), and every one of its actions
is navigation to the page that can actually perform the thing. The reference screen's "Re-run Release Gate"
button executes the gate; this page's equivalent navigates, because running the gate belongs to Release, where
the policy, the build and the baseline are chosen and an overwrite is confirmed.

**Analyze (FS-UI-02).** Header, metadata line and action row are now `PageHeader`; the three input capability
chips are one row; the report body is `PageSection` + `FactList`/`FactRow`/`Figure`/`Qualifier`; the local
copies of `capabilityState`, `Section` and `Row` are deleted, as are 128 lines of page CSS that the shared
components already express. `EmptyState` replaced the page's own first-run paragraph.

**Compare (FS-UI-03).** `Page` + `PageHeader`, the shared `Button` for all five controls, `EmptyState` for the
one-build case, and both change tables now sit inside `ScrollArea`. The last item is a defect fix, not a style
choice — see §4.

**Release Gate (FS-UI-04).** The verdict moved to the top of the page as a band (`Verdict`), which is §B.5's
biggest gap: it used to arrive at section 3, after a policy editor and a build picker. The band states Core's
disposition, `GateCountsDto` as a `StatusStrip`, and the sentence that names both aggregates when accepted
reviews move them. That sentence is now `gateVerdictSentence` in `stateWords.ts`, shared with Overview, because
the same run must not read two ways on two pages. `div.page` became `main`; 13 raw buttons became `Button`; the
four tables became `ScrollArea`. The findings were already grouped BLOCK → REVIEW → UNKNOWN → PASS → N/A before
this round and were left alone.

**Bundle & History (FS-UI-05).** `Page` + `PageHeader`, and the three stored tables now use the shared
`ScrollArea` instead of two page-local copies of the rule. The row-level `Details` disclosure F2R-02 installed
stays exactly as it was, and its compact `.action` control stays local — see §6.

**Details (the inspector under Analyze).** Contributor rows became `RankBar`, which is §7's chart order applied
honestly: the number is the reading and the bar is the ranking aid, and an unknown magnitude draws no bar. Its
three tables moved onto `ScrollArea`, and the four CSS rules that only existed for the old contributor markup
were deleted.

**FS-UI-06/07 and the reference-only items.** The dependency / declare / re-scan screen has no backend: the
desktop surface is 30 use-case commands (counted from `generate_handler!` in
`apps/desktop/src-tauri/src/lib.rs`; this sentence said 27 when it was first written, which was wrong and is
corrected in `U1_VISUAL_ACCEPTANCE_REPORT.md` §2) and none of them reads a package, writes a source or re-scans
a project, so its actions are `NOT_IMPLEMENTED_IN_U1`. `Link repository`, `Mark N/A`, `New bundle` / `Open folder` and
`Recent activity` are `DESIGN_REFERENCE_ONLY` — real-looking controls whose backing does not exist yet, so they
were not added. The mockups' `relay-controller · v0.3.0-rc2` values are declared sample data by
`03_DESIGN/06_UI_REFERENCE_SCREENS.md` and nothing prints them. The theme toggle and card elevation are
`TREATMENT_NOT_ADOPTED`: `DESIGN.md` freezes a light MVP and AGENTS.md §11 forbids panel, table and card
shadows outright.

## 3. Design-system conformance, measured

```text
grep -rn "box-shadow|linear-gradient|radial-gradient|backdrop-filter|text-shadow" apps/desktop/ui/src --include=*.css   -> 0
grep -rniE "#(6366|7c3a|8b5c|a855|9333)" apps/desktop/ui/src --include=*.css                                            -> 0
token audit (python over every *.css except styles/tokens.css): 55 distinct --fs-* tokens used, 0 used but undefined
```

`assets/design-tokens.json` and the generated `styles/tokens.css` are **not in the changed-file list**:
`git status --porcelain` against those two paths returns nothing, and `drift/design tokens`
(`generate_design_tokens.py --check`) passes. No parallel token system was created (§9.3): the shared components
read the frozen file, and `--fs-radius-pill` — defined and unused before this round — is now used by the chip
badge and `Chip`.

Eighteen defined tokens are still unused, sixteen of them pre-existing (shadows, modal radius, chart blues 1–3
and 5, `--fs-color-status-review`, the desktop min-size pair, and others). Two became unused *by this round's
decision*: `--fs-layout-evidence-inspector-default` and `--fs-row-max` were to bound the reference screens'
detail column, which this round deleted rather than ship unpopulated (§6).

## 4. The defect class F2R-01 closed on Analyze, closed here too

F2R-01 found the Analyze sections table rendering one character per line because `display: block` was set on the
`<table>` itself: the box is pinned to the pane's width and its columns are then laid out *below their own
minimum*. §B.9 measured the same shape live in two more pages when U1 started:

```text
Compare.module.css:411  .table { display: block; overflow-x: auto }   with .wrap cells that may break
Release.module.css:431  .table { display: block; overflow-x: auto }   with .wrap reason/basis cells
```

Both are now `display: table` inside the shared `ScrollArea`, and both prose columns carry the same `min-width:
24ch` measure F2R-01 pinned. Details and History already owned a local `.viewport`; all six of those wrappers
now use `ScrollArea` as well, so the rule exists once (`components/Layout.module.css`) instead of four times.

The regression is pinned the way F2R-01 pinned it — structurally, because jsdom cannot measure a clip:
`compare.test.tsx` and `release.test.tsx` each gained `the table layout contract`, which asserts that **every**
table on the page has a parent whose class contains `viewport` and that the element is still a `TABLE`. The
assertion is universal rather than counted, so a third table added later must obey it too. It could not have
passed before the change: no `viewport` element existed on either page.

## 5. Validation results (§11)

| Check | Command | Result |
| --- | --- | --- |
| Rust formatting | `cargo fmt --all -- --check` | clean, no diff |
| Rust lints | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| Rust tests | `cargo test --workspace`, summed `test result:` lines | **868 passed, 0 failed** |
| Desktop shell | `cargo test -p firmwaresight-desktop --lib` | 96 passed |
| Frontend types | `pnpm typecheck` (`tsc --noEmit`) | clean |
| Frontend lint | `pnpm lint` (`eslint .`) | clean |
| Frontend tests | `pnpm exec vitest run` | **236 passed, 0 failed in 9 files** |
| Frontend build | `pnpm build` | through `vite build` |
| Authoritative gate (pre-staging) | `python scripts/check.py` | 14 of 17 steps reached, 13 passed, 1 failed: `drift/ipc bindings unchanged`. That step is `git diff --exit-code -- apps/desktop/ui/src/ipc/generated`, so it fails for the ts-rs file `cargo test` had regenerated but not yet staged. It is a staging artifact, not a product failure. |
| Authoritative gate (closeout) | `python scripts/check.py` at the fully staged state | **17/17 steps passed**, 0 skipped: rust 3, frontend 5, drift 8 (including `drift/baseline integrity`), deny 1. |
| Baseline artifacts | `generate_baseline_artifacts.py tree` then `sums` (both redirected, both staged), then `python scripts/verify_baseline_artifacts.py` | `RESULT PASS`: 750 tracked paths, 748 manifest entries, `index blob mismatch` 0, `tracked but unlisted` 0, `listed but unindexed` 0, `duplicate entries` 0, `unmerged index entries` 0, `artifact worktree != blob` 0. The digests are of canonical stage-0 index blobs per ADR-0029, not of working-directory bytes. |

**Count deltas.** Rust 868 → 868 (the window-title test changed its expectations, not its count). UI 225 → 236
in 8 → 9 files: +9 in the new `overview.test.tsx`, +1 in `compare.test.tsx`, +1 in `release.test.tsx`. No test
was deleted. The gate shape (17 steps) is unchanged.

**Authority surfaces that did not move.** `AGENTS.md` was not edited. No dependency, feature flag, schema,
migration, storage contract, analysis wire field, release identity rule, bundle manifest, diagnostic, ADR,
`DESIGN.md`, `tauri.conf.json`, capability file or `.gitattributes` was touched. `crates/**` and `scripts/**` are
not in the changed-file list. `git status --porcelain -uall` counts **23 new files** — 18 under
`apps/desktop/ui/src` (seven shared components as tsx + module pairs, `Overview.tsx` + `Overview.module.css`,
`stateWords.ts`, `overview.test.tsx`), **1,905 lines** between them, and five documents under `U1_VALIDATION/`
including the two copied into `00_authority/` at close. Thirty-two tracked files are modified, and the staged
diff carries 34 modified paths once `DIRECTORY_TREE.txt` and `SHA256SUMS` are regenerated on top of them; the
page CSS modules lost 373 lines and gained 85 as their duplicated rules moved into the shared layer.

**Test assertions this round rewrote, and why.** Five pre-existing files changed, and each edit is a consequence
of the convergence rather than a relaxed expectation:

- `history.test.tsx` — the rail list is five entries now; the header assertion follows `PageHeader`, which puts
  the subhead beside the heading block instead of inside it, so the test reads `closest('header')` rather than
  `parentElement`.
- `compare.test.tsx`, `intake.test.tsx` — the same rail list, five entries.
- `release.test.tsx` — the rail's own name (`Release Gate page`), the project name now appears in the product bar
  as well as the page so the lookup is `findAllByText`, the export assertion widened from `/bundle/i` to
  `/build|export|prepare/i` because the button words are `Prepare bundle` / `Export bundle` and the earlier regex
  matched a rail label rather than an action, plus the new layout contract.
- `help.test.tsx` — one identity test had counted `get_app_identity` calls; the bar prints no version, so the
  page still makes exactly one read.
- `intake.test.tsx` — the forbidden-words screen narrowed its list to `Settings, SBOM, Pricing, Cloud`: `Overview`
  and `Gate` are now legitimately present (a page and the name of the Release stage), and the bare-`Gate` check is
  scoped to `main` so it still catches a rail entry advertising a stage that does not exist.

## 6. What this round did NOT reach (§13 — first-round productization, stated plainly)

- **The detail column.** The references put a right-hand inspector on Compare, Release and History. Building it
  properly means a selection model per page, and no page holds one today. A `SplitPanel` and a `DetailPanel` were
  written for the plan, then **deleted at close** rather than shipped unused; History already discloses the same
  facts in an expanded row (F2R-02) and replacing that with a side column is a larger interaction change than
  this round's breakage budget. The gap stays open and is named in `BASELINE.yaml`'s `deferred_explicitly`.
- **The compact control density.** `History`'s `.action` and `Details`' `.inspect`/`.apply` are a fourth button
  level — the same outline, but a row-height padding the shared `Button` does not model. Unifying them needs a
  `size` dimension on `Button`, which is a design-system change rather than a page change, so those three controls
  stayed local and the rest of the pages' buttons did not.
- **Real-desktop visual evidence.** `visual_evidence: STRUCTURAL_CONTRACTS_ONLY` in `BASELINE.yaml`. The
  before/after screenshots §11 asks for need the installed app run on this machine, and that is a machine-level
  operation: it parks the owner's own FirmwareSight store. It was not authorized in this session, so no screenshot
  was taken, no `02_before`/`03_after` evidence was written, and nothing in this report is a claim about pixels in
  a real window. What is claimed instead is: the DOM structure pinned by the tests above, the token audit in §3,
  and the layout contract in §4.
- **The window-title naming mismatch.** The rail and the title bar say `Bundle & History`; the page's own `h1`
  still says `History`. Fixing it moves an IPC title surface, which the round's own boundary said to leave alone.
- **Two of the five reference interactions.** `Mark N/A` and `Link repository` from FS-UI-04 have no command;
  FS-UI-07's dependency workflow has no parser. They are absent, not stubbed.

## 7. Verdict, in the words this round is allowed to use (§15)

The desktop UI **converged from an engineering-complete MVP candidate toward the intended desktop product
interface**: one shared design-system layer instead of five page dialects, one new Overview page made of facts the
shell already holds, the Release verdict where the eye starts, the F2R-01 layout defect closed on the two pages
that still carried it, and 236 UI tests and 868 Rust tests green with no authority surface moved.

It is **not** "V1 done", not GA, not "equals the mockups", and it does not speak for any future interaction.
`U1` claims no stage: `B1` Private Beta stays reserved and unauthorized, V1 stays `IN_PROGRESS /
RECRUITMENT_READY` with zero eligible sessions and its frozen cohort build, `paused_for` this track, and the
product stays `MVP_CANDIDATE` at baseline `0.6.0`.

**STOP.** The next repository write on V1 happens when a real external session happens, which is operator work.
No further UI round, no packaging round, no feature round and no stage is authorized by this document.
