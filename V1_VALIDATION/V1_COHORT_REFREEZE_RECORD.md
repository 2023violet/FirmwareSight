---
title: "V1 cohort re-freeze: authority record and execution report"
doc_id: "FS-V1-RF-001"
product: "FirmwareSight"
version: "0.6.0"
unit: "V1_COHORT_REFREEZE"
status: "BASELINE"
authority: "Owner's inline Execution Authorization v1.0 of 2026-10-08 + AGENTS.md 1 / 8 / 9 / 10 + V1 prompt v1.0 §1 §32 §46 + the U1 architect final verdict"
opened_on: "2026-10-08"
---

# FS-V1-RF-001 — moving the cohort build, and nothing else

V1 freezes **one** Windows build for the whole formal cohort, because a head is not a build and a metric that
mixes two builds is not a metric. The build that was frozen — P5 Commit F3's installer — is no longer the build the
project intends to put in front of people: U1 changed the interface the participant reads, and the Architect's
visual acceptance was granted on the **candidate's** bytes, not F3's. Leaving the cohort on F3 would have meant
studying a product that had already been superseded, and answering L11 about screens nobody ships.

This round moves that one pointer, and provably moves nothing else. It changes no product byte, no test, no token,
no schema, no migration and no Gate rule; it recruits nobody, installs nothing, measures nothing, and leaves every
session count at zero. The whole deliverable is an identity decision with the arithmetic behind it.

## 1. The authorization, and what it refused to be

The instruction was **delivered inline as chat message text**, not as a file, so there is no owner-side byte stream
to hash. `10_AUDIT/SOURCE_PROMPTS/README.md` registers it with no file and no invented digest — the entry's label is
**"File: none (delivered inline)"**, the same treatment the earlier inline authorizations in that register
received. A labelled agent transcription is archived at
`V1_VALIDATION/00_authority/SOURCE_PROMPT_V1_REFREEZE_transcription.md` — 5,366 bytes, 142 lines, 0 CR, SHA-256
`88ef07bd2ec830b51e13f5a1335a5aaa6ed561d417310604fdf57784aca636a2`, stored blob `b5c68606141b1ea01baf37fe44ff24866bdd85b6`
— and **that digest is the transcription's, never the authorization's**.

The authorization grants `V1_COHORT_REFREEZE` and nothing else. It explicitly does not start participant sessions,
does not authorize public redistribution, and does not open B1, RC or GA. Its §四 boundary list — no product source,
no dependency, no change to the M1–M6 metrics, denominators or thresholds, no deletion or rewriting of historical
research evidence, no invented participant, recording or result, no default-on recruitment, session, install or
external distribution, no signing, updater, capability or licence change — is reproduced here because a later reader
should be able to check the round against its own leash.

## 2. Start authority, measured before any write

`HEAD` = `origin/main` = `6e3ee22307e83b05ea3a318f9c9a3853f13ba552` (the U1 verdict record), `git status --porcelain`
empty, one branch, one worktree, no install directory, and that head's own run `37874177649` (#87) already read back
at 10 of 10. **No uncommitted change was found, so there was nothing to preserve, refuse, stash, reset or delete** —
the §一 instruction that forbids cleaning a dirty tree was satisfied by the tree being clean, and that is stated as
a measurement rather than assumed. Full capture: `00_authority/PRE_ROUND_MACHINE_STATE.json`.

