---
title: "P5 Commit F2R — Installed UI Productization Corrective, evidence report"
doc_id: "FS-P5-F2R-REPORT"
product: "FirmwareSight"
version: "0.6.0"
status: "EVIDENCE"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 Commit F2R — installed UI productization corrective: the evidence

Authorized by *FirmwareSight — P5 Commit F2R — Installed UI Productization Corrective, Execution Prompt
v1.0*, archived at `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitF2R_UI_Productization_Corrective_v1.0.txt`
(delivered SHA-256 `5d8719eca4a6ee5e2ee5b1b2c126c69d8a1ef027881da6ef98e11be9bd1fb375`, 47,100 B, 2,117 CRLF
lines; stored blob `5f2bbda77c82fdcf57a6f9dfb48a4e622b28aa3e`, SHA-256 `6281f5d2067ce6975597d609fa556c6a7f3626c828a624b706c8c2335c8fe4ed`,
44,984 B, 2,116 LF, no final newline).

Evidence root: `%TEMP%\FirmwareSight-P5-F2R-20261005T221112`, outside this repository. Every path below in
`backticks-with-a-directory-prefix` (`09_owner_park/…`, `14_smoke/…`) is inside that root.

This round fixes three findings that Commit F2 measured on an installed binary. It changes display and
layout only. It is two heads: **F2R1** `fb5f628` (product) and **F2R2** (this file and the records around
it, docs/evidence/governance only).

## 1. Start authority

Measured at the §2 hard preflight, `2026-10-06T06:10:07Z`, before any product write:

| Fact | Value |
| --- | --- |
| `git rev-parse --show-toplevel` | `D:/study/Software/FirmwareSight` |
| `git branch --show-current` | `main` |
| `git rev-parse HEAD` | `55fad63d2071c5511133ea1cfffc664885b0b47e` |
| `git rev-parse origin/main` | `55fad63d2071c5511133ea1cfffc664885b0b47e` — identical, so this round starts on the remote's tip |
| `git fetch --prune origin` | rc=0 |
| `git worktree list` | one worktree, the main one |
| working tree at preflight | the archived prompt staged, `10_AUDIT/SOURCE_PROMPTS/README.md` modified, nothing else |

The two F2 heads are `d1dc61c` (installed acceptance and the eight-document closure pack) and `55fad63`
(the entry documents reconciled with the round that landed). **F2's CI run is #69, id `37367382516`, at
head `55fad63`, and it concluded success on attempt 3.** Read per attempt, not summarized:

| Attempt | Outcome |
| --- | --- |
| 1 | 3 jobs success, **7 jobs `cancelled` with 0 executed steps** (cancelled 2026-10-05T20:18:46Z, 15 min after starting — runner allocation, no step ran) |
| 2 | 6 jobs success, **4 jobs `cancelled` with 0 executed steps** (cancelled 2026-10-05T20:36:37Z) |
| 3 | the four cancelled jobs re-executed and passed; the six earlier successes carried forward; **10 of 10 success** |

