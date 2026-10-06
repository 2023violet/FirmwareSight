---
title: "V1 Consent and Privacy"
doc_id: "FS-V1-011"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-06"
---

# V1 Consent and Privacy

Continues `V0_VALIDATION/protocol/CONSENT_PRIVACY.md`. V1 adds one thing V0 did not need: the participant runs
**their own real firmware** through software installed on a machine, so consent now covers an install of an
unsigned build as well as a recording.

## Consent before the session (§11)

Explicit consent is obtained before each formal session, and it must cover all of:

- the purpose is research on a product and a workflow, not a sale and not an exam of the participant;
- the build they are installing is an **unsigned research/evaluation build** (see the disclosure below);
- their own artifacts remain **local by default** — nothing is uploaded, and no copy is requested;
- they can stop at any point, and stop means their data is not used;
- recording is optional, and recording requires its own explicit consent;
- reporting is anonymized;
- no employer or customer secrets are being asked for.

Audio, screen or video recording needs explicit consent **before recording starts**. If recording is declined,
moderated notes are acceptable — and post-hoc opinion alone is **not** enough for behavioural metrics: a task
that was not observed cannot be coded as completed (§11).

## Unsigned-build disclosure (§2)

Before install, say, in plain words:

> This is an unsigned research and evaluation build. Windows may show a security or reputation warning because
> nobody has signed it. You can decline to install it. Please do not disable Windows security globally to get
> past the warning — if you are not permitted to install unsigned software, tell me and we will stop. Nothing
> about how the product works will be explained to you before we measure.

Install friction, the SmartScreen or reputation warning, and any launch refusal or typed refusal are recorded
as **evidence under T0**, not as an obstacle to route around. A participant who cannot install unsigned
software is a real finding about distribution, and is reported as one.

## Distribution limits (§2)

The research build moves to **named, consenting participants only**: private direct transfer, a
participant-restricted temporary link, supervised installation, or a controlled research machine. `PUBLIC_
DISTRIBUTION` remains `NOT_AUTHORIZED`. Forbidden: a public download page, a public GitHub Release, a public
forum link, package-manager publication, an unrestricted shared link, participant redistribution, and calling
the build a public beta.

## Not collected by default (§10)

Firmware binary bytes · MAP files · source code · private repositories · credentials · customer or device
identifiers · confidential release information · private Git remotes.

Participant artifacts stay on **their** machine, or on a controlled research machine.

## Repo-safe vs Git-forbidden

Repo-safe facts about a participant artifact: `OWN_ARTIFACT = YES`, artifact type, toolchain category,
ecosystem category, size band, `MAP YES/NO`. A full artifact SHA may remain private if the participant prefers.

Repo-forbidden: real names, emails, company or customer names, private repos, absolute private paths, Git
remotes, device serials, firmware bytes, MAP content, confidential release information, recording URLs or
private recording locations, and any private absolute path.

## Where raw evidence lives (§41)

Outside Git, in the external evidence root:

```text
FirmwareSight-V1-External-Validation-20261006/
  03_consent_private     consent and recording permissions, never committed
  04_sessions_raw        moderator notes and transcripts as they actually happened
  05_recordings_private  audio/screen/video, only where consented
  06_artifact_metadata_private  participant artifact facts that must not enter Git
  contacts               participant contact details — never in Git, an audit pack or a release
```

Committed session documents contain **anonymized evidence only**.

## The scan before any `git add` (§50)

Run against the staged text of every session or analysis file, and redact or move outside Git anything found:

```text
real name · email · company or customer name · private repo · absolute private path · git remote ·
device serial · firmware bytes · MAP content · confidential release info · recording URL or private location
```

A hit that cannot be resolved by redaction is not committed. The correct outcome is a smaller committed file
or none at all, with the raw material kept in `04_sessions_raw`.

## Quotes (§28)

Quotes must be verbatim. Keep `QUOTE` separate from `INTERPRETATION`. If exact wording is not certain, label
it `MODERATOR_NOTE — NOT VERBATIM QUOTE` rather than presenting a reconstruction as a quotation. Redact
employer or project secrets out of a quote even when the quote is real.
