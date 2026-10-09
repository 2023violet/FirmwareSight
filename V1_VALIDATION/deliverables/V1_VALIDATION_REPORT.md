---
title: "V1 Validation Report"
doc_id: "FS-V1-050"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-10-08"
---

# V1 Validation Report

**Skeleton, written at activation with no results in it.** §53 fixes the 24 sections this report must have;
writing them now means the structure is decided before the data exists, which is the same discipline that fixes
the thresholds in `V1_METRICS.md` before the first session. Every metric line reads `NOT_MEASURED` because
`N = 0`.

## 1. Authority and artifact
V1 prompt v1.0 (delivered SHA-256 `48768ff0…ec10a0f`), canonical unit `V1_OWN_ARTIFACT_EXTERNAL_VALIDATION`,
product `MVP_CANDIDATE` at baseline `0.6.0`. **Cohort build refrozen 2026-10-08** (`V1_COHORT_REFREEZE_RECORD.md`,
owner's inline Execution Authorization v1.0) to run `37828673549`, head `41bb6a36`, artifact
`11573661113`; installer `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, 3,896,257 bytes, SHA-256
`9a51e86a…87d93`; internal `SHA256SUMS.txt` verified entry by entry on re-download. The build frozen at activation —
run `37475580080`, head `08fdfcb`, artifact `11419727517`, 3,888,432 bytes, `182506f2…63d12` — is recorded there as
superseded history and is **not** to be handed out. _To be completed: per-session
confirmation that each participant ran these exact bytes._

## 2. Protocol
Five protocol documents in `protocol/`, continued from V0 rather than restarted; neutral T0–T8 wording; §24
intervention discipline; §48 human/agent division. _To be completed: any deviation actually observed._

## 3. Eligibility
_Screened: 0. Eligible: 0. Excluded, with reasons: 0._

## 4. Cohort composition
_To be filled from `V1_PARTICIPANT_REGISTER.md`: role families, experience bands, team-size and
release-frequency bands, cohort concentration._

## 5. Sample and exclusions
_N = 0 against a minimum of 8 and a target of 12–15. Every exclusion will be listed with its reason and its
quality flag; a follow-up session will never be counted as a new participant._

## 6. Toolchain and artifact distribution
_Supported-cohort ELF/Clang counts, MAP presence, build-pair presence, `DISCOVERY_OUT_OF_COHORT` counts._

## 7. M1 Import activation — `NOT_MEASURED` (0 / 0; threshold > 80 %)

## 8. M2 Time to First Value — `NOT_MEASURED` (no timestamps exist; median threshold < 60 s; nothing guessed)

## 9. M3 Analyze→Compare — `NOT_MEASURED` (0 two-build-eligible / 0; threshold ≥ 60 %)

## 10. M4 Gate comprehension — `NOT_MEASURED` (0 / 0; threshold > 80 %; rubric A–E reported per concept)

## 11. M5 Real new information — `NOT_MEASURED` (0 / 0; threshold ≥ 30 %; concrete own-build facts only)

## 12. M6 Return intent — `NOT_MEASURED` (0 YES / 0; threshold ≥ 62.5 % **and** ≥ 5 unique YES participants)

## 13. Moderator interventions
_0. Count with `MODERATOR_INTERVENTION = YES`, count before first value, and what each intervention was._

## 14. Misunderstandings
_0 entries. M01–M17 frequencies with severity kept separate from frequency; any M18+ with its justification._

## 15. Critical watches
_C1–C8, each reported as `1 / N observed`, never generalized to "users"._

## 16. Product findings
_0 entries. §31 classification, S0–S4 severity, and what the freeze rule did with each._

## 17. Unsupported cohort
_Keil/IAR and out-of-boundary inputs as `DISCOVERY_OUT_OF_COHORT` demand evidence, explicitly not as defects._

## 18. Quotes
_None. Verbatim only, each attributed to a participant id, with `MODERATOR_NOTE — NOT VERBATIM QUOTE` marking
anything paraphrased, and interpretation always separated from quotation._

## 19. Value signal
_None._

## 20. Commercial / pilot signal, if collected
_Qualitative only. No pricing model, no paid plans, no invented anchors (§22). Nothing collected yet._

## 21. Privacy and protocol integrity
_To be completed: §50 scan results per committed session, consent records held outside Git, and any protocol
contamination and how it was handled._

## 22. Study limitations
_Already statable: single-platform cohort (Windows only, because that is the only platform with real install
evidence — macOS and Ubuntu are `CI_BUILD_ONLY`); one frozen unsigned build; moderated sessions rather than
unobserved use; participants recruited through the owner's network, which is a concentration risk; and N = 0
until recruitment happens._

## 23. Metric verdict
`V1_INCOMPLETE_INSUFFICIENT_SAMPLE` — N = 0 against a minimum of 8. No forced PASS/FAIL conclusion (§12).

## 24. Recommendation
`V1_INCOMPLETE_INSUFFICIENT_SAMPLE`. `V1_READY_FOR_ARCHITECT_VERDICT` requires all six metrics met at N ≥ 8 with
no unresolved S0/S1, no invalidating Critical misunderstanding, and protocol/privacy intact (§38, §39). This
report may never write `V1_PASS_COMPLETE`, `B1_READY` or `PRIVATE_BETA` (§54).
