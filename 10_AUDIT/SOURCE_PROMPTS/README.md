---
title: "Execution Prompt Register"
doc_id: "FS-AUDIT-SRC-003"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Audit"
last_updated: "2026-09-28"
---

# Execution Prompt Register

Every authorized execution prompt is preserved here verbatim. Each SHA-256 below was
recomputed from the archived copy in this directory, so the archived bytes match what was
delivered rather than a retranscription.

## V0 Workflow Prototype Validation

- File: `FirmwareSight_V0_Workflow_Prototype_Validation_PROMPT_v1.1.txt`
- SHA-256: `3431a688c65c08aa6104fe4cb785ef25b6ae9df4e2d3272dddda653704e1df20`
- Authority: execution instruction subordinate to the unique v0.5.0 product baseline, and authoritative for the authorized V0 task.

## V0 Batch A External Validation

- File: `FirmwareSight_V0_Batch_A_External_Validation_PROMPT_v1.0.txt`
- SHA-256: `e62c236b090de9da589bcf570415edd7a0296925910d65437b2105a9612589e7`
- Authority: execution instruction for the Batch A external validation, subordinate to the v0.5.1 baseline. It withheld `v0.5.2` until real Batch A evidence exists, and left P0 unauthorized in that execution.

## P0 Technical Vertical Slice

- File: `FirmwareSight_P0_Technical_Vertical_Slice_EXECUTION_PROMPT_v1.1_ARCHITECT_REVIEWED.txt`
- SHA-256: `60a59196708a53592be4d828a0c1275cf74b3bfad0bdc6455f863ab38d32929b`
- Size: 94802 bytes / 5064 lines
- Authority: execution instruction for the P0 engineering slice against the unique v0.5.1 baseline. Consistent with `09_ADR/ADR-0020-validation-sequence.md`, which already authorizes V0 and P0 as parallel tracks. It overrides only the execution state recorded by the Batch A prompt (`V0` blocking, `P0` not authorized); it does not override any frozen product, architecture, evidence, gate or design baseline, and it does not authorize P1.
- Provenance record: `P0_TECHNICAL_VALIDATION/P0_EXECUTION_PROVENANCE.md`
- Outcome, 2026-09-28: executed to its stop condition as `EXECUTED — CONDITIONAL_PASS (LOCAL)`.
  Baseline stays `v0.5.1`, `v0.6.0` withheld, G1 not claimed, P1 still unauthorized. The conditions
  are that CI and `cargo deny` have never run, which needs a push this environment is not
  authorized to make.
