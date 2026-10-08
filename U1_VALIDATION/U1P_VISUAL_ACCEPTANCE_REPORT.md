---
title: "U1P Installed Visual Acceptance Report"
doc_id: "FS-U1P-VISUAL-ACCEPTANCE"
product: "FirmwareSight"
version: "1.0"
status: "EVIDENCE"
round: "U1P"
canonical_unit: "U1P_INSTALLED_ACCEPTANCE_AFTER_CI_RERUN"
date: "2026-10-08"
evidence_root: "%TEMP%\\FirmwareSight-U1P-Visual-Acceptance-20261008T001110Z"
---

# U1P installed visual acceptance

This report records what was captured on the installed U1P bytes and what it showed. It issues no product stage.
The verdict on the visual track belongs to the human reviewer, so the words this file is allowed to use about
U1 itself are limited to the recommendation in §9.

## 1. CI authority - run 37668291893, attempts 1 and 2

Read back from GitHub by run id and full head SHA, not by "latest".

| Field | Value |
|---|---|
| Repository | https://github.com/2023violet/FirmwareSight |
| Head SHA | `6a071c51c81a00b7a35facaefa5849e9975e715c` |
| Commit | `U1P: converge core pages on decision-first desktop hierarchy` |
| Run id / number | 37668291893 / #79 |
| Final status | COMPLETED / SUCCESS, attempt 2, effective 10 of 10 authoritative jobs successful |

**Attempt 1** finished with eight jobs successful and two cancelled at the six-hour runner ceiling:
`Generated output drift` and `Package Ubuntu`. Both were executing the Linux prerequisite `apt-get update`;
neither job had started any repository-specific step. That is the class the repository's own retry policy
allows one same-SHA rerun for (`P5_VALIDATION/P5_CI_AUTHORITY.md`: a pure runner-provisioning failure before any
repository step could run, answered by a single rerun with the log read first and both attempts retained).
Corroboration that the SHA was not the problem: `Rust (ubuntu-latest)`, `Desktop UI (ubuntu-latest)` and
`Package macOS` all passed the same commit in attempt 1.

**One** `gh run rerun --failed` was issued. There was no attempt 3.

**Attempt 2** shows all ten jobs successful. Only the two cancelled jobs re-executed; the other eight carry
attempt 1's execution window to the second and are carried forward, not newly run:

| Job | Attempt 2 window | Provenance |
|---|---|---|
| Generated output drift | 04:12:25Z → 04:15:07Z | re-executed |
| Package Ubuntu | 04:13:01Z → 04:24:51Z | re-executed |
| the other eight | identical to attempt 1 | carried forward |

The two re-executed jobs were read individually. `Generated output drift` ran `=== [drift] baseline integrity`,
reported `RESULT PASS` and 8/8 steps passed. `Package Ubuntu` finished a bundle and reported 4/4 steps passed.
`Package Windows` (attempt 1) built, verified and uploaded.

Counts read from the remote logs, not from the implementation report: Rust 868 passed / 0 failed across 47
result lines on both runners; `Test Files 9 passed (9)` and `Tests 253 passed (253)` on both Desktop UI jobs.

## 2. The installed artifact

