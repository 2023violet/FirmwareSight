---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-10-03"
---

# ACTIVE TASK

```text
P5_PRODUCTIZATION — stage P5, state IN_PROGRESS, opened 2026-10-03.
Authorized by FirmwareSight — P5 Productization, Execution Prompt v1.0 (file, SHA-256
722125f5aa68e324ba1dea4826f66d8392acab9ad9a015c4919ed5ade471e0ae, 73,722 bytes, 3,442 lines), archived in
10_AUDIT/SOURCE_PROMPTS/ and registered in that directory's README.
Goal: carry the G2-passed MVP CANDIDATE to a Productized MVP a stranger engineer can install, understand,
use (Analyze / Compare / Gate / Bundle), inspect in local History and Diagnostics, recover from, and
uninstall or reinstall, without the dev team present.
Discipline: AUDIT FIRST / NO FEATURE SPRAWL / LOCAL-FIRST / FAIL CLOSED / PRESERVE UNKNOWN / PACKAGE WHAT
WAS PROVEN / DO NOT CLAIM BETA, RC OR GA.

AGENTS.md 1 still binds: this pointer names one stage and nothing beyond it. P5 does not authorize V1,
G3, a private beta, a release, a licence change, a signing or updater capability, or any new product verb,
format, adapter or crate.
```

## What is already true when P5 opens

The product is **not** in question: G2 is PASS, Product MVP is ENGINEERING COMPLETE, the state is
**MVP CANDIDATE**, the baseline is **0.6.0**, and the three post-G2 findings were closed on 2026-10-02 in
`e816dcb` and `971015f`. What moved is the tree, so the G2 numbers are the history of a different commit:
**Rust is 775 tests, UI is 160 in 6 files, and `scripts/check.py` is 16 steps** (the drift group gained
`version identity` when packaging landed).

`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` is written and answers §4 A–H from commands run at `d83175a`
(HEAD = `origin/main`, tree clean, Run `37101619245` 7 of 7). It found no package has ever been produced
(`bundle.active: false`, no CI packaging step, no `.icns`), that every artifact version is `0.1.0` while
`0.6.0` is document-only, that History needs read APIs rather than tables, that migration `0005` is
required for the L6/L7 numeric-Unknown reasons and not for History, that `PRAGMA integrity_check` and any
database backup appear nowhere, and that the committed fixtures are all `arm-none-eabi-gcc` + GNU ld with
zero Clang-produced evidence despite the cohort claim.

## What has landed since the audit

