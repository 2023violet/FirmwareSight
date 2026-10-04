---
title: "Current Project State"
doc_id: "FS-AI-002"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-04"
---

# Current State

- Date: 2026-10-04
- Baseline: **v0.6.0 — P0 Technical Foundation Baseline**
- Active task: **`P5_PRODUCTIZATION`** — stage **P5**, state **`IN_PROGRESS`**, opened on 2026-10-03 by
  *FirmwareSight — P5 Productization, Execution Prompt v1.0* (file, SHA-256 `722125f5…71e0ae`, 73,722 bytes,
  3,442 lines, archived in `10_AUDIT/SOURCE_PROMPTS/`). Its §4 required a productization audit before any
  product code, and that audit is written: **`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md`** answers §4 A–H
  from commands run on this tree at `d83175a` (HEAD = `origin/main`, tree clean, Run `37101619245` 7 of 7).
  The owner's checkpoint then answered the four questions the audit reserved for the owner — artifact
  versions **unify on `0.6.0`** with the workspace version as the single source, **migration `0005` only
  after a written `P5_MIGRATION_DECISION.md`**, the package matrix is **Windows with real install evidence
  on this host plus macOS/Ubuntu `CI_BUILD_ONLY`**, and the shipped store **keeps the name
  `firmwaresight-p0.sqlite`** with its path made visible rather than moved. One question stayed another
  person's and one has now been answered. **L22 (release identity versus line endings) is decided**: the
  draft that cost the three answers (`P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md`, option C = today's
  bytes-as-evidence semantics left in place, its premise pinned by
  `a_notes_file_that_differs_only_in_line_endings_is_a_different_release` and a mutation proof) went to the
  Architect and came back as **`ADR-0028`, Accepted: release identity uses the exact bytes observed on
  disk`** — `require_clean_git = false` stays valid with its caveat written down, `.gitattributes` is
  recommended for the *user's* repository, and this repository does not edit its own. The draft is
  re-statused `EXECUTION_RECORD`, not rewritten. **§54 still leaves the licence with the owner**, so
  `OPEN_SOURCE_LICENSE_DECISION = PENDING_OWNER_CONFIRMATION` stands in every P5 document. **No P5 verdict exists**: §5 forbids `P5 PASS`, `BETA`,
  `RC` or `GA` before closure evidence, `baseline_version` stays `0.6.0`, and no tag, GitHub Release or
  published installer is created.
- **What has landed since that checkpoint**, measured rather than summarized: `4cc8d93` opened governance
  and archived the prompt (Run `37125456689`, 7 of 7); `812b472` wrote `P5_MIGRATION_DECISION.md` then
  migration `0005`, which keeps the two numeric `Unknown` reasons the storage write path had been
  discarding (Run `37127791999`, 7 of 7); `9e3b1de` unified every artifact version on `0.6.0` and
  regenerated the seven goldens it moved (Run `37128593254`, **6 of 7 — failed**); `20b03e3` repaired the
  race that run lost, test-only (Run `37129900728`, 7 of 7); and the packaging commit turned `bundle.active`
  on, added the three CI package jobs prompt §41 asks for, wrote `scripts/verify_package_artifacts.py` and
  gave the gate a `drift/version identity` step (Run `37133706214`, **7 of 10 — failed**: the seven gate
  jobs were green and all three package jobs skipped their own build). The tree reached **775 Rust tests /
  160 UI tests in 6 files / `check.py` 16 steps** at that point, and the authoritative CI set is **10 jobs**
  for the length of P5 — recorded in `05_ENGINEERING/06_CI_CD_BASELINE.md` and tracked run by run in
  `P5_VALIDATION/P5_CI_AUTHORITY.md`.
- **The first 10-job run went red because the gate reported a pass it had not earned.** `cargo install
  tauri-cli` leaves a binary named `cargo-tauri`, which is run as `cargo tauri`; the package group probed
  the bare name `tauri`, found nothing on the three runners that had installed the CLI one step earlier,
  printed `SKIPPED: the Tauri CLI is not installed on this machine`, summarised `4/4 steps passed` and
  exited 0 — and each job was then caught red by its own artifact upload. Two rules replace the instance
  fix: the group probes the forms in the order the install methods produce them and runs the one that
  answered, and `check.py` now records a skip as `SKIP`, leaves it out of the passed total, and exits
  non-zero on any skip when `CI` is set. Measured on this host three ways, in `P5_VALIDATION/
  P5_PACKAGING_REPORT.md` §5.
- **Packaging is proved on all three platforms, and the index defect that read-back found is now closed by
  its own read-back.** Run `37138881977` at head `1055242` is the first **10 of 10** green run and the first
  to attach a built package: the three §42-named sets — Windows NSIS, Ubuntu `.deb`, macOS `.app` + `.dmg` —
  each with its own `SHA256SUMS.txt` and `artifact-metadata.json`. They were downloaded with `gh run download`
  and read back, which closed the rows that had only ever been read from upstream source: the macOS runner
  derives an `.icns` from the five committed PNGs, the executable inside `Contents/MacOS` is the Cargo bin name
  (`firmwaresight-desktop`, exactly as the source said), the `.deb` really is unpacked by `dpkg-deb -x`, and
  all three jobs build from `apps/desktop`. That read-back also caught a defect of this repository's own: the
  darwin index gave the `.app` one line naming a directory, so `sha256sum -c` answered
  `FAILED open or read` on a good build. A directory bundle now contributes one line per file, with the
  aggregate tree digest kept in the metadata as identity and each scanned payload's digest recorded — and
  run `37143046338` at head `53578e9`, again **10 of 10**, said it back: five darwin index lines, every one
  `OK`, exit 0, with one flipped byte in a copy of `Contents/Info.plist` making the same index exit 1 and name
  that file. Comparing the two runs, whose heads differ only in `scripts/` and docs, produced a fact nobody
  predicted: the macOS `.app` tree digest is identical across both, while the `.dmg` wrapping it — same
  length — is not. A repeating digest is one pair of runs on one runner image and licenses no reproducibility
  claim; a changing one is not evidence the program changed. The installers for §38 exist on both this host
  and the runner, and the Windows one has now been run here — see the next bullet.
- **The first real install ran on this machine, and it produced three findings that no source read could.**
  Windows Sandbox is absent here (`WindowsSandbox.exe` missing, the feature query needs elevation, which this
  round does not take), so prompt §38's fallback ran: the owner's live store was hashed, parked, and restored
  to three byte-identical digests, and the packaged installer was driven through its own UI. Install, launch
  from the Start Menu shortcut, Analyze of a real fixture, close/reopen, repair, uninstall, reinstall and the
  data-retention check are all transcribed in `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md` with §39's record
  (per-user install path, no UAC prompt, `PATH` entries mentioning FirmwareSight: 0 before and after, no shell
  profile, no TCP endpoint owned by the app, WebView origin `http://tauri.localhost/`). What it found: the
  window title still reads `FirmwareSight - Analyze` while the page is Compare — L21 reproduced on the
  packaged artifact; a returning user sees nothing they did before, because the store kept 1 build, 19
  sections, 42 symbols and 10 evidence rows across a reopen while the UI says "Nothing has been analyzed in
  this session yet"; and the uninstaller never mentions user data, deletes none, and the reinstall reopened
  the same store without re-running the migrations — a retention rule that must now be written where a user
  reads it. §38 item C (first-run onboarding) was not built when that round ran, so §64's full journey stayed
  open until the next bullet, and §65's negative half stays open because this host has the whole toolchain
  installed.
