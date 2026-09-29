---
title: "FirmwareSight Project Baseline"
doc_id: "FS-ROOT-README"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-29"
---

# FirmwareSight v0.6.0

**Embedded Firmware Release Workbench**  
**Know exactly what ships.**

v0.6.0 is the **P0 Technical Foundation Baseline**: the first version of this repository that ships
source, and the version record of P0 closing `PASS`. It is built on `FirmwareSight_Project_Baseline_v0.5.1`
and does **not** redefine the product, the architecture, the gate definitions or the design contract.

```text
G0      PASS
V0      ACTIVE EXTERNAL VALIDATION — 0 / 8 eligible external sessions, waiting for real participants
        Active task since 2026-09-29: V0 Batch A recruitment and moderation (research, no product code)
P0      PASS — remote CI Runs #3, #4, #5 and #6, 7 of 7 jobs green on each; frozen at v0.6.0
G1      NOT CLAIMED — requires V0 PASS + P0 PASS; only the P0 half exists
Pre-G1  P1-A0 Real Artifact Intake — authorized by ADR-0025, green on Run #9, and CLOSED together with
        its evidence-identity / persistence correctness closure (bounded; it stopped at P1-A0)
P1      NOT PASS / NOT CLOSED; P1-A1, P2, P3 and P4 are NOT AUTHORIZED
```

`v0.6.0` does not mean: V0 passed, G1 passed, the product MVP is complete, Compare / Gate / Bundle
workflows exist, an installer or signed build is ready, user value is validated, or the tree is free of
vulnerabilities. See `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`.

## What v0.6.0 adds

A Rust workspace with exactly four Phase-0 library crates (`core`, `artifact`, `report`, `storage`),
the `fwsight` CLI, a thin Tauri 2 desktop shell with a React summary, real ARM ELF + GNU ld MAP fixtures
with recorded provenance, goldens, ts-rs-generated typed IPC, minimal SQLite at schema version 2, a
512 MiB input guard, and one verification gate that CI and a laptop run unchanged.

```
python scripts/check.py                     # 14 steps across rust / frontend / drift / deny
python scripts/check.py --only core-smoke   # the macOS-on-main core smoke CI runs separately
cargo run -q --bin fwsight -- analyze fixtures/elf/p0-dual-region/firmware.elf \
  --map fixtures/elf/p0-dual-region/firmware.map --json
```

142 Rust tests and 31 UI tests — 104 and 19 of them at the v0.6.0 promotion. Measurement status of
every claim is in `P0_TECHNICAL_VALIDATION/`, starting from `P0_EXIT_CHECKLIST.md`, and for the
Pre-G1 slice in `P1_A0_VALIDATION/`.

## What v0.6.0 does not change

Product scope stays as frozen at v0.5.0. There is no fifth product verb, no new artifact class and no
new gate semantics; the version records technical closure, not a product change. `PRODUCT_BASELINE.md`
carries the dated note saying so.

## Current V0 status

# INCOMPLETE — insufficient external sample

Formal eligible external participants completed:

`0 / 8 minimum`

No participant data is fabricated. V0 was re-sequenced as a non-blocking research track when P0 was
authorized, and it is now the **active task**: `V0_BATCH_A_EXTERNAL_VALIDATION`, activated 2026-09-29 to
recruit and moderate 4-5 real eligible sessions on the frozen `v0.1.0` prototype and then write a Batch A
interim review. Neither deferral nor resumption is completion, and P0's `PASS` does not move V0 by one
session. Activation wrote no evidence, because no participant had been met.

Therefore this baseline does **not** claim:
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

## P0 Technical Vertical Slice — closed PASS

The slice reached `PASS` on evidence, not on a wording change. Remote CI Run #3 (`36399805005`) executed
the engineering-validated HEAD `1cd6309` and concluded `success` with **7 of 7 jobs green**; Run #4
(`36402637251`) executed the architect-reviewed HEAD `5e58f77` — documentation only, no source moved —
and concluded `success` with **7 of 7 jobs green**; Run #5 (`36416146281`) executed the promotion commit
itself, `738ae78`, and concluded `success` with **7 of 7 jobs green** again, which is what makes the baseline record green
on its own commit; Run #6 (`36419864513`) executed its consistency-closure successor `7d2f38a` and
concluded `success` with **7 of 7 jobs green**, verifying that the baseline's own duplicate-key and
stale-field fix did not break the gate. All four were read back with `gh run view`, not from this file:

```
gh run view 36399805005 --repo 2023violet/FirmwareSight
gh run view 36402637251 --repo 2023violet/FirmwareSight
gh run view 36416146281 --repo 2023violet/FirmwareSight
gh run view 36419864513 --repo 2023violet/FirmwareSight
```

