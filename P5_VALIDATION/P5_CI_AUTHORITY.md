---
title: "P5 CI Authority"
doc_id: "FS-P5-CI-AUTHORITY"
product: "FirmwareSight"
version: "1.1"
status: "IN_PROGRESS"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-05"
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

diff.

**Ten jobs is not ten steps.** Job 5 runs `scripts/check.py --only drift`, which was seven steps from the
packaging commit until Commit E and is **eight** from Commit F1, where `drift/baseline integrity`
(`python scripts/verify_baseline_artifacts.py`) was added under `ADR-0029`. The authoritative set above did
not change and no job-count drift occurred, which is what §41 forbids; what changed is the load inside one
job, so the F1 candidate is measured by the same ten jobs with a step in one of them that did not exist
before. The full local gate moved with it: 16 steps through `80b47c4`, 17 from F1.

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
| `37159810291` | `2cdfced` | documentation only: that run read back into this file, and the governance sentences corrected to name the two jobs that carry the proof | **10 of 10** | success, first attempt |
| `37200245520` | `3400981` | Commit D's product head: storage-owned `integrity_check`, the WAL-safe online-backup snapshot taken before any older store is migrated, the migration matrix re-run against it, the closed 41-key Diagnostics allowlist with its Rust-side export, the typed startup refusal, `ADR-0028`, the installed-app walk and its two product fixes | **8 of 10** | **failure**, first attempt — the rule broke here, and the break is this commit's own doing. Two jobs red on one assertion: `macOS Core Smoke` and `Rust (ubuntu-latest)` each panicked at `integrity_and_backup.rs:814`, where the snapshot-retention test compared two whole-file byte strings that were **equal**, because the only difference its two fixtures could offer was the second-granular `applied_at` default on `schema_migrations` — a clock, not a fact. The product took no blame and the repair is test-only: `Rust (windows-latest)`, both `Desktop UI` jobs, `Generated output drift`, `Dependency policy` and **all three `Package` jobs** were green on the same head, so the installer this run attached is the one §27 walked |
| `37202016141` | `bccea88` | the test-only repair of that assertion: each v4 store now carries a named `sections` row and the test reads it back out of the standing snapshot, so the claim is about data rather than about a second of clock; eighth mutation proof (a stale snapshot left standing reddens the new read-back alone), 20 fresh-process runs of the repaired test, 131 storage tests green | **10 of 10** | success, first attempt, every job verified individually — including the two that were red on the head it repairs. `macOS Core Smoke` **and** `Rust (ubuntu-latest)` are the load-bearing pair for this head: they are the runners that lost the race, and a repair proven only on Windows would be no repair at all |
| `37202301591` | `90aa69d` | documentation: the failed head and its repair read into §0a and `P5_CI_AUTHORITY.md`, and **L26** recorded — `SHA256SUMS` verifies on the host that wrote it and disagrees with a clean checkout for 17 entries | **10 of 10** | success, first attempt. A docs-only head still owes the ten jobs, and this one owes them twice over: it is the commit that writes down a failure, so it had better not fail |
| `37204847474` | `956e250` | §32's read-back successor: report §8 carrying the twelve facts the prompt names, the two repair heads read back, and the "31-field" figure corrected to the **41** keys `ALLOWED_KEYS` actually declares. No product source in the diff | **10 of 10** | success, first attempt — §33's last box, and the end of Commit D. This successor's own run is the external evidence: per §32 no further commit is written to record it, and the closeout that follows records it here because it is now a *previous* head |
| `37212546755` | `57a904e` | the documentation closeout: the four handoff entry points reconciled with the tree Commit D actually left — the P3-era `556 / 135 / 15` figures labelled as a snapshot beside the measured `854 / 210 / 16 + 3 + 4`, the P5 head chain added to the `.ai/HANDOFF.md` ledger, `.ai/README.md`'s stage block rewritten, the install report's "not built" claims dated to the run that saw them, and `P5_MIGRATION_DECISION.md` brought to `DECIDED_AND_IMPLEMENTED`. No product source in the diff | **10 of 10** | success, first attempt, every job read individually |
| `37214675036` | `bfbc1aa` | the correction `57a904e` made necessary: `.ai/README.md` and `.ai/HANDOFF.md` no longer call any head "current HEAD", and the closeout's own run is recorded here rather than in the head it records. This is also the head Commit E started from, chosen by the owner after this run concluded rather than beside it | **10 of 10** | success, first attempt, all ten job names read — the fact that made `bfbc1aa` a safe base |
| no run of its own | `859648e` | **Commit E1**: the fixture cohort — 6 new sets and 31 new files under `fixtures/elf/p5-compat/` (manifest 25 → 56), `scripts/gen_p5_compat_fixtures.py`, `p5_compat_fixtures.rs` (14 tests), the `SHF_ALLOC` fix in `crates/firmwaresight-artifact/src/elf.rs`, the matrix and the fixture report, and the archived Commit E prompt | not applicable | **The push was one `git push` of two commits, and GitHub started one workflow run — for the tip head only.** `gh run list --json headSha` returns no run whose head is `859648e`; its bytes are verified by `37228929762`, whose tree contains them, and by the clean detached worktree recorded in `P5_COMPATIBILITY_FIXTURE_REPORT.md` §7a. Recording a run number against E1 would be inventing one |
| `37228929762` | `59d85c3` | **Commit E2**, and with it the whole Commit E tree: the §48 dispositions, the L19/L20 label work in `Analyze.tsx` and `Compare.tsx` with its six new `compare.test.tsx` and one new `details.test.tsx` case, L23's seventh instance repaired test-only in `history.test.tsx`, `P5_COMMIT_E_SCHEMA_DECISION.md` (a question, `STOPPED_FOR_ARCHITECT`), the workload-size sentence corrected, the governance closeout, and `DIRECTORY_TREE.txt` + `SHA256SUMS` regenerated in that order | **10 of 10** | success, attempt 1, run #65, `event: push`, `headSha` 59d85c308ad2c68cdc80956b47641a67ebf63b41 — every job read individually and every step read: `Rust (windows-latest)`, `Rust (ubuntu-latest)`, `Desktop UI (windows-latest)`, `Desktop UI (ubuntu-latest)`, `Generated output drift`, `Dependency policy`, `macOS Core Smoke`, `Package Windows`, `Package Ubuntu`, `Package macOS`. The single `skipped` step in the whole run is the conditional Ubuntu apt block on the Windows runner, which is the job's design and not a gate skip. Load-bearing for Commit E: the two Rust jobs (the fixture hashes and the derived expectations must survive three checkouts), `Generated output drift` (the tracked-set guard over 56 manifest paths), and all three `Package` jobs green with `Build and verify…` and `Upload the artifact set` both succeeding |
| `37230689636` | `6981625` | Commit E's §59 read-back successor: `59d85c3`'s run written into this file and the governance entry points, the "last head that changed a line of Rust" sentence dated instead of left to mislead, and §7a added to the fixture report for the clean detached worktree. Documentation only | **10 of 10** | success, attempt 1, every job and every step read individually — the first Commit E head able to carry its own run, because the run it needed belonged to the head before it |
| `37262147348` | `80b47c4` | **Commit E closure normalization**: the matrix status column closed to five canonical values (38 cells, 0 outside the vocabulary), the A–H dispositions closed to four with the Architect's mapping, **L15 answered as Option E**, `04_TECH/23` §7 added, the prompt archived. No path under `crates/`, `apps/`, `scripts/`, `fixtures/`, `schemas/`, `golden/` or `.github/` was touched | attempt 1 **9 of 10**, attempt 2 **10 of 10** | **two attempts, and both stay in the record.** Attempt 1's single red was `Generated output drift`: rustup on the Ubuntu runner logged "recovering from a partially installed toolchain", then `failed to install component: 'clippy-preview-x86_64-unknown-linux-gnu', detected conflict: 'bin/cargo-clippy'`, and rolled back ~0.5 s in without compiling a line — the provisioning class `BASELINE.yaml:514` already records for `157f749`. Read the log before touching the rerun button, and corroborate it: the same SHA's `Rust (ubuntu-latest)` and `Package Ubuntu` both passed, and they bootstrap the same toolchain. Then one `gh run rerun --failed` — the workflow has no `needs:` graph, so this re-executed that one job and carried the other nine attempt-1 executions into the attempt-2 object — drift returned **7 of 7**, and the run concluded **10 of 10 success**. Attempt 2 is therefore not written as first-attempt green |

