---
title: "Exploratory Pilot Protocol"
doc_id: "FS-PL-001"
product: "FirmwareSight"
version: "0.6.0"
status: "PROTOCOL_APPROVED_IN_PRINCIPLE"
owner: "Research"
last_updated: "2026-10-17"
---

# Exploratory Pilot Protocol

**D2C-R revision (2026-10-17).** The Owner approved the direction of this Pilot in principle and asked for admission preparation, not sessions. Nothing is handed out until §3's entry conditions hold.

The changes from the D2C draft:
- the target is now the R7 build;
- PT3 studies the pinned baseline that R5 shipped, instead of asking whether one is wanted;
- PT4 observes the Markdown report;
- §3 names the minimum human QA.

Recruiting and session-day materials: [RECRUITMENT.md](RECRUITMENT.md) and [MODERATOR_RUN_SHEET.md](MODERATOR_RUN_SHEET.md).

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

- **Target build:** the Windows installer from `main` `5015143` (the R-series final; CI run `39093441971`, 10/10), artifact `FirmwareSight-0.6.0-windows-x86_64`. That artifact expires on 2026-11-01, so download and keep it before then. Verify it like the V1 builds (installer SHA-256, `toolchain.git_commit`). The session record names the commit and the installer SHA-256.
- If a later `main` replaces it before the first session (for example the section-ranking fix of `06_DELIVERY/24`), the Pilot moves to that build, and the minimum QA below is re-run on it.
- **Entry conditions.** No Pilot build is handed to anyone until all of these hold:
  1. The Owner has given **final approval** of this protocol, separately from the in-principle approval of 2026-10-17.
  2. The **Pilot minimum** of the human QA pack has passed on that build: [evidence/D2C-R/HUMAN_QA_PACK.md](../evidence/D2C-R/HUMAN_QA_PACK.md) §5, meaning Q1, Q2, Q8, Q16 at 125%, and installation on a non-Owner account. Any limit the Owner accepts in writing instead is named in the register.
  3. The register records the build and the pool decision of each invitee.
- These conditions replace the D2C wording ("closed AC24, AC26 and AC20"). They keep its intent and the deadline in `NEXT47_VALIDATION/evidence/04_NEXT47_DEFERRED_NATIVE_QA.md`.
- The same bytes for every Pilot session. A new build starts a new Pilot round with its own register section.
- This is **not** the V1 cohort build. The V1 re-freeze (route R3) is a separate Owner-authorized round.

## 4. Tasks

The moderator reads the wording as written. A hint is an intervention and is logged as one, with the reason. Never name a page, button or file type.

| Task | Say this | Never say | Observe |
|---|---|---|---|
| **PT1** First look | "Please open your own build in this tool and tell me what you learn about it." | "Analyze", "choose ELF", "add MAP" | First action, path, stalls, whether a MAP is added unprompted, the first fact they read aloud |
| **PT2** Growth source | "Suppose this build is bigger than you expected. Use the tool to find where the size comes from." | "Objects", "Libraries", "Details" | Whether they reach object/library rows; whether they trust the share and the "not attributed" line; what they would do next |
| **PT3** Two builds (only with a second build) | "If you have brought an older build of the same project, please open it too. What changed between the two?" | "Compare", "baseline", "pin" | How they choose base and target; whether the object changes answer the question |
| **PT3b** Reference build (only with a second build, after PT3) | "Suppose the older build is the one you shipped last, and you will check every new build against it for a while. Set the tool up for that, then check the newer build against it." | "pin", "History", "baseline" | Whether they find the pin unprompted, and where they look first; whether the Compare they start next opens on the pinned build; whether they understand that unpinning changes no stored build; time to first pin |
| **PT4** Hand-off | "If you had to send what you found to a teammate, what would you do?" | "export", "report", "Markdown", "Bundle" | What they look for; which format they choose. If Markdown: where they would paste it (PR, issue, chat), whether they open the file before sending, and anything they say about its first line, tables or wording |

PT3b's wording tells the participant that the tool can be set up, but not how; any further hint is an intervention and is logged with its reason. PT3b is the D3 usability check: R5 shipped the pinned baseline under an Owner exception (`.ai/DECISIONS.md` RD4), so the Pilot now tests whether it is found and understood. It no longer asks whether it is wanted.

Record per task: outcome (`completed unassisted` / `completed after intervention` / `not completed` / `not applicable`), first path, wrong turns, interventions, and quotes kept verbatim and separate from interpretation.

## 5. Debrief (after the tasks, never before)

1. How often do you compare a build with an earlier one today, and how? (frequency band, tool). If PT3b ran: would a pinned reference build replace what you do today?
2. Was anything here new to you about your own build? What exactly?
3. What did you expect to find that was not there?
4. Which toolchain and linker do your other projects use?
5. Have you needed per-function stack usage, third-party component versions, or archived symbols for a release? When was the last time?
6. Would you open this again after your next build? Why or why not? Record the answer verbatim. It is **not** V1's M6 and is not coded as one.

## 6. Coding and reporting

- Each session is coded against the route questions in the README: pinned-baseline usability (D3, shipped), attribution usefulness, hand-off format, toolchain mix, and deeper-evidence demand. Each is coded `YES` / `NO` / `UNCLEAR` with the evidence line.
- Defects found are filed as product issues, with the build SHA and steps, and never with participant data.
- After the round, a short findings note goes into `PILOT_VALIDATION/analysis/` with counts ("2 of 4 sessions…"), never percentages. It closes with which route entry conditions the round does and does not satisfy (`06_DELIVERY/12` D4, D5 and D6), and with any usability defect in the shipped D3 workflow.
- A route decision citing the Pilot quotes the findings note. It never cites a single session.