`d1dc61c` has **no run of its own**: `gh api "repos/2023violet/FirmwareSight/actions/runs?head_sha=d1dc61c"`
returns nothing, and `origin/main`'s reflog has exactly one entry landing `55fad63`
(`2026-10-05 13:03:38 -0700`, i.e. 20:03:38Z — run #69 was created three seconds later) and none landing
`d1dc61c`. One push of two commits, one run at the tip. That is recorded rather than papered over, the same
way Commit E1's head was. F2's attempt history is not rewritten as first-attempt green anywhere in this file.

## 2. The F2 findings F2R was given

All three come from `P5_DESKTOP_ACCEPTANCE_REPORT.md` §6, graded there on installed evidence, and none was
patched in F2 because F2's own prompt forbade touching `apps/**`.

| F2R id | F2 finding | F2 severity | F2 evidence |
| --- | --- | --- | --- |
| F2R-01 | Analyze → Sections collapses its text column at a narrow window: rows become unreadable instead of reflowing | **S2** (layout) | `07_analyze/L00–L22` at the default 1056 × 799 and narrower |
| F2R-02 | History's `Details` button is clipped at the default window width | **S3** (layout) | `11_history/R00–R12` |
| F2R-03 | Release's Evidence basis column prints the raw Core enum `MapRegionAndElfLoad` while every other page prints human text | **S3** (presentation) | the `15`/`16` captures plus `08_compare/N00–N12` |

F2R reproduced all three locally before changing anything, and §19's pre-fix reproduction is in
`02_test_first_red/RELEASE_RED_AT_55fad63.txt` (the caption test, red against the pre-fix tree) and
`01_layout_probe/` (the geometry, measured in a replica at the three acceptance sizes).

## 3. The exact code changes

Twelve product files, all inside `apps/desktop/ui/src/` — the only path §16 allows. Full diffstat of F2R1:
19 files, **+3,155 / −394**, of which the other seven are the archived prompt, its README row, the design
record, `DIRECTORY_TREE.txt`, `SHA256SUMS`, and the two `.ai` entry documents.

| File | Change | Finding |
| --- | --- | --- |
| `Details.module.css` | `.viewport { overflow-x: auto }` **added**; `.table` **loses** `display: block` and its own `overflow-x`; cells take `white-space: normal`; `.table td.mono` keeps `nowrap`; `.table td.value` re-declares `normal` + `overflow-wrap: break-word` + `min-width: 24ch`; `.unknown` becomes a **column** flex so a reason sits on its own line; `.note` gets `break-word` + `24ch` | F2R-01 |
| `Details.tsx` | every detail table wrapped in `<div className={styles['viewport']}>` | F2R-01 |
| `History.module.css` | same viewport/nowrap pattern; `.action { white-space: nowrap }` | F2R-02 |
| `History.tsx` | the `Details` cell and the srOnly `Row actions` header move to the **leading** position of each row and each header row, in all three table families; tables wrapped in the viewport; `colSpan` 8 / 7 / 7 unchanged | F2R-02 |
| `evidenceBasis.ts` | **new**: a ten-entry map from both `Debug` names and kebab wire aliases to captions, `evidenceBasisCaption(basis)`, unknown input → `'Unrecognized evidence basis'` | F2R-03 |
| `Release.tsx` | the Evidence basis cell calls `evidenceBasisCaption(row.basis)` instead of printing `row.basis` | F2R-03 |
| `Compare.tsx` | its inlined caption table **moved into** `evidenceBasis.ts`; calls the shared helper; keeps its own null wording | F2R-03 |
| `Analyze.tsx` | the weakest-basis sentence routes through the same helper (lines 420 and 428 printed the raw value) | F2R-03 |

The mechanism, measured before it was chosen (`01_layout_probe/MEASUREMENTS.md`, replica, Blink, three
acceptance sizes):

1. `display: block` on the `<table>` pinned the box to the pane and let the table be laid out **narrower
   than its own minimum**; `overflow-wrap: anywhere` made the prose column's minimum **one character**, so
   that column absorbed the whole deficit — measured 18 × 380 px, row 529 px, at both 1024 and 1056.
2. A wrapper alone is **necessary but not sufficient**: with `anywhere` still on the prose the intrinsic
   minimum still contains an 18 px column (identical 18 × 380 px). Both halves had to change.
3. History's action was clipped for a different reason: the table's rigid minimum measured **939 px**
   against a **736 px** pane at the frozen 1024 minimum, and the action was the **last** cell — 195 px
   beyond the right edge of the scroller. Not a CSS bug; a placement that cannot survive a table wider than
   the window. Making it the leading cell is what the Evidence table already does (`details.test.tsx:751`
   records that precedent and why).

Rejected, each on a measurement, in `P5_F2R_UI_CORRECTIVE_DESIGN.md` §5: wrapper-only (defect survives
behind a scrollbar), floor-without-the-own-line-reason (table scrolls 127–140 px at the 1440 design target),
`position: sticky; right: 0` (the pinned cell **covers 195 px of stored facts at 1024 and 163 px at 1056**,
and its separator would need a shadow, which AGENTS.md §11 forbids on tables), letting mono ids and
timestamps break, and an icon-only row action (§8 forbids it; `Details` stays a text button with
`aria-expanded`).

## 4. Tests

UI direct count **219 → 225** in the **same 8 files**; six new tests, one per invariant, and each one was
written red before the fix:

| File | New test | Proves |
| --- | --- | --- |
| `details.test.tsx` | *puts every detail table inside its viewport and keeps all of its columns* | F2R-01 structure |
| `history.test.tsx` | *leads each stored row with its action, and its header with the same column* | F2R-02 placement |
| `history.test.tsx` | *gives the scroll axis to a wrapper around each stored table* | F2R-02 scroll ownership |
| `intake.test.tsx` | *names the weakest basis in the words a reader needs, not the identifier behind it* | F2R-03 on Analyze |
| `release.test.tsx` | *names a memory basis by what it means, not by the Core word behind it* | F2R-03 on Release |
| `release.test.tsx` | *says plainly what it cannot name, and what was never recorded* | the unknown-basis and null fallbacks |

Four mutation proofs (`03_mutation_proofs/MUTATIONS.txt`), each reddening its own test and nothing else
that mattered: restoring `row.basis` directly → Release red and the DOM dump shows `MapRegionAndElfLoad`;
removing the `MapRegionAndElfLoad` entry → Release red, Compare still green (it feeds the kebab alias of the
same caption); removing **both** spellings → Release **and** Compare red, which is the proof the two surfaces
read one map rather than two copies; sending the History action back to the end of its row → its structural
test red; taking `viewport` off the wrapper → the History and Details wrapper tests red. Every mutated file
was restored from `03_mutation_proofs/backup/` and the restore verified by SHA-256 against
`backup/SHA256.txt` (four digests recorded).

What jsdom is **not** claimed to prove: none of those tests measures a clip, a column width or a scroll.
`01_layout_probe` measures mechanism on a replica, and §30–§32's installed captures are the geometry
authority. A CSS-source assertion (that the stylesheet carries no `display: block` on the table) was written
and then **removed**: with Vitest's default `css: false`, `?raw` resolves to `{}` and `?inline` to `""`, and
making it readable would have needed a `vite.config.ts` change or a dev dependency — both outside §16 and §18.

## 5. Design-token impact (§5 / §6)

`13_token_impact/TOKEN_IMPACT.txt`, measured on the committed diff:

- `assets/design-tokens.json`: **0 bytes changed**. Not staged, not modified, not in the diff.
- Every literal length added to the two stylesheets: `24ch` ×2. Everything else (`60ch`, `68ch`, `72ch`,
  `80ch`, `88ch`) was already in the tree.
- Every `var()` reference added: `var(--fs-space-1)` ×1 — an existing token.
- The `ch` unit is what the shipped prose rules already use: `Analyze.module.css:23` 60ch,
  `Compare.module.css:27` 60ch, `Release.module.css:27` 68ch and `:96` 72ch and `:312`/`:319`/`:467` 88ch,
  `History.module.css:27`/`:108` 80ch, `GettingStarted`/`Help` 80ch.
- Preferred outcome achieved: **no token change, no invented pixel constant.** No gradient, glass, glow,
  purple, shadow on a panel or table, colour-only state, or dark theme. Row density stays
  `--fs-row-compact`; `Unknown` keeps its neutral treatment. No focus-visible rule touched.

## 6. Local validation (§21 / §22)

`06_local_validation/S21_S22_FINAL.txt`, run 2026-10-06T07:05:37Z → 07:08:56Z against the tree that was
committed, with the tree proven not to move under the run (manifest of all `apps/desktop/ui/src` digests
identical before and after: `e5713cf5…08b79`, unchanged YES):

```
cargo fmt --all -- --check            rc=0     cargo clippy -D warnings          rc=0
cargo test --workspace                rc=0     pnpm install --frozen-lockfile    rc=0
tsc --noEmit                          rc=0     eslint .                          rc=0
vitest run                            rc=0     tsc --noEmit && vite build        rc=0
```

Rust **868 passed / 0 failed across 47 suites** (unchanged, as §21 requires). UI **225 passed in 8 files**
(219 → 225). Build: 46 modules, `dist/assets/index-R0fRHss-.css` 41.35 kB, `index-DCexUFJu.js` 344.08 kB.

§22 reliability campaign: **20 fresh-process `vitest run` invocations, 20 green, 0 failing**, every one
reporting 8 files / 225 tests, wall time 7 min 25 s (06:48:56Z → 06:56:23Z), no retry wrapper, no sleep, no
timeout override, no run discarded or replaced, and the repetitions are **not** multiplied into the canonical
UI count.

Two supersessions, kept rather than deleted: an earlier §21 pair of logs (06:43:00Z / 06:44:03Z) and a first
§22 campaign both pre-dated three comment-only lines that moved at 06:47:03Z, so they described a tree that
was not the candidate. The campaign was stopped mid-flight and re-run; the partial file is
`05_ui_reliability/CAMPAIGN_superseded_partial.txt` and backs no claim. vitest emits colour escapes, so the
counts were taken with the escapes stripped — a plain grep for the summary text matches 0 lines in that
file, and the record says so rather than hiding it.

Full gate at the identical staged state (§23 step 9): **17 of 17** steps, rc=0; drift **8 of 8**, deny
**1 of 1**, `core-smoke` **3 of 3**, package **4 of 4**, no `SKIP`. `verify_baseline_artifacts.py`: tracked
707, entries 705, every mismatch counter 0, RESULT PASS. Clean detached worktree at the same staged content:
**19 of 19** (17 authoritative + the two bootstrap steps a cold tree needs), 868 Rust / 225 UI there too.
The worktree was removed with `git worktree remove` and, when that hit Windows' "Filename too long",
`cmd //c rmdir /s /q` plus `git worktree prune`; `git worktree list` now shows the main worktree only and
`git status --porcelain` is empty.

## 7. Product candidate SHA (F2R1)

`fb5f62852a4c3b83bc472f5903bff214888ab621`, message `P5: close installed UI productization findings`, parent
`55fad63d2071c5511133ea1cfffc664885b0b47e` (= `origin/main` at start, so the push was a normal
fast-forward; no force, no history rewrite, no amend).

Baseline closeout followed ADR-0029's order (§23): edit → inspect → stage intended non-baseline changes →
regenerate `DIRECTORY_TREE.txt` **from the index** (path set changed, 825 lines) → stage it → generate
`SHA256SUMS` from **stage-0 blobs** (705 entries) → stage it → `verify_baseline_artifacts.py` → full gate →
inspect the staged diff → commit. `git archive fb5f628 | tar -x` **outside** the repository: 707 files
extracted, 705 manifest entries, `sha256sum -c` rc=0 with 705 `: OK` lines and 0 lines that are not OK.

Diff-allowlist proof (`08_allowlist/ALLOWLIST.txt`): every staged path is inside `apps/desktop/ui/src/**`,
or is the archived prompt plus its README row, the design record, or the two baseline artifacts; forbidden
paths in the diff **0**; unstaged drift on the frozen token file **0**.

## 8. Candidate remote CI (§41)

Run **#70**, id `37431977428`, `event: push`, headSha `fb5f62852a4c3b83bc472f5903bff214888ab621`,
**attempt 1**, `completed / success`, created 2026-10-06T07:48:28Z (`origin/main` reflog put the push at
00:48:25 -0700 = 07:48:25Z, three seconds earlier). There is exactly one run for this head — no cancellation,
no rerun, no borrowed green.

**10 of 10 jobs green, every job and every step read individually** (`16_ci_readback/CI_READBACK.txt`):
`Dependency policy`, `Desktop UI (ubuntu-latest)`, `Desktop UI (windows-latest)`, `Generated output drift`,
`Package Ubuntu`, `Package Windows`, `Package macOS`, `Rust (ubuntu-latest)`, `Rust (windows-latest)`,
`macOS Core Smoke`. The one step outside `success` in the whole run is `Rust (windows-latest)`'s conditional
`Install Linux prerequisites for the Tauri shell`, which is the job's design, not a gate skip.

Read out of the runner's own logs rather than recalled: `Rust (ubuntu-latest)` (job `112164719342`) sums to
**47 suites, 868 passed, 0 failed, 0 ignored**; `Desktop UI (ubuntu-latest)` (job `112164719325`) prints
`Test Files  8 passed (8)` / `Tests  225 passed (225)`, with `release.test.tsx (60)`, `compare.test.tsx (44)`,
`help.test.tsx (25)`, `intake.test.tsx (23)` among the per-file lines. So §41's two expectations hold on the
remote: Rust stays 868 and UI rises.

## 9. The exact Windows artifact (§26)

Downloaded from **run #70 itself**, not from F1 and not rebuilt locally:
`gh run download 37431977428 -n FirmwareSight-0.6.0-windows-x86_64 -D <evidence>/14_artifact`.

| Identity | Value |
| --- | --- |
| artifact id / name | `11397938806` / `FirmwareSight-0.6.0-windows-x86_64` (set reported at 5,534,455 B) |
| installer | `FirmwareSight_0.6.0_x64-setup.exe`, 3,886,598 B, SHA-256 `efbc45a3258986e1763343f7f46e927e8450b48293aa9bbd5b447a5a246d5fd4` |
| index files | `SHA256SUMS.txt` and `artifact-metadata.json`, both verified (`sha256sum -c` rc=0, 2/2 `OK`) |
| cli companion | `…-cli-fwsight.zip`, 1,668,153 B, `696288e89b146509aa213611a333e53c92ba722365fd3e9762754bc2901cc0bf` |
| payload (pre-bundle) binary | `4ff84ad67bb816666f0db4d3485d89805f321f2a47619ff217d21f0b9e25815f` |
| installed binary | `2cf01a6d5887a09981c3f338c217f7270bf1fc303101b9a2ec7d77d406eb670b`, 15,362,048 B, mtime 2026-10-06T08:07:40Z |
| CI toolchain | rustc/cargo 1.98.1, node v24.21.0, pnpm 12.7.0, tauri-cli 2.12.1, runner `win25-vs2026` |
| `git_commit` in metadata | `fb5f6285…` — the same SHA as the candidate, so this package is that head's bytes |
| signing | unsigned; no certificate, no signature, no notarization. Updater not enabled (`createUpdaterArtifacts: false`) |

The installed binary is **not** byte-identical to the payload digest, and the reason is in the metadata:
the bundler rewrites the 27-byte token `__TAURI_BUNDLE_TYPE_VAR_UNK` to `…_NSS`. Checked on the installed
bytes instead of taken on trust: 1 occurrence of the NSS spelling, 0 of UNK.

Invariants: F1's artifact `11337963032` appears nowhere in this round; no F2 binary was reused; no local
`cargo`/`tauri` build was installed.

## 10. Owner-store isolation (§28)

Parked **before** any install action (`09_owner_park/PARK.json`):

```
OWNER_STORE_PARKED      = YES
OWNER_BACKUP_HASH_MATCH = YES
```

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `firmwaresight-p0.sqlite` | 155,648 | `d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468` |
| `firmwaresight-p0.sqlite-wal` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `firmwaresight-p0.sqlite-shm` | 32,768 | `fd4c9fda9cd3f9ae7c962b0ddf37232294d55580e1aa165aa06129b8549389eb` |

The live slot was then seeded with a **copy** of F2's preserved disposable v5 store (3 builds / 1 Gate run /
1 release record), proven by digest on both sides in `09_owner_park/DISPOSABLE_SEED.txt` — the evidence
original was never moved, and §28's "do not move the only evidence copy" held. Nothing was seeded by SQL and
no owner data was used as a product input.

