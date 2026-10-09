---
title: "FirmwareSight V1 Validation Workspace"
doc_id: "FS-V1-ROOT"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-08"
---

# V1 Own-artifact External Validation

V1 asks one question that no earlier round could answer: **can a firmware engineer who has never seen
FirmwareSight, using their own real firmware build, learn something true from it and choose to come
back?** V0 could not ask it, because V0 ran against a clickable prototype and never reached a real
participant either.

- Canonical unit: `V1_OWN_ARTIFACT_EXTERNAL_VALIDATION`
- Stage: **V1** · state: **`IN_PROGRESS`** · research state: **`RECRUITMENT_READY`**
- Eligible external sessions at the time this pack was written: **0**
- Product under test: **FirmwareSight 0.6.0**, `MVP_CANDIDATE`, baseline `0.6.0`
- Authorized by *FirmwareSight — V1 Own-artifact External Validation, Execution Prompt v1.0 — Architect
  Authorized*, delivered as a file: SHA-256 `48768ff025e3d6351b7b1d8d593ea8bfb18a7930a0a21a101af3093bbec10a0f`,
  46,207 bytes, 1,697 CRLF pairs, no lone `CR`, no final newline, 1,698 logical lines. Archived at
  `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_V1_Own_Artifact_External_Validation_v1.0.txt`; the stored copy is the
  LF form (Git blob `d5e8d4575581bd656b568eed97090e7130a51774`, SHA-256
  `c62314fee5032ca5ffdbcfe95f51cf3bdb783e7d55a42a165ea526686690c34e`, 44,510 bytes). The two digests differ by
  exactly the 1,697 removed `CR` bytes and split into 1,698 identical lines on each side, so the archived text
  is the delivered text — the same transport-safe rule P5 Commit F3 followed.

## What V1 is not

V1 is **not** product development, Beta, Private Beta, RC, GA, pricing, public release, licensing,
signing, notarization or feature expansion (§ Mission). The product code is frozen for the length of a
formal cohort (§32). Nothing in this directory authorizes a next stage: even a clean V1 does not open B1
(§40), and the P5 closure that precedes it authorized nothing either.

## The build under test is frozen, and it is a specific set of bytes

**REFROZEN 2026-10-08** under the owner's inline *V1 Cohort Re-freeze — Execution Authorization v1.0*. The cohort
build is the U1-accepted product build, the exact bytes the Architect's visual acceptance was granted over. Record:
`V1_COHORT_REFREEZE_RECORD.md`. The previous frozen build is preserved below and is **not** the cohort build any more.

