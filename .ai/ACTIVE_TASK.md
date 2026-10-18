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
OSS_01_APACHE_2_OPEN_SOURCE
```

- **Authorized by:** the Owner in chat on 2026-10-18: open source under Apache-2.0; automated QA stands in for human testing; continue with features worth adding; do not over-deliberate (`.ai/DECISIONS.md` OS1–OS3).
- **Branch:** `feature/oss-apache-2.0` from `main` `abace77`.
- **What it does:**
  - `LICENSE` (Apache-2.0), `NOTICE`, `CONTRIBUTING.md`, `SECURITY.md`;
  - `THIRD_PARTY_LICENSES.md` and its generator, plus a check in `check.py`;
  - `[workspace.package] license = "Apache-2.0"`;
  - ADR-0035;
  - the automated QA for Q1, Q2 and Q8 (`evidence/QA-AUTO`).
- **Next:** D4, the armlink MAP adapter. Then one Owner confirmation to push and publish a release.
- **Last task:** `CORE_SECTION_RANKING_CORRECTNESS`, on `main` at `abace77` (CI run `39104386478`, 10/10).

The history of earlier pointers is in [archive/2026-10-14/ACTIVE_TASK.md](archive/2026-10-14/ACTIVE_TASK.md).
