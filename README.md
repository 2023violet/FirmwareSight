---
title: "FirmwareSight Project Baseline"
doc_id: "FS-ROOT-README"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-10-02"
---

# FirmwareSight v0.6.0

**Embedded Firmware Release Workbench**  
**Know exactly what ships.**

v0.6.0 is the **P0 Technical Foundation Baseline**: the first version of this repository that ships
source, and the version record of P0 closing `PASS`. It is built on `FirmwareSight_Project_Baseline_v0.5.1`
and does **not** redefine the product, the architecture, the gate definitions or the design contract.

```text
G0      PASS
P0      PASS — remote CI Runs #3, #4, #5 and #6, 7 of 7 jobs green on each; frozen at v0.6.0
G1      PASS — basis: P0 PASS, under ADR-0026 (2026-09-29). Before that date G1 required V0 PASS + P0 PASS
        and was NOT CLAIMED; the P0 and P1-A0 evidence packs record the older formula as it stood then
V0      NON_BLOCKING_USER_FEEDBACK_TRACK — 0 / 8 eligible external sessions, an honest zero that now
        gates no stage. The v0.1.0 prototype and protocol stay frozen for a possible later feedback round
Pre-G1  P1-A0 Real Artifact Intake — authorized by ADR-0025, green on Run #9, and CLOSED together with
        its evidence-identity / persistence correctness closure (bounded; it stopped at P1-A0)
P1      PASS / COMPLETE — the Analyze verb as one product verb: intake, summary, top contributors,
        bounded Sections / Symbols / Evidence details, the Evidence Inspector, and the US-001 bytes/KiB
        presentation switch. Evidence: P1_VALIDATION/, item by item against the frozen US-001 list.
        Shipped as f649afd (storage + shell), ae7759a (UI) and e63afaf (evidence + governance) in one
        push; remote CI Run #13 36556735551 on e63afaf is success, 7 of 7 jobs, and no run exists for
        the individual commits because the gate fires per push
P2      PASS / COMPLETE — Compare over persisted snapshots: Core-owned deterministic diff, bounded Compare
        IPC behind an opaque session handle, the second desktop page, `fwsight diff`, and portable Diff
        JSON v1 (urn:firmwaresight:schema:diff:1) plus self-contained HTML. Evidence: P2_VALIDATION/, item
        by item against the frozen US-002 list. It started from HEAD 7a13660, measured green on remote Run
        #16 36576568426. Its mid-round push c7fc2a3 went red on Run #17 36596452341 — .gitignore's
        `**/target/` had hidden the target half of the P2 fixture pair from git, so a clean checkout could
        not load it while every local run passed on bytes left on disk. cfee1e5 committed the missing half,
        un-ignored that path and added a `drift fixtures tracked` gate step; the head 4a77ea1 was then
        pushed and Run #18 36648718199 is success, 7 of 7 jobs. The round's own gate numbers stay LOCAL
        PASS, because they were measured before any push
P3      PASS / COMPLETE — Release Gate, authorized on 2026-09-29 by its own architect prompt (v1.1, supplied
        inline) and by ADR-0027, which puts project policy and read-only Git provenance in a fifth crate
        instead of in Core. It started from HEAD 32b23aa, measured green on remote Run #19 36665007523
        (success, 7 of 7 jobs), and closed on 2026-09-30 item by item against US-003 and PRD P0-5 in
        P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md: 556 Rust tests in 28 executable suites, 135 UI tests,
        15/15 gate steps, an 18/18 CLI Gate smoke, and all forty §61 desktop steps walked on the shipping
        binary. The stage closed as LOCAL PASS while its commits were still unpushed; they were pushed
        afterwards, and remote Run 36774472141 on 219178af completed failure with 6 of 7 jobs green —
        every build, UI, macOS-smoke and drift job passed, only Dependency policy failed, on a crate
        (`yoke-derive 0.8.3`) yanked on crates.io after the local deny step had gone green on a stale
        index. Fixed by a patch bump to 0.8.4, verified remotely by Run 36779321479 at 7 of 7 jobs green.
        The next push (893a635) went red on Desktop UI (windows-latest) alone: a call-count race in
        compare.test.tsx that the ubuntu job won on the same commit — defect J, fixed with its delta
        assertions unchanged, the fix proved by mutation, and Run 36783457030 on 02e8a81 then green on
        7 of 7 jobs — as was the documentation-only successor f66a93d on Run 36784382005, the last run
        recorded here
P4      PASS / COMPLETE — Release Bundle, opened 2026-09-30 by *FirmwareSight — P4 Release Bundle MVP
        Implementation, Execution Prompt v1.0 — Architect Reviewed* and closed 2026-10-01, the last core
        product-implementation stage of the ADR-0026 line: it assembles the trusted Analyze + Compare +
        Gate + accepted-Reviews result, the selected current artifacts and the Release Notes into a
        portable, independently readable, hash-verifiable, host-path-free directory that never
        overwrites without an explicit confirmation. Start fact measured before the first write:
        HEAD = origin/main = 323afad, green on remote Run 36810689645 at 7 of 7 jobs; start counts 556
        Rust and 135 UI. Verdict item by item in P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md; the fifty
        §59 steps of the shipped-window smoke in P4_BUNDLE_DESKTOP_SMOKE_REPORT.md; the final
        implementation head e799f2f green on Run 36872456446, 7 of 7 jobs
G2      PASS — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE (2026-10-01). The whole-MVP
        engineering closure audit ran under its own architect prompt and storage-path addendum: the
        complete workflow on the CLI and in the shipping window, byte-identical cross-surface parity, a
        clean detached worktree, fail-closed failure paths, a relocated bundle read with project, source
        and store absent. Evidence G2_VALIDATION/; product tree e35cfe7 (Run 36906482900), evidence head
        f75cbc5 (Run 36948719972), both 7 of 7 on the first attempt. Not production-ready, beta, RC or
        GA; not signed, installable, user-validated, security-clean or licensed open source
Post-G2 real-desktop acceptance: PASS_WITH_FINDINGS (2026-10-02), and its findings remediation closed
        the same day. 283 black-box cases of mouse, keyboard and native dialogs against the shipping
        binary found three product defects and no S0 or S1: E2E-F001 (S3, a surviving analysis did not
        say which selection it described), E2E-F002 (S2, a folder the engine refuses to replace was
        offered for replacement) and E2E-F003 (S3, a GNU ld MAP refused as another linker's output). All
        three fixed narrowly in e816dcb and 971015f with a regression and a mutation proof each, then
        re-validated on a rebuilt shipping binary: F001 3/3, F002 3/3 with a canary that survived, F003
        5/5 including a positive control that renders byte-identically. 770 Rust / 159 UI / check.py
        15/15, fix head green on Run #43 37100371601 at 7 of 7 on the first attempt, the owner's store
        restored byte-exact. Evidence POST_G2_E2E_REMEDIATION/, with the 283-case root outside the
        repository. G2 stays PASS and the product stays MVP CANDIDATE; large-file latency, peak RSS,
        125/150 % DPI and mouse-wheel behaviour are carried forward, not fixed
Pricing / willingness-to-pay / pilot signal: DEFERRED_POST_MVP
```