Install ran through the real wizard with real input (`10_install/INSTALL_RECORD.txt`): pages
Welcome → Choose Install Location → Choose Start Menu Folder → Installing → Installation Complete → Finish,
each press going through the ownership guard. Two harness facts, both owned as harness and not product: the
step table wrongly assumed page 4 was "Installation Complete" (it was "Installing"), and one `Next` at
(1072, 689) was **refused** by `click_verified` because a Chrome `Chrome_RenderWidgetHostHWND` of pid 26036
owned that pixel — `POINT_OWNED_BY_OTHER_WINDOW`, no input sent — after which the wizard was moved with
`SetWindowPos(SWP_NOSIZE)` into clear space, the foreground re-confirmed, and the presses re-issued against
re-read rectangles. That guard is the repair for the defect F2 owned: F2's `clickscreen` had silently
delivered installer clicks to a browser window for most of its round.

Harness hardening came **before** any installed evidence (`11_harness/DRIVER.txt`): the first probe of
`f2r_ui.py geometry` matched a window titled "Claude", because `f2_ui.main_window()` falls back to the first
visible titled window when the target process is absent. Survivable for a click, which re-tests ownership at
the pixel; not survivable for a resize. `require_app()` now refuses with `NO_PROCESS` / `NO_WINDOW` and no
fallback of any kind, reproduced by the refusals in that record. Only documented Win32 entry points are used
(`SetWindowPos`, `GetWindowRect`, `GetClientRect`, `ClientToScreen`, `GetDpiForWindow`,
`GetForegroundWindow`, `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS=9)`, `EnumChildWindows`,
`GetClassNameW`, `WindowFromPoint`, `SendInput`); no `WM_*` message, no `SendMessage`/`PostMessage`, no CDP
or DevTools, no injected `click()`, no direct Tauri IPC, no silent installer flag, no DB mutation as a
product action.

