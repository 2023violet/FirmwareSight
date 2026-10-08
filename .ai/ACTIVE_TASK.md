---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-10-08"
---

# ACTIVE TASK

```text
U1_UI_PRODUCTIZATION_CONVERGENCE — stage U1, a UI productization track, opened 2026-10-07.
Authorized by 《FirmwareSight B1 — UI Productization / Design Convergence》 v1.0 — Architect prompt, delivered
INLINE as message text (no file, so no owner-side byte stream to hash). The owner re-registered the track as
`U1` because `B1` is the canonical Private Beta identifier in 06_DELIVERY/06_STAGE_GATES.md and F3 §37 and V1 §40
both record it NOT AUTHORIZED. The transcription the agent archived lives at
U1_VALIDATION/00_authority/SOURCE_PROMPT_U1_transcription.md (503 lines, 16,762 bytes, SHA-256
67067dce624a6dc458055dd79e21b6ed4e983c7ac665426561e72b939de52865) and that digest is the transcription's, never
the prompt's.
Start authority: HEAD = origin/main = f481b78059c14e1c83d3ba18e082004b7de72ee2 (V1's activation commit), tree
clean, one worktree, and its run 37508243514 (#73, attempt 1) is 10 of 10 authoritative jobs.
Goal: converge the desktop React UI toward the frozen seven-screen reference set — one shared design-system
layer, then Overview, Analyze, Compare, Release Gate and Bundle & History — without touching Core semantics,
evidence classes, schema, migrations, storage, the analysis wire format, release identity, ADR-0028 or ADR-0029.
The reference images are direction authority for structure, hierarchy and density. They are NOT a functional
contract: no backend is faked for them, and the frozen precedence chain (治理红线 > frozen assets / ADRs /
assets/design-tokens.json > DESIGN.md > accepted screenshots) still decides what may enter the tree.
Deliverables: U1_VALIDATION/U1_UI_GAP_AUDIT.md (§6, written before code), U1_DESIGN_CONVERGENCE_PLAN.md (§12 B),
the UI code itself, and U1_VALIDATION/U1_VALIDATION_REPORT.md (§12 D, §11 validation, §13 honesty).
V1 IS PAUSED, NOT CLOSED: stage_status IN_PROGRESS, research_state RECRUITMENT_READY, eligible external
sessions 0, cohort build still frozen at the F3 artifact, and `v1_execution.paused_for` names this track.
P5 stays PASS_COMPLETE, Productization stays ENGINEERING_COMPLETE, G2 stays PASS, the product stays
MVP_CANDIDATE at baseline 0.6.0. Signing READY_NOT_EXECUTED, Notarization READY_NOT_EXECUTED, update
MANUAL_UPGRADE_READY, licence PENDING_OWNER_CONFIRMATION, public distribution NOT_AUTHORIZED, tag NOT_CREATED,
GitHub Release NOT_CREATED.
U1 may never self-issue a stage, a beta, an RC or a GA, may not resume V1 by itself, and its own §16 ends with
STOP. If you were sent here to "continue U1": the round closed; read
U1_VALIDATION/U1_VALIDATION_REPORT.md §6 for what it explicitly did not reach.

CURRENT STATUS OF THIS TRACK, as of the second U1 round on 2026-10-07:
U1 = READY_FOR_ARCHITECT_VISUAL_REVIEW.  participant execution = PAUSED_FOR_ARCHITECT_UI_REVIEW.
The continuation unit U1_PUSH_CI_AND_INSTALLED_VISUAL_ACCEPTANCE (prompt v1.1, Architect authorized; v1.0 is its
prior revision) pushed 8efe9c8 + 7dc2ca8, read its own remote CI at 10 of 10 (run 37609108402, #74, attempt 1),
installed the Windows artifact THAT run produced (artifact 11477857379, installer sha256 372631c367b34dc5…,
installed executable sha256 afdc528b97dcdce1…), and looked at it on a real screen: six primary states at
1440x900, nine responsive captures at 1024x720 and 1056x799, and a functional smoke pass on the same bytes.
Evidence and verdict: U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md, with the review pack outside Git at
%TEMP%\FirmwareSight-U1-Visual-Acceptance-20261007T1037Z\14_review_pack\U1_VISUAL_REVIEW_PACK.
One U1-V2 presentation finding is open and NOT self-waived (stale capability chips during a failed parse); no
U1-V3 or U1-V4 was observed. The owner's store was parked for the round and is restored byte-exact
(ORIGINAL_DB_RESTORED = YES, ORIGINAL_DB_SHA_MATCH = YES, OWNER_STORE_OPENED_BY_U1 = NO), and the machine is
back to uninstalled, which is how it was found.
This status is not visual approval, not PASS_COMPLETE, not DESIGN_COMPLETE and not MOCKUP_MATCHED; those belong
to the Architect. V1 stays paused with 0 eligible sessions and its frozen cohort build untouched.

CURRENT STATUS OF THIS TRACK, as of the third U1 round on 2026-10-07 (this block supersedes the line above it
that says "One U1-V2 presentation finding is open"; the earlier block is kept because it is a dated record of
what that round knew):
U1 = READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT.  U1-V2-06 = CLOSED_BY_U1R.  participant execution stays
PAUSED_FOR_ARCHITECT_UI_REVIEW.
The corrective unit U1R_STALE_CAPABILITY_CORRECTIVE_AND_FINAL_RECHECK (prompt v1.0, Architect authorized,
archived at 10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1R_Stale_Capability_Corrective_Final_Recheck_v1.0.txt,
delivered 28,659 bytes / SHA-256 0ff7b3f6495fb29b73ff5aaed03895f47ce0a133762b0e737718118008b650bb, 1,302 logical
lines) fixed the one material presentation defect the installed pass raised. The Analyze page resolved its
visible summary as the stored last-good one and rendered the top capability strip from it whenever any summary
existed, so a failed attempt or an unanalyzed selection showed the previous file's green "ELF supported /
MAP provided" pills above a row naming a different file. Retention was not the bug and was not removed: the
header and the strip now read from a derived presentation value that is populated only when the current attempt
earned it, the retained report and Details keep reading from the last good summary and keep saying whose they
are, no domain state was defined, no Gate verdict moved, and assets/design-tokens.json is byte-identical.
Three product paths changed (Analyze.tsx, Details.tsx, intake.test.tsx) and no Rust source did: 868 Rust tests
unchanged, 242 UI tests in 9 files (six added, none deleted), gate 17 of 17 with no SKIP, drift 8/8, deny 1/1,
core-smoke 3/3, package 4/4, baseline verifier PASS at 755 tracked files / 753 sum entries. Product commit
4d36f103d2a8fd103e8e3b466e4d2b2010a0a4ef, pushed 6f35b14..4d36f10 as a normal fast-forward; remote run
37637056980 (#76, attempt 1) 10 of 10; Windows artifact 11491960105 built from that commit installed and
re-checked on a real screen in the success, pending, failed-at-1440x900/1056x799/1024x720 and recovered states.
The owner's store was parked and restored byte-exact (OWNER_STORE_PARKED = YES, OWNER_BACKUP_HASH_MATCH = YES,
ORIGINAL_DB_RESTORED = YES, ORIGINAL_DB_SHA_MATCH = YES, OWNER_STORE_OPENED_BY_U1R = NO) and the machine is
uninstalled again, as it was found. Evidence: U1_VALIDATION/U1R_CAPABILITY_STATE_DESIGN.md,
U1_VALIDATION/U1R_CORRECTIVE_REPORT.md and %TEMP%\FirmwareSight-U1R-Visual-Recheck-20261007T145124Z\.
What U1R did NOT do: it did not implement the seven U1-V1 minor gaps or U1-V0-09, which stay
OPEN_FOR_ARCHITECT_VISUAL_JUDGEMENT; it did not resume V1, run a session or re-baseline V1's cohort artifact; it
did not open B1, Private Beta, RC, GA, a tag, a GitHub Release, a public installer, signing, notarization, an
updater, a licence choice or pricing. If you were sent here to "continue U1" or "finish the UI": the next word is
the Architect's visual verdict on the two screenshot packs, not another code round.
```

CURRENT STATUS OF THIS TRACK, as of the fourth and fifth U1 rounds on 2026-10-08 (this block supersedes the
`READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT` line above; its own opening status lines are superseded in turn by the
sixth round's block below it. Both are kept, because dated records are not rewritten here — each is what that round
knew when it stopped):

