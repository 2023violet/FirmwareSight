# M1 — First-use experience: delivery and native QA evidence

Spec: [06_DELIVERY/10_M1_FIRST_USE_EXPERIENCE.md](../../06_DELIVERY/10_M1_FIRST_USE_EXPERIENCE.md). Base `cee270c`. Native QA ran on code SHA `2b0f9331ee07cecd2bea48f041a04ebe64afc586`, binary SHA-256 `efccb5e9a3af149842600e183ab0ef6e869419cfb6d7a171695b5d83ecaab2df` (QA identifier build). Later commits only shorten the History intro text, and then apply the review fixes listed under "Review fixes" below; native QA was not re-run after them.

- [AC_MATRIX.tsv](AC_MATRIX.tsv): 27 recorded cases, each with expected, actual, verdict and UTC time.
- [M1_NATIVE_QA_EVIDENCE.zip](M1_NATIVE_QA_EVIDENCE.zip) (revision 2), SHA-256 `00d901dd8ab4dec8327ef9c8c40deabee9ceee1b1355f3d524499fc63e117add`. It contains the results, the environment, the Owner app-data hashes before and after, screenshots, the drivers with the local user name replaced by `LOCAL_USER`, the hand-run follow-up commands (`drivers/followups.sh`) with their raw output, and `COVERAGE_NOTES.md`, which lists what is not shown.

## What shipped

| Task | Result |
|---|---|
| T1 copy and jargon | Page intros meet the targets (≤ 120 English characters, ≤ 50 Chinese), measured below. Getting started is three numbered actions; the seven reference answers stay in Help. Primary-surface jargon is replaced; Core's state and evidence words are unchanged. |
| T2 drag and drop | Rust holds the dropped paths behind a single-use token; the WebView sees names only. Refusals cover folders, more than two files and ambiguous pairs. After a successful drop, Analyze runs once. |
| T3 memory bars | Rust computes `usedPercent` (integer arithmetic; `<0.1%` and `>99.9%` at the ends; partial usage is a lower bound rounded down; explicit unknown statuses). Overview draws one bar per region with an icon and a word. **Not done:** T3's "latest Compare delta on Overview" was not built; it is carried to a later round. |
| T4 Release order | The setup reads Step 1 build, Step 2 policy, Step 3 attachments, then a sticky Run Gate bar. Results keep the U1P order, and keyboard focus moves to the verdict after a run. |
| T5 tables | Scroll areas are bounded to the minimum window height and have sticky headers, so the horizontal scrollbar is always visible. |
| T6 native dialog language | The UI sends display-only dialog text, which Rust bounds and checks against an English fallback. Rust still does not read the language preference (ADR-0032). |

Intro length, before → after (characters): Analyze 116→100 / 61→36 · Compare 118→58 / 47→21 · Gate 220→103 / 87→29 · History 344→120 / 116→50 · Settings 151→99 / 71→36. Hints: Compare export 427→179 / 191→93, Release attachments 289→164 / 115→52, ranking basis 403→203 / 150→74 (en / zh).

## Native QA (T7): NEXT44-AC24, NEXT44-AC26, NEXT45-AC20

Operator: Claude, as an automated driver, on the Owner's Windows 10 host (120 dpi, zh-CN, WebView2 155). Page input went through the WebView2 DevTools protocol. Real native dialogs were driven with Win32 messages and keystrokes, and focus and roles were read through Windows UI Automation. The QA build used its own identifier, so its data directory was separate. The Owner's 23 app-data files had identical hashes before and after.

The pass ran C01–C22 in one script. C11 and C20 **failed on the first attempt** (the script truncated the alert text; the harness could not take the foreground) and passed when re-run by hand; C23–C25 were also run by hand. The commands and raw output are in the ZIP. The independent review rated all three ACs **PASS_WITH_LIMITS**.

