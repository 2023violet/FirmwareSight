---
title: "FirmwareSight Project Baseline"
doc_id: "FS-ROOT-README"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-10-07"
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
P5 packaging: PROVED ON THREE RUNNERS (2026-10-03). Run 37138881977 at head 1055242 is the first 10 of 10
        and the first to attach a built package; run 37143046338 at 53578e9 is the second, and its downloaded
        darwin set is what closed the checksum-index row. Evidence P5_VALIDATION/P5_PACKAGING_REPORT.md
P5 real Windows install acceptance: PROVED ON THE INSTALLED F1 ARTIFACT (first run 2026-10-03, accepted run
        2026-10-05). The packaged installer ran on 2026-10-03 through install, OS-surface launch, Analyze of a
        real fixture, close/reopen, repair, uninstall, reinstall and the data-retention check, with the owner's
        store parked, hashed and restored byte-identically; §38 C (first-run onboarding) and the §64 journey
        were not built at that run, so that line was not a verdict. Onboarding has since been built (Commit C)
        and **seen operating in the installed binary on 2026-10-04**, and **Commit F2 walked the whole §64
        journey contiguously on the exact CI-built F1 installer on 2026-10-05** — install through final
        uninstall, twice, with the migration proof and the owner-data barrier. What that round did not cover is
        listed rather than rounded off: `P5_DESKTOP_ACCEPTANCE_REPORT.md` §7 and the boundaries in
        `P5_MIGRATION_RECOVERY_REPORT.md` §4. **Commit F2R then installed its own CI artifact on 2026-10-06** —
        a *focused* revalidation of the three findings F2 raised at 1024×720 / 1056×799 / 1440×900 plus the
        §33 smoke list, not a second §64 walk, and §9 of the acceptance report says exactly which F2
        boundaries that left untouched.
        Evidence P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md, P5_DESKTOP_ACCEPTANCE_REPORT.md
P5 onboarding, Help and local History: BUILT, NOT YET ACCEPTED IN THE PACKAGE (2026-10-03). §13 guidance is
        one component read by Analyze's empty state and by Help; §14 Help reports the running binary's own
        identity and ships zero links; §15–§18 History is a fourth rail page over three new bounded storage
        read APIs with no new migration, and the L21 window title the install round reproduced is fixed in
        Rust over a closed page enum with no capability change. check.py 16/16, 812 Rust / 200 UI, seven
        mutation proofs, page reads 467.9µs / 541.5µs / 248.8µs at 100 builds / 100 runs / 50 releases.
        That §38 C line above is now about the code, not the package: walking the new screens in an installed
        binary is still open.
        Evidence P5_VALIDATION/P5_ONBOARDING_HISTORY_REPORT.md and
        P5_VALIDATION/P5_ONBOARDING_HISTORY_DESIGN_CHECKLIST.md
P5 release identity versus line endings (L22): DOCUMENTED, TESTED, NOT CHANGED — STOPPED FOR THE ARCHITECT
        (2026-10-03). Section 32 allows P5 to document and test this and forbids silently normalizing release
        identity. The draft lays out the three answers — hash the bytes (today), normalize before hashing, or
        hash the Git blob — with the reason each of the last two is worse than the problem: a normalized
        digest no longer describes the bytes inside the bundle it identifies, and a blob hash assumes the
        notes file is tracked in a repository, which the product does not assume today. The premise is now a
        test (same words, LF vs CRLF: both ids move, the verdict does not, and the bundle ships the bytes it
        hashed) and it was shown to bite by temporarily normalizing at the hash. One fact the G2 row did not
        carry: the fail-closed protection is policy-dependent, because `require_clean_git = false` makes
        `git.clean` N/A and lets the id move with nothing on screen saying why.
        Evidence P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md
P5 UI test races (L23), sixth instance: FOUND BY THE LOCAL GATE, CLOSED TEST-ONLY, STILL OPEN AS A CLASS
        (2026-10-03). Eight commits after `20b03e3` closed this class's fifth instance, on a tree whose UI source
        was identical to a head CI had passed twice, `compare.test.tsx` awaited the ranking region and then read
        the added rows synchronously — content that comes from a second IPC wave and commits on its own, with its
        own `Reading the added rows…` branch. Three full-suite runs on an idle host passed; the gate, running
        under its own compile load, did not. Repaired by one awaited query plus a test that holds that wave open,
        three mutation proofs and 20 clean fresh-process runs, with `Compare.tsx` untouched. The bounded sweep of
        chained IPC waves that §0a had asked for is recorded there with its limits named — Release's run-keyed
        waves and every `within(region)` read were not walked assertion by assertion — so L23 stays SHOULD_CLOSE_P5
        and the complete sweep is a remaining task, not a closed class. check.py 16/16 on that tree, 813 Rust /
        201 UI, and Run `37158606478` on `111fe32` returned 10 of 10 with both `Desktop UI` jobs green.
        Evidence P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md §0a
P5 release identity (L22): DECIDED — ADR-0028, THE BYTES ON DISK ARE THE EVIDENCE (2026-10-04). The
        Architect answered the draft above. Release identity uses the exact bytes observed on disk: LF and CRLF
        are distinct evidence, the Release Notes digest hashes the file as it was found, the Gate run id and the
        release id may therefore move between two checkouts of one commit, and the bundle ships the bytes it
        hashed and still verifies against them. No normalization, no Git-blob substitution, no identity-schema
        change — and no production identity code moved, because the rule is what the code already did.
        `a_notes_file_that_differs_only_in_line_endings_is_a_different_release` is now ADR-0028's canonical
        contract test rather than a premise written to prove a limitation, and it may not be weakened.
        `release.require_clean_git = false` stays a valid, explicit project-policy option and is not forced to
        `true`; its caveat is stated wherever the flag is reported, because with `false` a working-tree byte
        difference — line endings included — can change the digest, the run id and the release id without
        `git.clean` blocking the release. The recommended project practice is a `.gitattributes` rule pinning
        byte-sensitive release files, in the *user's* repository: FirmwareSight does not edit `.gitattributes`
        and never writes into the project folder it is analyzing. Diagnostics carries the caveat sentence
        beside the flag it explains, so a support file states what the flag costs.
        Evidence 09_ADR/ADR-0028-release-identity-bytes-as-evidence.md and
        P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md
