---
title: "Windows Human QA Pack (D2C-R)"
doc_id: "FS-EV-D2CR-001"
product: "FirmwareSight"
version: "0.6.0"
status: "PACK_NOT_RUN"
owner: "QA"
last_updated: "2026-10-17"
---

# Windows human QA pack for the R7 build

**Nothing in this pack has run.** It replaces the D2C runbook's case list for any build at or after `main` `5015143`. The D2C runbook ([HUMAN_NATIVE_QA_RUNBOOK.md](../D2C/HUMAN_NATIVE_QA_RUNBOOK.md)) still holds the rules on who runs it, on which machine and with what evidence. Its H01–H14 are mapped below.

Automated evidence is never re-rated as a human pass. "Covered" below means the case does not need a person, because a real-input or automated run already observed what the case asks. The source is named for each.

## 1. Build under test

| Field | Value |
|---|---|
| Commit | `main` `5015143eaaa5493835f7545c49e997933879c5cc` (R-series final; CI run `39093441971`, 10/10) |
| CI artifact | `FirmwareSight-0.6.0-windows-x86_64`, artifact digest `sha256:5acb171274669ec3eabd347adddb069c9d49a3fb0e5d0917946969ba4dfd6cf3`. **It expires on 2026-11-01**: download and keep it before then, or re-run CI on the same commit. |
| Installer SHA-256 | Read by the operator after download and recorded here. Not yet known. |
| Account | A dedicated non-Owner Windows account (runbook "Who, where, what") |
| Inputs | `fixtures/elf/p2-diff/{base,target}`, `fixtures/elf/m2-objects/{base,target}`, a synthetic writable folder, a synthetic non-writable folder |

## 2. H01–H14 against the evidence that exists

| D2C case | What the evidence shows | Source | Left for a person |
|---|---|---|---|
| H01 Visual export: cancel, deny overwrite (OS and app prompt), confirm overwrite, non-writable folder, retry | Cancel only: a cancelled PNG (Release), a cancelled Compare export and a cancelled Markdown export (Release) write nothing and raise no error. New-file Markdown writes from all three pages were observed. | R3 C07, R4 R11, R6 M01–M04 | **Deny at both prompts, confirm overwrite, non-writable folder, retry** → Q1 |
| H02 Compare again while the replace confirmation is open | Unit tests only (stale export guard) | `compare.test.tsx` | **All** → Q2 |
| H03 English replace confirmation | Not observed | — | **All** → Q3 |
| H04 Loading state in each language | Not observed | — | **All** → Q4 |
| H05 Tab / Shift-Tab order, focus ring, Esc returns focus | Real keys: the skip link is the first stop, Enter lands on the page heading, Ctrl+K/arrows/Esc in the palette, Ctrl+O/Esc on the native dialog, palette to Settings. Esc on a Sheet returns focus to its opener (policy Sheet, settings-reset Sheet). Opening the Inspector puts focus on its Close; its close-return was not observed. | R7C O02–O06; R4 R08, R5 H07; R2 B04 | **Recording the full Tab order on every page, the visible ring, and the Inspector's close-return** → Q5 |
| H06 Narrator reads state words, a header row, an UNKNOWN badge | The accessibility tree names every interactive element; one `main`; tables named by caption. Nothing was listened to. | R7C S03 | **All of the listening** → Q6 (Narrator and NVDA) |
| H07 1440×900 and 1024×720, Light and Dark, each language | 1440 (Chinese, Light): every workspace. Dark (1440, Chinese): Analyze, Compare, Release, History only. English (1440): Compare, Release, History. 1024×720: the window reaches it (O01); overflow was checked only in Chinese at content scale 1.25 and 1.5, all seven pages (S01/S02). | R2–R5 smokes, R7C | **1024×720 at 100%: Chinese and English, Light and Dark, all seven pages; Dark on Overview, Help, Settings** → Q7 |
| H08 Drag an ELF **and its MAP together** from Explorer | ELF alone, real drag from Explorer: staged, analyzed, source unchanged | R7C D01 | **ELF + MAP together** → Q8 |
| H09 Analyze › Objects: sort, filter `libdrv`, page | The Objects view lists 4 rows including `imu`. | R2 B06 | **Sort by Flash, RAM, name; filter; page; compare with `fwsight analyze --objects`** → Q9 |
| H10 Same ELF without MAP › Objects | Not observed natively; component tests only | `details.test.tsx` | **All** → Q10 |
| H11 Compare › Objects › Libraries, a shortlist entry drills down | Objects view: 3 rows including `imu`; a ranking pick filters its table (Sections) | R3 C02, C03 | **Libraries, `libdrv.a(spi.o)` Added with "Not present", drill-down** → Q11 |
| H12 Overview "Latest comparison" → Open Compare | Overview checked before any comparison | R2 B08 | **All** → Q12 |
| H13 H01's flows across theme × language × size | Not observed | — | Folded into Q1 as a reduced matrix (§4) |
| H14 No write without a confirmed Replace | Not observed; M1's unexplained overwrite is still unexplained | `evidence/M1/README.md` | **Watch during Q1 and Q2** → Q1, Q2 |

Since R7 there are new surfaces the D2C runbook never listed. They are added below as Q13–Q16: the pinned baseline, Markdown export, the policy Sheet, and real system DPI.

## 3. The checklist (only what needs a person)

Each case records: expected, actual, verdict (`PASS` / `FAIL` / `PASS_WITH_LIMITS` / `NOT_RUN`), UTC time, operator role. Use sentinel files: a known file at the target path whose SHA-256 is recorded before and checked after.

