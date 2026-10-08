---
title: "U1P-R1 Overview gate-subject consistency: corrective report"
doc_id: "FS-U1P-R1-002"
product: "FirmwareSight"
unit: "U1P_R1_OVERVIEW_GATE_SUBJECT_CONSISTENCY"
closes: "U1P-V2-01"
status: "BASELINE"
authority: "10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R1_Overview_Gate_Subject_Consistency_v1.0.txt + AGENTS.md 3, 8, 9, 11 + DESIGN.md + ADR-0029"
plan: "U1_VALIDATION/U1P_R1_SUBJECT_CONSISTENCY_PLAN.md (FS-U1P-R1-001, written_before_code: true)"
---

# FS-U1P-R1-002 — what changed, what was proved, and what is left to the Architect

This unit closed **U1P-V2-01**: Overview's dominant "Can we ship now?" answer could belong to a Gate run for one
build while the page's header, capability band and metric band described another. It is a subject-binding and
presentation-correctness corrective. No verdict is computed here, no Core fact moved, and no capability was
invented.

## 1. Start authority

| Item | Value | Measured by |
|---|---|---|
| `origin/main` at start | `a2e6b3030163d36ad1d67252fd55d5ea9ab34d6c` | `git rev-parse origin/main` after `git fetch --prune origin` |
| Local `HEAD` at start | same | `git rev-parse HEAD` |
| Worktree at start | clean; the configured Git content filters make `git status` uninformative about cleanliness, so cleanliness was asserted from `git diff --binary --stat` being empty and from the two identity checks above | `git diff --binary --stat` |
| Prior CI authority | run `37737545060` (#80) 10/10 at that head | `gh run view 37737545060` |
| Prompt delivered | 33,484 bytes, SHA-256 `0263e0fe92b7065fcc6a8cf1280630ff753eb2e4415ecb07fd40c72bd8e8d024` | `wc -c` / `sha256sum` on the Downloads file |
| Prompt archived | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R1_Overview_Gate_Subject_Consistency_v1.0.txt`, blob `df5b2d0dcc28e2438861c26e00ae5b0369a3e8a3`; delivered bytes == stored bytes, measured with `git show :path \| sha256sum` rather than assumed | both digests in §2 of the plan doc |

## 2. Verified code root cause

The defect was read in the product before it was changed, and it is an ownership gap rather than a wrong string:

- `App.tsx` holds `gateRun` and the last successful analysis as **independent** state. `gateRun` is written only
  by the Release page's `onRunChange`; `lastGood` and `analyzedSelectionId` are written together only on a
  successful `analyzeSelection`.
- `App.tsx` passed `analyzedSelectionId` to `Analyze` — which had used it for its own pending badge since before
  this round (`Analyze.tsx:136`) — and **did not pass it to `Overview`**. `Overview` therefore could not ask the
  one question its dominant element depends on: *did the run in my hand judge the build I am describing?*
- `Readiness` rendered the verdict strip whenever `gateRun !== null`. Nothing was false — the run id was printed
  and the panel scoped itself to "the run named above" — but the pairing was unverifiable from that page.

## 3. Derived state and identity table (the shipped shape)

`readinessScope()` in `Overview.tsx` is read-only presentation state. It decides nothing about a build; it asks
whether identities the shell already owns agree.

| State | Condition, in precedence order | What the dominant slot says |
|---|---|---|
| `selectionPending` | a selection exists whose `selectionId` ≠ `analyzedSelectionId` | UNKNOWN chip "Not assessed for the selected artifact" |
| `noAnalysis` | no `AnalysisSummaryDto` in this session | UNKNOWN chip "No analysis in this session" |
| `otherBuild` | `gateRun.snapshotId` ≠ `summary.identity.snapshotId` | UNKNOWN chip "Not assessed for this build" |
| `otherPolicy` | a project is loaded and `project.policySha256` ≠ `gateRun.policySha256` | UNKNOWN chip "Not assessed under the policy loaded now" |
| `current` | none of the above | the run's own verdict sentence and counts, unchanged from U1P |

Substitutes that were **rejected** as evidence of subject identity, because each can agree while the subjects
differ: truncated ids, file names, timestamps, commits, byte sizes, and the run's baseline. The comparison is on
full identities only, and it is direction-agnostic — the U1P finding had the run on the *target* and the page on
the *base*; this recheck drove the opposite direction and was caught the same way.

## 4. Exact changed paths, and what was not touched

`git show --stat 42f75a7` — 9 files, 1,537 insertions, 13 deletions:

| Path | Δ | Nature |
|---|---|---|
| `apps/desktop/ui/src/Overview.tsx` | +203/−4 | the derived scope, the four neutral wordings, `CurrentReadiness` / `NotCurrentReadiness`, the full judged-snapshot line |
| `apps/desktop/ui/src/App.tsx` | +5 | one prop, `analyzedSelectionId={analyzedSelectionId}` |
| `apps/desktop/ui/src/Overview.module.css` | +24 | two classes, `.identity` and `.pastRun`, both token-valued |
| `apps/desktop/ui/src/overview.test.tsx` | +458 | thirteen contract tests |
| `U1_VALIDATION/U1P_R1_SUBJECT_CONSISTENCY_PLAN.md` | +168 | the plan, written before code |
| `10_AUDIT/SOURCE_PROMPTS/…v1.0.txt`, `…/README.md` | +654 / +22 | prompt archive |
| `DIRECTORY_TREE.txt`, `SHA256SUMS` | +2 / ±14 | ADR-0029 regenerated baseline artifacts |

**No** `crates/**`, `apps/desktop/src-tauri/**`, `apps/desktop/ui/src/ipc/generated/**`, schema, migration,
storage, Gate or analysis semantics, release identity, bundle contents, package/CI topology, dependency, or
design token changed. `assets/design-tokens.json` is byte-identical (`git diff --stat` empty for that path). No
UI state is persisted that was not persisted before. No capability was invented to make a state renderable: the
policy-mismatch state reuses the two fingerprints the shell already had, and the pending state reuses the handle
Analyze already had.

## 5. Tests, count delta, mutation proof

- Thirteen new tests, all defect-guarding, taking `overview.test.tsx` from 10 to 23.
- **Red before green:** five of the thirteen failed against `a2e6b30`'s unmodified `Overview.tsx` before any code
  changed (`target/u1p_r1_red_run.txt`: `5 failed | 10 passed (15)`).
- **Mutation proof:** with only the subject dispatch deleted from the finished file, eight fail
  (`target/u1p_r1_mutation_run.txt`: `8 failed | 15 passed (23)`). The guard is load-bearing in the product, not
  in the test file.
- **Wiring proof:** removing the prop from `App.tsx`'s `<Overview>` call breaks the authoritative typecheck with
  `TS2741`, so the feed cannot be silently dropped.
- Rust: **868 passed / 0 failed** across 47 result lines, unchanged — this unit added no Rust test and removed
  none. UI: **253 → 266** across the same 9 files.

## 6. Local gates (ADR-0029 order: stage → regenerate → verify → check → commit)

`check.py` **17/17**; drift **8/8**; deny **1/1**; core-smoke **3/3**; package **4/4**; baseline verifier
`RESULT PASS` at **889** tree lines and **762** sum entries over **764** tracked paths. Frontend typecheck, lint
and test all clean. No required SKIP.

## 7. Product commit, the amend disclosure, and the first CI

Product commit **`42f75a7d2d90a4c25ca1a3c5c91f6ece16c2d5bf`**, pushed as a normal fast-forward
(`a2e6b30..42f75a7`) after the operator's separate direct authorization message.

**Disclosure, required by §9's "no amend" rule:** the commit message was amended **once, before any push**, to
correct a wrong figure it had shipped ("768 tracked files" → the measured 889/762/764). The tree was
`5291183eb0ee3445949f38babb4a9d3a6123683e` both before and after, so no content changed; both entries remain in
the reflog. Nothing published was rewritten — the first push happened after the amend, and there has been no
force-push, rebase or history edit since.

Remote CI **run `37753793004`** (workflow run #81), attempt 1, `completed` / `success`, **10/10**:

| Job | Job ID | Duration |
|---|---|---|
| Dependency policy | 113233228565 | 1m37s |
| Generated output drift | 113233228482 | 2m39s |
| Rust (ubuntu-latest) | 113233228312 | 4m3s |
| Rust (windows-latest) | 113233228519 | 7m29s |
| Desktop UI (ubuntu-latest) | 113233228423 | 32s |
| Desktop UI (windows-latest) | 113233228462 | 1m19s |
| macOS Core Smoke | 113233228486 | 1m46s |
| Package Ubuntu | 113233228409 | 11m39s |
| Package Windows | 113233228081 | 16m16s |
| Package macOS | 113233228334 | 18m7s |

Each job's own `startedAt`→`completedAt` window was read from the API rather than inferred from the conclusion;
the UI jobs report 9 files / 266 tests in their logs.

## 8. Artifact provenance

Windows artifact **11539359554** (`FirmwareSight-0.6.0-windows-x86_64`) from that run and no other:

| Item | Bytes | SHA-256 |
|---|---|---|
| artifact ZIP | 5,539,797 | `45922e90d1ca19b1fa42966497f9ec262cd24d123f769a08f4d1b24e1aec56d8` |
| NSIS installer | 3,891,883 | `dd5c8de8efbc8926845b42a73c7670e324cea8804cb25e2b586672886c2cd5aa` |
| installed `firmwaresight-desktop.exe` | 15,367,168 | `3bf541bbac3d344a0cd39eefb43ed4b1841b2661efc0f7a1539fc647d35e39d3` |

`artifact-metadata.json`'s `toolchain.git_commit` equals the product head; the ZIP's internal `SHA256SUMS.txt`
verified all four payloads. GitHub's `sha256_digest` field was `null` for this artifact, so the download was
checked against the live API digest, the toolchain commit, the internal manifest and the byte count instead —
a non-circular check, recorded in `01_artifact/ARTIFACT_IDENTITY.json`. The installer was driven page by page
with real `SendInput` (no `/S`, no message injection); `01_artifact/INSTALL.json` carries the wizard trail and
the digest of the bytes actually launched, which was re-hashed from disk before it ran.

## 9. Owner store protection and restore

The owner's live store was recorded (size, mtime, full SHA-256), independently backed up and hash-verified, then
moved out of the live path **before** install or launch, with `OWNER_STORE_PARKED = YES` and
`OWNER_BACKUP_HASH_MATCH = YES` asserted first. The installed app therefore wrote its own new database.

After testing the app was closed, the product uninstalled through its own wizard, and the disposable store this
round created was hashed and moved out by file name. Nothing was deleted, the broad "delete the application data"
option was read through `BM_GETCHECK`, found unticked, and never activated.

```
ORIGINAL_DB_RESTORED                = YES
ORIGINAL_DB_SHA_MATCH               = YES
OWNER_STORE_OPENED_BY_U1P_R1        = NO
UNRELATED_SIBLING_STORES_UNTOUCHED  = YES   (17 entries before and after, none changed)
```

Owner main `d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468` / 155,648 B / mtime
2026-09-30T11:56:40, WAL `e3b0c442…` / 0 B, SHM `fd4c9fda…` / 32,768 B — identical to the pre-park baseline.
The disposable store was 4,096 B main + 679,832 B WAL with a different main digest
(`da63a52bdf62b3fc…`), which is the positive proof that the owner's file was never the one the app opened.
Adjudicated by recomputation in `02_owner_backup/OWNER_STORE_STATUS.json`.

## 10. Scenario manifest (real installed captures, nothing simulated)

Every row below is a line in `00_authority/CAPTURES.jsonl`, carrying the requested client size, the client
rectangle measured before and after the grab, the window title, and the artifact/NSIS/EXE digests of the bytes
being certified. All fifteen measured exactly what was requested.

| Scenario | File | Client | SHA-256 |
|---|---|---|---|
| S1 matched | `R1_S01_Matched_1440x900.png` | 1440×900 | `16e37008e1f39ba3e647a9582f2b770a434ec0ba6fef053aec1eb2b7ee055126` |
| S1 responsive | `R1_R01_Matched_1024x720.png` | 1024×720 | `3319846330f0207f254a3dd21da0bf592576ecf628efe88bbe17a7a80a609bbd` |
| S2 mismatch | `R1_S02_Mismatch_1440x900.png` | 1440×900 | `cb9e803991df703c5133202ea1712ef03082b3c2e86ebec654dc44a2d5cfe0fb` |
| S2 responsive | `R1_R02_Mismatch_1024x720.png` | 1024×720 | `702877ee338ad0d44f525eeff3e85d5918dd400a429adc3d1253a20ab6e1c83b` |
| S2 third width | `R1_R03_Mismatch_1056x799.png` | 1056×799 | `a1f20be19dc79c81dad03d86906a6fb1a08eba54e6c54d1783b9be4bba66c542` |
| S3 unanalyzed selection | `R1_S03_UnanalyzedSelection_1440x900.png` | 1440×900 | `7b10ad80296fcad2c6f07146ee0249313cbaeff917c1df2447025c8c188c257a` |
| S4 matched after a new run | `R1_S04_MatchedAfterNewGate_1440x900.png` | 1440×900 | `ec056f0ad9c38025ba4703dbcc7c701eae43a8d721b8fe45d4431b3e4279747a` |
| S5 policy mismatch | `R1_S05_PolicyChanged_1440x900.png` | 1440×900 | `c60006ba9e1055914c95e2903b60f56e8e29cf285945e74a34e327e352a521b7` |
| regression · Analyze pending | `REG_Analyze_pending_1440x900.png` | 1440×900 | `47e833f83f451f0039754aaabdba8eb8e2cdb92f47cf0a687a4f5ed459c4ae8e` |
| regression · Compare | `REG_Compare_1440x900.png` | 1440×900 | `2cd7c9afdd5bd91c94b04523ba5882f47f443c892341441084f3996fa01c9972` |
| regression · Release Gate | `REG_Release_runB_1440x900.png` | 1440×900 | `e16bd68d2747e82b4c6135e2f7029929b9d7a68e2756cb01bd6362fdedbf51cd` |
| regression · Bundle & History builds | `REG_History_1440x900.png` | 1440×900 | `29c512fa928f4c6a463c107ada176c0ade64e810c3347ed6d00dfae18ac22f16` |
| regression · Gate runs tab | `REG_History_GateRuns_1440x900.png` | 1440×900 | `35e204f5b288df938389269ee062759e5b66e60932419767a33b6364459c9e69` |
| regression · parse failure | `REG_ParseFailure_1440x900.png` | 1440×900 | `32610fcb601c600ebd4a9565067effd52761883fd31d25c9c372386d1243481d` |
| regression · keyboard focus | `REG_KeyboardFocus_1440x900.png` | 1440×900 | `f0089dc1ea318299507954364403bd0c53534902d94c2edd3bc256dab3a5786d` |

**Bookkeeping, stated rather than tidied:** `REG_History_GateRuns_1440x900.png` appears twice in the log. The
first line (38,291 B, `806499930166656f…`) captured the *Release records* tab after a mis-measured tab click; the
name was then overwritten by the correct Gate-runs capture (52,843 B, `35e204f5b288df93…`), which is the row
above. The earlier image no longer exists on disk and the log line was left as written — the log is an
append-only record of what was grabbed, and editing it would make it worth less as evidence.

**S5 was EXECUTED, not recorded as NOT_EXECUTED.** Two real project policies exist in the disposable fixtures,
differing only in `flash_budget 512 → 640`, so the loaded fingerprint genuinely differs from the one the stored
run was judged under; both fingerprints are printed in the capture.

Real identities used by the drive:

- snapshot A `snap-3f615b6247179d930f59b76dcdbaea1a5eca8c835ab6d410feb20a062f33f21f-p0-normalize-1-832690060a8d25f8acacd85a62ce76bc83dfb6768e435bcc04c51f9664ed363e`
- run A `gate-ff7fae0be1318f32e62eff3a7e216c080d90534a50ec596cc9a1916a1cbea96`, stored 2026-10-08T09:39:05Z
- snapshot B `snap-4b4087e1407ceddcb1e1cfd978eedbc8906cf88083ba3b4672c7c225ddd8374-p0-normalize-1-a38575cd51ec2730442a0f3b8506c99fc49eec68d4b28ecdc9dda9434709ace6`
- run B `gate-38a07cc0830d93a53412dcd6b9ccfae71b728d3e59c74368033d8b48bbad6bfb`, stored 2026-10-08T09:45:21Z
- loaded policy `bcc2f9da1e5a1004428cd98eedffb4a182c2bbf363104db727f8ea3d9c5e83ed`; S5's other policy
  `b87aa057854d8a0089594ad9445b0626174dc0308d636dd6d6854902ae1e5e98`

## 11. §14 acceptance gate, box by box

All thirteen hold. The machine-readable copy with the same reasoning is
`04_scenarios/ACCEPTANCE_GATE.json`.

1. **Exact CI installer verified and installed** — §8 above; `INSTALL.json` re-hashed the bytes it launched.
2. **Matched subject named, verdict correct** — S1: the header snapshot and the "Judged snapshot" line carry the
   identical full id; the BLOCK sentence and counts 1/0/1/6/2 belong to that run.
3. **Mismatched A and B explicitly distinguished** — S2 prints "Judged snapshot snap-3f615b…64ed363e" beside
   "Snapshot this page describes snap-4b4087…4709ace6", both untruncated.
4. **Mismatched build shows no unqualified current answer** — S2's dominant slot is the hollow UNKNOWN chip and
   a sentence that says the page has no shipping answer. Neither the BLOCK word nor A's counts are restated.
5. **Unanalyzed selection does not inherit** — S3: "Not assessed for the selected artifact", with the retained
   B report still labelled as retained.
6. **New run on B restores the normal verdict** — S4: BLOCK sentence back, run `gate-38a…bbad6bfb`, judged
   snapshot equal to B.
7. **Historical run A unchanged** — the Gate-runs tab lists both rows, A still `gate-ff7…a1cbea96` stored
   09:39:05Z, under the page's own "A stored run is immutable" sentence.
8. **No regression in counts, accepted-review semantics or policy scope** — counts identical on both pages for
   both runs; "Disposition: BLOCK … As computed: BLOCK" intact; the scope sentence present verbatim in every
   readiness capture including the four neutral states.
9. **1440×900 decision-first, 1024×720 readable** — five 1440×900 scenario captures plus 1024×720 and 1056×799;
   every row's measured client equals its request.
10. **Focus / keyboard / screen-reader identifiers usable** — ten real TAB presses land a visible focus ring on
    "Attach one on Analyze"; the region and group names the tests assert are the strings the installed page
    renders.
11. **Tests and installed smoke pass** — §5, §6, §7 (266 UI tests on both UI jobs in CI) and this drive itself.
12. **Owner files restored byte-exact** — §9.
13. **No new S0/S1/S2 or U1P-V2/V3/V4 introduced** — see the observation below; nothing in the five scenarios or
    the regression glance deviated from the shipped contract.

### One observation, reported rather than buried

The Analyze capability pill reads **"Git unknown"** while the Gate's `git.clean` rule on the same snapshot
returns a definite BLOCK with `git:status` / `git:head` evidence. That is **pre-existing Core behaviour, not
something this unit introduced**: `Capabilities::with_git()` has no production caller — the analysis path builds
capabilities only through `Capabilities::default()` (`crates/firmwaresight-core/src/domain/build_snapshot.rs:67`),
whose `git` field is `Availability::Unknown`
(`crates/firmwaresight-core/src/domain/capability.rs:83`). This unit's diff touches no Rust and no Analyze
markup. It is recorded here for the Architect's judgement and deliberately **not** filed as a new U1P-V2/V3/V4
and **not** fixed, because §6 puts Core out of this unit's boundary.

## 12. Residual items, unchanged

`U1P-V2-01` → **CLOSED_BY_U1P_R1**. Still open and untouched by this unit, exactly as U1P recorded them:
`U1P-V1-01` (a wrapped band leaves an empty filled cell at 1024), `U1P-V1-02` (the Compare result does not
survive navigation), note `U1P-V0-01`, and the eight `U1-V1` / `U1-V0` items carried from the U1 round. No
`U1P-V3`, no `U1P-V4`. U1R's truthfulness still holds — the Analyze pending and parse-failure captures in §10
show the neutral chips, not borrowed pills.

## 13. User-uploadable ZIP identity

`FirmwareSight_U1P_R1_Overview_Subject_Review.zip`, placed in `C:\Users\16429\Downloads\`. Its exact path, byte
count and SHA-256 are printed in the final response and recorded in `.ai/HANDOFF.md`. The copy of **this** report
inside the pack necessarily predates the digest of the archive that contains it, which is why the digest is
quoted outside the pack rather than claimed inside it.

## 14. V1 interlock

V1 remains `IN_PROGRESS / RECRUITMENT_READY` with **0 eligible participants** and participant execution paused.
No V1 session, recruitment or metrics artefact was edited; the F3 frozen research cohort was not re-baselined;
B1 / Private Beta, RC / GA, tags, GitHub Releases, a public installer, signing, notarization, updater, and
licence/pricing were all out of scope and untouched.

## 15. Recommendation

All thirteen §14 boxes hold on this head's own CI-built bytes, and no `U1P-V2`, `U1P-V3` or `U1P-V4` remains
open. Under §16's rule the status moves to:

**U1 = `READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT`**

That is **not** `PASS_COMPLETE`, not `DESIGN_COMPLETE`, not `MOCKUP_MATCHED`, and not a visual acceptance: the
eight `U1P-V1` / `U1P-V0` items and the §11 observation above are the Architect's to weigh, and no product source
changed after the captures (`git status` clean at `42f75a7` when the last screenshot was taken and again after
the owner store was restored).
