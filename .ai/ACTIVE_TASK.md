---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-10-17"
---

# Active Task

```text
R7_QUALITY
```

- **Authorized by:** the Owner's standing goal in chat on 2026-10-17: carry the plan through to the complete product without intermediate confirmations.
- **Plan and status:** [06_DELIVERY/21_R7_QUALITY.md](../06_DELIVERY/21_R7_QUALITY.md).
- **Base:** `feature/r6-reports` (`37ee635`). **Branch:** `feature/r7-quality`.
- **Stack, all local and not yet on `main`:** `feature/r4-release` → `feature/r5-history` → `feature/r6-reports` → `feature/r7-quality`. Each is a fast-forward of the one before it, and `feature/r4-release` fast-forwards `main` (`465fe5f`).
- **Waiting on the Owner:** the push of the stack to `main`. The session's permission guard refused the push; it stays an Owner action. Pushing `feature/r7-quality` alone fast-forwards `main` through all four phases.
- **Waiting on a person:** the human-only checks in [06_DELIVERY/21](../06_DELIVERY/21_R7_QUALITY.md) §3 (window-minimum drag, NVDA/Narrator, 125%/150% DPI, a keyboard-only pass, Explorer drag-and-drop).
- **Not in this task:** deleting data outside an approved scope, including the QA stores the smoke runs kept.

- **Earlier tasks in this stack:** `R6_REPORT_EDITION` (`feature/r6-reports`, `37ee635`), `R5_HISTORY_PIN_PALETTE` (`feature/r5-history`, `29b13bb`), `R4_RELEASE_WORKBENCH` (`feature/r4-release`, `2a81d4b`).

The history of earlier pointers is in [archive/2026-10-14/ACTIVE_TASK.md](archive/2026-10-14/ACTIVE_TASK.md).
