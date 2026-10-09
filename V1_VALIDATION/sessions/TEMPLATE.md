---
title: "V1 Session — Participant [ID]"
doc_id: "FS-V1-021"
product: "FirmwareSight"
version: "0.6.0"
status: "TEMPLATE"
owner: "Research"
last_updated: "2026-10-08"
---

# V1 Session — `V1-P0XX`

> Template only. Copy to `sessions/V1-P0XX.md` after a session that actually happened. Every field is filled
> from real notes or a real transcript, or is marked `NOT_MEASURED` / `NOT_RECORDED`. A blank in this template
> is not a value.

## Participant (§30, anonymized)

| Field | Value |
|---|---|
| Participant ID | `V1-P0XX` |
| Eligibility (§7 A–G) | |
| Quality flag | `VALID` / `VALID_WITH_INTERVENTION` / `DISCOVERY_OUT_OF_COHORT` / `EXCLUDED_*` |
| Role | |
| Experience band | |
| Toolchain | |
| MCU / RTOS ecosystem | |
| Team-size band | |
| Release-frequency band | |
| Current pre-release workflow | |

## Own artifact (§9, repo-safe facts only)

| Field | Value |
|---|---|
| `OWN_ARTIFACT` | YES / NO |
| Artifact type | |
| Toolchain category | |
| Ecosystem category | |
| Size band | |
| `MAP` | YES / NO |
| `OWN_BASELINE_TARGET_PAIR` | YES / NO (drives M3 eligibility) |
| Artifact SHA shared | YES / NO / private |

## Consent and recording (§11)

| Field | Value |
|---|---|
| Consent obtained before session | YES |
| Unsigned-build disclosure given | YES |
| Recording | AUDIO / SCREEN / VIDEO / NOTES_ONLY |
| Participant stopped early | YES / NO |

## Build under test (§30)

| Field | Value |
|---|---|
| Product | FirmwareSight 0.6.0 |
| Cohort head | `41bb6a36dd6e1ce40d4ae9e3c5e706d7f8544177` (U1P-R3 product commit; refrozen 2026-10-08 by `V1_COHORT_REFREEZE_RECORD.md`) |
| CI run | `37828673549` (#85, attempt 1, 10 of 10) |
| GitHub artifact id | `11573661113` |
| Installer SHA-256 the participant actually ran | (the frozen cohort bytes are `9a51e86aa5c571e43a8e1598ca64efb9c47d85e7c826e2cfa4a2faa8f3c87d93`, 3,896,257 bytes; the container holding them is 5,544,133 bytes `330e83af…`, which is **not** the installer size) |
| Matches frozen artifact | YES / NO — a NO is a §36 early-stop trigger |
| Machine / OS the participant used | band only, no hostname |
| Install friction and security warning (T0 evidence) | |

## Task outcomes (§23)

| Task | Outcome | First click | Path | Wrong turns | Backtracks | Stalls | Help used | Intervention |
|---|---|---|---|---|---|---|---|---|
| T0 Install / launch | | | | | | | | |
| T1 Analyze | | | | | | | | |
| T2 Evidence improvement | | | | | | | | |
| T3 Compare | | | | | | | | |
| T4 Release readiness | | | | | | | | |
| T5 Gate semantics | | — | — | — | — | — | — | |
| T6 Bundle | | | | | | | | |
| T7 History | | | | | | | | |
| T8 Support / Diagnostics | | | | | | | | |

## Metrics from this session

| Metric | Value | Basis |
|---|---|---|
| M1 Import activation | YES / NO / not eligible | T1 unassisted |
| M2 TTFV | `nnn` s / `NOT_MEASURED` | start and end timestamps actually captured |
| `INTERVENTION_BEFORE_FIRST_VALUE` | YES / NO | |
| M3 Compare completion | YES / NO / excluded (no valid pair) | T3 |
| M4 Gate comprehension | PASS / FAIL, with per-concept misses A–E | T5 rubric |
| M5 New information | YES / NO, with the concrete fact | participant's own words |
| M6 Return intent | YES / MAYBE / NO | fixed question |

## Misunderstandings (§26)

| Code | Meaning observed | Severity (Critical/High/Medium/Low) | Evidence line |
|---|---|---|---|
| | | | |

M18+ only when genuinely not representable, and then record why.

## Critical watches (§27)

| Code | Observed | Report |
|---|---|---|
| C1–C8 | | `1 / N observed` |

## Verbatim quotes (§28)

```text
QUOTE (verbatim):
```

Separate interpretation from observation, always:

```text
MODERATOR NOTE — NOT VERBATIM QUOTE:
INTERPRETATION:
```

## Observations

## Product findings (§31)

| ID | Classification (PRODUCT / RESEARCH_HARNESS / ENVIRONMENT / UNSUPPORTED_COHORT / KNOWN_LIMITATION / PREFERENCE / FEATURE_REQUEST / OBSERVATION) | Severity S0–S4 | What happened | Reproducible on the frozen build? |
|---|---|---|---|---|

## Participant recommendations

## Moderator interpretation (clearly separated from everything above)

## Privacy check (§50)

- [ ] no real name, email, employer or customer identity
- [ ] no private repository, Git remote or absolute private path
- [ ] no device serial or customer/device identifier
- [ ] no firmware bytes and no MAP content
- [ ] no confidential release information
- [ ] no recording URL or private recording location
- [ ] raw material retained outside Git in `04_sessions_raw`, not committed