## 11. Analyze at 1024×720 / 1056×799 / 1440×900 (§30)

Installed F2R1 candidate, host `DESKTOP-S0IK346`, 96 dpi / 100 % scale. Sizes are outer `GetWindowRect`
values, the same convention F2 used when it named the default 1056 × 799; each resize is read back after
`SetWindowPos` and prints `SIZE_OK` only when the measured rectangle equals the requested one.

| Size | Result | Evidence |
| --- | --- | --- |
| 1024 × 720 | **PASS** | `S11_sections_1024.png` (reasons wrap at a readable measure, three to four lines of ~20–24 characters), `S18_caption_1024.png` (focus ring on *Apply filter* after Tab, sort arrow on Name, rows alphabetical), `S29_filterrow2.png`, `S30_filter_applied_1024.png` (`.text` → one row), `S32_sorted_name_1024.png`, `S36_pager_1024.png`, `S19_scrollleft_1024.png` |
| 1056 × 799 | **PASS** | `T01_sections_1056.png`, `S00_launch_default.png` (outer 1056×799, client 1040×760) |
| 1440 × 900 | **PASS** | `S09_sections_1440.png` (all ten columns in view), `S10_sections_1440_scrolled.png` (its own scrollbar dragged to the end: the Region column sentence complete), `R05_analyze_1440.png` |