`v0.6.0` itself did not mean: V0 passed, G1 passed, the product MVP is complete, Compare / Gate / Bundle
workflows exist, an installer or signed build is ready, user value is validated, or the tree is free of
vulnerabilities. That statement stays true of the promotion act; `ADR-0026` later re-based G1 on the P0
evidence and opened MVP implementation on engineering grounds. See
`P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md` and
`09_ADR/ADR-0026-open-source-mvp-first-delivery.md`.

## What v0.6.0 adds

A Rust workspace with exactly four Phase-0 library crates (`core`, `artifact`, `report`, `storage`),
the `fwsight` CLI, a thin Tauri 2 desktop shell with a React summary, real ARM ELF + GNU ld MAP fixtures
with recorded provenance, goldens, ts-rs-generated typed IPC, minimal SQLite at schema version 2, a
512 MiB input guard, and one verification gate that CI and a laptop run unchanged.

```
python scripts/check.py                     # 15 steps across rust / frontend / drift / deny
python scripts/check.py --only core-smoke   # the macOS-on-main core smoke CI runs separately
cargo run -q --bin fwsight -- analyze fixtures/elf/p0-dual-region/firmware.elf \
  --map fixtures/elf/p0-dual-region/firmware.map --json
```

**556 Rust tests and 135 UI tests on the tree P4 starts from**, re-measured on 2026-09-30 with one
`cargo test --workspace` and one `corepack pnpm test` before the first write; **769 Rust tests in 41
executable suites and 155 UI tests in 6 files on the tree P4 closed on**, re-measured the same way on
2026-10-01. The chain that produced them:
104 and 19 at the v0.6.0 promotion, 142 and 31 after the P1-A0 slice and its correctness closure, 184 and 58
when P1 Analyze closed, 345 and 99 when P2 Compare closed, 556 and 135 when P3 Release Gate closed, and
769 and 155 when P4 Release Bundle closed. A Rust total must come from ONE invocation: an
earlier P3 revision read 715 in 42 suites, because summing a whole-gate log counts the desktop crate twice
(`drift` re-runs it after the workspace test step). Measurement status of every claim is in
`P0_TECHNICAL_VALIDATION/`, starting from `P0_EXIT_CHECKLIST.md`, in `P1_A0_VALIDATION/` for the intake
slice, in `P1_VALIDATION/` for the rest of Analyze, in `P2_VALIDATION/` for Compare, in
`P3_VALIDATION/` for the Gate and in `P4_VALIDATION/` for the Bundle.

