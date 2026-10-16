---
title: "Post-M2 Delivery Route"
doc_id: "FS-DEL-013"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Product / Delivery"
last_updated: "2026-10-15"
---

# Post-M2 Delivery Route

The route the Owner confirmed on 2026-10-15 ("确认执行") after reviewing a proposed M3–M6 plan. This document is a sequence with entry conditions, not an authorization. Each batch starts only when `.ai/ACTIVE_TASK.md` names it. The E1/E2/E3/GX candidate rules of `ADR-0024` and the G3/G4/B1 gates of `06_STAGE_GATES.md` still apply in full.

## 1. Decisions recorded with this route

| # | Decision | Status |
|---|---|---|
| R1 | Delivery batches after M2 are named **D2C, D3, D4, D5, D6**, not M3–M6. `V1_VALIDATION/V1_METRICS.md` already defines metrics M1–M6, and V1's M3 (Analyze→Compare completion) would collide with a delivery batch of the same name. This is the same reason `ADR-0024` gave the post-MVP candidates their own namespace. The existing M1 and M2 batch names stay as they are; they are history. | Decided |
| R2 | `ADR-0033` (object and library attribution) moves from PROPOSED to **BASELINE**. Its own text said it would on merge approval after independent review: three review rounds, then the push of `04d800c` (Actions #117, 10/10). | Decided |
| R3 | Formal V1 takes **option (b)**: re-freeze the cohort build to an M2-era build, so the study measures the product being shipped. The re-freeze itself is a separate Owner-authorized round like `V1_COHORT_REFREEZE` of 2026-10-08: it moves one pointer and no metric, denominator or threshold. **Not executed here.** | Direction decided; execution needs its own authorization |
| R4 | M2 review S2.1: should builds stored before migration 0007 gain object rows on re-import? The options are a backfill (an additive INSERT; rows stay immutable) or the current behaviour. | **Open — Owner decision** |

## 2. Sequence

| Batch | Content | Entry condition | Exit evidence |
|---|---|---|---|
| **D2C** M2 closeout and Pilot preparation | §3 | Named in ACTIVE_TASK | Native QA register updated; R4 decided; Pilot protocol and V1 re-freeze authorization ready |
| **D3** History workflow | Pin one stored build as a long-term baseline (shown as Compare's default base); start a Compare from a History row | Pilot shows repeated build-to-build comparison, **or** a written Owner exception to `ADR-0024`'s "V1/user signal" condition, recorded in `.ai/DECISIONS.md`; Scope Creep and dependency review | A user compares the current build with the pinned one without hunting for files; tests; native QA |
| **D4** ArmClang / ArmLink | MAP adapter for armlink, own ADR and fixtures; support stated per layer (ELF read, memory accounting, object attribution, compare) | Same-origin `.axf`, `.map` and scatter file / linker configuration from a real project, plus an independent cross-check (Standing Owner rule 3) | Per-layer support matrix in `04_TECH/39` backed by fixtures |
| **D5** Evidence depth | `.su` worst-stack evidence (**ADR first**, `E2-A`), then the SBOM completeness model (`E2-B`), then a release symbol archive (`E2-C`) | Pilot or V1 feedback naming the need | Per item: ADR, spec, acceptance tests |
| **D6** Release readiness and G4 | License (L13), signing and updater ADR, install and upgrade, feedback channel; then G4 Private Beta (20–50 users, 3–5 teams, `06_STAGE_GATES.md`) | Native QA risks closed; Owner decisions on license and signing | G4 gate evidence |

D4 may move ahead of D3 if real ArmClang samples arrive first and a user is waiting on them. Getting samples takes longest, so the search for them starts in D2C, not in D4.

## 3. D2C — what "start now" means

1. **Native QA (human operator, separate Windows account).** The M1 deferred items AC24, AC26 and AC20 (`06_DELIVERY/11` §6), plus the M2 surfaces: Objects/Libraries tabs, Compare "By object and library" and Overview "Latest comparison". Results go into the deferred register; nothing is reported PASS without observation (Standing Owner rule 2).
2. **Owner decision R4** (S2.1).
3. **Pilot protocol**: an exploratory study, separate from V1, with its own record. Tasks: import an own ELF+MAP, find the largest growth source, compare two builds, export a report. Consent and privacy wording follow the V1 materials. Pilot sessions never count toward V1's 8 participants.
4. **V1 re-freeze authorization** (R3), prepared for the Owner as its own round.
5. **ArmClang sample search** for D4.
6. **Small known defects**, in scope only if the Owner names them:
   - `fwsight diff` text does not terminal-escape section and symbol names (pre-existing).
   - The `diff.schema.json` description of `objectChanges` is stale; editing it is an `ADR-0022` decision.
   - Two M2 S3 items in `06_DELIVERY/11` §8.3.

## 4. Kept out of this route

- Large UI reworks, treemaps, watch mode, XLSX and DWARF ABI detail (`E3`, conditional).
- IAR, until real samples and a user pull exist (`E3`).
- CSV/Markdown export of object rows. It is a new interchange format for per-object data, which `ADR-0033` D7 sends to an `ADR-0022` decision. It is a D3 follow-up candidate, not part of D3.
- A new acceptance framework for each small feature: batches keep the existing plan, acceptance and review pattern used in M1 and M2.
- Any claim that V1 or a gate has passed on the strength of Pilot data.
