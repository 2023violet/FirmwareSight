---
title: "Project Handoff"
doc_id: "FS-AI-004"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-29"
---

# Handoff — FirmwareSight v0.6.0 / P0 closed PASS / P1-A0 complete including its correctness closure

## Purpose

P0 is finished, promoted and **frozen**: `P0 = PASS`, `baseline_version = 0.6.0`, and no further P0
closure or promotion prompt will be written. **One** architect-authorized Pre-G1 slice was built on
top of it, `P1-A0 Real Artifact Intake`, permitted by ADR-0025, which supersedes a single clause of
ADR-0020 and nothing else; it and its correctness closure are both finished. This file's job is to
state that boundary precisely: P1-A0 ends at P1-A0, and the closure round stayed inside it.

If you were sent here to "continue P0", the correct reply is that P0 is closed. If you were sent here
to continue past P1-A0 - P1-A1, Compare, Gate, Bundle - the correct reply is that no such work is
authorized: V0 Batch A `>= 4` eligible external sessions plus an interim architect review, then a new
prompt, is the precondition.

## Baseline and HEADs

```text
Baseline                       v0.6.0 — FirmwareSight_Project_Baseline_v0.6.0 (P0 Technical Foundation Baseline)
Engineering-validated HEAD     1cd6309a09313a0a900a83cb12e054e1a3d7c5e3 — the tree the gate measured
Architect-reviewed HEAD        5e58f778aad35f33188b96d0b8873401a31ccc3c — governance/audit/evidence docs only
Promotion commit               738ae78e00e682a5f82f679ea167304a558af864 — on origin/main; 23 files, no production source
Consistency commit             7d2f38a046ea7e036f95adf7ea8c5d0c9be5c9ae — on origin/main; BASELINE.yaml duplicate-key and stale-field closure
Remote CI Run #3               36399805005  on 1cd6309  success, 7 of 7 jobs — engineering closure
Remote CI Run #4               36402637251  on 5e58f77  success, 7 of 7 jobs — reviewed-HEAD revalidation
Remote CI Run #5               36416146281  on 738ae78  success, 7 of 7 jobs — promotion-commit revalidation
Remote CI Run #6               36419864513  on 7d2f38a  success, 7 of 7 jobs — last remote fact of the P0 chain
Design-token commit            ecd1c878a6efebd93338ee7777a600b49a9baf84 — on origin/main; tokens v0.2.1, generator, tokens.css, App.module.css, BASELINE.yaml
Design tokens                  v0.2.1 (border.width.hairline added; P0's documented 1px gap closed)
P1-A0 governance commit        311f9fce62c4dc207f9daab2c9843bdf5bb33592 — on origin/main; ADR-0025 scope recorded in the .ai/ files
P1-A0 implementation commit    29601735a2eff2c9e3677ef1dfa88cf752484694 — on origin/main; real artifact intake and the Analyze summary
Remote CI Run #7               36431884747  on ecd1c87  success, 7 of 7 jobs — design-token commit
Remote CI Run #8               36439949352  on 311f9fc  success, 7 of 7 jobs — governance commit
Remote CI Run #9               36499759371  on 2960173  success, 7 of 7 jobs — the intake commit, and this round's §1 gate fact
Correctness-closure commit     local and unpushed as this file is written; the owner pushes, and prompt §23 keeps this task's own CI result as external evidence rather than writing a run number back into its own commit
```

Run #1 (`36360310447`, `f9b8ccb`) and Run #2 (`36378384225`, `ebda52d`) concluded `failure` and stay
published as history in `P0_CI_REPORT.md`. Read any run with
`gh run view <id> --repo 2023violet/FirmwareSight`, not from this repository's reports.

## Gate state as promoted

