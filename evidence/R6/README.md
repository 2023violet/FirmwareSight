# R6 Reports: native smoke

Plan: [06_DELIVERY/20_R6_REPORTS.md](../../06_DELIVERY/20_R6_REPORTS.md).

- **Build:** the QA-identifier build of the R6 code after the review fixes (`95a22fc`). Binary SHA-256 `84bafdca3035928fd3bbcac4987381454dd49173166d29a0c25b84a9154b25ef`. The first final run (`f2daf2b`, binary `060e4c49…`) is superseded; its store is kept below.
- **Operator:** Claude, as an automated driver on the Owner's Windows 10 host (zh-CN, WebView2). The native Open and Save dialogs were driven with Win32 messages, against the QA process only.
- **Input:** `fixtures/elf/m2-objects/{base,target}`, analyzed, compared and gated under the stated default policy.

**Result:** 5 PASS. See [AC_MATRIX.tsv](AC_MATRIX.tsv). On each of Analyze, Compare and Release:
- **Export ▸ Export Markdown** wrote a GFM report through the real Save dialog;
- the file was read back and checked: masthead, figures, no HTML, no path, and N/A rows without a severity on the Gate report.

A cancelled export wrote nothing and raised no error, and the Owner's data was unchanged.

**Fixed during the run.** The first run reported FAIL (`NO_EDIT`) because the smoke driver could find only an Open dialog's file field. It now also finds a Save dialog's file-name field, which sits inside a combo box. The product did not change.

**Written reports.** The three Markdown files the final run wrote are in the ZIP under `reports/`. They are kept as evidence.

**QA stores.** Kept, none deleted:
- `com.firmwaresight.desktop.qa.r6-kept-084048` (the driver-fault run)
- `com.firmwaresight.desktop.qa.r6-kept-084243` (final run before the review fixes)
- `com.firmwaresight.desktop.qa.r6-kept-091332` (final run after the review fixes)

**Evidence ZIP.** `R6_NATIVE_SMOKE_EVIDENCE.zip` holds two screenshots, the three Markdown reports, `results.tsv` and the four driver scripts, with no local paths. SHA-256 `9c3a6efaf0f8f422f90bad30a88693fa144cf35703ca823f9be334589ad9427b`.

**After the review fixes.** The smoke was run again on the fixed build: 5 PASS. The Gate report the desktop writes now carries the stored disposition; with no acceptance on this run it reads as before. An accepted run is covered by the shell test `the_gate_markdown_reports_the_disposition_and_the_acceptance_as_an_audit_record`, not by this smoke.
