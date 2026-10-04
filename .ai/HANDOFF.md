---
title: "Project Handoff"
doc_id: "FS-AI-004"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-04"
---

# Handoff — FirmwareSight v0.6.0 / P0 closed PASS / G1 PASS on the P0 basis / P1 Analyze COMPLETE / P2 Compare COMPLETE / P3 Release Gate COMPLETE / P4 Release Bundle COMPLETE / G2 PASS — MVP CANDIDATE / P5 productization IN_PROGRESS — active_task P5_PRODUCTIZATION

## Purpose

P0 is finished, promoted and **frozen**: `P0 = PASS`, `baseline_version = 0.6.0`, and no further P0
closure or promotion prompt will be written. The one bounded pre-G1 slice, `P1-A0 Real Artifact Intake`,
is complete together with its correctness closure.

The direction changed on 2026-09-29 by `ADR-0026-open-source-mvp-first-delivery.md`: FirmwareSight builds
its local MVP first, `G1 = P0 PASS` for that delivery, and V0 is a **non-blocking** user-feedback track
rather than a precondition. The engineering round that followed - **`P1_ANALYZE_DETAILS`** - finished the
same day: bounded Sections, Symbols and Evidence queries, top contributors, the Evidence Inspector, and
the `bytes / KiB` switch US-001 requires. Later that day the architect issued
*FirmwareSight — P2 Compare MVP Implementation, Execution Prompt v1.0 — Architect Reviewed*, and that
round finished the same day: **`P2` is `PASS / COMPLETE`**. Compare reads persisted snapshots, Core owns
every diff semantic, and one portable document comes out of the CLI and out of the desktop byte-for-byte.
`P2_VALIDATION/` holds the four documents and US-002 is checked item by item in
`P2_COMPARE_EXIT_CHECKLIST.md`. The architect then issued
*FirmwareSight — P3 Release Gate MVP Implementation, Execution Prompt v1.1 — Architect Reviewed*, and that
round finished on 2026-09-30: **`P3` is `PASS / COMPLETE` and SEALED.** One verb, `Gate`,
now runs end to end — `firmwaresight.toml` read by the adapter ADR-0027 created, ten rules judged in Core,
an immutable run and an immutable review acceptance in SQLite, `fwsight gate`, and the desktop `Release`
page. US-003 is checked item by item in `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md`. The same day the
architect issued P4's prompt, and that round finished on 2026-10-01: **`P4` is `PASS / COMPLETE`**, so
**`active_task` is `NONE` again**. One verb, `Bundle`, packaged the Gate's answer together with Analyze,
Compare, accepted Reviews, the shipped artifacts and the Release Notes into a directory another person
can read and verify without FirmwareSight running; US-004 is checked item by item in
`P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md`. The MVP product line was then complete, and the architect
opened the **G2 engineering closure audit** with its own prompt and a storage-path addendum. It closed
on 2026-10-01: **`G2` is `PASS` — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE**, and
`active_task` is `NONE` again. Evidence: `G2_VALIDATION/`, verdict box by box in
`G2_EXIT_CHECKLIST.md`. If you were sent here to "continue G2", it is closed; V1 own-artifact validation
and P5 productization each need a separate architect decision, and nothing here authorizes either.

**That last sentence held until 2026-10-03, when the architect issued P5's prompt.** `active_task` is now
**`P5_PRODUCTIZATION`**, stage **P5**, state **`IN_PROGRESS`**. The prompt came as a file (SHA-256
`722125f5aa68e324ba1dea4826f66d8392acab9ad9a015c4919ed5ade471e0ae`, 73,722 bytes, 3,442 lines), is archived
in `10_AUDIT/SOURCE_PROMPTS/`, and its §4 required a productization audit before any product code:
`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` answers §4 A–H from commands run on the tree at `d83175a`, and
the owner's checkpoint settled the four questions it reserved for the owner — artifact versions unify on
`0.6.0`, migration `0005` only after `P5_MIGRATION_DECISION.md` is written, the package matrix is Windows
with a real install on this host plus macOS/Ubuntu `CI_BUILD_ONLY`, and the store keeps the name
`firmwaresight-p0.sqlite` with its path made visible instead of moved. L22 goes to the Architect as an ADR
draft — `P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md`, written, changing no behaviour, and it stops there
per §32 — and the licence stays the owner's (§54 forbids this round from choosing one). **P5 is IN_PROGRESS,
not passed:** the product is still the G2-passed MVP CANDIDATE at `0.6.0`, and no `P5 PASS`, `BETA`, `RC`,
`GA`, tag, Release, published installer, signature, updater or new network capability may be written before
closure evidence exists. If you were sent here to "continue P5", read the audit and `ACTIVE_TASK.md` first —
the workstreams and the commit split are the prompt's §6 and §61, not this file's invention.

Four commits of that split are done, and the fifth is the one that makes the rest installable. Governance
and the audit landed in `4cc8d93` (Run `37125456689`, 7 of 7); migration `0005` and its written decision in
`812b472` (Run `37127791999`, 7 of 7); the `0.6.0` artifact unification in `9e3b1de`, whose gate run
`37128593254` **failed 6 of 7** on a pre-existing `compare.test.tsx` race that the commit did not introduce
and did not cause; `20b03e3` repaired that race in one awaited query and came back 7 of 7 on Run
`37129900728`. The packaging commit `0c031cd` then enabled `bundle.active`, added the three §41 package jobs
(the authoritative set is **ten** now — `P5_CI_AUTHORITY.md` is the running record), and wrote
`scripts/verify_package_artifacts.py`, whose §8 "no dev-server dependency" check had to be built on the
embedded asset keys rather than on the absence of `localhost:5173`: both a good and a broken binary contain
that string, so the intuitive check passes exactly the builds it should refuse. `04_TECH/17` and
`05_ENGINEERING/06_CI_CD_BASELINE.md` carry the packaging decisions; `P5_VALIDATION/P5_PACKAGING_REPORT.md`
carries the measurements and, in §9, the list of what packaging has *not* yet proved. That commit's own run
`37133706214` **failed 7 of 10**, and the reason is the one thing to carry forward: `cargo install` leaves
`cargo-tauri`, the group probed the bare `tauri`, skipped the build, and still printed `4/4 steps passed` —
only the upload step's `if-no-files-found: error` told the truth. Its successor makes a `SKIP` impossible to
count as a pass and fails any skip under `CI`.

