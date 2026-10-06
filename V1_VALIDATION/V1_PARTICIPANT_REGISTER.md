---
title: "V1 Participant Register"
doc_id: "FS-V1-003"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-10-06"
---

# V1 Participant Register

Anonymized only (§7). This register is the source of truth for N, and N is the number every V1 denominator
is built from — so its empty state is stated as loudly as any filled row would be.

## Counters

```text
unique eligible external participants  = 0   (minimum for a V1 pass set: 8; target 12-15)
sessions conducted                     = 0
formal (supported-cohort) participants = 0
discovery-out-of-cohort participants   = 0
follow-up sessions                     = 0   (a follow-up does not increase unique N)
participants excluded                  = 0
```

## Fields recorded per participant

| Field | Allowed form |
|---|---|
| Participant ID | `V1-P001`, `V1-P002`, … · follow-ups `V1-P001-F1` |
| Role | free text role family from §7 E |
| Experience band | e.g. `<3`, `3-5`, `5-10`, `10+` years in embedded |
| Toolchain category | GCC / arm-none-eabi-gcc / Clang / IAR / Keil / other |
| MCU or RTOS ecosystem | category, not product name tied to an employer |
| Team-size band | e.g. `1-5`, `6-20`, `21-100`, `100+` |
| Release-frequency band | e.g. `weekly`, `monthly`, `quarterly`, `per-project` |
| Current pre-release workflow | short description, no confidential specifics |
| Own artifact | `YES` / `NO`, artifact type, `MAP YES/NO`, size band |
| Own baseline+target pair | `YES` / `NO` (drives M3 eligibility) |
| Cohort status | `FORMAL_SUPPORTED_COHORT` / `DISCOVERY_OUT_OF_COHORT` |
| Quality flag | `VALID` / `VALID_WITH_INTERVENTION` / `DISCOVERY_OUT_OF_COHORT` / `EXCLUDED_PROTOCOL_CONTAMINATED` / `EXCLUDED_INTERNAL` / `EXCLUDED_NO_CONSENT` / `EXCLUDED_NO_REAL_ARTIFACT` |
| Consent | `YES` (formal sessions require it before the session starts) |
| Recording | `AUDIO` / `SCREEN` / `VIDEO` / `NOTES_ONLY` |
| Build received | artifact id `11419727517`, head `08fdfcb`, **and the SHA of the installer bytes that participant actually used** |

Never recorded here: name, email, employer or customer identity, private repository or remote, device serial,
absolute private paths, firmware bytes, MAP content, source, or confidential release information (§10, §50).

## Rows

| ID | Role | Experience | Toolchain | Ecosystem | Team band | Release band | Own artifact | Pair | MAP | Cohort | Quality flag |
|---|---|---|---|---|---|---|---|---|---|---|---|
| — | | | | | | | | | | | |

**No participant has been recruited, screened, consented or sessioned.** Rows are added only from a real
session that happened, by the human operator who ran it, from notes or a transcript that exists.

## Rules this register enforces

1. **No simulated participant, no LLM persona, no invented quote, no fabricated completion** (§5). V0's
   Batch A prompt excluded AI Agents and LLM personas from N on the same terms; the rule did not expire.
2. **Internal people are not formal participants** (§7 A/B, §12): anyone who authored FirmwareSight code,
   prompts, design, protocol or evidence is excluded, and so is the project team. Excluding them is recorded
   as `EXCLUDED_INTERNAL`, not silently omitted.
3. **A repeat session from the same person is a follow-up**, not a new participant.
4. **A taught-before-task session is `EXCLUDED_PROTOCOL_CONTAMINATED`** and does not enter a numerator or
   denominator.
5. **Keil/IAR discovery participants stay out of supported-workflow denominators** unless they independently
   provide a supported artifact (§8).
6. **Fixture-based sessions do not count** (§9): a repository demo ELF is not an own artifact.
7. Every exclusion is written down with its reason. An unexplained absence from a denominator is denominator
   laundering (§37).