## What v0.6.0 does not change

Product scope stays as frozen at v0.5.0. There is no fifth product verb, no new artifact class and no
new gate semantics; the version records technical closure, not a product change. `PRODUCT_BASELINE.md`
carries the dated note saying so.

## Current V0 status

# NON_BLOCKING_USER_FEEDBACK_TRACK — sample remains an honest zero

Formal eligible external participants completed:

`0 / 8 minimum`

No participant data is fabricated, and none was ever going to be: the missing input is real people, which
no engineering round can supply. On 2026-09-29 `ADR-0026` moved this track off the critical path, so the
zero no longer blocks P1, P2, P3, P4 or the gate. A Batch A activation earlier that same day verified the
recruitment pack and wrote nothing else, and the price-anchor prompt that would have followed it was
withdrawn before execution.

The instrument stays frozen and reusable - `V0_VALIDATION/prototype/` at `v0.1.0`,
`protocol/TASK_SCRIPT.md`, `sessions/TEMPLATE.md`, both registers - so a later feedback round is still
comparable with the protocol that exists. What V0 was meant to answer (does a real engineer read `Unknown`,
`Review` and a MAP request correctly) is now unverified rather than deferred, and
`ADR-0026` records that as an accepted risk.

Therefore this baseline does **not** claim:
- V0 PASS;
- V0 Conditional PASS;
- V0 FAIL based on users;
- user value validated, willingness to pay, or a purchase signal;
- product MVP complete, nor Compare / Gate / Bundle workflows existing;
- an installer, signed build, updater or release-ready package;
- zero vulnerabilities or a security-clean tree (`P0_KNOWN_LIMITATIONS.md` lists the accepted advisories);
- CRA compliance.

What it does claim, as of 2026-09-29: `P0 PASS`, and `G1 PASS` on that basis under `ADR-0026`.

## Prototype

Open:

`V0_VALIDATION/prototype/index.html`

No dependency installation is required.

## State model

```text
STATE A
ELF ✓
MAP not provided
Git not linked
Sections available
Symbols unavailable
MAP-dependent Gate findings UNKNOWN

      ↓ explicit Add MAP

STATE B
ELF ✓
MAP ✓ firmware.map
1,284 symbols
Dependencies enabled
MAP-dependent Gate findings re-evaluated

      ↓ invalid candidate replacement

STATE C
Parse Failure
Last Good Artifact preserved

      ↓ recover

STATE D
Bundle & History
```

## P0 Technical Vertical Slice — closed PASS

