---
title: "U1R corrective report: stale capability state on the Analyze page"
doc_id: "FS-U1R-002"
product: "FirmwareSight"
stage: "U1R (corrective inside the U1 UI productization track)"
canonical_unit: "U1R_STALE_CAPABILITY_CORRECTIVE_AND_FINAL_RECHECK"
related:
  - U1_VALIDATION/U1R_CAPABILITY_STATE_DESIGN.md # FS-U1R-001, written before the installed recheck
  - U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md # where U1-V2-06 was raised
---

# U1R corrective report

Eighteen parts, in the order prompt §22 asks for them. Every number here was produced by a command in this
session and is quoted with the command that produced it. Evidence root (outside Git, retained):

```text
C:\Users\16429\AppData\Local\Temp\FirmwareSight-U1R-Visual-Recheck-20261007T145124Z
```

## 1. Start authority

The prompt's §0 was checked against the machine rather than trusted:

| claim in §0 | measured | how |
| --- | --- | --- |
| `origin/main = 6f35b144d4c161390fba0c1b2f15b1bbdd5d728f` | equal | `git ls-remote origin -h refs/heads/main`, `git rev-parse HEAD origin/main` — all three the same, tree clean |
| candidate product head `7dc2ca80e5273b9aac48bb7ca216d24c2c1585ce` | its parent chain is on `main`; `6f35b14` is its child | `git log --oneline` |
| candidate remote CI run `37609108402`, #74, attempt 1, 10/10 | confirmed | `gh run view 37609108402 --json …` → `completed / success`, headSha `7dc2ca8…`, 10 jobs at `success` |
| evidence successor run `37619704397`, #75, attempt 1, 10/10 | confirmed | same command → headSha `6f35b14…`, 10 jobs at `success` |
| Rust = 868 passed / 0 failed | 868 / 0 | `cargo test --workspace` on this tree before any edit |
| UI = 236 passed / 9 files | 236 / 9 | `corepack pnpm test` |
| U1 = `READY_FOR_ARCHITECT_VISUAL_REVIEW` | as recorded | `U1_VISUAL_ACCEPTANCE_REPORT.md` §15 |
| U1-V2-06 = OPEN | as recorded | `U1_VISUAL_ACCEPTANCE_REPORT.md` §12 |
| V1 paused with 0 eligible sessions; V3/V4 not started | as recorded | `BASELINE.yaml` `v1_execution`, `.ai/CURRENT_STATE.md` |

No remote movement had to be classified: the round opened on the head the prompt names.

## 2. Root cause

`apps/desktop/ui/src/Analyze.tsx` resolved its visible summary as the stored last-good one
(`const summary = lastGood;`) and rendered the top capability strip from
`summary.capabilities.elf / .map / .git` whenever `summary !== null`, with no conditioning on whether that
summary belonged to the current attempt. Reachable wrong read, confirmed in the code before editing and
reproduced on the installed app in the U1 round: analyze `firmware.elf` with its MAP, then select a second
file and let its analysis fail, and the page keeps a green "ELF supported / MAP provided / Git unknown"
strip — the previous file's facts — above a row naming the new file with `MAP: Not provided` and an error
panel that says the parse failed.

Retention was not the defect and was not removed. The prompt's warning ("Do NOT 'fix' it by deleting
last-good retention") is honored: `lastGood` still moves only when the shell returns a summary, the retained
report still renders, and the Details tables still render the last good snapshot.

## 3. UI contract

Prompt §5 A–E, and what the shipped page now says in each case:

| case | required read | shipped wording |
| --- | --- | --- |
| A current success | chips from the current summary | `ELF supported` / `MAP provided` / `Git unknown` as `StateBadge variant="chip"` |
| B no analysis yet | neutral, nothing invented | `No artifact selected yet` |
| C selection moved, not analyzed | not the last-good chips | `<file> selected, not analyzed yet`, plus `Previous result retained below` when a summary exists |
| D current attempt failed | not the last-good chips | `Current analysis failed`, plus `Previous result retained below` |
| E retained report and Details | stay visible, stay labelled | the report band's "Previous analysis of X. It is not an analysis of Y…" and Details named `Snapshot details of the last good analysis` while the attempt is not authoritative |