- **Commit C closed §13, §14 and §15–§18 in the code, and closed L21 with them.** First-run guidance is one
  component read by two surfaces (`GettingStarted.tsx`: the seven answers as a list, plus a dismissible panel
  Analyze shows in its empty state), Help is a page whose identity block is asked from the running binary
  rather than kept in the front end, and History is a fourth rail page over three new bounded storage reads —
  `list_history_builds`, `list_history_gate_runs`, `list_history_releases` — with **no migration added**,
  which is what §18 asked for first. The window title the install round found frozen is fixed on the Rust
  side: `set_window_title(MainWindowPage)` composes the text from a closed five-variant enum, because
  granting the WebView `core:window:allow-set-title` would be a capability change (`AGENTS.md` 9) and a
  channel from a file name into a window property; `the_title_fix_took_no_new_capability` reads
  `capabilities/main.json` and asserts the window still holds exactly `["core:default"]`. Two decisions were
  made from evidence rather than habit: a History filter searches **stored identity columns only**, because
  matching `artifacts.path` would turn the search box into a directory oracle (the test that seeds real builds
  into a real temporary directory asserts filtering by that directory's name returns zero rows), and Help
  names the store **file** and not its folder, because the path is a Diagnostics question with its own
  allowlist and Diagnostics is not in this commit. Measured: `python scripts/check.py` **16 of 16**, the tree
  now **812 Rust tests / 200 UI tests in 8 files** (19 storage + 11 desktop + 24 History UI + 16 Help UI),
  and the page reads 100 builds / 100 runs / 50 releases in 467.9µs / 541.5µs / 248.8µs on the release
  profile. Seven mutation proofs are in `P5_VALIDATION/P5_ONBOARDING_HISTORY_REPORT.md` §5 and the design
  review, including the one disclosed non-token measurement, in
  `P5_ONBOARDING_HISTORY_DESIGN_CHECKLIST.md`. **What this does not close:** the page has not been operated
  in an installed binary — §38 C's onboarding and the whole §64 journey still have to be walked against the
  package, which is the remaining install acceptance. The remote said 10 of 10 on the first attempt for
  this head (Run `37154946484` on `e863d0c`), so the new page's UI tests and the rail guards have run on
  both Windows and Ubuntu runners.
- **One gate run went red for a reason that was not in the commit it ran on**, and that is worth knowing
  before the next stage trusts a green. Run `37128593254` failed `Desktop UI (windows-latest)` on a
  pre-existing race in `compare.test.tsx:1094`: the `Section Changes` region mounts before its first page
  does (`Compare.tsx:1001-1004` clears the page, the `<table>` renders only from a resolved one), so a
  synchronous `getAllByRole('columnheader')` after `findByRole('region')` waits for nothing. It reproduced
  on this host without CI (1 of 30 fresh runs) and 3 of 3 under 25 ms of injected mock latency; the repair
  is one awaited query, and a never-resolving query keeps the repaired test red, so the wait is
  load-bearing rather than a widened timeout. `055b54e` had closed the identical shape at this file's pager
  and said the rest was "not converted then" — this is the third such close (`e83950c` was the first). The
  remaining instances are a sweep for P5's validation work, not an excuse to rewrite unrelated tests.
- **The sweep happened, and the class produced one more instance first.** Eight commits after `20b03e3`, on a
  tree whose UI source was identical to a head CI had passed 10 of 10 twice, the **local** full gate went red at
  `frontend/test`: L23's sixth instance, in the same file. `compare.test.tsx` awaited the ranking region and then
  read the *added rows* synchronously, though those come from a second IPC wave that commits on its own
  (`Compare.tsx:217-256`, with its own `Reading the added rows…` branch at `Compare.tsx:876-877`). Three full-suite
  runs on an idle host passed; the gate, loaded by its own compilation, did not — which is why the record says the
  wait is load-bearing rather than the reproduction being the proof. Repaired test-only (one awaited query) plus a
  contract test that holds that wave open; three mutation proofs; 20 clean fresh-process runs of the changed suite;
  `Compare.tsx` untouched. §0a of the audit now carries the bounded sweep of chained IPC waves that §0a had asked
  for, the five synchronous top-level reads that remain after it, and the sweep's own limits written next to it:
  Release's `run`-keyed waves were not walked assertion by assertion, `within(region)` reads inside already-awaited
  regions were not audited, and the repetitions cover the changed suite rather than every suite — so L23 stays
  `SHOULD_CLOSE_P5` and the complete sweep is a named remaining task. Present counts on that tree: `check.py`
  **16 of 16**, **813 Rust / 201 UI in 8 files** — and the remote agreed: Run `37158606478` on `111fe32` came back
  **10 of 10** at its first attempt, with `Desktop UI` green on **both** Windows and Ubuntu runners, which is where
  an awaited second-wave query earns its keep.
- **Commit D landed the recovery half, and the installed walk is what made it honest.** Storage now owns its
  own health: `integrity_check()` asks the engine and reports, and repairs nothing — no `PRAGMA` write, no
  VACUUM, no rebuild, which is what lets the Help screen say a damaged store will not be touched. Any older
  file-backed store is now snapshotted with SQLite's **online backup API** before it is migrated (a plain
  file copy is wrong under WAL and `VACUUM INTO` binds its filename as text, so a non-UTF-8 path fails or
  names the wrong file), the snapshot is opened and health-checked before it is renamed into place, and a
  snapshot that cannot be written stops the upgrade rather than starting it. Diagnostics is a closed
  31-field allowlist assembled in Rust and exported through a native dialog that opens only when a person
  names a folder; the exported file on the packaged build contains **no `/` and no `\` character at all**,
  and the store is named by file, never by directory, on screen and in the file.
- **What that walk found is the reason it was run.** Two defects, neither reachable from a unit test:
  the startup refusal exited **101 through a Tauri panic** instead of the typed exit 1, because a `setup`
  error is panicked by the framework inside its event-loop callback
  (`tauri-2.12.0/src/app.rs:1443-1445`) and the `Error::Setup` arm written for it was unreachable — and a
  file that is not a database was being described as a rolled-back migration to "schema version 0", a step
  that never ran. Both are fixed in this commit, the second with the seventh mutation proof. It also
  falsified two sentences this repository had written for itself: a `setup` failure does *not* return
  before the event loop starts, and a window *is* mapped — measured visible from t=20 ms until the process
  leaves at t=360 ms — so §22's stop is now described as it measures (a window with nothing true to say)
  rather than as it was reasoned (no window at all). And `payload_sha256` never was the digest of the file
  a user launches: the bundler overwrites 3 bytes of it to name the bundle type
  (`tauri-bundler-2.10.1/src/bundle.rs:41-95`), which is why the installed binary reports
  `installChannel: "nsis"`. Present counts: `check.py` **16 of 16**, package group **4 of 4**,
  **854 Rust / 210 UI in 8 files**. The owner's store was parked, backed up on a second volume and restored
  byte-identically at both cycles; `ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`.
- **The remote found a third, and it was in Commit D's own test.** Run `37200245520` on the product head came
  back **8 of 10**: `macOS Core Smoke` and `Rust (ubuntu-latest)` each panicked on one assertion in the
  snapshot-retention test this commit added, which compared two whole snapshot files whose only available
  difference was the **second-granular** `applied_at` default on `schema_migrations` — a clock, not a fact. The
  same test was green on this host and on `Rust (windows-latest)`, so nothing about the product was in
  question and all three package jobs passed on that head. Rejected: a sleep, a retry, and deleting the claim;
  the repair marks each store with a named row and reads that row back out of the standing snapshot, proven
  non-vacuous by an eighth mutation (`backup.rs` keeping a stale snapshot reddens it alone), then 20
  fresh-process runs of the repaired test and a local `16 of 16` + `3 of 3` core-smoke. The failed head is
  listed in `P5_CI_AUTHORITY.md` rather than replaced, and **the repair head's own run still has to be read
  back before any sentence calls Commit D green.**
- **A limitation surfaced by that record, and deliberately not fixed: L26.** `SHA256SUMS` is generated from
  working-copy bytes, so with `core.autocrlf=true` and `.gitattributes` `text eol=lf` 17 entries verify on the
  host that wrote them and disagree with a clean checkout elsewhere; measured identically at `3400981` and at
  the repair head, so it predates this round. No CI job runs `verify_baseline_artifacts.py`, which is why nothing
  has ever contradicted it. It is recorded with its measurement in `P5_PRODUCTIZATION_AUDIT.md` §G, and choosing
  which bytes the artifact means is Commit F's decision, not a task to pick up between other work.
- The task before it, kept as history: the G2 Product MVP engineering closure audit reached **PASS** on
  2026-10-01: **Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE.** Authorized by *FirmwareSight — G2
  Product MVP Engineering Closure Audit, Execution Prompt v1.0 — Architect Reviewed* (file, SHA-256
  `3c6ab83e…2bac9`, archived) and its inline *Storage Path Semantics Clarification Addendum v1.0*. Product
  tree `e35cfe7` (`055b54e` + the G2-F1 test-only fix) on Run `36906482900`, evidence head `f75cbc5` on Run
  `36948719972`, each 7 of 7 on the first attempt; 769 Rust / 155 UI / `check.py` 15/15 / clean worktree
  17/17; the whole chain on both surfaces with byte-identical parity; evidence in `G2_VALIDATION/`. At the
  time it closed, the next tracks (V1, P5) needed a separate architect decision and nothing there
  authorized one — P5 has since arrived with its own prompt, and V1 still has not.
- Post-G2, and the reason the source above is no longer the whole picture: the product then passed a
  **real-desktop MVP end-to-end acceptance** round on 2026-10-02 — 283 cases, black-box mouse, keyboard
  and native dialogs against the shipping binary — verdict **`PASS_WITH_FINDINGS`**, three findings, no
  S0 and no S1. Its evidence root lives outside the repository, under `%TEMP%`, and the owner's live
  store was restored byte-exact. A follow-on round, *Post-G2 E2E Findings Remediation & Focused
  Re-Validation, Execution Prompt v1.0 — Architect Reviewed*, then fixed all three, narrowly:
  **E2E-F001** (S3) — a surviving analysis did not say which selection it described, so choosing a
  second artifact of the same file name left the first one's figures looking like the new one's;
  **E2E-F002** (S2) — a destination folder that is not a bundle this engine wrote was offered for
  replacement above an engine sentence claiming it already holds one, a decision the engine refuses to
  carry out; **E2E-F003** (S3) — a GNU ld MAP whose banner sat past a 4 KB head window was refused as
  another linker's output. Fixes are `e816dcb` and `971015f`: 770 Rust / 159 UI / `check.py` 15/15,
  fix head green on Run #43 `37100371601` at 7 of 7 on the first attempt, evidence in
  `POST_G2_E2E_REMEDIATION/`. **G2 stayed PASS, the product stayed MVP CANDIDATE, the baseline stayed
  0.6.0, and that round created no P5 authority of its own** — P5 arrived the next day with its own
  prompt, recorded above. What the remediation deliberately did not take on is
  recorded there too: large-file latency and peak RSS stay carried forward with the wording
  `PARTIAL / environment-sensitive` and `MEASURED FOR TESTED WORKLOAD`, and 125/150 % DPI, mouse-wheel,
  the `update_goldens` issue and the licence choice are all still open.
- Previous task, kept as history: `P4_RELEASE_BUNDLE` reached `PASS / COMPLETE` on 2026-10-01 and the
  pointer returned to empty. P4 was opened on 2026-09-30 by *FirmwareSight — P4 Release Bundle MVP
  Implementation, Execution Prompt v1.0 — Architect Reviewed*, delivered as a file (the first stage prompt
  since P0 to arrive that way) and archived with its measured SHA-256
  `1baaec9204a1d2aa5aa53bd735b34d376ee56db7557a04d6abb79265c30840c5` under `10_AUDIT/SOURCE_PROMPTS/`.
  Its acceptance list was the frozen **US-004 Export Bundle** criteria at
  `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:42-49`, plus **PRD P0-6** at `01_PRODUCT/01_PRD_MVP.md:85-95`
  and `01_PRODUCT/01_PRD_MVP.md:126`'s independent-readability requirement. Evidence: `P4_VALIDATION/`.
  The only G2 statement the stage was allowed to write is the one it wrote:
  `READY_FOR_ENGINEERING_GATE_REVIEW` — never `PASS`.
  `P3_RELEASE_GATE`, `P2_COMPARE` and `P1_ANALYZE_DETAILS` closed earlier, on 2026-09-30, 2026-09-29 and
  2026-09-29, in `P3_VALIDATION/`, `P2_VALIDATION/` and `P1_VALIDATION/`.
  **The P4 start fact is `323afad`, one commit ahead of the `ba5e59e` the prompt's §0/§2 names.** §2's
  "if remote is newer, inspect and reconcile before writing" was run: `git diff --name-only ba5e59e
  323afad` moves eleven documentation, governance and integrity files plus two newly tracked
  baseline-artifact scripts, and no product source, fixture, schema or migration, and Run `36810689645`
  on `323afad` is `completed` / `success` / **7 of 7 jobs** (`gh run view 36810689645
  --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs`). Start counts re-measured
  before the first write: one `cargo test --workspace` = **556 passed / 0 failed / 0 ignored**,
  `corepack pnpm test` = **135 passed in 6 files**.
  At P4's closure the G2 engineering closure audit was the next authorizable act and P4's own prompt
  stopped at P4; that audit has since run and closed `PASS` (above).
  Carried forward from P3, still true: `origin/main` sat at
  `219178af195569ec6b13728d84d992ef78df8c04`, where Run `36774472141` concluded `failure` on one job of
  seven — `Dependency policy`, reporting `error[yanked]` for `yoke-derive 0.8.3`, a transitive proc-macro
  that crates.io yanked the same afternoon, after the local `deny` step had passed against an older
  index. Rust on both platforms, Desktop UI on both platforms, the macOS core smoke and the drift check
  passed remotely, and the lockfile moved to `0.8.4` in a successor commit. The prior head was `32b23aa`
  on Run #19 `36665007523`, success, 7 of 7 jobs, following Run #18 `36648718199` on the P2 implementation
  tree `4a77ea1` and the mid-round failure Run #17 on `c7fc2a3` — the fixture-pair story in
  `P2_COMPARE_EXECUTION_REPORT.md` §5.1. Every P3 gate number in this baseline is still a locally measured
  number; the remote result is recorded separately rather than folded into them.

## Product/architecture baseline

The v0.5.0 product, architecture and design decisions remain inherited unchanged. v0.6.0 adds the
validated P0 technical foundation; it does not redefine the product, does not add a fifth product
verb, and does not change MVP scope. Nothing in the frozen baseline was renegotiated to build the
slice, and nothing was renegotiated to promote it.

Four governance changes are in force now, all decided by the architect rather than by the coding side:

- **ADR-0025** supersedes exactly one clause of ADR-0020 (`P1 Product MVP implementation 只有两者都 PASS
  后开始`). After `P0 PASS` the architect may authorize a bounded, reversible Pre-G1 Analyze slice.
  Its Batch A precondition on the *next* slice was superseded the same day by ADR-0026 below; the slice
  it authorized, P1-A0, is complete and its record stands.
- **ADR-0026** (2026-09-29) makes this an **open-source MVP-first delivery**: MVP proceeds
  P1 → P2 → P3 → P4 → G2 on engineering grounds, `G1 = P0 PASS` for this delivery, V0 becomes
  `NON_BLOCKING_USER_FEEDBACK_TRACK`, and pricing / willingness-to-pay / pilot signals leave the gates.
  It supersedes sequencing only: no V0 artifact, P0 evidence, P1-A0 evidence or technical safety boundary
  is relaxed, and it authorizes no cloud, account, telemetry, AI, updater or licensing work.
- **Design tokens `0.2.0 → 0.2.1`**, adding one numeric semantic (`border.width.hairline = 1`) to close
  the documented P0 gap. No color, spacing, radius, typography, layout, motion, shadow or status value
  changed, and the focus ring keeps its own 2px token.
- **ADR-0027** (2026-09-29) is P3 prompt §6 written down as an architecture record rather than
  re-derived in code: a fifth first-party library crate, `firmwaresight-project`, owns
  `firmwaresight.toml`, project-local release evidence, the read-only system Git adapter and the
  deterministic Gate input fingerprints, because CLI and Desktop need one implementation of those
  semantics and Core must stay filesystem-free, Git-process-free and dependency-free. The boundary,
  the prohibited dependencies and the short admission list were specified by the architect in the
  prompt; the coding side's role was to record them, measure the locked versions and licenses, and not
  install anything the requirement does not need. It authorizes no Gate rule semantic outside Core, no
  artifact parsing inside the adapter, and no network.

## P0 — Technical Vertical Slice

Status: **`PASS`** · promoted to v0.6.0 by the architect-signed
*P0 Final Promotion / v0.6.0 Baseline Closure v1.0* · Task at promotion time: **NONE** (recorded as
`active_task: NONE` in `BASELINE.yaml`; the task live on 2026-09-29 was `P1_ANALYZE_DETAILS`, then after it
closed the field read `NONE` for a few hours until the P2 prompt arrived the same day — it read
`P2_COMPARE`, returned to `NONE` when P2 closed, and reads `P3_RELEASE_GATE` now. Each move changes the
pointer and not this frozen P0 verdict)

The chain P0 claimed is proven by executed commands, on this machine and on GitHub's runners:

```text
Real ELF + GNU ld MAP → guarded intake → parse → normalize → evidence
→ memory accounting → BuildSnapshot → deterministic CLI JSON → SQLite → typed IPC → Desktop summary
```

Exactly four Phase-0 library crates, with `firmwaresight-core` declaring zero dependencies. 104 Rust
tests and 19 UI tests as promoted; the tree now runs 142 and 31, and `P1_A0_VALIDATION/` accounts for
every test after those. SQLite schema version 2, including the v1→v2 migration. Deterministic CLI JSON
reproduced byte-identical. Memory accounting reproduced by hand from `readelf` and the MAP region
table. A 512 MiB guard measured from both sides of the boundary. A typed ts-rs IPC boundary whose
generated TypeScript is drift-checked. A real Windows desktop window opened and driven in the
shipping configuration.

### Remote verification

| Run | HEAD | Conclusion | Jobs |
| --- | --- | --- | --- |
| #1 `36360310447` | `f9b8ccb` | `failure` | 2 of 6 green — Rust (win), Rust (ubuntu), Generated output drift, Dependency policy red |
| #2 `36378384225` | `ebda52d` | `failure` | 6 of 7 green — only `Generated output drift` red (Ubuntu provisioning in a second job) |
| #3 `36399805005` | `1cd6309` | **`success`** | 7 of 7 — engineering closure |
| #4 `36402637251` | `5e58f77` | **`success`** | 7 of 7 — pre-promotion revalidation of the governance-only successor |
| #5 `36416146281` | `738ae78` | **`success`** | 7 of 7 — revalidation of the v0.6.0 promotion commit itself |
| #6 `36419864513` | `7d2f38a` | **`success`** | 7 of 7 — revalidation of the baseline consistency-closure commit; the last remote fact of the P0 chain, and still `last_remote_ci` inside that block |

Read with `gh run view 36419864513 --repo 2023violet/FirmwareSight`, not from this file. Run #6's
matrix: `Rust (windows-latest)`, `Rust (ubuntu-latest)`, `Desktop UI (windows-latest)`,
`Desktop UI (ubuntu-latest)`, `Generated output drift`, `Dependency policy`, `macOS Core Smoke` —
all `success`. The consistency-closure commit is on `origin/main`, so the baseline record is green on
its own commit, not only on the commit the architect read.

The two failures are kept as history rather than edited. They are the reason the four successes that
followed mean anything: Run #1 exposed a checkout that could rewrite committed evidence bytes,
a runner with no
Tauri Linux prerequisites, a drift assertion that compared third-party encoder output instead of
pixels, and a supply-chain policy the pinned tool could not parse; Run #2 then exposed that a second
job compiled the desktop crate without the prerequisites its sibling had been given.

### What the two remediation rounds changed

| Cause | Fix | Confirmed |
| --- | --- | --- |
| Checkout changed fixture bytes | `.gitattributes` text policy (`*.ld text eol=lf`, `*.map -text`, binaries unchanged); one manifest value moved to the blob's own hash | Runs #2, #3, #4 |
| Ubuntu runner lacked Tauri prerequisites | the documented apt block, `if: matrix.os == 'ubuntu-latest'`, desktop crate still compiled | Runs #2, #3, #4 |
| Icon check compared encoder bytes | decode and compare pixels; missing ICO frame still fails | Runs #2, #3, #4 |
| `deny.toml` unparseable by 0.20.2 | rewritten to the keys the tool accepts; allow list from `cargo deny list`; bans evaluated over the four shipping targets | Runs #2, #3, #4 |
| No macOS job | `macos-core` running `check.py --only core-smoke` on `main` pushes | Runs #3, #4 |
| `drift` job lacked the same prerequisites | the proven apt block copied into that job — no abstraction for two jobs | Runs #3, #4 |

A sixth defect was found by the authorized desktop launch rather than by CI: `evidence` had a
whole-table primary key while `04_TECH/15` §4 declares `Build 1─N Evidence`, so a second artifact
could never be stored. Migration `0002` keys evidence by `(build_id, id)`; the two failing tests were
written before the fix; the upgrade was replayed against the real user-profile database.

## V0

Status: **`NON_BLOCKING_USER_FEEDBACK_TRACK`** since ADR-0026 (2026-09-29). The sample state is
unchanged and is still an honest zero.

Formal eligible external sessions: `0 / 8 minimum`, `0 / 4–5 Batch A target`.

Completed: clickable prototype, fixture/state machine, protocol/session templates, internal
functional dry run, Batch A takeover, Batch A recruitment-ready package, Batch A activation.

No CI run, no green gate and no promotion signature can supply the missing input, which is real human
participants; the coding side authors none, and none were authored. What changed on 2026-09-29 is the
consequence of that zero: it no longer blocks P1, P2, P3, P4 or G1. The activation round earlier the same
day had already verified the recruitment pack and written nothing else; `ADR-0026` then moved the track
off the critical path, and the price-anchor prompt that would have followed it was
**`WITHDRAWN_BY_ARCHITECT`, never executed**.

The instrument stays frozen and reusable on purpose: `V0_VALIDATION/prototype/` at `v0.1.0` with
`protocol/TASK_SCRIPT.md`, `sessions/TEMPLATE.md` and both registers, so a later feedback round remains
comparable with the protocol that already exists. What is now unverified rather than merely deferred is
the thing V0 existed to check: whether a real firmware engineer distinguishes `Unknown` from `PASS`,
understands why a MAP is requested, or reads a `Review` correctly. That risk is carried forward and
recorded in `ADR-0026`'s Consequences, not argued away here.

## Gates

```text
G0: PASS
P0: PASS — promoted to the v0.6.0 Technical Foundation Baseline; frozen, no further P0 closure prompts
G1: PASS — basis is P0 PASS under ADR-0026 (2026-09-29). Before that date this file read `NOT CLAIMED` against `G1 = V0 PASS + P0 PASS`, and the historical records still say so
V0: NON_BLOCKING_USER_FEEDBACK_TRACK — 0 of 8 eligible external sessions, an honest zero that gates no P-stage and no G1
Pre-G1 (ADR-0025): P1-A0 REAL ARTIFACT INTAKE — COMPLETE, including its evidence-identity and persistence correctness closure
P1: PASS / COMPLETE — the Analyze verb as one product verb: intake, summary, top contributors, bounded Sections / Symbols / Evidence details, the Evidence Inspector, and the US-001 bytes/KiB presentation switch. Evidence: P1_VALIDATION/
P2: PASS / COMPLETE — Compare over persisted snapshots: Core-owned deterministic diff, bounded Compare IPC with a session-local registry, the second desktop page, `fwsight diff`, and portable Diff JSON v1 plus self-contained HTML. Evidence: P2_VALIDATION/. The round's own measurements are LOCAL PASS; the pushed head `4a77ea1` is green on Run #18 `36648718199` (7 of 7), which closes defect E — the fixture half `.gitignore` hid — on the remote too. Open on purpose: desktop smoke step 27 was not observed in the shipped window
P3: PASS / COMPLETE — Release Gate, authorized 2026-09-29 by *P3 Release Gate MVP Implementation, Execution
Prompt v1.1* and by ADR-0027, closed 2026-09-30. Evidence: P3_VALIDATION/. The round's own gate numbers are
LOCALLY measured; what the remote then said is recorded separately in that pack, and P3 is SEALED — no
further P3 documentation-only successor.
P4: PASS / COMPLETE — Release Bundle, opened 2026-09-30 by its own architect prompt (v1.0, delivered as
a file and registered with its SHA-256), closed 2026-10-01. Evidence: P4_VALIDATION/. The round's own
numbers are LOCALLY measured — one `cargo test --workspace` = 769 passed / 0 failed, UI 155 in 6 files,
`check.py` 15/15, the fifty-step shipped-window smoke, the independent reader at 64/64 and 59/59 — and
the remote result is recorded separately: the final implementation head `e799f2f` is green on run
`36872456446`, 7 of 7 jobs. P4 is the last core product-implementation stage of the MVP line.
G2: PASS — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE (2026-10-01). The whole-MVP engineering
closure audit, under its own prompt and storage-path addendum; evidence G2_VALIDATION/; product tree
e35cfe7 and evidence head f75cbc5 each 7 of 7 on the first attempt. Not productization, beta, RC or GA
Open-source licence: OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION — a technical MVP candidate is not a licensed open-source release
Pricing / willingness-to-pay / team-pilot signal: DEFERRED_POST_MVP; the price-anchor prompt was WITHDRAWN_BY_ARCHITECT and never executed
Active task: NONE — G2 closed 2026-10-01; V1 and P5 each need a separate architect decision
Design tokens: v0.2.1
```

`ADR-0026-open-source-mvp-first-delivery.md` is what changed here. It re-bases G1 on `P0 PASS` for the
current open-source MVP and moves V0 off the critical path, superseding the sequencing conclusions of
ADR-0020 and ADR-0025 while leaving their decisions, evidence and safety boundaries in place. G1's
technical content in `06_DELIVERY/06_STAGE_GATES.md` is unchanged and still has to hold; what was removed
is the requirement that a moderated human panel exist before more of the product can be built. That is a
real cost, recorded in the ADR's own Consequences section: whether users distinguish `Unknown` from
`PASS`, or understand why a MAP is asked for, is now unverified rather than deferred.

Full reasoning: `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md`; the promotion act:
`P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`; per-item evidence:
`P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`; run history: `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`;
the two remediation rounds: `P0_CI_REMEDIATION_REPORT.md` and `P0_CI_RUN_2_CLOSURE_REPORT.md`; the
shipped-window result: `P0_DESKTOP_SMOKE_REPORT.md`.

## Known gaps that promotion does not remove

- **Peak RSS: `NOT MEASURED`**, with the reason and the measurements that do exist in
  `P0_PERFORMANCE_REPORT.md`.
- **Fuzz campaign: `NOT RUN`** — `cargo-fuzz` needs a nightly toolchain the tool policy did not
  authorize. The no-panic claim rests on regression tests over malformed fixtures.
- **Two RustSec advisories accepted, not resolved**: `RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and
  `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`, unmaintained) enter through the `gtk-rs 0.18` line
  Tauri 2.12.0 requires and have no compatible upgrade inside it. Disposition: accepted as explicit
  P0 transitive risk, not a promotion blocker, with five revisit triggers. The dependency policy is
  "PASS under a documented policy with two explicitly accepted transitive advisories" — not
  "zero vulnerabilities" and not "security clean".
