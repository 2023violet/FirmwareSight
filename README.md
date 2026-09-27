---
title: "FirmwareSight Project Baseline"
doc_id: "FS-ROOT-README"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-28"
---

# FirmwareSight v0.5.1

**Embedded Firmware Release Workbench**  
**Know exactly what ships.**

v0.5.1 is a **V0 Workflow Prototype Validation execution patch** built only on the unique `FirmwareSight_Project_Baseline_v0.5.0`.

It does not redefine the product or architecture.

## What is newly executed

- V0 authority takeover;
- seven-screen clickable high-fidelity static prototype;
- coherent single fixture narrative;
- explicit MAP STATE A→B transition;
- Unknown Dependency declaration flow;
- Review acceptance audit flow;
- simulated signature-block recovery;
- conditional Bundle creation;
- parse-failure recovery with Last Good Artifact preservation;
- moderator/screening/task/interview/privacy protocol;
- session/metrics/misunderstanding/payment/toolchain records;
- automated internal state-machine and scope dry run.

## Current V0 status

# INCOMPLETE — insufficient external sample

Formal eligible external participants completed:

`0 / 8 minimum`

No participant data is fabricated.

Therefore v0.5.1 does **not** claim:
- V0 PASS;
- V0 Conditional PASS;
- V0 FAIL based on users;
- G1;
- Product MVP authorization.

## Prototype

Open:

`V0_VALIDATION/prototype/index.html`

No dependency installation is required.

## State model

```text
STATE A
ELF ✓
MAP not provided
Git not linked
Sections available
Symbols unavailable
MAP-dependent Gate findings UNKNOWN

      ↓ explicit Add MAP

STATE B
ELF ✓
MAP ✓ firmware.map
1,284 symbols
Dependencies enabled
MAP-dependent Gate findings re-evaluated

      ↓ invalid candidate replacement

STATE C
Parse Failure
Last Good Artifact preserved

      ↓ recover

STATE D
Bundle & History
```

## Next work

Continue **V0 only**:
- recruit real eligible participants;
- Batch A 4–5;
- interim review;
- Batch B 4–5;
- optional Batch C to 12–15;
- record actual n/N;
- issue V0 Gate recommendation.

P0 was not authorized by this V0 execution. P0 was authorized separately, by the prompt preserved in
`10_AUDIT/SOURCE_PROMPTS/README.md`, and has since been executed — see the P0 section below.

## P0 Technical Vertical Slice — executed, CONDITIONAL_PASS (LOCAL)

The slice exists as source, not as a plan: a Rust workspace with exactly four Phase-0 library crates
(`core`, `artifact`, `report`, `storage`), the `fwsight` CLI, a thin Tauri 2 desktop shell with a
React summary, real ARM ELF + GNU ld MAP fixtures with recorded provenance, goldens, ts-rs-generated
typed IPC, minimal SQLite, a 512 MiB input guard, and one verification gate that CI and a laptop run
unchanged.

```
python scripts/check.py        # 14 steps; the only step not run is cargo-deny, recorded as SKIPPED
cargo run -q --bin fwsight -- analyze fixtures/elf/p0-dual-region/firmware.elf \
  --map fixtures/elf/p0-dual-region/firmware.map --json
```

Proof and limits live in `P0_TECHNICAL_VALIDATION/`:

- `P0_EXIT_CHECKLIST.md` — each prompt criterion, the command that exercised it, and what it showed;
- `P0_KNOWN_LIMITATIONS.md` — what is deliberately absent, including that no window was ever opened;
- `P0_IMPLEMENTATION_LOG.md` — every non-trivial engineering decision with its evidence;
- `P0_EXECUTION_PROVENANCE.md` — start HEADs, environment, prompt SHA-256.

Status is **`EXECUTED — CONDITIONAL_PASS (LOCAL)`**, and the conditions are outside this machine:
CI has never run and `cargo deny` has never run, because there is no push authorization here and the
tool was not installed. `LOCAL PASS` and `CI PASS` are kept as different claims throughout the pack.

The baseline therefore stays **v0.5.1**. `v0.6.0` is reserved for an unconditional P0 `PASS`, G1 is
not claimed (V0 still has `0 / 8` external sessions), and P1 requires its own authorization prompt,
which has not been issued.

## Read order

1. `README.md`
2. `PRODUCT_BASELINE.md`
3. `V0_VALIDATION/README.md`
4. `V0_VALIDATION/V0_TAKEOVER_REPORT.md`
5. `V0_VALIDATION/V0_PLAN.md`
6. `V0_VALIDATION/prototype/FIXTURE_NARRATIVE.md`
7. `V0_VALIDATION/internal/DRY_RUN_REPORT.md`
8. `V0_VALIDATION/deliverables/V0_VALIDATION_REPORT.md`
9. `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`
10. `.ai/ACTIVE_TASK.md`
11. `P0_TECHNICAL_VALIDATION/README.md`
12. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`


## Batch A recruitment-ready status

Current V0 Batch A status:

`WAITING FOR REAL PARTICIPANTS`

The recruitment/screening/moderator/scheduling package is located at:

`V0_VALIDATION/batch_a/recruitment_ready/`

Formal external sessions remain:

`0 / 4–5 Batch A target`

Per the authorized Batch A Prompt, v0.5.2 is intentionally withheld until real Batch A evidence exists.