No column is near-zero wide; the Index / Name / Role / Flags / address columns keep their figures on one
line because only prose wraps. The horizontal scrollbar belongs to the **table area**, never the page: at
1024×720 the thumb spans ≈419 px of a ≈708 px track (content ≈1.69× the pane), at 1440×900 ≈1,075 of
≈1,140 (≈1.06×). Filter, sort, pager and keyboard order all still work; `Showing 1 to 19 of 19 sections`
with Previous page disabled.

**One trade recorded rather than glossed:** at 1440×900 the table now needs that small contained scroll to
reach the last column, where the pre-fix squeezed layout fitted the pane by being unreadable. `P5_F2R_UI_CORRECTIVE_DESIGN.md`
§7 measured and accepted it, and §6 requires the table area to own the scroll, which is what happens. The
alternative that fits the target with zero scroll leaves a 71 × 120 px prose column at 1024.

## 12. History at 1024×720 / 1056×799 / 1440×900 (§31)

| Size | Result | Evidence |
| --- | --- | --- |
| 1024 × 720 | **PASS** | `H03_builds_1024.png` — action fully visible, row open |
| 1056 × 799 | **PASS** | `H01_builds_1056.png` — `Details` leads every row; `H02_row_open_1056.png` — the opened digests still wrap |
| 1440 × 900 | **PASS** | `H04_builds_1440.png` — all eight columns fit, no scroll needed |

