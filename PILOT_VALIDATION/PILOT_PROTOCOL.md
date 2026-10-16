---
title: "Exploratory Pilot Protocol"
doc_id: "FS-PL-001"
product: "FirmwareSight"
version: "0.6.0"
status: "PROTOCOL_DRAFT"
owner: "Research"
last_updated: "2026-10-16"
---

# Exploratory Pilot Protocol

## 1. Participants

- **Size:** 3–6 sessions. That is enough to see repeated problems and demand, and too few for any rate. No percentage from the Pilot is reported as a measure.
- **Who:** external firmware engineers in the V1 target role families (`V1_VALIDATION/protocol/PARTICIPANT_SCREENING.md`). Rules A, B, D, E and F apply as written. Rule C (not pre-trained on the product's semantics) applies at *entry*: someone the team has already briefed teaches us nothing about first use.
- **Pool separation:** a Pilot participant is permanently ineligible for formal V1 (README). Record the pool decision in the register before the invitation goes out.
- **Own artifacts:** they bring a real own-project ELF. A matching GNU ld MAP and a second build of the same project are wanted, but not required. Fixtures from this repository are never handed out as a participant's artifact.
- **Keil, IAR or ArmClang users** are welcome. Their sessions are coded `OUT_OF_COHORT` and feed D4 demand. They are never told those toolchains are supported, and unsupported-toolchain behaviour is not coded as a usability defect.

## 2. Consent, privacy and distribution

Use `V1_VALIDATION/protocol/CONSENT_PRIVACY.md` unchanged. That covers the consent items, the unsigned-build disclosure, recording only with its own consent, the distribution limits, what is never collected (firmware bytes, MAP content, source, names, paths), the raw evidence kept outside Git, and the scan before any `git add`. The Pilot's external evidence root is `FirmwareSight-Pilot-<date>/`, laid out like V1's.

Add one sentence to the consent: *"This is an early exploratory session; taking part means you will not be asked to join the later formal study."*

## 3. Build under test

- One Windows installer, built from a named `main` commit at or after the D2C merge, verified like the V1 builds (installer SHA-256, `toolchain.git_commit`). The session record names it.
- The same bytes for every Pilot session. A new build starts a new Pilot round with its own register section.
- This is **not** the V1 cohort build. The V1 re-freeze (route R3) is a separate Owner-authorized round.

## 4. Tasks

The moderator reads the wording as written. A hint is an intervention and is logged as one, with the reason. Never name a page, button or file type.

| Task | Say this | Never say | Observe |
|---|---|---|---|
| **PT1** First look | "Please open your own build in this tool and tell me what you learn about it." | "Analyze", "choose ELF", "add MAP" | First action, path, stalls, whether a MAP is added unprompted, the first fact they read aloud |
| **PT2** Growth source | "Suppose this build is bigger than you expected. Use the tool to find where the size comes from." | "Objects", "Libraries", "Details" | Whether they reach object/library rows; whether they trust the share and the "not attributed" line; what they would do next |
| **PT3** Two builds (only with a second build) | "Here is an older build of the same project. What changed between the two?" | "Compare", "baseline" | How they choose base and target; whether they'd want to keep a fixed baseline; whether the object changes answer the question |
| **PT4** Hand-off | "If you had to send what you found to a teammate, what would you do?" | "export", "report", "Bundle" | What they look for; which format they expect (HTML, CSV, Markdown, a screenshot) |

Record per task: outcome (`completed unassisted` / `completed after intervention` / `not completed` / `not applicable`), first path, wrong turns, interventions, and quotes kept verbatim and separate from interpretation.

## 5. Debrief (after the tasks, never before)

1. How often do you compare a build with an earlier one today, and how? (frequency band, tool)
2. Was anything here new to you about your own build? What exactly?
3. What did you expect to find that was not there?
4. Which toolchain and linker do your other projects use?
5. Have you needed per-function stack usage, third-party component versions, or archived symbols for a release? When was the last time?
6. Would you open this again after your next build? Why or why not? Record the answer verbatim. It is **not** V1's M6 and is not coded as one.

## 6. Coding and reporting

- Each session is coded against the four route questions in the README: repeat comparison, attribution usefulness, toolchain mix, and deeper-evidence demand. Each is coded `YES` / `NO` / `UNCLEAR` with the evidence line.
- Defects found are filed as product issues, with the build SHA and steps, and never with participant data.
- After the round, a short findings note goes into `PILOT_VALIDATION/analysis/` with counts ("2 of 4 sessions…"), never percentages. It closes with which route entry conditions the round does and does not satisfy (`06_DELIVERY/12` D3 and D4).
- A D3 entry decision citing the Pilot quotes the findings note. It never cites a single session.