No new domain state. Nothing was mapped onto PASS / REVIEW / BLOCK / UNKNOWN / N/A that was not already
mapped by `capabilityState()`; the failure and pending readings are neutral `Chip` text, not a verdict. This
is presentation state, and the Gate page's verdict vocabulary is untouched.

## 4. Code change

One derived value, in `Analyze.tsx`:

```tsx
const currentSummary =
  summary !== null && error === null && !pendingSelection ? summary : null;
```

That is the prompt's §6 shape. `currentSummary` feeds the header meta line and the capability strip;
`summary` continues to feed the retained report and `Details`, so no fact a reader could already see was
rewritten or hidden. `pendingSelection` and `error` are the page's existing reads of shell state. There is no
new IPC call, no mutation of the stored value, no duplicated domain logic, no cleared history, and no
backend, storage, schema, wire-DTO, release-identity, Gate or bundle change.

`Details.tsx` gained one optional prop, `stale` (default `false`), used only to name its region; `Analyze`
passes `stale={error !== null || pendingSelection}` — the same condition the report region already uses for
its `Last good analysis` label, so one screen cannot call the same snapshot current in one place and
previous in another. The strip's container also gained `role="group"`, which `overview.test.tsx` already
depended on for the same label on that page.

Changed product paths, all three inside the UI layer:

```text
apps/desktop/ui/src/Analyze.tsx        +50 -21
apps/desktop/ui/src/Details.tsx        +12 -1
apps/desktop/ui/src/intake.test.tsx    +142 -0
```

`git diff --quiet HEAD -- assets/design-tokens.json` exits 0 and the file's digest is the same value in the
commit and in the worktree (`94336906997d23b1f36cbaf81711c6824082d88b18aa89c38dd12721564a14d4`), so the
frozen token file is byte-identical. No token value, no CSS rule and no component was added; `Chip` and
`StateBadge variant="chip"` were already this page's vocabulary.

## 5. Test and mutation proof

Six tests in `apps/desktop/ui/src/intake.test.tsx` under
`describe('U1R capability strip: current attempt, not retained evidence')`, driving the existing
`selection()` / `summary()` / `chooseArtifact()` / `analyze()` stubs:

| id | asserts |
| --- | --- |
| T1 | after a success the strip states `ELF supported`, `MAP provided`, `Git unknown` and the header carries the current figures |
| T2 | moving the selection without analyzing replaces those words with `other.elf selected, not analyzed yet` and removes every capability word |
| T3 | a failed attempt shows `Current analysis failed` and `Previous result retained below`, with no borrowed chip in the strip |
| T4 | the retained report and Details stay visible and keep their Previous / last-good accessible names |
| T5 | the failure that actually happened is still reported verbatim, including `ERR-PARSE-2002` |
| T6 | a later success returns the current words, so the strip is not a one-way latch |

All six are semantic and accessibility assertions — text, roles and accessible names — not pixels.

Mutation proof: with the derivation reverted to the shipped-bug form (`const currentSummary = summary;`) and
the tests untouched, `corepack pnpm test -- src/intake.test.tsx` reported

```text
❯ U1R capability strip: current attempt, not retained evidence (6)
  ✓ T1   × T2   × T3   ✓ T4   ✓ T5   × T6
Tests  3 failed | 26 passed (29)
```

so T2, T3 and T6 fail on the pre-fix behavior. T1, T4 and T5 stay green under the mutation because they
assert behavior the bug also got right (T1, T4) or a panel the mutation does not touch (T5). The mutation was
then undone by digest rather than by hand:

```text
$ sha256sum -c target/u1r_pre_mutation.txt
Analyze.tsx: OK
Details.tsx: OK
intake.test.tsx: OK
$ grep -c MUTATION-PROOF Analyze.tsx Details.tsx intake.test.tsx
0 / 0 / 0
```

No mutation code is in the repository; the logs live under gitignored `target/`.

## 6. BASELINE reconciliation

Two stale facts corrected, both in `u1_execution`, and neither rewritten as if the other round had measured
it:

