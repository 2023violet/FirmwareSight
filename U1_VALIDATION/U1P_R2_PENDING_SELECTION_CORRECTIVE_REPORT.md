---
title: "U1P-R2 Overview pending-selection evidence scope: corrective report"
doc_id: "FS-U1P-R2-002"
product: "FirmwareSight"
unit: "U1P_R2_OVERVIEW_PENDING_SELECTION_EVIDENCE_SCOPE"
closes: "U1P-V2-02"
status: "BASELINE"
authority: "10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R2_Overview_Pending_Selection_Evidence_Scope_v1.0.txt + AGENTS.md 3, 8, 9, 11 + DESIGN.md + ADR-0029"
plan: "the round's own §4 rebase audit, quoted in §3 and §4 below"
written_before_code: true
---

# FS-U1P-R2-002 — what changed, what was proved, and what is left to the Architect

This unit closes **U1P-V2-02 — PENDING_SELECTION_STALE_ANALYSIS_PRESENTATION**: on a selection the user had not
analyzed yet, Overview's capability band and figure band wore the *retained* analysis's facts as if they were the
current selection's, while the MAP sentence described the new selection. It is a presentation-scope corrective. No
verdict is computed here, no Core fact moved, no parser, diff, gate or persistence semantics changed, and no
capability was invented. The strongest state this round may claim is recorded in §19.

## 1. Start authority

