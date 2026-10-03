---
title: "P5 CI Authority"
doc_id: "FS-P5-CI-AUTHORITY"
product: "FirmwareSight"
version: "1.0"
status: "IN_PROGRESS"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-03"
---

# P5 — the authoritative CI job set (prompt §41)

§41 requires this file if P5's final matrix differs from the ten jobs it lists, and forbids a silent
job-count drift. It records the set, why it is that size, and every run measured against it. A run id is
read back from `gh run view <id> --json headSha,conclusion,jobs`, never recalled.

## The authoritative set is ten jobs

| # | Job name as Actions reports it | Runner | Group it runs |
| --- | --- | --- | --- |
| 1 | Rust (windows-latest) | `windows-latest` | `check.py --only rust` |
| 2 | Rust (ubuntu-latest) | `ubuntu-latest` | `check.py --only rust` |
| 3 | Desktop UI (windows-latest) | `windows-latest` | `check.py --only frontend` |
| 4 | Desktop UI (ubuntu-latest) | `ubuntu-latest` | `check.py --only frontend` |
| 5 | Generated output drift | `ubuntu-latest` | `check.py --only drift` |
| 6 | Dependency policy | `ubuntu-latest` | `check.py --only deny` |
| 7 | macOS Core Smoke | `macos-latest` | `check.py --only core-smoke` |
| 8 | Package Windows | `windows-latest` | `check.py --only package` |
| 9 | Package Ubuntu | `ubuntu-latest` | `check.py --only package` |
| 10 | Package macOS | `macos-latest` | `check.py --only package` |

Jobs 1-7 are the pre-P5 set, unchanged in name, runner, command and trigger condition. Jobs 8-10 are
new with the packaging commit and run on a push to `main` and on a manual dispatch, but **not** on a pull
request, which is the same cost rule job 7 already carries (`if: github.event_name != 'pull_request'`).

**Why packaging moved onto a push.** `05_ENGINEERING/06_CI_CD_BASELINE.md` put packaging on a nightly or
a release tag, and this repository has never had a scheduled workflow: `.github/workflows/` contains one
file, triggered by `push`, `pull_request` and `workflow_dispatch`. A row in the matrix that no workflow
implements cannot produce §41's evidence, so P5 runs the three package jobs on `main` for the length of
the stage. The change is written into that baseline document in the same commit, not left implied by the
diff.

## Runs measured against it

| Run | Head | Content of the head | Jobs green | Conclusion |
| --- | --- | --- | --- | --- |
| `37101619245` | `d83175a` | P5 start head (post-G2 remediation closure) | 7 of 7 | success |
| `37125456689` | `4cc8d93` | §5 governance opened + the audit + the archived prompt | 7 of 7 | success |
| `37127791999` | `812b472` | `P5_MIGRATION_DECISION.md` + migration `0005` | 7 of 7 | success |
| `37128593254` | `9e3b1de` | artifact versions unified on `0.6.0` | **6 of 7** | **failure** — `Desktop UI (windows-latest)`, a pre-existing `compare.test.tsx` race, not the version bump |
| `37129900728` | `20b03e3` | test-only repair of that race | 7 of 7 | success |
| _(packaging commit)_ | — | the first 10-job run | — | recorded here when it concludes |

The failed run is listed rather than dropped. Every P5 closure claim in this repository is "N of N on the
first attempt", and a head that went red is part of the chain that produced the head that did not:
`9e3b1de` touched sixteen files and neither `Compare.tsx` nor `compare.test.tsx` is among them, but the
runner it landed on is the one that lost the race, and pretending otherwise would make the green run
meaningless. The reproduction and repair are in `P5_PRODUCTIZATION_AUDIT.md` §0a, with the evidence root
outside this repository.

## What closes P5 against this file

§41 requires **all authoritative jobs green** at closure. That means a run of the closure head with
10 of 10 jobs concluded `success`, on the first attempt, with the job set above unchanged; if any job is
removed, renamed or still failing at that moment, this file records the exact set and the reason before
any closure sentence is written, and §41's "no silent job-count drift" is the rule the sentence is checked
against. Package jobs are not permitted to close on a skipped group: `check.py` records the desktop
package and its verification as `(skipped)` when the Tauri CLI is absent, which is honest locally and is
not a green CI job, because the CI jobs install the pinned CLI and upload with
`if-no-files-found: error`.
