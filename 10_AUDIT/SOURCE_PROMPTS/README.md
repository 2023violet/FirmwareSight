---
title: "Execution Prompt Register"
doc_id: "FS-AUDIT-SRC-003"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Audit"
last_updated: "2026-09-28"
---

# Execution Prompt Register

Every authorized execution prompt is preserved here verbatim **where a verbatim copy exists**. Each
SHA-256 below was recomputed from the archived copy in this directory, so the archived bytes match what
was delivered rather than a retranscription. Three P0 rounds were supplied to the executing agent as
text with no source file; each says so in its own entry instead of standing for a hash it cannot have.

## V0 Workflow Prototype Validation

- File: `FirmwareSight_V0_Workflow_Prototype_Validation_PROMPT_v1.1.txt`
- SHA-256: `3431a688c65c08aa6104fe4cb785ef25b6ae9df4e2d3272dddda653704e1df20`
- Authority: execution instruction subordinate to the unique v0.5.0 product baseline, and authoritative for the authorized V0 task.

## V0 Batch A External Validation

- File: `FirmwareSight_V0_Batch_A_External_Validation_PROMPT_v1.0.txt`
- SHA-256: `e62c236b090de9da589bcf570415edd7a0296925910d65437b2105a9612589e7`
- Authority: execution instruction for the Batch A external validation, subordinate to the v0.5.1 baseline. It withheld `v0.5.2` until real Batch A evidence exists, and left P0 unauthorized in that execution.

## P0 Technical Vertical Slice

- File: `FirmwareSight_P0_Technical_Vertical_Slice_EXECUTION_PROMPT_v1.1_ARCHITECT_REVIEWED.txt`
- SHA-256: `60a59196708a53592be4d828a0c1275cf74b3bfad0bdc6455f863ab38d32929b`
- Size: 94802 bytes / 5064 lines
- Authority: execution instruction for the P0 engineering slice against the unique v0.5.1 baseline. Consistent with `09_ADR/ADR-0020-validation-sequence.md`, which already authorizes V0 and P0 as parallel tracks. It overrides only the execution state recorded by the Batch A prompt (`V0` blocking, `P0` not authorized); it does not override any frozen product, architecture, evidence, gate or design baseline, and it does not authorize P1.
- Provenance record: `P0_TECHNICAL_VALIDATION/P0_EXECUTION_PROVENANCE.md`
- Outcome, 2026-09-28: executed to its stop condition as `EXECUTED — CONDITIONAL_PASS (LOCAL)`.
  Baseline stayed `v0.5.1`, `v0.6.0` withheld, G1 not claimed, P1 still unauthorized. The conditions
  were that CI and `cargo deny` had never run, which needed a push this environment was not
  authorized to make.
- Outcome, superseded the same day by measurement: the delivered tree was pushed as `f9b8ccb`, GitHub
  Actions run `36360310447` concluded `failure` with 2 of 6 jobs green, and the status became
  `FAIL — REMOTE CI RUN #1`. `CONDITIONAL_PASS (LOCAL)` is no longer the current label; the
  `LOCAL PASS` / `CI PASS` distinction this pack kept is what made the failure legible rather than
  surprising. Evidence: `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`.
- Final disposition, 2026-09-28: **`P0 = PASS`**, promoted into `FirmwareSight_Project_Baseline_v0.6.0`
  after remote CI Runs #3 and #4 each came back 7/7 green. Nothing in this prompt's scope was renegotiated
  to get there; the two rounds registered below changed how the tree is built and checked, not what the
  slice was asked to produce.

## P0 CI Closure / Cross-Platform Reproducibility Remediation

- File: none. The prompt was supplied inline to the executing agent, so unlike the records above
  there is no stored copy whose SHA-256 can be recorded here. The repository's copy of its terms is
  `.ai/DECISIONS.md` ("P0 CI closure remediation") and
  `P0_TECHNICAL_VALIDATION/P0_CI_REMEDIATION_REPORT.md`.
- Version: v1.0, architect reviewed.
- Authority: local remediation of what Run #1 measured, one `cargo-deny@0.20.2` installation, and one
  real desktop window launch in the shipping configuration. It does not authorize a push, does not
  permit `REMOTE CI PASS` to be written by the executing side, stops before P0 promotion, and does not
  change `ADR-0020`.
- Outcome, superseded by measurement: the owner pushed `ebda52d`, run `36378384225` executed and
  concluded `failure` with **six of seven jobs green**. The first round's five fixes are therefore
  confirmed remotely; the residue is a second job needing the same Linux prerequisites, which is what
  *Remote CI Run #2 Final Drift Closure v1.0*, registered below, closes.