**What is next, in order:** head `1055242` went **10 of 10** on Run `37138881977` and attached three
§42-named artifact sets, so packaging is now proved on all three platforms in the `CI_BUILD_ONLY` sense the
owner allowed — and the darwin set was what found the item that was still open: its `SHA256SUMS.txt` gave the
`.app` one line naming a directory, so `sha256sum -c`, the tool §41 names, answered `FAILED open or read` on a
correct build. A bundle is indexed file by file from `53578e9`, and **that commit's own run was read back the
same way** — `37143046338`, **10 of 10**, five darwin index lines every one `OK` with exit 0, one flipped byte
in a copy of `Contents/Info.plist` turning the same index red and naming the file. §9's index row is closed,
and the read-back's comparison of the two runs is what §5e now states as a measurement: the macOS `.app` tree
digest repeated exactly between them while its `.dmg`, at identical length, did not. A green tick on the job
still was not the proof — the job verifies the bundle before it writes the index, which is why the artifact
set has to be downloaded. **The real Windows install has since run on this host** — §38 A–L and §39 with the
owner's store parked, hashed and restored byte-identically, transcribed in
`P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md` — and it produced three things worth carrying: the window title
still said `FirmwareSight - Analyze` while the page is Compare (L21, observed on the packaged build, and
**closed by Commit C** below); a reopened app shows "Nothing has been analyzed in this session yet" while its
store keeps the whole build (1 build, 19 sections, 42 symbols, 10 evidence rows), which was the concrete case
for History; and the uninstaller asks nothing about user data and deletes none, which is a documentation
obligation for Commit F, not a bug to fix.