| Item | Value | Measured by |
|---|---|---|
| `HEAD` / `origin/main` at start | `60f10a6473ef446dc933c94a5edbab8e9b662ad1` (the U1P-R1 evidence head) | `git rev-parse HEAD`, `git rev-parse origin/main` after `git fetch --prune origin` |
| Worktree at start | clean; the configured Git content filters make `git status` uninformative about cleanliness, so cleanliness was asserted from `git diff --binary --stat` being empty and from the two identity checks | `git diff --binary --stat` |
| Prior CI authority | run `37765205830` 10/10 at `60f10a6` | `gh run list --json databaseId,headSha,conclusion` |
| Prompt delivered | 21,194 bytes, 449 lines, 448 CRLF terminators, SHA-256 `3b6108b6e61582f9e31004d41e9768ba1a0daaaee1150b345c194a7e52b6221e` | `wc -c` / `wc -l` / `sha256sum` on the attachment file |
| Prompt archived | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R2_Overview_Pending_Selection_Evidence_Scope_v1.0.txt`, stored blob `aea814d62902b78cd9a98ba164807abe3b0b7e95` = 20,746 bytes, SHA-256 `fd8d28aafe411ac76dbf7262852f22a125c4b12ec3cadafbebc42b35f7a35102` | `git hash-object`, `git cat-file -s`, `git show :path \| sha256sum` — the 448-byte difference is the CRLF terminators Git normalises out of the blob, so the working file and the stored content are the same text |
| Canonical unit, read from the file | `U1P_R2_OVERVIEW_PENDING_SELECTION_EVIDENCE_SCOPE` | line 6 of the archived prompt |
| Evidence root | `%TEMP%\FirmwareSight-U1P-R2-Pending-Selection-20261008T132730Z` | written before any install or launch |

## 2. Verified code root cause

Read in the product before anything was changed. The defect is a scope gap, not a wrong string:

- `App.tsx` keeps three separate pieces of state: `selection` (what the user has picked now, carried by
  `SelectionDto.selectionId`), `lastGood`/`analysis` (the last **successful** analysis) and `analyzedSelectionId`
  (the selection handle that successful analysis belongs to, which moves only on a successful `analyzeSelection`).
  `gateRun` is written by the Release page alone.
- `Overview` received the retained analysis and rendered from it unconditionally. Its capability band and its four
  figure cells therefore described `lastGood`'s artifact, and its MAP detail line read the *new* selection's MAP
  state. Both halves were individually true and jointly misleading — which is exactly the pair the Architect saw on
  S3: `MAP provided` beside `Symbol-level analysis depends on it`.
- U1P-R1 had already bound the **verdict** to its judged snapshot, so the Gate card itself was correctly neutral
  (`Not assessed for the selected artifact`). The finding was that the bands *around* that card were not bound the
  same way. `current` facts and `retained` facts were rendered in one voice.

The fix is a presentation-scope rule, implemented in the UI from data Core already outputs: a fact may be shown as
*this selection's current fact* only when `selection.selectionId === analyzedSelectionId`. Otherwise the selection
is named as not analyzed, and the retained work is shown under an explicit `Previous analysis` heading carrying its
own artifact name and its full judged snapshot id.

## 3. Current versus previous authority, stated as the code states it

`apps/desktop/ui/src/Overview.tsx` now distinguishes three states and derives each band from one of them:

| Band | Authority when the selection is analyzed | Authority when it is not |
|---|---|---|
| Selected-artifact row | the analyzed selection's own summary | the pending selection itself: `<file> · not analyzed yet`, `MAP attached: <name>` / `No MAP attached to this selection` |
| Capability cells | current band, from the analyzed summary | **no cells at all** in the current band; the retained chips move to `Previous input capabilities` |
| Figure cells | current band | **no cells** in the current band; retained values move to `Previous key figures`, prefixed `As reported by the analysis of <file>.` |
| `Can we ship now?` | the run, guarded by U1P-R1's subject/policy checks | unchanged from U1P-R1: neutral, plus the retained run's identity in full |
| Primary action | `Resolve on Release Gate` / the real verdict path | `Analyze selected artifact` (primary), `View previous Gate run` (secondary) |

Nothing is hidden: the retained result stays on screen, attributed. What changed is which subject each number
answers for, and that the attribution is in text a reader hears (`role="note"`), not only in a distant heading.

## 4. Same-name artifact test

The sharpest form of the finding: the user re-selects the *same file name* that was analyzed a minute ago. `fileName`
equality proves nothing, because `selectionId` is the handle and the bytes may differ. `Overview.tsx:373-376`
computes `sameName = candidate.fileName === artifact.fileName` and says, in the note:

> `Previous analysis of firmware.elf. The artifact selected now has the same name and has not been analyzed;
> nothing here describes it.`

and, when the names differ:

> `… It is not an analysis of not-an-elf.elf: that selection has not been analyzed yet.`

R2-T6 asserts the second form; R2-T12 asserts that the note is a `role="note"` element whose text carries the
attribution itself; S3 and S5 are the installed proof of both branches.

## 5. The MAP contradiction, before and after

- Before (U1P-R1 bytes, same state): the chip said `MAP provided`, taken from the retained analysis, while the
  sentence `Symbol-level analysis depends on it` was written about the new selection — two subjects in one breath.
- After (this round's bytes): the pending row says `MAP attached: firmware.map` about the selection it names, and the
  dependent sentence is gone from that row. A selection with no MAP says `No MAP attached to this selection` and
  gets no pill (S4, and R2_S04). `Symbol-level analysis depends on it` was verified absent from S3, S4, S5 and R02.
- R2-T3 locks the invariant structurally: a MAP chip and any sentence about symbol analysis must live inside one
  artifact's context.

## 6. Value display truth

R2-T2 and R2-T12 keep the four figures out of the current band while the selection is pending, and name the retained
band in audible text. Installed: on S3 the current band carries no capability pill; `FLASH FOOTPRINT`, `RUNTIME RAM`,
`SYMBOLS`, `EVIDENCE` appear only inside `Previous key figures`, below the heading and the full snapshot id. On S6 a
genuine new analysis of base replaced them with current values (256 / 8 / 47 / 10 over the 16,820-byte
`firmware.elf`, sha256 `3f615b62…33f21f`) and left no pending band and no `Previous analysis` panel. Unknown stays
Unknown: no zero was substituted anywhere.

## 7. CTA truth

What is proved, and by what, is split deliberately:

- **Installed, on these bytes:** the pending state shows exactly two actions — `Analyze selected artifact` (primary)
  and `View previous Gate run` (secondary) — recorded in the S3 text and in `REG_KeyboardFocus_1440x900.png`, whose
  note records thirteen Tab presses from the document start landing on the primary action with the focus token's ring,
  with no mouse used for that capture. Every row of `CAPTURES.jsonl` carries the OS window title it was captured
  under, so no capture is presented as a page it was not on: the twelve Overview captures are `FirmwareSight -
  Overview`, the Analyze glances are `FirmwareSight - Analyze`, the Release glance is `FirmwareSight - Release Gate`.
- **In the committed product:** R2-T5 fails if the pending selection's primary action is a Gate claim, and mutation
  `M3_pending_cta` (rewriting that handler to `onOpen('release')` and its label to `Choose this build on Release
  Gate`) kills exactly that one test. So the routing is proved by the code and its contract test.
- **Limitation, stated rather than smoothed:** this round did **not** record a press-then-title pair on the installed
  build for those two buttons — the `REG_Analyze_pending` and `REG_Release_run` glances were reached by navigation, not
  by pressing the Overview actions, and their notes say so. The installed evidence therefore proves the actions are
  present, correctly named, keyboard-reachable and focus-visible; the destination itself rests on R2-T5 and M3. If the
  Architect wants that last link covered by an installed capture, it is a one-action follow-up, and this round did not
  claim it.

`Choose this build on Release Gate` is verified absent from the pending states (S3, S5) — it belongs to S2, where the
analyzed build and the retained run are genuinely different builds, and it is still there and still correct.

## 8. Tests: RED before the change, GREEN after, and what mutation kills them

- **RED** on the unmodified implementation at `60f10a6`: `Test Files 1 failed (1)`, `Tests 6 failed | 29 passed (35)`
  (`target/u1p_r2_red_final.txt`). The six are exactly R2-T2, T3, T5, T6, T7, T12 — the new locks. The other six
  tests in the file are regression locks on already-accepted behaviour and were expected to pass before the change.
- **GREEN** after: focused `Tests 35 passed (35)` in one file; full suite `Test Files 9 passed (9)`,
  `Tests 278 passed (278)`.
- **Bounded mutations** (one expression each, `target/u1p_r2_proof.py`, each undone by digest and re-hashed equal to
  the committed implementation):

  | Mutation | What it breaks | Result |
  |---|---|---|
  | `M1_pending_dispatch` | the pending row never renders | 4 failed / 31 passed |
  | `M2_previous_labels` | `Previous input capabilities` → `Input capabilities` | 3 failed / 32 passed |
  | `M2b_previous_figures_label` | `Previous key figures` → `Key figures` | 3 failed / 32 passed |
  | `M3_pending_cta` | pending primary action becomes the Gate claim | 1 failed / 34 passed |
  | `M4_stale_note_removed` | the attribution note loses its `stale` styling/role path | 1 failed / 34 passed |

  Each mutation is killed by a test, so each sentence the page tells is load-bearing.

## 9. Changed paths, and the forbidden ones absent

`git show --numstat 2e2e219` — 7 files, `+990 / −76`:

```
449   0  10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R2_..._v1.0.txt
 28   0  10_AUDIT/SOURCE_PROMPTS/README.md
  1   0  DIRECTORY_TREE.txt
  6   5  SHA256SUMS
 13   0  apps/desktop/ui/src/Overview.module.css
