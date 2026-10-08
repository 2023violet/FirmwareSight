---
title: "U1P-R3 final narrow corrective and installed visual closure: report"
doc_id: "FS-U1P-R3-002"
product: "FirmwareSight"
unit: "U1P_R3_FINAL_NARROW_CORRECTIVE_AND_INSTALLED_VISUAL_REVIEW"
closes: "U1P-V1-02 (the comparison did not outlive the page that computed it) and U1P-V1-01 (the Overview capability band left an unfilled track at 1024)"
protects: "U1P-V2-01 (CLOSED_BY_U1P_R1), U1P-V2-02 (CLOSED_BY_U1P_R2), U1R's failure/last-good semantics, U1's decision-first composition"
status: "BASELINE"
authority: "10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R3_Final_Narrow_Corrective_and_Visual_Closure_v1.0.txt + AGENTS.md 1, 3, 8, 9, 10, 11 + DESIGN.md + ADR-0018 + ADR-0029"
plan: "U1_VALIDATION/U1P_R3_NARROW_CORRECTIVE_PLAN.md (written before the code, with §12 recording what changed after it)"
written_before_code: true
---

# FS-U1P-R3-002 — the comparison now survives its page, the band has three cells, and the pack counts itself

This unit does three things the Architect asked for and nothing else: it makes a valid comparison outlive the
page that computed it, it removes the empty fourth cell from Overview's capability band at 1024 px, and it
delivers an evidence pack whose totals are computed from its own acceptance record. It adds no capability, moves
no Core fact, changes no schema, no IPC, no Gate rule and no frozen asset. **The ceiling this round may claim is
`U1 = READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT`.** `PASS_COMPLETE`, `VISUAL_ACCEPTED` and `MOCKUP_MATCHED` are
the Architect's to issue.

## 1. Start authority, measured before any write

| Fact | Value | How it was measured |
|---|---|---|
| Delivered prompt | `C:\Users\16429\Downloads\FirmwareSight_U1P_R3_Final_Narrow_Corrective_and_Visual_Closure_v1.0.txt` | read from disk |
| Prompt bytes / SHA-256 | 32,849 / `a185890edd043344a4332dfcc048fd63add1f8a5226dead4af373cc1ceab4de9` | `wc -c`, `sha256sum` |
| Line endings | 745 LF, 0 CR | byte count of each |
| Archived blob | `384e8ca42db8d8238d263c18111d58e5d8da1ab1` | `git hash-object` |
| File digest equals blob digest | yes | the prompt arrived LF-only, so the stored blob hashes identically |
| Head at start | `f01eec1` (`f01eec110c76591c741d283e4ee862566e211a2f`) | `git rev-parse HEAD` |
| That head's CI | run `37801519343` (#84, attempt 1), 10 jobs, all success | read at **docs closeout**, not at round start, and recorded as such in `00_authority/START_AUTHORITY_CI.txt` — the governance block cites that run, so the citation has its own file rather than resting on a sentence |
| Owner store before anything ran | `firmwaresight-p0.sqlite` 155,648 B `d6e41034…a836468`, `-wal` 0 B `e3b0c442…`, `-shm` 32,768 B `fd4c9fda…` | `00_authority/PRE_ROUND_MACHINE_STATE.txt` |
| Install present before the round | NO | `Test-Path %LOCALAPPDATA%\FirmwareSight\firmwaresight-desktop.exe` |
| Evidence root | `%TEMP%\FirmwareSight-U1P-R3-Narrow-Corrective-20261008T190057Z` | created at §1 |

Because the pre-round machine had no install, §13 requires the round to uninstall at the end. It did; see §10.

## 2. Root cause of the Compare defect (F1)

`apps/desktop/ui/src/App.tsx` renders one page at a time out of a ternary chain, so switching pages **unmounts**
the previous one. `Compare.tsx` kept its computed `summary` in component `useState`. The consequence is structural
rather than visual: a comparison the user had already paid for was destroyed by the act of looking at another
page, and the only empty state on the page answered a different question ("fewer than two builds"), so what came
back was the pair bar, the Compare button, and nothing beneath either.

The store already holds both builds, so persistence was not the fix and §5 forbade it: no SQLite write, no
backend IPC, no `localStorage`, no schema, no new command. A comparison is a fact about *this session*, so the
session owns it, and the shell is the only component that outlives a page.

## 3. What changed in the product