**Commit C has since landed** (§13 onboarding, §14 Help/About, §15–§18 local History):
`P5_VALIDATION/P5_ONBOARDING_HISTORY_REPORT.md` and its design checklist are the record, and they answer the
two findings above — History is a fourth rail page over three new bounded storage **read** APIs with **no new
migration**, and the title is now Rust's, composed from a closed `MainWindowPage` enum, with
`the_title_fix_took_no_new_capability` proving `capabilities/main.json` still holds exactly
`["core:default"]`. Locally: `check.py` **16 of 16**, **812 Rust / 200 UI in 8 files**, seven mutation proofs.
L23's sixth instance has since closed in the same test-only way — `compare.test.tsx` read the ranking's second
IPC wave synchronously, and the *local* full gate lost it before CI did — with the evidence, three mutation
proofs and a bounded sweep of chained waves in `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` §0a.
**Commit D has since landed** (§§4-27 of its continuation prompt): storage owns its health
(`integrity_check()`, read-only, repairs nothing), any older file-backed store is snapshotted with SQLite's
online backup API **before** it is migrated and a snapshot that cannot be written stops the upgrade, and
Diagnostics is a closed 31-field allowlist on the Help page exported through a Rust-side native dialog.
`P5_VALIDATION/P5_DIAGNOSTICS_RECOVERY_REPORT.md` and its design checklist are the record, and §27's focused
walk ran against the packaged installer: the exported file contains no `/` and no `\` character at all, the
store is named by file and never by folder, and the owner's store was parked, copied to a second volume and
restored byte-identically at both cycles (`ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`).
Locally: `check.py` **16 of 16**, package group **4 of 4**, **854 Rust / 210 UI in 8 files**, seven mutation
proofs. Two of this commit's fixes exist **because the installed walk found them** and no unit test could
have: the startup refusal exited 101 through a Tauri `panic!` rather than the typed exit 1 (a `setup` error
is panicked by the framework inside its event loop, `tauri-2.12.0/src/app.rs:1443-1445`), and a file that is
not a database was being described as a rolled-back migration to "schema version 0". Read
`P5_COMMIT_D_DESIGN.md` §9 before touching that boundary again.
The remaining workstream order is unchanged: Commit E's fixture cohort, then Commit F's documentation and
closure. **What Commits C and D do not close is the acceptance of Commit C**: the
onboarding panel and the History page have never been operated in the installed binary, so the full §64
journey (install → onboarding → Analyze → Compare → Gate → Bundle → History → Diagnostics → close → reopen →
reinstall → uninstall → reinstall → documented data behaviour) still runs once, on a
disposable firmware project, and only then does `P5_INSTALL_RECOVERY_REPORT.md` leave `IN_PROGRESS`. The
remote has seen this head: Run `37154946484` on `e863d0c` came back **10 of 10** on the first attempt, which
is the repository's rule for every P5 head, and `P5_VALIDATION/P5_CI_AUTHORITY.md` carries the row. **The
current head honours that rule too**: `111fe32` — §32's L22 draft with its identity premise test, plus the
test-only repair of L23's sixth race instance — went **10 of 10** on Run `37158606478`, first attempt. The §32
STOP has been **answered**: `ADR-0028` is Accepted and decides that release identity uses the exact bytes
observed on disk, so the line-ending finding stays exactly as it behaves — do not arrive here and "fix" it by
normalizing a digest, and do not edit this repository's `.gitattributes`. **Do not start a
workstream the prompt has not authorized and do not close P5 from this file**: §61's split is the scope, and
§5's list of forbidden verdicts still applies to every sentence you write.

Four wrong turns to refuse. If you were sent here to "continue P0", P0 is closed. If you were sent here to
continue P1, P2 or P3, all three are `PASS / COMPLETE` and their acceptance lists are checked item by item
in `P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md`, `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md` and
`P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md` — there is nothing left inside any of them to do. And if you were
sent here to start a **Bundle** without a prompt: that window opened on 2026-09-30 and closed on
2026-10-01. P4 ran under *FirmwareSight — P4 Release Bundle MVP Implementation, Execution Prompt v1.0 —
Architect Reviewed*, reached `PASS / COMPLETE`, and returned `active_task` to `NONE`. ADR-0026 removed
the *research* gate, not the requirement that each stage carry its own architect prompt, so "governance
got easier" is not a licence to widen scope: **nothing was authorized by that closure** — a statement that
stood until P5 arrived with a prompt of its own, which is the only way a stage opens here. P4's own §78 said stop after
P4, and its §5 said the round may not claim `G2 = PASS` — the only G2 statement it wrote is
`READY_FOR_ENGINEERING_GATE_REVIEW`. The G2 audit's own round, under its own prompt, then closed
`G2 = PASS` (above). Installer-shaped and History-shaped have since become P5's own authorized scope (§8 and
§§15–18 of its prompt), and Commit C has built the History page; signing-shaped, updater-shaped and anything
cloud-shaped are still outside the MVP, and V1 stays unstarted until the architect opens it. The fourth is
specific to this round's state: P3's commits were **local at closure and pushed after it**, so the pack's
gate numbers stay locally measured and the remote is a separate, later fact — Run `36774472141` on
`219178af`, 6 of 7 jobs green, the one red being `Dependency policy` over a crate that crates.io yanked
the same afternoon. Do not read that red as a build failure and do not read the green six as if they had
produced the verdict.

A fourth thing to know about any P2 number: **the round measured itself `LOCAL PASS`, and the remote only
agreed afterwards.** The mid-round push `c7fc2a3` produced Run #17 `36596452341`, `failure` on
`committed_fixtures_match_their_recorded_hashes` - `.gitignore`'s `**/target/` had been hiding the whole
`fixtures/elf/p2-diff/target/` half of the fixture pair, so a clean checkout could not load it while every
local run passed on files left on disk. That is defect E. `cfee1e5` commits those six files, un-ignores
that one path, and adds `drift/fixtures tracked` to the gate; the fix was verified from a `git archive`
checkout, not from the authoring directory, and then confirmed on the remote: **Run #18 `36648718199` on
`4a77ea1` is `success`, 7 of 7 jobs.** Keep the two claims straight anyway - the gate results were
measured before any push, so the round's verdict is `LOCAL PASS` and #18 is the successor verification of
the tree that carries it. Full story: `P2_COMPARE_EXECUTION_REPORT.md` §5.1.

A fifth number to know, because it is this round's start fact and it is not a P2 measurement: after the
owner pushed the P2 closure record, **Run #19 `36665007523` on `32b23aa` concluded `success`, 7 of 7
jobs** (`gh run view 36665007523 --repo 2023violet/FirmwareSight`). It verifies a documentation and
integrity successor whose delta over `4a77ea1` touches no production Rust, TypeScript, fixture, schema,
migration or product behaviour - which is why the architect could seal P2 on `4a77ea1` and still name
`32b23aa` as the HEAD P3 starts from.

## Baseline and HEADs

```text
Baseline                       v0.6.0 — FirmwareSight_Project_Baseline_v0.6.0 (P0 Technical Foundation Baseline)
Engineering-validated HEAD     1cd6309a09313a0a900a83cb12e054e1a3d7c5e3 — the tree the gate measured
Architect-reviewed HEAD        5e58f778aad35f33188b96d0b8873401a31ccc3c — governance/audit/evidence docs only
Promotion commit               738ae78e00e682a5f82f679ea167304a558af864 — on origin/main; 23 files, no production source
Consistency commit             7d2f38a046ea7e036f95adf7ea8c5d0c9be5c9ae — on origin/main; BASELINE.yaml duplicate-key and stale-field closure
Remote CI Run #3               36399805005  on 1cd6309  success, 7 of 7 jobs — engineering closure
Remote CI Run #4               36402637251  on 5e58f77  success, 7 of 7 jobs — reviewed-HEAD revalidation
Remote CI Run #5               36416146281  on 738ae78  success, 7 of 7 jobs — promotion-commit revalidation
Remote CI Run #6               36419864513  on 7d2f38a  success, 7 of 7 jobs — last remote fact of the P0 chain
Design-token commit            ecd1c878a6efebd93338ee7777a600b49a9baf84 — on origin/main; tokens v0.2.1, generator, tokens.css, App.module.css, BASELINE.yaml
Design tokens                  v0.2.1 (border.width.hairline added; P0's documented 1px gap closed)
P1-A0 governance commit        311f9fce62c4dc207f9daab2c9843bdf5bb33592 — on origin/main; ADR-0025 scope recorded in the .ai/ files
P1-A0 implementation commit    29601735a2eff2c9e3677ef1dfa88cf752484694 — on origin/main; real artifact intake and the Analyze summary
Remote CI Run #7               36431884747  on ecd1c87  success, 7 of 7 jobs — design-token commit
Remote CI Run #8               36439949352  on 311f9fc  success, 7 of 7 jobs — governance commit
Remote CI Run #9               36499759371  on 2960173  success, 7 of 7 jobs — the intake commit, and this round's §1 gate fact
Correctness-closure commit     ace6fbe12fe90d2886d8128aeda3879c52ad539c — on origin/main; MAP companion identity, basis-aware provenance, deterministic primary-artifact query, from_map removal
Remote CI Run #10              36515470263  on ace6fbe  success, 7 of 7 jobs — the closure commit, recorded here after the owner pushed rather than inside its own commit (§23's anti-recursion rule)
Governance successor           3b59585e3843336b8237449e72f147ed35cf672d — on origin/main; documentation and integrity only, no product source
Remote CI Run #11              36516209283  on 3b59585  success, 7 of 7 jobs - the HEAD the V0 Batch A activation round started from, verified before any file was written
V0 Batch A activation          2026-09-29 by architect prompt; `active_task` moved from P1A0_REAL_ARTIFACT_INTAKE to V0_BATCH_A_EXTERNAL_VALIDATION. Research execution: no product code writable, and zero participant evidence existed so zero was written
Batch A activation commit      be09c65f451dc1b625c6bf597929a7ce93f5c509 — on origin/main; Run #12 36520562718 success, 7 of 7 jobs — the verified starting point of the P1 round
ADR-0026 governance reset      2026-09-29, same day: V0 becomes NON_BLOCKING_USER_FEEDBACK_TRACK, G1 is re-based on P0 PASS, `active_task` moves again to P1_ANALYZE_DETAILS. The Batch A activation above is real history and stands as a record; only its sequencing authority was superseded hours later
Withdrawn prompt               FirmwareSight_V0_Batch_A_Price_Anchor_Authorization_Participant_Acquisition_Pack_EXECUTION_PROMPT_v1.0 — WITHDRAWN_BY_ARCHITECT, never executed, no price anchors written, no recruitment pack produced, V0 sample still 0/8
P1 Analyze round               2026-09-29, started from 872ad7e (child of be09c65): bounded query layer in firmwaresight-storage, three use-case IPC commands, the detail UI and Evidence Inspector, the US-001 unit switch, and P1_VALIDATION/
P1 storage + shell commit      f649afd (f649afddd94d1144d0f23e7a35190f52b60b3808) — on origin/main; 23 files: query.rs, details.rs, the two new Rust suites, 15 files under ipc/generated
P1 UI commit                   ae7759a (ae7759a22a19d0d22a2c445a7fac484712a219d1) — on origin/main; 10 files: Details.tsx, Details.module.css and details.test.tsx new, plus App.tsx, format.ts, format.test.ts, intake.test.tsx, bridge.ts, bridge.test.ts and types.ts touched
P1 evidence + governance       e63afaf (e63afaf80cd0cd230a544ef61254a5598145bb3f) — on origin/main; 16 files: P1_VALIDATION/ ×4, the .ai/ pack ×4, README.md, INDEX.md, BASELINE.yaml, two delivery docs, the prompt registry, DIRECTORY_TREE.txt and SHA256SUMS
Remote CI Run #13              36556735551  on e63afaf  success, 7 of 7 jobs — one run for the whole four-commit push, so 872ad7e, f649afd and ae7759a have no run of their own and none is claimed for them
Remote CI Run #14              36558893544  on 1e5880a  success, 7 of 7 jobs — the first successor record of that push, pushed on its own so it has its own run
Remote CI Run #15              36559834259  on 653e313  success, 7 of 7 jobs — the handoff-row alignment successor
Remote CI Run #16              36576568426  on 7a13660  success, 7 of 7 jobs — the P1-round index and stale-claim dating commit, and the verified start HEAD of P2 Compare
P2 Compare start               7a13660db873439f66eedee850561e8dae1cb3cf — HEAD = origin/main, worktree clean, 184 Rust and 58 UI tests re-measured green before the first write
P2 pushed mid-round            0ff77f3 + c7fc2a3 — the first push of the round, and the commit the remote stayed on while the round finished
Remote CI Run #17              36596452341  on c7fc2a3  **failure**, 3 of 7 jobs (Rust windows, Rust ubuntu, macOS Core Smoke) — committed_fixtures_match_their_recorded_hashes could not load the target half of the P2 pair; defect E, caused by `**/target/` in .gitignore
P2 fixture + gate fix          cfee1e5 — commits fixtures/elf/p2-diff/target/ (6 files), un-ignores that path, adds `drift/fixtures tracked`; verified from a clean `git archive` checkout
P2 closing commits             78fd40e (three window defects) · c4f4124 (layout label + two goldens) · cfee1e5 (fixtures + gate) · 14c4f75 (validation + governance) · 4a77ea1 (integrity) — 14 commits from 7a13660
Remote CI Run #18              36648718199  on 4a77ea1  **success**, 7 of 7 jobs — the head the owner pushed after the round closed; defect E closed on the remote too
P2 closure record              32b23aa78323d315f6643f04c2343f75576d483f — on origin/main; 12 files, P2_VALIDATION/ + .ai/ + BASELINE.yaml + README/INDEX/prompt registry + SHA256SUMS. `git diff --name-only 4a77ea1 32b23aa` shows no production source, fixture, schema, migration or configuration file
Remote CI Run #19              36665007523  on 32b23aa  **success**, 7 of 7 jobs — the successor verification, and P3's verified start HEAD
P3 Release Gate start          32b23aa78323d315f6643f04c2343f75576d483f — HEAD = origin/main, worktree clean, 345 Rust and 99 UI tests re-measured green before the first write; authorized by prompt v1.1 + ADR-0027
P4 final implementation        e799f2f — Run 36872456446 success, 7 of 7; closure heads 157f749 (Run 36879477561, green on attempt 2 after a runner rustup conflict) and 94f7bd9 (Run 36881101091)
Pre-G2 closures                fe420d1 BASELINE alignment (Run 36892307828, attempt 2 7 of 7) · 055b54e Compare test race (Run 36899128645, attempt 1 7 of 7)
G2 product tree                e35cfe70aa0e8f273a75ac14b9dc62377483af36 — 055b54e + the G2-F1 test-only fix; Run 36906482900 attempt 1, 7 of 7
G2 evidence head               f75cbc5f40fa823d17463855102dfaeca3fb8c43 — G2_VALIDATION/, prompt archives, governance; Run 36948719972 attempt 1, 7 of 7
G2 closure                     the successor of f75cbc5 sets G2 PASS; its own run is external evidence, not recorded in a further commit
```

Run #1 (`36360310447`, `f9b8ccb`) and Run #2 (`36378384225`, `ebda52d`) concluded `failure` and stay
published as history in `P0_CI_REPORT.md`. Read any run with
`gh run view <id> --repo 2023violet/FirmwareSight`, not from this repository's reports.

## Gate state

```text
G0      PASS
P0      PASS — frozen at v0.6.0
G1      PASS — basis: P0 PASS, per ADR-0026 (2026-09-29). Before that date G1 was `V0 PASS + P0 PASS`
        and NOT CLAIMED; P0's promotion pack and the P1-A0 pack record the older formula, and they are
        not rewritten
V0      NON_BLOCKING_USER_FEEDBACK_TRACK — 0 / 8 eligible sessions, an honest zero that gates no
        P-stage. The v0.1.0 prototype and protocol stay frozen so a later feedback round is comparable
Pre-G1  P1-A0 REAL ARTIFACT INTAKE — AUTHORIZED (ADR-0025), ON origin/main, GREEN ON RUN #9.
        ITS EVIDENCE-IDENTITY / PERSISTENCE CORRECTNESS CLOSURE IS IMPLEMENTED, TESTED, AND ALSO ON
        origin/main (`ace6fbe`), GREEN ON RUN #10
P1      PASS / COMPLETE — P1_ANALYZE_DETAILS closed 2026-09-29. PASS was earned item by item against the
        frozen US-001 acceptance list, in P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md, and 14/14 local
        gate steps plus a real shipped-binary window run. Remote CI for those commits is a successor
        document's job; it was NOT RUN when this line was written
P2      PASS / COMPLETE — P2_COMPARE closed 2026-09-29 item by item against the frozen US-002 acceptance
        list, in P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md: 15/15 local gate steps, 345 Rust and 99 UI
        tests, a real shipped-binary window run, and a clean-`git archive` check of the fixture pair. The
        gate results are LOCAL PASS as measured; the pushed head 4a77ea1 was then verified green by remote
        Run #18 36648718199, 7 of 7 jobs, which closes defect E on the remote as well. One sub-item stays
        open and is stated, not smoothed: desktop smoke step 27 is NOT VERIFIED in the shipped window
P3      PASS / COMPLETE — P3_RELEASE_GATE closed 2026-09-30, authorized 2026-09-29 by its own architect
        prompt (v1.1) and by ADR-0027. Forty §69 boxes settled item by item against US-003 and PRD P0-5 in
        P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md: 556 Rust tests in 28 executable suites, 135 UI tests,
        15/15 local gate steps, an 18/18 CLI Gate smoke and all forty §61 desktop steps walked on the
        shipping binary. The stage closed as LOCAL PASS because the commits were still unpushed; they were
        then pushed and produced three runs. 36774472141 (219178af) FAILURE, 6 of 7 — only Dependency
        policy, on `error[yanked]` for `yoke-derive 0.8.3`, a crate yanked on crates.io the same afternoon
        after the local deny step had already gone green on a stale index; fixed by `cargo update -p
        yoke-derive --precise 0.8.4`. 36779321479 (2d1bcea) SUCCESS, 7 of 7, which is that fix verified
        remotely. 36779715108 (893a635) FAILURE, 6 of 7 — Desktop UI (windows-latest) lost a call-count
        race in compare.test.tsx that the ubuntu job won on the same commit (defect J), fixed with the
        delta assertions unchanged and the fix proved by mutation. Four sub-items are stated rather than
        smoothed: desktop step 30's prior-run re-read was never observed through the window, this round's
        first full gate came back 13/14 on a P2 test-side race (defect I) that the round diagnosed and
        fixed before closing, the closure report's own Rust test total was a double count that is
        corrected in §4 of that report rather than replaced silently, and two races in that one file were
        each caught by a different mechanism — I locally, J only on Windows CI — and J's fix verified by
        Run 36783457030 on 02e8a81 at 7 of 7 jobs green
P4      PASS / COMPLETE, closed 2026-10-01 under its own architect prompt; `active_task` returned to
        NONE. Evidence: P4_VALIDATION/
G2      PASS (2026-10-01) — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE. Audit-first under its
        own prompt + storage-path addendum; one test race found and fixed test-only (G2-F1, e35cfe7);
        artifacts.path adjudicated expected local-only storage (G2-F2); evidence G2_VALIDATION/. Not
        productization, beta, RC or GA; not licensed open source until the owner chooses a licence —
        root Cargo.toml still reads license = "Proprietary", no root LICENSE,
        OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION
Pricing / commercial research   DEFERRED_POST_MVP; the price-anchor prompt was withdrawn unexecuted
```

## What the code does and how it is proven

- 556 Rust tests in 28 executable suites (plus 6 empty doc-test suites) and 135 UI tests in 6 files on
  one gate: `python scripts/check.py`, which
  CI calls unchanged — 15 steps on a tree that already has the built frontend, 17 when it builds that too,
  plus 3 under `--only core-smoke`. The v0.6.0 promotion measured 104 / 19; P1-A0 and its correctness
  closure took that to 142 / 31, P1 Analyze added 42 Rust and 27 UI, P2 Compare added 161 Rust and 41 UI,
  and P3 Release Gate took `cargo test --workspace` from 345 to **556** — 211 Rust and 36 UI. Both of
  those endpoint figures are single `cargo test --workspace` invocations. P3's own first total was not: a
  revision of this document said 715 in 42 suites, which is 556 plus the 159 desktop tests that the
  `drift` group re-runs inside the same gate log. The pre-P3 figures in the chain come from their own
  validation packs and were not re-derived here. Per-suite, counted once: Core's diff module 36 and its
  `domain::gate` module 32,
  the P2 fixture pair 12, storage candidate queries 14, storage Gate history 22, the project crate 68 plus
  its 8 config-schema contract tests, the portable-schema contracts 19 plus 13 for Gate output, CLI diff
  15 and its golden 11, CLI Gate 18, desktop Compare IPC 24 and desktop Release Gate 29; `compare.test.tsx`
  37 and `release.test.tsx` 36 of the UI tests.
- Four Phase-0 library crates plus CLI and desktop apps. Core is headless and synchronous; no Tauri,
  rusqlite, Tokio or `object::*` type crosses out of it.
- Real ARM ELF/MAP fixtures with recorded provenance and hash-first tests, so no test needs
  `arm-none-eabi-gcc`.
- Deterministic CLI JSON, memory accounting reproduced by hand from `readelf`, a typed ts-rs IPC
  boundary whose drift is checked by regeneration plus `git diff --exit-code`.
- SQLite via `rusqlite + bundled`, migrations, transactional import, **schema version 4** — migration
  `0002` rebuilds `evidence` on `(build_id, id)` because version 1 contradicted `04_TECH/15` §4,
  `0003_gate_history.sql` adds `gate_runs`, `gate_findings`, `gate_finding_evidence` and
  `accepted_reviews` additively, and `0004_release_records.sql` adds the immutable
  `release_records` index additively, with the v1→v4, v2→v4 and v3→v4 upgrade paths tested rather than a
  constant edited to make a test pass.
- A 512 MiB input guard measured from both sides of the boundary; Core/CLI/Desktop parity on the same
  bytes; the desktop window opened and driven for real on the shipping configuration.
- P1-A0 adds the intake path: a native dialog opens Rust-side, the selection is held behind an opaque
  session-local id, and the chosen ELF - optionally with a GNU ld MAP - runs through the same validated
  Analyze summary. **Stronger evidence is a second build, never an update to the first:** the MAP is
  sealed as a companion `ArtifactKind::Map` artifact, which changes `SnapshotId` through the identity
  model's existing optional map component, and each memory evidence item names the source type and
  locator its own rule actually used.
- P1 Analyze adds the inspect path: `query_sections`, `query_symbols` and `query_evidence` over the
  snapshot SQLite already stores, each taking a typed request and returning one bounded page
  (default 100, hard max 500, clamped in Rust) with its total and next offset. Filter, sort, class and
  offset are executed in SQLite - the WebView never receives a whole table and never sorts one - and the
  bytes/KiB switch is a presentation division that issues no command, changes no address and creates no
  snapshot. The Evidence Inspector shows a fact's full source type, locator, rule and evidence id.
- P2 Compare adds the diff: `firmwaresight-core`'s `domain::diff` pairs sections by name and symbols by
  name + kind + binding, refuses to pair a duplicate by row position, keeps `Added` and `Removed` as
  absence rather than `0 → N`, makes a delta `Unknown` when either side is, and inverts cleanly on a
  reversed pair. Six use-case commands carry it across IPC behind an opaque `cmp-<pid>-<n>` session
  handle, each detail query page-bound exactly like P1's, and the whole diff never crosses by default.
  `fwsight diff old new` and the desktop's Export JSON write the *same* portable document
  (`urn:firmwaresight:schema:diff:1`), byte-for-byte, with no host path and no timestamp in it; the HTML
  export is self-contained and byte-identical to its golden. Diff truth is computed in Core — never in
  SQL — and rehydrating a stored snapshot needs the files to be gone, which is the point.

## Known gaps that promotion did not remove

```text
Peak RSS          NOT MEASURED — reason in P0_PERFORMANCE_REPORT.md; the P0 prompt accepted the gap
Fuzzing           NOT RUN
RustSec           RUSTSEC-2024-0429 (glib 0.18.5, unsound), RUSTSEC-2024-0370
                  (proc-macro-error 1.0.4, unmaintained) — ACCEPTED AS EXPLICIT P0 TRANSITIVE RISK,
                  five revisit triggers, no ADR because no architecture choice changed
Desktop smoke     one Windows 10 host, one WebView, at 100% scaling
Linker layouts    GNU ld MAP plus one dual-region ELF layout
Stored-reason gap sections.file_offset and symbols.address are nullable with NO *_unknown column, so a
                  reason is lost at write time. P1 reports it instead of migrating: SCHEMA_VERSION stays
                  2. P1_VALIDATION/P1_ANALYZE_EXECUTION_REPORT.md §6.1
Unresolved UI     one Apply-filter click did nothing on an intermediate P1 build and could not be
                  reproduced on the shipped binary or in the automated reproducer. Reported as
                  UNRESOLVED, not as fixed: P1_VALIDATION/P1_ANALYZE_DETAILS_SMOKE_REPORT.md §6
Gate blind spot   `drift/ipc bindings unchanged` is `git diff --exit-code`, so it cannot see a binding
                  file that is not yet tracked. For P1's 14 new bindings the invariant was checked by
                  hashing the directory before and after regeneration instead
P2 smoke gap      desktop step 27, the same-pair lock, is NOT VERIFIED in the shipped window: a native
                  <select> popup cannot be driven or captured through the available window path. Covered
                  by compare.test.tsx and the IPC tests, recorded PARTIAL: P2_VALIDATION/
                  P2_COMPARE_DESKTOP_SMOKE_REPORT.md
Remote state      Green at the head. Five pushes, five runs: origin/main moved 219178af -> 2d1bcea ->
                  893a635 -> 02e8a81 -> f66a93d. Run 36774472141 (219178af) FAILURE, 6 of 7: only
                  Dependency policy, on `error[yanked]` for yoke-derive 0.8.3, yanked on crates.io at
                  2026-09-30T13:19:39Z after the local deny step had passed on a stale index. Run
                  36779321479 (2d1bcea) SUCCESS, 7 of 7 - the lock moved to 0.8.4 and CI agreed on a fresh
                  index. Run 36779715108 (893a635) FAILURE, 6 of 7: Desktop UI (windows-latest) lost a
                  call-count race in compare.test.tsx that Desktop UI (ubuntu-latest) won on the same
                  commit, and eight local runs had won; fixed as defect J with the delta assertions
                  unchanged and the fix proved by mutation. Run 36783457030 (02e8a81) SUCCESS, 7 of 7 -
                  defect J verified, both Desktop UI jobs included. Run 36784382005 (f66a93d, which moved
                  documentation and SHA256SUMS only) SUCCESS, 7 of 7, and is the last head whose run this
                  pack names: a documentation-only successor's own result is read with gh run list rather
                  than chased into itself. The previous head was
                  32b23aa on Run #19 36665007523 (success, 7 of 7), the documentation-and-integrity
                  successor to 4a77ea1, which P2 closed green on Run #18 36648718199. The earlier Run #17 on
                  c7fc2a3 was FAILURE because `**/target/` hid the target half of the P2 fixture pair
                  (defect E); cfee1e5 fixed it and the head pushes confirmed it. P1/P2/P3's own gate numbers
                  stay locally measured - the runs verify the trees, they did not produce the verdicts
Attribution       object / module attribution is Unavailable by evidence, not by omission: ELF symbol
                  values attribute nothing back to a source object, so P2 reports the gap with its reason
                  and ships no Object tab (PRD P0-3 makes it conditional on sufficient evidence)
Index coverage    `drift/fixtures tracked` now compares fixtures/manifest.json against `git ls-files`,
                  which is the gate's only index-reading step. A source file ignored by accident is still
                  unseen, because no manifest lists it
Open design items capability labels show Core's enum words. The eight 1px borders that used to be
                  listed here are closed: `border.width.hairline` in design tokens v0.2.1
Evidence model  UNTESTED with real users. V0 is now a non-blocking track at 0 / 8, so whether people
  comprehension   distinguish Unknown from PASS, or read a Review correctly, is carried forward as risk
                  rather than measured. Recorded in ADR-0026's Consequences
Known engineering `scripts/update_goldens.py` no longer reproduces the committed goldens' key order;
  items           `ElfProgramHeader` names a source the ELF parser never reads; the frozen release-manifest
                  schema describes one artifact per build; a release build without
                  `--features custom-protocol` shows a WebView network error. Each with its reason in
                  `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` §9.7
CI duplication    two jobs install the same ten apt lines on purpose
```

Never describe this tree as "zero vulnerabilities" or "security clean"; the two advisories above are
accepted, not fixed, and they are reachable only through the gtk-rs `0.18` line Tauri `2.12.0`
requires (`cargo update -p glib --precise 0.20.0` fails against `gtk = "^0.18"`).

## Read first

1. `README.md`
2. `PRODUCT_BASELINE.md` and `BASELINE.yaml`
3. `AGENTS.md`
4. `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`, `.ai/ACTIVE_TASK.md`
5. `10_AUDIT/SOURCE_PROMPTS/README.md` — which prompt authorizes what, and which was withdrawn
6. `09_ADR/ADR-0026-open-source-mvp-first-delivery.md` — why G1 now rests on P0 and what it did NOT relax
7. `09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md` — the P1-A0 slice and its safety terms
8. `04_TECH/14_IPC_DATA_CONTRACTS.md` and `04_TECH/15_STORAGE_DATABASE_BASELINE.md` — the bounded-query
   and schema rules P1 was written against, and the rules any next stage inherits
9. `09_ADR/ADR-0027-project-policy-and-provenance-adapter.md`, `04_TECH/27_GATE_STATE_SEMANTICS.md`,
   `04_TECH/22_GIT_PROVENANCE_ADAPTER.md`, `04_TECH/08_CONFIG_SPEC.md` and
   `09_ADR/ADR-0023-gate-five-state-semantics.md` — the shape of the live P3 stage: which crate owns
   config and Git facts, the five frozen Gate states and their effective severity, the read-only Git
   boundary, and the draft `firmwaresight.toml` P3 freezes
10. `P2_VALIDATION/` — the completed P2 round, whose `urn:firmwaresight:schema:diff:1` is now a public
    compatibility promise that P3's growth rule must reuse rather than recompute
11. `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md` — the promotion record
12. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md` — what is proven, and how
13. `P0_TECHNICAL_VALIDATION/P0_IMPLEMENTATION_LOG.md` — decisions already taken
14. `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` — the P0-chain runs, including the two failures
15. `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md`
16. `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md`, `P1_A0_EXIT_CHECKLIST.md` and
    `P1_A0_CORRECTNESS_SMOKE_REPORT.md` — the completed slice, and how it was proven
17. `P1_VALIDATION/` — the completed P1 round: `P1_ANALYZE_EXECUTION_REPORT.md` (what was built, the
    defects fixed, the findings reported), `P1_ANALYZE_EXIT_CHECKLIST.md` (US-001 item by item and the
    verdict), `P1_ANALYZE_DESIGN_CHECKLIST.md`, `P1_ANALYZE_DETAILS_SMOKE_REPORT.md` (shipped binary)
18. `DESIGN.md` + `assets/design-tokens.json` (any UI work)
19. The V0 track, for a later non-blocking feedback round: `V0_VALIDATION/README.md`, `V0_PLAN.md`,
    `V0_VALIDATION/protocol/**`, `V0_VALIDATION/sessions/TEMPLATE.md`,
    `V0_VALIDATION/batch_a/**` including `recruitment_ready/**`, and
    `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`

## Boundaries still in force

- The completed slice stays closed: `P1-A0` was real artifact intake plus the Analyze summary, and its
  follow-up was a **correctness closure inside that same slice** - the MAP that strengthened evidence in
  memory but not in the persisted identity, and the evidence items that claimed MAP provenance they did
  not earn. Both are fixed and neither is listed as open. A green P1-A0 was not P1 progress, and the
  current P1 task does not retroactively make it so.
- Intake security shape is fixed by ADR-0025 and `AGENTS.md` 7, and ADR-0026 did not relax it: dialogs
  open Rust-side, the selection is held behind an opaque session-local id, no generic
  filesystem/shell/network capability, no `read_file(path)` or `get_any_path` command, no full path in
  normal IPC or UI. The same discipline governs every command added since: P1's three detail queries,
  P2's six Compare commands, and P3's five Gate commands (`open_project_config`, `save_project_policy`,
  `run_release_gate`, `accept_review`, `get_gate_run`) are all use-case oriented, with no `run_sql` /
  `read_table` / `query_any` / `get_database`, no `read_project_file` / `write_project_file`, no
  `execute_git`, no raw SQL and no `rusqlite` type across IPC. System Git runs read-only inside
  `firmwaresight-project` with a bounded timeout; the WebView never gets a shell and never gets the
  repository root.
- Schema discipline: P1 and P2 changed nothing (`SCHEMA_VERSION` stayed 2), and **P3 changes it by
  authorization, not by convenience** — additive migration `0003_gate_history.sql`, version 2 → 3, four
  new tables, `foreign_keys ON` kept, v1→v3 and v2→v3 upgrade paths tested with rows preserved, a future
  version still a hard fail. No destructive migration, no Bundle table, no absolute path in any column.
  Raw artifact bytes still stay out of SQLite and `NORMALIZATION_VERSION` stays `p0-normalize-1`. A
  measured need for an index outside what the prompt names still stops a round and comes back as a
  proposal.
- Payload discipline is now part of the contract: default page size 100, hard maximum 500, enforced in
  Rust, with `rows` / total / next offset returned. The whole symbol table never crosses IPC.
- Details follow the last-good snapshot. A failed later attempt keeps the summary and the details pointed
  at the snapshot actually shown, and a detail-query error belongs to the details area only.
- Presentation-only unit switch: bytes and KiB at 1024 bytes per KiB, applied to sizes and to nothing
  else - addresses, offsets, hashes, counts, ordinals and snapshot ids never convert, and the choice
  lives in UI state, never in SQLite, project policy or a settings page.
- Design: existing tokens only, `border.width.hairline` included. No new visual semantic, no shadow on
  tables or panels, no gradient, no glass, no purple, no colour-only state, no dark theme. If a value
  genuinely has no token, stop and report rather than editing a frozen asset.
- `ADR-0026` relaxed **sequencing**, nothing else. It removed the V0 precondition and re-based G1 on P0;
  it did not authorize cloud, accounts, telemetry, AI, updater, licensing work, or a stage beyond the one
  a prompt names. Each stage still needs its own architect prompt.
- Four Phase-0 library crates were the P0 baseline; **`ADR-0027` adds the fifth**,
  `crates/firmwaresight-project`, and the architecture review that boundary requires is that ADR. A sixth
  still needs architecture review plus an ADR. The new crate may not depend on Tauri, SQLite or Tokio,
  may not implement Gate rule semantics, may not parse firmware artifacts and may not make network calls;
  `firmwaresight-core` stays the only owner of Gate state, severity and rule identity, and remains
  headless, synchronous and dependency-free.
- AGENTS.md §2: no silent baseline change — async-first Core, framework swaps, SQLx, HTTP client,
  wgpu, cloud/auth/telemetry, evidence classes, schema semantics, updater/signing model all need an
  ADR first.
- The workflow list above is the P3-era boundary, and four stages have moved it since: P2 shipped Compare,
  P3 shipped the Gate, P4 shipped the **Release Bundle**, and P5's Commit C shipped **local History** plus
  Help. Packaging is now enabled and an installer exists on this host and on three runners (§8, §41), which
  P3 could only have called out of scope. What has *not* moved is the harder list, and it still binds:
  signing and notarization stay `READY_NOT_EXECUTED`, the updater stays `UPDATE_READY_MANUAL` — no
  updater, no endpoint, no certificate, no committed private key — and there is still no tag, no GitHub
  Release, no published installer and no cloud, telemetry or account surface. v0.6.0 is a baseline
  promotion, not a stable release, and this repository has no tag convention; none has been invented.
- `V0_VALIDATION/**` is research evidence and stays where it is: not deleted by the reset, not edited to
  imply a session that did not happen, and not a gate any more. Its `0 / 8` is an honest zero.

## Continuation rules

- If the V0 feedback track is ever resumed, it resumes under its own rules, not these: one real session at
  a time, processing exactly the evidence supplied; `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_EVIDENCE_INTEGRITY_CHECKLIST.md`
  must pass in full before any `PA-00X.md` counts, and a session failing a mandatory item is recorded
  `DISCOVERY_ONLY` or `EXCLUDED_FROM_FORMAL_N` with its reason rather than counted silently. No synthetic
  participant, no fabricated quote, outcome, timing or count - that rule does not expire with the gate.
- Engineering rounds resume by finishing the authorized task and stopping; they do not continue into the
  next stage because it is now sequenced more loosely. `P1` ending did not authorize `P2`, `P2` ending did
  not authorize `P3` — each stage needed its own architect prompt, and it got one — and `P3` ending will
  not authorize `P4`.

- Re-run the read-only Git preflight before writing, and report start HEADs again; do not trust the
  HEADs recorded in this file.
- Never clean, reset, stash, restore or delete to get a tidy tree. User work outranks cleanliness.
- Reports cite real commands and real output. Unmeasured stays `NOT MEASURED`; unrun stays `NOT RUN`.
- Do not weaken or delete a test to reach green, and do not edit a baseline document to make a
  historical failure read as a pass.
- Do not globally replace `version: 0.5.1` in historical evidence; only current baseline authority
  documents carry v0.6.0.
- Quote any `BASELINE.yaml` value containing ` #`: an unquoted `Run #2` truncates at the comment and
  the file still parses. Verify with `python -c "import yaml, …; yaml.safe_load(...)"` after editing.
- Read remote CI with `gh run view`. A run's ID, head SHA and per-job conclusion are external facts,
  and a report that describes them is a claim about a moment that has already moved.
- After any commit that touches baseline-controlled files, `SHA256SUMS` and `DIRECTORY_TREE.txt` are
  stale: regenerate both, in that order, and verify with `sha256sum -c` plus an independent checker.

## The two rounds that ran after G2 (2026-10-02)

G2 is still the gate that stands, and this file's G2 paragraphs are still true about the tree they
measured. What is now stale in them is the set of numbers: **Rust is 770 tests, UI is 159 in 6 files**,
and the newest evidence directory is `POST_G2_E2E_REMEDIATION/`.

Two rounds happened, and the difference between them is the point:

- **Real-desktop MVP acceptance** drove the shipping binary by hand through 283 black-box cases — real
  mouse, real keyboard, native dialogs, no IPC substituted for a click — and closed
  `PASS_WITH_FINDINGS`: three product defects, no S0, no S1. It changed no code and made no commit; its
  evidence root lives under `%TEMP%`, outside the repository, and the owner's live SQLite store was
  restored byte-exact afterwards.
- **Findings remediation** then fixed exactly those three, narrowly, each with a regression written
  first, a mutation proof after, thirty fresh-process repetitions, and a focused real-desktop re-check
  against a rebuilt binary. `e816dcb` carries the two UI fixes; `971015f` carries the MAP detection fix
  as its own commit so a reviewer can read them apart.

What the product now says that it did not before, because it is easy to undo without noticing:

- A preserved analysis names the selection it belongs to, and it keys on the shell's selection handle —
  **not** on the file name. Two artifacts called `firmware.elf` are the case that has to work. The
  identity is one React state value; it is not in the portable schema and not a SQLite column.
- A destination folder that exists and is not a bundle this engine wrote gets no Replace offer and no
  Export press, and the page says why. The engine's guard is untouched and still refuses; do not
  "simplify" the UI back into asking a question the engine cannot honor.
- GNU ld MAP detection searches the whole buffered text. The banner sits behind preambles whose length
  the project does not control, so a fixed head window is not a conservative choice — it is a false
  statement about the linker.

Carried forward, deliberately not fixed, and still open: near-500 MiB first-use latency and peak RSS
(`PARTIAL / environment-sensitive`, `MEASURED FOR TESTED WORKLOAD`), 125/150 % DPI coverage,
mouse-wheel behaviour, the `update_goldens` issue, the E2E harness refactor, and the licence decision.