1. **Command count.** `new_commands_added: 0` stays 0, but the total it guards is **30**, not 27. Measured by
   extracting the `generate_handler!` list from `apps/desktop/src-tauri/src/lib.rs`: 30 entries at HEAD, 30
   already at `f481b78` (U1's own start head), and the list reached 30 in `3400981` during P5. So the 27 was
   stale before U1 opened and U1 added nothing to it. The prompt's instruction is honored literally: U1 did
   not add three commands, and the record no longer implies the total was ever 27 at this point in history.
2. **core-smoke and package.** The first-round sentence stays true for the first round and now says so
   ("were not run during the initial U1 implementation round"). A new dated sibling,
   `gates_continuation_evidence`, carries what the continuation measured — `--only core-smoke` 3 of 3,
   `--only package` 4 of 4, no required SKIP — and a third line says the current evidence is measured, not
   recalled. `u1r_recheck` records this round's own counts (Rust 868, UI 242 in 9 files, 30 commands,
   token digest, mutation outcome, scope, and what U1R never self-issued).

## 7. Report path correction

`U1_VISUAL_ACCEPTANCE_REPORT.md` §14 listed `OWNER_RESTORE.json` under `02_owner_backup/`. `ls` of the U1
evidence root puts it in `13_owner_restore/` and leaves only `RESTORE.json` in `02_owner_backup/`, so the
listing was corrected and a dated note added under the tree. The owner-restore verdict did not move with it:
`13_owner_restore/OWNER_RESTORE.json` still records `ORIGINAL_DB_RESTORED = YES` and
`ORIGINAL_DB_SHA_MATCH = YES`, and no other word in that report's findings changed. This is a path
correction only.

## 8. Local validation