233  71  apps/desktop/ui/src/Overview.tsx
260   0  apps/desktop/ui/src/overview.test.tsx
```

Product source is three files under `apps/desktop/ui/src`; the rest is the prompt archive and ADR-0029 baseline
artifacts. `git diff --name-only 60f10a6..2e2e219` contains no path under `crates/`, `schema/`, `migrations/`,
`apps/desktop/src-tauri/src/ipc.rs`, `assets/design-tokens.json`, `pnpm-lock.yaml`, `Cargo.lock` or `.github/`.
`assets/design-tokens.json` is byte-identical (SHA-256 `94336906997d23b1f36cbaf81711c6824082d88b18aa89c38dd12721564a14d4`),
and the `generate_handler![…]` list in `lib.rs` holds **30** command entries at `60f10a6` and the same 30, name for
name, at `2e2e219` — no new IPC surface. That count is taken by splitting the list on commas: counting `module::`
pairs under-reads it as 28, because `list_fixtures` and `get_analysis_summary` are registered without a module
prefix, and BASELINE.yaml has guarded 30 since P5.

## 10. Local gates

`python scripts/check.py` at `2e2e219`: **17/17 steps passed**, 0 FAIL, 0 SKIP
(`target/u1p_r2_gate.txt`) — 3 rust, 5 frontend, 8 drift (including `drift/baseline integrity` under ADR-0029), 1
deny. `cargo test`: **868 passed, 0 failed** across 47 `test result:` lines. `pnpm test`: **278 tests in 9 files**,
up from U1P-R1's 266 by exactly the twelve R2 contract tests. fmt/clippy/typecheck/lint/build are steps inside the
17.

## 11. Product commit, push and exact remote CI

`2e2e21997ae01f035ac72b97658b3a0b329da362` — "U1P-R2: scope Overview analysis facts to the current selection",
pushed as a normal fast-forward onto `60f10a6`. Run `37783219031`: **10 of 10 authoritative jobs success** —
Rust (windows-latest), Rust (ubuntu-latest), Desktop UI (ubuntu-latest), Desktop UI (windows-latest), Generated
output drift, Dependency policy, macOS Core Smoke, Package Ubuntu, Package macOS, Package Windows. Read with
`gh api repos/2023violet/FirmwareSight/actions/runs/37783219031/jobs`; per-job wall times are in
`09_acceptance/ACCEPTANCE_GATE.json`. No install was attempted before that read.

## 12. Exact artifact, installer and installed executable

| Item | Value |
|---|---|
| Artifact | id `11554271914`, name `FirmwareSight-0.6.0-windows-x86_64`, 5,542,981 bytes, ZIP SHA-256 `94f16ab5c4e0f9b59f27326c3ad49a4a78f0befc57ff3716501999db50baead1` |
| Provenance | belongs to run `37783219031`; `artifact-metadata.json` `toolchain.git_commit` = `2e2e21997ae01f035ac72b97658b3a0b329da362`; internal `SHA256SUMS.txt` all match; downloaded bytes re-hashed against GitHub's own digest |
| NSIS installer | `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, 3,895,064 bytes, SHA-256 `fd2d4935e275e20786690182e3e249bd8be3b772abff94223395080c2bf3f3b6` |
| Installed executable | `firmwaresight-desktop.exe`, 15,367,680 bytes, SHA-256 `196036e22152f702d0f289c9547b9ba12e9f7e8c93f2f0e994541ac0210328e9` |
| Not used | artifacts `11539359554` and `11504871803` (earlier rounds' builds) and any locally built binary |
| Install method | real SendInput through the NSIS wizard, `/S` never used; each grab preceded by a client-area measurement and each window re-placed before 1440-width coordinates were clicked |

## 13. Owner park and backup, before any launch

`02_owner_backup/BASELINE.json` and the before-inventory `SIBLINGS.txt` were written at 13:29:49Z, `PARK.json` at
13:40:55Z — both **before** the installer ran, which `01_artifact/INSTALL.json` records at 13:44:18Z after re-reading
the two flags as YES. The park records the owner's three files (`firmwaresight-p0.sqlite`, `-wal`, `-shm`) with size,
mtime and SHA-256, moves them to `02_owner_backup/backup/`, and confirms the live path is clear:

- `OWNER_STORE_PARKED = YES`, `OWNER_BACKUP_HASH_MATCH = YES`, `baseline_matches_live = true`, and the same two
  values re-read from the live path immediately before launch in `INSTALL.json`.
- The owner's main store at park: 155,648 bytes, SHA-256 `d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468`.
- Only those three names were handled. The 14 unrelated siblings in the same folder were inventoried by name and
  digest before and after and never opened.
- The app was found **not installed** before this round (`PRE_ROUND_MACHINE_STATE.txt`: process absent, executable
  absent), so §12 requires this round to uninstall at the end rather than leave a product behind.

## 14. Installed scenario captures (all on the exact installed bytes)

Every row of `00_authority/CAPTURES.jsonl` carries the requested client size, the client area measured before and
after the grab, the installed EXE digest, its own UTC time and the artifact identity. `measured_matches_request` is
true for all of them. Two grabs were recorded **VOID** and re-taken — the shared main scroll container carried a
stale offset into two navigations — and the void rows stay in the log rather than being deleted; the acceptance gate
filters them out of adjudication.

| Capture (live) | SHA-256 | State it certifies |
|---|---|---|
| `R2_S01_Matched_1440x900.png` | `b51f3c88de122411beb0c80557db012c6fec42df0903a60f3f32236bd6d29735` | current REVIEW for the analyzed selection, judged snapshot named in full |
| `R2_S02_OtherBuild_1440x900.png` | `674dbbebb71b13d4e8dba8cd6a6c33614d68f0f8ac00d5e5899c20c2cce023c4` | `Not assessed for this build`, both ids printed in full |
| `R2_S03_PendingSelection_1440x900.png` | `200bfd7e0a21c0e69d1e592541848a5724b6567397b5370745ad865e8e2c04fc` | the finding's own state, corrected |
| `R2_S03b_PreviousAnalysis_Scrolled_1440x900.png` | `1f21c373768f67316ab4ab2e1f7ec08335d19929c74259f52eb1bf83f686b4ba` | the retained panel below the fold, inside `Previous …` bands |
| `R2_S04_PendingSelection_NoMap_1440x900.png` | `f4303d35715d09c85745e2e6ee981cf0327d34998714638ea7312d05da79a316` | `No MAP attached to this selection`, no pill |
| `R2_S05_ParseFailureAfterLastGood_1440x900.png` | `e7e3119d8654ec80de07bb1ff683168c0459a36ae4b669bd711106979e749c1d` | U1R truth intact, retained work attributed |
| `R2_S06_RecoverySuccess_1440x900.png` | `4660dd0c34051c2856ccba968192fa4a24b6d6684e1300edfe71300184f6679f` | current values restored, nothing sticky |
| `R2_S07_PolicyMismatch_1440x900.png` | `40a7e7624b48425792620fb29a852ec61fc8345cdefbeb089c090324050d7ab8` | `Not assessed under the policy loaded now`, both policy fingerprints |
| `R2_R01_Matched_1024x720.png` | `392671f1faac884754f845643cd454de7312b16c5c9df066289b11b37f9bbad6` | matched state at the small width |
| `R2_R02_PendingSelection_1024x720.png` | `8e985d30d3ec10785be59e52fece321b7699e5b6f691a40fb4518f2f23855097` | pending state at 1024, two cells side by side, nothing clipped |
| `R2_R03_PendingSelection_NoMap_1024x720.png` | `9b2b860b04f01e04f0ad560d6b433349294a770f9bdc90709ba485291659b562` | no-MAP pending at 1024 |
| `R2_R04_PendingSelection_1056x799.png` | `1f6978fdc8cc1767b4144f4938de833a0319320d57d39a315b6e3d33bd694386` | taken because the matched band wraps to two columns at 1024 and leaves one empty cell |

Product-side identities, read back out of the disposable store's own `gate_runs` and `builds` rows (read-only URI)
and length-checked: this round's run is
`gate-d02804939af1ace6f27f1fb5624b7b31352d9c284274014eaf054bb833ca992b` (69 characters), effective severity REVIEW,
created `2026-10-08T13:50:21Z`, judging
`snap-3f615b6247179d930f59b76dcdbaea1a5eca8c835ab6d410feb20a062f33f21f-p0-normalize-1-832690060a8d25f8acacd85a62ce76bc83dfb6768e435bcc04c51f9664ed363e`
(149 characters), which equals the snapshot derived from the committed base fixtures. S7's two policies: run
`b87aa057854d8a0089594ad9445b0626174dc0308d636dd6d6854902ae1e5e98` against loaded
`bcc2f9da1e5a1004428cd98eedffb4a182c2bbf363104db727f8ea3d9c5e83ed`.

## 15. Responsive and regression

`REG_*` captures cover Analyze (pending), Compare, Release Gate, History, History → Gate runs, keyboard focus and the
parse-failure composition on these bytes; all pages keep the decision-first hierarchy U1P established and none
acquired horizontal overflow. At 1024×720 the pending band keeps its two cells side by side with no clipped primary
control. The one cosmetic observation left is the *matched* capability band wrapping to two columns at 1024 and
leaving one empty cell — reported, not fixed, and listed in §18.

## 16. U1P-V2-01 and U1R regression status

- **U1P-V2-01 stays `CLOSED_BY_U1P_R1`.** S1 answers with its own run and judged snapshot; S2 goes neutral with both
  ids in full and R2-T8 keeps that guard; S7 goes neutral under a re-loaded policy and R2-T9 keeps that guard. The
  policy fingerprint of the alternate project, `bcc2f9da…`, is the same value U1P-R1 recorded for it — an
  independent cross-round corroboration that the alt fixture was not altered.
- **U1R's truth stands.** S5: a failed attempt on `not-an-elf.elf` produces no fabricated success. The build-time
  error code the round's notes had expected (`ERR-PARSE-2002`) is **not** what the installed product answers: it
  answers `ERR-FORMAT-0001` — "That file is not a format FirmwareSight can analyze yet", *why we know*
  `unsupported format: Binary at not-an-elf.elf`, *what to do* "Provide an ELF linker output, or a BIN/HEX image",
  diagnostics `op-5e44-18dc92f7299de5b0` — read off the installed Analyze page and recorded in
  `00_authority/DISPOSABLE_PROJECT_NOTE.txt`, which also quotes the build-time sentence unmodified so the two can be
  compared. The scenario still stands, because the input still does what S5 needs; only the code named before the
  file was ever offered was guessed, and a guess about a code is a claim about the product. Diagnostics live in the
  session, not in `gate_runs`/`builds`, which is why this is quoted from the screen and not from the store.

## 17. Owner restore and the safety flags

`02_owner_backup/OWNER_STORE_STATUS.json`, recomputed from BASELINE / PARK / DISPOSABLE_STORE / RESTORE /
SIBLINGS_COMPARE rather than restated:

```
OWNER_STORE_PARKED                    = YES   (before the installer ran)
OWNER_BACKUP_HASH_MATCH               = YES   (before the installer ran)
ORIGINAL_DB_RESTORED                  = YES
ORIGINAL_DB_SHA_MATCH                 = YES
OWNER_STORE_OPENED_BY_U1P_R2          = NO
UNRELATED_SIBLING_STORES_UNTOUCHED    = YES
```

The store the installed app wrote (4,096-byte main `da63a52bdf62b3fc863995d79f497584be3ba97c1de4c55d1d301afbafab6cfc`,
605,672-byte WAL `bc61c44b3bf5688396bdf5db9942cb4945b4f1e9421a7a927ed875a325d6ef41`, 32,768-byte SHM `683e6cdd…`) is a
different database from the owner's by hash and by size; it was hashed first, then moved by name to
`02_owner_backup/disposable_u1p_r2/` at 14:49:27Z, and nothing was deleted. The owner's three files were then put back
(14:49:28Z) and re-hashed equal to the pre-park baseline (`d6e41034…`, `e3b0c442…` for the 0-byte WAL, `fd4c9fda…`).
The 14 unrelated siblings compared name-for-name and digest-for-digest across the 17-entry inventories taken 13:29:49Z
and 14:49:52Z: 0 changed, 0 appeared, 0 vanished (comparison 14:50:47Z, adjudication 14:50:51Z). The uninstaller's
`Delete the application data` option was read through `BM_GETCHECK` as **unchecked** at 14:49:01Z and never activated;
the wizard was then closed with its own `&Close`, and the install directory is gone.

## 18. §7 read-only identity audit, and the one real mismatch

The round was told to audit the two quoted identities read-only, correct only a demonstrable mismatch, and not to
create a separate documentation commit for the audit. Measured against two independent authorities
(`target/u1p_r2_id_audit.txt`, regenerate with `python target/u1p_r2_id_audit.py`):

1. **Authority 1** — `sha256sum` of the four committed fixtures the installed app was fed: every digest is 64 hex
   characters, so a 63-character half cannot be a product identity.
2. **Authority 2** — the product's own SQLite rows, read read-only from the preserved disposable store:
   `gate_runs.id` lengths 69, `builds.id` lengths 155, and the stored build ids carry exactly the two
   fixture-derived snapshots.

Against those, the two strings quoted in the accepted U1P-R1 report are each **one character short**, and the
correction is:

| | value |
|---|---|
| before (run A) | `gate-ff7fae0be1318f32e62eff3a7e216c080d90534a50ec596cc9a1916a1cbea96` (68) |
| after (run A) | `gate-ff7fae0be1318f32e62eeff3a7e216c080d90534a50ec596cc9a1916a1cbea96` (69) |
| before (snapshot B) | `snap-4b4087e1407ceddcb1e1cfd978eedbc8906cf88083ba3b4672c7c225ddd8374-p0-normalize-1-a38575cd51ec2730442a0f3b8506c99fc49eec68d4b28ecdc9dda9434709ace6` (148) |
| after (snapshot B) | `snap-4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374-p0-normalize-1-a38575cd51ec2730442a0f3b8506c99fc49eec68d4b28ecdc9dda9434709ace6` (149) |

Each is the authority with exactly one character deleted (divergence at index 25 and index 27), and neither is a
well-formed identity. The short form occurs once and the true form zero times in the repo-tracked U1P-R1 report and
in the corresponding members of the historical ZIP.

Two things follow, and both are recorded rather than smoothed over. First, the prompt's §7 premise that the full ids
are already identical does not hold: the prompt's own two *expected* strings are the same 63-hex forms, so
document-to-document comparison cannot see the defect — it is mutually consistent copies of one transcription. The
audit therefore takes §7's second branch: the minimum documented location,
`U1_VALIDATION/U1P_R1_OVERVIEW_SUBJECT_CORRECTIVE_REPORT.md` lines 207-208, is corrected inside this round's normal
single docs-only successor, no separate audit commit is made, the already-hashed historical ZIP is neither rewritten
nor repacked, and the baseline validation was rerun. Second, this round's §12 scenario identities were derived from
the authorities rather than copied from prose, which is why they are all 69/149/155 characters long.

## 19. The single shareable pack, and where its historical images came from

`REFERENCE/` (7 mockups), `U1P_BASELINE/` (the 16 accepted U1P primary and responsive images), `U1P_R1_BEFORE/` (R1
S1–S5, its responsive set and the `U1P-V2-01` before-image), `U1P_R2_AFTER/` (this round's 12 captures),
`REGRESSION/`, `SMOKE/`, `SIDE_BY_SIDE/` (4 composites, each labelled DERIVED, with the raw originals retained),
`REPORTS/` (R1's report as hashed, this report, the disposition matrix, the identity matrix, the §7 erratum text,
gate/capture/artifact/install/uninstall/owner records, the archived prompt) and `HASHES/` (SHA256SUMS + source map).
No owner DB/WAL/SHM, no firmware binary, no installer, no nested archive, no private or participant material.

On the §13 requirement that the originals be used rather than regenerated, one present-day fact must be reported
rather than glossed: **both earlier delivery archives had left `C:\Users\16429\Downloads\`**, which is a live user
folder, not an evidence directory. A read-only check (`00_authority/HISTORICAL_PACKS.json`,
`python target/u1p_r2_historical_packs.py`) found both in the Recycle Bin, each carrying the same recorded deletion
time-stamp `2026-10-08T14:34:48Z` — after this round's captures and before its pack — and re-hashed them there:

- `FirmwareSight_U1P_Final_Visual_Review.zip` — 4,727,314 bytes, SHA-256
  `0f59f4cee90bd31e388e90e0b2097a621ab0c26811fbf50162b84f0f17f3881b` — **matches** BASELINE.yaml;
- `FirmwareSight_U1P_R1_Overview_Subject_Review.zip` — 1,381,951 bytes, SHA-256
  `342fb89be6baf9f4e1670edba901ace57576112ec680af5e0e1f6cb3dee299a6` — **matches** BASELINE.yaml and the R1 round's
  own `ZIP_IDENTITY.json`.

This round restored nothing, moved nothing and deleted nothing in the bin. Both rounds' staging directories are
intact as a second source (U1P: 59 staged files with its own `HASHES/SHA256SUMS.txt`; U1P-R1: 38 staged files with
its own manifest), and the pack's `HASHES/SOURCE_MAP.txt` names the exact archive member or staged file each
historical image came from. So the
six-page baseline this verdict needs is the delivered bytes, not a regeneration — **but whether a moved delivery copy
is itself a §13 gap is the Architect's call, not this round's**; if it is, the pack can be re-issued from the bin
copy without touching a single pixel.

## 20. What is left, and what this round may not say

Carried forward unchanged, for the Architect's triage, with no product change attempted here (§15's out-of-scope
list): `U1P-V1-01` and `U1P-V1-02` (the Compare result not surviving navigation — a V1-class observation),
`U1P-V0-01` (the matched capability band's empty cell at 1024), the Analyze table at the 1024 fold, FS-UI-07
dependency features, and the Git capability's Unknown-versus-`git.clean` question. **V1 remains paused, not closed**
(`stage_status IN_PROGRESS`, `research_state RECRUITMENT_READY`, 0 sessions run), and the frozen F3 research artifact
is unchanged. No B1, no Beta, no RC, no GA was claimed or implied by anything in this round.

The acceptance gate for this round is `09_acceptance/ACCEPTANCE_GATE.json`: 20 boxes PASS and 1 box
MISMATCH_PROVED, `all_boxes_hold = true`, conclusion "U1P-V2-02 is closed by this round's installed evidence".
That is a statement about U1P-V2-02 only. **U1 is `READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT`.** `PASS_COMPLETE`,
`VISUAL_ACCEPTED`, `MOCKUP_MATCHED`, V1 PASS, B1, Beta, RC and GA are not states an agent may issue, and the visual
verdict belongs to the Architect reading the pack.
