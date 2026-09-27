---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# ACTIVE TASK

```text
NONE
```

The P0 Technical Vertical Slice - the task this file carried - reached its stop condition on
2026-09-28 with final status `CONDITIONAL_PASS (LOCAL)`. No new business function may be created
from here; `AGENTS.md` 1 forbids inventing work while this file reads `NONE`.

## Awaiting authorization (not tasks until authorized)

| Ref | Action | Why it is not being done here |
| --- | --- | --- |
| C1 | Let `.github/workflows/p0-check.yml` run on `main` | requires a push, and none was authorized |
| C2 | Let the `deny` job execute `cargo deny` | same push, or explicit permission to install the tool locally |
| C3 | Launch `firmwaresight-desktop` once and confirm the summary screen on a real display | starting a GUI in the user's session needs a yes |
| - | Promote P0 from `CONDITIONAL_PASS` to `PASS` and cut `v0.6.0` | only valid after C1-C3 close |
| - | P1 | requires its own execution prompt |

## What P0 proved

- One headless Core produces the same facts for the CLI and the Desktop, asserted field by field
  against a committed golden.
- ADR-0021's two budgets are demonstrable on real linker output, and the evidence ladder changes
  the *strength* of the claim when the MAP is withheld while the numbers stay the same.
- The 512 MiB guard refuses an oversized artifact before allocating for it (measured: 7.7 ms, no
  hash time), and the whole slice stays within the frozen architecture with no ADR required.

Evidence: `P0_TECHNICAL_VALIDATION/` - 19 files: the 17 documents the prompt names, plus
`P0_EXECUTION_PROVENANCE.md` from the takeover and `P0_DESIGN_CHECKLIST.md` required by AGENTS.md 11.

## Durable facts preserved by this change

V0 remains `DEFERRED / NOT YET EVIDENCE-VALIDATED — NON-BLOCKING RESEARCH TRACK`.

Formal eligible external participants completed `0 / 8 minimum`; Batch A target `0 / 4–5`. The V0
blocker is the absence of real human participants, not a technical failure. `V0_VALIDATION/` and
every Batch A recruitment artifact remain intact and unmodified; P0 did not touch them.

## Gate status

```text
Formal G1: NOT CLAIMED  (g1_requires V0_PASS + P0_PASS; V0 still unvalidated)
P1:        NOT AUTHORIZED
```

## Version gate

Do not generate `FirmwareSight_Project_Baseline_v0.6.0` unless P0 final status is `PASS` with real
engineering evidence. It is `CONDITIONAL_PASS`, so `baseline_version` stays `0.5.1`.
`CONDITIONAL_PASS`, `FAIL` and `BLOCKED` must not be renamed into a PASS baseline.

## Not authorized

- P1–P4 Product MVP implementation
- E1 / E2 / E3 / GX candidates
- Cloud / account / auth / telemetry / AI
- SBOM / CVE / OTA / flashing / HIL / updater / wgpu / SQLx
- A fifth Phase-0 library crate without architecture review plus ADR
- Synthetic Persona substitution for V0 evidence
- Claiming V0 PASS, G1 PASS, or that product value has been user-validated
