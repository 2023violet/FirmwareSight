---
title: "Exploratory Pilot"
doc_id: "FS-PL-000"
product: "FirmwareSight"
version: "0.6.0"
status: "PROTOCOL_APPROVED_IN_PRINCIPLE"
owner: "Research"
last_updated: "2026-10-17"
---

# Exploratory Pilot

Prepared in D2C (`06_DELIVERY/13` C4) and revised in D2C-R (`06_DELIVERY/23`). **No session has run, and the register says 0.** The Owner approved the direction in principle on 2026-10-17. Recruiting and installation wait for the entry conditions in the protocol §3: final approval, and the Pilot minimum of the human QA pack.

## What the Pilot is for

It puts the R7 build in front of a few real firmware engineers, with their own builds, to answer the questions that decide the next batches of `06_DELIVERY/12`:

- Do people find and understand the pinned baseline that R5 shipped (D3), and does it fit how often they compare builds?
- Do the object and library tables explain a real growth source well enough to act on?
- Which toolchains do the participants actually bring, and how many fall outside GNU ld (D4)?
- How do they hand findings to a teammate, and does the Markdown report fit where they paste it?
- Which deeper evidence do they ask for unprompted: stack usage, components, symbols (D5)?

## What it is not

- **Not V1.** Pilot sessions never count toward V1's 8 participants or any V1 metric M1–M6. Nothing here is reported as V1 passing or failing, or as any gate result.
- **Not a V1 recruiting shortcut.** V1 eligibility rule C excludes anyone already shown the product (`V1_VALIDATION/protocol/PARTICIPANT_SCREENING.md`). **A Pilot participant can never become a formal V1 participant.** Before inviting anyone, decide which pool they belong to; once someone has had a Pilot session, the decision is final.
- **Not public distribution.** The V1 distribution limits apply unchanged: named, consenting participants only; no public link or release (`V1_VALIDATION/protocol/CONSENT_PRIVACY.md`).

## Files

| File | Purpose |
|---|---|
| [PILOT_PROTOCOL.md](PILOT_PROTOCOL.md) | Eligibility, consent, build and entry conditions, tasks PT1–PT4 and PT3b, debrief, coding, privacy |
| [RECRUITMENT.md](RECRUITMENT.md) | Who to invite, the screening steps, and the invitation text |
| [MODERATOR_RUN_SHEET.md](MODERATOR_RUN_SHEET.md) | The session day, step by step, and what to record |
| [PILOT_REGISTER.md](PILOT_REGISTER.md) | Counters and the session list; every number starts at 0 |
| [sessions/TEMPLATE.md](sessions/TEMPLATE.md) | One record per session that actually happened, `PL-01`… |

Naming: tasks are `PT1–PT4` and sessions `PL-xx`. `P0–P4` are MVP phases, `M1–M6` are V1 metrics, and `D*` are delivery batches.