```text
U1 = REQUIRES_ARCHITECT_POLISH_REVIEW.  U1-V2-06 stays CLOSED_BY_U1R.  U1P-V2-01 is OPEN and NOT self-waived.
Participant execution stays PAUSED_FOR_ARCHITECT_UI_REVIEW.

The fourth round, U1P_FINAL_VISUAL_POLISH_AND_INFORMATION_HIERARCHY_CONVERGENCE, changed no capability and no
semantic surface. It recomposed the shell and five pages so the first viewport answers the page's own question:
the rail became a sibling of an independently scrolling main rather than a sticky element inside it; Overview
puts the ship verdict above the key figures and the ELF/MAP/Git facts became one compact band; Analyze leads
with the result band and the Sections/Symbols tables and drops the identity dossier below them; Compare reads as
a pair bar with the direction between the pickers and reports what moved before what it cost; Release Gate puts
the run's own sections directly after the verdict and numbers them in the order a reader meets them; Bundle &
History shows one stored entity at a time behind an aria-pressed button group, with the page heading finally
equal to the rail label. 253 UI tests in the same 9 files (was 242; eleven structural tests added, none deleted
or skipped), 868 Rust unchanged, 30 registered commands unchanged, assets/design-tokens.json byte-identical,
gate 17/17 with drift 8/8 including baseline integrity, deny 1/1, core-smoke 3/3, package 4/4. Five reversals
were run to prove the new hierarchy tests bite: each fails the guarding test with the mutation and passes after
the undo, and the tree came back byte-for-byte. Product commit
6a071c51c81a00b7a35facaefa5849e9975e715c, pushed as a normal fast-forward.

The fifth round, U1P_INSTALLED_ACCEPTANCE_AFTER_CI_RERUN, took those exact bytes to a real screen. Run
37668291893 (#79) is 10 of 10 on attempt 2, and the attempt history is part of the record rather than something
the rerun erased: attempt 1 left eight jobs green and cancelled Generated output drift and Package Ubuntu at the
six-hour runner ceiling while each was still executing the Linux prerequisite apt-get update, with no
repository-specific step started in either. That is the one class P5_VALIDATION/P5_CI_AUTHORITY.md lets a single
same-SHA rerun answer, and the owner authorized exactly one gh run rerun --failed. In attempt 2 only those two
jobs re-executed (drift 04:12:25Z->04:15:07Z, Package Ubuntu 04:13:01Z->04:24:51Z); the other eight carry
attempt 1's execution window to the second and are not described as newly run. No attempt 3 was issued. Windows
artifact 11504871803 (zip 5,538,316 bytes sha256 57651a3e...c1bb equal to its own API digest, NSIS 3,890,447
bytes f69cda6c..., installed executable 15,366,656 bytes 7327d374..., toolchain.git_commit 6a071c5...) was
installed with real input after the owner's store was parked. Sixteen captures were taken with the client size
re-read from the live window each time: P01-P06 at 1440x900, R01-R06 at 1024x720, R07-R10 at 1056x799. All six
first-viewport contracts came back PASS and the readiness verdict is still visible at 1024x720; the shell is
SHELL_PASS, proved by comparing before/after captures across six real single wheel notches (rail band
identical, main band and scrollbar strip both move) rather than by reading CSS. A thirteen-step functional smoke
found no S0/S1/S2 regression, and Prepare bundle was stopped at its preview so nothing was exported to disk.
The owner's store came back byte-exact with OWNER_STORE_PARKED, OWNER_BACKUP_HASH_MATCH, ORIGINAL_DB_RESTORED
and ORIGINAL_DB_SHA_MATCH all YES, OWNER_STORE_OPENED_BY_U1P = NO, and the fourteen unrelated entries in the
application-data folder compare identical on kind, size and digest. The machine had no install before the round,
so the round uninstalled, and the uninstaller's Delete-the-application-data option was never pressed.

Why the status moved down and not up: U1P-V2-01 is open. Overview's ship verdict does not name the build it
judges, and the page's own subject line can name a different one - the captured pair shows snapshot
snap-3f6...64ed363e with that build's 256 bytes and 47 symbols beside a PASS belonging to run
gate-deb...b202fdd6, whose subject build is snap-4b4087e1407c... as that row's own Details confirms. Nothing
stated is false: the run id is printed and the panel scopes its claim to the run named. The gap is that the
reader cannot check the pairing from Overview, because App.tsx holds the gate run and the last analysis as
independent state and clears nothing when a different build becomes current. U1P did not create that ownership;
promoting the verdict to the dominant element is what made the mismatch material. A narrow correction would name
the run's subject build in the panel, or suppress the verdict when it does not belong to the named snapshot.
Three lesser items are recorded with it: U1P-V1-01 (a wrapped band leaves an empty filled cell at 1024),
U1P-V1-02 (the Compare result does not survive navigation) and note U1P-V0-01 (the Analyze table header sits at
the fold at 1024x720). No U1P-V3 and no U1P-V4, and U1R's truthfulness holds in both failure captures.

Evidence: U1_VALIDATION/U1P_VISUAL_POLISH_PLAN.md, U1_VALIDATION/U1P_VISUAL_ACCEPTANCE_REPORT.md, the raw set
under %TEMP%\FirmwareSight-U1P-Visual-Acceptance-20261008T001110Z\, and the uploadable pack
C:\Users\16429\Downloads\FirmwareSight_U1P_Final_Visual_Review.zip (4,727,314 bytes, sha256
0f59f4cee90bd31e388e90e0b2097a621ab0c26811fbf50162b84f0f17f3881b). The BEFORE/ material came from the retained
Temp roots, not from Downloads, because the U1 review folder and its ZIP are no longer in Downloads; both roots
were verified against their own manifests first (28/28 and 51/51) and the seven mockups match the digests the
accepted U1 pack recorded.

What these two rounds did NOT do: they issued no stage - not PASS_COMPLETE, not VISUAL_ACCEPTED, not
MOCKUP_MATCHED, not U1_PRODUCTIZATION_COMPLETE; they did not resume V1, run a session, contact a participant or
re-baseline V1's frozen cohort artifact; they did not open B1, Private Beta, RC, GA, a tag, a GitHub Release, a
public installer, signing, notarization, an updater, a licence choice or pricing; and no product code was edited
after the screenshots, which is the condition that makes those screenshots evidence about these bytes.
If you were sent here to "continue U1": the next word is the Architect's on U1P-V2-01 and the three lesser
items, not another code round.

CURRENT STATUS OF THIS TRACK, as of the sixth U1 round on 2026-10-08 (this block supersedes the "Why the status
moved down and not up" paragraph above; that paragraph is kept because it is the dated record of what U1P-A1
knew, and it is the finding this unit was issued to close):
U1 = READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT.  U1P-V2-01 = CLOSED_BY_U1P_R1.  participant execution stays
PAUSED_FOR_ARCHITECT_UI_REVIEW.
The corrective unit U1P_R1_OVERVIEW_GATE_SUBJECT_CONSISTENCY (prompt v1.0, Architect authorized, delivered as a
file and archived byte-exact at
10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R1_Overview_Gate_Subject_Consistency_v1.0.txt, 33,484 bytes /
SHA-256 0263e0fe92b7065fcc6a8cf1280630ff753eb2e4415ecb07fd40c72bd8e8d024, blob df5b2d0dcc28e2438861c26e00ae5b0369a3e8a3)
bound the subject instead of restyling the page. App now hands Overview the selection handle that earned its
last-good analysis - the same handle Analyze has used for its own pending badge since before this round - and
Overview asks whether GateRunDto.snapshotId is the snapshot it is describing. Four states answer NEUTRAL: a run
that judged another build, a selection never analyzed, a stored run with no analysis in the session, and a loaded
policy whose fingerprint is not the one a run was judged under. Each names both full identities as text rather
than truncated-with-title, because a title attribute is unreachable by a keyboard reader, and each stops
restating the other subject's aggregate. Only the matching state shows the run's own sentence and counts, and it
now prints the judged snapshot in full. The no-run panel and the scope disclaimer are byte-for-byte what U1P
shipped. Truncated ids, file names, timestamps, commits, byte sizes and the run's baseline were all rejected as
substitutes, because any of them can agree while the subjects differ.
Four product paths changed (Overview.tsx, App.tsx, Overview.module.css, overview.test.tsx) and no Rust source
did: 868 Rust tests unchanged, 266 UI tests in 9 files (thirteen added, none deleted), gate 17 of 17 with no
SKIP, drift 8/8, deny 1/1, core-smoke 3/3, package 4/4, baseline verifier PASS at 889 tree lines / 762 sum
entries over 764 tracked paths, assets/design-tokens.json byte-identical. Five of the thirteen new tests failed
against a2e6b30's unmodified Overview.tsx before any code was written; with only the subject dispatch deleted
afterwards eight fail; removing the prop from App's Overview call breaks the authoritative typecheck with TS2741.
Product commit 42f75a7d2d90a4c25ca1a3c5c91f6ece16c2d5bf, pushed a2e6b30..42f75a7 as a normal fast-forward under
the operator's separate direct authorization. Its message was amended once, before any push, to replace a wrong
file count, with the tree identical before and after - both reflog entries remain, and nothing published was
rewritten.
Run 37753793004 (#81, attempt 1) is 10 of 10, read job by job with each job's own execution window checked.
Windows artifact 11539359554 (zip 5,539,797 bytes 45922e90..., NSIS 3,891,883 bytes dd5c8de8..., installed
executable 15,367,168 bytes 3bf541bb..., toolchain.git_commit 42f75a7...) was installed with real mouse input
after the owner's store was parked. Five scenarios ran on that screen: matched, mismatched, unanalyzed
selection, matched again after a new run, and a real policy swap that was EXECUTED rather than recorded
unsupported - with two analyzed builds, two stored runs and two loaded policy fingerprints, all of them named in
the evidence. A 1056x799 mismatch capture, a keyboard-only traversal that lands a visible focus ring inside the
readiness card, and a five-page regression glance came with them; U1R's stale-capability contract still holds in
both the pending and the ERR-FORMAT-0001 parse-failure states. Fifteen captures, sixteen log lines, one of them
a superseded mis-click kept in the log rather than edited out.
The owner's store came back byte-exact: OWNER_STORE_PARKED, OWNER_BACKUP_HASH_MATCH, ORIGINAL_DB_RESTORED,
ORIGINAL_DB_SHA_MATCH and UNRELATED_SIBLING_STORES_UNTOUCHED all YES, OWNER_STORE_OPENED_BY_U1P_R1 = NO, and the
seventeen application-data entries compare identical on kind, size and digest. The app ran against a disposable
store it created itself, hashed and then moved out by name rather than deleted; the round uninstalled as the
machine was found, and the uninstaller's Delete-the-application-data option was read through BM_GETCHECK, found
unticked, and never activated.
Two things are stated rather than smoothed. The NSIS uninstaller re-launches itself as Un.exe, so the borrowed
pid-waiting driver could not see its window and the wizard was finished through the handle it actually owns.
And one pre-existing Core behaviour was observed: capabilities.git renders "unknown" on Analyze while the Gate's
git.clean rule returns a definite BLOCK from real git evidence for the same snapshot, because
Capabilities::with_git() has no production caller. It is reported for the Architect's judgement, not filed as a
new U1P-V2/V3/V4 and not fixed, because Core is outside this unit's boundary.
U1P-V1-01, U1P-V1-02 and note U1P-V0-01 stay OPEN_FOR_ARCHITECT_JUDGEMENT exactly as U1P recorded them; this
unit neither fixed nor waived them. Evidence: U1_VALIDATION/U1P_R1_SUBJECT_CONSISTENCY_PLAN.md (written before
the code), U1_VALIDATION/U1P_R1_OVERVIEW_SUBJECT_CORRECTIVE_REPORT.md, the raw set under
%TEMP%\FirmwareSight-U1P-R1-Subject-Recheck-20261008T090319Z\, and the uploadable pack
C:\Users\16429\Downloads\FirmwareSight_U1P_R1_Overview_Subject_Review.zip (1,381,951 bytes, sha256
342fb89be6baf9f4e1670edba901ace57576112ec680af5e0e1f6cb3dee299a6).
This is still not visual approval: PASS_COMPLETE, VISUAL_ACCEPTED and MOCKUP_MATCHED remain states no UI round
may issue, V1 stays paused at 0 eligible sessions with its frozen cohort untouched, and B1 stays not authorized.
If you were sent here to "continue U1": the next word is the Architect's on the final visual verdict, and on the
three lesser items plus the git-capability observation above - not another code round.
```

The seventh U1 round, run the same day, is the current word on this track:

