---
title: "Project Handoff"
doc_id: "FS-AI-004"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-06"
---

# Handoff — FirmwareSight v0.6.0 / P0 closed PASS / G1 PASS on the P0 basis / P1 Analyze COMPLETE / P2 Compare COMPLETE / P3 Release Gate COMPLETE / P4 Release Bundle COMPLETE / G2 PASS — MVP CANDIDATE / P5 productization PASS_COMPLETE — V1 own-artifact external validation IN_PROGRESS / RECRUITMENT_READY — active_task V1_OWN_ARTIFACT_EXTERNAL_VALIDATION — FirmwareSight Productized MVP Candidate — 0 real external sessions

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

**And that round is now closed: `active_task` is `NONE` again, with P5 `PASS_COMPLETE`.** Stage P5 ran from
2026-10-03 to 2026-10-06 in eleven heads — Commit A (audit, then governance), B (packaging and the three CI
package jobs), C (onboarding, Help/About, History), D (integrity check, pre-migration backup, Diagnostics,
`ADR-0028`), E (compatibility cohort, §48 dispositions, the L15 answer, and its closure normalization), F1
(`ADR-0029` and the index-blob baseline), F2 (the §64 installed journey), F2R1 + F2R2 (the narrow installed-UI
corrective the Architect inserted, and its read-back) and **F3**, which re-audited every exit criterion and
wrote the closure sentence the earlier rounds were forbidden to write. Canonical state now: **P5 =
`PASS_COMPLETE`**, **Productization = `ENGINEERING_COMPLETE`**, product **`MVP_CANDIDATE`**, narrative
**FirmwareSight Productized MVP Candidate**, `baseline_version` **`0.6.0`**, `active_task` **`NONE`**. Verdict
and boundaries: `P5_VALIDATION/P5_FINAL_CLOSURE_REPORT.md`; the head-by-head lineage with every red and
interrupted run kept: `P5_EXECUTION_REPORT.md` §7; the exit re-audit: `P5_EXIT_CHECKLIST.md` §7; what the
remote said, run by run: `P5_CI_AUTHORITY.md`.

**Closing P5 authorizes nothing next, and this is the sentence a newcomer most needs to read honestly.** V1
own-artifact / real-user validation is the likely next decision — it is the track that exists to answer L11,
which P5 carried forward rather than closing — but it needs a **new architect prompt**, and so do B1 / private
beta, RC, GA, commercialization, a licence, signing, notarization, an updater, and any new product verb,
format, adapter or crate. `AGENTS.md` 1 is unchanged: with `active_task` `NONE`, no agent picks the next track
or invents business functionality.