Because the action is now the **leading** cell of every row in all three table families, it is the one
control that cannot be the thing falling off the right edge, and the srOnly `Row actions` header leads each
header row. Keyboard: `H05_keyboard_activation_1056.png` plus the `KB_A/B/C` sequence — mouse-opened row,
then `SPACE` closed it and `SPACE` opened it again, with the focus ring visible on the button in both
keyboard frames.

## 13. Release evidence basis (§32)

Two evidence states on the same surface, both driven through the product:

| State | Expected | Observed installed |
| --- | --- | --- |
| no MAP attached | "ELF address/flags evidence" | `R04_budgets2_1440.png` renders exactly that (was `ElfAddressAndFlags`) |
| MAP attached | **"MAP regions + ELF load evidence"** | `R18_budgets_map.png` renders exactly that — the string §32 names, and the raw word F2 captured was `MapRegionAndElfLoad` |
| must **not** appear | `MapRegionAndElfLoad` | does not appear on Release; nor on Analyze, which prints "Weakest basis MAP regions + ELF load evidence; region evidence came from the linker." (`R16_memory_map_evidence.png`) |

The stored value and the wire token are unchanged, and the caption is never serialized — §11 and §17. The
MAP-backed analysis deduplicated onto F2's existing `snap-4b4…4709ace6` row, which is why the build count
stayed at 3; no bypass of the product was used to force a second row.

## 14. Quick functional smoke (§33)

`SMOKE_AND_DISPOSITIONS.txt`, at 1056 × 799 on the installed F2R1 candidate, disposable store only.
Captures `14_smoke/K01–K05`.