| Field | Value |
|---|---|
| CI run | `37828673549` (U1P-R3's product run, #85, attempt 1, 10 of 10 jobs, read job by job 2026-10-08) |
| Head | `41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177` |
| Artifact name | `FirmwareSight-0.6.0-windows-x86_64` |
| GitHub artifact id | `11573661113` |
| Artifact ZIP container | 5,544,133 bytes, SHA-256 `330e83af75db705b2acae6ff559baf0e863d7766947bc21240488861cd4f3fb4` — **not** the installer size |
| NSIS installer as delivered | `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, built name `FirmwareSight_0.6.0_x64-setup.exe`, 3,896,257 bytes, SHA-256 `9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93` |
| CLI companion | `FirmwareSight-0.6.0-windows-x86_64-cli-fwsight.zip`, 1,668,149 bytes, SHA-256 `924f3a7130b3be7bba95dac9e14331aacfb0bdbb761d839af96d51d28c1d8373` |
| Internal index | `SHA256SUMS.txt` — re-downloaded and verified entry by entry on 2026-10-08: both entries `OK`, CRC clean |
| Payload executable identity | `target/release/firmwaresight-desktop.exe`, `payload_sha256` `5026c47b82f34229d8d5198b16288b35d37f3048935d58c46b2eaac4390ddaae`, `installer_version_resource` `0.6.0` |
| Toolchain | rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), tauri-cli 2.12.1, node v24.21.0, pnpm 12.7.0, runner `win25-vs2026` X64 — identical to the previous build, as are `Cargo.lock` (`ee99d5bb…`) and `pnpm-lock.yaml` (`7aac74e3…`) digests in the metadata, so no dependency moved |
| Signing | **unsigned** — no certificate, no signature, no notarization; the participant disclosure is unchanged |
| Updater | not enabled (`bundle.createUpdaterArtifacts` is false) |
| Expiry | `expired: false`, measured 2026-10-08; GitHub expiry **2026-10-22T19:18:25Z** — the bytes must be privately archived before then |
| GitHub's own published checksum | the artifact API's **`digest`** field: `sha256:330e83af75db705b2acae6ff559baf0e863d7766947bc21240488861cd4f3fb4` — **equal to the container hash measured above**, so GitHub is a second authority outside the download. There is no field named `sha256_digest` on these endpoints; where earlier documents in this repository wrote "`sha256_digest: null" they were reading a *missing key* through `--jq`, and this row is the dated correction of that wording (evidence: re-freeze evidence root `00_authority/GITHUB_ARTIFACT_DIGEST_FIELD.json`) |

### The first frozen build, recorded and superseded on 2026-10-08

These are the bytes V1 froze at activation (`BASELINE.yaml` `v1_execution.cohort_build_frozen`, and the external
evidence root's `01_artifact/ARTIFACT_VERIFICATION.txt`). They are kept here as history; **a session must not run
them.** Re-downloaded and re-hashed on 2026-10-08, all ten checks still true (dates in this pack are the owner's
local date; the evidence root's own name is the UTC instant it was created):

| Field | Value |
|---|---|
| CI run / head | `37475580080` (#72, attempt 1, 10 of 10) / `08fdfcb710f78f8084bfcf614dc508c8b6e7e25b` (P5 Commit F3) |
| GitHub artifact id | `11419727517` |
| Artifact ZIP container | 5,536,303 bytes, SHA-256 `40cb5c9933e84583ae0895923249c873e129fd4f3c5a51d2b549e5e3ee37850e` |
| NSIS installer | 3,888,432 bytes, SHA-256 `182506f213383cfe00865f199fcec4fb17079535e8ea370f954fc15097263d12` |
| CLI companion | 1,668,153 bytes, SHA-256 `76bf7d0c1c4acb0f64a5fdda75b036ec658f272086b2283ce0ba56d3543f7ddd` |
| Payload executable | `6598880dd8dde479d9326e678d0c22eddfc859a6c8cbc98ce7049b8f3c310646` |
| GitHub expiry | **2026-10-20T14:21:52Z** |

GitHub's published `digest` for `11419727517` is `sha256:40cb5c9933e84583ae0895923249c873e129fd4f3c5a51d2b549e5e3ee37850e`,
which equals the container hash re-measured on 2026-10-08 — so the superseded build is as independently checkable as the
live one, and neither row here rests on this repository's own arithmetic alone.

### What did and did not change between them

14 commits, 96 paths, **44 of them product paths**: 42 under `apps/desktop/ui/src`, plus
`apps/desktop/src-tauri/src/ipc.rs` (the additive `MainWindowPage::Overview` variant and two window-title strings —
the variant names, which are the wire form, are unchanged) and its test. `crates/` (all Core, storage and the
migrations inside it), `assets/` including `design-tokens.json`, `schemas/`, `fixtures/`, `golden/`, `.github/`,
`scripts/`, `09_ADR/` and `templates/` are **byte-identical subtrees**, and `generate_handler!` registers the same
**30** commands at both heads. Both CI runs report **868 Rust tests across 47 result lines**; the UI suite moved
225 in 8 files → **291 in 9 files**. So no metric operation, denominator or threshold in `V1_METRICS.md` is affected
by the swap — what changed is the interface a participant reads, which is exactly why the accepted build and the
frozen build have to be the same bytes.

Two consequences follow, and both are protocol rather than footnote:

1. **The installer digest is an identity of a build, not an equality key across builds.** `P5_PACKAGING_REPORT.md`
   §"Are the produced packages byte-reproducible?" measured that two builds of the same tree give different
   installer sizes and that the payload differs in 20 of 15,001,088 bytes (PE `TimeDateStamp` plus the 16-byte
   RSDS CodeView GUID, set per link). F3's installer therefore differs from F2R1's even though their product
   trees are identical. Each session records the SHA of the bytes that participant actually received (§30).
2. **No substitute is allowed** (§1): not F2R1's artifact `11397938806`, not the superseded F3 `11419727517`, not
   the documentation head's own artifact `11592690212` (run `37874177649`, #87 — it packages a docs-only commit and
   is enumerated here only to be refused), not a local rebuild, not `cargo run`, not Vite, not a later docs-only CI
   artifact, not any unreviewed product head. The V1 activation commit's own CI artifact is likewise **not** the
   cohort build — §46 says so explicitly, because the activation commit only proves the product did not move.
   **Changing this table requires the owner's authority; a round may not re-point it to make a comparison easier.**

## Distribution is controlled, one-to-one, and not public

`PUBLIC_DISTRIBUTION` stays `NOT_AUTHORIZED` (§2). This round authorizes transfer of the exact research build
to **named, consenting participants only**: private direct transfer, a participant-restricted temporary link,
supervised installation, or a controlled research machine. No public download page, no public GitHub Release,
no forum link, no package manager, no unrestricted shared link, and no participant redistribution — and the
build must never be called a public beta.

Before a participant installs, the moderator states: this is an **unsigned research/evaluation build**; Windows
may show a security or reputation warning; they may decline; they should **not** disable Windows security
globally to get around it; and no product semantics will be taught before measurement. Install friction is
evidence, recorded under T0 — not an obstacle to route around.

## Hard principles

`REAL PEOPLE. REAL OWN ARTIFACTS. NO SIMULATED USERS. NO INTERNAL TEAM AS FORMAL PARTICIPANTS. NO LEADING
MODERATION. NO SECRET PRODUCT PATCHES DURING A COHORT. REPORT ACTUAL NUMERATOR / DENOMINATOR. PRESERVE
PRIVACY. CANDIDATE ≠ TASK.`

§5 is the rule that decides what this round could and could not do. With no real eligible participant and no
session evidence, the round is **Recruitment Ready only**: the pack is built, V1 is opened, `research_state` is
`RECRUITMENT_READY`, P5 stays `PASS_COMPLETE`, the product stays `MVP_CANDIDATE`, the commit is docs and
governance only, CI passes, and the round stops. No participant was simulated, no LLM persona was used, no
internal member was counted, no quote was invented, no task completion was marked, and no session was written.

## Human / agent division (§48)

| Human operator | Agent |
|---|---|
| recruits participants | maintains the protocol |
| obtains consent | validates eligibility against §7 |
| transfers the research build | converts **real** notes/transcripts into anonymized records |
| moderates, or receives the recording | calculates metrics and classifies findings |
| protects private artifact data | synthesizes evidence, and **refuses to fabricate missing data** |

An agent cannot substitute for a participant. That is not a humility clause; it is the whole content of V1,
because a simulated engineer tells you nothing about a real one's build.

## External evidence root

Raw evidence lives **outside Git** (§41), because it contains consent records and other people's proprietary
build facts. Created at activation:

```text
C:\Users\16429\AppData\Local\Temp\FirmwareSight-V1-External-Validation-20261006\
  00_authority  01_artifact  02_recruitment  03_consent_private  04_sessions_raw
  05_recordings_private  06_artifact_metadata_private  07_analysis_working  08_reports  09_archive
```

What enters the repository is the anonymized session record and the aggregate analysis only, after §50's
privacy scan.

## Pack index

| Path | Purpose | State |
|---|---|---|
| `V1_PLAN.md` | scope, cohort, batching, freeze rule, what V1 owns and cannot prove | written |
| `V1_METRICS.md` | M1–M6 contract, thresholds fixed before any data exists | written |
| `V1_PARTICIPANT_REGISTER.md` | anonymized register of participants and cohort status | **0 rows** |
| `protocol/PARTICIPANT_SCREENING.md` | §7 eligibility A–G, §8 cohort, §12 exclusions | written |
| `protocol/CONSENT_PRIVACY.md` | §10 privacy, §11 consent, §50 pre-commit scan | written |
| `protocol/MODERATOR_GUIDE.md` | §24 discipline, session order, §27 critical watch | written |
| `protocol/TASK_SCRIPT.md` | §23 T0–T8 neutral wording and the forbidden hints | written |
| `protocol/INTERVIEW_SCRIPT.md` | §19 M4 rubric, §20/§21 questions, §22 qualitative value | written |
| `sessions/README.md` | session register and intake path (§49) | **0 sessions** |
| `sessions/TEMPLATE.md` | the §30 per-session record shape | `TEMPLATE` |
| `analysis/METRICS.md` | aggregate M1–M6 table | all `NOT_MEASURED` |
| `analysis/MISUNDERSTANDINGS.md` | M01–M17 log, with V0 lineage | **0 entries** |
| `analysis/FINDINGS.md` | product findings, §31 classification and severity | **0 entries** |
| `deliverables/V1_VALIDATION_REPORT.md` | §53's 24-section report skeleton | skeleton, no results |
| `deliverables/V1_GATE_RECOMMENDATION.md` | §39/§54 verdict, currently insufficient sample | written |

Not yet created, on purpose: `analysis/BATCH_A_INTERIM_REVIEW.md` (§35 — it needs four real eligible
participants) and the §52 final analysis files (`V1_METRIC_TABLE.md`, `V1_COHORT_SUMMARY.md`,
`V1_MISUNDERSTANDING_MATRIX.md`, `V1_PRODUCT_FINDINGS.md`, `V1_VALUE_SIGNALS.md`, `V1_RETURN_INTENT.md`,
`V1_TOOLCHAIN_DEMAND.md`) — they need `N >= 8` or an explicit interim request from the Architect. Creating
them now would mean creating them empty of the thing they exist to summarize.

## Recruitment message (§47) — prepared, not yet sent

Outreach is the human operator's job (§48), and nothing in this pack has been sent to anyone. This is the
template to send, and it satisfies §47's required content in order: it says what the research is, who it wants,
how long it takes, what artifact is used, that no source is requested, that artifacts stay local, that the build
is unsigned, that redistribution is not public, that feedback is anonymized, and that recording is optional. It
promises no compensation (the owner has offered none), and it never calls FirmwareSight a beta or a release.

```text
Subject: 30–45 minutes on your firmware release evidence — research, not a sales call

Hi — I'm running usability research on a local desktop tool that looks at embedded
firmware release evidence (ELF size and symbol attribution, changes between two
builds, and what the evidence does and doesn't support before a release).

I'm looking for firmware and embedded engineers who actually take part in a release:
writing, reviewing, or signing off on one.

What a session involves
  • about 30–45 minutes, one-to-one, remote or at your machine
  • you use the tool on a build from your own project — one you're authorized to
    test tools on and that isn't confidential
  • you think out loud while you work; I stay out of the way and don't coach you
  • then I ask what made sense and what didn't

What I don't want
  • no source code, no repository access, no private firmware images
  • your artifacts stay on your machine by default — nothing is uploaded anywhere
  • this is not a product sale and not a test of you

Before you install
  • the build is unsigned. Windows may show a security or reputation warning; that's
    expected, not a defect. Please don't disable Windows security globally to get
    around it — if policy blocks unsigned software, tell me and we'll stop there
  • I won't explain how the tool works beforehand; what you figure out on your own is
    the thing I need to see

Consent and privacy
  • you can stop at any time and your session won't be used
  • recording (audio/screen) is optional and only with your explicit say-so; if you
    decline, I take notes instead
  • reporting is anonymized: role, experience band, toolchain, ecosystem — no name,
    no employer, no customer
  • this is a controlled one-to-one evaluation build, not a public release, not a beta
    program, and not something to share onward

If you're in, reply and I'll send the build and a short screening questionnaire.
```

Screening runs from `protocol/PARTICIPANT_SCREENING.md`, consent from `protocol/CONSENT_PRIVACY.md`, and the
current count a sender should expect to report is in `V1_PARTICIPANT_REGISTER.md`: **0 eligible participants,
0 sessions.**

## Relationship to V0

V1 reuses V0's research discipline rather than restarting it: the misunderstanding codes M01–M17, the
neutral-task rule, the exclusion of team members and pre-briefed people, the anonymized register, and the
refusal to fabricate all come from `V0_VALIDATION/protocol/` and the archived V0 Batch A prompt. Three things
change deliberately, and each is recorded where it is used rather than left implicit:

| | V0 | V1 |
|---|---|---|
| instrument | clickable 7-screen prototype at v0.1.0 | the installed cohort Windows build — `11573661113` from head `41bb6a36`, refrozen 2026-10-08 (the first frozen build was F3's `11419727517`) |
| thresholds | no preset mathematical threshold | M1–M6 thresholds fixed **before** data (§15, §18 "do not redefine after seeing data") |
| critical watch | C1–C5 | C1–C8, adding product-risk codes C6–C8 (§27) |
| gating role | `NON_BLOCKING_USER_FEEDBACK_TRACK` since ADR-0026 | V1 is the track that P5 explicitly handed its L11 limitation to |

V0's honest zero is untouched: `0 / 8` eligible external sessions, `V0 INCOMPLETE — insufficient external
sample`. V1 does not inherit that zero as its own result; it inherits the discipline that kept it honest.
