---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-09-30"
---

# ACTIVE TASK

```text
P4_RELEASE_BUNDLE — opened 2026-09-30 by *FirmwareSight — P4 Release Bundle MVP Implementation, Execution
Prompt v1.0 — Architect Reviewed*, delivered as a file and archived with its SHA-256
`1baaec9204a1d2aa5aa53bd735b34d376ee56db7557a04d6abb79265c30840c5` in
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P4_Release_Bundle_MVP_Implementation_EXECUTION_PROMPT_v1.0_ARCHITECT_REVIEWED.txt`.

This is the last core product-implementation stage of the open-source MVP line ADR-0026 opened. One verb,
`Bundle`: assemble the already-trusted Analyze + Compare + Gate + accepted-Reviews result, the selected
current firmware artifacts and the Release Notes into a directory that is portable, independently readable
without FirmwareSight, hash-verifiable and host-path-free, and that never overwrites an existing directory
without an explicit confirmation.

AGENTS.md 1 binds exactly as it did for P1, P2 and P3: this file names ONE stage. P4 does not authorize a
History page, Project Wizard, installer, signing, notarization, updater, SBOM, CVE, OTA, flashing, HIL,
cloud, accounts, telemetry, AI, pricing or commercial work, and it does not create `v0.7.0`, P5, V1, B1,
RC1 or GA1. Its own §78 says STOP AFTER P4.