Two heads between Commit D and Commit E are documentation-only, and no head in this file is described as "the
current HEAD". That is deliberate: the moment a commit calls itself the current head, pushing it is what makes
the sentence false, and `57a904e` shipped exactly that error in two places — `.ai/README.md` and
`.ai/HANDOFF.md` both named `956e250` as current — which the head after it corrected. The durable facts are the
chain above plus one date marker: `bccea88` was the last head to change a line of Rust **until Commit E**, whose
E1 head `859648e` changed `crates/firmwaresight-artifact/src/elf.rs` and E2 head `59d85c3` changed five files
under `apps/desktop/ui/src` (two of them product source). The moving fact is still `git rev-parse HEAD`, which is
where the next reader should get it.

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

`3400981` is the second failed head, and the two failures teach opposite lessons. `9e3b1de` was reddened by a
race it never touched; `3400981` was reddened by a race **it wrote**, in a test added by the commit under
review. And where `111fe32`'s row records a local gate that caught something CI missed, this head records the
inverse: the same test passed here on every run made before the push, passed the whole `16 of 16` gate twice,
and the two machines that disagree on clock granularity are the ones that found it. A snapshot-replacement
claim that survives only on a slow host is not a claim. Both red jobs, their assertion, and the mutation that
proves the repaired assertion carries the claim are in `P5_COMMIT_D_DESIGN.md` §11 and §12.

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