```text
G0      PASS
V0      ACTIVE EXTERNAL VALIDATION / WAITING FOR REAL PARTICIPANTS — 0 / 8 (Batch A 0 / 4–5)
P0      PASS — frozen at v0.6.0
Formal G1   NOT CLAIMED — still V0 PASS + P0 PASS; ADR-0025 did not move the gate
Pre-G1  P1-A0 REAL ARTIFACT INTAKE — AUTHORIZED (ADR-0025), ON origin/main, GREEN ON RUN #9.
        ITS EVIDENCE-IDENTITY / PERSISTENCE CORRECTNESS CLOSURE IS IMPLEMENTED, TESTED AND COMMITTED
        LOCALLY, AND WAITS ON THE OWNER'S PUSH FOR REMOTE CI
P1      NOT PASS / NOT CLOSED
P1-A1   NOT AUTHORIZED — V0 Batch A >= 4 sessions + interim review + a new prompt
P2 / P3 / P4   NOT AUTHORIZED
```

## What the code does and how it is proven

- 142 Rust tests and 31 UI tests on one gate: `python scripts/check.py`, which CI calls unchanged —
  14 steps on a tree that already has the built frontend, 16 when it builds that too, plus 3 under
  `--only core-smoke`. The v0.6.0 promotion measured 104 / 19; the P1-A0 intake round and its
  correctness closure account for every test after those.
- Four Phase-0 library crates plus CLI and desktop apps. Core is headless and synchronous; no Tauri,
  rusqlite, Tokio or `object::*` type crosses out of it.
- Real ARM ELF/MAP fixtures with recorded provenance and hash-first tests, so no test needs
  `arm-none-eabi-gcc`.
- Deterministic CLI JSON, memory accounting reproduced by hand from `readelf`, a typed ts-rs IPC
  boundary whose drift is checked by regeneration plus `git diff --exit-code`.
- SQLite via `rusqlite + bundled`, migrations, transactional import, **schema version 2** — migration
  `0002` rebuilds `evidence` on `(build_id, id)` because version 1 contradicted `04_TECH/15` §4.
- A 512 MiB input guard measured from both sides of the boundary; Core/CLI/Desktop parity on the same
  bytes; the desktop window opened and driven for real on the shipping configuration.
- P1-A0 adds the intake path: a native dialog opens Rust-side, the selection is held behind an opaque
  session-local id, and the chosen ELF - optionally with a GNU ld MAP - runs through the same validated
  Analyze summary. **Stronger evidence is a second build, never an update to the first:** the MAP is
  sealed as a companion `ArtifactKind::Map` artifact, which changes `SnapshotId` through the identity
  model's existing optional map component, and each memory evidence item names the source type and
  locator its own rule actually used.

## Known gaps that promotion did not remove

```text
Peak RSS          NOT MEASURED — reason in P0_PERFORMANCE_REPORT.md; the P0 prompt accepted the gap
Fuzzing           NOT RUN
RustSec           RUSTSEC-2024-0429 (glib 0.18.5, unsound), RUSTSEC-2024-0370
                  (proc-macro-error 1.0.4, unmaintained) — ACCEPTED AS EXPLICIT P0 TRANSITIVE RISK,
                  five revisit triggers, no ADR because no architecture choice changed
Desktop smoke     one Windows 10 host, one WebView, at 100% scaling
Linker layouts    GNU ld MAP plus one dual-region ELF layout
Open design items capability labels show Core's enum words. The eight 1px borders that used to be
                  listed here are closed: `border.width.hairline` in design tokens v0.2.1
CI duplication    two jobs install the same ten apt lines on purpose
```

Never describe this tree as "zero vulnerabilities" or "security clean"; the two advisories above are
accepted, not fixed, and they are reachable only through the gtk-rs `0.18` line Tauri `2.12.0`
requires (`cargo update -p glib --precise 0.20.0` fails against `gtk = "^0.18"`).

## Read first