What P4 may not do is declare the MVP finished. The only gate statement it may write is
`G2 = READY_FOR_ENGINEERING_GATE_REVIEW`; the architect performs the whole-MVP G2 closure audit in a
separate round.
```

## Start facts measured before the first write

The prompt's §0/§2 anchor is `ba5e59e79208969a4687db11e69fdef2ababd8ab`, green on Run #25 `36785425648` —
true as written. `origin/main` had moved one commit further by the time this round started, so §2's
"if remote is newer, inspect and reconcile before writing" ran first:

- `HEAD = origin/main = 323afad155348bd5b721fa2b7e2ed2388bdc2691`, worktree clean, single worktree on `main`.
- `git diff --name-only ba5e59e 323afad` is eleven documentation, governance and integrity files plus two
  newly tracked baseline-artifact scripts. **No** production source, fixture, schema, migration or config
  file moved, so the newer HEAD is the same tree for engineering purposes. It is therefore the start fact
  this round records, the same way P3 recorded Run #19 on its own successor HEAD rather than on the sealed
  P2 tree.
- Run `36810689645` on `323afad`: `completed` / `success` / **7 of 7 jobs**, read with
  `gh run view 36810689645 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs`.
- Start counts re-measured, not inherited from the prompt: one `cargo test --workspace` = **556 passed /
  0 failed / 0 ignored**; `corepack pnpm test` = **135 passed in 6 files**. Both match §0's numbers, which
  is the check that the retired `715` double-count was not reintroduced.

## What just closed

`P3_RELEASE_GATE` — the third MVP stage of the open-source MVP-first line ADR-0026 opened, authorized by
*FirmwareSight — P3 Release Gate MVP Implementation, Execution Prompt v1.1 — Architect Reviewed*
(supplied inline, registered without a SHA-256 because no source file reached this repository) and by
`ADR-0027`, which keeps project policy and Git provenance out of Core.

```text
firmwaresight.toml → project / Git evidence → GatePolicy → Core Gate
→ PASS / REVIEW / BLOCK / UNKNOWN / N/A → immutable GateRun
→ immutable Review acceptance → CLI gate → Desktop Release
```

The acceptance list was the frozen **US-003 Prepare Release** criteria in
`01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:33-40`, all four items, plus the PRD **P0-5 Release Gate** ten
built-in checks at `01_PRODUCT/01_PRD_MVP.md:66-77`.

| Document | Holds |
| --- | --- |
| `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md` | §69's forty boxes, each settled by a `gh` query, a command, a named test or a smoke step; what the checklist does not cover; the verdict |
| `P3_VALIDATION/P3_GATE_EXECUTION_REPORT.md` | what was built, every §66 command and its measured output, the §62 CLI smoke, the three defects, the carried-forward limits, the license gap |
| `P3_VALIDATION/P3_GATE_DESKTOP_SMOKE_REPORT.md` | the release binary in a real Windows window: all 40 §61 steps, the CLI/desktop run-id parity measurement, native dialogs, database handled reversibly |
| `P3_VALIDATION/P3_GATE_DESIGN_CHECKLIST.md` | `AGENTS.md` 11 / `DESIGN.md` review with the greps and token counts run rather than asserted |

Measured on this machine: `cargo test --workspace` **556 passed / 0 failed / 0 ignored across 28
executable suites** (an earlier revision of this line said 715 in 42, which summed a gate log where the
`drift` group re-runs the desktop crate — corrected, not re-scoped), UI **135 passed** in 6 files,
`python scripts/check.py` **15/15**, `cargo fmt --all -- --check` and
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean, `git diff --check` clean,
`drift` 6/6, `deny/cargo-deny` executed for real — locally green on an index that was already stale, see
the CI note below — `core-smoke` 3/3. The CLI and the desktop produced the
same run id — `gate-a64631b287fcb33478e23ac6462dacc80e1a39cac1e321e047d02bb50ff152cc` — for one project,
one policy and one HEAD. `fwsight gate` exits 0 / 4 / 5 as the policy decides. Baseline stays **v0.6.0**;
`SCHEMA_VERSION` moved 2 → 3 through additive `0003_gate_history.sql` with v1→v3 and v2→v3 upgrade tests;
no new third-party package entered the graph beyond what ADR-0027 admitted; no new design token.

`NOT MEASURED`: peak RSS, and no Gate throughput or latency number is claimed anywhere. `NOT RUN`:
fuzzing, a macOS or Linux window, keyboard-only traversal of the shipped binary. `NOT VERIFIED`: desktop smoke step 30's prior-run re-read *through the window* — the Release
page has no run-id input, so `get_gate_run` is tested but was not observed reopening an older record; and
the gate's own first pass was **13/14** — `frontend/test` failed on a `compare.test.tsx` row query racing
the table's own `Loading symbol changes…` state (defect I, a test-side race in P2's file, now fixed with
the assertions unchanged and the gate re-run green). That is recorded rather than smoothed over.

**P3's code is on the remote, and its first CI pass was 6 of 7.** The closure commits were pushed after
§69 was settled: `origin/main` is `219178af195569ec6b13728d84d992ef78df8c04`, and Run
[`36774472141`](https://github.com/2023violet/FirmwareSight/actions/runs/36774472141) completed
`failure` with one red job, `Dependency policy`, which reported `error[yanked]` on `yoke-derive 0.8.3` —
a transitive proc-macro under `url`, yanked on crates.io at 2026-09-30T13:19:39Z, i.e. after this
machine's `deny` step had passed on an older index. Rust on both platforms, Desktop UI on both
platforms, the macOS core smoke and the drift check all passed remotely. The lockfile moved to `0.8.4`
in a successor commit and the gate was re-run 15/15, and the push after that (Run `36779321479` on
`2d1bcea`) came back **success, 7 of 7 jobs** — the yank fix verified on CI's fresh index. The head after
that, `893a635`, went red a third time on a different job: `Desktop UI (windows-latest)` lost a
call-count race in `compare.test.tsx` that the ubuntu job and eight local runs had passed, fixed as
defect J with the assertion kept and proved by mutation; the push after that, Run `36783457030` on
`02e8a81`, concluded **success with 7 of 7 jobs**; its own documentation-only successor, Run
`36784382005` on `f66a93d`, did too, and that is where this pack stops naming runs — a doc-only
successor's own result is read with `gh run list`, not chased into itself.
Before that first P3 push, `origin/main` was `32b23aa`,
whose Run #19 `36665007523` is `completed / success` with 7 of 7 jobs, following Run #18
`36648718199` on the P2 implementation tree `4a77ea1`. §67 forbade writing a future CI run into the
commit that would trigger it, so nothing was predicted and this paragraph is a successor record.

Three product defects were found by this round's own validation and fixed in product code: a version
pattern embedded in an evidence locator made a Gate run unpersistable for any project using the only MVP
version source; one build produced two run ids across the two surfaces; and a disabled primary button kept
its accent border. Each has a regression test named in the closure entry of `.ai/DECISIONS.md`.

The license gap deliberately did not close with the stage: `license = "Proprietary"` stands in the
`[workspace.package]` table of the root `Cargo.toml`, there is no root `LICENSE`, and
`OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`
remains. `AGENTS.md` 9 puts that decision in front of a human.
