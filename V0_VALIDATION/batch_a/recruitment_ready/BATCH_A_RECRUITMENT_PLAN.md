---
title: "Batch A Recruitment Plan"
doc_id: "FS-V0-BA-010"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-09-27"
---

# Batch A Recruitment Plan

## Goal

Recruit 4–5 **real external** eligible participants for the first moderated V0 batch.

Batch A is not market-size research. It is a high-signal workflow/comprehension validation batch.

## Preferred participant mix

Recommended target mix for 5 participants:

- 2–3 primary-cohort firmware engineers using GCC/Clang/GNU ld / ESP-IDF / Zephyr
- 1 senior/lead participant who reviews or approves releases
- 1 Discovery-cohort participant from Keil/ArmClang/IAR if available

This is a recruitment target, not a quota that should force inclusion of an ineligible person.

## Eligibility

Must have:
- real embedded/firmware engineering work;
- participated in firmware build/release or pre-release checks;
- enough experience to discuss current release workflow.

Record:
- role;
- embedded years;
- primary toolchain;
- MCU/RTOS/framework;
- team size;
- release frequency;
- current checks.

## Exclusions from Formal N

Do not count:
- FirmwareSight project/design team;
- people already trained on the prototype semantics;
- the same participant’s repeat session;
- AI/LLM Persona;
- internal dry-run participants;
- anyone whose task completion required pre-session teaching of the workflow.

## Recruiting channels

Use channels where real embedded engineers can be reached, for example:
- professional/personal embedded-engineering contacts;
- local engineering groups;
- firmware/MCU communities;
- past colleagues/classmates doing real embedded work;
- relevant professional Discord/Slack/QQ/WeChat groups where research recruitment is allowed.

Do not scrape private contact details.

## Recruitment message

> 我们正在测试一个面向嵌入式固件工程师的桌面工具原型，主要围绕固件发布前的 Build 分析、版本差异、Release 检查和发布证据留存。
>
> 本次不是产品销售，也不是技术考试。
>
> 希望邀请有实际 Firmware 开发/发布经验的工程师参加一次约 30–45 分钟的远程/现场原型测试。
>
> 测试的重点是：第一次看到这个工具时，你会如何理解和操作它？
>
> 不要求提供公司源码、私有 Firmware、ELF/MAP、私有仓库或客户数据。
>
> 如需录屏/录音，会提前单独征得同意。

## Do not reveal before session

Do not pre-teach:
- Observed / Declared / Unknown;
- Gate;
- Review;
- Bundle;
- MAP workflow.

## Recruitment tracking

Use `BATCH_A_SCREENING_REGISTER.csv`.

Do not place private phone/email contact details in the shared research package.