- **Desktop smoke: one Windows host, one WebView2 version, 100% scaling.** No Windows 11, no other
  DPI, no Linux or macOS window, and the loading state stayed a test-only claim (recorded NOT
  OBSERVED).
- **Two linker layouts tested.** Executable-in-RAM, external SRAM, DMA pools, overlays and
  MPU-aligned sections are unproven; DWARF is recognized but not consumed.
- **Two stored facts carry no reason.** `sections.file_offset` and `symbols.address` are nullable with
  no companion `*_unknown` column, so a reason cannot survive the write. The read layer types them as a
  plain option and the UI says `Unknown` with no invented explanation. Closing this needs a migration,
  which P1 deliberately did not write; found and reported in
  `P1_VALIDATION/P1_ANALYZE_EXECUTION_REPORT.md` §6.
- **One dead `Apply filter` click is unresolved.** It happened once on an intermediate build and could
  not be reproduced on the shipped binary or in the automated reproducer. The leading hypothesis is the
  WebView2 autofill popup, and `autoComplete="off"` removes that surface, but no cause is established.
  Recorded as unresolved in `P1_ANALYZE_DETAILS_SMOKE_REPORT.md` §6 rather than written up as fixed.
- **Icon provisioning duplication is a real cost.** Two jobs copy the same apt block; a third job that
  compiles the shell on Linux is the trigger to extract it.
