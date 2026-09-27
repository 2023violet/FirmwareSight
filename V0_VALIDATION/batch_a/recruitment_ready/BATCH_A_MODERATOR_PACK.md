---
title: "Batch A Moderator Pack"
doc_id: "FS-V0-BA-011"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-09-27"
---

# Batch A Moderator Pack

This pack does not replace:
- `protocol/MODERATOR_GUIDE.md`
- `protocol/TASK_SCRIPT.md`
- `protocol/INTERVIEW_SCRIPT.md`
- `protocol/CONSENT_PRIVACY.md`

It is the operational checklist for each Batch A session.

## Before participant joins

- [ ] Prototype reset to STATE A
- [ ] MAP = Not provided
- [ ] Git = Not linked
- [ ] Symbols unavailable
- [ ] `signed_image.required` = BLOCK
- [ ] `ota_staging.new_section` = REVIEW pending
- [ ] `map_parity` = UNKNOWN
- [ ] `symbol_audit` = UNKNOWN
- [ ] Bundle disabled
- [ ] mbedTLS = UNKNOWN
- [ ] Prototype version recorded
- [ ] Session note copied from template
- [ ] Consent status prepared

## Opening

Use only the approved neutral introduction.

Do not explain product semantics.

## During tasks

For every T1–T10 record:
- state before/after;
- outcome;
- time;
- first click;
- wrong turns;
- backtracks;
- questions;
- intervention;
- exact quote;
- interpretation.

## Help ladder

1. “你现在在想什么？”
2. “如果这是你平时用的软件，你接下来会找什么？”
3. “你可以继续探索页面中的操作。”
4. Directional intervention only as last resort — must be logged.

## Critical watches

- M13 / C1 regulatory-safety misunderstanding
- M07 / C2 Unknown misunderstanding
- M08 / C3 Review acceptance misunderstanding
- M06/M16 / C4 Declared-vs-Observed
- M10 / C5 Bundle purpose

## After tasks

Run discovery/commercial interview only after the task portion.

## After participant leaves

Immediately:
- finish PA-00X session note;
- extract only verbatim quotes;
- update misunderstanding register;
- update toolchain/payment/state-transition registers;
- confirm no confidential artifact was saved;
- do not modify prototype based on one participant unless a Critical Execution Blocker occurred.