`git show --stat 41bb6a36` — 12 files, +1,532 / −58. Product source is six files, all inside §10's allowlist:

| File | Change |
|---|---|
| `App.tsx` | the comparison session becomes shell state: `const [comparison, setComparison] = useState<ComparisonSession>(EMPTY_COMPARISON)` is passed to `Compare` as `session` / `onSessionChange`. Six pieces of state now live in the shell, and the file says which six and why |
| `Compare.tsx` | exports `ComparisonSession` and `EMPTY_COMPARISON`; reads become controlled and writes become functional updaters (`onSessionChange(previous => …)`), so two writes in one render cannot clobber each other and an obsolete async response cannot resurrect a stale pair; a new `Ready to compare` panel explains the waiting state; the size-unit switch moves into the heading row it belongs to and disappears once a result is on screen; the subhead is one sentence; the footer states the evidence basis |
| `Compare.module.css` | `.sectionHead` (heading + its own unit switch on one baseline row), `.readyMessage`, and two spacing moves that keep the result above the fold — every value a token |
| `Overview.tsx` | the two capability bands are wrapped in a local `styles['fillBand']` div, and the Git cell's sentence is corrected (see §6) |
| `Overview.module.css` | `.fillBand > [role='group'] { display: flex; flex-wrap: wrap }` with `flex: 1 1 var(--fs-layout-evidence-inspector-min)` children, so a wrapped line fills instead of leaving a track unpainted |
| `compare.test.tsx` / `overview.test.tsx` | 13 new tests, T1–T9 and T10–T12 |

**`components/Panel.tsx` and `Panel.module.css` are not in this commit.** A shared `fill` prop was implemented,
reviewed and reverted to its HEAD digest, because §10 prefers a local Overview class and "provably inert on other
pages" is a weaker claim than "not touched". The bands on Analyze, Compare, Release and History are the ones the
Architect already approved.

## 4. RED first, then GREEN, then mutation

`target/u1p_r3_red.txt` records the run against the unchanged `f01eec1` tree, and the record keeps its own
corrections rather than being smoothed after the fact:

* **The RED run itself reported `7 failed | 49 passed (56)`**, all of it from `compare.test.tsx`: **T1, T3, T4, T5,
  T6, T8, T9** — each one a behaviour the fix has to provide, from the ready state through navigation retention to
  the refusal path.
* **T3b is RED too, but it was not in that run.** It was written after the run, so it was measured on its own the
  same way (HEAD `App.tsx` and `Compare.tsx` written over the working copies, new tests left in place):
  `1 failed | 56 skipped (57)`, the failure being T3b. So the honest count is **eight of the thirteen new tests are
  RED on the untouched tree**, one of them measured in a second, separately recorded run.
* **Passed as intended (2 locks):** T2 protects the existing "fewer than two builds" empty state and T7 protects
  what a fresh session legitimately shows. A test that fails before a change and after it too is not evidence of
  anything, so the block header names which is which.
* **The Overview trio (T10–T12) is a lock set, not a RED proof.** Re-measured once the parse typo (§5) was fixed,
  with HEAD copies of `Overview.tsx`, `components/Panel.tsx` and `Panel.module.css` written back over the working
  copies, all 38 tests in the file pass against the unchanged product code. jsdom lays nothing out, so the empty
  capability-band cell at 1024 px cannot be answered by a DOM test at all; it is answered by the installed
  screenshot and the probe in §11. The three files were then restored by digest and re-checked
  (`d8f1fd1f…`, `d44438f3…`, `8a7e72da…`).

GREEN at the product head: the two R3 files carry 57 + 38 = **95 tests, all passing**, and the full UI suite is
**291 in 9 files** (`target/u1p_r3_gate_final.txt`).

Three mutation records, described exactly as `target/u1p_r3_mutation.txt` has them:

* **M2 — the `Ready to compare` panel removed, nothing else changed:** `2 failed | 54 passed (56)`, and the two are
  T1 and T9, the ready state and its accessible names.
* **M3 — the `stale` rule hard-wired to false,** so a standing report can no longer say which pair it answers for
  once the selectors move: `4 failed | 52 passed (56)` — T5 and T6, which this round wrote for the retained case,
  **and two pre-existing U1P tests written for the same rule.** That is the useful part: the rule is load-bearing
  across rounds, and this round inherited its wording instead of inventing a second rule for the same fact.
