---
title: "P3 Release Gate Desktop Smoke Report"
doc_id: "FS-P3-SMOKE-DESKTOP"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P3_RELEASE_GATE"
owner: "Engineering"
last_updated: "2026-09-30"
---

# P3 Release Gate — shipped Windows desktop smoke (prompt §61)

Every line below is something this report's author saw in a capture of the running window, or
computed from a file the running app wrote. Nothing is inferred from the test suite; where a step
could not be reached through the window, it says so.

## Run under test

| | |
| --- | --- |
| Binary | `target/release/firmwaresight-desktop.exe`, sha256 `d6709847473954f7c9f3b1db6f4e52524b18f69daa5f3c7d3d1130f24fa2ce6a` |
| Build | `cargo build --release -p firmwaresight-desktop --features custom-protocol` over `ui/dist` from `corepack pnpm build` |
| Window | 1048 × 768 client at (112,135), default size, no scaling applied |
| Database | clean validation DB (the pre-existing DB was moved aside and restored afterwards — see "Database handling") |
| Gate subject | `%TEMP%\p3-smoke\brake-node`, `brake-node-nogit` — throwaway Git repositories. This repository is never a Gate subject (§26, §61) |
| Runs | Run 1 and Run 2 reached step 10 and found defects F and H; **Run 3 walked all 40 steps** and is the run reported here |

`cargo build --release` **without** `--features custom-protocol` produces a binary that navigates to
the dev URL and shows a WebView2 `无法访问此页面 / localhost 拒绝连接` page. That happened on the first
attempt at step 1 of Run 3, and the capture was replaced when the correct build was made. It is
recorded because a smoke of a dev build would prove nothing about shipping.

## Inputs

Hashes are from `sha256sum` on the staged files, not from the screen.

| Side | File | Bytes | SHA-256 |
| --- | --- | --- | --- |
| base | `build/base/firmware.elf` | 16,820 | `3f615b6247179d930f59b76dcdbaea1a5eca8c835ab6d410feb20a062f33f21f` |
| base | `build/base/firmware.map` | 4,511 | `832690060a8d25f8acacd85a62ce76bc83dfb6768e435bcc04c51f9664ed363e` |
| target | `build/target/firmware.elf` | 17,464 | `4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374` |
| target | `build/target/firmware.map` | 4,664 | `a38575cd51ec2730442a0f3b8506c99fc49eec68d4b28ecdc9dda9434709ace6` |

Workspace HEAD `a0b54568b87a24076b752b89827236ee8f54f4d5`, tagged `v1.2.3`; policy
`dc68f391aa9a34ef086d157a7c308fea48db6274b71e61cdc7ae34da087ca1aa` from a 426-byte
`firmwaresight.toml` (flash 512, RAM 256, growth review 100, required `["elf","map"]`,
`source = "git_tag"`, `pattern = '^v(?P<version>\d+\.\d+\.\d+)$'`, expected version `1.2.3`,
notes at `docs/RELEASE_NOTES.md`).

## How the window was driven

The Qoder browser/computer-use connector was unavailable for this smoke, so input and capture came
from `target/p3win.ps1`: `PrintWindow(PW_RENDERFULLCONTENT)` for the WebView window,
`CopyFromScreen` for native dialogs, and `SendInput` with `AttachThreadInput` for keyboard events.
Click targets were computed from PNG pixels by `target/p3find.py` (colour boxes and measured crops),
never guessed from prose. Every click was followed by a capture to confirm it landed, and several did not
land the first time: `Add MAP`, `Accept review`, `Record acceptance`, `Open project config` and the `KiB`
radio each needed a corrected coordinate (the radio took four attempts before its label was hit), and one
mis-click on `Choose firmware artifact` had actually registered late, so a second dialog opened on top of
the first and both had to be dismissed with `Escape` before step 3 could be redone. The visible result, not
the intent, is what is reported.

## The 40 steps

