---
title: "ADR-0024 Post-MVP Candidate Governance"
doc_id: "ADR-0024"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture / Product"
last_updated: "2026-09-27"
---

# ADR-0024 — Post-MVP Candidate Governance and Namespace

## Status
Accepted in v0.5.0.

## Context
Three expert exploration rounds produced useful future recommendations but were authored against v0.3.0 and used “P1/P2/P4” labels for future features.

v0.4.0 already defines P1–P4 as canonical Product MVP implementation stages.

Importing the old labels literally would create two incompatible schedules and could accidentally expand the current MVP.

## Decision

1. Current MVP scope and P0–P4 lifecycle remain unchanged.
2. Directional expert decisions are preserved as **candidate decisions**, not implementation commitments.
3. Post-MVP candidates use:
   - E1 low-cost workflow depth
   - E2 evidence depth
   - E3 conditional advanced experience
   - GX growth/ecosystem
4. Any candidate promotion requires:
   - V1/user signal when applicable;
   - Scope Creep review;
   - dependency review;
   - ADR for domain/architecture/network trust-boundary changes;
   - feature spec and acceptance evidence.
5. Source documents keep their historical stage names for audit traceability, but current planning uses only the new namespace.

## Alternatives

### Rewrite P1–P4 to include new ideas
Rejected: silently expands MVP and destroys v0.4 gate semantics.

### Ignore all expert exploration
Rejected: discards useful validated direction.

### Keep two P1/P2 vocabularies
Rejected: creates deterministic agent confusion.

## Consequences

Positive:
- all expert conclusions can be retained without scope expansion;
- future roadmap is readable;
- AI agents cannot treat “candidate” as “task”.

Cost:
- old reports require stage-name translation when referenced.

## Revisit trigger

If product exits V1 and formally opens a Post-MVP release train, E1/E2/E3/GX may be converted into a dated/versioned roadmap by a new decision.