Every raw artifact this record cites lives in one external evidence root, named by its creation instant in UTC:
`%TEMP%\FirmwareSight-V1-Refreeze-20261009T035521Z\`, i.e.
`C:\Users\16429\AppData\Local\Temp\FirmwareSight-V1-Refreeze-20261009T035521Z\`, holding `00_authority` (the
pre-round state, both builds' verification, the F3→candidate delta), `01_old_f3_build` (the superseded container
and its extracted payload), `02_candidate_build` (the frozen container, extracted payload and its verification
JSON), `03_ci` (both cohort runs read job by job), `04_record` (this document and the transcription) and
`05_validation` (the local gate run and the post-push CI read-back). Paths in this repository are written as
`00_authority/…` relative to that root. Repository dates are the owner's local date, 2026-10-08, while the root's
own name is the UTC instant it was created — the two differ by hours, not by days of work. Nothing in it was
committed, and nothing in it was deleted.

## 3. The two builds, side by side, from the bytes

Both artifacts were downloaded again for this record and hashed from the downloaded bytes. Nothing was installed,
launched, or opened.

| | The build now superseded (F3) | The build this round freezes (U1 candidate) |
|---|---|---|
| Product commit | `08fdfcb710f78f8084bfcf614dc508c8b6e7e25b` | `41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177` |
| CI run | `37475580080` (#72, push, attempt 1, 10 of 10) | `37828673549` (#85, push, attempt 1, 10 of 10) |
| Artifact id | `11419727517` | `11573661113` |
| Artifact name | `FirmwareSight-0.6.0-windows-x86_64` | `FirmwareSight-0.6.0-windows-x86_64` |
| Container | 5,536,303 B / `40cb5c9933e84583ae0895923249c873e129fd4f3c5a51d2b549e5e3ee37850e` | 5,544,133 B / `330e83af75db705b2acae6ff559baf0e863d7766947bc21240488861cd4f3fb4` |
| NSIS installer | `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, 3,888,432 B / `182506f213383cfe00865f199fcec4fb17079535e8ea370f954fc15097263d12` | same file name, 3,896,257 B / `9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93` |
| Built (user-facing) name | `FirmwareSight_0.6.0_x64-setup.exe` | `FirmwareSight_0.6.0_x64-setup.exe` |
| CLI companion zip | 1,668,153 B / `76bf7d0c1c4acb0f64a5fdda75b036ec658f272086b2283ce0ba56d3543f7ddd` | 1,668,149 B / `924f3a7130b3be7bba95dac9e14331aacfb0bdbb761d839af96d51d28c1d8373` |
| Payload desktop exe | `6598880dd8dde479d9326e678d0c22eddfc859a6c8cbc98ce7049b8f3c310646` | `5026c47b82f34229d8d5198b16288b35d37f3048935d58c46b2eaac4390ddaae` |
| Internal `SHA256SUMS.txt` | 2 entries, both `OK` | 2 entries, both `OK` |
| Version resource | `0.6.0` | `0.6.0` |
| Toolchain | rustc 1.98.1 (48a229cea 2026-09-01) | rustc 1.98.1 (48a229cea 2026-09-01) |
| `toolchain.git_commit` | equals the F3 head | equals the candidate head |
| Signed | **no** — unsigned, disclosed to participants before install | **no** — same, and the disclosure is unchanged |
| GitHub expiry | `2026-10-20T14:21:52Z` | `2026-10-22T19:18:25Z` |

Two honest notes about the method, one of which corrects an earlier round's wording **and this round's own first draft
of it**. **The container size is not the installer size** — 5,536,303 against 3,888,432, and conflating them is a trap
this project has recorded before. And the GitHub-side digest authority does exist, under a different field name than
this repository has been writing: the artifact API publishes **`digest`, whose value is `sha256:<hex>`**, and there is
**no field named `sha256_digest`** on either the run-artifacts list or the single-artifact endpoint. Querying
`sha256_digest` returns `null` because the key is *absent*, not because GitHub published a null digest — and that is
what U1P-R1, the U1P-R3 verdict record and the first draft of this paragraph reported. Re-measured here with the key's
presence checked rather than assumed: GitHub's published `digest` for `11573661113` is
`sha256:330e83af75db705b2acae6ff559baf0e863d7766947bc21240488861cd4f3fb4` and for the superseded `11419727517` is
`sha256:40cb5c9933e84583ae0895923249c873e129fd4f3c5a51d2b549e5e3ee37850e` — **each equal to the container digest this
round hashed from the downloaded bytes**, so the freeze is corroborated by an authority outside the download, which is
what the owner's §五 first acceptance criterion asks for. Two further authorities stay in place: the artifact's
internal `SHA256SUMS.txt` verified entry by entry, and `artifact-metadata.json`'s `toolchain.git_commit` matching the
head the run reports. The payload digest *locates* the build; it is not the digest of the file a participant launches,
because the bundler rewrites a 27-byte bundle-type token inside the copy it packs. Full records:
`01_old_f3_build/BUILD_VERIFICATION.json`, `02_candidate_build/BUILD_VERIFICATION.json`,
`00_authority/BOTH_BUILDS_VERIFIED.txt`, `00_authority/GITHUB_ARTIFACT_DIGEST_FIELD.json`.