The slice reached `PASS` on evidence, not on a wording change. Remote CI Run #3 (`36399805005`) executed
the engineering-validated HEAD `1cd6309` and concluded `success` with **7 of 7 jobs green**; Run #4
(`36402637251`) executed the architect-reviewed HEAD `5e58f77` — documentation only, no source moved —
and concluded `success` with **7 of 7 jobs green**; Run #5 (`36416146281`) executed the promotion commit
itself, `738ae78`, and concluded `success` with **7 of 7 jobs green** again, which is what makes the baseline record green
on its own commit; Run #6 (`36419864513`) executed its consistency-closure successor `7d2f38a` and
concluded `success` with **7 of 7 jobs green**, verifying that the baseline's own duplicate-key and
stale-field fix did not break the gate. All four were read back with `gh run view`, not from this file:

```
gh run view 36399805005 --repo 2023violet/FirmwareSight
gh run view 36402637251 --repo 2023violet/FirmwareSight
gh run view 36416146281 --repo 2023violet/FirmwareSight
gh run view 36419864513 --repo 2023violet/FirmwareSight
```

It took six runs to get there and the two failures are kept as history in
`P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`. Run #1 (`36360310447`, `f9b8ccb`) had four jobs red; each
cause was reproduced with a command before being fixed — a `.gitattributes` text policy so a checkout
cannot change a fixture's bytes, icon drift judged by decoded pixels rather than by encoder bytes, a
`deny.toml` that cargo-deny 0.20.2 actually parses, Linux prerequisites on the Ubuntu job, and the
macOS core smoke `05_ENGINEERING/06_CI_CD_BASELINE.md` requires. Run #2 (`36378384225`, `ebda52d`) came
back six of seven green with one job missing a provisioning step its sibling job had.

The authorized desktop window launch found a sixth defect no test had reached: a second artifact raised
`UNIQUE constraint failed: evidence.id`, because `evidence` was keyed on `id` alone while
`04_TECH/15` §4 declares `Build 1─N Evidence`. Migration `0002` rebuilds the table on `(build_id, id)`;
schema version is 2.

`LOCAL PASS`, `CI PASS`, `NOT RUN` and `NOT MEASURED` stay different claims throughout the pack. What
promotion does not remove: peak RSS is `NOT MEASURED`, fuzzing is `NOT RUN`, and two RustSec advisories
(`RUSTSEC-2024-0429`, `RUSTSEC-2024-0370`) are **accepted as explicit P0 transitive risk** with five
revisit triggers — they are reachable only through the gtk-rs `0.18` line Tauri 2.12.0 requires, and no
compatible upgrade exists inside that pin.

## Read order

