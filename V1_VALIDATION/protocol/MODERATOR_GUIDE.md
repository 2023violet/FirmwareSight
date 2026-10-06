---
title: "V1 Moderator Guide"
doc_id: "FS-V1-012"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-06"
---

# V1 Moderator Guide

The moderator's job is to stay out of the way in a measurable way. Everything that makes M1–M6 mean anything
is in this file or in `TASK_SCRIPT.md`; the human operator runs the session, and the agent only processes what
the session actually produced (§48).

## Opening script

> Thank you for doing this. What you are looking at is a local desktop tool for firmware release evidence. I am
> not here to teach it to you and this is not a test of you — I want to see what happens when someone who did
> not build it tries to use it on a build they do own.
>
> Please think aloud the whole time. If you get stuck, stay stuck for a moment and tell me what you are
> looking for; that is the most useful thing you can give me.
>
> You can stop at any time. Nothing you own gets uploaded. Before we start, two questions about consent and
> about the fact that this build is unsigned.

Do **not** explain before or during the tasks: Observed / Derived / Declared / Unknown · the five Gate states ·
how Compare navigation works · what a Bundle is for · what a MAP file adds · where History lives. Pre-teaching
any of these contaminates exactly what M4 and M5 exist to measure.

## Session order (§23)

1. consent and privacy, including the unsigned-build disclosure (§11, §2);
2. background questions, recorded anonymized (§7);
3. T0 install / launch, with install friction observed rather than assisted;
4. T1–T8 tasks, neutral wording only, in order;
5. the Gate comprehension interview (M4), **after** task behaviour;
6. new-information and return-intent questions (M5, M6);
7. qualitative value and pilot questions (§22) — no price anchors unless the Architect supplies them;
8. debrief, and only now any explanation the participant asks for.

## Discipline during tasks (§24)

Forbidden before a task is complete:

```text
"Click Analyze."          "Go to Compare."           "You need to add MAP."
"UNKNOWN means missing evidence."                    "PASS does not mean compliance."
"Use History."            "This is correct."         "Great, you understood it."
```

If the participant stalls: **wait.** Then, and only then, use a neutral prompt:

- "What would you try next?"
- "If this were software at work, where would you look?"

Anything more directional is a moderator intervention: set `MODERATOR_INTERVENTION = YES`, record the exact
hesitation that preceded it, and treat the resulting completion as taught. **A taught completion does not
count as unassisted success** — it never enters an M1 or M3 numerator, and if it happened before first value
the M2 time is kept and flagged `INTERVENTION_BEFORE_FIRST_VALUE = YES`.

Do not rescue early. Do not praise a correct answer. Do not call a product behaviour "obvious". Do not tell the
participant what FirmwareSight is "supposed" to mean.

## What to capture (§25, §28, §30)

For T1, T3, T4, T6 and T7: first click, first navigation path, wrong turns, backtracks, stalls, whether Help
was opened, and every intervention. No first-click threshold is invented — the path is described. Quotes are
verbatim or labelled `MODERATOR_NOTE — NOT VERBATIM QUOTE`, and interpretation is written separately from
observation. Timestamps are written down as they happen; where they are missing, M2 is `NOT_MEASURED` and
nothing is inferred (§17, §49).

## Critical watch (§27)

Any single occurrence enters Interim Review. Report it as `1 / N observed` and never generalize one observation
to "users".

| Code | What you would be hearing or seeing | Where it is coded |
|---|---|---|
| C1 | PASS treated as legal or regulatory certification | M13 Critical |
| C2 | UNKNOWN treated as "no problem" | M07 Critical |
| C3 | an accepted REVIEW treated as objective proof | M08 Critical |
| C4 | missing or unsupported evidence treated as exact fact | M06 / M16 Critical |
| C5 | they would ship on the strength of a misunderstood state | M05 Critical |
| C6 | a supported-artifact result that is materially wrong | S1 product finding |
| C7 | the product damaged or modified their artifact or project | S0 product finding |
| C8 | a crash or data corruption in the formal workflow | S0 product finding |

## State preparation before each session

V0 reset a prototype to a known state. V1's equivalent, because the store is real and persists:

1. confirm the participant is installing artifact `11419727517` from run `37475580080`, and record the
   installer SHA of the bytes they actually received;
2. start from a fresh local store unless the session is deliberately about History — if a prior session left
   data, say which state the participant is seeing;
3. do **not** use the owner's real FirmwareSight store as a demo surface; the owner's data is out of scope for
   research, and this round's rule of parking and restoring it still applies;
4. have the participant's own ELF (and MAP, if any) on their machine, never copied into the repository;
5. confirm nothing about the product has been explained yet.

## After the session

Write the session record the same day, from the notes or transcript that exist (§49). Anonymize, run §50's
privacy scan, then update `V1_PARTICIPANT_REGISTER.md` and `analysis/METRICS.md`. If the session revealed a
product problem, classify it under §31 — and if it is S0 or S1, **stop formal sessions**, preserve the evidence
and return to the Architect (§32, §36).

## Early stop triggers (§36)

Any S0 or S1 · a credible C1–C8 product risk · the same S2 independently observed by ≥ 3 participants · the
same misunderstanding independently observed by ≥ 3 participants · import activation < 60 % after the first 4
valid sessions · Gate comprehension < 50 % after the first 4 · any privacy or consent breach · an artifact
provenance mismatch · a participant who received the wrong build. Do not wait for N = 8 when the protocol or
the product is invalid.