- **Every gate step except one reads the working tree, not the index.** That blind spot is how P2's
  fixture pair survived eight commits with only half of it in the repository: `.gitignore`'s
  `**/target/` matched `fixtures/elf/p2-diff/target/`, the manifest recorded all twelve paths, and the
  hash test happily digested the files the generator had left on disk. CI caught it because CI clones.
  `drift/fixtures tracked` now closes the case for anything a manifest lists; a source file ignored by
  accident is still unchecked, because no manifest names it. Recorded in
  `P2_VALIDATION/P2_COMPARE_EXECUTION_REPORT.md` §5.1.

## Version rule

v0.6.0 exists because P0 reached `PASS` with real engineering evidence measured on three platforms,
signed by the architect. The rule that produced it stays in force for the next promotion: a failed run
must not be relabeled into a PASS baseline, a locally green gate must not be relabeled into a CI PASS,
and a green CI does not equal an architectural promotion. `v0.7.0` and G1 claims will require their own
measured evidence and their own signed prompt.

## Integrity record

`DIRECTORY_TREE.txt` and `SHA256SUMS` are the **v0.6.0** baseline artifacts, regenerated by the promotion
round after every document change and gate run, in that order: the tree lists the tracked layout rooted
at `FirmwareSight_Project_Baseline_v0.6.0/`, and the manifest hashes each baseline-controlled tracked
file with a lowercase SHA-256, LF line endings, lexicographic path order and no self-entry. Both are
produced by `scripts/generate_baseline_artifacts.py` (`tree`, then `sums`) and re-checked by
`scripts/verify_baseline_artifacts.py`, which shares no code with the generator: it re-derives the
tracked set from `git ls-files` and hashes the files straight off disk, because the manifest cannot
certify itself. `sha256sum -c SHA256SUMS` is the third, external pass. This tooling used to live under
gitignored `target/`, which meant a fresh clone could not regenerate either artifact; it is tracked now.
`manifest.txt` is left exactly as the v0.5.1 delivery wrote it - the frozen package list, not
this tree - and `P0_FINAL_PROMOTION_REPORT.md` §10 records the commands and the counts. P4's closure
regenerated both artifacts after the tracked path set grew by this stage's fixtures, golden bundle,
tests, scripts and validation pack — `DIRECTORY_TREE.txt` first and `SHA256SUMS` last, as the order
requires — and the closure commit records the checker's zero-mismatch result.