* **M1 — retention across navigation — is not a separate edit.** It *is* the RED run: page-local state is exactly
  what `f01eec1` ships, and T1, T3, T4, T5, T6, T8 and T9 fail there. Recording it as a fourth mutation would have
  counted one measurement twice.

Each measurement put the tree back and proved it: the lock re-measurement restored `Overview.tsx` to `d8f1fd1f…`,
`components/Panel.tsx` to `d44438f3…` and `Panel.module.css` to `8a7e72da…`; T3b's own run restored `App.tsx` to
`af85dcb5…` and `Compare.tsx` to `511ddee6…`; the mutation record ends with `Compare.tsx` back at the implemented
`2f0bcd88…`. The two CSS modules were never mutated, so they carry no restore line — and no mutation code is
committed.

## 5. Three corrections this round made about itself

1. `overview.test.tsx` carried a stray `}` at line 1094, so the file collected **zero** tests while the banked
   RED header asserted T10–T12 had passed. The typo is fixed, the raw `PARSE_ERROR` line is kept in the log, and
   the header now carries an explicit CORRECTION block instead of the false claim.
2. The historical `U1P_BASELINE/P03_Compare_1440x900.png` contains the Windows taskbar across its bottom 33 px.
   The image is **not** altered — it certifies its own round — and `u1p_r3_capture.py` now pins the window to the
   origin and refuses any grab whose client bottom falls below the work area (screen 1920×1080, work area
   1920×1040). Every R3 capture records `client_inside_work_area: true`.
3. An earlier draft of §4 above described the mutations from memory and got them wrong: it said M2 removed the
   shell ownership, M3 removed the ready panel, and M1 was a negative control that changed nothing.
   `target/u1p_r3_mutation.txt` says otherwise — M2 is the ready panel (T1 and T9 fail), M3 is the `stale` rule
   (4 fail, two of them pre-existing U1P tests), and M1 is not an edit at all but the RED run itself. It also said
   the RED run reported eight failures where the log reports seven, T3b having been measured in a second run
   because it was written after the first. §4 now quotes the records. The claim being corrected is this report's,
   the evidence is a file this round wrote, and no product byte moved to accommodate it.

## 6. The §8 read-only audit, and the one copy defect it found

The audit asked whether Analyze's `Git unknown` and the Gate's `git.clean` rule contradict each other. They do
not, and the installed bytes show both: Analyze's chip is Core's *analysis capability* for the artifact's
provenance; `git.clean` is the gate's separate question about the opened project, and it answered
`PASS — Workspace has no uncommitted changes at Gate evaluation time` with evidence `git:status`, `git:head` on a
worktree `00_authority/DISPOSABLE_PROJECT.json` records as clean. **No safety-critical false Gate PASS appeared,
so §8 produced no STOP.**

What the audit did find was a sentence in `Overview.tsx` that attributed the wrong source to the Git cell. It was
repaired inside this round's authorized scope, and its replacement is quoted off the installed screen in
`P01_Overview_Matched_1440x900.png` and `R01_Overview_Matched_1024x720.png`.

## 7. §11 local validation at the product head

```
RESULT PASS      17/17 steps passed
rust/fmt clippy test · frontend/install typecheck lint test build
drift/design tokens · desktop icons · baseline integrity · ipc bindings
      · ipc bindings unchanged · fixtures tracked · version identity · goldens unchanged
deny/cargo-deny
rust group: 47 result lines, 868 passed, 0 failed
frontend:   Tests 291 passed (291) in 9 files
```

`drift/baseline integrity` PASS is the ADR-0029 verifier over the staged index blobs; the standalone verifier run
and the `--only` runs for drift (8/8), deny (1/1), core-smoke (3/3) and package (4/4) are in
`00_authority/VALIDATION.txt`. `assets/design-tokens.json` is byte-identical. UI went 278 → 291 because this
round added 13 tests and removed none. The whole-gate log is never summed for a Rust figure: the drift group
re-runs the desktop crate, and adding it would double-count 222 tests.

## 8. §12 product commit and the exact CI read-back

One normal commit, `41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177`, message
`U1P-R3: retain comparison context and close narrow viewport gaps`, pushed as a normal fast-forward under the
user's authorization. No force, no rebase, no squash, no amend, and no published commit rewritten.