| command | result |
| --- | --- |
| `git diff --check` | exit 0 (two CRLF-normalization warnings, no whitespace errors) |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --workspace` | 47 result lines, **868 passed / 0 failed** — unchanged, so the prompt's STOP-on-Rust-count condition was never reached |
| `corepack pnpm install --frozen-lockfile` | exit 0 |
| `corepack pnpm typecheck` | exit 0 |
| `corepack pnpm lint` | exit 0 |
| `corepack pnpm test` | **242 passed in 9 files** (was 236 in 9; six added, none deleted) |
| `corepack pnpm build` | exit 0 |
| `python scripts/check.py` | **17 of 17 steps PASS, no SKIP** (rust 3, frontend 5, drift 8 including `drift/baseline integrity`, deny 1) |
| `python scripts/check.py --only drift` | 8 of 8 |
| `python scripts/check.py --only deny` | 1 of 1 |
| `python scripts/check.py --only core-smoke` | 3 of 3 |
| `python scripts/check.py --only package` | 4 of 4 (`frontend deps`, `cli companion`, `desktop package`, `artifacts verified`) |
| `python scripts/verify_baseline_artifacts.py` | RESULT PASS — 755 tracked files, 753 sum entries, every mismatch counter 0 |

The ADR-0029 order was followed: edit → stage → regenerate `DIRECTORY_TREE.txt` → stage → regenerate
`SHA256SUMS` → stage → verify → gate.

## 9. U1R product commit

```text
4d36f103d2a8fd103e8e3b466e4d2b2010a0a4ef
U1R: keep stale capability state out of failed Analyze attempts
```

Nine paths, and the product diff is not buried in governance edits:

```text
10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1R_Stale_Capability_Corrective_Final_Recheck_v1.0.txt  (new, 1302 lines)
BASELINE.yaml                                         |  25 +-
DIRECTORY_TREE.txt                                    |   4 +-
SHA256SUMS                                            |  14 +-
U1_VALIDATION/U1R_CAPABILITY_STATE_DESIGN.md          | 135 ++ (new)
U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md          |  10 +-
apps/desktop/ui/src/Analyze.tsx                       |  71 +-
apps/desktop/ui/src/Details.tsx                       |  13 +-
apps/desktop/ui/src/intake.test.tsx                   | 142 +++
```

Pushed as a normal fast-forward: `6f35b14..4d36f10 main -> main`, with `git ls-remote` read before
(`6f35b14…`) and after (`4d36f10…`). No force, no history rewrite, one commit for the corrective.

## 10. U1R remote CI

```text
run 37637056980   #76   event push   attempt 1   completed / success
headSha 4d36f103d2a8fd103e8e3b466e4d2b2010a0a4ef
url https://github.com/2023violet/FirmwareSight/actions/runs/37637056980
```

10 of 10 authoritative jobs, each read individually from the run's step list:

| job | conclusion | the figure this round cares about |
| --- | --- | --- |
| Rust (ubuntu-latest) | success | 47 result lines, 868 passed, 0 failed |
| Rust (windows-latest) | success | 47 result lines, 868 passed, 0 failed |
| macOS Core Smoke | success | 36 result lines, 646 passed, 0 failed (the core subset, as designed) |
| Desktop UI (ubuntu-latest) | success | 242 passed, 9 test files |
| Desktop UI (windows-latest) | success | 242 passed, 9 test files |
| Generated output drift | success | 8/8, including `drift/baseline integrity` → `RESULT PASS`, 755 tracked files / 753 sum entries on a clean checkout |
| Dependency policy | success | 1/1 |
| Package Windows | success | step 7 "Build and verify the Windows package", step 8 name prefix, step 9 "Upload the artifact set" |
| Package macOS | success | build and verify, upload |
| Package Ubuntu | success | build and verify, upload |

Attempt history is preserved: this is attempt 1 of run 76 and no job was re-run. Nothing was a
repository-content failure, so there was nothing to blind-rerun.

## 11. Exact Windows artifact

Downloaded from run 76 — not the U1 artifact `11477857379`, not the F3/V1 artifact, not the local
`target/dist-package` the `package` group produced, not `cargo run`, not Vite.

```text
artifact id    11491960105
artifact name  FirmwareSight-0.6.0-windows-x86_64
zip bytes      5,538,922
zip sha256     2627f405f7a8bfc0eda6b7dadb93dc632f446b2b6cfb2b6fb1c441b91eb01f51
               (equals the artifact's own `digest` field from the GitHub API)
NSIS file      FirmwareSight-0.6.0-windows-x86_64-nsis.exe
NSIS bytes     3,890,969
NSIS sha256    41f26a8c50aaa25e2cc1e2592d3263f547e3512667546fcc82ce5241259e9f79
               (verified: `sha256sum -c SHA256SUMS.txt` inside the artifact -> OK for both entries)
metadata       artifact-metadata.json -> toolchain.git_commit = 4d36f103d2a8fd103e8e3b466e4d2b2010a0a4ef
installed exe  %LOCALAPPDATA%\FirmwareSight\firmwaresight-desktop.exe
               15,366,144 bytes, sha256 e62652bb6c7e0c5684c80064757d79ec4423a5bf126978e2b9be270999daa8db
```

The installed digest differs from the metadata's `payload_sha256`
(`2349fcc8ea62bae9a29f14fd23e7ba1529f4c388ded4e801dd399edf6e081e34`) for the reason `artifact-metadata.json`
states itself: the bundler rewrites a 27-byte bundle-type token in the copy it packs, so the payload digest
locates the build and is not the digest of the file a user launches.

Installation used real mouse input on the NSIS wizard (Next, Next, Install, Next, Finish) with the
pixel-ownership check on every press; no `/S`, no silent flag. The shared installer helper stopped at the
progress page because it asks that page for a "Close" button, which this build does not show until "Next"
advances to the completion page whose button reads "Finish"; the same installer process was finished by a
follow-up script rather than restarted, and both logs are in the pack.

## 12. Owner-store safety

Parked before installation, restored after, with the independent backup kept. Five required fields:

```text
OWNER_STORE_PARKED        = YES
OWNER_BACKUP_HASH_MATCH   = YES
ORIGINAL_DB_RESTORED      = YES
ORIGINAL_DB_SHA_MATCH     = YES
OWNER_STORE_OPENED_BY_U1R = NO
```

The owner's pre-round bytes were `firmwaresight-p0.sqlite` 155,648 B
(`d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468`), `-wal` 0 B
(`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`) and `-shm` 32,768 B
(`fd4c9fda9cd3f9ae7c962b0ddf37232294d55580e1aa165aa06129b8549389eb`). All three were re-hashed after the
restore and match, with their original mtimes. They were not at the path the product reads for the whole
round, so the product could not have opened them; nothing migrated, seeded, repaired or read them.

The app ran against a disposable store it created itself (4,096 / 552,112 / 32,768 bytes, digests in
`PRE_RESTORE_LIVE.json`), which was removed by explicit name after its digests were re-measured and
confirmed not to be the owner's. The uninstaller's "Delete the application data" option was never clicked;
its ticked-or-unticked state is not claimed, because the style probe returns no checkbox-class control on
that page and an assertion there would pass vacuously. The app-data folder went from 17 entries to 14 — the
14 that predate this round, untouched.

Machine state is back where it started: the product is uninstalled, as the pre-round snapshot recorded.

## 13. Installed recheck

Focused, per §18; the full U1 matrix was not re-run.

| scenario | result |
| --- | --- |
| A Analyze success | `firmware.elf` + MAP analyzed: 19 sections, 52 symbols, sha256 `4b4087e1…5ddd8374`; strip reads `ELF supported` / `MAP provided` / `Git unknown` |
| B no analysis yet | observed on first launch: `No artifact selected yet`, header `no artifact selected` |
| C selection moved, not analyzed | `truncated-elf.bin selected, not analyzed yet` + `Previous result retained below`; no borrowed chip; the retained band names `firmware.elf` |
| D parse failure after a prior success | `ERR-PARSE-2002`, "Could not parse artifact as ELF.", diagnostics ids `op-6e8c-18dc475f1d6bb6c0` (1440×900) and `op-6e8c-18dc477dcdb9a6b8` (1024×720) |
| D at three sizes | 1440×900, 1056×799, 1024×720 — all seven §18-C conditions hold simultaneously in each; the per-size table is in `U1R_STATE_CONSISTENCY_MATRIX.md` |
| E retained report and Details | both still render after the failure, with the report band's own sentence and the Details region carrying the last-good snapshot id `snap-3f615b62…` |
| D recovery | `base/firmware.elf` + MAP analyzed: 19 sections, **47** symbols, sha256 `3f615b62…33f21f`, 16,820 bytes — the current chips return, over figures that can only be the new summary's |

Two things are recorded rather than smoothed over:

* One capture attempt at 1024×720 showed the History page although the resize immediately before it had read
  `window_title=FirmwareSight - Analyze` and no click was issued in between. The cause was not determined.
  The deliverable S04 is a second attempt, reached by an explicit Analyze click whose title read-back said
  `FirmwareSight - Analyze`, and it shows the failure state.
* The transient error clears if the reader navigates away and back: Analyze then reads
  `truncated-elf.bin selected, not analyzed yet` rather than `Current analysis failed`. That is the shell's
  existing session behavior, unchanged by U1R, and truthful in both readings.

## 14. Screenshot manifest

```text
CURRENT/U1R_S01_AnalyzeSuccess_1440x900.png    1440x900  1d1e384fdf27db5fd407bef75c18eeccce975bcd88c64256655fbc13023cca1b
CURRENT/U1R_S02_ParseFailure_1440x900.png      1440x900  79376ea665645de6e76995f5a1fb3eb98b21daba546a20c7a3acd293e322c023
CURRENT/U1R_S03_ParseFailure_1056x799.png      1056x799  a47e43f88e5e5eec423c9a07c15d36251730c5c2ee7d453f620dcea8aa9d649b
CURRENT/U1R_S04_ParseFailure_1024x720.png      1024x720  f9114f29de781b3720ab0570abdd112af34ef7b06812c95c0cb872e1b109fdd5
CURRENT/U1R_S05_RecoverySuccess_1440x900.png   1440x900  0b893006eba60ab8e8f1bd3d0bec05644dbf465a5586924c60ae4b50fbf3ea17
CURRENT/U1R_S06_PendingSelection_1440x900.png  1440x900  973205db9c756ae9b26ead46fcc49ac1392ddd5b55ab2e811d3764459adccff8
```

All six names §19 asks for exist, including S06: the pending-selection state was meaningfully capturable, so
it was captured. Ten `work_*.png` probes (initial state, selection, MAP attached, Details after failure, the
Details tabs, and the smoke navigations) sit beside them, and `SHA256SUMS.txt` indexes all 51 files in the
pack, each verified by `sha256sum -c`.

## 15. U1-V2-06 disposition

`U1-V2-06 = CLOSED_BY_U1R`. The nine conditions §20 lists, each against its evidence:

| # | condition | evidence |
| --- | --- | --- |
| 1 | exact U1R CI artifact installed | §11: artifact `11491960105` from run 76 at `4d36f10…`, NSIS digest verified against the artifact's own index |
| 2 | parse failure reproduced after a previous success | S01 then S02/S03/S04 |
| 3 | current failure visible | the "Analysis failed" panel with `ERR-PARSE-2002` in all three |
| 4 | stale previous capability chips no longer look current | the strip holds two neutral chips and no capability pill at any of the three sizes |
| 5 | retained previous analysis remains clearly labelled | the band sentence plus the `Previous result retained below` chip, and Details' accessible name (T4/T5) |
| 6 | recovery restores normal current chips | S05, over figures only the new summary can produce |
| 7 | 1440 / 1056 / 1024 layouts remain usable | matrix §"seven conditions", per size |
| 8 | focused tests pass | 242/9 locally and on both CI UI jobs; T1–T6 green, T2/T3/T6 red under mutation |
| 9 | no new U1-V2 / V3 / V4 introduced | none observed; the two items in §13 and the Details caption are V1-or-below wording and observation, listed in part 16 |

The two observations that are not state defects are recorded in the matrix and in part 13, and one of them is
a candidate for the designer: the Details region's accessible name says "of the last good analysis" while the
visible caption under its tabs still says "Sections of the analyzed artifact".

## 16. Remaining U1-V1 minor findings

Unchanged and unimplemented by this round, exactly as §12 requires:

```text
U1-V1-01  Overview's stacked state cards push the metrics below the fold
U1-V1-02  Analyze leads with fact lists instead of its tables
U1-V1-03  no paired right-hand detail on Analyze, Compare, Release Gate or History
U1-V1-04  the page heading "History" against the rail and window title "Bundle & History"
U1-V1-05  the scrolling rail
U1-V1-07  RankBar labels running name and role together (.debug_infoDebug)
U1-V1-08  wide tables' last column behind a table-scoped scrollbar at small sizes
U1-V0-09  the product bar's project chip present on some pages and absent on others
```

Disposition: `OPEN_FOR_ARCHITECT_VISUAL_JUDGEMENT` — not `CLOSED`, not `IGNORED`. One further V1-class wording
item is added by this round's observation (the Details caption under a failed attempt) and is offered to the
same judgement rather than fixed here, because fixing it would widen the scope §10 set.

## 17. V1 interlock

V1 is untouched: `stage_status` stays `IN_PROGRESS`, `research_state` stays `RECRUITMENT_READY`, eligible
external sessions stay **0**, participant execution stays `PAUSED_FOR_ARCHITECT_UI_REVIEW`, and the cohort
artifact stays the frozen F3 build (head `08fdfcb710f78f8084bfcf614dc508c8b6e7e25b`, run `37475580080`,
artifact `11419727517`), which this round neither re-baselined nor re-downloaded. No session was run, no
participant recruited, no M1–M6 metric definition changed. Whether to resume V1 on the current UI, hold it
for another visual round, or resume on the F3 build is the Architect's choice among the options §24 lists,
and U1R took none of them.

## 18. Recommendation

```text
U1 = READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT
```

The strongest statement this evidence supports: **the one material presentation defect U1 left open was fixed
in the presentation layer only, guarded by six contract tests that demonstrably fail without the fix, shipped
through a 17-step gate and a 10-job CI run, installed from the bytes CI built from that commit, and verified
on a real screen in the success, pending, failed and recovered states at three window sizes — with the
owner's data byte-identical before and after.**

It is **not** `PASS_COMPLETE`, not `DESIGN_COMPLETE`, not `MOCKUP_MATCHED`. The Architect has still not
looked directly at the U1 or U1R screenshot pack, and the eight items in part 16 are exactly the kind of
judgement that look belongs to. Nothing here issues a product stage: not B1, not Private Beta, not RC, not
GA, not a tag, not a GitHub Release, not a public installer, not signing, notarization, an updater, a licence
choice, pricing or cloud.