| # | Step | Result | What was actually seen |
| --- | --- | --- | --- |
| 1 | launch | PASS | Window `FirmwareSight - Analyze`; rail lists exactly Analyze / Compare / Release; `Nothing has been analyzed in this session yet. Choose an artifact and run Analyze.` on a clean DB |
| 2 | Analyze base + MAP | PASS | Native dialog `Choose the firmware artifact to analyze`; then `MAP: Not provided` → `MAP: firmware.map`; report shows `firmware.elf`, SHA `3f615b62…`, `16,820 bytes`, `Arm 32-bit little` |
| 3 | Analyze target + MAP | PASS | SHA `4b4087e1…`, 17,464 bytes; `nonvolatile 376 bytes (exact) · runtime 76 bytes (exact)` |
| 4 | open Release | PASS | Heading `Release Gate`; `1 · Project policy` with `No project policy is loaded. A run judges FirmwareSight's stated default policy, and the run below shows which values those were.` |
| 5 | load config, native dialog | PASS | OS dialog `Choose the project's firmwaresight.toml`; path typed into `文件名(N)`, accepted with Return |
| 6 | project / policy visible | PASS | `brake-node  firmwaresight.toml`, `schema_version 1 · policy dc68f391aa9a34ef086d157a7c308fea48db6274b71e61cdc7ae34da087ca1aa`, `Loaded firmwaresight.toml` |
| 7 | no absolute path | PASS | The config line names the project and the file only; no `C:\`, no `%TEMP%`, no `p3-smoke` anywhere in the block (full sweep at step 39) |
| 8 | current build = target | PASS | `firmware.elf · snap-4b4…4709ace6`, SHA `4b4087e1…5ddd8374`, 17,464 bytes, stored `2026-09-30T18:31:11Z` |
| 9 | baseline = base | PASS | `firmware.elf · snap-3f6…64ed363e`, 16,820 bytes, `nonvolatile 256 bytes (exact) · runtime 8 bytes (exact)`, stored `18:27:47Z`; with no baseline the same control reads `No build is chosen.` and says the growth rule answers Unknown |
| 10 | Run Gate | PASS | `3 · FirmwareSight policy readiness` — `Disposition: REVIEW`, `As computed: REVIEW`, `Block 0 Review 1 Unknown 0 Pass 8 Not applicable 1`, run `gate-a64631b2…0ff152cc`, stored `18:33:50Z`, Workspace `HEAD a0b54568…85f4f4d5`, `exact tag v1.2.3 on workspace HEAD`, `clean (workspace, not artifact)` |
| 11 | Git clean PASS | PASS | `git.clean` PASS / effective PASS — `Workspace has no uncommitted changes at Gate evaluation time.`, refs `git:status`, `git:head` |
| 12 | commit rule | PASS | `release.commit_matches_expected` **N/A** / effective PASS — `No `release.expected_commit` is declared for this release.`, ref `policy:release.expected_commit`; grouped under `Not applicable 1 — The policy or this build puts this rule outside the run.` |
| 13 | version / tag rule | PASS | `release.version_matches_policy` PASS — `Workspace tag `v1.2.3` matches the version pattern and its version `1.2.3` is the declared release version.`, refs `policy:version.source`, `policy:version.pattern`, `git:exact-tag=v1.2.3`, `policy:release.expected_version` — **no locator carries a separator** (defect F's regression, seen on screen) |
| 14 | required artifacts | PASS | `Every required artifact kind is present in the snapshot (elf, map).`; table `elf Required Present 4b4087e1…5ddd8374 17,464 bytes`, `map Required Present a38575cd…4709ace6 4,664 bytes` |
| 15 | hashes | PASS | `All 2 snapshot artifact(s) carry a SHA-256 digest.` with both `artifact:` refs and `policy:artifacts.hashes` |
| 16 | FLASH budget | PASS | `Nonvolatile footprint 376 B is within the 512 B budget (136 B headroom).`, refs `policy:memory.flash_budget=512` + `evidence:ev-memory-nonvolatile_image_footprint_bytes`; table row `FLASH 376 bytes / 512 bytes / 136 bytes headroom / PASS effective PASS / MapRegionAndElfLoad · complete attribution` |
| 17 | RAM budget | PASS | `Runtime RAM footprint 76 B is within the 256 B budget (180 B headroom).`, ref `evidence:ev-memory-runtime_ram_footprint_bytes`; row `RAM 76 / 256 / 180 bytes headroom` |
| 18 | growth | PASS | `Growth against the baseline — The deltas are the same ones Compare shows: Core computed them once and this page moves them.` Row `FLASH old 256 bytes, new 376 bytes, delta +120 bytes, review threshold 100 bytes, REVIEW effective REVIEW · delta exact` |
| 19 | notes | PASS | `Release Notes `docs/RELEASE_NOTES.md` is present.`; notes block `Path docs/RELEASE_NOTES.md`, `Required · Present effective severity: PASS`, SHA-256 `2084502…2b7`, and `The path is relative to the project folder, and the folder itself is never shown. There is no editor here…` |
| 20 | unknown evidence | PASS | `0 snapshot evidence item(s) are Unknown, below the review threshold of 1.`, ref `policy:gate.unknown_evidence_review_count=1` |
| 21 | rule / state / evidence / remediation | PASS | All ten rows carry rule id, state icon + label, `effective severity:`, a factual summary, a `Next step …` remediation and their locator list |
| 22 | deterministic REVIEW case | PASS | `diff.growth` REVIEW with `+120 B over the 100 B threshold`; pressing Run Gate again with identical inputs left SQLite at **1** run / 10 findings and `created_at 18:33:50Z` unchanged — §33 dedupe, observed through the store the app writes |
| 23 | Accept Review, actor + reason | PASS | Form `Who accepts this review` / `Why it is acceptable` / `Record acceptance` disabled until both hold text, with `Both are required. An acceptance without a name and a reason is not an audit record.` |
| 24 | finding remains REVIEW | PASS | The row still reads `diff.growth REVIEW effective severity: REVIEW`; the stored row is still `('REVIEW','REVIEW')` after acceptance |
| 25 | audit actor / time / reason | PASS | `Accepted — By lin.we — When 2026-09-30T18:41:31Z — Reason "Growth is intended: the pad-wear table added 120 B of nonvolatile. Reviewed in Compare against the v1.2.2 baseline." — Accepted state REVIEW`, plus `An acceptance is a record: it cannot be edited or deleted, and this row stays REVIEW.` |
| 26 | aggregate moves only as allowed | PASS | `Disposition: PASS` over `As computed: REVIEW`, counts still `Review 1`, run id unchanged — the finding did not move, the aggregate did |
| 27 | dirty the temp repo | PASS | `?? untracked-caliper.txt` created in the subject folder; the app was not restarted or reloaded |
| 28 | rerun | PASS | New stored run at `18:43:23Z` |
| 29 | `git.clean` BLOCK | PASS | `Block 1 — The rule evaluated and failed in a way that stops this run.`; `git.clean BLOCK effective severity: BLOCK` — `Workspace has uncommitted changes at Gate evaluation time.` / `Next step Commit or discard the pending changes, then rerun the Gate against the resulting HEAD.` The untracked file's name and path appear nowhere |
| 30 | new run id / prior identifiable | PARTIAL | New id `gate-02d4404b…0ff75f` on screen, distinct from `gate-a64631b2…`. The prior run and its acceptance were read back from SQLite (4 rows, distinct ids, run #1 keeps its acceptance). **Not observed through the window**: the Release page has no run-id input, so an older record cannot be re-opened from this build — see "Not verified" |
| 31 | remove the notes file | PASS | `docs/RELEASE_NOTES.md` moved to `docs/RELEASE_NOTES.md.set-aside` (reversible, not deleted) |
| 32 | rerun | PASS | Third stored run at `18:45:26Z` |
| 33 | notes BLOCK | PASS | `Block 2`: `release.notes BLOCK` — `Required Release Notes `docs/RELEASE_NOTES.md` does not exist in the project.` / `Next step Write the release notes at that project-relative path, or point `release.release_notes_path` at where they actually live.`, refs `policy:release.require_release_notes`, `file:docs/RELEASE_NOTES.md`. `git.clean` blocks in the same run because removing a tracked file is itself an uncommitted change — two findings, not one mislabelled |
| 34 | Git unavailable / not-a-repo context | PASS | `brake-node-nogit` config loaded (a byte-identical copy of the policy, so the policy block correctly renders unchanged — the project root is never shown, which is what makes this step look like a no-op); run stored at `18:50:34Z` |
| 35 | Git finding UNKNOWN | PASS | `Unknown 2 — The evidence this rule is missing, so it could not be evaluated.`; `git.clean UNKNOWN` — `Workspace dirty state is unavailable: this directory is not a git repository, so workspace provenance is unavailable.`; `release.version_matches_policy UNKNOWN` — `Git workspace facts are unavailable…` |
| 36 | severity follows policy | PASS | Both UNKNOWN rows carry `effective severity: REVIEW`; the aggregate reads `Disposition: REVIEW`, never BLOCK and never clean |
| 37 | UNKNOWN cannot be accepted | PASS | Neither UNKNOWN row offers an `Accept review` control — only the REVIEW row does. The portable document carries no acceptance field for UNKNOWN, and storage answers a non-review acceptance with `ERR-STORAGE-4008` in `release_gate.rs::only_a_review_can_be_accepted` |
| 38 | Bytes / KiB budget display | PASS | Selecting `KiB` relabels every figure: artifacts `17.1 KiB` / `4.55 KiB`; budgets `FLASH 0.367 / 0.500 / 0.133 KiB headroom`, `RAM 0.0742 / 0.250 / 0.176 KiB headroom`; growth `0.250 → 0.367 KiB, +0.117 KiB, threshold 0.0977 KiB`. Same bytes, /1024 only; the notes SHA-256 is untouched |
| 39 | no path leak | PASS | 147 persisted text fields across the 4 runs (summaries, remediations, 89 evidence locators, acceptance actor/reason, `projects`, `builds`) plus the two CLI documents: **0** hits for the temp root, the smoke marker, any backslash or any drive-letter path; **0** locators outside the six Core schemes. `projects` has no root column at all (`id, name, created_at`) |
| 40 | clean close | PASS | `taskkill` (WM_CLOSE, not forced) → window and process gone; reopening the file the app left reports `integrity_check ok` with 4 runs / 40 findings / 89 locators / 1 acceptance / 2 builds still readable, and the WAL checkpoints cleanly |

## Cross-surface parity (extra, not a §61 step)

The same project, same stored builds, same HEAD and same policy, judged through the CLI:

```
fwsight gate --project . --artifact build/target/firmware.elf --map build/target/firmware.map \
             --baseline build/base/firmware.elf --baseline-map build/base/firmware.map --json
