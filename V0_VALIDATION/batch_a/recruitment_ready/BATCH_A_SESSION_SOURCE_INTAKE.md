---
title: "Batch A Session Source Intake"
doc_id: "FS-V0-BA-013"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-09-27"
---

# Batch A Session Source Intake

Use this when a real session is returned to the project.

## Accepted source types

At least one:
- live moderated notes;
- consented transcript;
- consented recording plus notes;
- participant-submitted test record.

## Source classification

Record one:
- `LIVE_MODERATED_NOTES`
- `VERBATIM_TRANSCRIPT`
- `CONSENTED_RECORDING`
- `SECONDARY_SESSION_SUMMARY`

A secondary summary cannot create a verbatim quote unless the quote is present in the original source.

## Intake checklist

- [ ] real human participant verified
- [ ] eligibility checked
- [ ] not project/design team
- [ ] prototype version known
- [ ] session date known
- [ ] recording consent status known
- [ ] task evidence can be mapped to T1–T10
- [ ] private/company-sensitive material removed or protected
- [ ] source is not synthetic Persona output

## Synthetic suspicion

If an input looks like a fully generated “ideal participant answer” without evidence of a real session:

mark:
`SUSPECTED_SYNTHETIC`

Do not include it in Formal N until the human session source is established.
