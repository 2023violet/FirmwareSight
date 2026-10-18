---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-10-18"
---

# Active Task

```text
V07_ARMLINK_REAL_IAR_SIGNING
```

- **Authorized by:** the Owner in chat on 2026-10-18 (`.ai/DECISIONS.md` V7-1 to V7-3).
- **Branch:** `feature/v0.7-armlink-iar-signing` from `main` `3f8fa7d`.
- **What it does:**
  - armlink checked on two real Keil MDK projects; compressed RW data fixed (ADR-0036 A1, `evidence/D4-REAL`);
  - Compare pairs a repeated section name by role (A2);
  - signed build-provenance attestations for every package from `main` (ADR-0037);
  - version 0.7.0.
- **Blocked:** IAR (no real samples).
- **Next:** one Owner confirmation to push, then one to publish v0.7.0.
- **Last task:** `OSS_01_APACHE_2_OPEN_SOURCE` with D4, on `main` at `3f8fa7d`, released as v0.6.0.

The history of earlier pointers is in [archive/2026-10-14/ACTIVE_TASK.md](archive/2026-10-14/ACTIVE_TASK.md).
