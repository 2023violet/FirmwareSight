# D4-IAR: the IAR ILINK MAP adapter checked against real IAR builds

**Date:** 2026-10-18. **Branch:** `feature/iar-ilink-map`. **Decision:** [ADR-0038](../../09_ADR/ADR-0038-iar-ilink-map-adapter.md).

## Where the samples come from

No IAR toolchain was installed. The samples are build outputs that open-source projects committed to GitHub. Each one is a linked `.out` and the `.map` that ILINK wrote in the same link, taken at the commit where both files last changed together. Only permissively licensed repositories were used (MIT, Apache-2.0, BSD-3-Clause), and nothing from them is committed here. The table identifies each file so the check can be repeated: download `https://raw.githubusercontent.com/<repository>/<commit>/<path>`, then run `python scripts/crosscheck_iar_samples.py <folder> <fwsight>`.

| Sample | Repository @ commit | Licence | IAR ILINK | Target |
|---|---|---|---|---|
| A | `iliasam/STM32_DisplayLink` @ `74e4966` | MIT | 7.50.2 | STM32F407, LVGL, CCM RAM |
| B | `chcbaram/arm_seminar_fw` @ `40a68f4` | Apache-2.0 | 8.11.2 | STM32F746 bootloader, two RAM ranges |
| C | `eydonmlau/LoRaWAN_AT-Module` @ `1130eef` | Apache-2.0 | 8.20.1 | STM32L1 LoRaWAN |
| D | `gym487/STM32UltrasonicRADAR` @ `cd0522d` | MIT | 8.30.1 | STM32F103 |
| E | `adamcovaklara/LSM6DSOX-SensorHub-Demo` @ `a9b9168` | BSD-3-Clause | 8.40.2 | STM32F401, lz77-compressed initializers |
| F | `nicokorn/STM32F1XX_WS2812B` @ `8b03b06` | MIT | 9.10.1 | STM32F103 |
| G | `nicokorn/STM32F4XX_RNDIS_DEMO` @ `4553082` | MIT | 9.10.1 | STM32F4, USB RNDIS |
| H | `nicokorn/STM32H7XX_WS2812B` @ `94188fd` | MIT | 9.10.1 | STM32H743, absolute-address RAM sections |
| I | `blues/sparrow-lora` @ `34be2fc` | MIT | 9.20.2 | STM32WL, fixed-order ROM block |

| Sample | Path of the `.out` (the `.map` is beside it under `List/`) | `.out` SHA-256 | `.map` SHA-256 |
|---|---|---|---|
| A | `LVGL_Src/EWARM/firmware/Exe/firmware.out` | `e0dea10a…692b436d` | `153faf90…eee11529` |
| B | `opencr/cr_bootloader/prj/iar/Debug/Exe/cr_bootloader.out` | `74fdb6c4…e8055843` | `5ff5aebb…3c5f9e00` |
| C | `Debug/Exe/LoRaMac_AT-Module.out` | `e05b1d86…524b0786` | `d7341b93…97113815` |
| D | `EWARM/sn/Exe/sn.out` | `f9e9f107…b5c3d0ae` | `bfe76cfd…3ebfbe86` |
| E | `EWARM/IKS0LAB_SensorHub/Exe/IKS0LAB_SensorHub.out` | `478d6c80…954a9f26` | `e33637e5…0ae0d797` |
| F | `EWARM/WS2812B/Exe/WS2812B.out` | `adb54664…2093bc85` | `089e5cd8…7f9f3155` |
| G | `EWARM/ITAT_RNDIS/Exe/ITAT_RNDIS.out` | `8c2b9a4d…0a852f08` | `2e8c9135…697bf24a` |
| H | `EWARM/STM32H7XX_WS2812B/Exe/STM32H7XX_WS2812B.out` | `8df99006…242e30d0` | `fb29b5bd…8891e1c0` |
| I | `Application/EWARM/sparrow/Exe/sparrow.out` | `0fa055cd…6193f7d6` | `a2ab085b…41dc8a9b` |

The full digests are in [CROSSCHECK.txt](CROSSCHECK.txt).

## Results against IAR's own figures

IAR prints its totals at the end of the map (`readonly code`, `readonly data`, `readwrite data`) and per module in its `MODULE SUMMARY`. These are the reference.

