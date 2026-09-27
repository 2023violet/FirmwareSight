---
title: "ADR-0014 GPU Deferred and Native Island"
doc_id: "ADR-0014"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Decision
No `wgpu` in MVP.

Use:
- normal Web UI；
- SVG；
- Canvas；
- query/pagination/virtualization。

wgpu can be introduced only as a contained Native/GPU Island after benchmark evidence and ADR.

# Why
The supplied Rust research explicitly concludes GPU should be used where GPU matters, not as a default UI architecture.