**How to read a `file:line` citation in this repository.** A closed validation pack is evidence about the
tree it measured, so its line numbers are as-of-writing, not live: `App.module.css` and `App.tsx` have
both shrunk since P0, and several P0/P1/P2 checklist lines now point past the end of those files. Those
packs are not rewritten to match current code - that would turn a record into a claim. Within a *living*
document, cite the key or the symbol rather than the line number, because a line number goes stale the
moment an adjacent line is added: P3 inserting `crates/firmwaresight-project` into the workspace member
list pushed `license = "Proprietary"` from `Cargo.toml` line 17 to line 18 and silently invalidated every
citation written before it. The current stage's pack is the one place a line pointer is still checkable.

## Next work

**`active_task: P5_PRODUCTIZATION`, stage P5, state `IN_PROGRESS`, opened 2026-10-03.** The G2 engineering
closure audit closed `PASS` on 2026-10-01, so the MVP engineering candidate is complete and the product
state is still **MVP CANDIDATE** at baseline `0.6.0`. `AGENTS.md` 1 still means what it has meant in every
round: the pointer names ONE stage, and no agent lifts the next one off the roadmap. P5 arrived the
legitimate way — its own architect prompt, delivered as a file and archived with its hash — and it may run
only inside that prompt: no P5 verdict may be written before closure evidence, no tag, GitHub Release,
published installer, signing, updater or licence choice, and V1 own-artifact / real-user validation still
has no prompt.

