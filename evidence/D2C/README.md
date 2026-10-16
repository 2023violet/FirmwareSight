# D2C — automated native QA of the M2 screens (C2)

Plan: [06_DELIVERY/13_D2C_CLOSEOUT_AND_PILOT.md](../../06_DELIVERY/13_D2C_CLOSEOUT_AND_PILOT.md). Code SHA `71f1e8b40fa0b1350f32e204831d35aa24e1cee6`, which includes the C1 escaping fix. QA binary SHA-256 `c831b16ec2922ece26d3a809075f0c5dcc90b04823bb9a261cf2cf60300eecd1` (QA-identifier build).

- [AC_MATRIX.tsv](AC_MATRIX.tsv): 12 recorded results for 11 cases, each with expected, actual, verdict and UTC.
- [D2C_NATIVE_QA_EVIDENCE.zip](D2C_NATIVE_QA_EVIDENCE.zip), 520,102 bytes, SHA-256 `5b431aed87b82220eae232dfbf030fe86aeca788f98ebdd9a01e19a43299064a`. It contains the results, the environment, the Owner app-data hashes before and after, six screenshots, the drivers, and `COVERAGE_NOTES.md`. Local user names are replaced by `LOCAL_USER`.
- [HUMAN_NATIVE_QA_RUNBOOK.md](HUMAN_NATIVE_QA_RUNBOOK.md): the human pass this does **not** replace.

**Operator:** Claude, as an automated driver, on the Owner's Windows 10 host (120 dpi, zh-CN, WebView2 155). Page input went through the WebView2 DevTools protocol. The real native file pickers were driven with Win32 messages against the QA process only. The QA build used its own identifier and started from an empty store. The M1 QA store was moved aside and restored afterwards, and this pass's store is kept as `com.firmwaresight.desktop.qa.d2c-kept`. The Owner's 23 app-data files had identical hashes before and after. Inputs were `fixtures/elf/m2-objects/{base,target}` only.

| Case | What | Result |
|---|---|---|
| A05 | Objects tab, build without a MAP | PASS: UNKNOWN badge, no-MAP reason, attach-MAP next step, no table |
| A01 | Objects tab, `m2-objects/target` with its MAP | PASS: 4 objects, Flash descending (`sensors/imu.o` 80 bytes, 40.81 %), "not attributed" line |
| A02 | Sort by RAM, then by name | PASS: both orders come back from the shell |
| A03 | Filter `libdrv` | PASS: exactly the two `libdrv.a` members |
| A04 | Libraries tab | PASS: one row, `libdrv.a`, 40 bytes Flash and 36 bytes RAM |
| A06 | Compare "By object and library" | PASS: "对象：2 个变化，1 个新增，0 个删除，1 个未变"; `libdrv.a(spi.o)` Added with base "Not present" |
| A07 | Shortlist drill-down | PASS: picking `sensors/imu.o` filters the table to that row |
| A08 | Libraries view of the changes | PASS: `libdrv.a` Changed, Flash +28 bytes, RAM +32 bytes |
| A09 | Overview "Latest comparison" | **FAIL on the first run.** The driver's selector read only the panel's header block; the screenshot shows the full panel |
| A09-rerun | Same, selector corrected | PASS: both builds, Flash +64 bytes, RAM +32 bytes, `exact`, `sensors/imu.o +36 bytes`. ("Open Compare returns to the comparison" was observed in the first A09 run, which the rerun did not repeat.) |
| A10 | Objects tab at 1024×720, dark scheme | PASS: no sideways page scroll |
| A11 | UI rows against `fwsight analyze --objects` | PASS: all 4 rows match (Flash, RAM, MAP rows) |

**Not run although the plan listed them:** object-table paging (the fixture has 4 objects, below one page) and `fwsight diff` text inside this pass (covered instead by `apps/cli/tests/objects_cli.rs` in the gate).

**Not shown** (see `COVERAGE_NOTES.md`):
- keyboard-only use and the focus ring;
- a screen reader;
- the app's own theme setting (A10 emulated the OS colour scheme);
- 1440×900;
- the English UI;
- a human reading the screens;
- a non-Owner profile;
- native drag and drop.

These are H01–H12 in the human runbook. This pass does not touch export dialogs, so AC24, AC26 and AC20 keep their PASS_WITH_LIMITS state.
