---
title: "V1 Participant Screening"
doc_id: "FS-V1-010"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-06"
---

# V1 Participant Screening

Continues `V0_VALIDATION/protocol/PARTICIPANT_SCREENING.md` and the screening section of the archived V0
Batch A prompt, with one structural change: V0 screened against a prototype a participant never installed,
so V1 adds artifact authority and supported-cohort questions the prototype could not need.

## Formal eligibility — all of A–G (§7)

| | Requirement | How it is established |
|---|---|---|
| A | external to the FirmwareSight project and team | direct question; the register's `EXCLUDED_INTERNAL` flag exists for the near-misses |
| B | has **not** authored FirmwareSight code, prompts, design, protocol or evidence | direct question |
| C | has **not** been pre-trained on Analyze / Compare / Gate / Unknown / Review / Evidence semantics | ask what they know of the tool; anything the moderator explained before screening contaminates comprehension |
| D | real involvement in embedded firmware development, review or release | role plus a concrete recent release responsibility |
| E | fits a target role family | see the list below |
| F | has authority to use the selected artifact(s) in research | explicit question about permission to run their own build through a third-party tool |
| G | brings a **real own-project artifact** for the formal own-artifact tasks | see §9 below |

Target role families: Firmware Engineer · Senior Firmware Engineer · Firmware Lead · Embedded Software
Engineer · Embedded Team Lead · firmware-oriented QA or release reviewer · a small-hardware founder or team
member who personally handles firmware release evidence.

## Excluded from any numerator or denominator (§12)

project team members; anyone already briefed on the UI semantics; anyone who designed any part of the product
or this protocol; a second session counted as a new person; a session where the moderator taught the workflow
before the tasks; **simulated participants and AI/LLM personas**; and invalid or unsupported inputs inside
supported-workflow denominators.

## Supported cohort (§8)

Preferred inputs, all inside FirmwareSight's measured boundary: GCC / `arm-none-eabi-gcc` ELF; Clang ELF
within the documented measured Arm cohort; GNU ld MAP where available.

Ask, without leading: which toolchain produced the build they intend to bring; whether a link map exists for
it; whether the build is one they personally work with.

**Keil and IAR.** Valuable discovery participants. They must **not** be told those toolchains are currently
supported. If their real output cannot enter the supported input path, classify the session
`DISCOVERY_OUT_OF_COHORT`: it contributes qualitative demand, workflow and toolchain evidence, does not enter
supported-workflow denominators unless that participant independently provides a supported artifact, and its
unsupported-toolchain behaviour is **not** recorded as a usability defect.

## Own artifact screen (§9)

- ≥ 1 real own-project ELF → required for M1, M2, M4, M5, M6.
- matching MAP → wanted; its absence is a legitimate research condition, not a failure.
- **two real builds of the same project** (baseline + target) → required for M3. A participant with one valid
  build stays a formal participant and is excluded from M3's denominator with that reason printed.
- Repository or demo fixtures may not substitute. Never hand a participant a repo ELF for a formal task.
- Confidentiality: the participant keeps their bytes on their own machine or a controlled research machine
  (§10). Screening asks whether they are **allowed** to use the artifact in research, not for a copy.

## Recorded per candidate, anonymized (§7)

Participant ID · role · experience band · toolchain · MCU/RTOS ecosystem · team-size band · release-frequency
band · current pre-release workflow · cohort status. Employer and customer identity are not recorded by
default.

## Screening questions (asked before consent, answers recorded anonymized)

1. What do you work on, and what is your role in a firmware release?
2. What happens today, in the last week before you ship? (Do not name product features.)
3. Which toolchain produced your most recent release build?
4. Do you have a build of your own project you are allowed to run through a new local tool, and can you bring
   two builds of the same project?
5. Have you ever seen FirmwareSight, its screens, its documentation, or been told how it works? (If yes to any
   part → not eligible for formal sessions.)
6. Do you personally work with the release evidence, or does someone else own it?
7. Would you need approval to install an unsigned evaluation tool on your work machine? (Practical, not
   eligibility-affecting; it predicts T0 install friction.)
8. Are you comfortable that this session is research on a workflow, is not a product sale, and is not a test
   of you?

**Do not screen on whether they have used a MAP file.** MAP absence is a normal research condition; treating
it as an eligibility filter would quietly bias the cohort toward toolchains that emit one.