**`G2` closed `PASS` on 2026-10-01, under its own architect prompt and addendum.** Audit-first: the gate
ran before anything was written and found G2-F1, a test race fixed test-only in `e35cfe7`; then the whole
MVP was driven through the CLI twice and the shipping window once, the clean tracked tree was proven in a
detached worktree, the failure paths were forced, and the path boundary was proven in 17 checks after the
architect adjudicated `artifacts.path` as expected local-only storage. Per-item verdict:
`G2_VALIDATION/G2_EXIT_CHECKLIST.md`; the canonical known-limitations list:
`G2_VALIDATION/G2_KNOWN_LIMITATIONS.md`.

**`P4 Release Bundle` closed `PASS / COMPLETE` on 2026-10-01, under its own architect prompt.** One
product verb, `Bundle`, over results the earlier stages had already computed and stored: a release
plan — id, version, ten-file list with digests — assembled before any byte is written; a staged copy
of the composed documents and the current artifacts; verification of the staged bytes against their
own `SHA256SUMS` and manifest; publish by rename; replacement only under an explicit confirmation and
only of a destination this engine wrote, with rollback if the swap fails; and a `release_records` row
written after the bytes, never before. Per-item evidence:
`P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md`; what was built, every gate number, the CLI smoke and the
CI runs: `P4_BUNDLE_EXECUTION_REPORT.md`; the shipped-window result over all fifty §59 steps:
`P4_BUNDLE_DESKTOP_SMOKE_REPORT.md`; design review: `P4_BUNDLE_DESIGN_CHECKLIST.md`. The boundaries
held and were not relaxed: Core stayed headless and filesystem-free (`grep -rn "std::fs"
crates/firmwaresight-core/src/` counts zero), assembly lives in the project crate, no sixth crate and
no new third-party dependency entered, no host path crossed IPC or entered a composed document, the
only clock a bundle carries is the moment a named person accepted a review, and the stage stopped at
the Bundle — no History page, no installer, no signing, no updater.