1. `README.md`
2. `PRODUCT_BASELINE.md`
3. `BASELINE.yaml`
4. `.ai/CURRENT_STATE.md`
5. `P0_TECHNICAL_VALIDATION/README.md`
6. `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`
7. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`
8. `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md`
9. `V0_VALIDATION/README.md`
10. `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`
11. `10_AUDIT/SOURCE_PROMPTS/README.md`
12. `.ai/ACTIVE_TASK.md` — currently `NONE`; G2 closed `PASS` on 2026-10-01, so the MVP engineering
    candidate is complete and any next track (V1 own-artifact validation, P5 productization) needs its
    own architect decision
13. `G2_VALIDATION/G2_EXIT_CHECKLIST.md` — the whole-MVP verdict, box by box
13a. `POST_G2_E2E_REMEDIATION/EXIT_CHECKLIST.md` — the three real-desktop findings, each box settled by
     the test, mutation proof or desktop measurement that closed it

## Batch A recruitment-ready status

Current V0 Batch A status:

`WAITING FOR REAL PARTICIPANTS`

The recruitment/screening/moderator/scheduling package is located at:

`V0_VALIDATION/batch_a/recruitment_ready/`

Formal external sessions remain:

`0 / 4–5 Batch A target`

Per the authorized Batch A Prompt, v0.5.2 is intentionally withheld until real Batch A evidence exists.

## Prior baselines

`FirmwareSight_Project_Baseline_v0.5.1` was a **V0 Workflow Prototype Validation execution patch** built
only on the unique `v0.5.0`: V0 authority takeover, the seven-screen clickable static prototype, the
coherent single fixture narrative, the explicit MAP STATE A→B transition, the Unknown Dependency
declaration flow, the Review acceptance audit flow, simulated signature-block recovery, conditional
Bundle creation, parse-failure recovery with Last Good Artifact preservation, the
moderator/screening/task/interview/privacy protocol, the session/metrics/payment/toolchain records, and
the automated internal state-machine and scope dry run. Its status record is the v0.5.1 row in
`CHANGELOG.md`; P0 created no new product or architecture decision, it built the first production code
against the existing ones.

P0 was not authorized by that V0 execution. It was authorized separately, by the prompt preserved in
`10_AUDIT/SOURCE_PROMPTS/README.md`.

## Next work

**`active_task: P5_PRODUCTIZATION`** — stage **P5**, state **`IN_PROGRESS`**, opened on 2026-10-03 by
*FirmwareSight — P5 Productization, Execution Prompt v1.0* (file, SHA-256 `722125f5…71e0ae`, archived in
`10_AUDIT/SOURCE_PROMPTS/`). Its first deliverable is done and is the only thing that could precede code:
`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` answers the prompt's §4 A–H from commands run on the tree, and
the owner's checkpoint settled version identity (artifacts unify on `0.6.0`), the migration decision
(`P5_MIGRATION_DECISION.md`, then `0005`), the package matrix (Windows with real install evidence here,
macOS and Ubuntu `CI_BUILD_ONLY`) and the store filename (`firmwaresight-p0.sqlite` stays). Packaging is now
enabled: `bundle.active` with NSIS / `.app`+`.dmg` / `.deb` as the canonical targets, three CI package jobs
on top of the seven gate jobs, `scripts/verify_package_artifacts.py` to check the packaged version and the
embedded frontend and write a **distribution** `SHA256SUMS.txt`, and a `drift/version identity` gate step —
evidence and the limits of it in `P5_VALIDATION/P5_PACKAGING_REPORT.md`, the job set and every run in
`P5_VALIDATION/P5_CI_AUTHORITY.md`. **The first 10-job run failed**: `37133706214`, head `0c031cd`, kept its
seven gate jobs green and reddened the three package jobs, because `cargo install tauri-cli` leaves a binary
named `cargo-tauri` while the group looked for `tauri` — each job installed the CLI, printed `SKIPPED: the
Tauri CLI is not installed on this machine`, summed that as `4/4 steps passed`, and was stopped only by its
own empty artifact upload. The fix is the rule rather than the filename: a skip now prints as `SKIP`, stays
out of the passed total, and fails the run when `CI` is set. Installing the pinned CLI here then produced
this repository's **first real package** — a 3,811,140-byte `FirmwareSight_0.6.0_x64-setup.exe` — and that
build surfaced a second defect the stub could not have found: the CLI resolves its frontend directory from
the process cwd, so the package step has to run from `apps/desktop`, not from `apps/desktop/src-tauri`. Head
`1055242` then became this repository's **first 10-of-10 run** (`37138881977`), the first to attach a built
package for Windows, Ubuntu and macOS — and downloading those sets found a third defect, this one in the
index itself: the darwin `SHA256SUMS.txt` gave the `.app` one line naming a directory, so `sha256sum -c`
answered `FAILED open or read` on a correct build. A directory bundle is now indexed file by file, and each
scanned payload records its own digest — which turned an inference into a measurement: two builds of one
tree differ in exactly 20 bytes, all of them linker identity (the PE `TimeDateStamp` and the RSDS CodeView
GUID), so a package digest identifies the set a build produced and is not an equality key across builds.
All three repairs, the artifact sets with their checksums, and the limits of what has been proved are in
`P5_PACKAGING_REPORT.md` §5 through §5e.
**No P5 verdict
exists yet** — the product is still the G2-passed **MVP CANDIDATE** at baseline **0.6.0**, and the prompt
forbids writing `P5 PASS`, `BETA`, `RC` or `GA` before closure evidence, and forbids a tag, a GitHub
Release, an updater, a certificate and any licence choice. V1 own-artifact / real-user validation still has
no prompt and stays unauthorized. The open-source licence is still `PENDING OWNER CONFIRMATION`: a
technical MVP candidate is not a licensed open-source release. The paragraphs below are the running
narrative of how the pointer got here and back
to empty, so each one keeps the words its own round wrote.

**`G2` closed `PASS` on 2026-10-01, under its own architect prompt and addendum.** The audit ran the gate
first and found one test race (G2-F1, fixed test-only in `e35cfe7`); drove the whole MVP through the CLI
twice and through the shipping window once, with Diff JSON, Diff HTML and the whole bundle byte-identical
across the two; proved the tree needs no ignored file in a clean detached worktree; and proved the
path boundary in 17 checks after the architect adjudicated `artifacts.path` as the local-only storage the
baseline always had. Evidence and the canonical known-limitations list: `G2_VALIDATION/`.

**After G2, on 2026-10-02, the product was driven end to end by hand rather than by assertion.** A
real-desktop acceptance round ran 283 black-box cases against the shipping binary — real mouse, real
keyboard, native dialogs, no IPC substituted for a click — and closed `PASS_WITH_FINDINGS`: three
product defects, no S0, no S1, the owner's live store restored byte-exact. A second round then fixed
exactly those three and nothing else, each with a regression that went red before the fix, a mutation
proof after it, thirty fresh-process repetitions, and a focused real-desktop re-check on a rebuilt
binary. Two of them are worth naming for what they changed in the product's voice: a preserved analysis
now says which selection it belongs to, including when two artifacts share one file name; and a folder
the engine will not replace is no longer offered for replacement. The third widened GNU ld MAP detection
to the whole text, because the banner sits behind preambles whose length the project does not control.
What the round deliberately left alone is listed where it was measured: large-file latency and peak RSS
carry forward as `PARTIAL / environment-sensitive` and `MEASURED FOR TESTED WORKLOAD`, and no parser,
memory-model or guard change was made to earn a performance claim. Evidence: `POST_G2_E2E_REMEDIATION/`.

*(Recorded at the time as `active_task: NONE`.)* `P1_ANALYZE_DETAILS` completed on 2026-09-29, so the
Analyze verb is one whole
product verb: bounded `Sections`, `Symbols` and `Evidence` queries over the SQLite snapshot that already
existed, three use-case IPC commands, top contributors, an Evidence Inspector, and the `bytes / KiB`
presentation switch `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` US-001 requires. Every mandatory US-001
item is green with its evidence named per item in `P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md`, and
**P1 is `PASS / COMPLETE`**.

**`P2 Compare` closed `PASS / COMPLETE` the same day, under its own architect prompt.** Compare takes two
snapshots that Analyze already stored and answers with old / new / signed delta, section and symbol
changes paired conservatively, growth ranked from changed rows, unpaired rows counted instead of guessed,
and object attribution reported unavailable with its real reason. `fwsight diff` and the desktop's Export
JSON produce the same portable document byte-for-byte, and the HTML export is self-contained. US-002 is
checked item by item in `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md`; the four defects the shipped window
and the cross-renderer check found are in the pack beside it, with the fifth — the fixture half git was
ignoring — in `P2_COMPARE_EXECUTION_REPORT.md` §5.1.

**`P3 Release Gate` closed `PASS / COMPLETE` on 2026-09-30, under its own architect prompt.** One verb,
`Gate`: `firmwaresight.toml` is read by the adapter ADR-0027 created, ten rules are judged in Core over
the stored build plus workspace provenance, the run and any review acceptance are persisted immutably, and
both surfaces show the same answer — measured, not asserted: the CLI and the desktop produced the identical
run id `gate-a64631b287fcb33478e23ac6462dacc80e1a39cac1e321e047d02bb50ff152cc` for one project, one policy
and one HEAD. US-003 and PRD P0-5 are checked item by item in
`P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md`; the three defects the shipped window and the cross-surface check
found — a version pattern quoted into an evidence locator, one build carrying two run ids, and a disabled
button keeping its accent border — are in that pack with the test that pins each.

**`P4 Release Bundle` closed `PASS / COMPLETE` on 2026-10-01, under its own architect prompt.** One verb,
`Bundle`, over results the earlier stages had already computed and stored: a release plan — id, version,
ten-file list with digests — assembled before any byte is written; a staged copy verified against its own
`SHA256SUMS` and manifest; publish by rename; replacement only under an explicit confirmation and only of a
destination this engine wrote, with rollback if the swap fails; and a `release_records` row written after
the bytes, never before. US-004 and PRD P0-6 are checked item by item in
`P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md`; the fifty §59 steps of the shipped-window smoke, the 23-ok CLI
smoke and the two independent readers (64/64 with schemas, 59/59 relocated with project, artifacts and
database all unavailable) are in the pack beside it. The boundaries held: Core stayed headless and
filesystem-free, no sixth crate and no new dependency entered, no host path crossed IPC or entered a
composed document, and the only clock a bundle carries is the moment a named person accepted a review.

The rule that kept P2 unstarted until its prompt arrived still holds for everything after it: looser
sequencing is not standing authorization, `ADR-0026` removed the research gate, not the per-stage prompt
requirement. That rule held for `P4 Release Bundle` exactly as it held for P2 and P3: P3's own prompt said
**stop after P3** and authorized no bundle, History page, Project Wizard, installer, signing, updater,
SBOM, cloud, account, telemetry, AI or pricing work, so P4 sat unstarted until the architect issued
*FirmwareSight — P4 Release Bundle MVP Implementation, Execution Prompt v1.0 — Architect Reviewed* on
2026-09-30. **That prompt ran, closed the last core product-implementation stage of the MVP line, and
returned the pointer to `NONE`** — and P4's own §78 said stop after P4, which keeps every one of those same
items, plus the G2 closure audit and any `v0.7.0`, unauthorized until something else arrives for them. If
V0 is ever resumed
for usability feedback, it resumes with real participants or not at all - the `v0.1.0` instrument, the
protocol and both registers are still there, and no session, quote, timing or count may be invented.

One gap P3 does not close and has no authority to close: the project is delivered as open source under
ADR-0026, while the root `Cargo.toml`'s `[workspace.package]` table still reads
`license = "Proprietary"` and the repository root has no
`LICENSE` file. `AGENTS.md` 9 places a license change in front of a human, so the round records
`OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` and changes nothing.

The boundaries P1 worked inside are inherited by whatever comes next, unchanged: no schema migration
without its own decision, no dependency without its own admission, no new design token - a genuinely
missing token stops a round rather than becoming a magic number - default page size 100 and hard maximum
500 enforced in Rust, no whole-table payload across IPC, details bound to the last-good `snapshotId`,
`Unknown` never rendered as zero, and addresses never unit-converted. `ADR-0026` relaxed research
**sequencing**; it relaxed no technical boundary, and `AGENTS.md` 2 / 7 / 11 still apply in full.

P3 moved two of those boundaries, and did so **with** the decision that gates them rather than around it:
`SCHEMA_VERSION` went 2 → 3 through additive migration `0003_gate_history.sql`, and `toml` / `regex` became
direct dependencies of the new `firmwaresight-project` crate under `ADR-0027`, with `cargo-deny` executed
for real and the measured lock delta recorded in `P3_VALIDATION/`. What did not move, and what binds the
next round unchanged: no new design token, no whole config file or host path across IPC, no generic file,
shell or Git command, no `rusqlite` type outside storage, and no Gate semantic outside Core.

What P1 left open on purpose, with the reason in each case: `sections.file_offset` and `symbols.address`
are nullable columns with no reason column, so a reason is lost at write time and closing that needs a
migration; one dead `Apply filter` click on an intermediate build was never reproduced and is reported
unresolved; and `drift/ipc bindings unchanged` cannot see an untracked binding file, which is why this
round verified regeneration by hash. See `P1_ANALYZE_EXECUTION_REPORT.md` §6.

History of the pointer, so the older documents are readable: `P1A0_REAL_ARTIFACT_INTAKE` (authorized by
ADR-0025) is **complete**, together with the narrow correctness closure that fixed the two defects its own
desktop smoke reported; and on the same day `V0_BATCH_A_EXTERNAL_VALIDATION` was activated, then demoted to
a non-blocking track by ADR-0026 hours later. `G1 = V0 PASS + P0 PASS` no longer holds as a rule - it is
`P0 PASS` now - but every document written before 2026-09-29 states the older formula because that is what
those rounds believed.