| Sample | IAR Flash / RAM | FirmwareSight with the MAP | Modules: IAR / FirmwareSight / exact |
|---|---|---|---|
| A | 356883 / 124699 | 356883 / 124701 | 183 / 183 / 173 |
| B | 31667 / 47945 | 31667 / 47946 | 78 / 78 / 72 |
| C | 57496 / 8533 | 57496 / 8536 | 97 / 97 / 86 |
| D | 198620 / 14445 | 198620 / 14448 | 50 / 50 / 47 |
| E | 30483 / 3580 | 30483 / 3582 | 62 / 62 / 58 |
| F | 5623 / 2324 | 5623 / 2324 | 23 / 23 / 21 |
| G | 71098 / 129046 | 71098 / 129048 | 89 / 89 / 76 |
| H | 17988 / 1966 (1478 + 488 absolute) | 17988 / 1968 | 24 / 24 / 21 |
| I | 110247 / 65530 | 110247 / 65530 | 129 / 129 / 115 |

- **Flash is identical on all nine.** IAR keeps the stored copy of initialized data (`Initializer bytes`, lz77-compressed in E) inside the read-only block, so the ELF already accounts what Flash holds.
- **RAM is 0–3 bytes higher.** The ELF's zero-init section ends on an alignment boundary, while IAR counts the bytes placed in it. In E, `P2 zi` is 0x340 in the ELF and holds 0x33e. FirmwareSight reports the span the section occupies.
- **Every module and library member IAR lists is present in FirmwareSight, and nothing extra appears.** Library members are keyed `rt7M_tl.a(strlen.o)`.
- **Every module's RAM matches IAR's,** with one exception. In H, `ws2812b.o` also owns 488 bytes of absolute-address RAM (D2 SRAM), which IAR lists apart ("+ 488 absolute").
- **Where a module's Flash differs, the difference is its share of the `Initializer bytes`.** IAR charges that share to the module's `ro data`. FirmwareSight keeps it with the linker's own bytes, because the map does not give each module's share once it is compressed.
  - In A, B, E, F, G and I, the differences add up exactly to the `Initializer bytes`.
  - In C, D and H, they are 1, 3 and 2 bytes short of it.
- **Regions** are the `place in [from .. to ..]` ranges of the linker configuration. Examples: P1 0x08000000–0x080FFFFF as Flash and P2 0x20000000–0x2001FFFF as RAM in A; two separate RAM ranges in B (P2, P3).

## Limits

- Only IAR Embedded Workbench **for Arm** is read. ILINK for other targets (STM8, RISC-V, RX…) is refused with that reason.
- Regions are the placement ranges, not the `.icf` region names, which the map does not print. Ranges that share an address are one region (`P1+P3`).
- Every object row of the nine samples found the ELF section that holds it, so no module figure above rests on an Unknown charge.
- Sections at absolute addresses outside every placement range have no capacity. Their bytes still count, through ELF flags.
- An object name is not qualified by its object directory, so two objects with one name in two directories share a key.
- All nine samples are STM32 Cortex-M builds with IAR's default runtime libraries. Other vendors, C++-heavy projects and multi-image builds have not been seen.

## Desktop: real Windows input on the shipped window

A QA-identifier build of this branch was used (`com.firmwaresight.desktop.qa`, binary SHA-256 `1d34aabee317167a5fa57d66f0cefd12acb71570841eb1a5f719fbfafa9f0080`). Each `.out` and its map were opened through the real Open dialogs. The QA store is kept. The Owner's app data was only hashed, and it was unchanged.

| AC | Verdict | Observed |
|---|---|---|
| I01 | PASS | Sample G (ILINK 9.10) with its map: "Flash 已用 71,098 bytes · RAM 129,048 bytes（精确）", no alert; the build stores |
| I02 | PASS | Region `P1`, non-volatile, capacity 393216 bytes, from `place in [from 0x800'0000 to 0x805'ffff]` |
| I03 | PASS | Objects view: `httpserver.o`, `FreeRTOS_TCP_IP.o`, `tasks.o`… with known Flash and RAM, no Unknown |
| I04 | PASS | Sample A (ILINK 7.50) with its map: "Flash 已用 356,883 bytes · RAM 124,701 bytes（精确）", no alert |
| I05 | PASS | Owner app data unchanged (23 files, identical hashes) |

Screenshots and `results.tsv`: `D4-IAR_native_smoke.zip`, SHA-256 `505d7024a1cf4eb6d2d837a90c4d58e53d3b9dca84449f648fe93303d50fe21c`.