```text
U1 = READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT.  U1P-V2-01 stays CLOSED_BY_U1P_R1.  U1P-V2-02 is CLOSED_BY_U1P_R2.
U1-V2-06 stays CLOSED_BY_U1R.  Participant execution stays PAUSED_FOR_ARCHITECT_UI_REVIEW.

Unit U1P_R2_OVERVIEW_PENDING_SELECTION_EVIDENCE_SCOPE ran under 《FirmwareSight — U1P-R2, Overview
Pending-Selection Evidence Scope + Evidence-ID Consistency Audit, Architect execution prompt v1.0》 (delivered as a
file: 21,194 bytes, 449 logical lines, 448 CRLF pairs, SHA-256
3b6108b6e61582f9e31004d41e9768ba1a0daaaee1150b345c194a7e52b6221e; archived byte-exact at
10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R2_Overview_Pending_Selection_Evidence_Scope_v1.0.txt, stored blob
aea814d62902b78cd9a98ba164807abe3b0b7e95 = 20,746 bytes, SHA-256
fd8d28aafe411ac76dbf7262852f22a125c4b12ec3cadafbebc42b35f7a35102 — the 448-byte gap is the CRLF terminators Git
normalises out of the blob, measured rather than assumed). Start authority HEAD = origin/main =
60f10a6473ef446dc933c94a5edbab8e9b662ad1, tree clean, that head's run 37765205830 10 of 10.
What was wrong was scope, not wording. On a selection the user had not analyzed, Overview rendered its capability
band and its four figures from the RETAINED analysis while the MAP sentence described the NEW selection — the pair
the Architect saw, `MAP provided` beside `Symbol-level analysis depends on it`. U1P-R1 had already bound the VERDICT
to its judged snapshot; the bands around it were still speaking in one voice. Overview now derives every current
band from `selection.selectionId === analyzedSelectionId`: a pending selection is named `firmware.elf · not analyzed
yet` with its own MAP state and no pill and no figure, the Gate card stays neutral, the primary action is
`Analyze selected artifact` with `View previous Gate run` beside it, and the retained work sits under
`Previous analysis` inside bands named `Previous input capabilities` and `Previous key figures`, attributed in
audible text (a `role="note"`, not a `title`) — including the case where the new selection has the SAME file name as
the analyzed one, which is exactly why `selectionId` and not `fileName` is the handle. Nothing was hidden, no
retained value was deleted, and no Unknown was turned into a zero.
Three product paths changed (Overview.tsx +233/-71, Overview.module.css +13/0, overview.test.tsx +260/0) and no Rust
source did: 278 UI tests in 9 files (twelve added, R2-T1..T12, none deleted), 868 Rust tests unchanged across 47
result lines, gate 17 of 17 with no SKIP and no FAIL, drift 8/8, deny 1/1, assets/design-tokens.json byte-identical
at 94336906…, and the generate_handler! list is the same 30 named entries at both heads (counting `module::` pairs
under-reads it as 28 because list_fixtures and get_analysis_summary have no module prefix). Six of the twelve new tests fail on
60f10a6's unmodified Overview.tsx (6 failed / 29 passed) and all 35 pass after it; five bounded mutations each kill
at least one test and every one was undone by digest, the restored files re-hashing equal to the committed
implementation.
Product commit 2e2e21997ae01f035ac72b97658b3a0b329da362, pushed 60f10a6..2e2e219 as a normal fast-forward — the
authorization is the prompt's own §14 ("Push normal fast-forward (authorized if gates pass)") and the gate had just
come back 17 of 17 locally; no force, no history rewrite, no tag, no release. Run 37783219031 is 10 of 10, read job by job from GitHub's own API. Windows
artifact 11554271914 (zip 5,542,981 bytes 94f16ab5…, NSIS 3,895,064 bytes fd2d4935…, installed executable
15,367,680 bytes 196036e2…, toolchain.git_commit == 2e2e219) was installed with real mouse input AFTER the owner's
store was parked; the two artifacts this round must not use (11539359554, 11504871803) and any local build are named
in the report. Seven installed scenarios ran on that screen — matched, other-build, pending, pending-without-MAP,
parse failure after a good result, genuine recovery, and a real policy swap EXECUTED by loading a second project
whose policy differs by one committed line — with three 1024x720 captures, one 1056x799 capture, seven regression
glances and a keyboard-only traversal. Twenty-one capture rows and twenty observation rows: two grabs were
mis-scoped by the shared main scroll container, are labelled VOID in the log rather than edited out, and the gate
filters them from adjudication. All eleven identities the captures carry are length-checked against the product's own
SQLite rows and the committed fixture digests (69-character run ids, 149-character snapshots).
The acceptance gate 09_acceptance/ACCEPTANCE_GATE.json holds: 20 boxes PASS and 1 box MISMATCH_PROVED,
all_boxes_hold true, conclusion "U1P-V2-02 is closed by this round's installed evidence". The owner's store came
back byte-exact — OWNER_STORE_PARKED, OWNER_BACKUP_HASH_MATCH, ORIGINAL_DB_RESTORED, ORIGINAL_DB_SHA_MATCH and
UNRELATED_SIBLING_STORES_UNTOUCHED all YES, OWNER_STORE_OPENED_BY_U1P_R2 = NO — with the 14 unrelated application-data
entries identical on kind, size and digest across the 17-entry before and after inventories; this round's own
disposable store was hashed first and moved by name, never deleted; the machine was found uninstalled and is
uninstalled again; and the uninstaller's delete-the-application-data option was read through BM_GETCHECK as unchecked
and never activated.
Three things are stated rather than smoothed. The §7 identity audit did NOT confirm its own prompt's premise: the
U1P-R1 report's run A and snapshot B had each been transcribed one character short (68 where a SHA-256 forces 69,
148 where it forces 149) and the prompt's own expected strings carry the same short forms, so no document-to-document
comparison can see the defect. Both authorities — sha256sum of the committed fixtures, and the installed product's
own gate_runs/builds rows — say the same thing, so the minimum documented location was corrected with its exact
before/after bytes inside this unit's single docs-only successor; the already-hashed U1P-R1 pack was not rewritten or
repacked, and no separate audit commit was made. Second: the build-time note predicting ERR-PARSE-2002 for the junk
input was wrong; the installed product answers ERR-FORMAT-0001, and the original sentence is quoted unmodified beside
the correction. Third: both earlier delivery archives had left Downloads by pack time — a read-only check found them
in the Recycle Bin re-hashed to exactly the digests BASELINE.yaml records — so this pack's historical images are the
delivered bytes rather than a regeneration, but whether a moved delivery copy is itself a §13 gap is the Architect's
call, and one installed link is honestly missing: no press-then-window-title pair was recorded for Overview's two
pending actions, so their destination rests on R2-T5 and mutation M3 rather than on a pixel.
U1P-V1-01, U1P-V1-02 and note U1P-V0-01 stay OPEN_FOR_ARCHITECT_JUDGEMENT exactly as U1P recorded them, the
1024 wrapped-band empty cell and the git-capability question stay with the Architect, and this unit neither fixed nor
waived any of them. Evidence: U1_VALIDATION/U1P_R2_PENDING_SELECTION_CORRECTIVE_REPORT.md, the raw set under
%TEMP%\FirmwareSight-U1P-R2-Pending-Selection-20261008T132730Z\, and the single shareable pack
C:\Users\16429\Downloads\FirmwareSight_U1P_R2_Final_Visual_Review.zip (4,113,494 bytes, sha256
d3674569a0c4237452332b76ecda9b9a03b7ff9ece068f7a9d4a93d0441bb34c; 87 entries, CRC clean, 87/87 extraction round
trip, 0 missing / 0 unlisted / 0 invalid SHA entries, four composites labelled DERIVED with their raw originals
retained, forbidden-content scan clean). This is the SECOND build of that pack: the first (4,111,505 bytes, sha256
d74f6ee34f28fdede0c59b8ee40293ed9ea9068dcdbd8b2fd690fd5565e212e7, 86 entries) carried the same evidence but stated
28 registered commands in the report inside it where the list holds 30, and it was moved aside with an explanatory
note rather than deleted - see 09_review_pack\superseded\ in the evidence root.
This is still not visual approval: PASS_COMPLETE, VISUAL_ACCEPTED and MOCKUP_MATCHED remain states no UI round may
issue, V1 stays paused at 0 eligible sessions with its frozen cohort untouched, and B1 stays not authorized. If you
were sent here to "continue U1": the next word is the Architect's on the single U1P-R2 pack above — which contains
the U1P six-page baseline, the U1P-R1 before-images and this round's after-images in one archive — and on the four
open items listed above. There is no code round left for an agent to run.
```

The eighth U1 round, closed on 2026-10-08, is the current word on this track:

```text
U1 = READY_FOR_ARCHITECT_FINAL_VISUAL_VERDICT.  U1P-V2-01 stays CLOSED_BY_U1P_R1.  U1P-V2-02 stays CLOSED_BY_U1P_R2.
U1P-V1-01 and U1P-V1-02 are CLOSED_BY_U1P_R3, on installed bytes.  U1-V2-06 stays CLOSED_BY_U1R.
Participant execution stays PAUSED_FOR_ARCHITECT_UI_REVIEW.

Unit U1P_R3_FINAL_NARROW_CORRECTIVE_AND_INSTALLED_VISUAL_REVIEW ran under 《FirmwareSight — U1P-R3 Final Narrow
Corrective and Visual Closure, Execution Prompt v1.0》, delivered as a file: 32,849 bytes, 745 logical lines, 0 CR,
SHA-256 a185890edd043344a4332dfcc048fd63add1f8a5226dead4af373cc1ceab4de9; archived byte-exact at
10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R3_Final_Narrow_Corrective_and_Visual_Closure_v1.0.txt (blob
384e8ca42db8d8238d263c18111d58e5d8da1ab1, the same 32,849 bytes — this prompt was delivered with LF terminators, so
nothing was normalised away, measured rather than assumed). Start authority HEAD = origin/main =
f01eec110c76591c741d283e4ee862566e211a2f, tree clean, as recorded in
U1_VALIDATION/U1P_R3_NARROW_CORRECTIVE_PLAN.md before any code was written. That head's own CI was not read at round
start and is not claimed to have been: it was read at this closeout, job by job from GitHub's API — run
37801519343 (#84, attempt 1), 10 jobs, all success.
Three deliverables, and only three: Compare's unexplained blank, Overview's empty capability-band cell at 1024, and
an evidence pack whose totals are computed from its own acceptance record. No capability was added.
What was wrong in Compare was ownership, not rendering. App.tsx renders one page at a time, so Compare's own
useState died the moment the user navigated away, and the page came back showing its action above nothing. The
comparison session now lives in the shell — six pieces of state App owns and passes down as session/onSessionChange
— writes go through functional updaters so two writes in one render cannot clobber each other and an obsolete async
answer cannot resurrect a stale pair, a first session with two candidates and no result says "Ready to compare" and
names the next step, and a brand-new process does not resurrect an ephemeral diffId. The diffId is in-process only:
no SQLite row, no IPC call, no localStorage, no schema change.
What was wrong in Overview at 1024 was a track nothing filled. The shared band is an auto-fit grid whose gap is
painted in the border colour, so a wrapped line with three cells leaves a fourth cell-shaped rectangle of grey. A
local .fillBand class (display:flex, flex-wrap:wrap, children flex:1 1 the inspector-min token) fills the line
instead. components/Panel.tsx and Panel.module.css are NOT in the commit: a shared fill prop was implemented,
reviewed and reverted to its HEAD digest, because §10 prefers a local class and "provably inert on the other four
pages" is a weaker claim than "not touched". The bands the Architect already approved stayed untouched.
Thirteen tests came with it (compare.test.tsx T1-T9, overview.test.tsx T10-T12), 291 UI tests in 9 files, up from
278 with none deleted; 868 Rust tests unchanged in 47 result lines; gate 17 of 17 with no SKIP and no FAIL, drift
8/8 including baseline integrity, deny 1/1; assets/design-tokens.json byte-identical; generate_handler! still the
same 30 named entries. The RED run against the untouched tree reported seven failures — T1, T3, T4, T5, T6, T8, T9 —
and T3b, written after that run, is RED in its own separately recorded run, so eight of the thirteen are RED on the
old bytes and all thirteen pass at the head. T2 and T7 are recorded as locks rather than as RED, and so is the
Overview trio T10-T12: re-measured with HEAD copies of Overview.tsx, components/Panel.tsx and Panel.module.css
written back over the working copies, all 38 tests in that file pass against the unchanged product code, because
jsdom lays nothing out and a DOM test cannot see an empty band track — that question is answered only by the
installed screenshot. An earlier draft of the report claimed the trio was RED; that claim was wrong and the report
says so. Three mutation records, described as the log has them: M2 removes the Ready to compare panel (2 failed,
T1 and T9), M3 hard-wires the stale rule to false (4 failed — T5 and T6 plus two pre-existing U1P tests, which is
the proof the rule is load-bearing across rounds), and M1 is not a separate edit at all: page-local state IS
f01eec1, so the RED run is the retention measurement and recording it again would count one measurement twice. Each
measurement restored its files and re-checked their digests. Two self-corrections are in the report rather than smoothed over: a stray brace had made overview.test.tsx
collect zero tests while a header asserted three of them had passed, and the historical U1P Compare baseline image
carries the Windows taskbar in its bottom 33 px, so the capture harness now refuses any grab whose client bottom
falls below the work area and every R3 capture records client_inside_work_area: true.
Product commit 41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177 (12 files, +1,532/-58), pushed f01eec1..41bb6a36 as a
normal fast-forward under the owner's written authorization for this round; no force, no rewrite, no tag, no
release. Run 37828673549 (#85, attempt 1) is 10 of 10 read job by job from GitHub's own API. Windows artifact
11573661113 (zip 5,544,133 bytes 330e83af…, NSIS 3,896,257 bytes 9a51e86a…, installed executable 15,368,192 bytes
5c828ecf…, toolchain.git_commit == the product head) was downloaded from that run and installed with real mouse and
keyboard input after the owner's store was parked; the artifacts this round must not use (11539359554,
11504871803) and any local build are named in the report. Twenty-six capture rows and twenty-four observation rows:
the named matrix P01-P10 at 1440x900 and R01-R10 at 1024x720 / 1056x799 is complete, with C6's refusal sequence, the
Git cross-surface trace, the baseline-picker observation and a keyboard traversal beside it. One grab was discarded
for a misclick and ships in the pack labelled discarded, because a pack that only keeps flattering frames is not
evidence.
09_acceptance/ACCEPTANCE_GATE.json: 25 items — 23 PASS, 1 FAIL, 1 NOT_VERIFIED, 0 NOT_CAPTURED, and 2 of the 23 PASS
items flagged MISMATCH_PROVED (the stale policy fingerprint Overview names, and the refused same-build compare).
MISMATCH_PROVED is not a fifth verdict and not extra success: it marks PASS items whose subject IS a mismatch the
product proved, and none of them says the build is ready. The two findings this round measured and did NOT fix are
reported, not waived: Analyze's metric band leaves the same unfilled track at 1024 (126 wide rows, identical on the
R2-era image, so it predates this commit and is outside §7's scope and §10's allowlist), and the gate's baseline
picker resets on navigation. The §8 read-only trace found no false Gate PASS — git.clean is the gate's true
statement about a genuinely clean opened project while Analyze's Git UNKNOWN is about the artifact — so §8 produced
no STOP; the wrong-source sentence it did surface in Overview.tsx was repaired inside the authorized scope.
Owner data: OWNER_STORE_PARKED, OWNER_BACKUP_HASH_MATCH, ORIGINAL_DB_RESTORED, ORIGINAL_DB_SHA_MATCH and
UNRELATED_SIBLING_STORES_UNTOUCHED all YES, OWNER_STORE_OPENED_BY_R3 = NO, 12 unrelated digest-bearing application-
data files identical across the 17-entry before and after inventories, this round's disposable store hashed and moved
out whole rather than deleted, the uninstaller's delete-the-application-data box read as unticked and never clicked,
and the machine uninstalled again as it was found.
Evidence: U1_VALIDATION/U1P_R3_NARROW_CORRECTIVE_PLAN.md (written before the code),
U1_VALIDATION/U1P_R3_FINAL_NARROW_CORRECTIVE_REPORT.md, the raw set under
%TEMP%\FirmwareSight-U1P-R3-Narrow-Corrective-20261008T190057Z\, and the uploadable pack
C:\Users\16429\Downloads\FirmwareSight_U1P_R3_Final_Visual_Review.zip (4,224,650 bytes, sha256
4de07c7669574955493f2fd31594539422662415c14d8c5d378e2126ac831e52, 102 members, CRC clean, 102/102 extraction
round trip, 101 manifest entries with the manifest itself and the post-hoc verification record documented as
exclusions, 104 checks on the pack all passed, 8 composites labelled DERIVED over native-size unaltered originals,
forbidden-content scan clean). Its README counts are generated by summing the acceptance JSON's boxes array and the
builder refuses to finish when README and JSON disagree — including a negative control that alters one count and
requires the checker to reject it. The R2 archive is preserved untouched at its own digest and its stale
"MISMATCH_PROVED=0, PASS=0" README line is documented there as historical only. This is the SECOND build of the R3
pack: the first (4,223,656 bytes, sha256 b2e787cf411d808f6dcd21601bb3473976db1f2b2f3c2b263d766fd244b0ab45) had the
right counts but interleaved the negative control's intended rejection among its own checks, and it was moved aside
to 08_pack\superseded_first_build\ in the evidence root rather than deleted.
Three governance ordinals were wrong and are corrected by this successor, with the dated narrative around each left
intact: CURRENT_STATE.md called U1P-R1 "a fourth U1 unit" (it is the sixth) and U1P-R2 "a sixth U1 unit" (it is the
seventh), and ACTIVE_TASK.md introduced the U1P-R2 block as "the sixth U1 round". By the enumeration those documents
themselves give — U1, the continuation push, U1R, then the U1P product round and its installed acceptance as the
fourth and fifth — U1P-R1 is the sixth, U1P-R2 the seventh and this unit the eighth.
This is still not visual approval: PASS_COMPLETE, VISUAL_ACCEPTED and MOCKUP_MATCHED remain states no UI round may
issue, V1 stays IN_PROGRESS / RECRUITMENT_READY at 0 eligible sessions with its frozen F3 cohort artifact untouched,
and B1 stays not authorized. If you were sent here to "continue U1": the next word is the Architect's, on
FirmwareSight_U1P_R3_Final_Visual_Review.zip and on what it chooses to do with the two findings above and the
C2-first-viewport reading. There is no code round left for an agent to run, and §19 of the R3 prompt says do not
invent an R4.
```