Run **37828673549** (`P0 verification gate`, number **#85**, attempt **1**, event `push`, head
`41bb6a36…`, `refs/heads/main` read back as the same object): **10 of 10 jobs individually successful** —
Dependency policy, Rust (ubuntu), Desktop UI (ubuntu), Rust (windows), Generated output drift, Package Ubuntu,
Package macOS, macOS Core Smoke, Desktop UI (windows), Package Windows. Rust 868/0 in 47 groups on both OS
runners; UI 291 in 9 files on both; drift 8/8 including baseline integrity. Full table with job ids and
timestamps: `00_authority/CI_READBACK.txt`.

## 9. §13 artifact identity and the controlled install

| Item | Value |
|---|---|
| Artifact | id `11573661113`, `FirmwareSight-0.6.0-windows-x86_64`, 5,544,133 bytes |
| Container ZIP SHA-256 | `330e83af75db705b2acae6ff559baf0e863d7766947bc21240488861cd4f3fb4` — downloaded bytes equal GitHub's own digest |
| NSIS installer | `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, 3,896,257 bytes, `9a51e86a…f8c87d93` |
| Installed EXE | 15,368,192 bytes, `5c828ecf1925f85ab2919b9084541c716f23902cfc4b76dfdbd8ad110568ffd0`, re-measured after the installer released it |
| `artifact-metadata.json` | `toolchain.git_commit = 41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177`, version 0.6.0, cargo 1.98.1 |
| Internal `SHA256SUMS.txt` | both payload lines verified against the extracted bytes |

Install used real `SendInput` through the NSIS wizard: no `/S`, no `/SILENT`, no message posted into another
process. The harness walked four pages and then **stopped** rather than inventing a press, because this build's
last page offers `&Finish` where the inherited picker knew only Next/Install/Close. The remaining two presses
were made by hand through the same ownership-checked path and are recorded in `01_artifact/INSTALL_FINISH.json`:
`Create desktop shortcut` was found ticked **by screenshot** and unticked (no litter left behind), then
`&Finish`. The desktop shortcut box is the only option that was toggled, and the screenshot pair is in the pack.

## 10. §13 tail: uninstall, disposable state, owner restore

| Flag | Value | Proof |
|---|---|---|
| `OWNER_STORE_PARKED` | YES | `PARK.json`; the three files were unchanged between `BASELINE.json` and the park, and the live path held no owner file while the app ran |
| `OWNER_BACKUP_HASH_MATCH` | YES | the independent backup digests equal the originals |
| `OWNER_STORE_OPENED_BY_R3` | **NO** | proved, not asserted: the three names came back byte-identical to `PRE_ROUND_MACHINE_STATE.txt` |
| `ORIGINAL_DB_RESTORED` | YES | `RESTORE.json` |
| `ORIGINAL_DB_SHA_MATCH` | YES | against `BASELINE.json` |
| `UNRELATED_SIBLING_STORES_UNTOUCHED` | YES | 12 digest-bearing files in the app-data folder besides the owner store, identical before the cleanup and after the restore |

The machine had no install before the round, so the product was uninstalled with real input; the
`Delete the application data` box was read by screenshot, found unticked, and never clicked — the app-data folder
held 17 entries before the uninstall and 17 after. The disposable store the app wrote was hashed and moved out
whole, never deleted, and a read-only copy is kept in `07_disposable_store/` so every identity claim in the
acceptance record can be re-read after the machine is clean. The disposable project is a copy of committed
fixtures under `%TEMP%\U1P-R3-disposable\`, verified digest-for-digest on both sides; nothing under `fixtures/`
was written, and no private firmware or real customer data entered the round.

## 11. §14 installed captures and §15 adjudication

Twenty named captures were taken on the installed bytes: P01–P10 at 1440×900 and R01–R10 at 1024×720 /
1056×799, each logged with requested client size, client measured before and after the grab, window title, work
area, the client's bottom on screen, byte count and SHA-256, plus the artifact, NSIS and installed-EXE digests.
Six supplementary captures carry evidence §15 names but §14 does not (`S8_GIT_TRACE_…`, `S8_GATE_BASELINE_…`,
`C6_STEP1/2/3_…`, `KB_Focus_…`), and one is **discarded**: a first attempt at `P10_Overview_GateMismatch_…` was
taken after a click missed its button, so the state it names had not been produced. It is out of
`CAPTURES.jsonl`, kept as `00_authority/DISCARDED_P10_…`, and its reason is in `DISCARDED_CAPTURES.jsonl`. No
capture was mocked, cropped or substituted.

`09_acceptance/ACCEPTANCE_GATE.json` adjudicates **25 items: 23 PASS, 1 FAIL, 1 NOT_VERIFIED, 2
MISMATCH_PROVED**. The FAIL is not a product contract — it is a finding this round recorded instead of fixing
(§12 below). The NOT_VERIFIED is honest scope: the Overview "Previous analysis" band was not re-photographed by
this round, so its leg of O3 is marked as such rather than folded into a PASS.

The two MISMATCH_PROVED items are the policy-fingerprint mismatch (`P10`: the stored run policy
`b87aa057…` beside the loaded `bcc2f9da…`, produced by opening `project-alt`'s one-line-different
`firmwaresight.toml`) and the neutral pending/mismatch handling in O3. They are counted under PASS as well, and
neither says the build is ready — the page itself refuses to restate a stale verdict.

**C2 carries a stated boundary rather than a silent judgement.** The first viewport at 1440×900 holds the whole
computed answer under every heading the comparison has: `What moved`, the Sections and Symbols counts, the
Memory deltas (+120 B nonvolatile, +68 B runtime) and the comparability line. What it does **not** hold is a row
of the per-section table: that table's header sits at y≈869 and its first data row begins below the fold. Under
"a meaningful row about sections" C2 passes; under "a row of the section table" it does not. The item is marked
PASS with that sentence attached, because the arbiter of which reading governs is the Architect, not this round.

**The void metric was re-read for adjacency, and the re-read is in the record.** The calibration counts sampled
rows (every 2 px) whose longest horizontal border run reaches 100 px, and called four or fewer such rows "no
unfilled track". A row count is sensitive to where the measuring box ends — the same image reads 4 or 6 rows
depending on the box chosen — so the same probe data was re-read for the property that actually separates an
unfilled track from a border: **a void is a stack of adjacent qualifying rows, an edge is one row.** On that
reading Overview's defect is a 228 px rectangle (114 adjacent rows) on both the U1P baseline 1024 capture and the
R2 image the Architect flagged, and on this round's bytes no stack exists at either width: 3 rows at 1024 matched,
4 at 1056, and 5 on the pending capture — whose five rows are single-row edges spread over y 234..418, named here
rather than rounded away, because a reader who saw only the count could call them a hit. The 1440 control has 4
edge rows and no stack, which is why the bare threshold was never a good rule on its own. The full table with both
readings and every box is `09_acceptance/O1_STACK_ANALYSIS.txt` (machine form `.json`), produced by the same
unmodified harness. **The pack was not rebuilt for this.** It was written once, as §16 asks, and every figure it
prints is true for the box it was printed with; this refinement strengthens that claim instead of replacing it.

## 12. Two findings this round recorded and did not fix

1. **The Analyze page's four-figure metric band shows the same unfilled track at 1024 px** that §7 removed from
   Overview. Measured with the same code and the same shape of box: 126 rows with a ≥100 px border run over
   `R08_Analyze_Success_1024x720.png`, and the **identical** 126 rows / 715 px run on the R2 pack's accepted
   `R02_Analyze_1024x720.png` — so it predates this commit and this commit neither caused nor touched it. §7
   scoped the band work to the Overview capability band and §10 prefers a local class over a shared Panel change,
   so it is recorded in `09_acceptance/ANALYZE_BAND_VOID.txt` for scheduling.
2. **The Release Gate's baseline picker is page-local and resets on navigation.** The run in this round's
   evidence was made against `snap-3f6…64ed363e`, and after navigating away and back the same select reads
   `No baseline` while the run it produced is still shown. That is the same defect family as the one this round
   fixed for Compare, on a page §10 does not allow to change here. `S8_GATE_BASELINE_PICKER_AFTER_NAV_1440x900.png`
   holds both facts in one frame.

Neither is an S0/S1/S2: no data is wrong, no verdict is fabricated, and both are visible to the user as they are.

## 13. Evidence classification

Everything Observed here is a screen, a store row, a log line or a digest. The comparison figures are **Derived**
by Core and read off the product's own output; the fixture digests are **Declared** by
`00_authority/DISPOSABLE_PROJECT.json`; the two findings in §12 are **Observed** and explicitly not fixed. No
inference is written as observation, and no identity string in the acceptance record was typed from a screen —
they are read from the copied SQLite store.

## 14. The pack

`C:\Users\16429\Downloads\FirmwareSight_U1P_R3_Final_Visual_Review.zip` — **4,224,650 bytes**, SHA-256
`4de07c7669574955493f2fd31594539422662415c14d8c5d378e2126ac831e52`, **102 members**, written once by
`target/u1p_r3_pack.py` at 2026-10-08T20:37:57Z from the staging tree
`C:\Users\16429\Downloads\FirmwareSight-U1P-R3-Pack\U1P_R3_FINAL_VISUAL_REVIEW`.

It carries the untouched seven-image reference set, 11 R2-era originals pulled member by member out of the
hash-verified historical ZIP (`d3674569…1bb34c`, 4,113,494 bytes, still untouched) and checked against that
archive's own manifest, this round's 20 named captures plus 10 interaction and installer traces, 8 derived
side-by-sides that label themselves DERIVED and paste panels at native size, the reports, `ACCEPTANCE_GATE.json`,
`HASHES/SOURCE_MAP.txt`, `HASHES/SHA256SUMS.txt` (101 entries: every member except the manifest itself, which
cannot carry its own digest, and except the post-hoc `HASHES/ZIP_VERIFICATION.json`), and `HASHES/SELF_CHECK.txt`.

The counts are the point of the round's closing deliverable, so they are not typed. `00_README.txt` is generated
by summing the `boxes` array of `ACCEPTANCE_GATE.json`, and the builder then re-reads the README it wrote,
re-derives the counts from the JSON a second time, and refuses to continue unless they agree — including after
the archive is closed, when the README is read back out of the extracted copy. `HASHES/SELF_CHECK.txt` also
records a **negative control**: one count in a copy of the README was raised by one and the checker was required
to reject it, because a check that only ever sees consistent input proves nothing. Its two sections separate the
**104 checks on the pack as built, all 104 passed** from the one rejection the checker was asked to produce. The
recorded figures are ITEMS=25, PASS=23, FAIL=1, NOT_VERIFIED=1, NOT_CAPTURED=0, MISMATCH_PROVED=2, with
PASS+FAIL+NOT_VERIFIED+NOT_CAPTURED = ITEMS and MISMATCH_PROVED a flag on 2 of the 23 PASS items rather than a
fifth verdict. The archive's own verification — CRC over every member, byte-exact extraction rehash of all 102
members, a forbidden-content scan that refused any SQLite/WAL/SHM, firmware, installer, artifact container,
nested archive or `.git` path, and the README/JSON count comparison run against the extracted copy — is in
`HASHES/ZIP_VERIFICATION.json`, kept both beside the archive and at
`…T190057Z\08_pack\ZIP_VERIFICATION.json`.

**One build was superseded inside this round, and it is disclosed rather than quietly replaced.** The first
archive (4,223,656 bytes) was correct in its counts but its `HASHES/SELF_CHECK.txt` interleaved the negative
control's *intended* rejection among the pack's own checks, so a reader would have found a `FAIL` line in a file
that meant to say everything passed. That is a presentation defect in an evidence document, so the document was
rebuilt: the checker's deliberate failure is now recorded in its own labelled section. The superseded archive and
both of its staging trees were **moved, not deleted**, to
`…T190057Z\08_pack\superseded_first_build\`, where they can be opened and rehashed. Nothing else about the pack
changed; no screenshot, no report and no acceptance record differs between the two builds, and the change touched
no product source, so the product head and the CI evidence above still describe the bytes being shipped.

The R2 pack is retained untouched; the stale `MISMATCH_PROVED=0, PASS=0` line in its README is documented in the
pack itself — quoted verbatim in `00_README.txt` and reproduced in `REPORTS/R2_README_HISTORICAL.txt` beside that
archive's own gate JSON — as a historical defect of that pack, not rewritten.

## 15. Governance ordinals

Two stale ordinals were named in §8 and both were still present; a third of the same class was found while
repairing them, and all three were corrected in the single docs successor rather than in a separate commit:
`.ai/CURRENT_STATE.md` called U1P-R1 "a fourth U1 unit" and U1P-R2 "a sixth U1 unit", and `.ai/ACTIVE_TASK.md`
called U1P-R2 "the sixth U1 round". The enumeration those files themselves give — U1, the second unit, the third
unit, then "two more units ran on 2026-10-08" (the U1P product round and `U1P_INSTALLED_ACCEPTANCE_…`) — makes
U1P-R1 the **sixth**, U1P-R2 the **seventh** and this unit the **eighth**. The dated narrative around each label is
unchanged.

## 16. V1 interlock and what is left

V1 stays `IN_PROGRESS` / `RECRUITMENT_READY` with **0 participant sessions**; the F3 cohort artifact is
unchanged; no B1, no RC, no GA, no signing, no updater, no licence change. No stage was self-issued here.

Left with the Architect: the visual verdict on the installed screenshots; the two findings in §12; the C2
reading in §11; and the Overview "Previous analysis" band this round did not re-photograph.

## 17. The docs-only successor, and what it re-measured

§17 of the prompt allows exactly one docs/evidence successor after the owner restore, and this is it. Its staged
diff against product head `41bb6a36…` is **nine paths and no product source** — `.ai/ACTIVE_TASK.md`,
`.ai/CURRENT_STATE.md`, `.ai/HANDOFF.md`, `BASELINE.yaml`, `INDEX.md`, `README.md`, this report, and the two
ADR-0029 artifacts. No crate, DTO, schema, migration, storage contract, wire field, Gate rule, IPC surface,
dependency, capability, token or golden file appears in it. The two artifacts move because this report is a new
tracked path — the tree gains its one line and the manifest its one entry, which is why the counts are 895 lines
and 768 entries over 770 tracked paths rather than the 894 and 767 over 769 that product head `41bb6a36…`
carried.

Because the successor re-stages the index, `drift/baseline integrity` re-reads it, and the record of the successor's own
validation is a log, not this paragraph: `target/u1p_r3_gate_successor.txt` — **exit code 0, 17 of 17 steps PASS,
no SKIP and no FAIL** — read at the fully staged state. Its figures are pulled out of that log rather than
asserted: the `rust` group is **47 result lines, 868 passed, 0 failed**, and the frontend group prints
**`Test Files 9 passed (9)` / `Tests 291 passed (291)`**. Those are the same two numbers §7 measured at the
product head, so §17's "verify Rust/UI test counts unchanged from R3 product head" is satisfied by comparing two
logs, and neither was carried over from a document. `python scripts/verify_baseline_artifacts.py` reports
**RESULT PASS** at **895 tree lines, 768 sum entries over 770 tracked paths**, with index blob mismatch 0,
tracked-but-unlisted 0, duplicates 0 and `SHA256SUMS` self-excluded.

**One intermediate run of that gate was abandoned, and it is named rather than quietly dropped.** It was started
while the last two paragraphs of this section were still being edited, so it never described a state that could be
committed; it was left to drain against a superseded index and its log was replaced by the run described above.
The cited log is the one produced at the final fully staged state, and that distinction is why the abandoned run
gets a line here instead of silence.

**This successor is a normal commit and a normal fast-forward push under the operator's written authorization for
U1P-R3** — no force, no rebase, no squash, no tag, no release, and nothing already published is rewritten. It is
the round's only docs/evidence successor, and the §17 one-successor cap is not exceeded by it or after it.

Its CI is read **job by job from its own head** once the run exists, and recorded in
`00_authority/SUCCESSOR_CI_READBACK.txt` in the evidence root — **not in this file**. §17 forbids a further commit
just to write a run result back, and this document sits inside the commit whose run it would be describing, so the
repo record stops here by design and the evidence root carries the read-back. A reader of the repo should expect
that file to be written after this paragraph was.

The machine was re-read at closeout, after the acceptance record was already written, and the re-read is in
`02_owner_backup/CLOSEOUT_REPROOF.json` / `.txt`: the owner's three files still carry the byte counts and digests
`PRE_ROUND_MACHINE_STATE.txt` recorded before the installer ran (`d6e41034…` 155,648 B, `e3b0c442…` 0 B,
`fd4c9fda…` 32,768 B), the app-data folder still holds 17 entries, and the install directory is absent — so the
restore and the uninstall are true of the machine now, not merely of the log that claimed them. The re-proof script
(`target/u1p_r3_closeout_reproof.py`) only stats, reads and hashes; it writes nothing under the store or the
install path.

The pack is unchanged by any of this. It was written from the product head's evidence, no product byte moved after
the captures, and the two counts this section verifies — 868 and 291 — are the two the acceptance record already
prints for the bytes being shipped.