| Item | Value |
|---|---|
| Artifact id | 11504871803 |
| Artifact name | `FirmwareSight-0.6.0-windows-x86_64` |
| Produced by | run 37668291893 at head `6a071c5…` (uploaded in attempt 1; still the exact artifact for this run and head after a failed-jobs-only rerun) |
| Artifact ZIP | 5,538,316 bytes, sha256 `57651a3ee749d34fce6a7859ec979f4b845fbd15c31291140d779a3e8880c1bb` (matches the artifact's own API digest) |
| `toolchain.git_commit` | `6a071c51c81a00b7a35facaefa5849e9975e715c` |
| NSIS installer | `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, 3,890,447 bytes, sha256 `f69cda6c5f091707d79084547358e9f71b882c792a28abfc478cf8d54d4f7adb` |
| CLI companion | `FirmwareSight-0.6.0-windows-x86_64-cli-fwsight.zip`, 1,668,152 bytes, sha256 `f0fbee24…` |
| Internal `SHA256SUMS.txt` | both entries verified against the extracted payloads |
| Installed EXE | `%LOCALAPPDATA%\FirmwareSight\firmwaresight-desktop.exe`, 15,366,656 bytes, sha256 `7327d374f283e81fde5575868f3090c6ab47d1c2189f5c3b0ce9ee0a9aefbeb0` |

The installed EXE digest differs from the artifact's recorded `payload_sha256` for the reason the artifact itself
states: the bundler rewrites a 27-byte bundle-type token in the copy it packs.

Nothing was substituted. No U1, U1R or F3 artifact, no local `--only` package, no `cargo run`, no Vite server,
and no later documentation-only CI artifact appears anywhere in the capture set.

## 3. Owner-store safety

Recorded before installation and re-checked at every step in `02_owner_backup/` and `08_owner_restore/`.

| Flag | Value |
|---|---|
| `OWNER_STORE_PARKED` | YES |
| `OWNER_BACKUP_HASH_MATCH` | YES |
| `ORIGINAL_DB_RESTORED` | YES |
| `ORIGINAL_DB_SHA_MATCH` | YES |
| `OWNER_STORE_OPENED_BY_U1P` | NO |
| `UNRELATED_SIBLING_STORES_UNTOUCHED` | YES |

The owner's three files were identified, hashed, independently backed up, verified, and moved out of the live
path before the installer ran; the live path was then empty of those names. The application created its own
disposable store during the round, which was hashed and moved out by name afterwards. After the restore, all
three files match the pre-round baseline digests exactly
(`d6e41034…`, `e3b0c442…`, `fd4c9fda…`), and the fourteen unrelated entries in the application-data folder -
including the two directory entries and the historical `*.p0-smoke-history`, `*.p2-smoke`, `*.g2-smoke` and
`postg2-e2e-*` siblings - compare identical on kind, size and digest.

The machine did not have the product installed before the round, so the round uninstalled it at the end. The
uninstaller was driven with real mouse input through the ownership-checked path, and its
"Delete the application data" option was never pressed. What is claimed is that no input was sent to it; no
assertion is made about its ticked state, because the read-only control probe returns no checkbox class on that
page and a state claim would be vacuous.

## 4. Per-page first-viewport adjudication

Each row is read off the named file in `04_1440/`. Client sizes are measured, not assumed: every capture line
in `00_authority/CAPTURES.jsonl` records the requested client size and the value re-read from the live window
before and after the grab.

### Overview - `P01_Overview_1440x900.png` - PASS, page conclusion REQUIRES_POLISH

*Target.* Compact ELF/MAP/Git capability band, the ship verdict, a key-metric summary and the next-step action
above the fold, with the verdict holding more visual weight than the setup facts.

*Actual.* The capability band occupies y=130..260 of 900 - three cells, each with a state chip, one mono line of
facts and one link. "Can we ship now?" spans y=285..530 with the PASS chip, the sentence "Clear - 8 rule(s)
pass, nothing is blocked and nothing waits on a review.", the five-state count strip, the evidence run id and
timestamp, and the "Open the run on Release Gate" action inside the panel. The metric band (flash 376 bytes,
runtime RAM 76 bytes, symbols 52, evidence 11) sits below at y=465..565. The verdict is visibly the dominant
element and the capability band does not consume the viewport.

*What changed from U1.* The verdict moved above the metrics and the capability facts were compressed into a
single band; the ship answer no longer sits under a stack of setup detail.

*What remains.* `U1P-V2-01` (§7): the verdict does not name the build it judges, while the page's subject line
names the last-analyzed snapshot.

*Responsive.* `R01_Overview_1024x720.png` - the readiness verdict, its counts and its next-step action are all
still in the first viewport at 1024x720. Navigation and the three header actions remain usable.

*Functional.* Rail navigation and the "Open the run on Release Gate" action were exercised; the press moved to
the Release Gate page.

*Reference-only difference.* FS-UI-01 shows a capability band with per-cell icons and a compact metric strip;
the implemented cells carry real facts and links rather than the mock's placeholder values. No mock element was
invented to match.

### Analyze - `P02_Analyze_1440x900.png` - PASS, page conclusion ACCEPT

*Target.* A compact current-result summary with genuine analytical value immediately - Sections or Symbols in
the first viewport - and no long metadata dossier in front of the analysis output; U1R's state truthfulness
intact.

*Actual.* Summary band first (flash 376 bytes Exact, runtime RAM 76 bytes Exact, load evidence Admissible for a
hard limit, evidence recorded 11), then the Sections tab with real rows above the fold: `.text` 212 bytes,
`.rodata` 32, `.settings` 64, `.data` 4, `.bss` at the fold edge. The general metadata dossier is below the
analysis output.

*What changed from U1.* The dossier was demoted below the result and the summary became a single band; the
capability chips are the U1R-correct three.

*What remains.* Nothing material on this page. `U1P-V0-01` notes that at 1024x720 the table header sits at the
fold.

*Responsive.* `R02_Analyze_1024x720.png` and `R07_Analyze_1056x799.png`.

*Functional.* Valid analysis, invalid analysis, recovery, the Sections/Symbols/Evidence tabs and the Bytes→KiB
switch were all exercised on these bytes.

*Reference-only difference.* FS-UI-02's dossier-first arrangement is exactly what U1P moved away from; the
reference's ordering is not reproduced, by design and per the acceptance contract.

### Compare - `P03_Compare_1440x900.png` - PASS, page conclusion ACCEPT_WITH_MINOR_POLISH

*Target.* BASE→TARGET identity, the delta summary, actual diff content and growth contributors in the first
viewport, with selection reading as a compact comparison bar rather than a two-column form.

*Actual.* Base `firmware.elf 3f615b624717` → target `firmware.elf 4b4087e1407c` with a hairline arrow between
two compact fields, then "What moved" (sections Added 1 / Removed 1 / Changed 16 / 2 unchanged; symbols Added 38
/ Removed 33 / Changed 5 / 9 unchanged / 67 unpaired) and the Memory rows (nonvolatile +120 bytes, runtime RAM
+68 bytes) before the fold.

*What changed from U1.* The pair became a bar with a real arrow and the result was lifted above the setup and
evidence blocks.

*What remains.* `U1P-V1-02`: the result does not survive navigating away and back.

*Responsive.* `R03_Compare_1024x720.png`, `R08_Compare_1056x799.png`.

*Functional.* Pair selection, Swap, and the diff were exercised; the unit control is shared with the other
pages and was verified there.

*Reference-only difference.* FS-UI-03 shows a growth chart; the implemented page reports the same facts as
figures and rows, and no chart was invented to match the mock. There is no Export control on this page and none
was added.

### Release Gate - `P04_ReleaseGate_1440x900.png` - PASS, page conclusion ACCEPT

*Target.* Dominant verdict plus meaningful rule content, with the states actually present, and configuration
secondary once a run exists.

*Actual.* The verdict band first - PASS chip, the clear sentence, counts 0 block / 0 review / 0 unknown / 8 pass
/ 2 n-a - then "1 · Findings by state" with real rule rows (`git.clean`, `release.version_matches_policy`,
`artifacts.required`) each with its effective severity and evidence ids. "5 · Project policy" and
"6 · Build and baseline" are below. The run was over the target build with the base build as baseline, so
`diff.growth` reports a real PASS against configured thresholds rather than an Unknown.

*What changed from U1.* The headings were renumbered so the numbers read in DOM order, which removes the
1,2,5,6,3,4 contradiction the earlier page carried.

*What remains.* Nothing material.

*Responsive.* `R04_ReleaseGate_1024x720.png`, `R09_ReleaseGate_1056x799.png`.

*Functional.* Policy load, build/baseline selection, run, and the supported bundle flow were exercised.
"Prepare bundle" produced a preview with the release id, version 1.2.3 and both build ids and enabled
"Choose destination folder"; the flow was deliberately stopped there, so no bundle was written to disk.

*Reference-only difference.* FS-UI-04's review-state rows are not visible because this run produced none; the
count strip shows REVIEW 0 rather than fabricating a finding.

### Bundle & History - `P05_BundleHistory_1440x900.png` - PASS, page conclusion ACCEPT

*Target.* The canonical heading, one focused history category as the main workspace, the selected real detail in
the same viewport, and not three database-style tables stacked.

*Actual.* Heading "Bundle & History", one category control (Builds / Gate runs / Release records), the Gate runs
list, and the selected row expanded in place to its stored gate run id, build id, baseline build id and stored
time. Only the active category is mounted, so switching replaces the region rather than adding a second table.

*What changed from U1.* The three stacked persistence regions became one category with a focused list plus
selection.

*What remains.* Nothing material.

*Responsive.* `R05_BundleHistory_1024x720.png`, `R10_BundleHistory_1056x799.png`. At 1024 the wide table scrolls
inside its own container, which is the table's control and not a page-wide overflow.

*Functional.* Category switching, row selection, Details, the Builds filter (`3f6` narrowed 2 rows to 1 and said
so) and the pagination controls were exercised.

*Reference-only difference.* FS-UI-05's release-record timeline has no implemented counterpart because nothing
was published in this session; the Release records category shows its honest empty state.

### Parse Failure - `P06_ParseFailure_1440x900.png` - PASS, page conclusion ACCEPT

*Target.* Current failure, what happened and why we know, diagnostics, recovery action and the retained
previous-result status all discernible, with no regression of U1R's truthfulness.

*Actual.* Two neutral chips - "Current analysis failed" and "Previous result retained below" - then the failure
panel with What happened, Code `ERR-FORMAT-0001`, Why we know (`unsupported format: Binary at not-an-elf.elf`),
What to do, a diagnostics id, and the "Choose another artifact" action. The retained summary band begins below
the panel. No green capability chip claims the failed file is analysable.

*What changed from U1R.* Composition only: the failure moved above the retained result. The state semantics are
U1R's, unchanged.

*What remains.* Nothing.

*Responsive.* `R06_ParseFailure_1024x720.png`.

*Functional.* The failure was produced after a genuine success over the same session and recovered from by
re-selecting the target artifact, which restored the success view with the target's own figures.

*Reference-only difference.* FS-UI-06's exact code text differs because the installed product classifies the
TOML-bytes-under-an-.elf-name fixture as an unsupported format; the code shown is the one the product emitted.

## 5. Shell verdict

`SHELL_PASS`, from `07_smoke/SHELL_analyze_1440x900.json` and `07_smoke/SHELL_release_1024x720.json`. The
method is a measurement rather than a CSS reading: capture, six real single wheel notches into the main column,
capture again, then compare bands. On Analyze at 1440x900 the rail band is pixel-identical while the main band
differs over `[0, 59, 1161, 867]` and the scrollbar strip differs over `[0, 73, 9, 410]`; on Release Gate at
1024x720 the same pattern holds. A third run on Bundle & History reported `NOT_SCROLLABLE_AT_THIS_SIZE` - the
stored rows fit the viewport, so it proves nothing in either direction and is not counted either way.

Two driver artefacts were found while producing this and are recorded because they changed earlier readings: an
aggregated six-notch wheel event is ignored by the webview, so notches are sent one at a time; and a pointer
resting on a nav item draws its hover background into the "before" capture, which a first version mistook for a
travelling rail, so the pointer is parked before both captures.

## 6. Responsive captures

Six at 1024x720 (`R01`..`R06`) and four at 1056x799 (`R07`..`R10`), all measured. No unreadable collapse, no
clipped critical action, no accidental page-wide overflow, navigation usable and the primary action reachable at
every size.

## 7. Findings

| Id | Severity | Statement |
|---|---|---|
| `U1P-V2-01` | material, open | Overview's ship verdict does not name the build it judges. In `07_smoke/FINDING_U1P-V2-01_overview_verdict_subject_1440x900.png` the subject line names snapshot `snap-3f6…64ed363e` and the metric band shows that build's 256 bytes / 47 symbols, while the PASS verdict belongs to run `gate-deb…b202fdd6`, whose subject build is `snap-4b4087e1407c…` - confirmed from that row's own Details in `P05`. Nothing stated is false: the run id is printed and the panel scopes the claim to "the run named above". The gap is that the reader cannot check the pairing from Overview, because the run's subject build is not named there. `App.tsx` keeps `gateRun` and the last analysis as independent state and clears nothing when a different build becomes current. U1P did not create that ownership, but it promoted the verdict to the dominant element, which is what makes the mismatch material. A narrow correction would name the run's subject build in the verdict panel, or suppress the verdict when it does not belong to the named snapshot. Not self-waived. |
| `U1P-V1-01` | minor | A wrapped capability or metric band leaves an empty cell carrying the band's divider background at 1024 width (`R01`, `R02`). |
| `U1P-V1-02` | minor | The Compare result does not survive navigating away and back; the reader presses Compare again. The first `R03` capture was taken in that reset state and was re-captured after re-running the comparison. |
| `U1P-V0-01` | note | At 1024x720 the Analyze table header sits at the fold, so the first data row needs one scroll. |

No `U1P-V3` and no `U1P-V4`. U1R's truthfulness holds in both failure captures.

## 8. FS-UI-07 Unknown Dependency

`REFERENCE_ONLY_NOT_IMPLEMENTED`. The image is in the review pack untouched because it belongs to the
seven-image authority. There is no U1P capture for it and none was made: the product has no Dependency view, no
Declare action and no Re-scan action, and this round invented none.

## 9. Recommendation

Because a material `U1P-V2-01` remains open, §26's gate is not met:

**U1 = REQUIRES_ARCHITECT_POLISH_REVIEW**

If the Architect judges the Overview pairing acceptable as it stands, the remaining items are `U1P-V1` polish
only, and the six first-viewport contracts plus the shell and functional smoke all passed on these bytes. This
report does not issue `PASS_COMPLETE`, `VISUAL_ACCEPTED`, `MOCKUP_MATCHED` or a V1 pass, and no product code was
changed after the captures.

## 10. Delivery

Review pack: `%TEMP%\FirmwareSight-U1P-Visual-Acceptance-20261008T001110Z\09_review_pack\U1P_FINAL_VISUAL_REVIEW_PACK\`
- 58 files covered by `HASHES/SHA256SUMS.txt`, all re-verified, with `HASHES/SOURCE_MAP.txt` giving the absolute
source and both digests for every original.

Uploadable archive: `C:\Users\16429\Downloads\FirmwareSight_U1P_Final_Visual_Review.zip`,
4,727,314 bytes, sha256 `0f59f4cee90bd31e388e90e0b2097a621ab0c26811fbf50162b84f0f17f3881b`. CRC checked, 59
entries extracted and re-hashed against their sources, entry inventory matches the disk inventory, and the
forbidden-content scan is clean (no owner DB/WAL/SHM, no firmware binaries, no installers, no participant
material, no `.git`, no nested archives).

The prior-round material in `BEFORE/` came from the retained Temp evidence roots, not from Downloads: the
`FirmwareSight_U1_Final_Visual_Review` folder and its ZIP are no longer present in Downloads. Both retained
roots were verified before anything was copied - `FirmwareSight-U1R-Visual-Recheck-20261007T145124Z` checked
51/51 against its own `SHA256SUMS.txt`, and the U1 pack checked 28/28 - and the seven reference mockups copied
from the repository match the digests recorded in the accepted U1 pack byte for byte. No earlier evidence was
regenerated from a new build.

## 11. Interlocks that this round did not touch

P5 remains `PASS_COMPLETE`. The product remains `MVP_CANDIDATE` 0.6.0. V1 remains `IN_PROGRESS` /
`RECRUITMENT_READY` with zero eligible external sessions and participant execution paused; its F3 frozen cohort
artifact is unchanged and no external user was contacted. B1 / Private Beta, RC, GA, a GitHub Release, any tag,
signing, public distribution, the updater, licence selection and pricing all remain out of scope.
