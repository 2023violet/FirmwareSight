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
| `37143046338` | `53578e9` | a `.app` indexed file by file so `sha256sum -c` can read it, `bytes` summed for a directory bundle, and each scanned payload's own digest recorded | **10 of 10** | success, first attempt. Its darwin set was the point of the run: five index lines, every one `OK`, exit 0, where the previous run's three-line darwin index could not open the `.app`. Proof read from `gh run download`, not from the job summary, and reported in `P5_PACKAGING_REPORT.md` §5d |
| `37145302229` | `a674774` | the read-back of that run written into the reports, plus `04_TECH/17`'s corrected CLI-archive sentence | **10 of 10** | success, first attempt — docs only, so its value is that the record of the proof is itself on a green head |
| `37147434366` | `9ab089f` | the §38/§39 round on this host: the packaged product installed, run, uninstalled and reinstalled, with `P5_INSTALL_RECOVERY_REPORT.md` and the three findings it produced | **10 of 10** | success, first attempt — the first head in this repository whose evidence is an installer that was actually run on a machine rather than built |
| `37147577288` | `4719e1f` | that report corrected to say the uninstaller **empties** the install directory and leaves it behind, which is what the machine did | **10 of 10** | success, first attempt |
| `37154946484` | `e863d0c` | Commit C: §13 first-run onboarding, §14 Help/About with the window title fixed in Rust over a closed page enum, and §15–§18 local History over three new bounded storage read APIs with no new migration (19 storage + 11 desktop + 40 UI tests new) | **10 of 10** | success, first attempt. The two `Desktop UI` jobs are the load-bearing pair for this head: the History page's 40 UI tests and the rail's four-entry guard assertions ran on both `windows-latest` and `ubuntu-latest`, and the same head ran `Rust` on both, so the new reads are proven on two platforms' SQLite rather than one |
| `37156287999` | `e070508` | documentation only: Commit C's run read back into `P5_CI_AUTHORITY.md`, and four governance sentences corrected to say it had been read at all | **10 of 10** | success, first attempt. Worth its row for one negative fact: `Desktop UI (windows-latest)` was green on the head whose UI tree is byte-identical to the one that later lost L23's sixth instance **locally**, so CI's green said nothing about that race and the local full gate under its own compile load was what found it |
| `37158606478` | `111fe32` | §32's L22 ADR draft plus the identity premise test, and the test-only repair of L23's sixth instance | **10 of 10** | success, first attempt. `Desktop UI (windows-latest)` **and** `Desktop UI (ubuntu-latest)` are the load-bearing pair for this head: the repaired Compare suite ran on both, so the awaited second-wave query is proven on two runners rather than only on the host that lost it, and `Rust (windows)` + `Rust (ubuntu)` ran the new `bundle_builder.rs` identity test on two filesystems' line-ending behaviour |

Read back with `gh run view 37143046338 --json headSha,conclusion,jobs`, then
`gh run download 37143046338 -D <evidence root>/CI_PACKAGES_INDEX_FIX` — which at that moment returned the
darwin and Linux sets, `Package Windows` still uploading — and the third set with
`gh run download 37143046338 -n FirmwareSight-0.6.0-windows-x86_64`, whose files extract flat into the target
directory and were then moved into `CI_PACKAGES_INDEX_FIX/FirmwareSight-0.6.0-windows-x86_64/` beside the
other two. Each set was verified from inside its own directory with `sha256sum -c SHA256SUMS.txt`: darwin
five lines, Linux two, Windows two, every line `OK`, exit 0 in all three. The same verification had been run
against `37138881977`'s sets, downloaded whole with `gh run download 37138881977` after
`gh api repos/2023violet/FirmwareSight/actions/runs/37138881977/artifacts` listed them; two of those three
indexes returned `OK` on every line and the darwin index returned
`FAILED open or read` on the one line naming a directory. That defect, its repair and the bind test
that follows it are §5d of `P5_PACKAGING_REPORT.md`; it never changed a run's conclusion, because the job
verifies the bundle before it writes the index — which is exactly why a green tick on `Package macOS` was not
the proof, and why the artifact set has to be downloaded and checked line by line.

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