| AC | Cases | Observed on the real dialogs | Not shown |
|---|---|---|---|
| AC24 save / cancel / refuse overwrite / write failure / stale / retry | C06–C16 | Cancel; refusing at the OS prompt and at the app confirmation leaves the sentinel bytes unchanged; confirming replaces only the target with no temp file; a share-locked target gives `ERR-EXPORT-6001` with a diagnostics id and unchanged bytes; retry succeeds; Compare, Analyze and Gate each refuse a stale Save and say so. | Deny, confirm, failure and retry were all driven through **Compare JSON**, not through a visual export. A stale Compare JSON/HTML during the replace confirmation was found by review (F1) and fixed afterwards, but not re-observed natively. Failure used a locked file, not a non-writable directory. |
| AC26 keyboard, icon+word statuses, themes, minimum window | C04, C15, C19, C20, C22 | Enter opens the native Save, Esc cancels it and focus returns to the opener (C19); UI Automation reports Button "导出 SVG", keyboard-focusable (C20, rerun); dark theme at 1024×720 shows no sideways scroll on six pages (C22); status words sit beside icons in the screenshots. | Tab / Shift-Tab order and the focus ring were seen only during exploration and are not recorded. No screen reader. No theme × language × size repetition of the Save, deny and error flows. No 1440×900. |
| AC20 localized controls, error, cancellation, save, overwrite | C01, C03, C06, C07, C09, C11, C17, C21 | zh: picker titles, Save title and filter, the replace confirmation's title and buttons, the cancellation note, the error panel. en: Save title and filter. | The English replace confirmation was not opened. No case covers a loading state. The error's next-step line is Rust English (ADR-0032 D6). |

**Still open:**
- **Closing the deferred ACs:** the independent review rated them PASS_WITH_LIMITS. The gaps in the table above have to be observed before AC24, AC26 and AC20 are closed.
- **Environment:** the register asks for a dedicated non-Owner Windows profile and a human operator; this run used an automated driver on the Owner's profile, isolated by a separate app identifier.
- **Assistive technology:** no screen reader was used, only UI Automation.
- **Native drag and drop (C25):** NOT_RUN. Automation could not synthesise an OS-level OLE drag on this host after three attempts. One human drag is needed.
- **One unexplained event during exploration:** a sentinel was overwritten while two prompts were open and no automation action had been sent. I could not reproduce it in three attempts. The code path is fail-closed: the dialog plugin maps a failed dialog to cancel. The most likely cause is a person clicking the visible window.

**Known limits, not defects of this round:**
- The OS draws its own button labels in the OS language.
- The file picker's "All Files (*.*)" filter is the dialog library's default.
- An error's "what to do" line is Rust's English sentence (ADR-0032 D6 keeps Rust as the single source).

## Defects found by native QA, fixed in this round

| # | Defect | Fix commit |
|---|---|---|
| Q1 | Compare exports wrote the old comparison if the reader compared again while the Save dialog was open | `5003155` |
| Q2 | Equal confirm/keep labels from the UI would make Keep act as Replace (the plugin decides by label) | `5003155` |
| Q3 | Tiny usage shown as `0.0%`; lowercase format in the Save filter; Getting started steps unnumbered | `5003155` |
| Q4 | Analyze and Gate stale refusals were silent | `7e9e3ab`, `2b0f933` |
| Q5 | The sticky Run Gate bar sat under a sticky table header and could not be clicked | `8cd3332` |
| Q6 | Focus fell to the page body after a native dialog closed | `8cd3332`, `2b0f933` |

## Review fixes (after the independent review of `674e7aa`)

| Finding | Fix |
|---|---|
| F1 (S2) | Compare JSON/HTML now re-checks that the comparison is still current **after** the replace confirmation (`write_export_checked`), like the visual exports. Regression test; the test fails when the re-check is removed. |
| F2 (S2) | T3's Compare delta on Overview is recorded as not done, here and in the spec. |
| F3 (S2) | This page, `COVERAGE_NOTES.md` and `.ai/NOW.md` now state only what the raw cases show; rerun commands and raw output were added to the ZIP. |
| F4 (S3) | The Overview bar keeps Core's word `Unknown` in zh, matching Analyze (ADR-0032 D5). |
| F5 (S3) | Partial usage rounds down; a region that is not full shows `>99.9%`, never `100.0%`. |
| F8 (S3) | Restored "capacities can remain UNKNOWN" and the "product-compliance conclusion" disclaimer. |
| F9 (S3) | `diff_is_current` no longer takes over `with_diff`'s doc comment. |

Not changed in this round: F6 (swapped confirm/keep labels from a compromised WebView), F7 (Tauri's own drag events carry full paths to any listener; the app does not subscribe), F10 (1440×900 shots).

## Design checklist (`templates/DESIGN_CHECKLIST_TEMPLATE.md`), this round's changes

- **Don'ts:** no gradient, backdrop filter, shadow, raw colour or raw pixel value in any changed CSS (checked by grep). `generate_design_tokens.py --contrast` shows no new FAIL; only the existing documented exemption remains.
- **States:** shown as icon + word. Unknown is neutral grey with a hollow icon, a fact and a next step. Bars do not use colour alone.
- **Interaction and motion:** focus is visible; motion is limited to the micro token and stops under reduced motion.
- **Logic boundary:** no parser, diff or Gate logic in the UI; percentages come from Rust.