| Surface | Result |
| --- | --- |
| Analyze | ELF chosen through the real file dialog, MAP attached, analysis ran; Artifact / Memory / Capabilities / Counts / Sections / Symbols / Evidence all rendered, no error panel |
| Compare | two stored builds selected from the product's own dropdowns; Memory deltas Old 376 / New 376 / Delta 0, RAM 76 / 76 / 0; comparability `exact`; weakest basis rendered as words |
| Gate | ran twice against the default policy (ELF-only build `gate-15fd6f46a733552f0629aa9d0f18fd137a409ce3f7b447d71a9f881c15e37648`, disposition REVIEW; then the MAP-backed build); Required artifacts showed elf Present and map Present with digests |
| History | three table families rendered: 3 builds, 1 Gate run, 1 release record as captured at 01:54–01:56 local; by the end of the pass the store held **3** gate runs, because the two presses above wrote them |
| Diagnostics | Version 0.6.0, Store schema v5, Store health `healthy`, Git `available`, store named as `firmwaresight-p0.sqlite` |
| no crash | window survived the whole pass, title tracked the page, pid 17640 alive at the end, no unbidden error panel or dialog |
| no new path leak | the store is named by file, never by folder, on both surfaces that talk about it; Help still states "an absolute path stays inside the application"; no `C:\Users\16429` string on any inspected surface |

No new S0 and no new S1 appeared in this pass.

## 15. Owner restore (§28)

Uninstall first (`15_uninstall_restore/UNINSTALL.txt`): the NSIS uninstaller re-executes as a child pid, so
the driver followed pid 18736 after `wait_window` on the parent timed out. Page buttons read
`['&Uninstall', 'Cancel', 'Delete the application data']`; the checkbox was read with `BM_GETCHECK` →
**0**, and left alone — the destructive option is **still unexercised**, exactly as F2 recorded it, because
that folder also holds unrelated historical stores from earlier phases. After it: install dir **ABSENT**,
app-data entries still 17, the live disposable store still present, 14 unrelated historical siblings still
present.

Then, in order: the disposable store's final bytes preserved into `15_uninstall_restore/f2r_disposable_store/`
(main `da63a52b…`, wal `01f5b33f…`, shm `13a3885c…`, with `MANIFEST_DIGESTS.txt`), those three live files
removed, and the owner's store restored:

```
ORIGINAL_DB_RESTORED         = YES
ORIGINAL_DB_SHA_MATCH        = YES
OWNER_STORE_OPENED_BY_F2R    = NO
```

Restored bytes equal `d6e41034…` (155,648 B) / `e3b0c442…` (0 B) / `fd4c9fda…` (32,768 B) — the §28 table in
this file's part 10. `OWNER_STORE_OPENED_BY_F2R = NO` rests on observation, not assertion: the owner store
left the live slot before install and never returned until this restore; the product ran against the
disposable store; the returned bytes match the parked digests exactly, which any write path would have
changed; mtimes are preserved (`Sep 30 11:56` for the main file); and no new app-data entry was created.
`09_owner_park/parked/` is empty. Full record: `15_uninstall_restore/PARK_RESTORE.txt`.

## 16. Finding dispositions (§36)

§36 does not allow a CLOSED from unit tests alone — the installed new artifact is required, and it is what
parts 9–13 supply.

| Finding | Before (F2) | After (F2R, installed) |
| --- | --- | --- |
| F2R-01 Analyze narrow table | **S2 PRODUCT** | **CLOSED** — installed at all three sizes, `11_analyze_sizes/` |
| F2R-02 History Details clipping | **S3 PRODUCT** | **CLOSED** — installed at all three sizes, `12_history_sizes/` |
| F2R-03 Release raw Core enum | **S3 PRODUCT** | **CLOSED** — both evidence states installed, `13_release_caption/` |

F2's findings are not rewritten: F2 discovered them, F2R closed them, and `P5_DESKTOP_ACCEPTANCE_REPORT.md`
§6 keeps its original grading. F2's run #69 attempt history is unchanged in part 1 and in
`P5_CI_AUTHORITY.md`. No evidence was deleted; the superseded local runs and the quarantined harness events
are all still in the root.

## 17. Design / Accessibility — bounded verdict (§34)

`P5_DESIGN_ACCESSIBILITY = PASS_FOR_FROZEN_DESKTOP_SCOPE`

Each condition §34 lists, checked against the installed artifact:

