---
title: "U1 architect final verdict: independent visual acceptance record"
doc_id: "FS-U1P-R3-003"
product: "FirmwareSight"
unit: "U1_ARCHITECT_FINAL_VERDICT_GOVERNANCE_RECORD"
closes: "stage U1, on the Architect's own authority — U1 = PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS"
protects: "the 25-item acceptance tally, the two findings U1P-R3 measured and did not fix, and V1's separate authority"
status: "BASELINE"
authority: "10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R3_Architect_Final_Verdict_2026-10-08.txt + AGENTS.md 1, 8, 9, 10 + ADR-0018 + ADR-0029"
issued_by: "the Architect, not by any agent round"
written: "2026-10-08"
---

# FS-U1P-R3-003 — the verdict U1 was not allowed to write

Four U1 units — the third, sixth, seventh and eighth — closed on the same sentence, `U1 = READY_FOR_ARCHITECT_FINAL_
VISUAL_VERDICT`, and none of them went past it; the earlier ones stopped at `READY_FOR_ARCHITECT_VISUAL_REVIEW` and
`REQUIRES_ARCHITECT_POLISH_REVIEW`, which are the same ceiling wearing different words. This document is the record
of the verdict arriving from the other side. It changes **no product byte, no test, no token, no schema and no Gate
rule**; it writes down what the Architect decided about the evidence those rounds produced, and it keeps open the
four things the Architect kept open.

## 1. What arrived, measured rather than trusted

| Fact | Value | How it was checked |
|---|---|---|
| Delivered file | `FirmwareSight_U1P_R3_Architect_Final_Verdict_2026-10-08.txt` | in `C:\Users\16429\Downloads\`, 5,159 bytes |
| Logical lines / CR bytes | 52 lines, **0 CR** | counted on the bytes, not on a screen |
| Delivered SHA-256 | `8634321c5a289f68b83202c2016465fb21133afc7e23ca1d3a558e6bc036c619` | `sha256sum` |
| Stored blob | `f16f0e7ef1279883d62d909d5161d42ab756d74b`, 5,159 bytes, same SHA-256 | `git hash-object` / `git cat-file -s` / `git show :path \| sha256sum` |
| Archived vs delivered | byte-identical | `cmp` clean |

One hash, not two, because the file arrived LF-only and `.gitattributes` had nothing to normalise. That equality
is a measurement: every CRLF arrival before it needed a second digest and a line-by-line proof, and a reader
should not treat "one hash" as licence to skip the check.

It was archived at `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1P_R3_Architect_Final_Verdict_2026-10-08.txt` and
registered beside the execution prompts — **labelled there, and here, as an authority record rather than an
execution prompt**. It is the first thing in that directory that closes a track instead of opening one.

## 2. The verdict, in the Architect's words

```
U1 = PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS
Scope: visual productization readiness for a controlled V1 external-user study, NOT GA quality,
       feature completeness, or public distribution approval.