**`P3 Release Gate` closed `PASS / COMPLETE` on 2026-09-30, under its own architect prompt.** One product
verb, `Gate`, over a stored build plus the workspace facts outside the artifact:
`firmwaresight.toml` → project and Git evidence → `GatePolicy` → Core Gate → five factual states →
immutable `GateRun` → immutable review acceptance → `fwsight gate` → the desktop `Release` page. Per-item
evidence: `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md`; what was built, every gate number and the three
defects this round found: `P3_GATE_EXECUTION_REPORT.md`; the shipped-window result over all forty §61
steps: `P3_GATE_DESKTOP_SMOKE_REPORT.md`; design review: `P3_GATE_DESIGN_CHECKLIST.md`. The boundaries
held and were not relaxed: Core stayed headless and dependency-free, the adapter owns the config and the
read-only Git process, no rule is re-evaluated in SQL or React, no host path crossed IPC, the database or
the portable output, and the stage stopped at the Gate.

`P1_ANALYZE_DETAILS` completed on 2026-09-29 and P1 Analyze is `PASS / COMPLETE`:
bounded `Sections`, `Symbols` and `Evidence` queries over the snapshot SQLite already stores, three
use-case IPC commands, top contributors, the Evidence Inspector, and the `bytes / KiB` presentation
switch that `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` US-001 requires. Per-item evidence:
`P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md`; what was built and what was found:
`P1_ANALYZE_EXECUTION_REPORT.md`; the shipped-window result: `P1_ANALYZE_DETAILS_SMOKE_REPORT.md`;
design review: `P1_ANALYZE_DESIGN_CHECKLIST.md`. Earlier slices keep their own packs: `P0_TECHNICAL_VALIDATION/`
and `P1_A0_VALIDATION/`.

