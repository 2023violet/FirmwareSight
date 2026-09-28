---
title: "P0 Technical Validation Index"
doc_id: "FS-P0-001"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Technical Validation

The evidence pack for the P0 Technical Vertical Slice: the first real source tree, and whether
it holds the boundaries the v0.5.x baseline froze.

Nothing in this directory is a substitute for a test. Every claim carries the command that
produced it, and a status of `LOCAL PASS` is kept visibly separate from `CI PASS` and
`NOT RUN`, because a check that was not run has not passed.

## Reading order

| Document | Question it answers |
| --- | --- |
| `P0_EXECUTION_PROVENANCE.md` | Which prompt, which starting commit, which machine. |
| `P0_PLAN.md` | What was going to be built, in what order, with what exit items. |
| `P0_IMPLEMENTATION_LOG.md` | Each engineering decision, its authority and its evidence. |
| `P0_ARCHITECTURE_CHECK.md` | Did the crate and boundary rules survive contact with code. |
| `P0_FIXTURE_REGISTER.md` | Which real binaries are being trusted, and their hashes. |
| `P0_PARSER_RESULTS.md` | What the ELF and MAP readers actually returned. |
| `P0_MEMORY_ACCOUNTING_REPORT.md` | Is ADR-0021's two-budget model demonstrable. |
| `P0_CLI_PARITY_REPORT.md` | Does the CLI report Core facts and nothing else. |
| `P0_IPC_PARITY_REPORT.md` | Do CLI and Desktop agree field by field. |
| `P0_STORAGE_REPORT.md` | Do migrations, transactions and round-trips behave. |
| `P0_PERFORMANCE_REPORT.md` | Measured cost, and the 512 MiB guard's behaviour at the boundary. |
| `P0_SECURITY_INPUT_REPORT.md` | Untrusted input, capability surface, privacy boundary. |
| `P0_DEPENDENCY_REPORT.md` | What was added, why, and what was refused. |
| `P0_CI_REPORT.md` | What the gate runs, and what the remote runs measured. |
| `P0_CI_REMEDIATION_REPORT.md` | Each failed CI job, its reproducer, its fix, and what remains unproven. |
| `P0_DESKTOP_SMOKE_REPORT.md` | The shipped window opened for real, field by field. |
| `P0_CI_RUN_2_CLOSURE_REPORT.md` | What the second remote run measured, and the one provisioning step still failing there. |
| `P0_FINAL_PROMOTION_REPORT.md` | The status decision's endpoint: P0 promoted to `PASS` and the v0.6.0 baseline frozen. |
| `P0_DESIGN_CHECKLIST.md` | Did the UI pass the checklist AGENTS.md 11 requires, box by box. |
| `P0_KNOWN_LIMITATIONS.md` | What P0 does not do, said plainly. |
| `P0_EXIT_CHECKLIST.md` | Every exit item, with its evidence line. |
| `P0_TECHNICAL_VALIDATION_REPORT.md` | The status decision and its reasoning. |

## Verified totals

- Rust: 104 tests passing across 6 workspace members.
- TypeScript: 19 tests across 3 files, plus strict typecheck, ESLint and a production Vite build.
- Gate: `python scripts/check.py` - 14 steps on a tree that already has `apps/desktop/ui/dist/`, 16
  when the group has to build the frontend first, plus 3 under `--only core-smoke` for the macOS job
  the CI baseline requires. cargo-deny now runs here (`0.20.2`, exit 0) rather than being `SKIPPED`.
- Remote CI: four runs. `36360310447` was 2 of 6 green; `36378384225` was 6 of 7; `36399805005` on
  `1cd6309` and `36402637251` on `5e58f77` each concluded **`success`, 7 of 7 green** - 104 Rust tests on
  Windows and Ubuntu, 19 UI tests on both, 87 core tests on macOS, all four cargo-deny categories ok.
- Status: **`P0 = PASS`**, promoted by the architect-signed *P0 Final Promotion / v0.6.0 Baseline Closure
  v1.0* prompt on 2026-09-28. See `P0_FINAL_PROMOTION_REPORT.md`.
- Cold rebuild: `cargo clean` plus removing `node_modules/` and `dist/`, then the whole gate -> 16/16
  steps, 102 Rust tests and 18 UI tests, exit 0; the delivered tree then re-ran 14/14 with 19
  UI tests after an accessibility fix that touched no Rust file. That run belongs to the pre-remediation
  tree and its Rust count has since moved to 104.
- Baseline under validation when this pack was written: `FirmwareSight_Project_Baseline_v0.5.1`. The
  baseline it produced is `FirmwareSight_Project_Baseline_v0.6.0`, and each document here keeps its own
  `0.5.1` frontmatter as the provenance of when it was written.
- This pack: the 17 documents the prompt's `P0_TECHNICAL_VALIDATION/` list names, plus
  `P0_EXECUTION_PROVENANCE.md` from the takeover, `P0_DESIGN_CHECKLIST.md` required by
  `AGENTS.md` 11, and the four reports the CI track adds - 23 files.

## Final disposition — P0 PASS, v0.6.0 (2026-09-28)

P0 closed `PASS` and the
`FirmwareSight_Project_Baseline_v0.6.0` record was frozen by the architect-signed promotion prompt.
Every status line in this directory is a dated record of the round that wrote it: `CONDITIONAL_PASS`,
`FAIL — REMOTE CI RUN #1` and `FAIL — REMOTE CI RUN #2` were each true when written, and none of them was
edited into something it was not. `P0_FINAL_PROMOTION_REPORT.md` is the current endpoint, and
`P0_TECHNICAL_VALIDATION_REPORT.md` carries the corrected headline.

What the pack still does not claim: V0 validation (`0 / 8` external sessions), a G1 pass, P1
authorization, a measured peak RSS, or a fuzz campaign.
