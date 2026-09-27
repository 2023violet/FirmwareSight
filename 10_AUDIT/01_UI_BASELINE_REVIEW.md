---
title: "UI Baseline Review"
doc_id: "FS-AUDIT-002"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Audit"
last_updated: "2026-09-27"
---

# UI Baseline Review

## Decision

Accept the external team's UI direction with governance corrections.

### Accepted
- IBM/Carbon-inspired engineering temperament
- light surfaces
- thin borders
- single accent
- semantic status
- dense data tables
- no card shadow
- evidence-first panels
- six supplied high-fidelity screens

### Not accepted as authority
- upstream IBM numeric values
- IBM brand identity
- marketing layout patterns
- screenshot demo data as product contract
- adding Carbon React dependency by default
- any screenshot behavior that conflicts with Core/policy semantics

## Frozen FirmwareSight overrides

- radius = project token family, not square Carbon
- accent = project token
- Light-only MVP
- no gradient/glass/neon/purple
- overlay shadows only
- 5-state Gate
- Unknown neutral gray/hollow icon
- table density tokens
- system font MVP
- functional motion only

## Important semantic corrections to screenshots

1. `Can we ship now?`
   = configured release-policy readiness only, not legal/safety/security certification.

2. `Accepted review`
   = immutable disposition record; original REVIEW/evidence persists.

3. `UNKNOWN`
   = evidence deficiency; separate from N/A.

4. version labels
   must distinguish App Version vs Artifact/Release Version.

5. inspector width
   is adaptive token range, not fixed 340px contract.

## Missing seventh PNG

External transcript says an Unknown Dependency PNG exists, but it was not included in current uploaded assets.

v0.4 includes the normative screen spec only; no fabricated image.
