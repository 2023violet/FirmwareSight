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
Rust is **770** tests, UI is **159** in 6 files.

`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` is written and answers §4 A–H from commands run at `d83175a`
(HEAD = `origin/main`, tree clean, Run `37101619245` 7 of 7). It found no package has ever been produced
(`bundle.active: false`, no CI packaging step, no `.icns`), that every artifact version is `0.1.0` while
`0.6.0` is document-only, that History needs read APIs rather than tables, that migration `0005` is
required for the L6/L7 numeric-Unknown reasons and not for History, that `PRAGMA integrity_check` and any
database backup appear nowhere, and that the committed fixtures are all `arm-none-eabi-gcc` + GNU ld with
zero Clang-produced evidence despite the cohort claim.

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
different commit rather than the present: **Rust is 770 tests, UI is 159 in 6 files**, and the newest
evidence directory became `POST_G2_E2E_REMEDIATION/`.

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