*(the four paragraphs below are P5's opening record, written on 2026-10-03 and kept as the round began. Their
`state IN_PROGRESS`, "P5 is IN_PROGRESS, not passed" and "if you were sent here to continue P5" are the words
that were true then; the present answer is the two paragraphs above.)*

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
Diagnostics is a closed 41-key allowlist on the Help page exported through a Rust-side native dialog.
`P5_VALIDATION/P5_DIAGNOSTICS_RECOVERY_REPORT.md` and its design checklist are the record, and §27's focused
walk ran against the packaged installer: the exported file contains no `/` and no `\` character at all, the
store is named by file and never by folder, and the owner's store was parked, copied to a second volume and
restored byte-identically at both cycles (`ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`).
Locally: `check.py` **16 of 16**, package group **4 of 4**, **854 Rust / 210 UI in 8 files**, eight mutation
proofs. Two of this commit's fixes exist **because the installed walk found them** and no unit test could
have: the startup refusal exited 101 through a Tauri `panic!` rather than the typed exit 1 (a `setup` error
is panicked by the framework inside its event loop, `tauri-2.12.0/src/app.rs:1443-1445`), and a file that is
not a database was being described as a rolled-back migration to "schema version 0". Read
`P5_COMMIT_D_DESIGN.md` §9 before touching that boundary again.
A **third** fix exists because the remote found it: the same head went **8 of 10**, and the two red jobs —
`macOS Core Smoke` and `Rust (ubuntu-latest)` — panicked on one assertion in a test Commit D added, which
compared two snapshot files whose only available difference was a **second-granular** `applied_at` default.
That is a race with a clock, and it is this repository's own L23 shape wearing a storage badge; the repair
names the difference instead of timing it (§11 and §12 of `P5_COMMIT_D_DESIGN.md`, mutation H), so a claim
about which snapshot stands now holds a fact a reader can query rather than a window the runner happens to
leave open.
**One finding came out of writing that record, and it was not fixed then.** `SHA256SUMS` was generated from
working-copy bytes, so with `core.autocrlf=true` and `text eol=lf` its CRLF-in-tree entries verified on the host
that wrote them and disagreed with a clean checkout anywhere else; no CI job ran the verifier, so nothing has
ever disagreed. It was recorded as **L26** in `P5_PRODUCTIZATION_AUDIT.md` §G with the measurement, the
`golden/…/SHA256SUMS` row that has no entry, and the 25 names that method could not compare, and the instruction
then was **do not "fix" it from this handoff** — the choice of which bytes the artifact means belonged to
Commit F, and the file is the repository's own baseline evidence.

**That instruction is discharged: Commit F1 decided it, as ADR-0029.** Root `SHA256SUMS` now holds the SHA-256
of canonical Git **stage-0 index blob** bytes; `scripts/generate_baseline_artifacts.py` refuses to write over an
unstaged semantic change, `scripts/verify_baseline_artifacts.py` re-derives the same content through a different
Git path, and `drift/baseline integrity` runs it on every authoritative CI job. Read
`09_ADR/ADR-0029-repository-baseline-checksums-use-git-index-blobs.md` before touching either tool, and note
the one thing that will surprise you: **`sha256sum -c SHA256SUMS` against a working tree is no longer a valid
check** and reports 13 files on this host as failures that mean "this is a checkout". The measured count moved
19 → 19 → 14 → 14 → 17 → 13 across the heads before the fix, which is why the old prose quoting a single number
was never going to stay true.
**Commit E has since landed**, under a prompt of its own — *FirmwareSight P5 Commit E — Compatibility
Fixtures, Support Matrix & Supportability Closure, v1.0* (SHA-256 `030ca28233b964958d6aea8e59a312c66ceb4d117273cba9e667950b4ba21148`,
53,915 bytes, 2,459 LF lines, archived and registered in `10_AUDIT/SOURCE_PROMPTS/README.md`), which is the
answer Commit D's §34 STOP was waiting for. The compatibility cohort is six new fixture directories and 31 new
tracked files under `fixtures/elf/p5-compat/` (`p5-ram-exec`, `p5-extsram`, `p5-dma-region`, `p5-long-preamble`,
`p5-no-debug` with two ELFs, `p5-clang-arm`), built on this host by `arm-none-eabi-gcc 14.3.1` and
`clang 22.1.8` over GNU `ld 2.44.0.20250616`; `fixtures/manifest.json` moved 25 → 56 entries and re-running
`scripts/gen_p5_compat_fixtures.py --force` reproduces all 13 ELF and MAP bytes identically, with no binary byte
moved by the metadata the records gained. The 14 tests in `p5_compat_fixtures.rs` re-derive each fixture's
expected image and live-RAM totals from `readelf` section headers plus the MAP's own region attributes instead of
trusting the model's arithmetic, and that independent rule found the round's one product defect: clang emits
`.ARM.exidx.text.main` with `sh_type` `0x70000001`, and the parser had been inferring "not allocated" from
"unrecognised kind", so 8 real FLASH bytes entered neither budget while the total still printed `Exact`. Ten of
the eleven committed ELFs already agreed with the rule, which is why a per-fixture test could not see it. The fix
is the narrow one the owner approved — `map_section_kind` reads `SHF_ALLOC` from the section header, which is what
`04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §3 and §4 item 3 already require — proven by five mutations (A, B, C, E
and F). Locally on the final tree: **868 Rust / 217 UI in 8 files**, `check.py` **16 of 16**, `--only package`
**4 of 4** with no `SKIP`, `core-smoke` 3/3, drift 7/7, deny 1/1, and the UI reliability campaign 20 runs × 217
tests with **0 failing**. §48's fourteen dispositions are in `P5_VALIDATION/P5_SUPPORTABILITY_REPORT.md`, the
cohort evidence in `P5_COMPATIBILITY_FIXTURE_REPORT.md`, and `P5_COMPATIBILITY_MATRIX.md` was rewritten from the
tree rather than from prose. The remote agreed on the tip of that pair: Run `37228929762` at head `59d85c3` came
back **10 of 10** on attempt 1 with every job and every step read individually, and `859648e` — the E1 head —
has no run of its own because one push carried both commits and GitHub runs the tip; §7a of the fixture report
carries the clean detached worktree that verifies the same bytes a second way.
**Commit E then needed a documentation-only normalization, and got one** — *P5 Commit E Closure Normalization
v1.0*, archived with both its hashes because the delivered bytes were CRLF and the archive is LF (delivered
`9571df2cc08107ea134ed89acc4a984254803b40e451c57c6e9c3ca480075599`, 28,738 bytes; stored
`62040e3d02fffe0f7682909828e4a2d6a91d81b09a937c0bd0b4765e6c7af887`, 27,366 bytes, 1,373 lines proven identical
one by one). Two things changed and nothing else: the **vocabulary** — the matrix status column now holds only
`SUPPORTED / SUPPORTED_WITH_LIMITS / CI_BUILD_ONLY / NOT_TESTED / UNSUPPORTED` (38 cells validated, 0 outside;
the drifted words' meanings moved into the evidence column, and signing/notarization/updater readiness moved out
of a status column altogether into a `state` column, because `READY_NOT_EXECUTED` was never a compatibility
claim), and the A–H disposition column now holds only `PROVED_BY_EXISTING_FIXTURE / PROVED_BY_NEW_FIXTURE /
SUPPORTED_WITH_LIMITS / NOT_AVAILABLE` with **A, F and H recorded as existing** — and the **L15 answer**. The
Architect chose **Option E: preserve `SourceType::ElfProgramHeader` / the `analysis:1` wire token
`"elf.program-header"`, and document the accurate meaning instead** — "ELF address + flags evidence" — with no
enum rename, no wire rename, no rewritten history, **no migration 0006, no `analysis:2`**, no golden byte moved
and no Bundle change; `P5_COMMIT_E_SCHEMA_DECISION.md` is now `RESOLVED_BY_ARCHITECT` with its pricing sections
untouched, `04_TECH/23` §7 records the legacy identifier next to the precedence rule it names loosely, and L15 is
`CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`, explicitly **not** `CLOSED`. One user-visible residue was recorded
rather than fixed there, because that round may not touch code: the Evidence Inspector still printed the stored
token verbatim (`Details.tsx:667` ← `details.rs:223` ← `query.rs:503` ← `db.rs:507`), so a person expanding such
a row read `ElfProgramHeader`; a display-only caption in L20's closed-table shape was Commit F's call. **Commit
F1 made it**: `Details.tsx` now maps both namespaces of that identifier to `ELF address + flags evidence`, every
other stored value still renders verbatim, and `details.test.tsx` proves both halves while asserting the row
object still carries the identifier it was stored under. The wire debt is unchanged and stays
`CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`; only the presentation residue is `CLOSED`. Full record:
`P5_VALIDATION/P5_COMMIT_E_CLOSURE_NORMALIZATION.md`, and §8 of `P5_VALIDATION/P5_COMMIT_F_DESIGN.md` for what
F1 was allowed to touch.
The remaining workstream order is now: **Commit F's F2 layer** — the exact CI-built F1 Windows artifact, the
owner-store park, the installed old-schema migration proof and the full §64 journey — sequenced behind the
owner's explicit confirmation, then the closure pack, then **F3**, which alone may write `P5 = PASS_COMPLETE`.
**What Commits C, D, E and F1 do not
close is the acceptance of Commit C**: the
onboarding panel and the History page have never been operated in the installed binary, so the full §64
journey (install → onboarding → Analyze → Compare → Gate → Bundle → History → Diagnostics → close → reopen →
reinstall → uninstall → reinstall → documented data behaviour) still runs once, on a
disposable firmware project, and only then does `P5_INSTALL_RECOVERY_REPORT.md` leave `IN_PROGRESS`. The
remote has seen this head: Run `37154946484` on `e863d0c` came back **10 of 10** on the first attempt, which
is the repository's rule for every P5 head, and `P5_VALIDATION/P5_CI_AUTHORITY.md` carries the row. **Commit D's
product head broke that rule, and the row says so rather than waiting for the next green one**: Run
`37200245520` on `3400981` came back **8 of 10** on its first attempt, with the two red jobs both naming one
assertion in a test that head added. **The chain has since been read back and it is clean from the repair
forward**: `bccea88` — the test-only repair — went **10 of 10** on Run `37202016141`, attempt 1, and the record
commit `90aa69d` went **10 of 10** on Run `37202301591`, attempt 1, each job verified individually, including
the two runners that had lost the race. `P5_DIAGNOSTICS_RECOVERY_REPORT.md` §8 is that read-back, and §33's
exit list is checked against it. So **Commit D is COMPLETE and §34's STOP is in force**: return to the
Architect. `111fe32` before it had honoured the rule — **10 of 10** on Run
`37158606478`, first attempt. The §32
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
P5 — where the chain stands now (the row-level authority is P5_VALIDATION/P5_CI_AUTHORITY.md; this is the shape, not a copy of it)
P5 opening head                d83175a — the post-G2 remediation closure head, Run 37101619245, 7 of 7, the tree the §4 audit measured
P5 Commits A–B                 4cc8d93 (audit + §5 governance + prompt archive) · 812b472 (migration 0005) · 9e3b1de (artifacts unified on 0.6.0) · 20b03e3 (test-only race repair) · 0c031cd (packaging + three CI package jobs) · 1055242 (the package group finds `cargo-tauri`) · 53578e9 (the `.app` indexed file by file) · a674774 · 9ab089f · 4719e1f (the first real Windows install and its report)
P5 Commit C                    e863d0c — §13 onboarding, §14 Help/About with the title fixed in Rust, §15–§18 local History, no new migration · e070508 (its read-back)
P5 §32 / L23                   111fe32 — the L22 identity draft with its premise test, plus the repair of L23's sixth race instance · 2cdfced (its read-back)
P5 Commit D                    3400981 — integrity_check, the pre-migration online backup, the re-run matrix, the 41-key Diagnostics allowlist and its export, the typed startup refusal, ADR-0028 · bccea88 (the test-only repair of its clock race, and the last head that changed a line of Rust **until Commit E**, whose E1 head `859648e` edited `crates/firmwaresight-artifact/src/elf.rs` — the row below carries that, and this clause was left undated long enough to mislead) · 90aa69d (the record + L26) · 956e250 (the §32 read-back successor, documentation only) · 57a904e (the documentation closeout; the two "current HEAD" sentences it shipped are corrected by the head after it) · No head in this ledger names itself "current HEAD": the commit that wrote it down would be the one that made it false, so read git rev-parse HEAD
P5 Commit E                     split as §61 requires, in two heads: E1 — the fixture cohort (6 directories, 31 files, manifest 25 → 56), `scripts/gen_p5_compat_fixtures.py`, `p5_compat_fixtures.rs` (14 tests), the `SHF_ALLOC` fix in `crates/firmwaresight-artifact/src/elf.rs`, `P5_COMPATIBILITY_MATRIX.md`, `P5_COMPATIBILITY_FIXTURE_REPORT.md`, and the archived prompt with its registration · E2 — the §48 dispositions (`P5_SUPPORTABILITY_REPORT.md`), the §42 STOP draft (`P5_COMMIT_E_SCHEMA_DECISION.md`), the round's design record (`P5_COMMIT_E_DESIGN.md`, which describes both halves), the five UI files L19/L20/L23 touched (`Analyze.tsx`, `Compare.tsx`, `compare.test.tsx`, `details.test.tsx`, `history.test.tsx`), the workload-size sentence corrected, the governance closeout and the regenerated `DIRECTORY_TREE.txt` + `SHA256SUMS`. · E1 = `859648e`, E2 = `59d85c3`, and the head that writes this line is the §59 read-back successor after them. One `git push` carried both commits, so GitHub started one run — `37228929762` at the tip head `59d85c3`, attempt 1, **10 of 10**, every job and every step read — and `859648e` has no run of its own; that absence is a fact about push granularity, not a missing verification, and it is written in `P5_CI_AUTHORITY.md` beside the row that says so
P5 Commit E, after                    6981625 — the §59 read-back successor (docs only): the candidate's run written into `P5_CI_AUTHORITY.md`, §7a added to the fixture report, and the "last head that changed a line of Rust" sentence dated instead of left to mislead · Run `37230689636`, attempt 1, **10 of 10**, every job and step read · the normalization head after it, `80b47c4` — `P5: normalize Commit E evidence and record the L15 decision`: matrix status vocabulary closed to five values, A–H dispositions closed to four, **L15 answered as Option E**, `04_TECH/23` §7 added, and no path under `crates/`, `apps/`, `scripts/`, `fixtures/`, `schemas/`, `golden/` or `.github/` touched. Its run is now a read-back fact, not a self-claim: Run `37262147348` (#67) at that headSha went **9 of 10 on attempt 1**, the single red being `Generated output drift`, where rustup on the Ubuntu runner logged "recovering from a partially installed toolchain" and then `failed to install component: 'clippy-preview-x86_64-unknown-linux-gnu', detected conflict: 'bin/cargo-clippy'` and rolled back ~0.5 s later without compiling anything — the same provisioning class P4's `157f749` recorded at `BASELINE.yaml:514`. **Attempt 2** re-ran only that failed job (`gh run rerun --failed`; the workflow has no `needs:` graph), drift came back **7/7**, and the run concluded **10 of 10 success**, with the other nine jobs' attempt-1 executions carried into the attempt-2 object rather than run again. Two attempts, stated as two attempts
P5 Commit F1                         ADR-0029 and the L15 caption, landing 2026-10-05: root `SHA256SUMS` now means canonical Git stage-0 index blob bytes, the generator refuses to certify unstaged semantic changes, the verifier re-derives the same content through a different Git path, `drift/baseline integrity` joins the authoritative gate (drift 7 → 8, full gate 16 → 17), and the Evidence Inspector captions the legacy identifier instead of printing it (217 → 219 UI). **Run `37293381181`, attempt 1, `event: push`, headSha `0bca373ed46082f98a9f474908d09f836174ff1b`, 10 of 10 success**, every job read individually — and the row that makes F2 possible: this is the run whose Windows artifact F2 downloaded and installed, **artifact id `11337963032`**, 3,885,631 bytes, SHA-256 `a1152ef31a2c28b0fa522d88b7fe08eee543af1b86a161a4ab60acf04b3c076b`
P5 Commit F2                         Installed productization acceptance, landing 2026-10-05, and **documentation, evidence and governance only** — §3/§35 forbid any path under `crates/`, `apps/`, `scripts/`, `fixtures/`, `schemas/`, `migrations/`, `.github/`, `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json` or `deny.toml`, because one of them invalidates the F1 installer as the candidate under test. Three product findings were recorded and **not patched**. What it ran: the whole §64 journey once, contiguously, on the exact CI-built artifact above, driven by real `SendInput` with read-only SQL used only as independent evidence and no CDP and no synthetic UIA `Invoke`; the installed synthetic **v4 → v5** migration on 34 measured checks; and the owner's store parked and restored **twice** (the second time because §24 needed an installed cycle the first pass had not fully covered, and the installed app has no store-path override to fall back on). §26's gates — `ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`, `OWNER_STORE_OPENED_BY_F1 = NO` — were satisfied **before any commit was written**. Counts held exactly at F1's, which is §37's own docs-only check: **868 Rust / 219 UI in 8 files**, full gate **17 of 17**, drift 8/8, deny 1/1, core-smoke 3/3, package 4/4 with no SKIP, `verify_baseline_artifacts.py` PASS. Wrote §28's eight-document closure pack, §29's user-doc audit, §30's user-facing uninstall/retention wording and §31's coverage boundary. Verdict `F2 = COMPLETE`, `P5 = IN_PROGRESS`, `F3 = READY_FOR_ARCHITECT_REVIEW` — **F2 does not close P5**, and §42 returns the round to the Architect rather than forwarding it. Two boundaries it wrote down instead of smoothing over: installed migration is one path and the owner's real store is at **schema v2**, so the chained v2 → v5 installed upgrade is still unexecuted; and the uninstaller's "Delete the application data" option was **never exercised**, because that folder also holds unrelated historical stores from earlier phases
P5 Commit F2R1                       The narrow UI corrective the Architect inserted between F2 and F3, landing 2026-10-06 as `fb5f628`: F2's three installed findings fixed inside `apps/desktop/ui/src/**` and nowhere else. F2R-01 (S2) Analyze → Sections rendered one character per line because `Details.module.css` put `display: block` on the `<table>` (pinning the box to the pane and letting columns sit *below their own minimum*) while `overflow-wrap: anywhere` made that prose minimum one character; F2R-02 (S3) History's `Details` was the last cell of a table whose rigid minimum measures 939 px against a 736 px pane, so it sat 195 px past the right edge; F2R-03 (S3) Release printed the Core enum `MapRegionAndElfLoad` at a person. The fix is a `.viewport` wrapper owning `overflow-x: auto`, the tables back to `display: table`, prose bounded at a `24ch` measure with `break-word`, the Unknown reason on its own line, the row action moved to the **leading** cell (the Evidence table's own precedent), and one new shared `evidenceBasis.ts` read by Compare, Release and Analyze. **No design token byte changed, no dependency, no capability, no contract, no Rust.** 219 → **225 UI** with six new tests and four mutation proofs; Rust stays **868**. **Run `37431977428` (#70), attempt 1, headSha `fb5f62852a4c3b83bc472f5903bff214888ab621`, 10 of 10**, every job and step read individually, 868 Rust / 225 UI taken from the runner's own logs. Its §21 and §22 runs were **superseded and re-run** because three comment-only lines moved mid-campaign; the stale files are kept, not deleted
P5 Commit F2R2                       The evidence read-back head, **docs/evidence/governance only** (§42 forbids product code, and §43's proof is that both counts are identical to F2R1: **868 Rust / 225 UI in 8 files**). It installed **F2R1's own** CI artifact — `gh run download 37431977428 -n FirmwareSight-0.6.0-windows-x86_64`, artifact id `11397938806`, `FirmwareSight_0.6.0_x64-setup.exe` 3,886,598 B `efbc45a3…d5fd4`, installed binary 15,362,048 B `2cf01a6d…670b`; F1's `11337963032`, F2's binary and any local build were all excluded — with the owner's store parked first (`OWNER_STORE_PARKED = YES`, `OWNER_BACKUP_HASH_MATCH = YES`) and a copy of F2's preserved disposable v5 store in the live slot. Focused revalidation at 1024×720 / 1056×799 / 1440×900: **all three findings CLOSED**, §33 smoke green with no crash and no new path leak, §34 `P5_DESIGN_ACCESSIBILITY = PASS_FOR_FROZEN_DESKTOP_SCOPE` (one host, 96 dpi, 100 % scale; `WCAG_CERTIFICATION = NOT_PERFORMED`, `MULTI_DPI_125_150 = NOT_TESTED`, `SECOND_WINDOWS_HOST = NOT_TESTED`), §35 **L20 = CLOSED ACROSS VERIFIED HUMAN-FACING MEMORY-BASIS SURFACES**, and the store restored with `ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`, `OWNER_STORE_OPENED_BY_F2R = NO`. Uninstalled with "Delete the application data" read as unchecked and **left unchecked** — still `NOT_TESTED_BY_DESIGN`. Written: `P5_F2R_UI_CORRECTIVE_REPORT.md` (§37's twenty parts) plus addenda to `P5_KNOWN_LIMITATIONS.md` §8, `P5_EXIT_CHECKLIST.md` §4–§6, `P5_DESKTOP_ACCEPTANCE_REPORT.md` §9, `P5_EXECUTION_REPORT.md` §6, `P5_CI_AUTHORITY.md` (F2's two heads, F2R1's run) and these entry documents. F2's run #69 is recorded as **three attempts** — 7 jobs then 4 jobs cancelled with **0 executed steps** (allocation, §25), 10 of 10 on attempt 3 — and `d1dc61c` as having **no run of its own**, one push of two commits producing one run at the tip. Verdict: `F2R = FINAL PASS / COMPLETE`, `F2 = PASS`, `P5 = IN_PROGRESS`, product **MVP CANDIDATE**, `F3 = READY_FOR_ARCHITECT_REVIEW` and **not authorized** — F2R's §46 is a STOP
P5 Commit F3                         **Final governance closure, landed 2026-10-06: the last authorized unit of P5 and the only head allowed to write the closure sentence.** Authorized by *FirmwareSight — P5 Commit F3 Final Governance Closure, Execution Prompt v1.0 — Architect Authorized*, archived at `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitF3_Final_Governance_Closure_v1.0.txt` with **both** digests recorded because transport moves them — delivered bytes SHA-256 `860e00976b242f15de1473455941140599a8a5cd639409521e7c38a28a64514c` (36,458 B, 1,821 CRLF lines), stored Git blob `6cacd10ee1eff4a6c4ab90b4f9d51df987bad16cf94fe142961c0d37fd749c66` (34,637 B, 1,821 LF lines), the delta exactly the 1,821 carriage returns and every line proven identical one by one; `.gitattributes` was not touched to make the numbers agree, since that file is an `AGENTS.md` 9 integrity boundary. **What F3 did, in order:** §1 preflight at `a5ce7c4` (= `origin/main`, tree clean); §3 the named read set; **§6 the exit re-audit over the nineteen areas, which had to find no required engineering item `BLOCKED` before any sentence could be written** — it did not, and the one correction it owed the record was that `P5_EXIT_CHECKLIST.md` asserted every L1–L26 row was accounted for while `P5_KNOWN_LIMITATIONS.md` held **25 of 26: L12 had no row**, so F3 added the row and corrected the assertion instead of deleting the claim; §8/§23 L20's sequence reconciled with dates (E `CLOSED` on Compare → F2 `REOPENED_BY_F2` on Release → F2R `CLOSED_BY_F2R` → final `CLOSED`) without rewriting Commit E's measurement; §9 L15 kept split (presentation `CLOSED`, wire `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`, no `analysis:2`, no enum rename); §10 L26 `CLOSED — ADR-0029`, kept distinct from `ADR-0028`; §11 every carried limitation left carried — **no limitation became CLOSED because P5 closed**; §12 the measured performance truth untouched (519,179,252 B, ~2,020,073 symbols, warm ~4.73–4.89 s, cold ~68.7 s, ~1,428 MB working set, no "Not Responding"; near-500-MiB UI `MEASURED`, first-use-under-60 s **not proved at this workload**); §13/§14 the ten release-readiness states frozen and `PUBLIC OPEN-SOURCE REDISTRIBUTION CLAIM = BLOCKED BY OWNER LICENSE DECISION`; §15–§18 the canonical state in `BASELINE.yaml` (existing vocabulary only — `stage_status`, `closed_on`, `tests_at_close`, `product_state`, `next_stage_after_this_one`; **no new machine enum invented**) and in the entry documents; §19–§22 the lineage with every red run kept, the F2R2 external row, `F3 remote CI = PENDING_EXTERNAL_EVIDENCE`, and the new `P5_FINAL_CLOSURE_REPORT.md`; §24 **no install and no owner data** — installed authority stays F2R1's `fb5f628` / run `37431977428` / artifact `11397938806`; §25–§27 the allowlist proof and the full local validation; §26 the `ADR-0029` index-blob closeout order (edit → inspect → stage → tree → stage → sums → stage → verify → gate → commit); §28–§29 the detached-worktree and `git archive` proofs at the F3 SHA, the temporary worktree removed afterwards; §30 one governance commit and a normal fast-forward push, no force. **Verdict: `P5 = PASS_COMPLETE`, Productization = `ENGINEERING_COMPLETE`, `active_task = NONE`, product `MVP_CANDIDATE`, narrative "FirmwareSight Productized MVP Candidate", baseline `0.6.0`, G2 `PASS`, P0–P4 untouched.** Counts held at **868 Rust / 225 UI in 8 files** exactly as §5 demanded. **Final external acceptance requires F3 remote CI**, and §34 forbids a further commit written only to record it; if that run fails on repository content, §33 withholds the external verdict and the fix goes forward, never into a re-roll-until-green. What F3 does **not** mean: V1, B1, private beta, RC, GA, commercialization, a licence, signing, notarization, an updater, real-user validation, WCAG, all-DPI, all-platform runtime validation, or any new feature — `AGENTS.md` 1 and F3 §37 both stop the round here
Three red heads, kept in the record because dropping them would make the green ones meaningless: 9e3b1de (6 of 7 — a pre-existing `compare.test.tsx` race it never touched), 0c031cd (7 of 10 — three package jobs printing a skip and going red on their own upload), 3400981 (8 of 10 — a clock race inside a test Commit D added, fixed test-only at bccea88, which went 10 of 10)
The authoritative job set is TEN since 0c031cd: the seven gate jobs plus Package Windows / Ubuntu / macOS. `P5_CI_AUTHORITY.md` names them and every run measured against them
§34 STOP ran from 2026-10-04 and has been ANSWERED by the Architect's next prompt, the one that authorized Commit E (its digest is in the paragraph above). That STOP therefore now applies to what Commit E did not take: no §64 full journey, no general source-control linter, no V1/B1/RC/GA — and Commit E added two STOPs of its own, §63/§64 (the round stops at its exit lists), which still hold, and §42 (L15 as a serialized-contract decision for the Architect), which has been **ANSWERED**: Option E preserves `SourceType::ElfProgramHeader` / `elf.program-header` and documents the accurate meaning instead, so L15 is `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER` rather than CLOSED, and no rename, migration 0006 or `analysis:2` is authorized by that answer. The closure normalization prompt that carried the decision added its own stops (§14 L26 untouched, §15 §64 unrun, §16 consolidation docs unwritten, §27 no commit merely to record its run) — and **all four have since been discharged by their rightful owners**: §14 by Commit F1's `ADR-0029` (L26 is CLOSED, and a general source-control linter is still not authorized), §15 by the owner's own sequencing answer rather than by an assumption (F1 was landed first, and after it came back 10 of 10 the owner released F2, which has now **run §64 end to end on the installed F1 artifact** — the stop is discharged, not merely deferred), §16 by F2's assignment in the Commit F prompt (§45's consolidation documents are F2's, and F1 did not write them), and §27 by nobody — it still holds, which is why F1's run is not written into F1. The Commit F prompt now bounds the round: §18 keeps §64, any P5 verdict, any tag/release/sign/notarize/publish, any licence choice and any schema or migration change out of F1; §20 stops the round if L26 appears to need a portable-schema change; §67 reserves `P5 = PASS_COMPLETE` for F3 alone; §66 forbids an F4 written merely to record F3's run.

V1 — where the chain stands now (V1 has one commit and no CI authority file yet; this is the shape, and the pack itself is the record)
V1 opening head                08fdfcb710f78f8084bfcf614dc508c8b6e7e25b — P5 Commit F3's closure head. §3 preflight found HEAD = origin/main = it, tree clean, one worktree, and no remote delta to classify; §0 named the same head as the verified start authority.
V1 frozen cohort build         run 37475580080 · artifact id 11419727517 · name FirmwareSight-0.6.0-windows-x86_64 · ZIP container 5,536,303 B, which §1 insists is NOT the installer size · NSIS built name FirmwareSight_0.6.0_x64-setup.exe, delivered as FirmwareSight-0.6.0-windows-x86_64-nsis.exe, 3,888,432 B, SHA-256 182506f213383cfe00865f199fcec4fb17079535e8ea370f954fc15097263d12 · internal SHA256SUMS.txt verified: both entries OK, exit 0 · CLI zip 76bf7d0c…3f7ddd · payload executable target/release/firmwaresight-desktop.exe, payload_sha256 6598880dd8dde479d9326e678d0c22eddfc859a6c8cbc98ce7049b8f3c310646 · unsigned, no updater · expired: false at activation, GitHub expiry 2026-10-20T14:21:52Z · preserved OUTSIDE Git under FirmwareSight-V1-External-Validation-20261006/01_artifact and never committed (§1). Substitutions forbidden by name: F2R1's artifact 11397938806, a local rebuild, cargo run, Vite, a later docs-only CI artifact, and the V1 activation commit's own package.
V1 installer bytes move between CI runs, and that is known, not new  F3's installer differs from F2R1's (3,888,432 / 182506f2… vs 3,886,598 / efbc45a3…) even though their product trees are identical, because P5_PACKAGING_REPORT.md measured that these packages are not byte-reproducible — a payload differs in 20 of 15,001,088 bytes per link (PE TimeDateStamp plus the RSDS CodeView GUID). So a session records the SHA of the bytes its participant actually ran; a head is not a build.
V1 activation                  the head that lands this line. §5 decided the branch: no real eligible external participant and no session evidence existed, so the round wrote V1_VALIDATION/'s sixteen pack files (plan, metric contract with M1–M6 thresholds fixed before any data, participant register, five protocol documents continued from V0's discipline, sessions register and template, three analysis registers, two deliverables), left every register at zero, kept the report a skeleton, and simulated nobody. active_task = V1_OWN_ARTIFACT_EXTERNAL_VALIDATION · stage V1 = IN_PROGRESS · research_state = RECRUITMENT_READY · eligible external sessions = 0. P5 stays PASS_COMPLETE, G2 stays PASS, product stays MVP_CANDIDATE at 0.6.0, and product.status / validation.current_gate deliberately gained no V1 term — while P5 ran, neither carried a P5 term either, because those strings record verdicts. §43's boundary held with zero forbidden paths, and §45's counts had to stay 868 Rust / 225 UI in 8 files on the 17-step gate.
V1's stop, in its own terms    §5 step 9: Recruitment Ready, then STOP. The next move is the human operator's (§47, §48) — recruit real firmware/embedded engineers, obtain consent, transfer the frozen build one-to-one, moderate. Nothing may be written into sessions/ before a real transcript or real notes exist, and V1 may never self-issue V1_PASS_COMPLETE, B1_READY or PRIVATE_BETA (§39, §54); even a full pass does not open B1 (§40). L11 — real-user comprehension, the row V1 exists to answer — stays CARRIED_FORWARD in P5_VALIDATION/P5_KNOWN_LIMITATIONS.md, which V1 §43 does not let this round write to.
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
P5      PASS_COMPLETE (2026-10-06) — Productization ENGINEERING_COMPLETE, narrative FirmwareSight Productized
        MVP Candidate, closed by Commit F3 after §6's exit re-audit found no required engineering item
        BLOCKED. Evidence P5_VALIDATION/, verdict and boundaries P5_FINAL_CLOSURE_REPORT.md, 26 carried rows
        P5_KNOWN_LIMITATIONS.md. Not beta, RC, GA, signed, notarized or licensed open source
V1      IN_PROGRESS / RECRUITMENT_READY (opened 2026-10-06 under its own architect prompt) — a research
        track, not a product stage. 0 eligible external sessions, N minimum 8. Pack V1_VALIDATION/, cohort
        build frozen at artifact 11419727517. No B1, beta, RC, GA, licence, signing, notarization, updater
        or feature is authorized by it, and V1 may not self-issue its own PASS
Pricing / commercial research   DEFERRED_POST_MVP; the price-anchor prompt was withdrawn unexecuted
```

## What the code does and how it is proven

- **Present figures, measured on 2026-10-06 at the Commit F3 tree: 868 Rust tests across 47 executables and
  225 UI tests in 8 files**, on a gate of **17 steps** (`rust` 3 + `frontend` 5 + `drift` 8 + `deny` 1) plus
  3 under `--only core-smoke` and 4 under `--only package`, all green locally with **no `SKIP`**. Rust has not
  moved since Commit E; the UI line is 210 → 217 (Commit E) → 219 (F1's two L15 caption cases) → **225**
  (F2R1's three contracts and their guards: 1 in `details.test.tsx`, 2 in `history.test.tsx`, 1 in
  `intake.test.tsx`, 2 in `release.test.tsx`). **F2, F2R2 and F3 were each required to hold both numbers
  still** — F3 §5 states the rule in its sharpest form, because a docs-only head that moves a count is a
  product change wearing a documentation diff, and the round must stop rather than explain it.
- **Figures at the Commit E tree, measured 2026-10-04:** `cargo test --workspace` **868 Rust tests
  across 47 executables** (storage 131, desktop 222, and the +14 all in the new
  `crates/firmwaresight-artifact/tests/p5_compat_fixtures.rs`) and **217 UI tests in 8 files** — 6 added to
  `compare.test.tsx` and 1 to `details.test.tsx`, while `history.test.tsx` changed a test without adding one —
  on one gate: `python scripts/check.py` **16 steps** (`rust` 3 + `frontend` 5 + `drift` 7 + `deny` 1), plus
  3 under `--only core-smoke` and 4 under `--only package`, every one green locally and the package group with
  **no `SKIP`**. What the remote says about these heads is a separate fact and belongs to
  `P5_VALIDATION/P5_CI_AUTHORITY.md`, written by the run's reader rather than by the commit that provoked it.
- **The immediately previous total, measured 2026-10-04 at `956e250`, was 854 Rust tests** (storage 131,
  desktop 222) **and 210 UI tests in 8 files**, on the same 16 steps, all green locally and 10 of 10 jobs on the
  remote. The paragraph below is the **P3-era
  snapshot** it grew from, kept because its arithmetic and its suite-by-suite breakdown are the record of that
  round: 556 Rust tests in 28 executable suites (plus 6 empty doc-test suites) and 135 UI tests in 6 files on
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
20. `P4_VALIDATION/` and `G2_VALIDATION/` — the completed Release Bundle round and the whole-MVP engineering
    closure audit that made the product `MVP CANDIDATE`, including `G2_KNOWN_LIMITATIONS.md`, the L-list every
    P5 row is dispositioned against
21. **The live round, and the first thing to read if you are joining now:** `P5_VALIDATION/`.
    `P5_PRODUCTIZATION_AUDIT.md` (§4 A–H plus the §G limitation dispositions, L26 included),
    `P5_CI_AUTHORITY.md` (the ten jobs and every run measured against them), `P5_PACKAGING_REPORT.md`,
    `P5_INSTALL_RECOVERY_REPORT.md`, `P5_ONBOARDING_HISTORY_REPORT.md`, `P5_COMMIT_D_DESIGN.md` and
    `P5_DIAGNOSTICS_RECOVERY_REPORT.md` (whose §8 is the §32 read-back), then Commit E's own four:
    `P5_COMMIT_E_DESIGN.md` (§4 lists the five mutation proofs, §6 the reliability campaign),
    `P5_COMPATIBILITY_FIXTURE_REPORT.md` (per-fixture provenance, commands, hashes and product answers; §4 is
    the normalized A–H disposition table, §7a the clean detached worktree) with
    `P5_COMPATIBILITY_MATRIX.md` (status column closed to the five canonical values, §7b holding the
    release-readiness states that are not compatibility claims) and `P5_SUPPORTABILITY_REPORT.md`
    (§48's fourteen dispositions, L25's non-reproduction, and L15 now `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`),
    then `P5_COMMIT_E_SCHEMA_DECISION.md` — which was the question and is now **the answer too**: §1 to §10 are
    Commit E's pricing, §11 is the Architect's Option E — and the short record of the whole cleanup,
    `P5_COMMIT_E_CLOSURE_NORMALIZATION.md`. Finally `09_ADR/ADR-0028-release-identity-bytes-as-evidence.md`,
    and `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §7 if you are wondering what `elf.program-header` actually
    means. `.ai/ACTIVE_TASK.md` holds the commit
    table and the STOP chain, and it now reads **`active_task: NONE`**. The reading-path note above this list
    was written when Commit E's closure normalization was the live authority and it was true that **no prompt
    authorized a Commit F, a P5 closure, the §64 journey or an L26 fix**; all four were authorized afterwards,
    each by its own architect prompt — Commit F by *P5 Commit F — Final Productization Closure v1.0* (L26 as
    `ADR-0029` plus the §64 journey), F2's install by the owner's answer to the pause after F1, the F2R
    corrective by the Architect inserting it between F2 and F3, and the closure by *P5 Commit F3 — Final
    Governance Closure v1.0* — and the sentence is kept here dated rather than deleted because it is the
    record of what the tree did not yet have. **Current active authority: none.** `10_AUDIT/SOURCE_PROMPTS/README.md`
    lists every prompt P5 ran under with its measured digests, F3's last.

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
  stale. Since ADR-0029 the order is: stage the intended changes, regenerate the tree, **stage the tree**,
  regenerate the manifest, **stage the manifest**, then `python scripts/verify_baseline_artifacts.py`. The
  manifest hashes Git stage-0 index blobs, so an unstaged semantic edit makes `sums` refuse rather than
  certify stale content. Do **not** reach for `sha256sum -c SHA256SUMS` on a working tree — it is not the
  check any more; the archive form (`git archive <commit>` extracted outside the repository) is what still
  validates with it.

## The two rounds that ran after G2 (2026-10-02)

G2 is still the gate that stands, and this file's G2 paragraphs are still true about the tree they
measured. What was stale **as of 2026-10-02** is the set of numbers: that date stood at Rust 770 tests,
UI 159 in 6 files, and the newest evidence directory was `POST_G2_E2E_REMEDIATION/`. P5 has moved the tree
again since — the current figures are the ones at the top of "What the code does and how it is proven", and
`P5_VALIDATION/` is now the newest evidence directory. Read the rest of this section as the history it is.

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