exit=4  run_id gate-a64631b287fcb33478e23ac6462dacc80e1a39cac1e321e047d02bb50ff152cc
        policy_sha256 dc68f391aa9a34ef086d157a7c308fea48db6274b71e61cdc7ae34da087ca1aa
```

Identical to the run id the window displayed at step 10, and identical to the row SQLite holds. This
is the re-verification of defect H: a Gate run's identity is a hash of its facts, so one build must
present one set of facts whether those facts came off disk or out of the database.

## Defects this smoke found

**F — a project that used the only MVP version source could not persist a Gate run.** The first
attempt at step 10 ended in `ERR-STORAGE-4006 … CHECK constraint failed …
instr(evidence_ref, char(92)) = 0`: Core had put the `[version] pattern` text — `\d` and all —
inside an evidence locator, and the storage CHECK forbids a separator in a locator. Any real project
would have hit this. Fixed by making the locator a bare field reference (`policy:version.pattern`),
keeping the pattern in the summary where it is quoted and inside the fingerprint where it still
identifies the run; `release_gate.rs::a_version_pattern_that_carries_a_separator_still_persists_the_run`
and the Core test pin it. Step 13 above is the shipped-binary confirmation.

**H — the CLI and the desktop gave one judgement two run ids.** Parity testing isolated it to the
footprint evidence pointer: a desktop run hydrated its totals from SQLite, which stored the numbers
but not the `evidence:` row they came from, and Core's canonical form also carried a `reason` string
the database never recorded. Both were inside the fingerprint. Fixed by reading the two ids out of
`evidence` beside the totals (`FOOTPRINT_EVIDENCE_FIELDS` + `Database::evidence_id_for_field`) and
dropping `reason` from the canonical memory block, with
`a_stored_build_fingerprints_exactly_like_a_fresh_analysis` holding the two assembly paths against
each other.

**E — a disabled primary button kept its accent border** (pixel-sampled `#F1F3F5` fill, `#5D6673`
label, blue border). Fixed in `Release.module.css` and `Compare.module.css`; the `Record acceptance`
button at step 23 is the re-verification.