**What this round deliberately refused.** HEAD's own CI built a Windows artifact too — `11592690212`, run
`37874177649` (#87), container 5,536,796 B, enumerated from the API and never fetched. That
is the documentation head, and §2 forbids treating a docs-only head's package as the frozen research build — the
same rule V1's activation recorded when its own commit's packages proved only that the product had not moved. The
cohort is frozen to `41bb6a36`'s artifact and to no other.

## 4. What actually differs between the two builds

`08fdfcb..41bb6a36` is 14 commits, 96 paths, of which **44 are product paths**. Grouped by subtree, by Git tree OID
rather than by summary text:

| Subtree | F3 → candidate |
|---|---|
| `crates/` (all Core, storage, migrations inside `crates/firmwaresight-storage/migrations`) | **IDENTICAL** `0c933323…` |
| `assets/` (design tokens, mockups) | **IDENTICAL** `da1f6d16…`; `design-tokens.json` hashes `94336906…14d4` on both sides |
| `schemas/` | **IDENTICAL** `a8e7e542…` |
| `fixtures/` | **IDENTICAL** `0fcc2114…` |
| `golden/` | **IDENTICAL** `f4f89d0b…` |
| `.github/` | **IDENTICAL** `174bd58f…` |
| `scripts/` | **IDENTICAL** `76c1e1cb…` |
| `09_ADR/`, `templates/` | **IDENTICAL** |
| `apps/` | **DIFFERS** — 42 paths under `apps/desktop/ui/src`, plus `apps/desktop/src-tauri/src/ipc.rs` and `apps/desktop/src-tauri/tests/history_reads.rs` |

The Rust delta is one additive enum variant and two window-title strings: `MainWindowPage::Overview` appears, and
`Release` / `History` now title as `Release Gate` / `Bundle & History`, matching what each page calls itself on
screen. The variant **names** — which are the wire form of `set_window_title` — are unchanged, so no IPC contract
moved; `generate_handler!` registers the same 30 commands at both heads, added `[]`, removed `[]`. The companion
test file changes only its title assertions and its "five pages" → "six pages" wording.

That is the whole of the difference, and it is why the re-freeze is safe to make and necessary to make:

- **The research instrument is untouched.** No parser, diff, Gate rule, evidence class, storage contract, schema,
  migration, golden, fixture or dependency moved, so M1–M6's operations, denominators and thresholds mean exactly
  what V1's metrics contract says they mean. Both runs report the same **868 Rust tests across 47 result lines**.
- **What the participant reads did move.** Overview is a new page, the shell rail is fixed while the main region
  scrolls, the Release verdict sits above its detail, Compare keeps its result across navigation inside one process,
  Analyze stops borrowing the previous file's words during a failed attempt, and the 1024 band no longer paints a
  false fourth cell. The UI suite grew 225 in 8 files → **291 in 9 files** across those rounds.
- **Therefore the acceptance and the build must be the same bytes.** The Architect's `PASS_COMPLETE /
  VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS` was issued over captures of `41bb6a36`, and §32's own rule is what makes
  the old freeze the wrong instrument: *"Formal cohort uses one frozen artifact"* and *"Any product code change
  creates a new cohort version and NEW CI artifact."* U1 changed product code, so continuing to study F3's bytes
  would have measured a version the acceptance does not describe and the shipped product no longer is.

Full arithmetic: `00_authority/PRODUCT_DELTA_F3_TO_CANDIDATE.txt` (+ `.json`).

## 5. The decision, and the authority chain behind it

The chain is short and every link is a document, not an inference:

1. **P5 Commit F3** closed productization and froze the V1 cohort to its own artifact (`11419727517`), because at
   that moment it was the newest reviewed product build.
2. **The owner** opened U1, a UI productization track, which paused V1 without changing any of its fields.
3. **U1P-R3** built the last UI change and installed exactly its own CI bytes for acceptance; its evidence successor
   published the review pack.
4. **The Architect's final visual acceptance record** of 2026-10-08 closed U1 and *recommended, without executing*,
   a separate re-freeze from the F3 artifact to `11573661113`, before participant #1, conditional on independent byte
   re-verification and on preserving F3 and all historical research records.
5. **The owner's inline Execution Authorization v1.0** of the same date granted exactly `V1_COHORT_REFREEZE` and
   named the old and new identifiers to be checked.

A recommendation is not an authority and an agent reading it is not an approval: step 5 is what made this round
legitimate. **V1 remains `IN_PROGRESS` / `RECRUITMENT_READY` and no participant was contacted.**

## 6. What stays exactly as it was

| Thing | State | Why it did not move |
|---|---|---|
| Eligible external sessions | **0** | §四 forbids inventing one; recruitment is human work (§47/§48) |
| M1–M6 | **`NOT_MEASURED`, no denominator exists** | §四 forbids touching the metrics, definitions or thresholds; nothing was measured |
| F3 build record | preserved verbatim, marked superseded with a date | §三 requires the old version's immutable history; a dated supersession note is not a rewrite |
| U1's acceptance tally | 25 items, 23 PASS / 1 FAIL / 1 NOT_VERIFIED / 0 NOT_CAPTURED, 2 `MISMATCH_PROVED` | the verdict's guard: *"DO NOT rewrite this tally to 25/25 or to 100%."* |
| U1 deviations A–D | still open, still enumerated | they are prioritised from participant evidence, not closed by a re-freeze |
| U1 status | `PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS`, visual scope only | a build pointer moving cannot re-status another track's verdict |
| P5 / G2 / product | `PASS_COMPLETE` / `PASS` / `MVP_CANDIDATE` at `0.6.0` | unchanged, and this round could not change them |
| B1, RC, GA, signing, notarization, updater, licence | `NOT_AUTHORIZED` / `READY_NOT_EXECUTED` / `PENDING OWNER CONFIRMATION` | §四 and the standing stage interlock |
| L11 | still `CARRIED_FORWARD` | it is the row V1 exists to answer, and it is answered by people, not by hashes |
| Historical evidence under `V1_VALIDATION/` | untouched except the build-identity lines this round was told to reconcile | §四 forbids deleting or rewriting research evidence |

## 7. The protocol effect, in one sentence each

- **One frozen build per cohort** (`V1_PLAN.md` §7 / prompt §32): now `11573661113`, and every session must record
  the installer SHA the participant actually ran.
- **No unauthorized substitution**: forbidden by name are F2R1's `11397938806`, the superseded F3 `11419727517`,
  HEAD's docs-head artifact `11592690212` (run `37874177649` #87, container 5,536,796 B — enumerated from the API and
  not fetched, because it packages a documentation commit and is *not* the cohort build), any local rebuild,
  `cargo run`, the Vite dev server, and any unreviewed product head.
- **Protocol documents now name the frozen bytes** so a moderator cannot install the wrong thing. Eleven files in
  this pack were reconciled, each keeping the old identity beside it as superseded rather than losing it:
  `README.md`, `V1_PLAN.md`, `V1_PARTICIPANT_REGISTER.md`, `protocol/TASK_SCRIPT.md`, `protocol/MODERATOR_GUIDE.md`,
  `sessions/README.md`, `sessions/TEMPLATE.md`, `analysis/FINDINGS.md`, `analysis/METRICS.md`,
  `deliverables/V1_GATE_RECOMMENDATION.md` and `deliverables/V1_VALIDATION_REPORT.md`. Two are new: this record and
  the authorization transcription in `00_authority/`. `V1_METRICS.md` — the M1–M6 contract itself — was **not**
  touched, because §四 puts it out of bounds and nothing measured here belongs in it.
- **Consent and disclosure unchanged**: the build is still unsigned, participants are still told so before install,
  and install friction is still evidence rather than something to remove.

**Same-day correction, and the commit that carries it.** After `f633b28` was pushed, a script raised
`KeyError: 'sha256_digest'` — a bug in this round's own tooling that turned out to be a finding, because the same
missing key had been read as a published null by U1P-R1, by the U1P-R3 verdict record and by this record's first draft
(§3). The correction was propagated by *appending dated notes where earlier rounds wrote their own words* and rewriting
only this round's own assertions: `BASELINE.yaml` gained `github_digest_field_erratum_2026_10_08` beside U1P-R1's
`windows_artifact`, `github_digest_field_name_note_2026_10_08` beside U1P-R2's, an erratum sentence inside the verdict
block's `artifact_re_verified_before_writing_its_bytes_anywhere`, and a corrected `github_digest_field`;
`U1P_R1_OVERVIEW_SUBJECT_CORRECTIVE_REPORT.md` §8 and `U1P_R3_ARCHITECT_FINAL_VERDICT_RECORD.md` §3 each gained a
bracketed dated note; `.ai/ACTIVE_TASK.md`, `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md` and `.ai/HANDOFF.md` were
reconciled the same way. No verdict, tally, waiver, deviation or disposition changed: the correction *adds* an
authority to a byte identity that was already accepted, and removes one understatement. That is the whole content of the
docs-only successor commit this paragraph is written for.

## 8. The retention hazard this round surfaced, and did not paper over

GitHub artifacts expire, and the expiry dates are now the binding constraint on the whole cohort:

- F3 artifact `11419727517` expires **2026-10-20T14:21:52Z**.
- The frozen candidate `11573661113` expires **2026-10-22T19:18:25Z**.

V1 §1 forbids committing the installer into Git, so the only copies live in machine evidence roots outside the
repository. **If neither copy is deliberately kept, the frozen research build becomes unrecoverable fourteen days from
now, and a cohort cannot run against a build nobody can hand out.** This round downloaded both and hashed them; it
does not claim that a long-term private archive exists. Closing that gap is an owner action, listed in §10, and it is
the single most time-sensitive item in this record.

## 9. Evidence classification

**Observed**: the downloaded byte counts and SHA-256 digests of both containers and both installers; the internal
`SHA256SUMS.txt` results; `artifact-metadata.json` version, toolchain and `git_commit`; the API's `expired` and
expiry fields; the API's `digest` for both cohort artifacts, and the absence of any `sha256_digest` key on either
endpoint, checked by key presence rather than by a rendered default; the ten-job outcomes and the per-job test counts
of both runs; the subtree tree-OIDs; the
`generate_handler!` command list at both heads; the clean pre-round machine state. **Derived**: the tables above,
computed from those reads. **Declared**: the owner's authorization and its boundary list; the U1 verdict's
recommendation; the metrics contract's M1–M6 definitions — quoted, not re-derived. **Unknown**: whether the candidate
build behaves identically on a participant's machine — that is precisely what the cohort is for, and no argument in
this document substitutes for it.

## 10. Outstanding human actions, in order

1. **Preserve the frozen installer bytes outside Git before 2026-10-22** (and the F3 bytes before 2026-10-20 if
   history must stay readable), in private storage with the digest recorded. Nothing in §四 authorizes this round to
   upload them anywhere.
2. **Decide the transfer channel** for controlled private delivery of an unsigned installer, and the consent text
   that discloses the missing signature (§47/§48). Owner and Architect decision, not an agent's.
3. **Recruit real external firmware/embedded engineers** who bring their **own** artifacts. An agent cannot become a
   participant, and a synthetic one is the specific failure this track exists to forbid.
4. **Moderate real sessions**, record only what happened, and only then write anything under
   `V1_VALIDATION/sessions/`.

## 11. State after this round

```
V1                        = IN_PROGRESS / RECRUITMENT_READY
COHORT_BUILD              = REFROZEN_TO_U1_CANDIDATE
FROZEN_ARTIFACT           = 11573661113  (run 37828673549, head 41bb6a36)
NSIS SHA-256              = 9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93
PREVIOUS_COHORT_BUILD     = 11419727517 (run 37475580080, head 08fdfcb), preserved and marked superseded
ELIGIBLE_EXTERNAL_SESSIONS= 0
M1..M6                    = NOT_MEASURED
U1                        = PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS (visual scope; tally guard intact)
P5                        = PASS_COMPLETE
PRODUCT                   = 0.6.0 MVP_CANDIDATE
B1 / RC / GA              = NOT_AUTHORIZED
LICENCE                   = PENDING OWNER CONFIRMATION
ACTIVE_TASK               = NONE
```

No V1 pass is claimed here, and no participant was started. The next move is the owner's, and §10 lists it.
