# R4 Release workbench: native smoke

Plan: [06_DELIVERY/18_R4_RELEASE.md](../../06_DELIVERY/18_R4_RELEASE.md) §4.

- **Build:** the QA-identifier build (`com.firmwaresight.desktop.qa`) of the final R4 code. Binary SHA-256 `8f75166b0a67c26530244571cc8b3bedf4dd45a4646679aea2fa917bbd22e296`.
- **Operator:** Claude, as an automated driver on the Owner's Windows 10 host (zh-CN, WebView2).
  - Page input went through the WebView2 DevTools protocol.
  - The real native Open and Save dialogs were driven with Win32 messages, against the QA process only.
  - Window sizes were set with `MoveWindow`.
- **Input:** `fixtures/elf/m2-objects/{base,target}`, each an ELF plus its MAP. Both were analyzed, then the target was gated with the base as its baseline under the stated default policy (no project config loaded).

**Result:** 14 PASS, 1 NOT_RUN. See [AC_MATRIX.tsv](AC_MATRIX.tsv). The run covers:
- both languages and both themes;
- 1440 px and 1100 px wide;
- the Save dialog's Cancel path.

**Fixed during the run.** The first run (also all PASS) showed one wording defect:
- **The defect.** The verdict read "Needs a decision — 0 finding(s) wait on an explicit acceptance". The REVIEW disposition came from two UNKNOWN findings that `[gate.on_unknown]` turns into reviews, and the sentence had counted only REVIEW-state rows. The BLOCK sentence had the same flaw.
- **The fix.** The sentence now counts the rows whose effective severity is the verdict's word, which is Core's own field on each row, and says how many of those rows are missing evidence.
- **Checks.** A regression test was added, and the final run (R02) shows the corrected sentence.

**QA stores.** Each run's store is kept and none is deleted (Standing Owner rule: no deletion outside an approved scope):
- `com.firmwaresight.desktop.qa.r4-kept-025945`
- `com.firmwaresight.desktop.qa.r4-kept-030807` (final run)

The Owner's store was only hashed (R14).

**Evidence ZIP.** `R4_NATIVE_SMOKE_EVIDENCE.zip` holds ten screenshots, `results.tsv` and the four driver scripts, with no local paths. SHA-256 `f2ecaf58ac12f2d84ff83ce29c6811fbba308b4f30e364b0c514b4fb000e788f`.
