---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-09-29"
---

# ACTIVE TASK

```text
P3_RELEASE_GATE — the third MVP stage of the open-source MVP-first line ADR-0026 opened.

Authorization: "FirmwareSight — P3 Release Gate MVP Implementation, Execution Prompt v1.1 — Architect
Reviewed", supplied inline on 2026-09-29. It is registered in 10_AUDIT/SOURCE_PROMPTS/README.md without
a SHA-256, because no source file was delivered to this repository — recording a hash for bytes nobody
received would be a fabricated provenance record. ADR-0027 is the architecture half of the same
authorization: it exists to keep project policy and Git provenance out of Core.

Status while this file is live: IN_PROGRESS. Nothing in this document claims P3 is complete.
```

## What the stage is

One product verb, `Gate`, over a persisted or in-process build plus workspace facts outside the artifact:

```text
firmwaresight.toml → project / Git evidence → GatePolicy → Core Gate
→ PASS / REVIEW / BLOCK / UNKNOWN / N/A → immutable GateRun
→ immutable Review acceptance → CLI gate → Desktop Release
```

`firmwaresight-core` owns every Gate semantic — state, effective severity, aggregate precedence, rule
identity. `firmwaresight-project` (ADR-0027) owns the only two things P3 needs that Core must never do:
reading `firmwaresight.toml` and running read-only system Git. Storage persists the run; the shell
projects it; React paints it. No rule is re-evaluated in SQL, in a Tauri command handler or in the
frontend (`AGENTS.md` 3, prompt §6).

## Acceptance list

The frozen **US-003 Prepare Release** criteria in
`01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:33-40`, all four items, plus the PRD **P0-5 Release Gate**
ten built-in checks at `01_PRODUCT/01_PRD_MVP.md:66-77`. P3 is not `PASS` while any applicable item is
open.

| Source | What it binds |
| --- | --- |
| `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` US-003 | every finding shows rule, state, evidence, remediation; Review visually distinct from Block; Unknown never auto-Pass; policy savable to project config |
| `01_PRODUCT/01_PRD_MVP.md` P0-5 | the ten built-in checks: clean tree, commit matches release metadata, tag/version consistency, required artifacts present, hashes generated, FLASH budget, RAM budget, growth over baseline past threshold, required Release Notes present, Unknown evidence reaching the human-review condition |
| `09_ADR/ADR-0023` + `04_TECH/27_GATE_STATE_SEMANTICS.md` | five factual states with `effective_severity` as a separate field; UNKNOWN mapped by `on_unknown`, never rewritten; accepted Review is an immutable audit record that leaves the finding at REVIEW |
| `06_DELIVERY/08_MILESTONE_DELIVERABLE_MATRIX.md:22` | deliverables are 5-state rules, unknown policy and review audit; exit evidence is a rule matrix plus contract tests |
| `04_TECH/22_GIT_PROVENANCE_ADAPTER.md` | system `git` CLI, read-only, direct spawn, bounded output, timeout, sanitized logs; no `gix` |

One documented conflict is already settled and is not reopened here: PRD P0-5 lists four Gate results
(PASS / REVIEW / BLOCK / NOT_APPLICABLE), `04_TECH/27` §"Problem resolved" records that as the v0.3.0
draft, and ADR-0023 froze five states over it. This round implements five, and the PRD's four-state
sentence is not treated as authoritative.

## Boundaries this round works inside

```text
Baseline            v0.6.0 — unchanged; no v0.7.0 without its own promotion prompt
Crate               firmwaresight-project, the fifth first-party library crate, authorized by ADR-0027.
                    No Tauri, no SQLite, no Tokio, no Gate rule semantics, no artifact parsing, no UI,
                    no network inside it
Core                stays headless, sync, filesystem-free, Git-process-free and dependency-free;
                    scripts/check.py CORE_PACKAGES gains it, because a silent omission would drop the
                    crate from core-smoke coverage without failing anything
Schema              SCHEMA_VERSION 2 -> 3 through additive 0003_gate_history.sql only: gate_runs,
                    gate_findings, gate_finding_evidence, accepted_reviews. No destructive change, no
                    Bundle tables, no absolute paths
Dependency          toml + regex as direct deps, both already in Cargo.lock, so no new package enters
                    the graph; sha2, thiserror and serde reused at locked versions. cargo-deny executes
                    for real
Design token        none new. status.pass / review / review.strong / block / unknown exist at v0.2.1;
                    there is no status.na token, and N/A is rendered from the existing neutral
                    vocabulary rather than an invented one
IPC                 five use-case commands: open_project_config, save_project_policy, run_release_gate,
                    accept_review, get_gate_run. No run_sql, read_project_file, write_project_file,
                    execute_git, get_any_path
Config save         validate before write, same-directory temp then replace, never a zero-byte config,
                    and unknown keys are never silently dropped: a saver that cannot preserve them
                    refuses with a typed error
Evidence          Git is workspace provenance, never artifact build proof. Allowed wording is
                    "Workspace HEAD matches the declared release commit"; forbidden wording is
                    "Firmware was built from this commit"
Privacy             no host path in IPC, UI, database, portable output or run fingerprint — the project
                    root is read by the adapter and never leaves it
Identity            run_id = gate-<SHA-256 of the canonical Gate input>: snapshot ids, policy hash,
                    normalized Git facts, Release Notes path/presence/digest. Never wall clock, pid or
                    absolute root
CLI               fwsight gate only. Exit codes 0 PASS / 4 REVIEW / 5 BLOCK / 2 usage-config /
                    3 import-parse / 6 export; 1 is never produced. `release` stays unregistered.
                    No hidden project database for CLI Gate
```

