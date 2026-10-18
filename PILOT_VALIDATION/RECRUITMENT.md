---
title: "Pilot Recruitment"
doc_id: "FS-PL-004"
product: "FirmwareSight"
version: "0.6.0"
status: "MATERIAL_NOT_USED"
owner: "Research"
last_updated: "2026-10-17"
---

# Pilot recruitment (3–6 sessions)

**Not used yet.** Nobody has been contacted. Contact starts only after the entry conditions in [PILOT_PROTOCOL.md](PILOT_PROTOCOL.md) §3 hold. No participant, answer or need is ever invented to fill this file or the register.

## 1. Who

Formal V1 rules A–F apply as written ([PARTICIPANT_SCREENING.md](../V1_VALIDATION/protocol/PARTICIPANT_SCREENING.md)). Rule C applies at entry.

| Want | Why |
|---|---|
| 3–6 external firmware engineers, from at least 2 different teams or companies | One team's habits are not a pattern |
| At least 2 who use **GCC or Clang with GNU ld** (`IN_COHORT`) | The supported path must be seen working on real builds |
| At least 1 who uses **Keil MDK / Arm Compiler 6** if available (`OUT_OF_COHORT`) | D4 demand and real armlink shapes (RW compression, µVision builds). They are told before the session that their MAP is not read; nothing is promised. |
| Each brings **two builds of one own project**, ideally the last shipped one and a newer one, with GNU ld MAPs if they have them | PT3 and PT3b need a pair; PT2 needs the MAP for attribution |
| A mix of roles: at least one who **reviews or signs off releases** | PT4 hand-off and the Gate wording are judged by people who hand work on |
| Windows 10 or 11 machine they control, with permission to install an unsigned build | The build is unsigned (V1 unsigned-build disclosure) |

Not invited: anyone who has seen a FirmwareSight demo or document; anyone already in, or planned for, the formal V1 pool; anyone who cannot get permission to run their own build through a third-party tool (rule F).

## 2. Screening steps (in order, recorded anonymized)

1. **Pool decision.** Decide Pilot or V1 *before* any contact about the product. Record it in the register. A Pilot participant is permanently out of formal V1.
2. **Screening questions:** the V1 list, plus:
   - Which compiler and linker does the project you would bring use?
   - Can you bring two builds of it, and the MAP files?
   - How do you compare a new build with an older one today?
   - Who receives your findings about a build, and in what form?
3. **Own-artifact screen** (V1 §9): the participant confirms they may use the files. FirmwareSight never receives them, and nothing leaves their machine.
4. **Consent** ([CONSENT_PRIVACY.md](../V1_VALIDATION/protocol/CONSENT_PRIVACY.md) items, plus the Pilot sentence) before the session, never during it.

## 3. Invitation text (Chinese and English; send only after the entry conditions hold)

> 您好，我们在做一个面向嵌入式固件的本机分析工具，正在邀请 3–6 位工程师参加一次约 60 分钟的早期探索性体验。您会在自己的电脑上，用自己项目的两个构建试用它；文件不会离开您的电脑，我们也不会收集固件内容、源码或路径。这是早期测试版本（未签名），参与后您将不再参加我们之后的正式研究。如果您愿意了解详情，请回复，我们会先请您回答几个简短的筛选问题。

> Hello — we are building a local analysis tool for embedded firmware and are inviting 3–6 engineers to a 60-minute early exploratory session. You would try it on your own computer with two builds of your own project; the files never leave your machine, and we do not collect firmware content, source or paths. The build is an unsigned early version, and taking part means you will not be asked to join our later formal study. If you are interested, reply and we will send a few short screening questions first.

The text names no feature, no page and no toolchain support, so that PT1–PT4 are not primed.

## 4. Counting

Only sessions that happened are counted, in [PILOT_REGISTER.md](PILOT_REGISTER.md). An invitation, a scheduled session or a no-show goes under "Invitations". Findings are reported as counts ("2 of 4 sessions"), never as percentages.
