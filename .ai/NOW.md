---
title: "Current Project Status"
doc_id: "FS-AI-NOW"
product: "FirmwareSight"
version: "0.6.0"
status: "CURRENT_INDEX"
owner: "Engineering"
last_updated: "2026-10-14"
---

# FirmwareSight Now

This file is overwritten, never appended. History lives in [archive/](archive/2026-10-14/INDEX.md). Read Git and Actions for the live HEAD before making any claim. The SHAs below were last verified on 2026-10-14.

## Code and CI

- **Last integrated code:** `76b5c2c` (NEXT-46 export privacy and stale-result fixes, integrated by NEXT-47). [Actions #113](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036): 10/10 Jobs and 10/10 Check Runs.
- **Later documentation-only commits:** `71978d8` (NEXT-47 evidence upload, [Actions #114](https://github.com/2023violet/FirmwareSight/actions/runs/38602220781) 10/10). The 2026-10-14 lean batch follows; read Git for its SHA and CI.
- **M1 first-use round (branch `feature/m1-first-use`, not yet on `main`):** copy cut, drag-and-drop intake, Overview memory bars, Release step order, table scrolling, native dialog language, and six native-QA fixes. Evidence: [evidence/M1](../evidence/M1/README.md).
- **Local gate at `76b5c2c`:** `python scripts/check.py` 18/18 steps, 0 SKIP. Rust workspace 1,160 passed / 0 failed / 2 opt-in ignored. Frontend 397 tests in 13 files.

## Gates

| Gate | State |
|---|---|
| Code integration (NEXT-47) | PASS |
| Native desktop user acceptance | **PASS_WITH_LIMITS** (M1, automated driver; independent review); gaps listed in [evidence/M1](../evidence/M1/README.md); native drag-and-drop NOT_RUN |
| Public release, signing, updater, open-source license | NOT_AUTHORIZED |

## Open risks (must close before the first external Developer Preview, Private Beta or RC, whichever comes first)

- NEXT-44 AC24, NEXT-44 AC26, NEXT-45 AC20: rated PASS_WITH_LIMITS by the independent review of M1 and still open. Not yet observed: deny, confirm, failure and retry on a visual export; the English replace confirmation; recorded Tab order; a screen reader; 1440×900; the non-Owner profile and human operator the [deferred register](../NEXT47_VALIDATION/evidence/04_NEXT47_DEFERRED_NATIVE_QA.md) asks for. See [evidence/M1](../evidence/M1/README.md).
- Native drag-and-drop from Explorer: NOT_RUN (automation could not synthesize an OS drag); one human drag is needed.
- Screen-reader check: not run; UI Automation exposure was checked.

## Compatibility boundary

GCC + GNU ld and Clang + GNU ld are verified cohorts. Keil (armlink) and IAR (ILINK) compiler output is **UNVERIFIED**. ArmLink and IAR MAP files are **NOT_SUPPORTED / FIXTURE_BLOCKED**. See [04_TECH/39](../04_TECH/39_ELF_TOOLCHAIN_INTEROPERABILITY.md).

## Protected local state

The Owner's original `main` checkout stays at `e34f9e6` with nine staged documentation files. It is intentionally behind the remote. Do not reset, commit or push from it without an explicit Owner decision.

## Active task

[ACTIVE_TASK.md](ACTIVE_TASK.md).