`AGENTS.md` 2 / 3 / 5 / 6 / 7 / 8 / 11 apply unchanged, and `ADR-0026` relaxed no technical boundary.

## Start facts, measured before the first write

```text
HEAD = origin/main      32b23aa78323d315f6643f04c2343f75576d483f
worktree                clean at §0 (git status --short empty; git diff and git diff --cached empty)
Run #19                 36665007523 on 32b23aa — completed, success, 7 of 7 jobs
                        read with gh run view 36665007523 --repo 2023violet/FirmwareSight
Run #18                 36648718199 on 4a77ea1 — completed, success, 7 of 7 jobs: the P2
                        implementation tree the architect sealed as FINAL PASS
Rust tests at start     345 passed / 0 failed / 0 ignored   (cargo test --workspace)
UI tests at start       99 passed in 5 files                (corepack pnpm test)
cargo-deny              0.20.2 installed, so deny/cargo-deny executes rather than taking the SKIPPED path
Fixture toolchain       arm-none-eabi-gcc 14.3.Rel1 on PATH; the P2 fixture pair is the Gate smoke
                        subject, because §61 forbids using this source repository as one
```

P1 and P2 carry five known findings into this round — desktop smoke step 27's same-pair lock, the
un-re-smoked final binary after the `layoutSource` fix, object/module attribution Unavailable, the
nullable-reason columns `sections.file_offset` and `symbols.address`, and peak RSS / fuzz / macOS+Linux
window measurement. All five are NON-BLOCKING for P3: none of them is reopened for its own sake, and if
this round makes one reproducible it is fixed as a normal bug. Migration `0003` is the first
schema change since `0002`, so the nullable-reason gap now has a natural place to land **only if** this
round can prove it; it is not authorized as a P3 requirement.

## Stop condition

```text
STOP AFTER P3.

P4 Release Bundle is the next authorizable stage and is NOT authorized by this file: it needs its own
architect prompt. No release prepare command, no bundle directory, no bundle chooser, no
release-manifest generation, no bundle SHA256SUMS, no bundle release-report, no analysis/diff copies,
no History page, no Project Wizard, no installer, signing, updater, SBOM, CVE, OTA, flashing, HIL,
cloud, account, telemetry, AI judge, pricing or commercial work.

One gap deliberately does not close with the stage: FirmwareSight is an open-source project, the
workspace still declares license = "Proprietary" at Cargo.toml:17, and the repository root has no
LICENSE file. AGENTS.md 9 puts a license change in front of a human, and this prompt does not choose
one. It is recorded as OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION, it does not block P3
engineering, and it stays visible in the completion report.
```

Five existing assertions guard what this stage ships, and each is changed by name rather than quietly
deleted: `apps/cli/src/main.rs` `unregistered_future_commands_are_still_not_accepted` (drops `gate`,
keeps `release` / `watch` / `doctor`), the same file's `exit_code_one_is_never_produced` (its
`assert_eq!(all, [0,2,3,6])` gains `4` and `5`, which is exactly what this stage makes reachable),
`apps/desktop/ui/src/intake.test.tsx`'s third-nav-entry and "no Gate text" guards (prompt §43 replaces
them with the Analyze + Compare + Release assertion), and the four storage tests pinning
`SCHEMA_VERSION` 2 at `storage.rs:83/121/512`, `compare_candidates.rs:612`,
`map_companion_persistence.rs:298` and `real_artifact_intake.rs:745` (moved to 3, with the v1→v3 and
v2→v3 upgrade paths tested rather than the constant edited to make a test pass).
