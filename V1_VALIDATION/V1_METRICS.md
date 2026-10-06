---
title: "V1 Metric Contract"
doc_id: "FS-V1-002"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-06"
---

# V1 Metric Contract

Written **before** any participant data exists, and that order is the point: §15 forbids redefining a
threshold after seeing the data. If a threshold turns out to be wrong, the round stops and returns to the
Architect instead of editing this file.

## The six metrics

| ID | Metric | Denominator | Numerator | Threshold |
|---|---|---|---|---|
| M1 | Import activation | eligible supported-cohort sessions with a valid own ELF | participant reaches a rendered Analyze result on their own ELF **without directional moderator instruction about which product control to use** | **> 80 %** |
| M2 | Time to First Value | every participant whose timestamps support it | see the operational definition below | **median < 60 s** |
| M3 | Analyze→Compare completion | eligible participants with **two valid own-project builds of the same project** | unassisted meaningful Compare: correct baseline/target, Compare rendered, and can state ≥ 1 real delta | **≥ 60 %** |
| M4 | Gate meaning comprehension | eligible participants | rubric A–E all correct | **> 80 %** |
| M5 | Real new information | eligible participants | `NEW_INFORMATION = YES` under the rule below | **≥ 30 %** |
| M6 | Return intent | eligible participants | only `YES` answers | **≥ 62.5 % AND ≥ 5 unique YES participants** |

N ≥ 8 eligible unique external participants is required before any of these may be read as a pass set (§12,
§38). All six are required together; five of six is not a pass.

## M1 — Import activation (§16)

Neutral task wording is allowed; directional help is not. If the moderator says which control to use, the
intervention is flagged and that task does **not** count as unassisted success. Report `activated / eligible = %`.

## M2 — Time to First Value (§17)

Architect's operational definition, quoted in structure rather than paraphrased away:

- **START** — the participant has the installed app open and receives the neutral own-artifact task.
- **END** — both are true: (1) their own Analyze result is rendered, **and** (2) they correctly identify at
  least one concrete non-trivial fact. Acceptable kinds: Flash/image footprint, RAM footprint, a major
  contributor, an evidence limitation, an `Unknown` reason, or a MAP-driven evidence improvement.
- The timer does **not** stop at UI load alone.

Report every measured participant time, the median, p75 when useful, and the count under 60 s. If a
directional intervention happened before first value, keep the time and flag
`INTERVENTION_BEFORE_FIRST_VALUE = YES`. If the timestamps do not support the measurement,
`TTFV = NOT_MEASURED`. **Never guess a duration.**

## M3 — Analyze→Compare (§18)

Only participants with two valid own-project builds of the same project are in the denominator, and the
exclusion is printed with its reason. No demo fixture may substitute for a real pair.

## M4 — Gate meaning comprehension (§19)

Asked **after** task behaviour, never before, with neutral questions. A participant must correctly
distinguish PASS / REVIEW / BLOCK / UNKNOWN / N/A, and comprehension passes only with all five:

- **A** — PASS is a policy/evidence result, **not** universal release approval;
- **B** — UNKNOWN is insufficient or unresolved evidence, **not** "clean";
- **C** — REVIEW is accountable human judgment, **not** automatic proof;
- **D** — BLOCK prevents qualification;
- **E** — N/A means the rule does not apply in this context.

Report `comprehension-pass / eligible` plus the per-concept misses, so a 85 % that hides "nobody understood
UNKNOWN" is not possible. Any belief that PASS certifies legal, compliance, security or safety matters is
**Critical** (§19, §27 C1).

## M5 — Real new information (§20)

`NEW_INFORMATION = YES` only when the participant explicitly identifies a concrete fact about **their own**
build that they did not know, had not noticed, or would otherwise have investigated separately — unexpected
size growth, a surprising contributor, missing evidence, a custom section's impact, RAM growth, MAP resolving
an ambiguity, a release-policy issue, or a historical comparison fact. Generic praise does not count. Report
`new-info / eligible` plus anonymized examples.