This is an ARCHITECT waiver of specifically enumerated cosmetic/interaction residuals, not a claim
that every Agent acceptance box passed.
```

The scope sentence is part of the verdict, not a comment on it. `PASS_COMPLETE` here means *the UI is visually
ready to be put in front of real engineers in a controlled study*. It does not mean the product is GA-quality, it
does not mean feature-complete, and it does not open distribution: B1 private beta, RC, GA, public release,
signing, notarization, the updater and commercial distribution all stay `NOT_AUTHORIZED`.

The record also says who did what: *"R3 Agent did not and could not self-authorize this verdict; this record
represents Architect's judgment of the uploaded evidence."* The eight rounds' `never_self_issued` lists stay
exactly as they were written. This word arrived from outside them.

## 3. The engineering authority it names, re-read instead of copied

The verdict cites commits and runs. Re-reading them from GitHub rather than repeating them is what makes this a
record and not a transcription, so each was re-queried on 2026-10-08 at the start of this round
(`00_authority/CITED_CI_REREAD.json`):

| Item | Value | Re-read result |
|---|---|---|
| Product commit | `41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177` | `U1P-R3: retain comparison context and close narrow viewport gaps` |
| Product CI | run `37828673549` | #85, completed / success, **10 of 10** jobs, head `41bb6a36` |
| Evidence successor | `4d25c85f96a844a43b84a56f12a085d12b3f834d` | documentation only |
| Evidence CI | run `37846189269` | #86, completed / success, **10 of 10** jobs, head `4d25c85f` |
| Review pack | `FirmwareSight_U1P_R3_Final_Visual_Review.zip` | still 4,224,650 bytes, SHA-256 `4de07c76…831e52`, unchanged on disk |
| Acceptance JSON | 25 / 23 / 1 / 1 / 0, 2 flags | re-counted by the Architect; re-derived here from the same `boxes` array |

The artifact the verdict's recommendation turns on was re-downloaded and re-hashed before anything in this
document was written (`01_artifact_reverification/ARTIFACT_REVERIFICATION.txt`), because the recommendation asks
for exactly that before authority documents change. All ten checks true: container 5,544,133 bytes
`330e83af…4f3fb4`, NSIS `FirmwareSight-0.6.0-windows-x86_64-nsis.exe` **3,896,257 bytes
`9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93`** — matching the verdict's own figures —
the container's internal `SHA256SUMS.txt` verified entry by entry, CRC clean, `artifact-metadata.json` reporting
`toolchain.git_commit = 41bb6a36…`, and the artifact **not expired**. One honest subtraction: GitHub's
`sha256_digest` field is `null` for this artifact on both the list and the single-artifact endpoint, so the second
authority here is the internal manifest plus the metadata commit, not a self-referential digest. Re-downloading
the bytes is evidence about the bytes; it is not the re-freeze, and nothing was installed or launched.

*(Dated erratum, later on 2026-10-08, owned by the V1 re-freeze round: the subtraction in the paragraph above is
withdrawn. These endpoints publish the checksum as **`digest`**, formatted `sha256:<hex>`, and there is no field named
`sha256_digest` — `gh api … --jq .sha256_digest` prints `null` for a **missing key**, which is not a published null. For
`11573661113` GitHub's `digest` is `sha256:330e83af75db705b2acae6ff559baf0e863d7766947bc21240488861cd4f3fb4`, equal to
the container hash this record's own check measured, so a GitHub-side authority does exist and the byte identity is
corroborated outside this repository's arithmetic. The verdict, the tally and the recommendation are untouched; the
nine other checks in §3 all read the same. Key presence was re-tested with `in` on both endpoints.)*

## 4. What the Architect looked at directly

Eight readings, taken off the images rather than off the agent's description of them: Compare's ready state at
1440 and 1024 explaining itself with no fabricated diff; the real Sections and Symbols counts and the
nonvolatile/RAM deltas above the fold at 1440, with the section table's heading near y=766, its column headings
near y=865 and its **first actual data row below 900 px**; the genuine report surviving same-process navigation
while cross-process retention is correctly **not** claimed and `diffId` stays ephemeral; the 1024/1056 Overview
band with the fourth-cell-like grey void gone and three truthful cells; matched, pending and policy-mismatch
states kept separate with no stale current PASS manufactured; Analyze showing a real sections table in the first
viewport at 1440 and naming both the current failure and the retained previous result on parse failure; the
Release Gate's BLOCK and UNKNOWN rule findings readable, with a historical run identified by the facts stored with
it and the frontend picker reserved for new runs; and History's focused categories with detail actions present
though nothing is selected by default.

Two of those readings agree with measurements this track made and printed; one refines them. The R3 report placed
the section table header at y≈869 from its own probe, while the Architect reads the table **heading** at y≈766 and
the **column headings** at y≈865 — different rows of the same table, both above or at the fold, and both agreeing
on the operative fact: the first per-section data row sits below 900 px. The verdict's numbers govern, because
they are the ones attached to the review being recorded; the round's probe measured a different row and said so.

## 5. Four deviations accepted, and what each one forbids saying

These are enumerated, not absorbed. The record's own header for them is
`MUST NOT BE SILENTLY CLOSED`, so each keeps a row here and in `BASELINE.yaml`.

| # | The residual | Disposition the Architect set | What is now forbidden |
|---|---|---|---|
| **A** | Analyze's metric band at 1024×720 leaves a conspicuous empty grey track — the round's own **FAIL** box | Classify as a **U1 visual known issue / V1 UX observation**; prioritise from the first four participant reports or from layout impact | Treating it as a mandatory pre-V1 engineering gate, or deleting the FAIL because the track was waived |
| **B** | Compare's Section Changes **first data row** falls below the 1440×900 viewport while the aggregate counts and real deltas are visible | **One-scroll-detail deviation ACCEPTED for V1 only** | Claiming literal satisfaction of the stricter first-data-row-before-fold aspiration |
| **C** | The Release Gate's baseline picker loses its draft selection on navigation | Record the usability ambiguity **for the study**; the persisted `GateRunDto` carries its own `baselineSnapshotId`, `snapshotId` and `policySha256` | Implying the picker is a historical input transcript, or suggesting the reset rewrites a saved record |
| **D** | R3 did not re-capture Overview's `Previous analysis` panel — the round's **NOT_VERIFIED** box | R2's images plus unchanged scoped-source behaviour are **contextual evidence, not a substitute** | Marking that R3 box PASS, or folding it into the O3 PASS |

**E** is a document defect, and the record routes it rather than excusing it: the entry-document's
ordinal/paragraph-count note *may* be corrected **with the next already authorized governance update**, and
explicitly **not** by a stand-alone CI-loop commit. The defect was found after U1P-R3's single permitted successor
had already been pushed, so it was written into that round's evidence root instead of being fixed by an
unauthorized second commit (`00_authority/POST_SUCCESSOR_DOC_NOTES.txt`): `.ai/ACTIVE_TASK.md` said *"Two
self-corrections are in the report"* while the report's §5 carries **three**, and the omitted one is the round's
correction about its own reporting. This update is the next already authorized governance update, so the line now
reads three, names the third, and no extra commit was made for it.

## 6. The tally guard

```
Acceptance JSON (independently re-counted): 25 items, 23 PASS, 1 FAIL, 1 NOT_VERIFIED, 0 NOT_CAPTURED;
2 MISMATCH_PROVED flags inside the 23 PASS items.
DO NOT rewrite this tally to 25/25 or to 100%.
```

The verdict waives residuals; it does not convert them. `MISMATCH_PROVED` stays what the pack says it is — a flag
on PASS items whose **subject is a genuine mismatch the product proved** (the stale policy fingerprint Overview
names; the refused same-build comparison) — and none of it says the build is ready. The FAIL stays a FAIL, the
NOT_VERIFIED stays NOT_VERIFIED, and `PASS + FAIL + NOT_VERIFIED + NOT_CAPTURED = ITEMS` still has to hold in any
document that prints these numbers. A later round that reports 25/25 has contradicted its own authority.

## 7. Gate interlock, restated because it did not move

`P5` remains `PASS_COMPLETE`. The product remains `0.6.0` **MVP_CANDIDATE**. `V1` remains `IN_PROGRESS` /
`RECRUITMENT_READY` with **0 eligible external sessions until separate authority is granted**. B1 private beta,
RC, GA, public release, signing, notarization, the updater and commercial distribution remain `NOT_AUTHORIZED`.
The open-source licence decision stays `PENDING_OWNER_CONFIRMATION`. Nothing in this record resumes V1, and a
visual acceptance of the UI is not a research pass: **L11** stays carried, and only real sessions can close it.

## 8. The recommendation the Architect wrote and did not execute

The record's last section is titled `NEXT ACTION RECOMMENDATION, NOT EXECUTED`, and this round treated that title
as binding. It recommends — before participant #1 — authorising a **separate** V1 cohort re-freeze / research
protocol amendment from the old F3 research artifact (`11419727517`, NSIS 3,888,432 bytes, `182506f2…63d12`) to
this product's CI artifact `11573661113` (run #85, product head `41bb6a36…`), with the downloaded bytes
independently re-verified first, the F3 source and all historical immutable research records preserved, and no
automatic start of participants: recruitment and controlled private transfer need their own owner authorization,
carrying the unsigned-build disclosure and consent.

What this round did: re-verified the bytes, because that is a read and the record asks for it before any authority
document changes. What it did **not** do: touch `V1_VALIDATION/`, move `v1_execution.cohort_build_frozen`,
rewrite the protocol, or begin recruitment. The F3 artifact stays the frozen cohort build in every authority
document until a separate authority says otherwise, and this record's own citation of the newer bytes is
therefore filed as *evidence about a candidate*, not as the re-freeze. Saying the difference out loud is the point
of §8: a re-read artifact in an evidence root is not a stage transition.

## 9. What changed in the repository, and what did not

Changed, all of it documentation and governance: this record, the archived verdict and its register entry,
`BASELINE.yaml`, `.ai/ACTIVE_TASK.md`, `.ai/CURRENT_STATE.md`, `.ai/HANDOFF.md`, `.ai/README.md`, `README.md`,
`INDEX.md` and `06_DELIVERY/06_STAGE_GATES.md`, plus the two ADR-0029 artifacts regenerated from the staged index.

Did not change: any path under `apps/`, `crates/`, `fixtures/`, `golden/`, `assets/`, `schemas/`, `migrations/`,
`scripts/` or `.github/`; `assets/design-tokens.json` (`94336906…14d4`) or the generated `tokens.css`; any DTO,
IPC command, Gate rule, schema, migration or dependency; the test counts, which stay **868 Rust across 47 result
lines and 291 UI in 9 files** on the 17-step gate, and are re-measured rather than restated in
`03_validation/`; the delivered pack, whose digest is what the Architect actually reviewed.

`active_task` goes to **`NONE`**. U1 is closed by its own authority; V1's next move is a recommendation awaiting
a separate grant; and AGENTS.md 1 forbids inventing work where the pointer says none. What is blocked, and by
whom: the cohort re-freeze (owner authority), recruitment and consent (§47/§48, human), any UI follow-up (would
need its own prompt), and the two §18 residuals in A and C (prioritised by the Architect from participant
evidence, not by an agent deciding to tidy a layout).

## 10. Evidence classification

**Observed**: the delivered verdict bytes and their digests; the two runs re-read job by job; the re-downloaded
artifact and its hashes; the live pack digest; the machine state (no install directory, 17 owner-store entries,
three owner digests unchanged). **Derived**: this record's tables, computed from those reads. **Declared**: the
verdict's own dispositions A–E and the scope of `PASS_COMPLETE`, which belong to the Architect and are quoted
rather than paraphrased where they bind. **Unknown**: whether the two open residuals matter to real participants —
that is precisely what V1 exists to answer, and this record does not pre-empt it.

## 11. Where the evidence lives

Outside Git, retained and not deleted: `%TEMP%\FirmwareSight-U1-Verdict-Record-<stamp>Z\` with
`00_authority/` (the delivered copy, its provenance JSON, the CI re-read, the live pack recheck, the pre-round
machine state), `01_artifact_reverification/` (the container, its payload, and both record forms),
`02_record/`, `03_validation/` (the gate log at the staged state) and `04_ci/` (this update's own run, read after
the push). The repository does not carry this round's CI number: a commit cannot record the run its own push
produces, and no further commit is made just to write a number back.

The next word on this track is not an agent's. U1 is closed; the pointer is `NONE`; and the only thing that moves
V1 is an owner decision about real people holding their own firmware.
