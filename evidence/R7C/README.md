# R7 closeout: the checks that once needed a person

Plan: [06_DELIVERY/21_R7_QUALITY.md](../../06_DELIVERY/21_R7_QUALITY.md) §3 and Q6.

- **Build:** the QA-identifier build of `main` (`e1faeff`) plus Q6 (skip link, focus to the page heading). Binary SHA-256 `97ec95143be31757ff140cee8c02796b1094fb2e3231a1f92047fcc69e929684`.
- **Operator:** Claude, on the Owner's Windows 10 host (zh-CN, 100% scale, WebView2). Unlike the earlier smokes, the actions here go through the Windows input queue: `mouse_event`/`SetCursorPos` for the drags and `keybd_event` for the keys. CDP only reads the page. The Explorer drag used a scratch copy of the fixture.

**Result:** 66 PASS, 0 FAIL, 2 NOT_RUN, 3 SUPERSEDED. See [AC_MATRIX.tsv](AC_MATRIX.tsv).
- **OS-O01:** a real drag of the sizing corner stopped the client at 1024×720. This closes R1 A10; the three carried smoke rows are marked SUPERSEDED.
- **OS-O02..O06:** a keyboard-only flow with real keys.
- **DND-D01:** a real drag of `firmware.elf` from Explorer: staged, analyzed, no alert, source unchanged.
- **SCALE-S01/S02:** content scale 1.25 and 1.5 at the minimum window, seven pages, no overflow. This is not system DPI.
- **SCALE-S03:** the accessibility tree UI Automation reads: every interactive element named, landmarks and captioned tables. This is not a person listening.
- **R2..R6 smokes** re-run on this build: 53 PASS.
- **Left to a person:** listening with NVDA and Narrator, and real 125%/150% system scaling.

**First attempts, kept.** The first Explorer run (`dnd-kept-192009`) dropped and analyzed the file, as its screenshot shows, but the driver's read-back expression had a quoting error, so it recorded FAIL. The first scaling run (`scale-kept-192345`) used the managed UI Automation client, which cannot read landmarks; the audit moved to the accessibility tree.

**QA stores.** Kept, none deleted:
- `com.firmwaresight.desktop.qa.os-kept-191032`
- `com.firmwaresight.desktop.qa.r2-kept-191153`
- `com.firmwaresight.desktop.qa.r3-kept-191337`
- `com.firmwaresight.desktop.qa.r4-kept-191514`
- `com.firmwaresight.desktop.qa.r5-kept-191648`
- `com.firmwaresight.desktop.qa.r6-kept-191813`
- `com.firmwaresight.desktop.qa.dnd-kept-192009`
- `com.firmwaresight.desktop.qa.dnd-kept-192134`
- `com.firmwaresight.desktop.qa.scale-kept-192345`
- `com.firmwaresight.desktop.qa.scale-kept-192623`

The Owner's `%APPDATA%\com.firmwaresight.desktop` was only hashed, before and after each driver: 23 files, identical hashes every time.

**Evidence ZIP.** `R7C_CLOSEOUT_EVIDENCE.zip` holds nine screenshots, eight `results.tsv` files and seven driver scripts, with no local paths. SHA-256 `0dccfea220873d23ef801e04be6abc0874e38b14cc38a35e68a015ec500c80bf`.
