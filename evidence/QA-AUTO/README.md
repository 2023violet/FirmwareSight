# Automated QA for the release minimum (Q1, Q2, Q8)

The Owner decided on 2026-10-18 (`.ai/DECISIONS.md` OS2) that automated native QA, driven with real Windows input, stands in for the human pass before release. This run covers the export-safety and drag-and-drop cases of the [human QA pack](../D2C-R/HUMAN_QA_PACK.md) §5.

- **Build:** the QA-identifier build of `feature/oss-apache-2.0` (based on `main` `abace77`). Binary SHA-256 `4cc12577ad3058cda8b47a418d2611fbd6e81809bd6ec4fc2cb1385913f33d89`.
- **How:** the Windows Save dialog, its "确认另存为" overwrite prompt and its own refusals, and the app's "替换已有文件？" prompt are answered with real button clicks (posted `BM_CLICK` on the dialogs' own buttons) against the QA process only. The Explorer drag uses the real mouse. CDP only reads the page. Sentinel files sit in a scratch folder.

**Result:** 12 PASS, 0 FAIL, 2 NOT_RUN. See [AC_MATRIX.tsv](AC_MATRIX.tsv).

| Case | Observed |
|---|---|
| Q1a | Deny at Windows' overwrite prompt, then cancel: the file is unchanged; "已取消导出。未写入任何文件。" |
| Q1b | Accept Windows' prompt, choose 保留原文件 in the app: the file is unchanged; "保留了已经存在的文件。" |
| Q1c | 替换: exactly the chosen file is replaced, with no temporary file left behind |
| Q1d1 | A folder without write access: Windows' Save dialog refuses it before the app gets a path; nothing is written |
| Q1d2 | A read-only target: Windows refuses it as well |
| Q1d3 | A target another program holds open: the app's replace fails with an error panel (`ERR-EXPORT-6001`, the OS reason, the next step, a diagnostics id); the file is unchanged and no temporary file is left |
| Q1d4 | The same export after the file is released: replaced |
| Q1e | A retry to a new name: written |
| Q2 | A new comparison completes while the replace prompt is open; 替换 is then refused as stale; the file is unchanged |
| Q8 | `firmware.elf` and `firmware.map` dragged together from Explorer: one analysis with both files; no path on screen; the files are unchanged |

**Driver faults on the way, kept.** Four earlier runs failed because of the driver, not the product:
1. UI Automation does not see owned windows such as Windows' overwrite prompt.
2. A PowerShell variable-name clash (`$title` / `$Title`).
3. The prompt buttons are exposed as panes without an Invoke pattern.
4. A click by text hit the rail instead of the Compare button.

Their results are in the ZIP under `results/earlier-*`.

**Known limitation, by design.** In the Chinese UI the error panel's "下一步" (remediation) is the engine's English sentence. ADR-0032 D6 shows the engine's words as written and translates the labels and the diagnosis line.

**Not run, and why:**
- **Q16, real system DPI:** changing system display settings is outside this session.
- **Q6, screen-reader listening:** no screen-reader output can be observed.

Both are disclosed as untested. The automated stand-ins are in [evidence/R7C](../R7C/README.md).

The Owner's `%APPDATA%\com.firmwaresight.desktop` was only hashed (23 files, identical before and after). QA stores `q1-kept-*`, `q8-kept-*` and `dbg-kept-*` are kept.

**Evidence ZIP.** `QA_AUTO_EVIDENCE.zip` holds the driver scripts, the final and earlier results, and the Explorer-drop screenshot, with no local paths. SHA-256 `ac7ff10483ff23a5279eec07f19e9566d94fb119519ed1e6dd9faea55f4a23cb`.