1. `README.md`
2. `PRODUCT_BASELINE.md` and `BASELINE.yaml`
3. `AGENTS.md`
4. `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`, `.ai/ACTIVE_TASK.md`
5. `10_AUDIT/SOURCE_PROMPTS/README.md` — which prompt authorizes what
6. `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md` — the promotion record
7. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md` — what is proven, and how
8. `P0_TECHNICAL_VALIDATION/P0_IMPLEMENTATION_LOG.md` — decisions already taken
9. `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` — the P0-chain runs, including the two failures
10. `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md`
11. `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md`, `P1_A0_EXIT_CHECKLIST.md` and
    `P1_A0_CORRECTNESS_SMOKE_REPORT.md` — the slice this handoff leaves behind, and how it was proven
12. `DESIGN.md` + `assets/design-tokens.json` (any UI work)
13. V0 summary only, never to be re-run from here: `V0_VALIDATION/README.md`,
    `V0_VALIDATION/V0_EXECUTION_PROVENANCE.md`, `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`

## Boundaries still in force

- The task was `P1-A0` and nothing wider, and its follow-up was a **correctness closure inside that
  same slice**, not a new one: the two defects the desktop smoke reported - a MAP that strengthened
  evidence in memory but not in the persisted identity, and evidence items that claimed MAP
  provenance they did not earn. Both are fixed, so neither is listed as open anywhere in this pack.
  A green P1-A0 is not P1 progress and must not be written as such.
- Intake security shape is fixed by ADR-0025 and `AGENTS.md` 7: dialogs open Rust-side, the selection
  is held behind an opaque session-local id, no generic filesystem/shell/network capability, no
  `read_file(path)` or `get_any_path` command, no full path in normal IPC or UI.
- No schema change and no migration: real artifacts get an explicit local/default project identity,
  schema version 2 keeps working, raw artifact bytes stay out of SQLite. The correctness closure fixed
  persistence **within** version 2 - `0003` was not created, and `NORMALIZATION_VERSION` was not
  bumped to hide the identity bug.
- Design: existing tokens plus `border.width.hairline` only. No drop zone, no dashed border, no
  decorative upload card, no new visual semantic — if one seems necessary, stop and report it.
- V0 evidence stays untouched and its count stays `0`; the resumed wording is a governance statement,
  not a result.
- P1-A0 and its correctness closure have both shipped from the coding side, so the only legitimate
  next moves are real V0 participants, or a new architect prompt for a further pre-G1 slice — and that
  prompt requires V0 Batch A `>= 4` eligible sessions plus an interim review first. Neither exists
  now, so `.ai/ACTIVE_TASK.md` names finished work rather than work to resume, and nothing is
  authorized to code from this file.
- Four Phase-0 library crates; a fifth needs architecture review plus an ADR.
- AGENTS.md §2: no silent baseline change — async-first Core, framework swaps, SQLx, HTTP client,
  wgpu, cloud/auth/telemetry, evidence classes, schema semantics, updater/signing model all need an
  ADR first.
- No Compare / Gate / Bundle product workflow. No installer, NSIS/MSI, AppImage/deb, notarization,
  Authenticode, GitHub Release, tag, or updater metadata: v0.6.0 is a baseline promotion, not a
  stable release. The repository has no tag convention and none was invented.
- `V0_VALIDATION/**` is research evidence and stays unmodified.

## Continuation rules

- Re-run the read-only Git preflight before writing, and report start HEADs again; do not trust the
  HEADs recorded in this file.
- Never clean, reset, stash, restore or delete to get a tidy tree. User work outranks cleanliness.
- Reports cite real commands and real output. Unmeasured stays `NOT MEASURED`; unrun stays `NOT RUN`.
- Do not weaken or delete a test to reach green, and do not edit a baseline document to make a
  historical failure read as a pass.
- Do not globally replace `version: 0.5.1` in historical evidence; only current baseline authority
  documents carry v0.6.0.
- Quote any `BASELINE.yaml` value containing ` #`: an unquoted `Run #2` truncates at the comment and
  the file still parses. Verify with `python -c "import yaml, …; yaml.safe_load(...)"` after editing.
- Read remote CI with `gh run view`. A run's ID, head SHA and per-job conclusion are external facts,
  and a report that describes them is a claim about a moment that has already moved.
- After any commit that touches baseline-controlled files, `SHA256SUMS` and `DIRECTORY_TREE.txt` are
  stale: regenerate both, in that order, and verify with `sha256sum -c` plus an independent checker.
