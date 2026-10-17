# R5 History, pinned baseline, Settings, Help and command palette: native smoke

Plan: [06_DELIVERY/19_R5_HISTORY.md](../../06_DELIVERY/19_R5_HISTORY.md) §4.

- **Build:** the QA-identifier build (`com.firmwaresight.desktop.qa`) of the final R5 code, including the fixes from the independent review of `66ec11a`. Binary SHA-256 `91df7f0bf170fceeffeb3a8f2c67d2265ccea19aae58a54e15a4ee7cedef052f`.
- **Operator:** Claude, as an automated driver on the Owner's Windows 10 host (zh-CN, WebView2). Page input went through the WebView2 DevTools protocol. The native Open dialogs were driven with Win32 messages, against the QA process only.
- **Input:** `fixtures/elf/m2-objects/{base,target}`, each an ELF plus its MAP, both analyzed.

**Result:** 13 PASS, 1 NOT_RUN. See [AC_MATRIX.tsv](AC_MATRIX.tsv). The run covers:
- **History:** the new navigation names; pinning, the pin line and the pinned chip; a stored row in the Inspector; unpinning.
- **Compare:** a comparison started from History, run once, with the pinned base marked.
- **Command palette and shortcuts:** `Ctrl+K`, `]` and `/`.
- **Settings and Help:** the reset Sheet and Esc; the Help shortcut table.
- **Both themes and both languages.**

**Fixed during the run.** The first run (also all PASS) showed three visual defects. All were fixed before the final run:
- table values (byte counts, times) wrapped mid-value;
- the row buttons repeated the full build name in their visible text (it is now their accessible name only);
- the palette filled the window width.

**QA stores.** Kept, none deleted (Standing Owner rule):
- `com.firmwaresight.desktop.qa.r5-kept-065552`
- `com.firmwaresight.desktop.qa.r5-kept-070304`
- `com.firmwaresight.desktop.qa.r5-kept-074432` (final run, after the review fixes)

The Owner's store was only hashed (H13).

**Evidence ZIP.** `R5_NATIVE_SMOKE_EVIDENCE.zip` holds eight screenshots, `results.tsv` and the four driver scripts, with no local paths. SHA-256 `4eaee29505f0cce954a4c61bbc59e9491e949c608ce8269df671380f2d9a130a`.
