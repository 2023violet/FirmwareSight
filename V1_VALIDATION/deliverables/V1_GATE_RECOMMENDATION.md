---
title: "V1 Gate Recommendation"
doc_id: "FS-V1-051"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-10-08"
---

# V1 Gate Recommendation

## Current recommendation

```text
V1 = IN_PROGRESS / RECRUITMENT_READY
recommendation = V1_INCOMPLETE_INSUFFICIENT_SAMPLE
eligible unique external participants = 0   (minimum 8, target 12-15)
eligible external sessions            = 0
M1..M6                                = NOT_MEASURED
WAITING FOR REAL EXTERNAL PARTICIPANTS.
```

This is a recommendation about a **sample**, not about the product. Nothing was measured, so nothing about
FirmwareSight's usability was learned — and an empty result is reported as empty rather than dressed up as a
finding. V0 reached the same shape on honest terms and stayed at `0 / 8`; the difference now is that the
instrument is the real installed product and the protocol has fixed thresholds written down in advance.

## Why the recommendation cannot be anything else

§54 allows exactly three values, and two of them require data that does not exist:

| Value | Condition | Today |
|---|---|---|
| `V1_INCOMPLETE_INSUFFICIENT_SAMPLE` | N < 8 | **this one** — N = 0 |
| `V1_EVIDENCE_PARTIAL` | N ≥ 8 but a metric misses or a major risk remains | not reachable |
| `V1_READY_FOR_ARCHITECT_VERDICT` | N ≥ 8, all six thresholds met, no unresolved S0/S1, no invalidating Critical misunderstanding, protocol and privacy intact | not reachable |

Never self-issuable, in any state: `V1_PASS_COMPLETE`, `B1_READY`, `PRIVATE_BETA`.

## What a green V1 would still not mean

Metrics alone do not pass a stage. The Architect also weighs critical misunderstandings, S0/S1, cohort
concentration, protocol integrity and privacy integrity (§38). And §56 stands regardless of result: V1 proves
nothing about market size, PMF, production safety, security certification, regulatory compliance, enterprise
readiness, all-platform runtime support, optimal pricing, long-term retention, team deployment, support
scalability, or Beta readiness by itself.

**No automatic B1 (§40).** Even a full pass does not open Private Beta: G4 / B1 suggests 20–50 users and 3–5
teams, and needs its own beta support loop and release-train decision by a separate Architect prompt.

## What V1 is responsible for, on the record

P5's `P5_VALIDATION/P5_KNOWN_LIMITATIONS.md` carries **L11 — user comprehension untested (V0 0/8)** as
`CARRIED_FORWARD, owner V1`. V1 owns the evidence that could resolve or reduce it (§55). Today that row stays
`CARRIED_FORWARD`: with zero sessions there is nothing for the Architect to review. The most this track may ever
write before a verdict is `READY_FOR_ARCHITECT_REVIEW`, and only once real evidence exists.

L11 is not edited by this round in either direction. The limitation lives in `P5_VALIDATION/`, which V1 §43 does
not allow this round to write to; its disposition changes when the Architect says it does.

## Release-readiness state while V1 runs

Unchanged from P5 Commit F3, and V1 changes none of it:

```text
Signing           = READY_NOT_EXECUTED
Notarization      = READY_NOT_EXECUTED
Update            = MANUAL_UPGRADE_READY
Open-source license = PENDING_OWNER_CONFIRMATION
Public distribution = NOT_AUTHORIZED
Tag               = NOT_CREATED
GitHub Release    = NOT_CREATED
```

The research build given to participants is **unsigned** and moves one-to-one to named, consenting
participants only (§2). That is a controlled research transfer, not a distribution, not a beta and not a
release, and it must not be described as any of those three.

## What happens next, and who does it

Recruitment is a human job (§47, §48): the operator reaches real firmware and embedded engineers, screens them
against §7, obtains consent, transfers the exact frozen build, and moderates. The agent maintains the protocol,
validates eligibility, converts **real** notes and transcripts into anonymized records, calculates metrics,
classifies findings, and refuses to fabricate what is missing.

Nothing further happens in the repository until a real session exists. The next commits this track can make are
the Batch A evidence commit after four eligible participants, then the Batch B evidence commit at eight or more
(§51) — each one docs/evidence only, each one holding the cohort build's product counts and passing the same 10-job
authoritative CI. Those counts are **868 Rust across 47 result lines / 291 UI in 9 files** at the build re-frozen on
2026-10-08 (artifact `11573661113`, head `41bb6a36`); this file was written when the cohort was F3's build and the
figure was 225 UI in 8 files, and that is left as the record of what was true then.
