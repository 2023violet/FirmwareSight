---
title: "Post-MVP Presentation Rules"
doc_id: "FS-DESIGN-009"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Design"
last_updated: "2026-09-27"
---

# Post-MVP Presentation Rules

These are **presentation governance rules**, not feature authorization.

## 1. Chart discipline

> Charts are magnifying glasses for tables, not substitutes for tables.

A visual must answer a concrete engineering question.

Priority:
`table > ranked bars > compact stacked bar > conditional treemap`

### Time-series
If History trends are later implemented:
- zero baseline where the quantity requires it;
- stable series ordering;
- no dual Y axis;
- no hidden rescaling that exaggerates a delta.

### Delta
Positive/negative deltas are not intrinsically good/bad.
Do not encode “increase/decrease” as green/red verdict colors.

## 2. Change presentation

1. show change facts first;
2. Gate owns the verdict;
3. evidence class travels with the value;
4. unchanged items collapse by default.

## 3. Capability banner

“One evidence source, one slot.”

The banner answers **what evidence is available**, not what the product can brag about.

Examples:
- ELF ✓
- MAP ○ not provided
- Git ○ not linked
- future `.su` ○ not provided

## 4. Evidence text form

Machine/CI/plain-text compact form may use:
- `[OBS]`
- `[DRV]`
- `[DEC]`
- `[UNK]`

Human reports must show full terms/legend when ambiguity is possible.

`Inferred` in old exploration prose maps to the canonical `Derived`; it is not a fifth evidence class.

## 5. External port admission

Before designing an external port:
1. What evidence does it print/consume?
2. At what explicit threshold may it interrupt the user?
3. Which in-app reading/record does it return the user to?

No port creates a fifth product verb.

## 6. Notification discipline

- default silent;
- notifications are evidence-triggered, not engagement-triggered;
- every notification returns to a concrete History/Compare/Gate record;
- no streaks, badges or “come back” mechanics.

## 7. Watch contract, if later implemented

Directional contract only:
- default Off;
- Armed interval visible and user-controlled;
- 60 s may be a safe default/minimum candidate but must be revalidated during implementation;
- trigger creates the same History record shape as manual analysis;
- one system notification;
- default factory subscription only for BLOCK-level event.

## 8. Baseline pin, if later implemented

Pin is a measurement concept (“fixed comparison reference”), not a social bookmark.
A pinned baseline may become Compare A by default.

## 9. Dependency/evidence screen

UI-07 is accepted:
- Observed / Declared / Unknown remain explicit;
- manual declaration is audit data;
- Unknown must not be converted to 0 / clean / Pass;
- declaring data cannot erase observed data.
