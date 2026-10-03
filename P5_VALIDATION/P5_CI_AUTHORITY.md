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
| `37133706214` | `0c031cd` | packaging + release metadata + the three package jobs | **7 of 10** | **failure** — all seven gate jobs green; `Package Windows`, `Package Ubuntu` and `Package macOS` each installed the pinned CLI, then reported `SKIPPED: the Tauri CLI is not installed on this machine`, `4/4 steps passed`, exit 0, and were caught red by their own upload step (`No files were found with the provided path: target/dist-package/`) |
| `37138881977` | `1055242` | the CLI found through `cargo tauri`, a `SKIP` that cannot read as a pass, and the package step run from `apps/desktop` | **10 of 10** | success, first attempt — and the first run in this repository's history to attach a built package: `FirmwareSight-0.6.0-windows-x86_64`, `FirmwareSight-0.6.0-linux-x86_64` and `FirmwareSight-0.6.0-darwin-arm64`, each with its own `SHA256SUMS.txt` and `artifact-metadata.json` |
| _(the checksum-index repair)_ | — | a `.app` indexed file by file so `sha256sum -c` can read it | — | recorded here when it concludes |

Read back with `gh run view 37138881977 --json headSha,conclusion,jobs` and
`gh api repos/2023violet/FirmwareSight/actions/runs/37138881977/artifacts`; the three sets were then
downloaded with `gh run download 37138881977` and verified with `sha256sum -c` in the evidence root, where
two of the three indexes returned `OK` on every line and the darwin index returned
`FAILED open or read` on the one line naming a directory. That defect and its repair are §5d of
`P5_PACKAGING_REPORT.md`; it does not change this run's conclusion, because the job verifies the bundle
before it writes the index, and the index is what §41 asks a stranger to be able to check.

The failed run is listed rather than dropped. Every P5 closure claim in this repository is "N of N on the
first attempt", and a head that went red is part of the chain that produced the head that did not:
`9e3b1de` touched sixteen files and neither `Compare.tsx` nor `compare.test.tsx` is among them, but the
runner it landed on is the one that lost the race, and pretending otherwise would make the green run
meaningless. The reproduction and repair are in `P5_PRODUCTIZATION_AUDIT.md` §0a, with the evidence root
outside this repository.

`37133706214` is listed for a second reason: it is the run that showed the gate's own reporting could lie.
The failure was not the installer — no installer was attempted — and it was not visible in the group result
at all. `cargo install tauri-cli` leaves a binary named `cargo-tauri`, which is run as `cargo tauri`; the
group probed the bare name `tauri`, did not find it, printed a skip, and summarised `4/4 steps passed`. Only
`if-no-files-found: error` on the upload step made that visible as a failure. The repair and its three local
proofs are `P5_PACKAGING_REPORT.md` §5.

## What closes P5 against this file

§41 requires **all authoritative jobs green** at closure. That means a run of the closure head with
10 of 10 jobs concluded `success`, on the first attempt, with the job set above unchanged; if any job is
removed, renamed or still failing at that moment, this file records the exact set and the reason before
any closure sentence is written, and §41's "no silent job-count drift" is the rule the sentence is checked
against. Package jobs are not permitted to close on a skipped group: `check.py` records the desktop
package and its verification as `(skipped)` when the Tauri CLI is absent, which is honest locally and is
not a green CI job, because the CI jobs install the pinned CLI and upload with
`if-no-files-found: error`.
