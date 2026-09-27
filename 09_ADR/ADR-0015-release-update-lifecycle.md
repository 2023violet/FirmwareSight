---
title: "ADR-0015 Release and Update Lifecycle"
doc_id: "ADR-0015"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Decision
Design FirmwareSight delivery from the start as:
Build → Test → Bundle → Sign → Installer → Release → Update Metadata → Signed Update.

Auto-update is not enabled in MVP.

# Trigger to enable
- signing key management；
- CI secret isolation；
- release hosting；
- rollback；
- offline disable switch；
- platform package validation。

# Consequence
No late-stage architecture surprise, without prematurely shipping a risky updater.
