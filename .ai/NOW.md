---
title: "Current Project Status"
doc_id: "FS-AI-NOW"
product: "FirmwareSight"
version: "0.6.0"
status: "CURRENT_INDEX"
owner: "Engineering"
last_updated: "2026-10-17"
---

# FirmwareSight Now

This file is overwritten, never appended. History lives in [archive/](archive/2026-10-14/INDEX.md). Read Git and Actions for the live HEAD before making any claim. The SHAs below were last verified on 2026-10-15.

## Code and CI

- **Last integrated code:** `76b5c2c` (NEXT-46 export privacy and stale-result fixes, integrated by NEXT-47). [Actions #113](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036): 10/10 Jobs and 10/10 Check Runs.
- **Later documentation-only commits:** `71978d8` (NEXT-47 evidence upload, [Actions #114](https://github.com/2023violet/FirmwareSight/actions/runs/38602220781) 10/10). The 2026-10-14 lean batch follows; read Git for its SHA and CI.
- **M1 first-use round:** on `main` at `6804ae7` (Actions #116, 10/10). Copy cut, drag-and-drop intake, Overview memory bars, Release step order, table scrolling, native dialog language, and six native-QA fixes. Evidence: [evidence/M1](../evidence/M1/README.md).
- **M2 object and library attribution:** on `main` at `04d800c` ([Actions #117](https://github.com/2023violet/FirmwareSight/actions/runs/38826237431), 10/10). GNU ld MAP object rows are stored per build (migration 0007) and shown on Analyze, Compare, Overview and CLI text; M1 leftovers F2, N1 and N2 are closed. `ADR-0033` is BASELINE. Three independent review rounds; dispositions in [06_DELIVERY/11](../06_DELIVERY/11_M2_OBJECT_ATTRIBUTION.md) §8. Local gate at `04d800c`: `check.py` 18/18, 0 SKIP.
- **Actions #118 on `81e1345` (docs only):** `Desktop UI (ubuntu-latest)` failed on attempt 1 and passed on attempt 2. The failure is L23's eighth instance, repaired test-only with a mutation proof; a green rerun alone would not have closed it. See [P5_KNOWN_LIMITATIONS](../P5_VALIDATION/P5_KNOWN_LIMITATIONS.md), last section.
- **Route after M2:** batches D2C, D3–D6 with entry conditions in [06_DELIVERY/12](../06_DELIVERY/12_POST_M2_ROUTE.md).
- **D2C (on `main` at `e4d7f8b`, Actions #120):**
  - `fwsight diff` text now escapes control characters from user files.
  - Automated native QA of the M2 screens: 11 cases, all PASS ([evidence/D2C](../evidence/D2C/README.md)).
  - Written but waiting on a person: the Pilot protocol (Owner approval), the V1 re-freeze request (not authorized), the human native-QA runbook (not run), and the ArmClang sample request (no samples yet).
- **R-series redesign (ADR-0034):** complete. R0–R3 reached `main` at `465fe5f` (Actions #125); R4–R7 at `e1faeff` (run `39086793226`, 10/10). The closeout adds the skip link and heading focus (Q6) and runs the window-minimum drag, a keyboard-only flow, an Explorer drop, 125%/150% content scaling and an accessibility-tree audit with real Windows input ([evidence/R7C](../evidence/R7C/README.md)). Left to a person: NVDA/Narrator listening, real system DPI.
- **D2C-R (local branch `docs/d2c-r`):** R4 decided (no silent backfill); human QA pack for the R7 build (NOT_RUN); Pilot materials for the R7 build (approved in principle, register 0); a Core section-ranking defect found and documented, not yet fixed ([06_DELIVERY/24](../06_DELIVERY/24_CORE_SECTION_RANKING_REVIEW.md)); a synthetic armlink sample cross-checked, armlink still `NOT_SUPPORTED`. See [06_DELIVERY/23](../06_DELIVERY/23_D2C_R.md).
- **Latest `main`:** `e1faeff` (run `39086793226`, 10/10), then the closeout commit. Read Git for anything later.

## Gates

| Gate | State |
|---|---|
| Code integration (NEXT-47) | PASS |
| Native desktop user acceptance | **PASS_WITH_LIMITS** (M1, automated driver; independent review); gaps listed in [evidence/M1](../evidence/M1/README.md); native drag-and-drop NOT_RUN |
| Public release, signing, updater, open-source license | NOT_AUTHORIZED |

## Open risks (must close before the first external Developer Preview, Private Beta or RC, whichever comes first)

- NEXT-44 AC24, NEXT-44 AC26, NEXT-45 AC20: rated PASS_WITH_LIMITS by the independent review of M1 and still open. Not yet observed: deny, confirm, failure and retry on a visual export; the English replace confirmation; recorded Tab order; a screen reader; 1440×900; the non-Owner profile and human operator the [deferred register](../NEXT47_VALIDATION/evidence/04_NEXT47_DEFERRED_NATIVE_QA.md) asks for. See [evidence/M1](../evidence/M1/README.md).
- Native drag-and-drop from Explorer: PASS with a real mouse drag from Explorer (R7C DND-D01, 2026-10-17).
- Screen-reader check: listening with NVDA/Narrator not run; the accessibility tree UI Automation reads was audited on four workspaces (R7C SCALE-S03).
- M2's desktop surfaces (Objects/Libraries tabs, Compare "By object and library", Overview "Latest comparison"): automated native QA only (D2C C2). The human pass in [the runbook](../evidence/D2C/HUMAN_NATIVE_QA_RUNBOOK.md) has not run.

## Compatibility boundary

GCC + GNU ld and Clang + GNU ld are verified cohorts. Keil (armlink) and IAR (ILINK) compiler output is **UNVERIFIED**. ArmLink and IAR MAP files are **NOT_SUPPORTED / FIXTURE_BLOCKED**. See [04_TECH/39](../04_TECH/39_ELF_TOOLCHAIN_INTEROPERABILITY.md).

## Protected local state

The Owner's original `main` checkout stays at `e34f9e6` with nine staged documentation files. It is intentionally behind the remote. Do not reset, commit or push from it without an explicit Owner decision.

## Active task

[ACTIVE_TASK.md](ACTIVE_TASK.md).
