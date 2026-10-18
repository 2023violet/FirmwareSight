# NEXT-CORE-01: section growth ranking, evidence

Plan and acceptance: [06_DELIVERY/25](../../06_DELIVERY/25_CORE_SECTION_RANKING_FIX.md). Matrix: [AC_MATRIX.tsv](AC_MATRIX.tsv).

- **Build:** the QA-identifier build of `feature/core-section-ranking-correctness` (the change set of this task, before the candidate commit). Binary SHA-256 `2800096fa8493649b0a33ab92b67fad30ada14d9045a854ba80c05857fd5bb83`.
- **Operator:** Claude, as an automated driver on the Owner's Windows 10 host (zh-CN, WebView2), against the QA process only. Native dialogs were driven with Win32 messages.
- **Inputs:** `fixtures/elf/m2-objects` (GCC + GNU ld, with MAPs) and `fixtures/elf/d4-armlink` (Arm Compiler 6.24, AXF only).

**Result:**
- **Native desktop:** 12 PASS after one driver fix. The first armlink run recorded A05 FAIL because the check looked for `ER_IROM1` in Markdown, which writes it `ER\_IROM1`. The exported file was correct, and the driver was fixed. Both runs are kept.
- **Core:** the first version of the new tests failed on the unchanged code (`results/core-tests-before-fix.txt`). The final tests fail against the old ranking behaviour restored by mutation (`results/core-tests-mutation.txt`). All of them pass on the fix.
- **Goldens and gates:** listed in the matrix.

**Not covered here:** a human pass, other locales, a real-project armlink build. The meaning of `Section.memory_size` for unloaded sections is unchanged (open, `06_DELIVERY/24` option 1).

**QA stores.** Kept, none deleted:
- `com.firmwaresight.desktop.qa.rank-m2-kept-230718`
- `com.firmwaresight.desktop.qa.rank-armlink-kept-230821` (the driver-fault run)
- `com.firmwaresight.desktop.qa.rank-armlink-kept-231000`

The Owner's `%APPDATA%\com.firmwaresight.desktop` was only hashed: 23 files, identical before and after each run.

**Evidence ZIP.** `NEXT-CORE-01_EVIDENCE.zip` holds two screenshots, the two Compare Markdown exports, the three smoke result files, the Core test output from before the fix and from the mutation check, and the four driver scripts. It contains no local paths. SHA-256 `566fd56cdbd39ed3d59dac24312bca0e8e68d981d9ec010bc55402b4fe7cc0de`.