## The V1 pointer that was live before U1, kept as the record of 2026-10-06

```text
V1_OWN_ARTIFACT_EXTERNAL_VALIDATION — stage V1, state IN_PROGRESS, research_state RECRUITMENT_READY,
opened 2026-10-06.
Authorized by FirmwareSight — V1 Own-artifact External Validation, Execution Prompt v1.0 — Architect
Authorized (file, SHA-256 48768ff025e3d6351b7b1d8d593ea8bfb18a7930a0a21a101af3093bbec10a0f, 46,207 delivered
CRLF bytes, 1,697 CRLF pairs, 1,698 logical lines), archived as
10_AUDIT/SOURCE_PROMPTS/FirmwareSight_V1_Own_Artifact_External_Validation_v1.0.txt (Git blob
d5e8d4575581bd656b568eed97090e7130a51774, SHA-256 c62314fee5032ca5ffdbcfe95f51cf3bdb783e7d55a42a165ea526686690c34e,
44,510 LF bytes) and registered in that directory's README.
Start authority: HEAD = origin/main = 08fdfcb710f78f8084bfcf614dc508c8b6e7e25b (P5 Commit F3), tree clean, and
F3's Run 37475580080 (#72, attempt 1) is 10 of 10 authoritative jobs.
Goal: measure whether real, external firmware/embedded engineers, using THEIR OWN real artifacts on the frozen
F3 build, can independently use Analyze / Compare / Gate, discover real new information, and intend to come
back. Report actual numerator / denominator for M1–M6 at N >= 8 eligible unique participants.
Discipline: REAL PEOPLE / REAL OWN ARTIFACTS / NO SIMULATED USERS / NO INTERNAL TEAM AS FORMAL PARTICIPANTS /
NO LEADING MODERATION / NO SECRET PRODUCT PATCHES DURING A COHORT / REPORT ACTUAL NUMERATOR / DENOMINATOR /
PRESERVE PRIVACY / CANDIDATE != TASK.

The cohort build is frozen and named (§1): run 37475580080, head 08fdfcb, artifact id 11419727517, NSIS installer
FirmwareSight-0.6.0-windows-x86_64-nsis.exe, 3,888,432 bytes, SHA-256
182506f213383cfe00865f199fcec4fb17079535e8ea370f954fc15097263d12, internal SHA256SUMS.txt verified OK on
download. NOT a substitute: F2R1's artifact, a local rebuild, cargo run, Vite, a later docs-only CI artifact, or
the V1 activation commit's own package. It is UNSIGNED, and public distribution stays NOT_AUTHORIZED: the build
moves one-to-one to named, consenting participants only.

THIS ROUND RAN §5's BRANCH, NOT §23 ONWARD. There were no real eligible participants and no session evidence,
so the round wrote the Recruitment Ready pack, opened the track, set research_state = RECRUITMENT_READY, and
stopped. Eligible external sessions = 0. Nothing was simulated, no LLM stood in for a person, no internal member
was counted, no quote or completion was invented, and no session file was created.

What V1 may not do (§4, §32, §33, §40): no feature implementation, no redesign from participant preference, no
schema / migration / dependency change, no cloud / account / network / telemetry, no updater, no signing, no
notarization, no licence selection, no pricing model or paid plans or invented anchors, no E1/E2/E3/GX, no B1 or
Private Beta, no RC, no GA, no public release. Product code is frozen for the length of a formal cohort, and an
S0/S1 finding stops sessions and returns to the Architect. This round changed no product byte: 868 Rust /
225 UI in 8 files and a 17-step gate must stay exactly as they are, or the round stops (§45).

P5 stays PASS_COMPLETE, Productization stays ENGINEERING_COMPLETE, G2 stays PASS, the product stays
MVP_CANDIDATE at baseline 0.6.0, and the release-readiness states are untouched: Signing READY_NOT_EXECUTED,
Notarization READY_NOT_EXECUTED, Update MANUAL_UPGRADE_READY, licence PENDING_OWNER_CONFIRMATION, public
distribution NOT_AUTHORIZED, tag NOT_CREATED, GitHub Release NOT_CREATED.

V1 may never self-issue V1_PASS_COMPLETE, B1_READY or PRIVATE_BETA (§39, §54), and even a full pass does not
open B1 (§40). AGENTS.md 1 still binds: this pointer names V1 and nothing beyond it.

If you were sent here to "continue V1": read V1_VALIDATION/README.md, then V1_PLAN.md, V1_METRICS.md and the
five protocol documents. If no real session has happened since the activation commit, there is nothing for an
agent to do — recruitment, consent and moderation are the human operator's (§48) — and writing a session file
for a session that did not happen is the one thing this track exists to forbid. When a real transcript or set of
notes arrives, follow V1_VALIDATION/sessions/README.md's sixteen-step intake path.
```

## The P5 closure sentence, kept as the record of 2026-10-06

P5 (Productization) closed on 2026-10-06 as **`PASS_COMPLETE`**, written by Commit F3 — the only head its prompt
reserved the closure sentence for — after F3 §6 re-audited every P5 exit criterion and found no required
engineering item `BLOCKED`. It was opened by *FirmwareSight — P5 Productization, Execution Prompt v1.0* (file,
SHA-256 `722125f5…71e0ae`, 73,722 bytes, 3,442 lines) and closed under *FirmwareSight — P5 Commit F3 Final
Governance Closure, Execution Prompt v1.0* (file, SHA-256 `860e0097…64514c`, 36,458 delivered CRLF bytes), both
archived in `10_AUDIT/SOURCE_PROMPTS/`. Machine state at that head: stage P5 = `PASS_COMPLETE` · Productization =
`ENGINEERING_COMPLETE` · product = `MVP_CANDIDATE` · narrative = *FirmwareSight Productized MVP Candidate* ·
`baseline_version` = `0.6.0` · G2 = `PASS`, with P0/P1/P2/P3/P4 keeping their existing states.

What F3 wrote into `active_task` at its own head was `NONE`, and the sentence it attached — "V1 = NOT AUTHORIZED
BY P5" — was true of that commit: no file in that tree authorized V1. The pointer moved off `NONE` only when the
architect issued the V1 prompt itself, which is the same way P5, P4 and G2 each arrived. V1 arriving does not
retroactively authorize anything P5 declined: B1 / Private Beta, RC, GA, commercialization, licensing, signing,
notarization and an updater are still unauthorized, G3 is still not started, and V1's own §40 closes its door the
same way.


## What is already true when P5 opens

The product is **not** in question: G2 is PASS, Product MVP is ENGINEERING COMPLETE, the state is
**MVP CANDIDATE**, the baseline is **0.6.0**, and the three post-G2 findings were closed on 2026-10-02 in
`e816dcb` and `971015f`. What moved is the tree, so the G2 numbers are the history of a different commit: the
tree at that point read **775 Rust tests, 160 UI in 6 files, and `scripts/check.py` 16 steps** (the drift group
gained `version identity` when packaging landed). **Nothing in this paragraph is the present total** — the
present total is in the last ledger row above and in `.ai/HANDOFF.md`: **868 Rust / 225 UI in 8 files** on a
**17-step** gate.

`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` is written and answers §4 A–H from commands run at `d83175a`
(HEAD = `origin/main`, tree clean, Run `37101619245` 7 of 7). It found no package has ever been produced
(`bundle.active: false`, no CI packaging step, no `.icns`), that every artifact version is `0.1.0` while
`0.6.0` is document-only, that History needs read APIs rather than tables, that migration `0005` is
required for the L6/L7 numeric-Unknown reasons and not for History, that `PRAGMA integrity_check` and any
database backup appear nowhere, and that the committed fixtures are all `arm-none-eabi-gcc` + GNU ld with
zero Clang-produced evidence despite the cohort claim.

## What has landed since the audit

| Commit | What it did | Gate run | Result |
| --- | --- | --- | --- |
| `4cc8d93` | §5 governance opened + the audit + the archived prompt | `37125456689` | success, 7 of 7 |
| `812b472` | `P5_MIGRATION_DECISION.md`, then migration `0005` and its write/query/IPC/UI path | `37127791999` | success, 7 of 7 |
| `9e3b1de` | artifact versions unified on `0.6.0`; seven goldens regenerated | `37128593254` | **failure, 6 of 7** |
| `20b03e3` | test-only repair of the pre-existing race that run lost | `37129900728` | success, 7 of 7 |
| `0c031cd` | `bundle.active`, three CI package jobs, `scripts/verify_package_artifacts.py`, `drift/version identity` | `37133706214` | **failure, 7 of 10** |
| `1055242` | the package group finds `cargo-tauri` through `cargo tauri`, a `SKIP` can no longer read as a pass, and the step runs from `apps/desktop` | `37138881977` | success, **10 of 10** — and the first run to attach a built package on all three platforms |
| `53578e9` | a `.app` indexed file by file, so the index §41 asks for is readable by `sha256sum -c`, plus each payload's own digest in the metadata | `37143046338` | success, **10 of 10** — and its darwin set, downloaded and checked, closed the index row it was fixing |
| `a674774` | the read-back written into `P5_PACKAGING_REPORT.md` §5d/§5e and §9, `P5_CI_AUTHORITY.md`, the audit, and `04_TECH/17`'s stale sentence about the CLI companion corrected — the CI sets do archive it, and all three were listed | `37145302229` | success, **10 of 10** |
| `9ab089f` | `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md`: §38 A–L and §39 measured on this host with the owner's store parked, hashed and restored | `37147434366`, `37147577288` | success, **10 of 10** each — and §38 C stayed unimplemented, because onboarding did not exist yet |
| `e863d0c` | §13 first-run onboarding + §14 Help/About (with L21's window title fixed in Rust over a closed page enum) + §15–§18 local History over three new bounded storage **read** APIs and **no new migration**. 19 storage + 11 desktop + 40 UI tests, seven mutation proofs, `P5_ONBOARDING_HISTORY_REPORT.md` and its design checklist | `37154946484` | success, **10 of 10**, first attempt — locally too: **16 of 16** steps, **812 Rust / 200 UI** |
| `e070508` | documentation only: wrote Commit C's run back into `P5_CI_AUTHORITY.md` and corrected four governance sentences that had said the read-back had not happened | `37156287999` | success, **10 of 10**, first attempt |
| `111fe32` | two things, in one gate: §32's L22 item — `P5_RELEASE_IDENTITY_ADR_DRAFT.md` (options costed, option C is today's bytes-as-evidence semantics, four questions for the Architect, then STOP) with its premise pinned by `a_notes_file_that_differs_only_in_line_endings_is_a_different_release` — **and** the test-only repair of L23's sixth instance, which the **local** gate lost while verifying it and CI never saw | `37158606478` | success, **10 of 10**, first attempt — preceded by a local `python scripts/check.py` **16 of 16** at **813 Rust / 201 UI**, three mutation proofs and 20 clean fresh-process runs of the changed suite |

