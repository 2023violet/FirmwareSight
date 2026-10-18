---
title: "Pilot Session Template"
doc_id: "FS-PL-003"
product: "FirmwareSight"
version: "0.6.0"
status: "TEMPLATE"
owner: "Research"
last_updated: "2026-10-17"
---

# Pilot Session — `PL-XX`

> Template only. Copy it to `sessions/PL-XX.md` after a session that actually happened. Fill every field from real notes or a transcript, or mark it `NOT_RECORDED`. Before `git add`, run the scan in `V1_VALIDATION/protocol/CONSENT_PRIVACY.md`.

## Participant (anonymized)

| Field | Value |
|---|---|
| Session ID | `PL-XX` |
| Entry rules A, B, C, D, E, F | |
| Pool decision recorded (permanently out of formal V1) | YES / NO (NO = do not hold the session) |
| Cohort | `IN_COHORT` (GCC/Clang + GNU ld) / `OUT_OF_COHORT` (toolchain named) |
| Role family · experience band · team-size band | |
| Current way of comparing builds | |

## Own artifact (repo-safe facts only)

| Field | Value |
|---|---|
| `OWN_ARTIFACT` | YES / NO |
| Toolchain category · ecosystem category · size band | |
| `MAP` | YES / NO |
| Second build of the same project | YES / NO |

## Consent and build

| Field | Value |
|---|---|
| Consent (V1 items + the Pilot sentence) obtained before the session | YES / NO (NO = do not hold the session) |
| Unsigned-build disclosure given | YES / NO |
| Recording | AUDIO / SCREEN / VIDEO / NOTES_ONLY |
| Build commit · installer SHA-256 | |
| Install friction (T0-style notes) | |

## Tasks

| Task | Outcome | First path | Wrong turns / stalls | Interventions (and why) |
|---|---|---|---|---|
| PT1 First look | | | | |
| PT2 Growth source | | | | |
| PT3 Two builds | | | | |
| PT3b Reference build (pin) | | | | |
| PT4 Hand-off | | | | |

PT3b detail: pin found unprompted (YES / NO / not applicable) · where they looked first · time to first pin · next Compare opened on the pinned build (YES / NO) · unpinning understood (YES / NO / UNCLEAR).

PT4 detail: format chosen (Markdown / HTML / PNG / SVG / JSON / screenshot / other) · where it would go (PR / issue / chat / mail / other) · opened before sending (YES / NO) · remarks on the Markdown, verbatim.

## Debrief

Keep quotes verbatim and in quotation marks. Mark anything else `MODERATOR_NOTE — NOT VERBATIM QUOTE`.

1. Build comparison today:
2. New about own build:
3. Expected but missing:
4. Other toolchains and linkers:
5. Stack, components or symbols needed recently:
6. Would open again (verbatim; not V1 M6):

## Route coding

| Question | YES / NO / UNCLEAR | Evidence line |
|---|---|---|
| Pinned baseline found and understood (D3, shipped) | | |
| Repeat comparison | | |
| Markdown fitted the hand-off | | |
| Object/library view answered the growth question | | |
| Toolchain outside GNU ld (D4) | | |
| Deeper evidence wanted (D5): say whether it came up unprompted during the tasks or only at debrief question 5 | | |

## Defects found (filed without participant data)

| Issue | Steps | Build |
|---|---|---|
