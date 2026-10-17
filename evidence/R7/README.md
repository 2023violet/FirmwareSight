# R7 Quality: native regression and printed exports

Plan: [06_DELIVERY/21_R7_QUALITY.md](../../06_DELIVERY/21_R7_QUALITY.md).

- **Build:** the QA-identifier build of `feature/r7-quality` (Q1–Q3 applied, before Q4). Binary SHA-256 `28fbfe0b95db70cc2a20d5ab9a1afe0745b9592dc325c19aff0e178fa5af36ee`.
- **Operator:** Claude, as an automated driver on the Owner's Windows 10 host (zh-CN, WebView2). The native dialogs were driven with Win32 messages, against the QA process only. The printed-page checks ran in a local headless Edge on the two report goldens, with no network.
- **Input:** `fixtures/elf/m2-objects/{base,target}`; the report goldens under `golden/reports/`.

**Result:** 55 PASS, 0 FAIL, 8 NOT_RUN. See [AC_MATRIX.tsv](AC_MATRIX.tsv).
- The R2, R3, R4, R5 and R6 smoke drivers, in order, on one build: 53 PASS. Their three NOT_RUN rows are the same human window-drag check, carried from R1. The build includes the token, stylesheet and catalog changes (Q1–Q3); it predates Q4, the HTML reports' `scope` attributes and captions, which the printed-page checks below cover on the regenerated goldens.
- Both HTML reports as A4 pages: `@page` A4, nothing wider than the sheet, every table header repeats, no row splits, every table captioned, every header cell scoped, no script, no external URL.
- Five human-only checks, listed and not run (`HUMAN-1`..`HUMAN-5`).

**Driver fix during the run.** The first R4 run on this build failed nine checks because the driver still clicked the rail name R5 retired ("发布门禁"). It now clicks "发布". The product did not change.

**QA stores.** Kept, none deleted:
- `com.firmwaresight.desktop.qa.r2-kept-092916`
- `com.firmwaresight.desktop.qa.r3-kept-093103`
- `com.firmwaresight.desktop.qa.r4-kept-093233` (the driver-fault run)
- `com.firmwaresight.desktop.qa.r4-kept-093432`
- `com.firmwaresight.desktop.qa.r5-kept-093555`
- `com.firmwaresight.desktop.qa.r6-kept-093717`

The Owner's `%APPDATA%\com.firmwaresight.desktop` was hashed before and after each driver and read nothing else: 23 files, identical hashes, five times.

**Evidence ZIP.** `R7_QUALITY_EVIDENCE.zip` holds twelve screenshots, each driver's `results.tsv`, the three Markdown reports, two printed-page screenshots with their results, and the nine driver scripts, with no local paths. SHA-256 `6ab741129024fd1b759755fbe8863dd05d2c8634ecdc04e1404772e87087edb8`.

See also the [design checklist](DESIGN_CHECKLIST.md).
