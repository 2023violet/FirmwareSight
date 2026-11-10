---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-11-09"
---

# Active Task

```text
D8_TRENDS_HISTORY
```

- **Authorized by:** the Owner in chat on 2026-11-09 (approved plan; `.ai/DECISIONS.md` F1–F6; "Start D8").
- **Branch:** `feature/d8-trends` from `main` `3e88513`.
- **What it does:** ADR-0041 and ADR-0042:
  - migration 0009 (per-region usage per build, no backfill);
  - the History Trends tab, with Flash/RAM lines, region headroom and object movers;
  - Overview names the last stored build after a restart;
  - `fwsight analyze --record [--store FILE]`.
- **Last task:** `D9_CI_INTEGRATION`, merged at `3e88513` (CI 11/11). **Next:** v0.9.0.

The history of earlier pointers is in [archive/2026-10-14/ACTIVE_TASK.md](archive/2026-10-14/ACTIVE_TASK.md).