## M6 — Return intent (§21)

The question is fixed, in these words:

> "Thinking about a real release/review like the one you work on, would you intentionally open FirmwareSight
> again for a future release?"

Code the answer `YES` / `MAYBE` / `NO`. Only `YES` enters the numerator. The target is a **pair** of
conditions — `>= 62.5 %` **and** at least 5 unique `YES` participants — because a percentage alone can be
manufactured by shrinking a denominator. The prompt's own worked examples: 5/8 = 62.5 % with 5 yes → meets;
5/10 = 50 % → fails; 6/9 = 66.7 % with 6 yes → meets. Historical "5/8" is not a universal metric; this
repository's rule is the percentage **and** the headcount.

## Reporting discipline (§37)

Every number is reported as numerator, denominator, percentage and exclusion reason. Never a bare percentage,
never a denominator laundered by silently dropping people:

```text
Import   7/8  = 87.5%
Compare  5/7  = 71.4%   (1 eligible participant had no valid second build and is excluded with that reason)
Return   6/9  = 66.7%   with 6 unique YES
```

Commercial signal is **secondary** (§22): purchase process, approval owner, pilot willingness and budget
ownership may be recorded, but no V1 pass depends on pricing, no pricing model or paid plan is created, and no
price anchor is invented — with no Architect-provided anchors, only qualitative value and pilot questions are
asked.

## Critical watch (§27)

Any single occurrence enters Interim Review, reported as `1 / N observed` and never generalized to "users":

| Code | Observation |
|---|---|
| C1 | PASS interpreted as legal/regulatory certification |
| C2 | UNKNOWN interpreted as "no problem" |
| C3 | an accepted REVIEW interpreted as objective proof |
| C4 | missing or unsupported evidence treated as exact fact |
| C5 | the participant would ship based on a misunderstood state |
| C6 | a materially incorrect supported-artifact result |
| C7 | the product unexpectedly damages or modifies the participant's artifact or project |
| C8 | a crash or data corruption in the formal workflow |

C1–C5 continue V0's watch list; C6–C8 are V1's addition, because V1 runs against a real installed product on a
real own artifact where a wrong number or a damaged project is possible in a way a prototype could not manage.

## Product finding classification (§31)

Classify each finding as `PRODUCT`, `RESEARCH_HARNESS`, `ENVIRONMENT`, `UNSUPPORTED_COHORT`,
`KNOWN_LIMITATION`, `PREFERENCE`, `FEATURE_REQUEST` or `OBSERVATION`. Product severity:
`S0` catastrophic data/security/integrity · `S1` core workflow blocker or materially wrong supported evidence
· `S2` major product/usability defect · `S3` minor defect · `S4` cosmetic or observation. Preferences are not
bugs. Correctly-unsupported Keil/IAR input is not a parser defect.

## Verdict states (§39, §54)

| Condition | State |
|---|---|
| N < 8 | `V1_INCOMPLETE_INSUFFICIENT_SAMPLE` |
| N ≥ 8 but a metric misses or a major risk remains | `V1_EVIDENCE_PARTIAL` |
| N ≥ 8, all six thresholds met, no unresolved S0/S1, no invalidating Critical misunderstanding, protocol and privacy intact | `V1_READY_FOR_ARCHITECT_VERDICT`, then STOP |

`V1_PASS_COMPLETE`, `B1_READY` and `PRIVATE_BETA` may never be self-issued. Metrics alone do not pass a stage:
the Architect also reviews critical misunderstandings, S0/S1, cohort concentration, protocol integrity and
privacy integrity (§38).

## Current values

```text
eligible unique external participants  = 0
M1..M6                                 = NOT_MEASURED (no denominator exists yet)
critical watches C1..C8                = 0 observed / 0
product findings S0..S4               = none recorded
V1 recommendation                      = V1_INCOMPLETE_INSUFFICIENT_SAMPLE
```

These are zeros because no session has happened, which is a fact about recruitment, not a result about the
product.
