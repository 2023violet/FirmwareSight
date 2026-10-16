---
title: "D2C — M2 Closeout and Pilot Preparation"
doc_id: "FS-DEL-014"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE"
owner: "Product / Delivery"
last_updated: "2026-10-16"
---

# D2C — M2 Closeout and Pilot Preparation

Batch D2C of [12_POST_M2_ROUTE.md](12_POST_M2_ROUTE.md) §3, authorized by the Owner on 2026-10-16 ("规划并出具下一部分执行计划方案并进行实施"). Base: `f4b3a12`. Branch: `feature/d2c-closeout`.

D2C delivers no new product capability. It makes M2 trustworthy in a person's hands, and prepares the first contact with real engineers. Some of its items an agent can complete; others need the Owner or a human operator. This plan separates the two so that nothing waiting on a person is reported as done.

## 1. Items and who completes them

| # | Item | Who | Done when |
|---|---|---|---|
| C1 | `fwsight diff` text escapes every user-file value (file names, section and symbol keys, warning text) | Agent | Code + tests; a control character in a MAP or ELF name never reaches the terminal raw |
| C2 | Automated native QA of the M2 screens on the QA-identifier build | Agent | Evidence summary, AC matrix and hashed ZIP under `evidence/D2C/` (Standing Owner rule 4). Rated as automated evidence, never as the human pass |
| C3 | Runbook for the human native-QA pass | Agent writes; human runs | `evidence/D2C/HUMAN_NATIVE_QA_RUNBOOK.md` lists every case still open from M1 (AC24, AC26, AC20, native drag) and from M2, with the evidence format |
| C4 | Pilot protocol and session materials | Agent writes; Owner approves; moderator runs | `PILOT_VALIDATION/` with protocol, tasks, session template and register |
| C5 | V1 re-freeze request (route decision R3) | Agent drafts; Owner authorizes | A request in `V1_VALIDATION/00_authority/` naming the candidate build rule, what moves and what does not. **Not executed in D2C** |
| C6 | ArmClang / ArmLink sample request (prepares D4) | Agent writes; Owner sources | A request listing the files, provenance and checks a sample needs |
| C7 | Decision R4 (S2.1 backfill of pre-0007 builds) | **Owner** | Recorded in `.ai/DECISIONS.md` |
| C8 | Human native-QA pass (runs C3) | **Human operator, separate Windows account** | Results in the deferred register; only observed cases change state |
| C9 | Recruit Pilot participants and run sessions (runs C4) | **Owner / moderator** | Session records under `PILOT_VALIDATION/sessions/` |

The two remaining small items from the route (§3 item 6) stay out of D2C. The stale `diff.schema.json` description needs an `ADR-0022` decision, and the two M2 S3 items are recorded in `06_DELIVERY/11` §8.3.

## 2. Acceptance for the agent items

**C1**
- `diff_render::render_human` escapes C0/C1 control characters (and DEL) in the base and target file names, in every contributor key (sections, symbols, objects, libraries), and in warning messages. Printable text, including quotes and non-ASCII names, is unchanged.
- The object lists' own escaping (from M2) is folded into the shared path, so nothing is escaped twice.
- Unit tests cover an ESC sequence in a file name, a section key and a symbol key, and show that a printable name with quotes or CJK text is untouched.

**C2**
- It runs on a release build with the QA identifier (`com.firmwaresight.desktop.qa`) and its own data directory. The Owner's app-data files are hashed before and after and must be identical.
- Cases cover:
  - the Objects and Libraries tabs on a MAP-backed build: rows, share, remainder, sorting, filter, paging;
  - the "not available" states for a build without a MAP (UNKNOWN badge and next step);
  - Compare "By object and library" on the `m2-objects` pair: counts, growth and additions, the Objects/Libraries switch, a shortlist drill-down, Added rows showing "Not present";
  - Overview "Latest comparison" after a Compare, and Open Compare from it;
  - `fwsight analyze --objects` and `fwsight diff` text on the same pair.
- Each case records expected, actual, verdict and UTC time. A case that failed and was re-run keeps its first result.
- Each case records what the automation could not show: for example a screen reader, a human's reading of the screen, or a drag.

**C3–C6**
- Each document names its own limits.
- None of them claims a result, a participant or a pass.
- C4 and C5 follow the V1 consent, privacy and distribution rules (`V1_VALIDATION/protocol/CONSENT_PRIVACY.md`) instead of restating different ones.

## 3. Rules that hold for the whole batch

- Pilot data is **exploratory**: it never counts toward V1's 8 participants or any V1 metric, and it is never reported as a gate result.
- Pilot task IDs are `PT1…PT4` and sessions are `PL-01…`. `P0–P4` are MVP phases, `M1–M6` are V1 metrics, and `D*` are delivery batches.
- No push without Owner approval: one ordinary fast-forward.
- The Owner's protected checkout and nine staged files stay untouched.
- Worktrees are removed after use (Standing Owner rule 5).

## 4. Status

| # | State | Commit / evidence |
|---|---|---|
| C1 | Done | `02b88e1` |
| C2 | Done with named gaps: 11 cases, all PASS; A09 failed once on a driver selector and was re-run, both results kept. Paging and in-pass `fwsight diff` text were not run (see the evidence README) | `evidence/D2C/README.md`, ZIP `5b431aed…064a` |
| C3 | Written; **NOT_RUN** (C8) | `evidence/D2C/HUMAN_NATIVE_QA_RUNBOOK.md` |
| C4 | Written; **awaiting Owner approval**; register at 0 | `PILOT_VALIDATION/` |
| C5 | Written; **not authorized, not executed** | `V1_VALIDATION/00_authority/V1_REFREEZE_2_REQUEST.md` |
| C6 | Written; **awaiting samples** | `06_DELIVERY/14_D4_ARMCLANG_SAMPLE_REQUEST.md` |
| C7–C9 | Waiting on the Owner or a human operator | — |

Independent review of `4cbdf74`: PASS_WITH_FINDINGS (0 S1, 4 S2, 9 S3). All four S2s and all nine S3s are fixed in `dc58c21`. The fixes:
- C1 also escapes bidi, zero-width and backslash characters.
- Pilot naming, entry condition and wording.
- The full re-freeze file list.
- Runbook cases H13 and H14.
- The evidence README counts.
- NOW's date and gate line.

The re-review of `dc58c21` found three new S3s, all fixed in the commit after it:
- C1 now escapes everything `char::escape_debug` treats as non-printable, which adds U+061C, U+00AD and U+180E. That is the same table `analyze --objects` uses.
- The re-freeze request now places its `BASELINE.yaml` changes inside the existing `v1_execution` mapping and names `cohort_build_frozen`, `cohort_build_frozen_previous` and `refrozen_guard`.
- This note is corrected.

Two M2 items stay recorded, not changed, in `06_DELIVERY/11` §8.3 (same-named archives merging in the Libraries view; desktop counts that include ambiguous rows). Neither came from this review.