| Commit | What it did | Gate run | Result |
| --- | --- | --- | --- |
| `4cc8d93` | §5 governance opened + the audit + the archived prompt | `37125456689` | success, 7 of 7 |
| `812b472` | `P5_MIGRATION_DECISION.md`, then migration `0005` and its write/query/IPC/UI path | `37127791999` | success, 7 of 7 |
| `9e3b1de` | artifact versions unified on `0.6.0`; seven goldens regenerated | `37128593254` | **failure, 6 of 7** |
| `20b03e3` | test-only repair of the pre-existing race that run lost | `37129900728` | success, 7 of 7 |
| `0c031cd` | `bundle.active`, three CI package jobs, `scripts/verify_package_artifacts.py`, `drift/version identity` | `37133706214` | **failure, 7 of 10** |
| `1055242` | the package group finds `cargo-tauri` through `cargo tauri`, a `SKIP` can no longer read as a pass, and the step runs from `apps/desktop` | `37138881977` | success, **10 of 10** — and the first run to attach a built package on all three platforms |
| `53578e9` | a `.app` indexed file by file, so the index §41 asks for is readable by `sha256sum -c`, plus each payload's own digest in the metadata | `37143046338` | success, **10 of 10** — and its darwin set, downloaded and checked, closed the index row it was fixing |
| the commit that lands this row | the read-back written into `P5_PACKAGING_REPORT.md` §5d/§5e and §9, `P5_CI_AUTHORITY.md`, the audit, and `04_TECH/17`'s stale sentence about the CLI companion corrected — the CI sets do archive it, and all three were listed | `37145302229` | success, **10 of 10** |
| the commit that lands this row too | `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md`: §38 A–L and §39 measured on this host with the owner's store parked, hashed and restored | `37147434366`, `37147577288` | success, **10 of 10** each — and §38 C stayed unimplemented, because onboarding did not exist yet |
| the commit that lands this row as its code | §13 first-run onboarding + §14 Help/About (with L21's window title fixed in Rust over a closed page enum) + §15–§18 local History over three new bounded storage **read** APIs and **no new migration**. 19 storage + 11 desktop + 40 UI tests, seven mutation proofs, `P5_ONBOARDING_HISTORY_REPORT.md` and its design checklist | `37154946484` | success, **10 of 10**, first attempt — locally too: **16 of 16** steps, **812 Rust / 200 UI** |

The failure is recorded rather than re-run until a green attempt appeared: `Desktop UI (windows-latest)`
lost a race in `compare.test.tsx` that predates P5 (`055b54e` closed the same shape at the pager and said
it had left the rest). Reproduced here without CI — 1 of 30 fresh runs, and 3 of 3 under injected mock
latency — and repaired by awaiting the one synchronous read, no production source touched. `P5_VALIDATION/
P5_PRODUCTIZATION_AUDIT.md` §0a carries the evidence; the class is worth a sweep, which is a P5 validation
task, not a licence to rewrite unrelated tests.

`37133706214` is the second red run and the more instructive of the two, because nothing about the product
was wrong. All seven gate jobs passed; the three package jobs installed `tauri-cli@2.12.1`, looked for a
binary named `tauri` — `cargo install` leaves `cargo-tauri`, run as `cargo tauri` — reported a skip, and
printed `4/4 steps passed` with exit 0. What made them red was their own `if-no-files-found: error` upload
step, i.e. the failure surfaced one step after the verification lied. The repair is the rule, not the
filename: `check.py` distinguishes `SKIP` from `PASS` in its summary and totals, and exits non-zero when a
skip happens in CI, where every tool the gate needs is installed by the job itself. Three local proofs are
in `P5_VALIDATION/P5_PACKAGING_REPORT.md` §5, including a stand-in `cargo-tauri` that measured cargo's
argument forwarding and then failed verification for building nothing.

Installing the pinned CLI on this host then turned the group from a probe into a build, and it found the
second defect: run from `apps/desktop/src-tauri`, the CLI's own `beforeBuildCommand` hook executed in a
directory with no `package.json` (`ERR_PNPM_NO_IMPORTER_MANIFEST_FOUND`), because the CLI resolves its
frontend from the **process cwd** (`tauri-cli-2.12.1/src/helpers/app_paths.rs:148,153-174`) and this
repository keeps `ui/` and `src-tauri/` as siblings. The package step now runs from `apps/desktop`, and from
there it produced this repository's first installer — `FirmwareSight_0.6.0_x64-setup.exe`, 3,811,140 bytes,
`c5c8cf23…` in `target/dist-package/SHA256SUMS.txt`, verified with `sha256sum -c`. That head then went
**10 of 10 green** on Run `37138881977` — the first run in this repository to attach a built package on all
three platforms — and reading those artifacts back found the third defect (§5d of `P5_PACKAGING_REPORT.md`):
the darwin index gave the `.app` one line naming a directory, so `sha256sum -c` answered
`FAILED open or read` on a correct build. The next run fixed it and was read back the same way: Run
`37143046338` at head `53578e9`, **10 of 10**, five darwin index lines all `OK` with exit 0, and one flipped
byte in a copied `Contents/Info.plist` making the index exit 1 and name the file. Built is now installed:
§38 A–L and §39 ran on this host against the packaged installer, with the owner's store parked, hashed and
restored byte-identically, and are transcribed in `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md`. §38 C —
first-run onboarding — has since been **built** (Commit C, with L21's title fix that same round found on the
packaged build), but not yet **operated in an installed binary**, so §64's full journey stays open until that
walk runs after Commit D.

## What the owner decided at the checkpoint

The cadence the owner set was: audit first, then a checkpoint before any product code. That checkpoint ran
on 2026-10-03 and settled four questions.

| Question | Answer |
| --- | --- |
| Which version identity wins | **Unify the artifacts on `0.6.0`**, with the workspace version as the single source; `baseline_version` stays `0.6.0` and nothing is tagged or released |
| Migration `0005` for L6/L7 | **Write `P5_MIGRATION_DECISION.md` first, then add it** — additive columns for the two numeric `Unknown` reasons plus the `builds.created_at` index |
| How far the package matrix goes | **Windows with a real install here, macOS and Ubuntu `CI_BUILD_ONLY`** — those two rows may never be written as `SUPPORTED` |
| The store named `firmwaresight-p0.sqlite` | **Keep it.** No rename, no data move; the path becomes visible in Diagnostics instead |

Two questions are not this round's: §32 sends L22 (release identity versus line endings) to the Architect
as `P5_RELEASE_IDENTITY_ADR_DRAFT.md` followed by a STOP, and §54 forbids this agent from choosing a
licence, so `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` stands in every P5 document.

## What P5 must not write

No `P5 PASS`, `B1 READY`, `BETA`, `RC` or `GA` until closure evidence exists (§5). No tag, GitHub Release
or published installer (§72). No signing or notarization executed — those statuses stay
`READY_NOT_EXECUTED` (§11) — and updates stay `UPDATE_READY_MANUAL` (§12): no updater, no endpoint, no
certificate, no committed private key. No "security clean"; the permitted sentence is "dependency policy
passes with documented accepted risks". No new network capability, telemetry, analytics SDK, generic shell
or filesystem permission, and native dialogs stay Rust-side.

## What closed before P5

`G2_ENGINEERING_CLOSURE_AUDIT`, authorized by *FirmwareSight — G2 Product MVP Engineering Closure Audit,
Execution Prompt v1.0 — Architect Reviewed* (file, SHA-256
`3c6ab83e11ce6a4c91a609dd0a6bc318e700dc0cbbccd3d4493eaf65c3e2bac9`, archived in
`10_AUDIT/SOURCE_PROMPTS/`) and *G2 Storage Path Semantics Clarification Addendum v1.0* (inline).

| Document | Holds |
| --- | --- |
| `G2_VALIDATION/G2_EXIT_CHECKLIST.md` | the §34 checklist, persistence rows as the addendum corrected them, and the verdict |
| `G2_VALIDATION/G2_EVIDENCE_MATRIX.md` | every requirement classified and cited; the 17-point path boundary; the findings; the PRD metrics |
| `G2_VALIDATION/G2_END_TO_END_SMOKE_REPORT.md` | the CLI chain, the shipping window S1–S43, parity, failure and recovery, fail-closed inputs |
| `G2_VALIDATION/G2_ENGINEERING_CLOSURE_REPORT.md` | what was done and decided, including the remote closure |
| `G2_VALIDATION/G2_KNOWN_LIMITATIONS.md` | the one canonical list, 25 rows, none blocking |

Measured on the audited tree: one `cargo test --workspace` **769 / 0 / 0**; UI **155 in 6 files**;
`check.py` **15/15**, and **17/17** in a clean detached worktree; CLI and desktop byte-identical on Diff
JSON, Diff HTML and the whole bundle; independent reader 64/64, relocated 59/59. Remote: the product tree
`e35cfe7` on Run `36906482900` and the evidence head `f75cbc5` on Run `36948719972`, each 7 of 7 on the
first attempt. Findings: G2-F1 closed (test-only), G2-F2 adjudicated as expected local-only storage, G2-F3
recorded (tooling).

`NOT MEASURED`: peak RSS and a 500 MB working set in the window — not G2 blockers under the prompt's §27,
and not claimed. `NOT RUN`: fuzzing, a macOS or Linux window, a second Windows host, any DPI other than
100%.

**The closing commit's own CI run is not written into it.** Prompt §43 allows exactly one successor and
makes its run external evidence, read with `gh run list` and reported, not chased into another commit.

## What G2 PASS does not mean

Not G3 productization, private beta, release candidate or GA; not production-ready, signed or installable;
not real-user validated (V0 stays `0 / 8`) or commercially validated; not security-clean (two accepted
RustSec advisories). And not a licensed open-source release: `license = "Proprietary"` stands in the root
`Cargo.toml`, there is no root `LICENSE`, and `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`
remains — `AGENTS.md` 9 puts that decision in front of the owner. Baseline stays `0.6.0`; no tag, GitHub
Release, installer or `v0.7.0` was created.

## What closed after G2, and what that changes here

Those two rounds did not move the pointer: `active_task` stayed `NONE`, G2 stayed `PASS`, and the product
stayed an MVP CANDIDATE. What moved was the tree, so the numbers in the section above are the history of a
different commit rather than the present: the remediation round left **770 Rust / 159 UI**, P5's early
commits moved it to **775 Rust / 160 UI / `check.py` 16 steps** (migration `0005`'s five storage tests, the
Details reason test, and the `version identity` drift step), and **Commit C moved it again to 812 Rust /
200 UI in 8 files** on those same 16 steps — 19 storage History reads, 11 desktop boundary tests, 40 UI
tests over the two new screens. The evidence directory that round added is `POST_G2_E2E_REMEDIATION/`.

On 2026-10-02 a real-desktop acceptance round drove the shipping binary through 283 black-box cases and
closed `PASS_WITH_FINDINGS` — three product defects, no S0, no S1 — and a follow-on round fixed exactly
those three (`e816dcb`, `971015f`), each with a regression written first, a mutation proof after, and a
focused real-desktop re-check against a rebuilt binary. Neither round widened anything: no P5 surface, no
parser performance work, no schema, migration or dependency change, and no licence decision.

The rule above bound, and both rounds obeyed it: with no active task, no agent may create business
functionality or pick the next track. P5 has since been picked the only legitimate way — an architect
prompt of its own, archived with its hash, which is what opened this task. V1 still has no prompt and stays
unauthorized. The carried forward list — large-file latency, peak RSS, 125/150 % DPI, mouse-wheel,
`update_goldens`, the E2E harness — is carried into P5 as a disposition to answer (§60), not as a licence
to widen scope.