| # | Case | Steps | Pass means |
|---|---|---|---|
| Q1 | Export overwrite and failure (H01, H13, H14) | From Analyze (PNG, Markdown), Compare (HTML, Markdown) and Release (PNG, Markdown): (a) save over a sentinel, **deny** at the Windows prompt; (b) same, accept the Windows prompt, **Keep** at the app prompt; (c) same, **Replace**; (d) save into the non-writable folder; (e) after (d), save to a writable folder | (a)(b) sentinel bytes unchanged; (c) exactly the chosen file replaced; (d) an error panel with what happened, why, next step and a diagnostics id, sentinel untouched, no partial file; (e) succeeds. No write ever happens while a prompt is still open (H14). |
| Q2 | Stale replace (H02, H14) | Compare, Export HTML over a sentinel, reach the app's replace prompt; in the window behind it run Compare again; then press Replace | Refused as stale; sentinel unchanged |
| Q3 | English replace confirmation (H03) | Settings › Language English; repeat Q1(b) once | Title, body and buttons in English; Keep and Replace distinct |
| Q4 | Loading in each language (H04) | Analyze a large ELF (or from a slow disk) in Chinese, then English | Progress text localized; no untranslated string |
| Q5 | Keyboard order (H05) | From launch, Tab through each workspace (Overview, Analyze, Compare, Release, History, Help, Settings) and write down the order; Shift-Tab back; Esc on Inspector, Sheet, palette and native dialogs | Order matches the visual order; the focus ring is visible at every stop; no trap; Esc returns focus to the opener |
| Q6 | Screen reader (H06) | With **Narrator**, and with **NVDA** if installed by the operator: open Release after a Gate run; read the verdict, one finding row, an UNKNOWN badge and its note, a table header row; open the Inspector | Each state is announced as a word (PASS, REVIEW, BLOCK, UNKNOWN, N/A), never only as colour; headers announced with cells; the Inspector is announced as a dialog or complementary region and focus moves into it |
| Q7 | The minimum window (H07) | Window at 1024×720, scale 100%: Chinese then English, each in Light then Dark, on all seven pages (Overview, Analyze, Compare, Release, History, Help, Settings) | No page-level sideways scroll; no clipped state word or button label |
| Q8 | ELF + MAP drag (H08) | Select `firmware.elf` and `firmware.map` of `m2-objects/target` in Explorer and drag both onto the window | One analysis runs; the SubjectBar names both files; no path shown |
| Q9 | Objects table (H09) | Analyze `m2-objects/target` with its MAP › Objects: sort by Flash, RAM, name; filter `libdrv`; page if more than one page | Rows match `fwsight analyze --objects`; shares add up; the "not attributed" line is present |
| Q10 | Objects without MAP (H10) | Analyze the same ELF without a MAP › Objects | UNKNOWN with the no-MAP reason and the attach-MAP next step; no table of zeros |
| Q11 | Libraries in Compare (H11) | Compare `m2-objects` base → target › Objects › Libraries; click a shortlist entry | `libdrv.a(spi.o)` Added, base "Not present"; the entry filters the table |
| Q12 | Overview after Compare (H12) | After Q11, Overview › Latest comparison › Open Compare | The same pair and the same Flash change as Compare |
| Q13 | Pinned baseline | History › pin `m2-objects/base`; restart the app; Compare; then unpin | The pin survives a restart; Compare opens with the pinned base; unpin leaves a record and changes no stored build |
| Q14 | Markdown in a real viewer | Open the three Markdown exports from Q1 in GitHub, VS Code preview or another GFM viewer | Tables render; no HTML shown; the answer line is first; no local path |
| Q15 | Policy Sheet | Release › Edit policy; change one budget; Cancel; reopen; change and Save; run Gate | Cancel keeps the old value; Save changes the run; Esc closes and returns focus |
| Q16 | Real system DPI | Windows Settings › Display › Scale 125%, sign out and in, repeat Q7 in Chinese and one Save dialog; then 150%; restore the original | Same as Q7; the native dialogs and the title bar scale; the minimum window is still usable |

## 4. Reduced matrix for H13

The D2C runbook asked for every theme × language × size. One of each axis value is enough, provided each appears with a failing path:

| Run | Theme | Language | Size | Flow |
|---|---|---|---|---|
| M1 | Light | Chinese | 1440×900 | Q1 (a)–(e) |
| M2 | Dark | English | 1024×720 | Q1 (b), (d) |
| M3 | System | English | 1440×900 | Q1 (c) |

## 5. Pilot minimum (what must pass before any Pilot build is handed out)

From [PILOT_PROTOCOL.md](../../PILOT_VALIDATION/PILOT_PROTOCOL.md) §3. A participant's files and their trust are at stake, so data safety comes first:

| Required | Why |
|---|---|
| **Q1, Q2** PASS (or PASS_WITH_LIMITS with the Owner's written acceptance) | No export may overwrite a participant's file without a confirmed Replace (H14 is still unexplained) |
| **Q8** PASS | Drag-and-drop is the first thing many people try |
| **Q16** at 125% PASS | Laptops ship at 125–150%; the minimum window is 1024 logical px |
| Installation on the non-Owner account, with the installer SHA recorded | The participant installs the same bytes |

Q3–Q7 and Q9–Q15 are wanted before the Pilot but do not block it. Q6 (screen reader) blocks only a session with a participant who uses one.

## 6. Record

Use [QA_RECORD_TEMPLATE.tsv](QA_RECORD_TEMPLATE.tsv). The evidence rules are those of the D2C runbook: a summary and an AC matrix in Git, screenshots and the screen recording in a hashed ZIP, raw logs outside Git, and the first result kept beside any re-run.