- Outcome, final: the round-2 HEAD `1cd6309` was pushed and Run `36399805005` concluded `success`
  with **7 of 7 jobs green**, including `Generated output drift`. Remote CI is therefore closed by
  measurement; P0 promotion is not, because `P0 = PASS`, `v0.6.0` and a regenerated `SHA256SUMS` are
  the architect's signed act. The track read `CONDITIONAL_PASS` with that condition named until the
  promotion entry at the end of this file.
- Outcome, 2026-09-28: `LOCAL REMEDIATION COMPLETE / READY FOR REMOTE CI RERUN`. All five causes
  reproduced with a command before being changed, a sixth defect found by the authorized window launch
  and fixed with tests first, local gate 14/14 with zero skipped mandatory steps, desktop smoke `PASS`,
  and one architecture conflict (two advisories unreachable inside the frozen Tauri 2.12.0 dependency
  tree) reported rather than resolved. Baseline stays `v0.5.1`, `v0.6.0` is not generated, G1 is not
  claimed, P1 is not started, and the push that closes the round belongs to the owner.
- Final disposition: the round's five fixes plus the macOS job were confirmed remotely by Runs #2, #3 and
  #4, and the storage defect its authorized window launch found became migration `0002` (schema version 2).
  Superseded as a status by *P0 Remote CI Run #2 Final Drift Closure* below.

## P0 Remote CI Run #2 Final Drift Closure

- File: none — **original file unavailable, prompt supplied inline**. No SHA-256 is recorded because no
  archived bytes exist; claiming a "byte-exact copy" of text that arrived in a conversation would be a
  fabricated provenance record. The repository's copy of its terms is `.ai/DECISIONS.md` ("Remote CI Run #2
  and its final drift closure") and
  `P0_TECHNICAL_VALIDATION/P0_CI_RUN_2_CLOSURE_REPORT.md`.
- Version: v1.0, architect reviewed.
- Authority: a workflow-scoped fix for the one job Run #2 left red, plus the governance record of that run.
  Not a push, not `REMOTE CI PASS` from the executing side, not P0 promotion, not `v0.6.0`, not P1, and not
  an architecture redesign. It named the fixes it forbade: deleting the desktop test command, dropping or
  disabling the IPC bindings step, `continue-on-error`, `if: false`, moving the job to Windows, turning the
  failure into a warning, deleting the macOS job or the dependency policy, editing a fixture hash, or
  relabeling P0 as `PASS`.
- Outcome: Run `36399805005` on HEAD `1cd6309` concluded `success`, **7 of 7 jobs green**, with the drift
  job installing the Linux prerequisites and reporting `5/5 steps passed`. The owner pushed that HEAD;
  Run #4 `36402637251` then reproduced the result on the architect-reviewed documentation HEAD.

## P0 Final Promotion / v0.6.0 Baseline Closure

- File: none — **original file unavailable, prompt supplied inline**, so there is no source file to hash.
  The repository's copy of its terms is `.ai/DECISIONS.md` ("P0 Final Promotion Decision — 2026-09-28")
  and `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`.
- Version: v1.0, **architect signed**.
- Authority: exactly one act — promote P0 to `PASS` and freeze `FirmwareSight_Project_Baseline_v0.6.0` as
  the *P0 Technical Foundation Baseline*. It states in its own §1 and §2 that this does not mean V0 passed,
  G1 passed, the MVP is complete, P1 is authorized, Compare/Gate/Bundle exist, a release or installer is
  ready, user value is validated, or the tree is free of vulnerabilities; and it keeps `G1 = V0 PASS + P0
  PASS` unclaimed with `ADR-0020` untouched.
- Scope guards: no production source may move during promotion (`crates/**`, `apps/**`, `fixtures/**`,
  `golden/**`, `migrations/**`, both lockfiles, `deny.toml`, `scripts/check.py`, the workflow, the design
  tokens) — a needed source change would have stopped the round as
  `PROMOTION BLOCKED BY NEW ENGINEERING DEFECT`. No GitHub Release, tag, installer, signing, notarization
  or updater metadata. `V0_VALIDATION/**` not touched. Stale claims are to be audited phrase by phrase and
  classified historical versus current, not globally string-replaced, and version metadata is bumped only in
  current baseline authority documents.
- Outcome, 2026-09-28: executed. `P0 = PASS`, `baseline_version: 0.6.0`, `active_task: NONE`, V0 still
  `0 / 8`, G1 not claimed, P1 not authorized. `DIRECTORY_TREE.txt` and `SHA256SUMS` regenerated as the
  v0.6.0 record.
