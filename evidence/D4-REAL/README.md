# D4-REAL: armlink support checked against real Keil MDK projects

**Date:** 2026-10-18. **Branch:** `feature/v0.7-armlink-iar-signing`. **Decision:** ADR-0036 amendments A1 and A2.

## What was built

Two example projects that ship in Arm's own CMSIS packs were built unmodified, with µVision's own settings, on the Owner's licensed toolchain. Nothing was downloaded. No vendor build output is committed; the digests below identify it.

- **Toolchain:** Keil MDK Plus 5.43, Arm Compiler for Embedded 6.24.
- **Build command:** `UV4.exe -b <project>.uvprojx -t <target> -j0 -o build.log`, run in the project directory. µVision passes `--map --load_addr_map_info` by default.

| Project | Source | Shape |
|---|---|---|
| `rtx-blinky` | `ARM/CMSIS-RTX/5.9.0/Examples/Blinky` (project file identical to the pack's) | RTX5 RTOS, microlib, Cortex-M; `UNINIT` regions `RW_NOINIT` and `RW_EVR`, `ARM_LIB_HEAP`, `ARM_LIB_STACK` |
| `dsp-fir` | `ARM/CMSIS-DSP/1.15.0/Examples/ARM/arm_fir_example` (µVision migrated the device pack to `ARM.Cortex_DFP.1.1.0` when opening it) | CMSIS-DSP, the standard Arm C library (`c_w.l`, `fz_wm.l`, `m_wm.l`); **compressed RW data** (`COMPRESSED[0x1ac]`) |

| File | SHA-256 |
|---|---|
| `rtx-blinky/Objects/Blinky.axf` | `4f197a5511b5a9581489e69df2ed260b2ae5eb35e1c471dd348a892517fb31ec` |
| `rtx-blinky/Listings/Blinky.map` | `d62ae16e94fa9e0de7d9413e394771954742dd9576270ddec2e5985429488a0f` |
| `dsp-fir/ARMCM4_debug/arm_fir_example.axf` | `02f9c8bbea6423dbfb8d61726499e8ee57bcfa52012cd9d758e935aa7f871883` |
| `dsp-fir/ARMCM4_debug/arm_fir_example.map` | `899debb54e01388518f90c75a06682c23985b60d1cec964ad4f38546ba4c1d34` |

## Results against armlink's own totals

armlink's `Total ROM Size` and `Total RW Size` lines in the MAP are the reference.

| Project | armlink ROM / RW | FirmwareSight, ELF + MAP | FirmwareSight, ELF only |
|---|---|---|---|
| `rtx-blinky` | 11620 / 10992 | Flash **11624**, RAM **10992**, exact | Flash 11624, RAM 10992, exact |
| `dsp-fir` (before A1) | 6904 / 7776 | Flash short by the 428 stored bytes (load address unknown), RAM 5632 | Flash 6904, RAM 5632 reported as exact |
| `dsp-fir` (after A1) | 6904 / 7776 | Flash **6904**, RAM **7776**, exact | Flash 6904, RAM "5204 bytes or more, 1 section unattributed", derived |

- **`rtx-blinky` Flash, 4 bytes over armlink's total:** the load region `LR_ROM` is `Size: 0x00002d68` (11624) in the same MAP. armlink's "Total ROM Size" leaves out the 4 bytes of alignment padding between `ER_ROM` and the RW data. The load region size is what the device stores, so 11624 is kept.
- **`dsp-fir` before A1, two real defects:**
  - Flash undercount: the compressed region's load address was reported unknown, so its 428 stored bytes were left out of Flash.
  - RAM wrongly exact: the ELF section of compressed RW data holds the compressed bytes (428), not the 2572 bytes it occupies at runtime.
- **`dsp-fir` after A1:**
  - the stored bytes are charged from the region's `Load base`;
  - the runtime size comes from the MAP region's `Size`;
  - in the object view, rows inside the compressed region show Flash as Unknown and RAM as their size. Only the region's total stored size is known, not each row's share.
- **ELF only:** armlink's decompressor (`__decompress`) is in the image, so the runtime size of every initialized writable section is Unknown rather than the stored size. RAM is then a lower bound, never exact.

## Compare: repeated `RW_IRAM1` (A2)

armlink names both the RW data and the ZI output section after their execution region. The committed fixture pair `fixtures/elf/d4-armlink` used to show `RW_IRAM1` as 2 added and 2 removed rows. Pairing a repeated name by role (`InitializedData` / `UninitializedData`) now gives `Sections +0 -0 ~9 (0 rows whose name repeats)`, with `RW_IRAM1` +32 bytes as a Changed row. Rows whose name *and* role repeat on one side still stay unpaired. Tests: `diff/tests.rs`, `apps/cli/tests/objects_cli.rs`.

## Still open

- Real armlink projects with several load regions, overlays, or more than one compressed region: not seen.
- The armlink cohort is two real projects plus one synthetic sample. That is broader evidence, still not a large cohort (`04_TECH/39`).

## Desktop: real Windows input on the shipped window

A QA-identifier build of this branch was used (`com.firmwaresight.desktop.qa`, binary SHA-256 `6457881f3a1626600b36277cd7c708931159b7f357d95b5f5b882a5a5cf3c8ce`). Each AXF and its MAP were opened through the real Open dialogs. The QA store is kept. The Owner's app data was only hashed, and it was unchanged.

| AC | Verdict | Observed |
|---|---|---|
| K01 | PASS | RTX5 Blinky with its MAP: "Flash 已用 11,624 bytes · RAM 10,992 bytes（精确）", no alert |
| K02 | PASS | CMSIS-DSP FIR with its MAP: "Flash 已用 6,904 bytes · RAM 7,776 bytes（精确）", no alert |
| K03 | PASS | Objects view, row in the compressed region: `arm_fir_data.o · 应用 · Flash Unknown · RAM 2,560 bytes` |
| K04 | PASS | Owner app data unchanged (23 files, identical hashes) |
| A01–A05 | PASS | Regression on the synthetic fixture: Analyze 192 / 44, region usage, objects, Compare ▲84 bytes, Owner data unchanged |

**Defect found by the first desktop run, and fixed.**
- Storing either real build failed with `ERR-STORAGE-4006` ("CHECK constraint failed: size > 0").
- The cause: armlink lists zero-size rows, such as the C library's `.ARM.Collect$$$$...` markers, and the adapter passed them on as object contributions.
- They are now skipped, as GNU ld's zero-size rows already were. Test: `map_armlink::zero_size_rows_are_not_contributions`.
- The CLI never stores builds, so only the desktop run could show this.

Screenshots and `results.tsv` for both runs: `D4-REAL_native_smoke.zip`, SHA-256 `710f61aa28ce50e295500cd11d377a0ad90199bb576660f2c10320c3629f8537`.
