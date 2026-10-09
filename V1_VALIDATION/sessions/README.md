---
title: "V1 Session Register"
doc_id: "FS-V1-020"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-08"
---

# V1 Session Register

One file per real session, named `V1-P001.md`, `V1-P002.md`, … with follow-ups `V1-P001-F1.md`. Each file
follows `TEMPLATE.md` and is written the same day from notes or a transcript that exists.

```text
sessions written        = 0
eligible unique N       = 0   (minimum 8, target 12-15)
batch A progress        = 0 / 4 eligible external sessions
```

**No session file may be created for a session that did not happen.** §5 forbids simulating a participant,
using an LLM as one, counting an internal team member, inventing a quote, inventing a task completion, or
marking a session complete. A directory that stays empty until real people arrive is the correct state, not a
gap to fill.

## Intake path when a real session arrives (§49)

Applied in order, and the order matters because each step can invalidate the next:

1. validate consent (including the recording permission actually given);
2. validate Participant ID and §7 eligibility, including the internal-author and pre-briefed exclusions;
3. validate the product build against the frozen artifact — head `41bb6a36`, artifact id `11573661113`
   (refrozen 2026-10-08; the first frozen build, head `08fdfcb` / artifact `11419727517`, is superseded), and the
   installer SHA the participant actually ran;
4. extract the timeline;
5. extract the first path for T1/T3/T4/T6/T7;
6. identify interventions, directional or neutral;
7. calculate TTFV **only if the timestamps support it** — otherwise `TTFV = NOT_MEASURED`;
8. code task completion (`completed unassisted` / `after intervention` / `not completed`);
9. code misunderstandings M01–M17 (M18+ only with a documented reason);
10. code critical watches C1–C8;
11. extract verbatim quotes, and mark anything not verbatim as `MODERATOR_NOTE — NOT VERBATIM QUOTE`;
12. separate moderator interpretation from observation;
13. calculate this session's M1–M6 contribution;
14. redact secrets and PII;
15. write the anonymized session file;
16. update `analysis/METRICS.md` and `V1_PARTICIPANT_REGISTER.md`.

**Never infer missing timing.** A session that cannot support a measurement contributes a `NOT_MEASURED` row,
and that row is what gets reported.

## Before staging any of it (§50)

Scan the file for real name, email, company or customer name, private repo, absolute private path, Git remote,
device serial, firmware bytes, MAP content, confidential release information and recording locations. Redact, or
leave the material in `04_sessions_raw` outside Git.
