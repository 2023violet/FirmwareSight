---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Product / Research"
last_updated: "2026-09-29"
---

# ACTIVE TASK

```text
V0_BATCH_A_EXTERNAL_VALIDATION — the research track ADR-0025 makes the precondition for any further
pre-G1 engineering slice. This is a RESEARCH / EVIDENCE task. It is not a coding prompt, not P1-A1,
not a product development stage, and it does not authorize writing product code.
```

```text
STATUS 2026-09-29: ACTIVATED AT 3b59585 (Run #11 36516209283, success, 7 of 7 jobs).
P1-A0 and its correctness closure are COMPLETE and are not re-opened by this task; see
`pre_g1_execution` in `BASELINE.yaml`.

BATCH A ACTIVE — WAITING FOR REAL PARTICIPANTS
Formal V0: 0 / 8 eligible external sessions
Batch A:   0 / 4-5 eligible external sessions

The activation prompt is resumable and its first invocation had no participant evidence to process.
Nothing was invented in its place: no PA-00X file, no quote, no timing, no metric, no count, no
willingness-to-pay signal. The registers still read NOT_RECRUITED / NOT_SCHEDULED.
```

Authority: the *FirmwareSight V0 Batch A External Validation Activation & Interim Review Execution Prompt
v1.0 — Architect Reviewed*, supplied inline, recorded in `10_AUDIT/SOURCE_PROMPTS/README.md` without a
SHA-256 rather than inventing one. It succeeds
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_V0_Batch_A_External_Validation_PROMPT_v1.0.txt`, which authorized
the earlier Batch A takeover and withheld `v0.5.2`.

## The research outcome

Produce 4–5 real, eligible, externally moderated sessions against the frozen V0 workflow prototype, then a
Batch A interim review that lets the architect decide whether any further pre-G1 engineering slice is
worth authorizing. The deliverable is evidence about comprehension and commercial signal — not code.

```text
Recruit → screen → consent → moderate T1–T10 on the frozen prototype → PA-00X.md → registers →
metrics at real n/N → Batch A Interim Review → STOP for the architect
```

## The formal instrument is frozen

- Prototype: `V0_VALIDATION/prototype/`, version **`v0.1.0`**. Open `index.html` directly or serve the
  directory with `python -m http.server 8765`. Reset by reloading or `Alt + Shift + R`.
- Tasks: the exact T1–T10 wording in `V0_VALIDATION/protocol/TASK_SCRIPT.md`. Paraphrasing a task to make
  the product easier is prohibited; a hint is a moderator intervention and must be logged.
- Session record: `V0_VALIDATION/sessions/PA-00X.md`, created from `V0_VALIDATION/sessions/TEMPLATE.md`
  **only after a real session happens**.
- Moderator conduct, screening and privacy: `protocol/MODERATOR_GUIDE.md`,
  `protocol/PARTICIPANT_SCREENING.md`, `protocol/CONSENT_PRIVACY.md`, `protocol/INTERVIEW_SCRIPT.md`.
- **The P1-A0 production desktop is not the formal instrument.** Formal V0 tests comprehension of
  Analyze → Compare → Gate → Bundle → History; P1-A0 implements only real intake plus the Analyze summary,
  so substituting it would change the instrument and make Batch A incomparable. If a moderator shows it at
  all, it is after the full session, labelled `NON_FORMAL_APPENDIX`, and it enters no numerator, no
  denominator, no Formal N and no V0 claim. Default: do not add that appendix.

## Hard boundaries of this task

- **No product code.** Under this task none of `crates/**`, `apps/**`, `schemas/**`, `migrations/**`,
  `Cargo.toml`, `Cargo.lock`, `package.json`, the pnpm lockfile, `.github/**`, the design tokens, the
  product UI or the prototype may be modified. There is no exception clause. A Critical Execution Blocker
  in the prototype is reported and stops Batch A; it is not fixed here.
- **No invented humans.** No synthetic participant, AI persona, fabricated quote, task outcome, timing,
  willingness-to-pay, or n/N. An ineligible person is never counted into Formal N to reach a sample size.
- **`V0` is not PASS after Batch A.** 4–5 sessions is a research milestone, not validation: formal V0 needs
  a minimum N of 8, and no completion ratio is reported below it.
- **No engineering authorization.** `P1-A1`, `P2`, `P3`, `P4` remain unauthorized; the interim review's
  disposition is a research finding that goes back to the architect.
- **`G1 = V0 PASS + P0 PASS` stays NOT CLAIMED**, `baseline_version` stays `0.6.0`, and completing Batch A
  creates no new baseline version by itself.
- **Privacy by default.** Anonymized participant id, role/experience/toolchain/ecosystem, observed task
  behaviour, non-confidential quotes and purchase/pilot signals only. No real firmware binaries, MAP files,
  source code, private repositories, credentials, customer or device identifiers, or confidential release
  information. Audio/screen/video only with explicit recorded consent; otherwise moderated notes. Redact
  employer and project detail from every quote. No phone numbers or e-mail addresses in this package.
- **Discovery toolchains.** Keil / ArmClang / IAR people are useful to talk to, but must never be told those
  toolchains are Supported.

## State this task runs against

```text
P0                  PASS — frozen at v0.6.0; no further P0 closure or promotion prompt will be written
P1-A0               COMPLETE — intake plus its evidence-identity and persistence correctness closure
Baseline            v0.6.0
Design tokens       v0.2.1
V0                  ACTIVE EXTERNAL VALIDATION / WAITING FOR REAL PARTICIPANTS — 0 / 8, Batch A 0 / 4-5
Formal G1           NOT CLAIMED
P1                  NOT PASS / NOT CLOSED        P1-A1 / P2 / P3 / P4   NOT AUTHORIZED
Run #6              36419864513 on 7d2f38a — success, 7 of 7 jobs (the last P0-chain remote fact)
Run #9              36499759371 on 2960173 — success, 7 of 7 jobs (the intake commit)
Run #10             36515470263 on ace6fbe — success, 7 of 7 jobs (the closure commit)
Run #11             36516209283 on 3b59585 — success, 7 of 7 jobs (the governance successor)
Peak RSS            NOT MEASURED      Fuzz: NOT RUN
RustSec             two accepted transitive advisories, unchanged by this track
```

## What may not be done here

Fabricate V0 evidence of any kind. Counts move only when a real, eligible, externally moderated session
exists and its evidence-integrity checklist passes; if any mandatory item fails, the session is recorded
as `DISCOVERY_ONLY` or `EXCLUDED_FROM_FORMAL_N` with a reason rather than counted silently. No percentage
is reported without its n/N, and no pass threshold is invented for the interim review — it uses evidence.

`V0_VALIDATION/**` is this task's own working area, so it is edited here; that reverses nothing about the
P0 and P1-A0 rounds, which were both forbidden from touching it.

## Next gate after this task

At **>= 4 eligible external sessions** with intact evidence: write
`V0_VALIDATION/batch_a/BATCH_A_INTERIM_REVIEW.md` (sample, task evidence at real n/N, mental-model
failures, workflow evidence, commercial signal, toolchain demand, participant-by-participant inclusion
status, one disposition, and the exact decisions the architect now owes), then stop.

`STOP FOR ARCHITECT REVIEW. DO NOT START P1-A1.`
