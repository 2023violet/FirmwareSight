# R6 Reports: native smoke

Plan: [06_DELIVERY/20_R6_REPORTS.md](../../06_DELIVERY/20_R6_REPORTS.md).

- **Build:** the QA-identifier build of the R6 code (`f2daf2b`). Binary SHA-256 `060e4c49c34004de77ee63d7d6501a46d75f1f31308874b7c94383b8b4bfbb3a`.
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
- `com.firmwaresight.desktop.qa.r6-kept-084243` (final run)

**Evidence ZIP.** `R6_NATIVE_SMOKE_EVIDENCE.zip` holds two screenshots, the three Markdown reports, `results.tsv` and the four driver scripts, with no local paths. SHA-256 `6dc6196625beaf79ad2a051e914ed1806c2d145ba528adf95c9857ae566dcec7`.
