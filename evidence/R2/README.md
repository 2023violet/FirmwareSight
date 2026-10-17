# R2 Analyze and Overview: native smoke

Plan: [06_DELIVERY/16_R2_ANALYZE_OVERVIEW.md](../../06_DELIVERY/16_R2_ANALYZE_OVERVIEW.md) §4. The run covers the final R2 code. The QA binary is a `cargo tauri build --no-bundle` with the identifier `com.firmwaresight.desktop.qa`. Its SHA-256 is `8841a07606737d2f3390197ca464e705ce6ea5f91298f1402690fef8e4cc779a`.

**Operator.** Claude acted as an automated driver on the Owner's Windows 10 host (zh-CN, WebView2).
- Page input went through the WebView2 DevTools protocol.
- The real native file dialogs were driven with Win32 messages, against the QA process only.
- Window sizes were set with `MoveWindow`.
- The input was `fixtures/elf/m2-objects/target` (ELF and MAP).

**Data handling.**
- The QA build started from an empty store.
- The previous QA store was moved aside and then restored.
- This run's store is kept as `com.firmwaresight.desktop.qa.r2-kept-<time>`.
- The Owner's store was only hashed, never opened.

**Result:** 10 PASS, 1 NOT_RUN. See [AC_MATRIX.tsv](AC_MATRIX.tsv).

**Fixed during the run.** The smoke exposed four layout defects, and each was fixed before this final run:
- the icon in the Export button stacked vertically;
- the lead read figure-first, and now reads "Flash used 196 bytes";
- the SubjectBar chips overlapped next to the docked Inspector, and the bar now wraps to a second row instead;
- the docked Inspector stopped at its content, and is now a full-height sticky column.

`R2_NATIVE_SMOKE_EVIDENCE.zip` holds:
- six screenshots: Analyze Memory at 1440 px, the docked Inspector, the Inspector as a sheet at 1100 px, the Evidence view, Overview, and Analyze in dark;
- `results.tsv`;
- the four driver scripts.

The ZIP's SHA-256 is `dec4355b7e7c9ce2f6c9b2dd376ce7cb9673f3377fe5e0080013869b398c3e06`. Local paths were stripped before packing.