| `3400981` | Commit D: storage-owned `PRAGMA integrity_check`, a WAL-safe online-backup snapshot before any older store is migrated, the migration matrix re-run against it, a closed 41-key Diagnostics allowlist with a native-dialog export and positive-control privacy tests, the typed startup refusal, and §32's L22 answer as `ADR-0028` — `P5_COMMIT_D_DESIGN.md`, `P5_DIAGNOSTICS_RECOVERY_REPORT.md` and its design checklist. 131 storage / 222 desktop Rust, 210 UI, eight mutation proofs | `37200245520` | **failure, 8 of 10**, first attempt: `macOS Core Smoke` and `Rust (ubuntu-latest)` both panicked at `integrity_and_backup.rs:814`, on a test this commit added. Its `assert_ne!` compared two snapshot files whose only available difference was the second-granular `applied_at` default, so the claim was a race with the clock and it passed on Windows and lost on the other two runners. The product took no blame — `Rust (windows-latest)`, both UI jobs, drift, deny and all three package jobs were green on the head — and the **eighth** mutation proof (an un-replaced snapshot reddens the repaired assertion on its own) plus 20 fresh-process runs are the repair's evidence. Landed as the test-only commit `bccea88`, named in the row below |
| `bccea88` + `90aa69d` + `956e250` | the test-only repair of that race, then the record of it (§0a's third red run, L26 added to the disposition table), then §32's read-back successor — report §8 with the twelve facts, and the "31-field" allowlist figure corrected to the **41** keys `ALLOWED_KEYS` actually names | `37202016141`, `37202301591`, `37204847474` | success, **10 of 10** each, first attempt, every job read individually — including the two runners that had lost the race. **COMMIT_D = COMPLETE**, `P5 = IN_PROGRESS`, §34's STOP in force; `956e250` is the head these rows were written from and the closeout that follows records its run rather than this commit doing it |
| the head these rows describe is `59d85c3` (E2) over `859648e` (E1); `bfbc1aa` is the base they started from | **Commit E**, in two commits. E1: the compatibility cohort — 6 new fixture directories and 31 new files under `fixtures/elf/p5-compat/` (manifest 25 → 56 entries), built by this host's real `arm-none-eabi-gcc 14.3.1` (Arm GNU Toolchain 14.3.Rel1) and `clang 22.1.8`, both final-linked by GNU `ld 2.44.0.20250616`, and committed as bytes, covering §13's cases B/C/D/E/G, the Clang arm of case F, and per-fixture `PT_LOAD` counts for case H; `p5_compat_fixtures.rs` (14 tests) reads every recorded hash, proves each compiler identity from the ELF's own `.comment`, and holds the product to image/RAM expectations derived independently from `readelf` + the MAP region attributes rather than from the model's own arithmetic. Those expectations found the round's one product defect: clang's `.ARM.exidx.text.main` has `sh_type` `0x70000001`, which the parser read as "no role", which it read as "not allocated", so 8 real FLASH bytes entered neither budget while the total still printed `Exact`. Fixed the narrow way the owner approved — `SHF_ALLOC` read from the section header, as `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §3 and §4 item 3 already require — proven by five mutations (A, B, C, E and F, where A and E target the fix, B and C the fixture bytes, and F the one UI test the sweep rewrote). E2: the supportability sweep over §48's fourteen rows — L14, L16, L19, L20 and L24 **CLOSED** on measurement (L14 by revalidation, L24 closed-and-not-reproduced), L4, L5, L17 and L23 **REDUCED** with their residue named (L23's seventh instance repaired test-only, then 20/20 fresh-process runs), L25 **NOT_REPRODUCED** (30 legitimate Apply activations), L8 and L12 **CARRIED_FORWARD** with what was measured about them, and L15 written up as the three-namespace serialized-contract question it actually is and **STOPPED for the Architect** under §42 — a stop the Architect has since answered, see the next row | `37228929762` at `59d85c3`, attempt 1 — and `859648e` has **no run of its own**: the round was one `git push` of two commits, GitHub starts one run per push at the tip head, and `gh run list --json headSha` confirms it. That is recorded rather than papered over, and E1's bytes are verified by the tip run plus the clean detached worktree | **868 Rust / 217 UI in 8 files** (854 → 868, the +14 all in the new fixture target; 210 → 217, +6 in `compare.test.tsx` and +1 in `details.test.tsx`, while `history.test.tsx` changed a test without adding one), local gate **16 of 16**, `--only package` **4 of 4** with no `SKIP`, `core-smoke` 3/3, drift 7/7, deny 1/1, UI reliability campaign **20 runs / 217 tests / 0 failing**, worktree gate **18 of 18** — and the remote agreed: **10 of 10**, every job and every step read individually |
| `6981625` | Commit E's §59 read-back successor: the candidate's run written into `P5_CI_AUTHORITY.md` and the governance entry points, the "last head that changed a line of Rust" sentence dated instead of left to mislead, and §7a added to the fixture report for the clean detached worktree. Documentation only | `37230689636` | success, **10 of 10**, attempt 1, every job and every step read — this head is the first Commit E row that can carry its own run, because the run it needs belongs to the head after it |
| `80b47c4` | **Commit E closure normalization**, documentation-only under §1 of its own prompt: the matrix status column normalized to the five canonical values (38 cells, 0 outside the vocabulary — the old `BEST_EFFORT_NO_CLAIM`, `DEFERRED_NO_MVP`, `NOT_CLAIMED`, `MEASURED, NOT INFERRED`, `BUILT_AND_VERIFIED_IN_CI` and the half-sentence cells all moved to the evidence column, and release readiness moved to a `state` column so `READY_NOT_EXECUTED` / `UPDATE_READY_MANUAL` stop pretending to be compatibility statuses); the A–H disposition column normalized to the four canonical words with the architect's mapping — **A, F, H existing; B, C, D, E, G new**, because a new assertion on an old fixture is not a new fixture; **L15 answered** as Option E and rewritten from a stop into a decision (`RESOLVED_BY_ARCHITECT`), with `04_TECH/23` §7 documenting the legacy identifier's meaning and the supportability row now `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`; the prompt archived with both hashes recorded | its run is now a read-back fact: `37262147348` (#67) went **9 of 10 on attempt 1**, the single red being `Generated output drift` at rustup on the Ubuntu runner ("recovering from a partially installed toolchain", then `failed to install component: 'clippy-preview-x86_64-unknown-linux-gnu', detected conflict: 'bin/cargo-clippy'`, rolled back ~0.5 s in before compiling anything) — provisioning, corroborated by the same SHA's green `Rust (ubuntu-latest)` — and **10 of 10 success on attempt 2** after one `gh run rerun --failed`, which re-executed that one job and carried the other nine attempt-1 executions forward. Two attempts, stated as two attempts; §27 still forbids a further commit merely to record it | direct counts unchanged and required to stay unchanged: **868 Rust / 217 UI in 8 files**, gate **16 of 16**, `core-smoke` 3/3, drift 7/7, deny 1/1, package 4 of 4 with no `SKIP`. Zero paths under `crates/`, `apps/`, `scripts/`, `fixtures/`, `schemas/`, `golden/`, `.github/`; `Cargo.lock` and `pnpm-lock.yaml` untouched; owner's store never opened |
| `57a904e` | the documentation closeout after Commit D: the handoff entry points reconciled with the tree the round left — P3-era `556 / 135 / 15` figures labelled as a snapshot beside the measured `854 / 210`, the P5 head chain added to the `.ai/HANDOFF.md` ledger, `.ai/README.md`'s stage block rewritten, the install report's "not built" claims dated, `P5_MIGRATION_DECISION.md` moved to `DECIDED_AND_IMPLEMENTED`. No product source in the diff | `37212546755` | success, **10 of 10**, first attempt, every job read individually. Two sentences it shipped were false the moment it was committed — `.ai/README.md` and `.ai/HANDOFF.md` each called `956e250` the "current HEAD" — and this row's own commit, `bfbc1aa`, is that correction — which is also Commit E's base, so its run belongs to the head after it under the same rule |
| the head that lands this row | **Commit F1** — L26, decided and implemented. `09_ADR/ADR-0029-repository-baseline-checksums-use-git-index-blobs.md` settles that the root `SHA256SUMS` records the SHA-256 of each path's **canonical Git stage-0 index blob**, not its working-directory bytes, and is deliberately distinct from `ADR-0028` because a repository baseline and a release identity are different evidence domains. The generator now digests one batched `git cat-file --batch` fed by `git ls-files -s -z` — bytes end to end, so the non-ASCII paths survive — refuses any stage ≠ 0, and refuses to write while a tracked path carries an unstaged **semantic** change, which is what makes the recorded order (tree → stage → sums → stage) self-enforcing rather than a habit. The verifier shares no helper with it, reads both artifacts *from the index*, resolves content through `:<path>`, and fails closed on an unmerged entry or worktree/blob drift. It runs in the authoritative gate as `drift/baseline integrity`, so all three CI platforms prove it. L15's user-visible residue closes display-only: `Details.tsx` captions the legacy `ElfProgramHeader` / `elf.program-header` token as **ELF address + flags evidence** while the stored value, the DTO, `analysis:1`, the goldens and the bundle keep carrying the identifier — and the new test asserts both halves. Plus `P5_COMMIT_F_DESIGN.md` written before the code (§19), the prompt archived with its measured digest, and proofs A–G | its own run is external evidence, read with `gh run list --json headSha` | **868 Rust / 219 UI in 8 files** (the +2 are the caption regression's two cases), gate **17 of 17** — 16 → 17 because the drift group gained a step — with drift **8 of 8**, `core-smoke` 3/3, deny 1/1, package 4/4 and no `SKIP`, and a clean detached worktree at **19 of 19**. No schema, migration, dependency, capability, workflow, fixture or golden change; `ADR-0028` and `.gitattributes` untouched; §64 unrun; the owner's store never opened |

| the head that carries this row | **Commit F2 — installed productization acceptance.** Documentation, evidence and governance only: **zero** paths under `crates/`, `apps/`, `scripts/`, `fixtures/`, `schemas/`, `migrations/`, `.github/`, and no change to `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json` or `deny.toml` — §3/§35 forbid them because any one invalidates the F1 installer as the candidate under test, and three product findings were therefore recorded rather than patched. Ran the whole §64 journey once, contiguously, on the exact CI-built F1 Windows artifact (artifact id `11337963032` of run `37293381181`, 3,885,631 B, `a1152ef3…3c076b`, installed as downloaded), driven by real `SendInput` with read-only SQL used only as independent evidence. Proved the installed synthetic **v4 → v5** migration on 34 measured checks; proved retention semantically rather than by file digest, because a WAL database churns bytes legitimately; parked and restored the owner's store **twice** with the §26 gates satisfied before any commit. Wrote §28's eight-document closure pack, §29's user-doc audit — which found that `Help.tsx:50` already points readers at `P5_VALIDATION/P5_KNOWN_LIMITATIONS.md`, a path that **did not exist until F2 created it**, fixed without touching product code — §30's user-facing uninstall/retention wording, and §31's coverage boundary. **Two boundaries were written down instead of being smoothed over:** installed migration is one path and the owner's real store is at **schema v2**, so the chained v2 → v5 installed upgrade remains unexecuted; and the uninstaller's "Delete the application data" option was **never exercised**, because that folder holds unrelated historical stores from earlier phases. Also recorded against itself: three harness defects found and owned, one of which had been silently delivering installer clicks to a browser window for most of the round, and the F2 prompt file vanishing from `Downloads` after it was read — the archived copy is a transcript reconstruction, verified contiguous over lines 1–1,044 and matching the 19,040 bytes the directory listing recorded, and the register says so | its own run is external evidence, read with `gh run view <sha> --json headSha,conclusion,jobs` | **868 Rust / 219 UI in 8 files — both unchanged from F1, which is the check §37 names**: a docs-only round that moved either count would be a product change wearing a documentation diff. Full gate **17 of 17**, drift **8 of 8**, deny 1/1, `core-smoke` 3/3, package 4/4 with no `SKIP`. `F2 = COMPLETE`, `P5 = IN_PROGRESS`, `F3 = READY_FOR_ARCHITECT_REVIEW`. Not set: `P5 PASS_COMPLETE`, `active_task NONE`, a tag, a GitHub Release, a signature, a notarization, an updater, or a licence |
| `fb5f628` | **Commit F2R1 — the narrow installed-UI corrective the Architect inserted between F2 and F3.** Three F2 real-desktop findings, all of them display-layer, fixed inside §16's only allowed product path `apps/desktop/ui/src/**`: **F2R-01 / S2**, Analyze → Sections laid its prose column out one character per line, because `Details.module.css` put `display: block` on the `<table>` — which pins the box to the pane and lets its columns sit *below their own minimum* — while `overflow-wrap: anywhere` on the reason made that minimum one character, so the column absorbed the whole deficit; **F2R-02 / S3**, History's `Details` was the last cell of a table wider than the frozen minimum window, so the only action on the row was the thing that fell off the right edge; **F2R-03 / S3**, Release printed the Core enum `MapRegionAndElfLoad` at a person. The mechanism was measured before it was changed: `P5_F2R_UI_CORRECTIVE_DESIGN.md` §3 keeps the probe numbers, including the negative control showing that a wrapper alone does not fix it and the 195 px / 163 px occlusion that is why sticky pinning was rejected. The repair is what §5 asks for first — a `.viewport` wrapper owns `overflow-x: auto`, the tables are tables again and so keep their intrinsic minimum, prose cells take a `24ch` measure with `break-word`, an Unknown reason sits on its own line under the word, and the History action moves to the **leading** cell, the placement the Evidence table already uses. F2R-03 routes Compare, Release and the Analyze weakest-basis sentence through one new `evidenceBasis.ts` holding the captions Compare used to inline, because §11 forbids divergent wording for the same evidence and §13 authorizes the extra surfaces that share the helper. **No token changed, no pixel constant invented, no dependency, no capability, no contract move**: `assets/design-tokens.json` is byte-identical and `analysis:1`, `diff:1`, `gate-results:1`, `accepted-reviews:1`, `release-manifest:1`, SQLite schema 5, migrations 0001–0005, ADR-0028, ADR-0029 and `elf.program-header` all stay as they are — this round is display/layout only | run `37431977428` (#70), attempt 1 — read back in `P5_CI_AUTHORITY.md`, all ten jobs and every step read individually as §41 requires; §25 forbids borrowing F2's run, and this head has its own | **868 Rust unchanged / 225 UI in the same 8 files** (219 → 225: the three contracts and their guards), full gate **17 of 17** with drift **8 of 8**, deny **1 of 1**, `core-smoke` **3 of 3**, package **4 of 4** and no `SKIP`; §20's four mutation proofs each reddened their own test; §21 re-run on the tree actually committed and §22 reliability **20 repetitions / 20 green / 0 failing / 7 min 25 s**, both re-taken after three comment-only lines moved (the superseded runs are kept, not deleted, and §9 of the design record says why). **Not done and not claimed here:** §26's exact F2R1 artifact download and §30–§33's focused installed revalidation at 1024×720 / 1056×799 / 1440×900, so no F2 finding is closed by this head — §36 requires the installed artifact first. **F2R2 below is the head that ran it and closed them.** `P5 = IN_PROGRESS`; no `P5 PASS_COMPLETE`, no `active_task NONE`, no F3 state |
| the head that lands this row | **Commit F2R2 — the installed round F2R1 was forbidden to claim.** §26's artifact and §30–§33's focused revalidation ran on **run #70's own Windows package**: artifact id `11397938806`, `FirmwareSight_0.6.0_x64-setup.exe` 3,886,598 B `efbc45a3…d5fd4`, installed `firmwaresight-desktop.exe` 15,362,048 B `2cf01a6d…670b` — not F1's `11337963032`, not F2's binary, no local `cargo`/`tauri` build, and the artifact set's own `SHA256SUMS.txt` verified before it was run. The harness was hardened **before** any installed action (`require_app()` refuses `NO_PROCESS`/`NO_WINDOW` instead of falling back to another application's window — the first probe had matched a window titled "Claude"), and every click still passes the pixel-ownership guard that refused one real click this round (`POINT_OWNED_BY_OTHER_WINDOW`, pid 26036 owned the pixel, no input sent). **All three findings CLOSED** at 1024×720 / 1056×799 / 1440×900: Sections prose wraps at a readable measure with the table area owning the scroll, `Details` leads every History row and survives `SPACE` close/reopen with a visible focus ring, and Release reads "ELF address/flags evidence" / **"MAP regions + ELF load evidence"** where F2 captured `MapRegionAndElfLoad`. The 1440×900 trade is recorded rather than glossed: content ≈1.06× the pane, a contained scroll where the pre-fix layout fitted the pane by rendering one character per line. §33 smoke green (Analyze, Compare, Gate ×2, History, Diagnostics; no crash, no new path leak), §34 `P5_DESIGN_ACCESSIBILITY = PASS_FOR_FROZEN_DESKTOP_SCOPE` with `WCAG_CERTIFICATION = NOT_PERFORMED`, `MULTI_DPI_125_150 = NOT_TESTED`, `SECOND_WINDOWS_HOST = NOT_TESTED` unchanged, §35 **L20 = CLOSED ACROSS VERIFIED HUMAN-FACING MEMORY-BASIS SURFACES** without rewriting Commit E's text, and the owner's store restored byte-exact with `OWNER_STORE_OPENED_BY_F2R = NO`. F2R2 itself is **docs/evidence/governance only** (§42), writing `P5_F2R_UI_CORRECTIVE_REPORT.md` with §37's twenty parts plus the §38 addenda to `P5_KNOWN_LIMITATIONS.md` §8, `P5_EXIT_CHECKLIST.md` §4–§6, `P5_DESKTOP_ACCEPTANCE_REPORT.md` §9, `P5_CI_AUTHORITY.md` (F2's heads and F2R1's run, attempts read individually) and these entry documents | its own run is external evidence, read with `gh run list --json headSha,runAttempt` after the push — a head cannot certify the worktree and archive proofs of its own SHA, and §44 forbids a further head written only to record it | **868 Rust / 225 UI in 8 files — both identical to F2R1, which is exactly the check §43 names**: a docs-only round that moved either count would be a product change wearing a documentation diff. Full gate **17 of 17** with drift **8 of 8**, deny **1 of 1**, `core-smoke` **3 of 3**, package **4 of 4** and no `SKIP`, and the baseline verifier **RESULT PASS** at the identical staged state (tracked **708**, entries **706**, every mismatch counter 0). This row first carried F2R1's figures (707 / 705) and a worktree figure it had not yet earned; the head that lands it corrects them to the measured ones, because §23's order generates `SHA256SUMS` from the stage-0 blobs and only then counts them. The two proofs that can only address a SHA that exists — the clean detached worktree and `git archive` extracted outside the repository — stay §3's asymmetry: executed after this head, reported to the Architect, and not self-certified inside the commit they prove, which is also why §44 forbids a further head written only to record them. `F2R = FINAL PASS / COMPLETE`, `F2 = PASS`, `P5 = IN_PROGRESS`, product **MVP CANDIDATE**. `F3 = READY_FOR_ARCHITECT_REVIEW` is this round's conclusion **and** is gated on that external run reading 10 of 10; F3 is *not authorized here*, no F3 work was started, and no P5 closure token, `active_task NONE`, tag, GitHub Release, signature, notarization or updater appears in any of it |
| the head that lands this row | **Commit F3 — final governance closure: the last authorized unit of P5, and the only head that may write the stage's verdict.** Authorized by *FirmwareSight — P5 Commit F3 Final Governance Closure, Execution Prompt v1.0 — Architect Authorized*, archived at `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitF3_Final_Governance_Closure_v1.0.txt` with **both** digests because transport moves them: delivered bytes SHA-256 `860e00976b242f15de1473455941140599a8a5cd639409521e7c38a28a64514c` (36,458 B, 1,821 CRLF lines, no final newline) and stored Git blob `f00751304ebf789f67b8b5355864a5ca3eefcd5c` → SHA-256 `6cacd10ee1eff4a6c4ab90b4f9d51df987bad16cf94fe142961c0d37fd749c66` (34,637 B, 1,821 LF). The delta is exactly the 1,821 carriage returns and the line-by-line comparison returns identical text on all 1,822 logical lines with zero differing lines; `cmp` proves the archive is the delivered file, so nothing was reconstructed, and `.gitattributes` was not touched to make the hashes agree (`AGENTS.md` 9). **Order was not negotiable: §6's exit re-audit ran before any closure sentence.** All nineteen areas were re-read against measurements and **no required engineering item came back `BLOCKED`** (`P5_EXIT_CHECKLIST.md` §7) — Design/Accessibility carried F2R's `PASS_FOR_FROZEN_DESKTOP_SCOPE` **exactly**, with `WCAG_CERTIFICATION = NOT_PERFORMED`, `MULTI_DPI_125_150 = NOT_TESTED` and `SECOND_WINDOWS_HOST = NOT_TESTED` untouched (§7), and Security/supportability stayed at the bounded sentence, never "security clean". The re-audit's one real find was in this repository's own paperwork: the checklist asserted every L1–L26 row was accounted for while `P5_KNOWN_LIMITATIONS.md` held **25 of 26 — L12 had no row**, though the two accepted RustSec advisories it covers were carried honestly in `P5_SUPPORTABILITY_REPORT.md` §2 and `P5_SECURITY_SUPPORTABILITY_REVIEW.md` §2. F3 added the row and corrected the assertion instead of deleting the claim, then §11's rule held the rest: **no limitation became `CLOSED` merely because P5 closed** — L3, L4, L5, L8, L11, L12, L13, L15, L17, L23 and L25 all keep their carried dispositions. L15 stays split (presentation `CLOSED`, wire `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`, no enum rename, no `analysis:2`), L26 is `CLOSED — ADR-0029` kept deliberately distinct from `ADR-0028` with no new checksum semantic, L20's reopen/reclose is reconciled **with dates** rather than by rewriting Commit E's measurement (§8/§23), and §12's performance truth is quoted unchanged: 519,179,252 bytes, ~2,020,073 symbols, warm ~4.73–4.89 s, cold first read ~68.7 s, ~1,428 MB peak observed working set, never "Not Responding" — near-500-MiB UI `MEASURED`, first-use-under-60 s **not proved at this workload**, and no claim that performance was optimized, that a 500 MB target was achieved or that every large file finishes inside a stated window appears in the tree. §13 froze all ten release-readiness states and §14 wrote the consequence rather than resolving it: **`PUBLIC OPEN-SOURCE REDISTRIBUTION CLAIM = BLOCKED BY OWNER LICENSE DECISION`**. §24 kept this round off the machine entirely: **nothing installed, no owner app-data store opened, no parking needed** — installed authority remains F2R1's `fb5f628` / run `37431977428` / artifact `11397938806`. **Canonical state written (§15–§18):** `P5 = PASS_COMPLETE` · Productization `ENGINEERING_COMPLETE` · `active_task = NONE` · product `MVP_CANDIDATE` · narrative **FirmwareSight Productized MVP Candidate** · `baseline_version` `0.6.0` · `G2 = PASS` with P0–P4 untouched — in `BASELINE.yaml` using only that file's existing vocabulary (`stage_status`, `closed_on`, `tests_at_close`, `product_state`, `gates_at_close`, `remote_ci`, `next_stage_after_this_one`; **no key invented for prose, no new machine enum**), and in `.ai/ACTIVE_TASK.md`, `.ai/CURRENT_STATE.md`, `.ai/HANDOFF.md`, `.ai/README.md`, `.ai/DECISIONS.md`, `README.md`, `INDEX.md` and `06_DELIVERY/06_STAGE_GATES.md`, with every stale `IN_PROGRESS` / `F3 READY_FOR_ARCHITECT_REVIEW` current claim replaced by a dated state and each round's own words preserved beside it. §19–§22 finalized the lineage with every red and interrupted run kept (`3400981` 8/10; #67 attempt 1 infra-then-attempt 2; #69 attempts 1–2 zero-step cancellations then 3; #70 and #71 attempt 1, 10 of 10), added F2R2's external row while preserving `31f10c7`'s no-run row, and created `P5_VALIDATION/P5_FINAL_CLOSURE_REPORT.md` | **its own run is external evidence and cannot be certified from inside the commit that waits for it** — `F3 remote CI = PENDING_EXTERNAL_EVIDENCE`, read after the push by exact SHA with all ten jobs enumerated individually, and §34 forbids a further commit whose only content is the number. The predecessor `a5ce7c4` is green: run `37453402452` (#71), attempt 1, 10 of 10, re-read with `gh api` on 2026-10-06 | **868 Rust / 225 UI in 8 files — both exactly as §5 requires them to stand still**, which is the check that proves this is documentation and not a product change wearing a documentation diff: `cargo test --workspace` 868 passed / 0 failed across 47 test-result lines, `pnpm test` 225 passed in 8 files, `cargo fmt --check` 0, `clippy -D warnings` 0, full gate **17 of 17** (rust 3, frontend 5, drift 8 including `drift/baseline integrity`, deny 1) with **no `SKIP`**, `--only drift` 8/8, `--only deny` 1/1, `--only core-smoke` 3/3, `--only package` 4/4, and `verify_baseline_artifacts.py` **RESULT PASS** at the final staged state (710 tracked paths, 708 entries — the two new paths are the archived F3 prompt and the new closure report, and no product path is among the 30 changed). §25's allowlist is proved from the staged diff: **zero** paths under `apps/` `crates/` `scripts/` `fixtures/` `schemas/` `golden/` `migrations/` `.github/`, and no `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml`, `rust-toolchain.toml` or `assets/design-tokens.json` byte — stronger still, every one of those **tree OIDs is identical** between `a5ce7c4` and the staged F3 tree, so `apps` `dd8cccf…`, `crates` `0c93332…`, `apps/desktop/ui/src` `8e893af3…`, `scripts` `76c1e1c…`, `.github` `174bd58…` and the token file `b818d11…` cannot have moved. `P5 = PASS_COMPLETE`, `Productization = ENGINEERING_COMPLETE`, `active_task = NONE`, product **MVP CANDIDATE**, narrative **FirmwareSight Productized MVP Candidate**. **And §37 stops the stage: V1, B1, private beta, RC, GA, commercialization, licensing, signing, notarization, an updater and new feature development are all unauthorized, and final external acceptance requires F3 remote CI** |
| the head that lands this row | **V1 activation — the Recruitment Ready pack, and deliberately nothing more.** Authorized by *FirmwareSight — V1 Own-artifact External Validation, Execution Prompt v1.0 — Architect Authorized* (delivered file SHA-256 `48768ff0…ec10a0f`, 46,207 bytes, 1,697 CRLF pairs, 1,698 logical lines), archived as `FirmwareSight_V1_Own_Artifact_External_Validation_v1.0.txt` (Git blob `d5e8d457…51774`, SHA-256 `c62314fe…90c34e`, 44,510 LF bytes — the two digests differ by exactly the 1,697 removed `CR` bytes and split into the same 1,698 lines). §3 preflight confirmed `HEAD = origin/main = 08fdfcb` with a clean tree and one worktree, so there was no remote delta to classify. §1 then froze and **preserved the cohort build**: run `37475580080`, head `08fdfcb`, artifact id `11419727517`, ZIP container 5,536,303 bytes (which §1 insists is *not* the installer size), NSIS `FirmwareSight_0.6.0_x64-setup.exe` delivered as `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, 3,888,432 bytes, SHA-256 `182506f2…63d12`, internal `SHA256SUMS.txt` verified `OK` on both entries with exit 0, payload executable `6598880d…10646`, `expired: false` at activation, and the set preserved **outside Git** because §1 forbids committing the installer. **§5 is the branch this round took:** there were no real eligible external participants and no session evidence, so the round wrote the pack instead of running a cohort — §13's sixteen files (`V1_VALIDATION/README.md`, `V1_PLAN.md`, `V1_METRICS.md`, `V1_PARTICIPANT_REGISTER.md`, five `protocol/` documents, `sessions/README.md` and `TEMPLATE.md`, three `analysis/` registers, two `deliverables/`), every register at zero, the report a skeleton, and **no fabricated session, participant, quote, timing or completion** — which is the whole point of the branch. V0's M01–M17 taxonomy, its exclusion rules and its neutral-task discipline are carried forward rather than reinvented; the three deliberate changes (the installed product instead of a prototype, thresholds fixed before any data, and C6–C8 added to V0's C1–C5) are named in `README.md`'s comparison table. §43's boundary held: `V1_VALIDATION/**`, the archived prompt, `BASELINE.yaml`, `.ai/*.md`, `README.md`, `INDEX.md`, `06_DELIVERY/06_STAGE_GATES.md` (whose V1 line was materially stale — it carried no state while every other stage did) and the two regenerated baseline artifacts, with **zero** forbidden product paths. §14 activated `active_task = V1_OWN_ARTIFACT_EXTERNAL_VALIDATION`, `V1 = IN_PROGRESS`, `research_state = RECRUITMENT_READY` while **P5 stays PASS_COMPLETE** and the product stays `MVP_CANDIDATE` at `0.6.0`; `v1_execution` is the opening record, and `product.status` / `validation.current_gate` deliberately do **not** mention V1 — at `4cc8d93` neither mentioned P5 either, because those strings carry verdicts and V1 has earned none. Release readiness, signing, notarization, updater, licence, tag and GitHub Release are untouched; `PUBLIC_DISTRIBUTION` stays `NOT_AUTHORIZED` and the research build stays unsigned and one-to-one. **V1 = IN_PROGRESS / RECRUITMENT_READY, eligible external sessions = 0, and the round stops there (§5 step 9).** | its own run is external evidence, read with `gh run view <sha> --json headSha,conclusion,jobs` | **868 Rust / 225 UI in 8 files — both required to stay exactly where F3 left them (§45)**, on the same **17-step** gate: drift **8 of 8** with `baseline integrity` inside it, deny 1 of 1, `core-smoke` 3 of 3, package 4 of 4 with no `SKIP`. A docs-only round that moved either count would be a product change wearing a documentation diff. The owner's FirmwareSight store was never opened, no installer was run on this host, and no participant data exists yet. |
| the head that lands this row | **U1 — UI productization convergence, the round that paused V1.** Authorized by 《FirmwareSight B1 — UI Productization / Design Convergence》 v1.0, **delivered inline** (no file, so `10_AUDIT/SOURCE_PROMPTS/README.md` records `File: none (delivered inline)` the way the three inline P0 precedents are recorded, and the agent's labelled transcription is archived at `U1_VALIDATION/00_authority/` — 503 lines, 16,762 B, SHA-256 `67067dce624a6dc458055dd79e21b6ed4e983c7ac665426561e72b939de52865`, which is that transcription's digest and never the prompt's). §2A preflight measured `HEAD = origin/main = f481b78`, clean tree, one worktree, and it **reported the §0 start-state difference instead of smoothing it over**: the prompt said `active_task = NONE`, which was true at F3's head and had been overtaken by V1. The owner then decided three things before any file was written — the track is `U1` and not `B1` (Private Beta stays reserved and unauthorized), V1 pauses with every state field intact and gains only `paused_for`, and the frozen ADR-0018 precedence chain outranks the mockups. §6's gap audit ran **before** any code, as delivered, and wrote down the twelve measured gaps plus the dispositions. What converged: seven shared components (`Button`, `Chip`, `Layout`, `PageHeader`, `Panel`, `RankBar`, `TopBar`) replacing five page dialects of the same grammar — 373 lines of duplicated page CSS deleted, 0 added token values, `styles/tokens.css` and `assets/design-tokens.json` untouched and proved so by `drift/design tokens` — a new **Overview** page assembled only from DTOs the shell already returns with every action a navigation rather than a performed verb, the **Release verdict moved to the top of its page** with `GateCountsDto` as a strip and one shared `gateVerdictSentence` so the same run cannot read two ways on two pages, Analyze/Compare/History/Details onto the shared header, buttons, empty state and `RankBar`, and the **F2R-01 block-table defect closed on the two pages that still carried it** (Compare and Release), with all ten table wrappers now one `ScrollArea` and two new universal layout-contract tests. The single non-CSS change was declared before it was made: `MainWindowPage` gains an additive `Overview` variant because the window title is Rust's (AGENTS.md 7) — pinned by `history_reads.rs`, binding regenerated, and no DTO, schema, migration, storage, wire, release-identity, ADR-0028 or ADR-0029 surface moved; **no new IPC command** (30 before, 30 after, read from `generate_handler!`; the 27 written here at the time was
wrong — `U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md` §2). `SplitPanel`/`DetailPanel` were drafted for the references' detail column and **deleted at close** rather than shipped unused, because no page holds a selection to fill it; the six things the round did not reach are §6 of the report, and `visual_evidence` is honestly `STRUCTURAL_CONTRACTS_ONLY` — the real-desktop before/after pass needs the owner's store parked, which was not authorized, so **no screenshot was taken and no pixel claim is made**. `V1 = IN_PROGRESS / RECRUITMENT_READY` **paused** with 0 sessions and its frozen cohort build; `P5`, G2, product state and all ten release-readiness states unchanged; U1 self-issues no stage. | its own run is external evidence, read with `gh run view <sha> --json headSha,conclusion,jobs` | **868 Rust unchanged / 236 UI in 9 files** (was 225 in 8: +9 `overview.test.tsx`, +1 `compare.test.tsx`, +1 `release.test.tsx`, none deleted; five pre-existing test files had assertions reworded for the new markup and every one is named in the report §5), `cargo fmt --check` 0, `clippy --workspace --all-targets -D warnings` 0, `tsc --noEmit` clean, `eslint .` clean, `pnpm build` through, `pnpm exec vitest run` 236/236. The authoritative `python scripts/check.py` before staging reached **13 of the 14 steps it ran** — the one red being `drift/ipc bindings unchanged`, which is the unstaged ts-rs regeneration and nothing else — and the closeout run at the fully staged state is the authoritative number: **17 of 17 steps PASS with no SKIP** (rust 3, frontend 5, drift 8 with `baseline integrity`, deny 1), with `scripts/verify_baseline_artifacts.py` `RESULT PASS` at 750 tracked paths and 748 manifest entries and every mismatch counter 0. `core-smoke` was not claimed here because every package it selects is already inside `cargo test --workspace`, and `package` was not run at all — it builds a release installer for this host, so no package figure appeared in the first round's documents. **Both groups were run in the continuation round at the same head: `--only core-smoke` 3 of 3 and `--only package` 4 of 4 with no SKIP**, recorded in `U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md` §3.

The failure is recorded rather than re-run until a green attempt appeared: `Desktop UI (windows-latest)`
lost a race in `compare.test.tsx` that predates P5 (`055b54e` closed the same shape at the pager and said
it had left the rest). Reproduced here without CI — 1 of 30 fresh runs, and 3 of 3 under injected mock
latency — and repaired by awaiting the one synchronous read, no production source touched. `P5_VALIDATION/
P5_PRODUCTIZATION_AUDIT.md` §0a carries the evidence. **The sweep that paragraph asked for has now been done,
in bounded form**, because the same file produced the class's sixth instance — lost by the *local* full gate,
not by CI, on the tree that came after `20b03e3`. §0a records the repair, three mutation proofs, 20 clean
fresh-process repetitions, the one chained IPC wave the sweep found, and the five top-level synchronous reads
that remained after it. What is still open is the honest residue: absence assertions settle by timing, and a
wave whose shape differs from the one just closed may still exist.

`37133706214` is the second red run and the more instructive of the two, because nothing about the product
was wrong. All seven gate jobs passed; the three package jobs installed `tauri-cli@2.12.1`, looked for a
binary named `tauri` — `cargo install` leaves `cargo-tauri`, run as `cargo tauri` — reported a skip, and
printed `4/4 steps passed` with exit 0. What made them red was their own `if-no-files-found: error` upload
step, i.e. the failure surfaced one step after the verification lied. The repair is the rule, not the
filename: `check.py` distinguishes `SKIP` from `PASS` in its summary and totals, and exits non-zero when a
skip happens in CI, where every tool the gate needs is installed by the job itself. Three local proofs are
in `P5_VALIDATION/P5_PACKAGING_REPORT.md` §5, including a stand-in `cargo-tauri` that measured cargo's
argument forwarding and then failed verification for building nothing.

Installing the pinned CLI on this host then turned the group from a probe into a build, and it found the
second defect: run from `apps/desktop/src-tauri`, the CLI's own `beforeBuildCommand` hook executed in a
directory with no `package.json` (`ERR_PNPM_NO_IMPORTER_MANIFEST_FOUND`), because the CLI resolves its
frontend from the **process cwd** (`tauri-cli-2.12.1/src/helpers/app_paths.rs:148,153-174`) and this
repository keeps `ui/` and `src-tauri/` as siblings. The package step now runs from `apps/desktop`, and from
there it produced this repository's first installer — `FirmwareSight_0.6.0_x64-setup.exe`, 3,811,140 bytes,
`c5c8cf23…` in `target/dist-package/SHA256SUMS.txt`, verified with `sha256sum -c`. That head then went
**10 of 10 green** on Run `37138881977` — the first run in this repository to attach a built package on all
three platforms — and reading those artifacts back found the third defect (§5d of `P5_PACKAGING_REPORT.md`):
the darwin index gave the `.app` one line naming a directory, so `sha256sum -c` answered
`FAILED open or read` on a correct build. The next run fixed it and was read back the same way: Run
`37143046338` at head `53578e9`, **10 of 10**, five darwin index lines all `OK` with exit 0, and one flipped
byte in a copied `Contents/Info.plist` making the index exit 1 and name the file. Built is now installed:
§38 A–L and §39 ran on this host against the packaged installer, with the owner's store parked, hashed and
restored byte-identically, and are transcribed in `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md`. §38 C —
first-run onboarding — has since been **built** (Commit C, with L21's title fix that same round found on the
packaged build), but not yet **operated in an installed binary**, so §64's full journey stays open. Commit D is
now **COMPLETE** and its prompt's §34 STOP is in force. That STOP is what the Architect's next prompt
answered: *FirmwareSight — P5 Commit E — Compatibility Fixtures, Support Matrix & Supportability Closure,
Execution Prompt v1.0* (archived with its digest in `10_AUDIT/SOURCE_PROMPTS/`) authorizes the fixture
cohort, the compatibility matrix and the §48 dispositions — and explicitly not the §64 full journey, not
L26 and not a general source-control linter, so the installed-binary walk and Commit F stay behind a
further return to the Architect rather than in the next free pair of hands. *(That return happened: Commit E
closed, the Architect issued Commit F, and F1–F3 all landed. See the ledger rows above and
`P5_VALIDATION/P5_EXECUTION_REPORT.md` §7 for the head-by-head record; this paragraph is kept as what was true
when it was written, dated rather than deleted.)*

Commit E is since **FINAL PASS / COMPLETE**, and the Architect did return: *FirmwareSight — P5 Commit F —
Final Productization Closure, Execution Prompt v1.0 — Architect Reviewed* (archived with its measured digest
in `10_AUDIT/SOURCE_PROMPTS/`) authorizes exactly two things this stage had left undone — L26 as repository
tooling semantics under a new `ADR-0029`, and the §64 installed journey against the CI-built F1 artifact —
in three commits, F1 (product/tooling), F2 (installed evidence) and F3 (the only head allowed to write
`P5 = PASS_COMPLETE`). It forbids §64 inside F1, and it forbids a fourth commit merely to record a run. What
decides the order of the last two is the owner's own answer, given on 2026-10-05: **land F1 first, then
confirm**. So F1 is this head, no installer has been run, and no file of the owner's has moved; the §64 walk
stays behind that confirmation, not behind a free pair of hands. *(The owner then released F2, and F2 ran the
whole §64 journey on F1's own CI-built artifact with the owner's store parked, hashed and restored twice; the
Architect inserted F2R between F2 and F3 for the three findings that walk produced, and F3 closed the stage on
2026-10-06. This paragraph is F1's own state of play, kept as written and dated here rather than deleted.)*

## What the owner decided at the checkpoint

The cadence the owner set was: audit first, then a checkpoint before any product code. That checkpoint ran
on 2026-10-03 and settled four questions.

| Question | Answer |
| --- | --- |
| Which version identity wins | **Unify the artifacts on `0.6.0`**, with the workspace version as the single source; `baseline_version` stays `0.6.0` and nothing is tagged or released |
| Migration `0005` for L6/L7 | **Write `P5_MIGRATION_DECISION.md` first, then add it** — additive columns for the two numeric `Unknown` reasons plus the `builds.created_at` index |
| How far the package matrix goes | **Windows with a real install here, macOS and Ubuntu `CI_BUILD_ONLY`** — those two rows may never be written as `SUPPORTED` |
| The store named `firmwaresight-p0.sqlite` | **Keep it.** No rename, no data move; the path becomes visible in Diagnostics instead |

Two questions are not this round's: §32 sends L22 (release identity versus line endings) to the Architect
as `P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md` followed by a STOP — **written, and it changes no
behaviour**: it documents that the notes digest is the file's bytes (`evidence.rs:291` →
`fingerprint.rs:42`), lays out the normalization and Git-blob options with their costs, names the one
residual risk the G2 row did not state (`require_clean_git = false` makes the id move with nothing on screen
saying why), and stops. The premise is now tested rather than argued
(`crates/firmwaresight-project/tests/bundle_builder.rs`,
`a_notes_file_that_differs_only_in_line_endings_is_a_different_release`, with the normalization mutation
shown to redden it). And §54 forbids this agent from choosing a
licence, so `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` stands in every P5 document.

## What P5 must not write

*(This is the discipline list the P5 prompt §5/§11/§12/§72 wrote on 2026-10-03, and it is what the round
obeyed for eleven heads. One item of it has since been earned rather than lifted: F3 §5 permitted
`P5 = PASS_COMPLETE` only after §6's re-audit found no required engineering item `BLOCKED`, and that re-audit
is `P5_VALIDATION/P5_EXIT_CHECKLIST.md` §7. Everything else below still binds after closure, and none of it
was touched by F3.)*

No `B1 READY`, `BETA`, `RC` or `GA` — closure did not produce any of them. No tag, GitHub Release or published
installer (§72). No signing or notarization executed — those statuses stay `READY_NOT_EXECUTED` (§11) — and
updates stay `UPDATE_READY_MANUAL` (§12): no updater, no endpoint, no certificate, no committed private key.
No "security clean"; the permitted sentence is "dependency policy passes with documented accepted risks". No
new network capability, telemetry, analytics SDK, generic shell or filesystem permission, and native dialogs
stay Rust-side. No licence decision: `PUBLIC OPEN-SOURCE REDISTRIBUTION CLAIM = BLOCKED BY OWNER LICENSE
DECISION`, and that sentence is §14's, not a defect to repair. And no next track: V1, G3 and B1 each need
their own architect prompt.

## What closed before P5

`G2_ENGINEERING_CLOSURE_AUDIT`, authorized by *FirmwareSight — G2 Product MVP Engineering Closure Audit,
Execution Prompt v1.0 — Architect Reviewed* (file, SHA-256
`3c6ab83e11ce6a4c91a609dd0a6bc318e700dc0cbbccd3d4493eaf65c3e2bac9`, archived in
`10_AUDIT/SOURCE_PROMPTS/`) and *G2 Storage Path Semantics Clarification Addendum v1.0* (inline).

| Document | Holds |
| --- | --- |
| `G2_VALIDATION/G2_EXIT_CHECKLIST.md` | the §34 checklist, persistence rows as the addendum corrected them, and the verdict |
| `G2_VALIDATION/G2_EVIDENCE_MATRIX.md` | every requirement classified and cited; the 17-point path boundary; the findings; the PRD metrics |
| `G2_VALIDATION/G2_END_TO_END_SMOKE_REPORT.md` | the CLI chain, the shipping window S1–S43, parity, failure and recovery, fail-closed inputs |
| `G2_VALIDATION/G2_ENGINEERING_CLOSURE_REPORT.md` | what was done and decided, including the remote closure |
| `G2_VALIDATION/G2_KNOWN_LIMITATIONS.md` | the one canonical list, 25 rows, none blocking |

Measured on the audited tree: one `cargo test --workspace` **769 / 0 / 0**; UI **155 in 6 files**;
`check.py` **15/15**, and **17/17** in a clean detached worktree; CLI and desktop byte-identical on Diff
JSON, Diff HTML and the whole bundle; independent reader 64/64, relocated 59/59. Remote: the product tree
`e35cfe7` on Run `36906482900` and the evidence head `f75cbc5` on Run `36948719972`, each 7 of 7 on the
first attempt. Findings: G2-F1 closed (test-only), G2-F2 adjudicated as expected local-only storage, G2-F3
recorded (tooling).

`NOT MEASURED`: peak RSS and a 500 MB working set in the window — not G2 blockers under the prompt's §27,
and not claimed. `NOT RUN`: fuzzing, a macOS or Linux window, a second Windows host, any DPI other than
100%.

**The closing commit's own CI run is not written into it.** Prompt §43 allows exactly one successor and
makes its run external evidence, read with `gh run list` and reported, not chased into another commit.

## What G2 PASS does not mean

Not G3 productization, private beta, release candidate or GA; not production-ready, signed or installable;
not real-user validated (V0 stays `0 / 8`) or commercially validated; not security-clean (two accepted
RustSec advisories). And not a licensed open-source release: `license = "Proprietary"` stands in the root
`Cargo.toml`, there is no root `LICENSE`, and `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`
remains — `AGENTS.md` 9 puts that decision in front of the owner. Baseline stays `0.6.0`; no tag, GitHub
Release, installer or `v0.7.0` was created.

## What closed after G2, and what that changes here

Those two rounds did not move the pointer: `active_task` stayed `NONE`, G2 stayed `PASS`, and the product
stayed an MVP CANDIDATE. What moved was the tree, so the numbers in the section above are the history of a
different commit rather than the present: the remediation round left **770 Rust / 159 UI**, P5's early
commits moved it to **775 Rust / 160 UI / `check.py` 16 steps** (migration `0005`'s five storage tests, the
Details reason test, and the `version identity` drift step), and **Commit C moved it again to 812 Rust /
200 UI in 8 files** on those same 16 steps — 19 storage History reads, 11 desktop boundary tests, 40 UI
tests over the two new screens. §32's identity test and L23's contract test made that **813 / 201**, and
**Commit D made that 854 Rust / 210 UI in 8 files** — storage at 131 (5 unit, 126 integration,
of which 13 are the new `integrity_and_backup.rs`), desktop at 222 (96 unit including the 7 `startup.rs`
mappings, 126 integration including 8 in `tests/diagnostics.rs`), and 9 new UI tests over the section that
makes 210 — with the local gate at **16 of 16** and the package group at **4 of 4**.
**Commit E moved that to 868 Rust / 217 UI in 8 files** — the 14 new Rust tests all in
`crates/firmwaresight-artifact/tests/p5_compat_fixtures.rs` and the 7 new UI tests in `compare.test.tsx` (6)
and `details.test.tsx` (1) — on a gate that stayed at **16 of 16** because that round added no step, with
`core-smoke` 3/3, drift 7/7, deny 1/1 and package 4/4 holding and no `SKIP`.
**Commit F1's total was 868 Rust / 219 UI in 8 files** — no Rust test moved (F1 changes repository
tooling, not the product), and `details.test.tsx` gained the two L15 inspector cases — while the gate moved
**16 → 17 steps** because the drift group gained `baseline integrity`, so drift is **8 of 8**, `core-smoke`
3/3, deny 1/1 and package 4/4, all with no `SKIP`.
**The present total, as Commit F3 closed the stage on 2026-10-06, is 868 Rust / 225 UI in 8 files** — F2 held
both numbers as its own docs-only proof, **F2R1 took UI 219 → 225** with the three installed-UI contracts and
their guards (Rust unchanged), and F2R2 and F3 each held 868 / 225 still for the same reason F2 did. The gate
has been **17 steps** since F1: drift **8 of 8** with `baseline integrity` inside it, deny 1 of 1,
`core-smoke` 3 of 3, package 4 of 4, no `SKIP`, and the cold detached worktree at 19. The evidence
directory the post-G2 real-desktop remediation round added is `POST_G2_E2E_REMEDIATION/`.

On 2026-10-02 a real-desktop acceptance round drove the shipping binary through 283 black-box cases and
closed `PASS_WITH_FINDINGS` — three product defects, no S0, no S1 — and a follow-on round fixed exactly
those three (`e816dcb`, `971015f`), each with a regression written first, a mutation proof after, and a
focused real-desktop re-check against a rebuilt binary. Neither round widened anything: no P5 surface, no
parser performance work, no schema, migration or dependency change, and no licence decision.

The rule above bound, and both rounds obeyed it: with no active task, no agent may create business
functionality or pick the next track. P5 has since been picked the only legitimate way — an architect
prompt of its own, archived with its hash, which is what opened this task. V1 still has no prompt and stays
unauthorized. *(Dated correction: that sentence was true of the tree P5's F3 closure left, and it stopped
being true on 2026-10-06, when the architect issued the V1 prompt as a file and this pointer moved to
`V1_OWN_ARTIFACT_EXTERNAL_VALIDATION`. It is kept as written because it records what was knowable at that
head; the live state is the block at the top of this file and `v1_execution` in `BASELINE.yaml`.)* The carried
forward list — large-file latency, peak RSS, 125/150 % DPI, mouse-wheel, `update_goldens`, the E2E harness —
was carried into P5 as a disposition to answer (§60), not as a licence to widen scope, and P5 answered what it
could: the ones real-user validation owns are now L11's and V1's, which is exactly the hand-off
`V1_VALIDATION/deliverables/V1_GATE_RECOMMENDATION.md` records.
