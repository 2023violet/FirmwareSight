# M1 — First-use experience: delivery and native QA evidence

Spec: [06_DELIVERY/10_M1_FIRST_USE_EXPERIENCE.md](../../06_DELIVERY/10_M1_FIRST_USE_EXPERIENCE.md). Base `cee270c`. Native QA ran on code SHA `2b0f9331ee07cecd2bea48f041a04ebe64afc586`, binary SHA-256 `efccb5e9a3af149842600e183ab0ef6e869419cfb6d7a171695b5d83ecaab2df` (QA identifier build). One later commit only shortens the History page intro text in both catalogs and touches no behaviour.

- [AC_MATRIX.tsv](AC_MATRIX.tsv): 27 recorded cases, each with expected, actual, verdict and UTC time.
- [M1_NATIVE_QA_EVIDENCE.zip](M1_NATIVE_QA_EVIDENCE.zip), SHA-256 `253e1b79ef3b38ed3719808ef1aaafb379e4310e7147b926e72c1921ad1a0e4a`. It contains the results, the environment, the Owner app-data hashes before and after, screenshots, and the automation drivers with the local user name replaced by `LOCAL_USER`.

## What shipped

| Task | Result |
|---|---|
| T1 copy and jargon | Page intros meet the targets (≤ 120 English characters, ≤ 50 Chinese), measured below. Getting started is three numbered actions; the seven reference answers stay in Help. Primary-surface jargon is replaced; Core's state and evidence words are unchanged. |
| T2 drag and drop | Rust holds the dropped paths behind a single-use token; the WebView sees names only. Refusals cover folders, more than two files and ambiguous pairs. After a successful drop, Analyze runs once. |
| T3 memory bars | Rust computes `usedPercent` (integer arithmetic; `<0.1%` for usage too small to round up; a lower bound when usage is partial; explicit unknown statuses). Overview draws one bar per region with an icon and a word. |
| T4 Release order | The setup reads Step 1 build, Step 2 policy, Step 3 attachments, then a sticky Run Gate bar. Results keep the U1P order, and keyboard focus moves to the verdict after a run. |
| T5 tables | Scroll areas are bounded to the minimum window height and have sticky headers, so the horizontal scrollbar is always visible. |
| T6 native dialog language | The UI sends display-only dialog text, which Rust bounds and checks against an English fallback. Rust still does not read the language preference (ADR-0032). |

Intro length, before → after (characters): Analyze 116→100 / 61→36 · Compare 118→58 / 47→21 · Gate 220→103 / 87→29 · History 344→120 / 116→50 · Settings 151→99 / 71→36. Hints: Compare export 427→179 / 191→93, Release attachments 289→164 / 115→52, ranking basis 403→203 / 150→74 (en / zh).

## Native QA (T7): NEXT44-AC24, NEXT44-AC26, NEXT45-AC20

Operator: Claude, as an automated driver, on the Owner's Windows 10 host (120 dpi, zh-CN, WebView2 155). Page input went through the WebView2 DevTools protocol. Real native dialogs were driven with Win32 messages and keystrokes, and focus and roles were read through Windows UI Automation. The QA build used its own identifier, so its data directory was separate. The Owner's 23 app-data files had identical hashes before and after.

| AC | Cases | Observed |
|---|---|---|
| AC24 save / cancel / refuse overwrite / write failure / stale / retry | C06–C16 | **PASS** on the final build. The OS overwrite prompt and the app's own confirmation can each refuse; refusing leaves the sentinel bytes unchanged. Confirming replaces only the target and leaves no temp file. A locked target gives `ERR-EXPORT-6001` with a diagnostics id and the bytes unchanged, and a retry succeeds. If the result changes while the Save dialog is open, Compare, Analyze and Gate each refuse to write and say so. |
| AC26 keyboard, icon+word statuses, themes, minimum window | C04, C15, C19, C20, C22 | **PASS**. Tab order follows the visual order, with a 2 px focus ring. Enter opens the native Save, Esc cancels it, and focus returns to the control that opened it. UI Automation reports Button "导出 SVG", keyboard-focusable. Dark theme at 1024×720 shows no sideways page scroll on any of the six pages. Status words appear beside their icons. |
| AC20 localized controls, error, cancellation, save, overwrite | C01, C03, C06, C07, C09, C17, C21 | **PASS**, with the limits below. Native titles, filters and the replace confirmation follow the UI language in zh-CN and en. |

**Still open:**
- **Independent sign-off:** the deferred register's exit condition needs it, and it has not been done.
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

## Design checklist (`templates/DESIGN_CHECKLIST_TEMPLATE.md`), this round's changes

- **Don'ts:** no gradient, backdrop filter, shadow, raw colour or raw pixel value in any changed CSS (checked by grep). `generate_design_tokens.py --contrast` shows no new FAIL; only the existing documented exemption remains.
- **States:** shown as icon + word. Unknown is neutral grey with a hollow icon, a fact and a next step. Bars do not use colour alone.
- **Interaction and motion:** focus is visible; motion is limited to the micro token and stops under reduced motion.
- **Logic boundary:** no parser, diff or Gate logic in the UI; percentages come from Rust.
