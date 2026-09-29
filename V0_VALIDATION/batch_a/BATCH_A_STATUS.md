---
title: "Batch A Status"
doc_id: "FS-V0-BA-002"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-09-29"
---

# Batch A Status

## Current state

# BATCH A WAITING FOR REAL PARTICIPANTS

Formal external sessions:
`0 / 4–5 Batch A target`

V0 total:
`0 / 8 minimum`

## Completed in this execution

- Batch A takeover;
- recruitment plan;
- screening register;
- scheduling template;
- moderator pack;
- session-source intake rules;
- evidence integrity checklist;
- Batch A evidence index scaffold.

## Not completed

- PA-001
- PA-002
- PA-003
- PA-004
- optional PA-005
- Batch A metrics
- Batch A misunderstanding synthesis
- Batch A commercial signal synthesis
- Batch A interim review

## Version consequence

Per the authorized Batch A Prompt:

> v0.5.2 may only be created after 4–5 real eligible external sessions and their evidence are written into the project.

Therefore this execution **does not create v0.5.2**.

## Version consequence, corrected 2026-09-29

The rule above was written before P0 was promoted, and it is kept as history. It no longer describes the
project's version path:

- `v0.5.2` was the **pre-P0 planned** Batch A evidence-patch version. The live baseline is
  `FirmwareSight_Project_Baseline_v0.6.0`.
- Batch A completion **does not generate `v0.5.2`**, and no older baseline is created after `v0.6.0`. The
  project version is never decremented to match a research document.
- Any version promotion after Batch A and its interim review is an **architect decision**, taken through
  the normal promotion path - not an automatic consequence of reaching `4-5` sessions.
- See `V0.5.2_RELEASE_BLOCK.md`, which carries the same note and stays named as it is.

## Execution state, 2026-09-29

`active_task` is now `V0_BATCH_A_EXTERNAL_VALIDATION`, activated by architect prompt. Nothing in this
section changed by that activation: `0` real eligible external sessions, all five screening rows still
`NOT_RECRUITED`, all five scheduling rows still `NOT_SCHEDULED`, and no `PA-00X.md` exists. Activation
wrote no participant evidence, because no participant had been met.