| Condition | Result |
| --- | --- |
| frozen 1024 × 720 minimum works for the corrected surfaces | yes — `S11`/`S18`/`S29`/`S30`/`S32` |
| 1056 × 799 (F2's defect size) works | yes — `T01`, `H01`, `H02` |
| design target 1440 × 900 works | yes — `S09`/`S10`, `H04`, `R04`/`R18` |
| `Details` keyboard focusable and activatable | yes — mouse-opened row, then `SPACE` closed and `SPACE` opened again, focus ring visible in both frames |
| no text-only critical control clipped | yes — the row action leads its row on all three tables; filter/sort/pager each operated at 1024 × 720 |
| no raw evidence enum on a corrected human-facing surface | yes — words where F2 saw the enum |
| existing focus-visible / design-token rules remain | yes — no token byte changed; the only literal added is the 24ch prose measure |
| full UI suite green | yes — 225 tests in 8 files, 20/20 fresh-process repetitions |
| focused installed artifact green | yes — run #70 10 of 10, three findings revalidated |

Not claimed, and this is the boundary rather than a hedge:

```
WCAG_CERTIFICATION   = NOT_PERFORMED
MULTI_DPI_125_150    = NOT_TESTED
SECOND_WINDOWS_HOST  = NOT_TESTED
```

One host at 100 % scale, the frozen desktop range, the three corrected surfaces plus the §33 smoke list. No
conformance level, no other display, no other machine. And no claimed accessibility-role gain from the
wrapper either way: `01_layout_probe/a11y-control.html` found that the inspection tool flattens table, row
and cell roles identically for a plain and a `display: block` table, so the wrapper is justified by measured
layout behaviour only — the design record §14 says this rather than letting the DOM change read like an a11y
improvement.

## 18. L20 final disposition (§35)

```
L20 = CLOSED ACROSS VERIFIED HUMAN-FACING MEMORY-BASIS SURFACES
```

L20 was closed on Compare in Commit E, reopened by F2 for the Release surface, and F2R now covers every
human-facing memory-basis surface found: Compare (already closed), Release's Evidence basis column, and the
Analyze weakest-basis sentence — all three reading the one shared caption map, verified installed above. The
§13 re-audit (`04_enum_audit/`) classified every hit: domain and wire code keeps the identifier, `Compare.tsx`,
`Release.tsx` and `Analyze.tsx` map it for display, and **no human-facing surface prints a memory-basis value
raw any more**. Two spellings the prompt names have no code behind them — `ConfiguredRegionAndElfLoad`
(the real variant is `RegionConfigAndElfLoad`) and `InsufficientEvidence` (the real variant is `Insufficient`)
— 0 hits outside this round's own documents, recorded as not-found rather than added to the map, because §11
forbids inventing semantics.

Commit E's historical text is **not** rewritten: it says what was verified at Commit E, in date order, and
the addendum in `P5_KNOWN_LIMITATIONS.md` §8 carries the F2 reopening and this closure.

## 19. P5 state after F2R

- `F1 = FINAL PASS`, `F2 = PASS` (its three findings were real and it closed the journey it was asked to).
- `F2R1` is the head the installed artifact was built from; `F2R2` is this file plus the records around it,
  and §42 requires it to carry **no product code** — which §43's count rule proves: Rust stays **868** and UI
  stays **225 in 8 files**, exactly F2R1's numbers. A docs-only round that moved either count would be a
  product change wearing a documentation diff.
- `P5 = IN_PROGRESS`. The product stays **MVP CANDIDATE**; the baseline stays `0.6.0`.
- Still open and unchanged by this round: L15's wire half (`CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`),
  F2-1/F2-2 (installed migration is one path; the owner's store is at schema v2 so the chained v2 → v5
  installed upgrade is unexecuted), F2-3 (WebView2 as an untested negative), the uninstaller's data-deletion
  option (still `NOT_TESTED_BY_DESIGN`), the install directory's post-uninstall instability, L3 fuzzing,
  L8, L9, L11, L22's residual risk, L23's residue, L4's one-host boundary, and the licence decision, which
  AGENTS.md §9 reserves to the owner.
- No tag, no GitHub Release, no signature, no notarization, no updater, no new dependency, capability or
  permission, no schema or migration change, and no change to `analysis:1`, `diff:1`, `gate-results:1`,
  `accepted-reviews:1`, `release-manifest:1`, SQLite schema 5, migrations 0001–0005, ADR-0028, ADR-0029 or
  `elf.program-header`.

## 20. Stop

F2R ends here. §8 of the prompt and §39 of its structure both keep F3 behind the Architect: **F3 is not
authorized in this round**, no F3 work was started, and no F3 state is written anywhere in this repository.
No further corrective head is created merely to record a CI result (§44). The P5 completion token that F3
owns is not set, `active_task` stays `P5_PRODUCTIZATION`, and nothing was tagged, released, signed or
notarized.

Return to the Architect.
