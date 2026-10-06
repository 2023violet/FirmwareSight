---
title: "V1 Misunderstanding Log"
doc_id: "FS-V1-031"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-10-06"
---

# V1 Misunderstanding Log

The taxonomy is inherited from V0 (`V0_VALIDATION/analysis/MISUNDERSTANDING_LOG.md`, defined in the archived V0
Batch A prompt), which is why the codes are the same numbers doing the same jobs. V1 adds no codes at the start;
`M18+` is used only when something is genuinely not representable, and the reason is documented.

| Code | Meaning |
|---|---|
| M01 | Product positioning |
| M02 | Navigation |
| M03 | Analyze |
| M04 | Compare |
| M05 | Gate |
| M06 | Evidence |
| M07 | Unknown |
| M08 | Review |
| M09 | N/A |
| M10 | Bundle |
| M11 | History |
| M12 | Error recovery |
| M13 | Compliance misunderstanding |
| M14 | Pricing / value |
| M15 | Toolchain expectation |
| M16 | Evidence state transition |
| M17 | MAP / capability misunderstanding |

Severity is Critical / High / Medium / Low, and **severity is kept separate from frequency**: one Critical
occurrence triggers Interim Review (§27, §36) while four Low occurrences do not.

## Entries

| ID | Code | Severity | Participant | What they believed | Evidence (verbatim or moderator note) | Related critical watch | Related finding |
|---|---|---|---|---|---|---|---|
| — | | | | | | | |

**0 entries, because 0 sessions have been conducted.** Entries arrive only from real transcripts or notes, and
each one names the participant id it came from so the aggregate can be re-derived.

## Expected pressure points, stated before the data

Writing these down first is not a prediction of results; it is the record of what the protocol will look at, so
that "we found it" cannot later be confused with "we were watching for it".

| Code | Why V1 will probably see it |
|---|---|
| M07 / C2 | `UNKNOWN` looks like a status field, and neutral grey can read as "nothing to worry about" |
| M13 / C1 | a green PASS next to the word "release" invites a certification reading |
| M08 / C3 | an accepted `REVIEW` can look like the tool verified the change |
| M17 | a missing MAP can read as "the tool is wrong" rather than "evidence is thinner" |
| M15 | Keil/IAR users will reasonably expect their toolchain to work; that is `DISCOVERY_OUT_OF_COHORT`, not a defect |
| M11 | History is local SQLite; the assumption that it is a server is exactly what L11 was carried for |
| M16 | an evidence state that changed between two builds can read as a bug rather than as new information |

If none of these recur, that is a result worth reporting — and it is only worth reporting because the list was
written down before anyone looked at data.
