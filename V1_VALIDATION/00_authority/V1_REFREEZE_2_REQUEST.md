---
title: "Request: second V1 cohort re-freeze (to an M2-era build)"
doc_id: "FS-V1-RF-002-REQ"
product: "FirmwareSight"
version: "0.6.0"
status: "REQUEST_NOT_AUTHORIZED"
owner: "Research"
last_updated: "2026-10-16"
---

# Request — second V1 cohort re-freeze

**A request for the Owner, not an authorization and not an execution.** The cohort build is unchanged: the U1 candidate installer, SHA-256 `9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93`, frozen by `V1_COHORT_REFREEZE_RECORD.md` on 2026-10-08.

## Why

Route decision R3 (`06_DELIVERY/12`) chose option (b). The frozen U1 build predates M1 (first-use copy, drag and drop, memory bars, the Release step order) and M2 (object attribution, Overview latest comparison). A formal V1 on it would measure screens that have already been replaced. That is the reason the 2026-10-08 round gave for moving off F3.

## What the round would do, if authorized

1. **Candidate:** the Windows NSIS installer built by CI from one `main` commit. It is chosen only after:
   - D2C has merged;
   - the human native-QA pass (`evidence/D2C/HUMAN_NATIVE_QA_RUNBOOK.md`) has closed AC24, AC26 and AC20, or the Owner has explicitly accepted the remaining limits for research use.

   Running V1 on a build with open native risks would put participants in front of known defects.
2. **Verify both builds the same way as 2026-10-08:** installer and payload hashes, `toolchain.git_commit`, the old→new delta.
3. **Move one pointer, everywhere it is written.** The 2026-10-08 round reconciled eleven files under `V1_VALIDATION/`, each keeping the old identity beside the new one as superseded (`V1_COHORT_REFREEZE_RECORD.md`, "Protocol documents now name the frozen bytes"):
   - `README.md`, `V1_PLAN.md`, `V1_PARTICIPANT_REGISTER.md`;
   - `protocol/TASK_SCRIPT.md` (T0 hash) and `protocol/MODERATOR_GUIDE.md`. The guide treats any other installer hash as an early-stop trigger, so missing it would stop every session on the new build;
   - `sessions/README.md` and `sessions/TEMPLATE.md` (cohort head, CI run);
   - `analysis/FINDINGS.md`, `analysis/METRICS.md`, `deliverables/V1_GATE_RECOMMENDATION.md`, `deliverables/V1_VALIDATION_REPORT.md`.

   In `BASELINE.yaml`, everything goes **inside the existing `v1_execution` mapping**. A second top-level `v1_execution` key would silently replace the first one, the duplicate-key fault fixed in `fe420d1`. Follow the 2026-10-08 pattern:
   - a new dated entry beside `cohort_build_refreeze`, carrying the authorization and the new installer and payload hashes;
   - the pointer itself, `cohort_build_frozen`, moved to the new bytes, with its current value kept under `cohort_build_frozen_previous` as that round did;
   - `cohort_build_state`;
   - `refrozen_guard`, re-pointed at the new build.

   Every earlier value stays as history. `V1_METRICS.md` is not touched. The round's own record and the authorization transcription are new files, as in 2026-10-08.

## What the round would not change

- The metrics M1–M6, their denominators and their thresholds.
- The task wording.
- Eligibility, including rule C.
- Consent and the distribution limits.

The round also recruits nobody, installs nothing for anyone, measures nothing, and leaves every counter at 0.

Pilot sessions (`PILOT_VALIDATION/`) never feed V1, and anyone who took part in a Pilot is ineligible for V1.

## What the Owner decides

1. Authorize the round: yes or no.
2. Gate the candidate on full native-QA closure, or on an explicitly accepted remaining-limits list.
3. Who moderates V1 sessions once the build is frozen. That is not part of this round.
