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
- **Local gate at `76b5c2c`:** `python scripts/check.py` 18/18 steps, 0 SKIP. Rust workspace 1,160 passed / 0 failed / 2 opt-in ignored. Frontend 397 tests in 13 files.

## Gates

| Gate | State |
|---|---|
| Code integration (NEXT-47) | PASS |
| Native desktop user acceptance | **NOT_VERIFIED / BLOCKED_BY_ENV** |
| Public release, signing, updater, open-source license | NOT_AUTHORIZED |

## Open risks (must close before the first external Developer Preview, Private Beta or RC, whichever comes first)

- NEXT-44 AC24: native Save, cancel, overwrite refusal, write failure, and stale/retry flows.
- NEXT-44 AC26: native keyboard, focus, accessibility, Light/Dark theme and minimum window size.
- NEXT-45 AC20: localized native save and overwrite flow.

Retest steps and exit conditions: [deferred native QA register](../NEXT47_VALIDATION/evidence/04_NEXT47_DEFERRED_NATIVE_QA.md).

## Compatibility boundary

GCC + GNU ld and Clang + GNU ld are verified cohorts. Keil (armlink) and IAR (ILINK) compiler output is **UNVERIFIED**. ArmLink and IAR MAP files are **NOT_SUPPORTED / FIXTURE_BLOCKED**. See [04_TECH/39](../04_TECH/39_ELF_TOOLCHAIN_INTEROPERABILITY.md).

## Protected local state

The Owner's original `main` checkout stays at `e34f9e6` with nine staged documentation files. It is intentionally behind the remote. Do not reset, commit or push from it without an explicit Owner decision.

## Active task

[ACTIVE_TASK.md](ACTIVE_TASK.md).