**`P2 Compare` closed `PASS / COMPLETE` on 2026-09-29, under its own architect prompt.** The prompt
changed the pointer while the round ran, not the standards: `P2` was `IN_PROGRESS` until `US-002` was
green item by item, and it is now — per-item evidence in `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md`,
what was built and every gate number in `P2_COMPARE_EXECUTION_REPORT.md`, the shipped-window result in
`P2_COMPARE_DESKTOP_SMOKE_REPORT.md`, design review in `P2_COMPARE_DESIGN_CHECKLIST.md`. The boundaries
that held for P1 held for P2 and were not relaxed: no schema migration (`SCHEMA_VERSION` still 2, no
0003), no new dependency (`Cargo.lock` gained no package), no new design token (still v0.2.1) - a
genuinely missing token stops a round rather than being written as a magic number - no whole-table
payload across IPC (default limit 100, hard max 500, enforced in Rust), details always bound to the
last-good handle, `Unknown` never rendered as zero, addresses never unit-converted. `AGENTS.md` 2 / 7 /
11 keep applying in full; `ADR-0026` relaxed research sequencing, not a single technical boundary.

**`P3 Release Gate` closed `PASS / COMPLETE` on 2026-09-30, under its own architect prompt.** The same
prompt line that authorized the work authorized a structural change, so it is stated here rather than left
to be discovered in a diff: P3 added the fifth first-party library crate (`crates/firmwaresight-project`,
ADR-0027), the third migration (`0003_gate_history.sql`, `SCHEMA_VERSION` 2 → 3, additive only), the
first direct dependencies outside the frozen five (`toml`, `regex`, `sha2`, `thiserror`, `serde` — all but
`regex` already present in the graph or admitted by ADR-0027), a third top-level desktop page (`Release`),
two CLI exit codes that were unreachable and are now not (`4` REVIEW / `5` BLOCK — the surface test
asserted `[0, 2, 3, 6]`, and P3 changed it by name rather than by deleting it), and three new portable
contracts (`gate-results`, `accepted-reviews`, `project-config` at v1). It added **no** design token, did
not touch the license metadata, did not create a Bundle table and did not start History. The verdict is
item by item in `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md`; what the round found on the way is in the other
three documents of that pack.

Three product defects were found by P3's own validation and fixed in product code, each with a regression
test: a version pattern quoted inside an evidence locator made a Gate run unpersistable for any project
using the only MVP version source (`ERR-STORAGE-4006`); one build produced two different run ids on the
two surfaces because the stored footprint carried no evidence pointer and Core hashed a `reason` the
database never recorded; and a disabled primary button kept its accent border.

What belongs to the owner and the architect:

1. pushing remains the owner's act. The P2 round pushed twice. The first, mid-round at `c7fc2a3`, produced
   **Run #17 `36596452341`, `failure`, 3 of 7 jobs red** — `Rust (windows-latest)`, `Rust (ubuntu-latest)`
   and `macOS Core Smoke`, all on `committed_fixtures_match_their_recorded_hashes`, for the reason in the
   gap above: `.gitignore`'s `**/target/` had kept the target half of the fixture pair out of the
   repository while every local run digested the bytes the generator had left on disk. That is defect E,
   fixed at `cfee1e5` and re-verified from a clean `git archive` checkout. The owner then pushed the head,
   and **Run #18 `36648718199` on `4a77ea1` concluded `success`, 7 of 7 jobs** — the same seven that
   carried P1's `e63afaf` on Run #13 `36556735551`. `origin/main` is now green again. No commit carries the
   run its own push produced: #18 is recorded by the successor document that reports it, and the run this
   document's own push starts belongs to a later one;
3. the **open-source license decision**, which is not P3's to make. The root `Cargo.toml`'s
   `[workspace.package]` table still reads
   `license = "Proprietary"` and the repository root still has no `LICENSE` file, while the project is
   being delivered as open source under ADR-0026; `AGENTS.md` 9 places a license change in front of a
   human, so the P3 prompt explicitly declines to choose one and the gap is recorded as
   `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` rather than quietly resolved;
4. a **separate architect prompt** for P4 Release Bundle, which P3's closure will make authorizable and
   this file does not authorize. P3's own prompt says STOP AFTER P3, and nothing in P4's scope — a
   bundle directory, `release prepare`, manifest generation, bundle checksums, a History page — exists
   or is implied by it;
5. whether and when to resume the V0 feedback track with real participants - now a choice rather than a
   gate, with the `v0.1.0` instrument still frozen and ready;
6. re-authorizing pricing, paid-tier or pilot work only after MVP, since `ADR-0026` defers it and the
   matching prompt was withdrawn before execution;
7. re-opening the two accepted advisories only on one of their recorded triggers.

Four items the closure round surfaced and left alone, each with its reason in
`P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` §9.7: the golden updater no longer reproduces the
committed goldens' key order; `ElfProgramHeader` names a source the ELF parser never reads; the frozen
release-manifest schema describes one artifact per build; and a release build without
`--features custom-protocol` shows a WebView network error instead of the product.

Do not start P1, Compare, Gate, Bundle, installer, signing, updater, SBOM, cloud, accounts, AI or
telemetry from this file.
