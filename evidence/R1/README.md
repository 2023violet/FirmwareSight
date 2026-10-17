# R1 foundation: native smoke

Plan: [06_DELIVERY/15_R1_FOUNDATION.md](../../06_DELIVERY/15_R1_FOUNDATION.md) §4. The run covers the R1 code after the independent review fixes. The QA binary is a `cargo tauri build --no-bundle` with the identifier `com.firmwaresight.desktop.qa`. Its SHA-256 is `3fbf4450d5daba41480ba226b1876d5828238e52ec9e02ed3ac7f9e348d76972`.

**Operator.** Claude acted as an automated driver on the Owner's Windows 10 host (zh-CN, WebView2). Page input went through the WebView2 DevTools protocol, and window sizes were set with Win32 `MoveWindow`.

**Data handling.**
- The QA build started from an empty store.
- The previous QA store was moved aside and then restored.
- This run's store is kept as `com.firmwaresight.desktop.qa.r1-final-kept`.
- The Owner's store was only hashed, never opened.

**Result:** 9 PASS, 1 NOT_RUN. See [AC_MATRIX.tsv](AC_MATRIX.tsv).

- **Pages and rail.** Each of the seven rail entries opens its page under its existing accessible name. The rail collapses to 56 px below 1180 px and expands again above it. The reader's toggle works at 1440 px.
- **Default window.** It now opens 1280 wide, so the rail starts expanded. At the old 1040 width every first launch started collapsed (review S2-3).
- **Tokens and theme.** Tokens 0.4.0 are live (weight 600, figure 28 px, light `diff.removed` `#C7372C`). The dark palette follows the system preference.
- **Not run: window minimum.** The 1024×720 minimum is set in `tauri.conf.json`, and a unit test ties it to the token file. It still needs one human drag-resize, because a programmatic resize bypasses the window minimum.

`R1_NATIVE_SMOKE_EVIDENCE.zip` contains four screenshots (default light, collapsed at 1100 px, Help states in light, Help in dark), `results.tsv` and the three driver scripts. Its SHA-256 is `62950f25f1d8a659366a3366b3ffe36734b90b8fee4a6dfbefa851c7186a64c4`. Local paths were stripped before packing.
