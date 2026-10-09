---
title: "V1 Aggregate Metrics"
doc_id: "FS-V1-030"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-06"
---

# V1 Aggregate Metrics

Updated only from session files that exist. At the moment none exist, so every row below is a zero denominator
and `NOT_MEASURED`. This is a recruitment state, not a product result, and it must never be reported as though
the metrics had been tested and missed.

| Metric | Numerator | Denominator | Percentage | Threshold | Status |
|---|---|---|---|---|---|
| Sample size N | 0 eligible unique participants | — | — | ≥ 8 (target 12–15) | **insufficient** |
| M1 Import activation | 0 | 0 | — | > 80 % | `NOT_MEASURED` |
| M2 Time to First Value | 0 measured | 0 | — | median < 60 s | `NOT_MEASURED` |
| M3 Analyze→Compare | 0 | 0 two-build-eligible | — | ≥ 60 % | `NOT_MEASURED` |
| M4 Gate comprehension | 0 | 0 | — | > 80 % | `NOT_MEASURED` |
| M5 Real new information | 0 | 0 | — | ≥ 30 % | `NOT_MEASURED` |
| M6 Return intent | 0 YES | 0 | — | ≥ 62.5 % **and** ≥ 5 unique YES | `NOT_MEASURED` |

## Exclusions

| Reason | Count |
|---|---|
| `EXCLUDED_INTERNAL` | 0 |
| `EXCLUDED_NO_CONSENT` | 0 |
| `EXCLUDED_NO_REAL_ARTIFACT` | 0 |
| `EXCLUDED_PROTOCOL_CONTAMINATED` | 0 |
| `DISCOVERY_OUT_OF_COHORT` | 0 |
| excluded from M3 only (no valid build pair) | 0 |

Every exclusion carries its reason when it happens; a silent absence from a denominator is denominator
laundering (§37).

## Interventions and timestamps

```text
sessions with MODERATOR_INTERVENTION = YES        : 0
sessions with INTERVENTION_BEFORE_FIRST_VALUE     : 0
sessions where TTFV timestamps were insufficient  : 0 (each one reports TTFV = NOT_MEASURED, never a guess)
```

## Critical watches

| Code | Observed | Reported as |
|---|---|---|
| C1 PASS as legal/regulatory certification | 0 | `0 / 0` |
| C2 UNKNOWN as "no problem" | 0 | `0 / 0` |
| C3 accepted REVIEW as objective proof | 0 | `0 / 0` |
| C4 missing evidence treated as exact fact | 0 | `0 / 0` |
| C5 would ship on a misunderstood state | 0 | `0 / 0` |
| C6 materially incorrect supported-artifact result | 0 | `0 / 0` |
| C7 product damaged participant artifact/project | 0 | `0 / 0` |
| C8 crash or data corruption in formal workflow | 0 | `0 / 0` |

## Toolchain and cohort distribution

| Category | FORMAL_SUPPORTED_COHORT | DISCOVERY_OUT_OF_COHORT |
|---|---|---|
| GCC / arm-none-eabi-gcc | 0 | 0 |
| Clang (documented measured Arm cohort) | 0 | 0 |
| Keil | 0 | 0 |
| IAR | 0 | 0 |
| other | 0 | 0 |

Cohort concentration is one of the things the Architect reviews beyond the numbers (§38): six participants from
one team on one toolchain is not six independent samples, and the table is what makes that visible.

## Reading rules

1. No percentage is printed without its numerator and denominator (§37).
2. No threshold is adjusted after seeing data (§15). If a threshold looks wrong, that goes to the Architect as a
   question, not into this file as an edit.
3. Five of six metrics met is **not** a pass set (§38).
4. A clean pass set is `V1_READY_FOR_ARCHITECT_VERDICT`, never `V1_PASS_COMPLETE` (§39, §54).
5. The product counts this round must not move: 868 Rust / 225 UI in 8 files, gate 17, drift 8, package 4. If one
   of them changes, something other than documentation happened, and the round stops (§45). Dated 2026-10-08: the
   cohort build was re-frozen that day onto `41bb6a36`'s artifact `11573661113`, and the live guard figure is now
   **868 Rust across 47 result lines / 291 UI in 9 files** on the same 17-step gate — the UI line grew across U1,
   which ran after this pack was written. That is a build-identity guard, not a metric: none of the M1–M6 operations,
   denominators or thresholds in this file or in `V1_METRICS.md` moved, and rule 2 still forbids adjusting one after
   seeing data.