## Open observations, not fixed

- The `Memory budgets` table is wider than the page once `Evidence basis` is shown, so the report
  area gets a horizontal scrollbar (visible at steps 16–17 and 38). Nothing is clipped without a
  scroll, and no rule text is lost; it is a density defect, not a correctness one.
- The Release page cannot open an older run by id (step 30). `get_gate_run` accepts any run id and is
  covered by `a_stored_run_survives_a_restart_because_the_record_is_in_sqlite`; the *screen* offers
  only the run this session computed. Surfacing history is P4's History page (§64 forbids it here).
- One frontend test failed in one of twelve `pnpm test` runs earlier today and its name was not
  captured; the closure round's gate then failed on it too, which is what made it diagnosable. It was
  defect **I** — three `compare.test.tsx` row queries racing the `Loading symbol changes…` state that
  `Compare.tsx:1303` renders inside the same region. Fixed by awaiting those queries; see
  `P3_GATE_EXECUTION_REPORT.md` §7. Not a product defect and not a P3 surface, but it was P3's gate
  that caught it, so it is recorded here rather than credited elsewhere.

## Database handling

`%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite` held the user's P2-era data. It was
moved aside to `…\firmwaresight-p0.sqlite.pre-p3-smoke` (with its `-wal` and `-shm`) before the
smoke and restored afterwards, main + WAL + sidecar, verified by `PRAGMA integrity_check` → `ok`,
2 builds, 1 project, schema version 2 — the pre-P3 state. The smoke's own validation DB is archived
at `%TEMP%\p3-smoke\db-run3\` (and Run 2's at `db-run2\`). Nothing was deleted.
