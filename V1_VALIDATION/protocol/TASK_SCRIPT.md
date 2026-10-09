---
title: "V1 Task Script"
doc_id: "FS-V1-013"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-06"
---

# V1 Task Script

Neutral wording, exactly as §23 specifies. Each task lists what may be said and what must not. A hint added to
the wording is a moderator intervention and is logged as one (§24) — it does not become part of the script.

| Task | Say this | Never say |
|---|---|---|
| **T0** Install / launch | "Please install and open this evaluation build." | anything that routes around a security warning |
| **T1** Analyze | "Use FirmwareSight to understand this firmware build." | "click Analyze" · "choose ELF" · "add MAP" |
| **T2** Evidence improvement (only if a MAP exists) | "Please see whether you can get more detailed or more trustworthy evidence from the files you already have." | "Add .map file" |
| **T3** Compare (only if two builds exist) | "Please work out what materially changed between these two builds." | "open Compare" |
| **T4** Release readiness | "Assume this is a build your team is considering for release. Use the product to work out what it is telling you about release readiness." | "run Gate" |
| **T5** Gate semantics | the comprehension rubric in `INTERVIEW_SCRIPT.md`, **after** task behaviour | any explanation before asking |
| **T6** Bundle | "If you needed to hand the release evidence to someone else, what would you do from here?" | "Build Bundle" · "export" |
| **T7** History | "Imagine you return later and need to explain why a prior build was accepted. Where would you look?" | "use History" |
| **T8** Support / Diagnostics (only if relevant) | "If the tool behaved unexpectedly and you needed support, what information would you look for?" | "export diagnostics" |

**Never add hints to the task wording.** (V0's `TASK_SCRIPT.md` carried the same rule for its ten tasks; the
rule is unchanged, the tasks are not.)

## Per-task recording

For T1, T3, T4, T6 and T7 record: first click, first navigation path, wrong turns, backtracks, stalls, whether
Help was opened, and interventions (§25). For every task record outcome as
`completed unassisted` / `completed after intervention` / `not completed`, and the timestamps that support M2
if and only if they were actually captured.

### T0 also records

- install friction, minute by minute where it mattered;
- the SmartScreen / reputation warning, quoted or described, and how the participant reacted;
- launch success or a typed refusal;
- the SHA-256 of the installer they used, and whether it is the frozen cohort bytes
  (`9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93`, refrozen 2026-10-08; the first frozen build
  `182506f213383cfe00865f199fcec4fb17079535e8ea370f954fc15097263d12` is superseded and is not the build to hand out).

Install friction is **evidence about distribution**, not something to eliminate before the measurement starts.

## Task-to-metric map

| Metric | Tasks that feed it |
|---|---|
| M1 Import activation | T1 |
| M2 Time to First Value | T1, with the first-value END condition met |
| M3 Analyze→Compare | T3 (two valid own builds) |
| M4 Gate comprehension | T5, after T4 |
| M5 Real new information | T1, T2, T3, T4 plus the explicit M5 question |
| M6 Return intent | the fixed M6 question |
| First-path evidence | T1, T3, T4, T6, T7 |

T2 and T8 are conditional on what the participant actually has (a MAP, an unexpected behaviour). Running them
when the condition is absent, or skipping them when it is present, is a protocol deviation and is written down.