It took six runs to get there and the two failures are kept as history in
`P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`. Run #1 (`36360310447`, `f9b8ccb`) had four jobs red; each
cause was reproduced with a command before being fixed — a `.gitattributes` text policy so a checkout
cannot change a fixture's bytes, icon drift judged by decoded pixels rather than by encoder bytes, a
`deny.toml` that cargo-deny 0.20.2 actually parses, Linux prerequisites on the Ubuntu job, and the
macOS core smoke `05_ENGINEERING/06_CI_CD_BASELINE.md` requires. Run #2 (`36378384225`, `ebda52d`) came
back six of seven green with one job missing a provisioning step its sibling job had.

The authorized desktop window launch found a sixth defect no test had reached: a second artifact raised
`UNIQUE constraint failed: evidence.id`, because `evidence` was keyed on `id` alone while
`04_TECH/15` §4 declares `Build 1─N Evidence`. Migration `0002` rebuilds the table on `(build_id, id)`;
schema version is 2.

`LOCAL PASS`, `CI PASS`, `NOT RUN` and `NOT MEASURED` stay different claims throughout the pack. What
promotion does not remove: peak RSS is `NOT MEASURED`, fuzzing is `NOT RUN`, and two RustSec advisories
(`RUSTSEC-2024-0429`, `RUSTSEC-2024-0370`) are **accepted as explicit P0 transitive risk** with five
revisit triggers — they are reachable only through the gtk-rs `0.18` line Tauri 2.12.0 requires, and no
compatible upgrade exists inside that pin.

## Read order

1. `README.md`
2. `PRODUCT_BASELINE.md`
3. `BASELINE.yaml`
4. `.ai/CURRENT_STATE.md`
5. `P0_TECHNICAL_VALIDATION/README.md`
6. `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`
7. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`
8. `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md`
9. `V0_VALIDATION/README.md`
10. `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`
11. `10_AUDIT/SOURCE_PROMPTS/README.md`
12. `.ai/ACTIVE_TASK.md` — currently `P1A0_REAL_ARTIFACT_INTAKE`

## Batch A recruitment-ready status

Current V0 Batch A status:

`WAITING FOR REAL PARTICIPANTS`

The recruitment/screening/moderator/scheduling package is located at:

`V0_VALIDATION/batch_a/recruitment_ready/`

Formal external sessions remain:

`0 / 4–5 Batch A target`

Per the authorized Batch A Prompt, v0.5.2 is intentionally withheld until real Batch A evidence exists.

## Prior baselines

`FirmwareSight_Project_Baseline_v0.5.1` was a **V0 Workflow Prototype Validation execution patch** built
only on the unique `v0.5.0`: V0 authority takeover, the seven-screen clickable static prototype, the
coherent single fixture narrative, the explicit MAP STATE A→B transition, the Unknown Dependency
declaration flow, the Review acceptance audit flow, simulated signature-block recovery, conditional
Bundle creation, parse-failure recovery with Last Good Artifact preservation, the
moderator/screening/task/interview/privacy protocol, the session/metrics/payment/toolchain records, and
the automated internal state-machine and scope dry run. Its status record is the v0.5.1 row in
`CHANGELOG.md`; P0 created no new product or architecture decision, it built the first production code
against the existing ones.

P0 was not authorized by that V0 execution. It was authorized separately, by the prompt preserved in
`10_AUDIT/SOURCE_PROMPTS/README.md`.

## Next work

`active_task: V0_BATCH_A_EXTERNAL_VALIDATION`, activated by architect prompt on 2026-09-29. It is a
**research / evidence task, not a coding one**: recruit and screen 4-5 real eligible embedded
participants, moderate the frozen T1-T10 script against the `v0.1.0` clickable prototype, write one
session record per real person, then produce the Batch A interim review and stop for the architect.
**No product code is writable under it** - `crates/**`, `apps/**`, `schemas/**`, `migrations/**`, either
manifest or lockfile, `.github/**`, the design tokens, the product UI and the prototype are all out of
scope, with no exception, and no participant, quote, outcome, timing or count may be invented.

The previous task, `P1A0_REAL_ARTIFACT_INTAKE`, authorized by
`09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md`, **is finished**, together with the narrow
correctness closure that fixed the two defects its own desktop smoke reported. That ADR supersedes one
clause of ADR-0020: after `P0 PASS`, the architect may
authorize a limited, reversible, low-coupling Pre-G1 Analyze slice instead of waiting for V0. The
authorized slice was real artifact intake through a native dialog plus the already-validated Analyze
summary — and it stopped there. No engineering task is authorized from this file.
`G1 = V0 PASS + P0 PASS` is unchanged, `P1` is not closed, `P1-A1` needs V0 Batch A `>= 4` eligible
external sessions plus an interim architect review and a new prompt, and `P2`/`P3`/`P4` remain
unauthorized. V0 still needs the human half: recruit eligible participants, Batch A 4–5, interim review,
Batch B 4–5, optional Batch C to 12–15, record actual n/N, issue the V0 Gate recommendation.