P5 diagnostics and recovery (Commit D): BUILT, GATED, AND WALKED IN AN INSTALLED BINARY (2026-10-04). The
        store now answers for its own structure: `integrity_check()` asks SQLite and reports what comes back,
        read-only, and repairs nothing — no PRAGMA write, no VACUUM, no rebuild — which is what lets the Help
        screen state that a damaged store will be left damaged for a restore rather than rewritten. Any older
        file-backed store is snapshotted with SQLite's **online backup API before** it is migrated (a file copy
        is wrong under WAL, and `VACUUM INTO` binds its filename as text), the copy is opened and health-checked
        before it is renamed into place, and a snapshot that cannot be written stops the upgrade instead of
        starting it. Diagnostics is a closed 41-key allowlist assembled in Rust, shown on the page that
        already exists rather than as a fifth verb, and exported through a native dialog that opens only when a
        person names a folder; the store is named by file and never by directory, on screen and in the file, and
        the exported file on the packaged build contains no `/` and no `\` character at all. The startup refusal
        is now a typed sentence and exit code 1 rather than a panic. check.py 16/16 with the package group 4/4,
        854 Rust / 210 UI, eight mutation proofs, and §27's focused walk run against the installed package with
        the owner's store parked, copied to a second volume and restored byte-identically
        (ORIGINAL_DB_RESTORED = YES, ORIGINAL_DB_SHA_MATCH = YES). Two of the fixes exist only because the
        installed walk was run: a `setup` error is panicked by Tauri inside its event loop, so the exit path
        that looked typed was a 101 with a build-machine path in it, and a file that is not a database was being
        reported as a rolled-back migration to "schema version 0". A third exists only because the remote was
        run: the product head came back **8 of 10** (macOS Core Smoke, Rust ubuntu-latest) on a Commit D test
        that compared two snapshot files whose only available difference was a second-granular timestamp, so the
        assertion was a race with a clock; the repair is test-only, marks each store with a row it can name, and
        reads the standing snapshot to prove which one won. Both repair heads came back 10/10 on their first
        attempts (bccea88 on 37202016141, 90aa69d on 37202301591), every job read individually. COMMIT_D =
        COMPLETE; P5 remained IN_PROGRESS at that head and the prompt's section 34 STOP was in force.
        Evidence P5_VALIDATION/P5_DIAGNOSTICS_RECOVERY_REPORT.md, P5_VALIDATION/P5_COMMIT_D_DESIGN.md and
        P5_VALIDATION/P5_DIAGNOSTICS_DESIGN_CHECKLIST.md
P5 compatibility cohort and supportability (Commit E): BUILT FROM REAL TOOL OUTPUT, AND THE COHORT FOUND A
        PRODUCT DEFECT (2026-10-04). Run under its own architect prompt — FirmwareSight P5 Commit E —
        Compatibility Fixtures, Support Matrix & Supportability Closure v1.0, SHA-256 030ca282…ba21148, 53,915
        bytes, 2,459 lines, archived in 10_AUDIT/SOURCE_PROMPTS/ — which is what answered Commit D's §34 STOP.
        Six new fixture directories and 31 new tracked files now sit under fixtures/elf/p5-compat/ (executable-in-
        RAM, external SRAM, a DMA-shaped writable region, a linker whose MAP banner lands at byte 16571, a
        debug/no-debug pair, and one Clang-built Arm image), the manifest moved 25 → 56 entries, and every file
        was produced on this host by arm-none-eabi-gcc 14.3.1 and clang 22.1.8 final-linked by GNU ld 2.44.0 —
        no hand-edited evidence, and re-running the generator returns all 13 ELF and MAP bytes identical. The 14
        new tests derive each fixture's expected image and live-RAM totals from readelf section headers plus the
        MAP's own region attributes instead of from the model's arithmetic, and that independent rule found the
        round's one product defect: clang emits .ARM.exidx.text.main with sh_type 0x70000001 and SHF_ALLOC set,
        the parser had been reading "unrecognised kind" as "not allocated", and 8 real FLASH bytes entered
        neither budget while the total still printed Exact. Ten of the eleven committed ELFs already agreed, which
        is why no per-fixture test could see it. Fixed the narrow way the owner approved — SHF_ALLOC is read from
        the section header, which is what 04_TECH/23_MEMORY_ACCOUNTING_MODEL.md §3 and §4 item 3 already require —
        with a cohort-wide invariant test and five mutation proofs (A, B, C, E, F). §48's fourteen rows: L14, L16,
        L19, L20 and L24 CLOSED on measurement, L4, L5, L17 and L23 REDUCED with their residue named (L23's seventh
        instance repaired test-only, then 20 campaign runs × 217 tests with 0 failing), L25 NOT_REPRODUCED (30
        legitimate Apply activations), L8 and L12 CARRIED_FORWARD, and L15 stopped for the Architect because the
        label is serialized in three namespaces and renaming it is a public contract change, not a cleanup. No
        capability, schema, migration, dependency, licence or IPC surface changed. Locally: 868 Rust / 217 UI in 8
        files, check.py 16 of 16, core-smoke 3/3, drift 7/7, deny 1/1, package 4 of 4 with no SKIP, and a clean
        detached worktree at the candidate SHA green at 18 of 18 with all 56 manifest paths hash-matching and the
        fixture tests passing with no cross-compiler on PATH. The remote agreed: Run 37228929762 at head 59d85c3,
        attempt 1, 10 of 10, every job and every step read — the E1 head 859648e has no run of its own because one
        push carried both commits and GitHub runs the tip, which is recorded as an absence, not as a pass. The
        §59 read-back successor 6981625 came back 10 of 10 too, on Run 37230689636, attempt 1.
P5 Commit E closure normalization (evidence vocabulary + the L15 decision): DOCUMENTATION ONLY, NO PRODUCT
        SOURCE, NO FIXTURE BYTE (2026-10-04). Run under FirmwareSight P5 Commit E Closure Normalization v1.0 —
        delivered as 28,738 CRLF bytes with SHA-256 9571df2c…075599, archived under 10_AUDIT/SOURCE_PROMPTS/ as
        27,366 LF bytes / 1,373 lines with SHA-256 62040e3d…e6c7af887; both hashes recorded, because the stored
        copy differs from the delivered one only in the line terminator that .gitattributes requires for *.txt,
        and that difference is written down instead of hidden. Two fixes, no behavior: (1) the compatibility
        matrix's status column now uses exactly SUPPORTED / SUPPORTED_WITH_LIMITS / CI_BUILD_ONLY / NOT_TESTED /
        UNSUPPORTED — 38 cells validated with a table-parsing check, 0 outside the five — so the drifted values
        it had shipped with (BEST_EFFORT_NO_CLAIM, DEFERRED_NO_MVP, NOT_CLAIMED, "MEASURED, NOT INFERRED",
        BUILT_AND_VERIFIED_IN_CI and half-sentences like "SUPPORTED for the region") carry their meaning in the
        evidence column instead, and signing/notarization/updater readiness moved into a state column because
        READY_NOT_EXECUTED is a release-readiness state, not a compatibility claim; (2) the A-H disposition
        column now uses only PROVED_BY_EXISTING_FIXTURE / PROVED_BY_NEW_FIXTURE / SUPPORTED_WITH_LIMITS /
        NOT_AVAILABLE, with A, F and H recorded as EXISTING — every pre-Commit-E fixture was already a -g build
        and p0-dual-region and the p2-diff pair already carried 3, 5 and 4 PT_LOAD segments, so a new assertion
        on an old fixture is not a new fixture. And (3) the answer Commit E asked for: the Architect chose
        **Option E — legacy wire identifier preserved, accurate presentation / documentation**.
        SourceType::ElfProgramHeader and the analysis:1 token "elf.program-header" stay, the stored
        ElfProgramHeader rows stay, and nothing is renamed: no ElfSectionFlags, no wire rename, no rewritten
        history, no migration 0006, no analysis:2, no golden byte, no Bundle contract change, no release-identity
        move, ADR-0028 untouched. What replaces the accuracy the name never had is a definition —
        elf.program-header is a legacy compatibility identifier and promises nothing about PT_* headers, while
        MemoryEvidenceBasis::ElfAddressAndFlags means **ELF address + flags evidence**, now stated in
        P5_COMMIT_E_SCHEMA_DECISION.md §11 and 04_TECH/23_MEMORY_ACCOUNTING_MODEL.md §7. L15 is therefore
        CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER and explicitly NOT CLOSED, and one visible residue is recorded
        rather than fixed because this round may not touch code: the Evidence Inspector prints the stored token
        verbatim (Details.tsx:667 ← details.rs:223 ← query.rs:503 ← db.rs:507), so a user expanding such a row
        reads ElfProgramHeader; a display-only caption is Commit F's call. Direct counts unchanged as §19
        requires (868 Rust / 217 UI / gate 16 of 16 / drift 7 of 7 / deny 1 of 1 / package 4 of 4), the owner's
        store never opened, L26 still undecided, the §64 journey still unrun, and Commit F still unauthorized.
        Evidence P5_VALIDATION/P5_COMMIT_E_CLOSURE_NORMALIZATION.md. COMMIT_E lands
        under its own §42/§63 STOPs: no Commit F, no P5 closure, no §64 full journey, no L26 fix, no general
        source-control linter. Evidence P5_VALIDATION/P5_COMMIT_E_DESIGN.md,
        P5_COMPATIBILITY_FIXTURE_REPORT.md, P5_COMPATIBILITY_MATRIX.md, P5_SUPPORTABILITY_REPORT.md and
        P5_COMMIT_E_SCHEMA_DECISION.md
P5 Commit F1 (ADR-0029 + L15 presentation), documentation, tooling and one UI caption (2026-10-05). Authorized
        by FirmwareSight P5 Commit F Final Productization Closure v1.0, received 2026-10-04 as 68,315 bytes with
        SHA-256 887f2e9d…962fe9ca over 3,159 lines and archived byte-identically (it arrived LF-only, so unlike
        Commit E's closure prompt there is no delivered-versus-stored hash divergence to explain).
        **L26 is decided and closed: root SHA256SUMS now carries the SHA-256 of canonical Git stage-0 index blob
        bytes, not of working-directory bytes** (09_ADR/ADR-0029). The measured reason it moved: with
        core.autocrlf=true and text eol=lf, the count of entries disagreeing with a clean checkout was
        19, 19, 14, 14, 17, 13 at successive heads from 3400981 to 80b47c4 — 13 at this one — and no job anywhere
        had ever run the verifier, so nothing outside the writing host contradicted it. scripts/generate_baseline_artifacts.py
        now hashes index blobs through one batched git cat-file --batch and REFUSES to write while Git reports an
        unstaged semantic change to a tracked path; scripts/verify_baseline_artifacts.py re-derives the same
        content independently through :<path> index notation with no shared hashing helper; and
        drift/baseline integrity runs it inside the authoritative gate on Windows, Ubuntu and macOS, which is
        what makes the claim portable rather than host-local.
        **Operator-visible change: sha256sum -c SHA256SUMS against a working tree is no longer a valid check** and
        reports those 13 files as failures on this host by design; the canonical passes are
        python scripts/verify_baseline_artifacts.py and, externally, git archive <commit> extracted outside the
        repository. ADR-0028 is untouched — release identity is still the exact bytes observed on disk — and the
        three checksum domains (repository baseline, distribution SHA256SUMS.txt, Release Bundle SHA256SUMS) stay
        separate. ADR-0029 also keeps the basename exclusion, so golden/reports/p4-release/SHA256SUMS remains
        unlisted in the root manifest, by rule and on purpose.
        **L15's user-facing residue is closed, presentation only:** apps/desktop/ui/src/Details.tsx maps both
        namespaces of the legacy identifier (ElfProgramHeader from storage, elf.program-header from the wire) to
        **ELF address + flags evidence**, anything outside the table still renders verbatim, and details.test.tsx
        proves the caption while asserting the row object still carries the identifier it was stored under. No
        enum rename, no wire rename, no schema or golden change, no migration 0006, no analysis:2 — so
        **L15 itself stays CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER** with the presentation layer beneath it
        CLOSED.
        Counts at the F1 tree, printed rather than expected: 868 Rust passed / 0 failed across 47 targets
        (unchanged, F1 adds no Rust test), UI **219 in 8 files** (217 + the two inspector cases), full gate
        **17 of 17** with drift **8 of 8** (the new step), core-smoke 3/3, deny 1/1, package 4/4 with no SKIP,
        DIRECTORY_TREE.txt 813 lines and SHA256SUMS 693 entries over 695 tracked paths. A clean detached
        worktree at the F1 commit runs 19 of 19 for the same reason the main tree runs 17: the two cold-checkout
        asset steps. Evidence P5_VALIDATION/P5_COMMIT_F_DESIGN.md.
        **Sequencing, with both owner answers recorded:** the owner was asked before any live-machine work and
        chose *land F1, then confirm*, so F2 — downloading the exact F1 CI-built Windows artifact, parking the
        owner's app-data store, the installed old-schema migration proof and the full §64 journey — was NOT run
        in F1 and was not started on an assumption. Pre-test machine state was measured read-only first:
        %LOCALAPPDATA%\FirmwareSight absent, no uninstall registry entry, no running process, and the owner's
        store untouched at 155,648 bytes / d6e41034…ca836468 / last written 2026-09-30. After F1 returned 10 of
        10 the owner released F2, and the paragraph below is that round's record. F3, which alone may write
        P5 = PASS_COMPLETE, is not authorized until §62's ready-for-closure gate passes. So, at that head: P5 = IN_PROGRESS,
        product = MVP CANDIDATE at baseline 0.6.0, and still no P5 PASS, BETA, RC, GA or Production Ready anywhere.
P5 Commit F2 (installed productization acceptance), documentation, evidence and governance only (2026-10-05).
        Authorized by FirmwareSight P5 Commit F2 Installed Productization Acceptance v1.0 — Architect Authorized,
        whose scope line is 本 Prompt 只解除 F1 结束后的人工暂停，并授权 F2。不授权 F3: it lifts the manual pause
        after F1 and authorizes F2, and nothing beyond it. The prompt arrived at 19,040 bytes / 1,044 lines and
        vanished from Downloads after being read, so the archived copy is a transcript reconstruction, verified
        contiguous and byte-length-matched, and the register says so rather than implying provenance.
        **What was tested is the artifact CI built, not a local rebuild:** artifact 11337963032 of run
        37293381181 at head 0bca373 — 3,885,631 bytes, installer SHA-256 a1152ef3…3c076b, installed as
        downloaded, its inner package set verified file by file first, and the installed binary re-hashed to
        15,362,048 bytes / 93be8c8e…0c50. §3/§35 forbid product and tooling changes because any one of them
        would make that installer no longer the candidate under test, so **three product findings (one S2 layout,
        two S3) are recorded rather than patched**. Against this record itself: **three harness defects** — one of
        which had been silently delivering installer clicks to a browser window for most of the round — plus one
        harness-classed incident in which a title-bar close and some stray keystrokes reached a bystander window;
        all four are owned in `P5_DESKTOP_ACCEPTANCE_REPORT.md` §6 with their evidence files.
        **The §64 journey ran once, contiguously, on the installed binary driven by real mouse and keyboard
        input:** onboarding, Analyze baseline, Analyze target, Compare, Release Gate, Bundle, relocated-Bundle
        verify, History, History after source relocation, Diagnostics export, close, reopen, repair over a
        running app, uninstall, reinstall, retained-data re-verify, final uninstall. Synthetic **v4 → v5**
        migration through the installed binary passed on 34 measured checks, and retention was proved
        semantically rather than by digest because a WAL database churns bytes legitimately. The owner's store
        was parked and restored **twice**, hashes re-measured on both sides before any commit
        (ORIGINAL_DB_RESTORED / ORIGINAL_DB_SHA_MATCH / OWNER_STORE_OPENED_BY_F1 = YES / YES / NO).
        **Two boundaries are written down instead of smoothed over:** installed migration covers one path, and
        the owner's real store read back at **schema v2** — so the chained v2 → v5 installed upgrade has never
        been executed by the shipping artifact; and the uninstaller's "Delete the application data" option was
        never exercised, because that folder also holds unrelated historical stores from earlier phases.
        Counts at the F2 tree, printed rather than expected: **868 Rust / 219 UI in 8 files, both unchanged from
        F1** — §37's stop condition, since a docs-only round that moved either count would be a product change
        wearing a documentation diff — full gate **17 of 17**, drift **8 of 8**, core-smoke 3/3, deny 1/1,
        package 4/4 with no SKIP, and a clean detached worktree at **19 of 19** — the two extra steps are
        `rust/frontend assets (install)` and `rust/frontend assets (build)`, which a warm tree has already
        satisfied. DIRECTORY_TREE.txt 822 lines and SHA256SUMS 702 entries over 704 tracked paths, the baseline
        verifier PASS with `index blob mismatch 0`, and `git archive` of the candidate extracted outside the
        repository verifying **702 OK / 0 FAILED**. Both detached proofs were run at the F2 evidence head
        `d1dc61c` and are re-run at this head before anything is pushed.
        Evidence P5_VALIDATION/P5_DESKTOP_ACCEPTANCE_REPORT.md,
        P5_MIGRATION_RECOVERY_REPORT.md, P5_HISTORY_DIAGNOSTICS_REPORT.md,
        P5_SECURITY_SUPPORTABILITY_REVIEW.md, P5_KNOWN_LIMITATIONS.md, P5_EXECUTION_REPORT.md,
        P5_EXIT_CHECKLIST.md, P5_RELEASE_READINESS.md. Verdict: **F2 = COMPLETE, P5 = IN_PROGRESS,
        F3 = READY_FOR_ARCHITECT_REVIEW** — and this head sets none of P5 PASS_COMPLETE, active_task NONE, a
        tag, a GitHub Release, a signature, a notarization, an updater or a licence.
P5 Commit F2R (installed UI productization corrective), two heads (2026-10-06). Authorized by
        FirmwareSight P5 Commit F2R Installed UI Productization Corrective Candidate v1.0 — Architect
        Authorized, archived with both measured digests (delivered 5d8719ec…fb375 / 47,100 bytes / 2,117 CRLF
        lines; stored blob 6281f5d2…fe4ed / 44,984 bytes / 2,116 LF lines). It exists because Commit F2
        measured three product findings on an installed binary it was forbidden to patch, and the Architect
        chose a narrow corrective over letting F3 inherit them.
        **F2R1 `fb5f628` is the only head here that contains product code, and all of it sits in
        apps/desktop/ui/src/**.** F2R-01 (S2): Analyze → Sections laid its prose column out one character per
        line, because display: block on the `<table>` pinned the box to the pane and let its columns sit *below
        their own minimum* while overflow-wrap: anywhere made that minimum one character. F2R-02 (S3):
        History's Details was the **last** cell of a table whose rigid minimum measures 939 px against a 736 px
        pane, so the only action on the row was the thing that fell off the right edge. F2R-03 (S3): Release
        printed the Core enum MapRegionAndElfLoad at a person. The repair is one mechanism, measured first
        (P5_F2R_UI_CORRECTIVE_DESIGN.md keeps the probe numbers, the negative control that a wrapper alone does
        not fix it, and the 195 px / 163 px occlusion that is why sticky pinning was rejected): a `.viewport`
        wrapper owns overflow-x: auto, the tables are tables again, prose cells take a 24ch measure with
        break-word, the Unknown reason moves onto its own line, the History action moves to the leading cell
        (the Evidence table's own precedent), and one shared UI-only evidenceBasis.ts feeds Compare, Release and
        the Analyze weakest-basis sentence the same caption for the same basis. **No design token byte changed,
        no pixel constant invented, no dependency, no capability, no schema, migration or contract movement** —
        analysis:1, diff:1, gate-results:1, accepted-reviews:1, release-manifest:1, SQLite schema 5, migrations
        0001–0005, ADR-0028, ADR-0029 and elf.program-header are untouched, because this round is
        display/layout only. Counts at the F2R1 tree: **868 Rust unchanged / 225 UI in the same 8 files**
        (219 + 6: one details contract, two history contracts, one Analyze caption, two Release captions), four
        mutation proofs each reddening their own test, full gate **17 of 17** with drift 8/8, deny 1/1,
        core-smoke 3/3, package 4/4 and no SKIP, and a §22 reliability campaign of **20 fresh-process
        repetitions, 20 green**. Remote: Run `37431977428` (#70), **attempt 1, 10 of 10**, every job and step
        read individually, with 868 Rust / 225 UI taken out of the runner's own logs.
        **F2R2 is this head: docs, evidence and governance only, with both counts held identical to F2R1** —
        which is §43's own proof that a documentation round is not a product change wearing a documentation
        diff. It downloaded and installed **F2R1's own CI artifact** (§26 forbids borrowing F1's
        `11337963032`, F2's binary, or any local cargo/tauri build): artifact id `11397938806`,
        FirmwareSight_0.6.0_x64-setup.exe 3,886,598 bytes `efbc45a3…d5fd4`, installed
        firmwaresight-desktop.exe 15,362,048 bytes `2cf01a6d…670b`, verified against the set's own
        SHA256SUMS.txt before it ran, with the bundler's 3-byte `__TAURI_BUNDLE_TYPE_*` rewrite checked on the
        installed bytes rather than taken on trust. The owner's store was parked with digests first
        (OWNER_STORE_PARKED / OWNER_BACKUP_HASH_MATCH = YES / YES) and a copy of F2's preserved disposable v5
        store seeded the live slot; the product was never pointed at owner data. Focused revalidation at
        **1024×720, 1056×799 and 1440×900** — all three findings **CLOSED** on the installed window, `Details`
        surviving SPACE close/reopen under a visible focus ring, Release reading "ELF address/flags evidence"
        and **"MAP regions + ELF load evidence"** with no raw enum on that surface or on Analyze's
        weakest-basis line. §33 smoke green (Analyze, Compare, Gate twice, History, Diagnostics) with no crash,
        no new path leak, no new S0 and no new S1. §34 gives **P5_DESIGN_ACCESSIBILITY =
        PASS_FOR_FROZEN_DESKTOP_SCOPE and nothing more** — one host at 96 dpi / 100 % scale, with
        WCAG_CERTIFICATION = NOT_PERFORMED, MULTI_DPI_125_150 = NOT_TESTED and SECOND_WINDOWS_HOST =
        NOT_TESTED unchanged — and one trade is recorded rather than glossed: at 1440×900 the corrected Sections
        table now needs a contained scroll of about 1.06× the pane, where the pre-fix layout fitted the pane by
        being unreadable. §35 closes **L20 across verified human-facing memory-basis surfaces** (Compare →
        REOPENED_BY_F2 on Release → CLOSED_BY_F2R) without rewriting Commit E's sentence, and the §13 re-audit
        records ConfiguredRegionAndElfLoad and InsufficientEvidence as **spellings with no code behind them**
        rather than mapping invented semantics. Uninstalled with "Delete the application data" read
        (BM_GETCHECK) as unchecked and deliberately left unchecked — that option is **still NOT_TESTED_BY_DESIGN**,
        for the same reason F2 gave — then the disposable store's final bytes were preserved, its three live
        files removed, and the owner's store restored byte-exact: **ORIGINAL_DB_RESTORED = YES,
        ORIGINAL_DB_SHA_MATCH = YES, OWNER_STORE_OPENED_BY_F2R = NO**. Harness facts owned, not buried: one
        refused installer click (POINT_OWNED_BY_OTHER_WINDOW — a Chrome window owned the pixel and no input was
        sent), one mis-set installer step table, one FOREGROUND_NOT_ACQUIRED retried only after raising the app
        window, and a geometry driver that first matched somebody else's window and was fixed with
        require_app() **before** any installed evidence existed. Evidence root
        `FirmwareSight-P5-F2R-20261005T221112`, outside the repository. Verdict: **F2R = FINAL PASS / COMPLETE,
        F2 = PASS, P5 = IN_PROGRESS**, product stays **MVP CANDIDATE** at `0.6.0`,
        **F3 = READY_FOR_ARCHITECT_REVIEW and is not authorized here** — F2R's own prompt ends in a STOP. This
        head sets no P5 completion token, no active_task NONE, no tag, no GitHub Release, no signature, no
        notarization, no updater and no licence.
P5 Commit F3 (final governance closure), governance / evidence / indexing only (2026-10-06). Authorized by
        FirmwareSight P5 Commit F3 Final Governance Closure Execution Prompt v1.0 — Architect Authorized, the
        last authorized unit of the stage, archived at
        10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitF3_Final_Governance_Closure_v1.0.txt with BOTH digests:
        delivered bytes SHA-256 860e00976b242f15de1473455941140599a8a5cd639409521e7c38a28a64514c (36,458 B,
        1,821 CRLF lines) and stored Git blob 6cacd10ee1eff4a6c4ab90b4f9d51df987bad16cf94fe142961c0d37fd749c66
        (34,637 B, 1,821 LF lines). They differ only because .gitattributes normalizes *.txt to LF, and that is
        proved rather than asserted: the byte delta is exactly the 1,821 carriage returns, and comparing the
        delivered file split on CRLF against the stored blob split on LF gives identical text on every one of
        the 1,822 logical lines with zero differing lines. .gitattributes was not touched to make the hashes
        agree — it is an AGENTS.md 9 integrity boundary.
        **Order mattered: §6's exit re-audit ran before any closure sentence was written.** All nineteen areas
        were re-read against measurements — Packaging, Onboarding, History, Diagnostics, Migration/recovery,
        Compatibility, Design/Accessibility, Documentation, Security/supportability, Windows real install,
        Version identity, Repository baseline integrity, Signing, Notarization, Manual update, License, CI,
        Clean detached tree, Owner DB restore — and **no required engineering item came back BLOCKED**; the
        only BLOCKED phrase in the whole record is the one §14 requires about redistribution, which is a claim
        about publishing rather than an engineering item. The re-audit's one real find was in this
        repository's own paperwork: P5_EXIT_CHECKLIST.md asserted every L1–L26 row was accounted for while
        P5_KNOWN_LIMITATIONS.md held 25 of 26 — **L12 had no row**, though the fact it covers was carried
        honestly in two other documents. F3 added the row and corrected the assertion rather than deleting the
        claim. L20's history was reconciled with dates (Commit E CLOSED on Compare → F2 REOPENED_BY_F2 on
        Release → F2R CLOSED_BY_F2R → CLOSED) without rewriting Commit E's measurement; L15 stays split
        (presentation CLOSED, wire CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER, no analysis:2); L26 is CLOSED —
        ADR-0029, kept distinct from ADR-0028; and **no limitation became CLOSED merely because P5 closed** —
        L3, L4, L5, L8, L11, L12, L13, L15, L17, L23, L25 all keep their carried dispositions. The measured
        performance truth is untouched: a valid ELF of 519,179,252 bytes with about 2,020,073 symbols, warm
        Analyze ~4.73–4.89 s, cold first read ~68.7 s, peak observed working set ~1,428 MB, never "Not
        Responding" and never a crash — near-500-MiB UI is MEASURED, first-use-under-60 s is NOT PROVED at
        this workload, and no claim that performance was optimized, that a 500 MB target was achieved or that
        every large file finishes inside a stated window appears anywhere.
        **Canonical state now: P5 = PASS_COMPLETE · Productization = ENGINEERING_COMPLETE · product =
        MVP_CANDIDATE · narrative = FirmwareSight Productized MVP Candidate · baseline = 0.6.0 · active_task =
        NONE**, with G2 still PASS and P0–P4 untouched. BASELINE.yaml was updated only in its own existing
        vocabulary (stage_status, closed_on, tests_at_close, product_state, next_stage_after_this_one) and no
        machine enum was invented for the run F3 could not yet read: inside this commit **F3 remote CI =
        PENDING_EXTERNAL_EVIDENCE**, and FINAL EXTERNAL ACCEPTANCE REQUIRES F3 REMOTE CI — if that run fails on
        repository content the external verdict is withheld and the fix goes forward, never re-rolled until
        green; §34 forbids a fourth round written only to record a number. Proof it is still documentation:
        **868 Rust / 225 UI in 8 files unchanged** (§5 requires the counts to stand still and F3 stops if they
        do), zero paths under apps/ crates/ scripts/ fixtures/ schemas/ golden/ migrations/ .github/, and no
        Cargo.toml, Cargo.lock, package.json, pnpm-lock.yaml, tauri.conf.json, deny.toml, rust-toolchain.toml
        or assets/design-tokens.json byte. §24: **nothing was installed and no owner file was touched** —
        installed authority stays F2R1's head fb5f628, run 37431977428, artifact 11397938806. The record is
        P5_VALIDATION/P5_FINAL_CLOSURE_REPORT.md, with the full head-by-head lineage in P5_EXECUTION_REPORT.md
        §7, the exit re-audit in P5_EXIT_CHECKLIST.md §7 and the CI rows in P5_CI_AUTHORITY.md.
        **And the stage stops here.** Closing P5 authorizes nothing after it: V1 own-artifact / real-user
        validation (the track that exists to answer L11), B1 / private beta, RC, GA, commercialization, a
        licence, signing, notarization, an updater and any new feature all need a NEW architect prompt.
        PUBLIC OPEN-SOURCE REDISTRIBUTION CLAIM = BLOCKED BY OWNER LICENSE DECISION: license = "Proprietary"
        stands, there is no root LICENSE, and FirmwareSight is not a licensed open-source release.
V1 own-artifact external validation: OPENED 2026-10-06, IN_PROGRESS at RECRUITMENT_READY, 0 SESSIONS.
        Authorized by FirmwareSight — V1 Own-artifact External Validation, Execution Prompt v1.0 — Architect
        Authorized (delivered file SHA-256 48768ff025e3d6351b7b1d8d593ea8bfb18a7930a0a21a101af3093bbec10a0f,
        46,207 bytes, 1,697 CRLF pairs, 1,698 logical lines; archived as
        10_AUDIT/SOURCE_PROMPTS/FirmwareSight_V1_Own_Artifact_External_Validation_v1.0.txt, Git blob
        d5e8d4575581bd656b568eed97090e7130a51774, SHA-256 c62314fee5032ca5ffdbcfe95f51cf3bdb783e7d55a42a165ea526686690c34e,
        44,510 LF bytes — the two digests differ by exactly the 1,697 removed CR bytes and split into the same
        1,698 lines, every one identical, .gitattributes untouched). It is a RESEARCH track, not a product
        stage: no feature, schema, migration, dependency, cloud, account, telemetry, updater, signing,
        notarization, licence, pricing, B1, RC or GA, and the product code is frozen for the length of a formal
        cohort. §3 preflight: HEAD = origin/main = 08fdfcb (P5's F3 closure head), tree clean, one worktree, no
        remote delta. §1 froze the cohort build to that head's CI artifact — run 37475580080, artifact id
        11419727517, ZIP container 5,536,303 bytes which is NOT the installer size, NSIS
        FirmwareSight_0.6.0_x64-setup.exe delivered as FirmwareSight-0.6.0-windows-x86_64-nsis.exe at
        3,888,432 bytes / SHA-256 182506f213383cfe00865f199fcec4fb17079535e8ea370f954fc15097263d12, internal
        SHA256SUMS.txt verified OK on both entries with exit 0, payload firmwaresight-desktop.exe
        6598880dd8dde479d9326e678d0c22eddfc859a6c8cbc98ce7049b8f3c310646, unsigned, expired: false at
        activation — downloaded, verified and preserved OUTSIDE Git, because §1 forbids committing the
        installer. Those bytes legitimately differ from F2R1's installer even though the product trees are
        identical: P5_PACKAGING_REPORT.md measured these packages to be NOT byte-reproducible (a payload
        differs in 20 of 15,001,088 bytes per link — PE TimeDateStamp plus the RSDS CodeView GUID), so a head
        is not a build and every session must record the SHA the participant actually ran.
        **§5 is the branch this round took, and it is the whole story of the round: there are no real
        participants.** No agent may simulate one, use an LLM as one, count an internal team member, invent a
        quote, invent a task completion or mark a session complete. So V1 wrote the Recruitment Ready pack and
        stopped: V1_VALIDATION/'s sixteen §13 files — README, plan, metric contract (M1 >80 %, M2 median <60 s,
        M3 ≥60 %, M4 >80 %, M5 ≥30 %, M6 ≥62.5 % AND ≥5 unique YES, all fixed BEFORE any data because §15
        forbids moving a threshold after seeing it), participant register, five protocol documents carried
        forward from V0's discipline rather than reinvented (M01–M17, exclusions, neutral tasks; three named
        changes: installed product instead of a prototype, thresholds now exist, and C6–C8 added to V0's
        C1–C5), sessions register and template, three analysis registers, two deliverables — with every
        register at ZERO, the validation report a skeleton, and the recommendation
        V1_INCOMPLETE_INSUFFICIENT_SAMPLE at N = 0 against a minimum of 8.
        State written: active_task = V1_OWN_ARTIFACT_EXTERNAL_VALIDATION · V1 = IN_PROGRESS · research_state =
        RECRUITMENT_READY · eligible external sessions = 0. What did NOT move: P5 = PASS_COMPLETE,
        Productization = ENGINEERING_COMPLETE, G2 = PASS, product = MVP_CANDIDATE, baseline = 0.6.0, the ten
        frozen release-readiness states, the licence with the owner, and deliberately product.status /
        validation.current_gate — while P5 was running neither string mentioned P5, because those fields carry
        verdicts and V1 has earned none. V1_own_artifact_external_validation_after_G2 left
        next_authorizable_tracks for the same reason P5's entry did: it is now the live task, not a track
        awaiting a decision. §43's boundary held with zero forbidden paths, and §45 requires the counts to stand
        still at 868 Rust / 225 UI in 8 files on the 17-step gate. L11 — the row V1 exists to answer — stays
        CARRIED_FORWARD in P5_VALIDATION/P5_KNOWN_LIMITATIONS.md, which §43 does not let this round write to;
        §55 allows at most READY_FOR_ARCHITECT_REVIEW, and only once evidence exists to review.
        **And V1 stops here, at §5 step 9.** The next move is a human one (§47, §48): recruit real firmware and
        embedded engineers, screen them against §7, obtain consent, transfer the frozen unsigned build
        one-to-one to named consenting participants only, and moderate. PUBLIC_DISTRIBUTION remains
        NOT_AUTHORIZED and the research build is not a beta. V1 may never self-issue V1_PASS_COMPLETE, B1_READY
        or PRIVATE_BETA (§39, §54), and even a full pass at N ≥ 8 with all six metrics met yields only
        V1_READY_FOR_ARCHITECT_VERDICT — §40 says plainly that it does not open B1.
U1 UI productization convergence: OPENED AND CLOSED 2026-10-07, CLOSED_FIRST_ROUND, V1 PAUSED WITH STATE INTACT.
        Authorized by 《FirmwareSight B1 — UI Productization / Design Convergence》 v1.0, delivered INLINE as
        message text rather than a file, so there is no owner-side byte stream to hash: the source-prompt register
        records it as File: none (delivered inline), the way the three inline P0 precedents are recorded, and the
        executing agent's labelled transcription is archived at
        U1_VALIDATION/00_authority/SOURCE_PROMPT_U1_transcription.md (503 lines, 16,762 bytes, SHA-256
        67067dce624a6dc458055dd79e21b6ed4e983c7ac665426561e72b939de52865 — the transcription's digest, never the
        prompt's). The owner re-registered the track as U1 because B1 is the canonical Private Beta identifier and
        both F3 §37 and V1 §40 record it NOT AUTHORIZED. Start authority: HEAD = origin/main = f481b78, tree
        clean, run 37508243514 (#73) 10 of 10.
        What converged: seven shared components replacing five page dialects of the same grammar (373 lines of
        duplicated page CSS deleted, zero token values created, the generated styles/tokens.css and
        assets/design-tokens.json untouched and proved so by drift/design tokens), a new Overview page made only of
        DTOs the shell already returns with every action a navigation rather than a performed verb, the Release
        verdict moved to the top of its page with one shared verdict sentence so a run cannot read two ways on two
        pages, and the F2R-01 block-table defect closed on Compare and Release, where it was still live. The single
        non-CSS change was declared before it was made: MainWindowPage gained an additive Overview variant,
        because the window title is Rust's and not the WebView's.
        What did NOT move: no Core semantic, schema, migration, storage contract, analysis wire field, release
        identity rule, ADR-0028 or ADR-0029 conclusion, capability, dependency or IPC command (30 at both the
        base head and the U1 head). P5 stays
        PASS_COMPLETE, Productization ENGINEERING_COMPLETE, G2 PASS, product MVP_CANDIDATE at 0.6.0, the ten frozen
        release-readiness states, the licence with the owner, and every V1 state field — V1 gains only paused_for.
        L11 stays CARRIED_FORWARD: U1 built no participant evidence, it built a UI.
        Counts: 868 Rust unchanged / 236 UI in 9 files (was 225 in 8; eleven added, none deleted), same 17-step
        gate. visual_evidence = STRUCTURAL_CONTRACTS_ONLY: the real-desktop before/after pass needs the owner's
        store parked and was not authorized, so no screenshot was taken and no pixel claim is made.
        **U1 issues no stage and opens nothing.** It is not V1 done, not GA, not "equals the mockups"; the detail
        column, the compact control level, the History / Bundle & History naming, the two Release actions with no
        command behind them and the real-desktop evidence pass are listed as unfinished in
        U1_VALIDATION/U1_VALIDATION_REPORT.md §6 rather than presented as finished. The round ends at its own §16
        step 8: report, then STOP.
U1's two later units the same day: INSTALLED, INSPECTED, THEN CORRECTED. The record above is U1's first round and
        its visual_evidence line is what that round produced; two more units ran on 2026-10-07 under their own
        Architect prompts, and neither rewrote the other's lines.
        **U1_PUSH_CI_AND_INSTALLED_VISUAL_ACCEPTANCE** (prompt v1.1 operative, v1.0 archived as its prior revision)
        pushed 8efe9c8 + 7dc2ca8, read run 37609108402 (#74) at 10 of 10, installed the Windows artifact that run
        built (11477857379) and looked at it on a real screen: six primary states at 1440x900, nine responsive
        captures, a functional smoke pass and an eight-dimension convergence matrix. It raised one material
        presentation finding, U1-V2-06, and did not self-waive it. FS-U1-004,
        U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md.
        **U1R_STALE_CAPABILITY_CORRECTIVE_AND_FINAL_RECHECK** (prompt v1.0, archived with its delivered digest
        0ff7b3f6…650bb over 28,659 bytes and 1,302 logical lines) closed that finding. Analyze had been resolving
        its visible summary as the stored last-good one and rendering the top capability strip from it whenever any
        summary existed, so a failed attempt showed the previous file's green "ELF supported / MAP provided" pills
        above a row naming a different file. Retention stayed: the header and the strip now read from a derived
        presentation value populated only when the current attempt earned it, while the retained report and Details
        keep reading from the last good summary and keep saying whose they are. Three UI files changed and no Rust
        source did; no domain state was defined, no Gate verdict moved, and assets/design-tokens.json is
        byte-identical. Six contract tests guard it (242 UI in 9 files, was 236; 868 Rust unchanged), and reverting
        the derivation fails three of them. Gate 17 of 17 with no SKIP plus drift 8/8, deny 1/1, core-smoke 3/3 and
        package 4/4; product commit 4d36f10 pushed as a normal fast-forward; run 37637056980 (#76, attempt 1)
        10 of 10; Windows artifact 11491960105 installed and re-checked in the success, pending,
        failed-at-three-sizes and recovered states with the owner's store parked and restored byte-exact.
        FS-U1R-001 U1_VALIDATION/U1R_CAPABILITY_STATE_DESIGN.md and FS-U1R-002
        U1_VALIDATION/U1R_CORRECTIVE_REPORT.md.
        **Where that leaves U1:** U1-V2-06 = CLOSED_BY_U1R, and U1 = READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT — the
        strongest word those rounds allow themselves. It is not visual approval, not PASS_COMPLETE, not
        DESIGN_COMPLETE, not MOCKUP_MATCHED: the seven U1-V1 minor gaps and U1-V0-09 stay
        OPEN_FOR_ARCHITECT_VISUAL_JUDGEMENT, V1 stays paused at RECRUITMENT_READY with 0 eligible sessions and its
        frozen cohort build untouched, and no product stage moved: not B1, not Private Beta, not RC, not GA, no tag,
        no GitHub Release, no public installer, no signing, notarization, updater, licence choice or pricing.
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
12. `.ai/ACTIVE_TASK.md` — currently `NONE`: G2 closed `PASS` on 2026-10-01 and P5 productization closed
    `PASS_COMPLETE` on 2026-10-06, so the MVP engineering candidate is complete and productized to P5's
    engineering scope. **No next track is authorized by either closure**; V1 own-artifact validation, B1, RC
    and GA each need their own architect decision
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

**`active_task: U1_UI_PRODUCTIZATION_CONVERGENCE`, and its three rounds closed on 2026-10-07.** U1 was a **UI
productization track**, not a product stage: it converged the desktop interface toward the frozen seven-screen
reference set in `assets/ui-mockups/` — one shared design-system layer instead of five page dialects, a new
Overview page built only from facts the shell already reports, the Release verdict moved to the top of its page,
and the layout defect F2R-01 closed on the two pages that still carried it. It moved no Core semantic, no schema,
no migration, no storage contract, no wire format, no release identity rule and no ADR conclusion, added no IPC
command, and created no token value. `U1_VALIDATION/U1_VALIDATION_REPORT.md` is the record, and its §6 is the list
of six things the round deliberately did **not** reach. Two more U1 units ran the same day under their own
Architect prompts: the second installed the CI-built Windows bytes and inspected them on a real screen
(`U1_VISUAL_ACCEPTANCE_REPORT.md`), raising one material presentation finding it refused to self-waive, and the
third — **U1R** — closed exactly that finding in the presentation layer
(`U1R_CAPABILITY_STATE_DESIGN.md`, `U1R_CORRECTIVE_REPORT.md`). `U1-V2-06` is `CLOSED_BY_U1R` and
**`U1 = READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT`**, which is the strongest word those rounds allow: not visual
approval, not `PASS_COMPLETE`, not `DESIGN_COMPLETE`, not `MOCKUP_MATCHED`, and the seven `U1-V1` minor gaps plus
`U1-V0-09` stay open for the Architect's judgement. The product stays **`MVP_CANDIDATE`** at baseline
`0.6.0` with narrative **FirmwareSight Productized MVP Candidate**; P5 stays `PASS_COMPLETE`, G2 stays `PASS`, and
U1 authorizes no stage — `B1` Private Beta (the identifier the delivered prompt used for itself, which the owner
re-registered) remains reserved and NOT_AUTHORIZED, and so do RC, GA, signing, notarization, an updater, a licence
and any new feature.

**`V1 (Own-artifact External Validation)` is paused, not closed.** It opened on 2026-10-06 under its own
architect prompt at **`IN_PROGRESS` / `research_state = RECRUITMENT_READY`**, with **0 eligible external
sessions**, and it keeps every one of those values while paused (`v1_execution.paused_for` names U1). It is a
research track, not a product stage: the cohort build is frozen to P5
Commit F3's CI artifact (run `37475580080`, artifact id `11419727517`, NSIS installer 3,888,432 bytes, SHA-256
`182506f2…63d12`, **unsigned**, verified against its own `SHA256SUMS.txt` and preserved outside Git). While V1
ran, the product counts were required to hold at **868 Rust / 225 UI in 8 files**; U1 was authorized by its own
prompt to change the UI, so the live figure after U1R is **868 Rust / 242 UI in 9 files** on the same 17-step gate
(236 after U1's first round, plus U1R's six contract tests, with no Rust test moving and none deleted), and
nothing either track does can authorize a feature, a schema, a dependency, a licence, a signature or a release.

**The work that is actually blocked, and on whom.** V1 needs real firmware and embedded engineers running their
**own** artifacts. §48 assigns recruitment, consent, transfer and moderation to the **human operator**; an agent
maintains the protocol, validates eligibility, and turns real notes or transcripts into anonymized records — and
§5 forbids it from being a participant, using an LLM as one, or writing a session file for a session that did not
happen. Start at `V1_VALIDATION/README.md`, then `V1_PLAN.md`, `V1_METRICS.md` and the five `protocol/`
documents. The next repository write in this track should be a Batch A evidence commit **after** four eligible
participants exist.

**`P5` closed on 2026-10-06 and stays closed.** Stage **P5 (Productization)** opened on 2026-10-03 and **closed
`PASS_COMPLETE` on 2026-10-06** with Commit F3, the only head of the stage authorized to write that sentence and
only after §6's exit re-audit found no required engineering item `BLOCKED`. The state a newcomer inherits is
therefore: Productization **`ENGINEERING_COMPLETE`**, product **`MVP_CANDIDATE`** at baseline **`0.6.0`**,
narrative **FirmwareSight Productized MVP Candidate**, `G2 = PASS`, P0–P4 unchanged. Read
`P5_VALIDATION/P5_FINAL_CLOSURE_REPORT.md` for the verdict and its boundaries,
`P5_VALIDATION/P5_KNOWN_LIMITATIONS.md` for all 26 carried rows, and `P5_VALIDATION/P5_RELEASE_READINESS.md`
for the ten frozen release-readiness states.

**Nothing after P5 was authorized by P5's closure** — and that is still true of the closure sentence itself,
which wrote `active_task: NONE` and named no successor. V1 own-artifact / real-user validation was the likely
next decision (it is the track that exists to answer L11, which P5 carried forward rather than closing), and it
did need its own architect prompt: it arrived on 2026-10-06 and opened the stage above. B1 / private beta, RC,
GA, commercialization, a licence, signing, notarization, an updater and any new product verb, format, adapter or
crate still have no prompt. `AGENTS.md` 1 binds exactly as it did: an agent executes the task the pointer names
and stops there.

*(the paragraphs below are P5's running narrative from 2026-10-03 onward. Each keeps the words its own round
wrote, including the `IN_PROGRESS` states those rounds were required to hold; the present answer is the four
paragraphs above, and the live pointer is `.ai/ACTIVE_TASK.md`.)*

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
Head `53578e9` then went **10 of 10** too (`37143046338`), and its darwin set closed the index row from the
runner's own bytes: five lines, every one `OK`, exit 0, with one flipped byte in a copy of
`Contents/Info.plist` making the same index exit 1 and name the file. Comparing the two runs — heads
differing only in `scripts/` and docs — gave a fact nobody had predicted: the macOS `.app` tree digest is
identical across both, while the `.dmg` wrapping it, at identical length, is not.
All three repairs, the artifact sets with their checksums, and the limits of what has been proved are in
`P5_PACKAGING_REPORT.md` §5 through §5e. The packaged installer has since been run on this machine — §38 A–L
and §39 with the owner's live store parked, hashed and restored to three identical digests — and
`P5_INSTALL_RECOVERY_REPORT.md` records what that found: the window title still reads
`FirmwareSight - Analyze` on the Compare page, a reopened app shows an empty session while its store keeps
the whole build, and the uninstaller neither asks about user data nor deletes any.

**Commit C answered the first two of those.** §13 first-run guidance is one component read by two surfaces
(`GettingStarted.tsx`: the seven answers, shown as a dismissible panel in Analyze's empty state and repeated
on Help), §14 Help/About reports the identity of the running binary rather than a version the front end kept,
and the frozen title — L21 — is fixed on the Rust side over a closed five-variant page enum, so no new window
permission was needed and `capabilities/main.json` still grants the WebView exactly `core:default`. §15–§18
local History is now a fourth rail page over three new bounded storage **read** APIs with **no new
migration**: `crates/firmwaresight-storage/src/history.rs` pages builds, Gate runs and release records with
Rust owning the ceiling, the sort and which columns a filter may search — identity columns only, never the
stored intake directory, because a search box that answered "which folders hold firmware" would be a
question §16 forbids. Measured locally: `check.py` **16 of 16**, **812 Rust / 200 UI tests**, seven mutation
proofs, and page reads of 467.9µs / 541.5µs / 248.8µs over 100 builds / 100 runs / 50 releases
(`P5_VALIDATION/P5_ONBOARDING_HISTORY_REPORT.md`, design review in
`P5_VALIDATION/P5_ONBOARDING_HISTORY_DESIGN_CHECKLIST.md`). What that left open was the acceptance of these
screens rather than their code: §38 C had not been walked in an installed binary, and that head had no CI run
behind it. **Commit F2 closed the first of those two on 2026-10-05** — onboarding, History and Diagnostics were
each driven in the installed F1 artifact, History read back after its source project moved, and the export
proved to carry no path — while the stage's verdict stayed open, because only F3 may write it. **F3 wrote it on
2026-10-06: `P5 = PASS_COMPLETE`.**

**P5 closed `PASS_COMPLETE` on 2026-10-06 under Commit F3**, and what that verdict does *not* mean is written
where the verdict is: `P5_VALIDATION/P5_FINAL_CLOSURE_REPORT.md` §14, and the ledger entry above. The product
remains the G2-passed **MVP CANDIDATE** at baseline **0.6.0**, now with a productized engineering scope:
installable packages on three platforms, an installed Windows journey walked twice on CI-built artifacts,
onboarding, History, Diagnostics, migration behind a pre-migration snapshot, and a repository baseline that
verifies from Git index blobs. What is still forbidden and still unwritten: `BETA`, `RC`, `GA`, Production
Ready, a tag, a GitHub Release, a published installer, an updater, a certificate, a signature, a notarization
and any licence choice. **`PUBLIC OPEN-SOURCE REDISTRIBUTION CLAIM = BLOCKED BY OWNER LICENSE DECISION`** —
`license = "Proprietary"` stands, there is no root `LICENSE`, and a technically complete MVP candidate is not a
licensed open-source release. V1 own-artifact / real-user validation still has no prompt and stays
unauthorized: **P5's closure opens nothing.** V1 own-artifact / real-user validation still has
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
