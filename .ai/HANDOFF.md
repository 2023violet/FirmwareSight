---
title: "Project Handoff"
doc_id: "FS-AI-004"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-28"
---

# Handoff — FirmwareSight v0.6.0 / P0 closed PASS / active task P1-A0 (bounded pre-G1 slice)

## Purpose

P0 is finished, promoted and **frozen**: `P0 = PASS`, `baseline_version = 0.6.0`, and no further P0
closure or promotion prompt will be written. What is live now is **one** architect-authorized Pre-G1
slice, `P1-A0 Real Artifact Intake`, permitted by ADR-0025, which supersedes a single clause of
ADR-0020 and nothing else. This file's job is to state that boundary precisely: P1-A0 ends at P1-A0.

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
Remote CI Run #6               36419864513  on 7d2f38a  success, 7 of 7 jobs — the remote fact recorded before P1-A0 began
Design-token commit              ecd1c878a6efebd93338ee7777a600b49a9baf84 — on origin/main; tokens v0.2.1, generator, tokens.css, App.module.css, BASELINE.yaml
Design tokens                  v0.2.1 (border.width.hairline added; P0's documented 1px gap closed)
HEAD at this round's preflight was ecd1c87; this round added the governance commit 311f9fc (now on origin/main) and the P1-A0 implementation commit (local). Run #6 on 7d2f38a is the last remote run this file records — prompt §28 keeps this task's own CI results as external evidence rather than writing them back into its commits
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
Pre-G1  P1-A0 REAL ARTIFACT INTAKE — AUTHORIZED (ADR-0025) AND IMPLEMENTED LOCALLY, UNPUSHED
P1      NOT PASS / NOT CLOSED
P1-A1   NOT AUTHORIZED — V0 Batch A >= 4 sessions + interim review + a new prompt
P2 / P3 / P4   NOT AUTHORIZED
```

## What the code does and how it is proven

- 104 Rust tests, 19 UI tests, one gate: `python scripts/check.py`, which CI calls unchanged —
  14 steps on a tree that already has the built frontend, 16 when it builds that too, plus 3 under
  `--only core-smoke`.
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

## Known gaps that promotion did not remove

```text
Peak RSS          NOT MEASURED — reason in P0_PERFORMANCE_REPORT.md; the P0 prompt accepted the gap
Fuzzing           NOT RUN
RustSec           RUSTSEC-2024-0429 (glib 0.18.5, unsound), RUSTSEC-2024-0370
                  (proc-macro-error 1.0.4, unmaintained) — ACCEPTED AS EXPLICIT P0 TRANSITIVE RISK,
                  five revisit triggers, no ADR because no architecture choice changed
Desktop smoke     one Windows 10 host, one WebView, at 100% scaling
Linker layouts    GNU ld MAP plus one dual-region ELF layout
Open design items capability labels show Core's enum words; eight 1px borders have no token, which
                  needs a frozen-asset assets/design-tokens.json bump
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
9. `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` — all five runs, including the two failures
10. `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md`
11. `DESIGN.md` + `assets/design-tokens.json` (any UI work)
12. V0 summary only, never to be re-run from here: `V0_VALIDATION/README.md`,
    `V0_VALIDATION/V0_EXECUTION_PROVENANCE.md`, `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`

## Boundaries still in force

- The task is `P1-A0` and nothing wider. Its own stop condition is completion of that slice: real
  artifact intake through a **native dialog**, optional GNU ld MAP, reuse of the validated Analyze
  summary. A green P1-A0 is not P1 progress and must not be written as such.
- Intake security shape is fixed by ADR-0025 and `AGENTS.md` 7: dialogs open Rust-side, the selection
  is held behind an opaque session-local id, no generic filesystem/shell/network capability, no
  `read_file(path)` or `get_any_path` command, no full path in normal IPC or UI.
- No schema change and no migration: real artifacts get an explicit local/default project identity,
  schema version 2 keeps working, raw artifact bytes stay out of SQLite.
- Design: existing tokens plus `border.width.hairline` only. No drop zone, no dashed border, no
  decorative upload card, no new visual semantic — if one seems necessary, stop and report it.
- V0 evidence stays untouched and its count stays `0`; the resumed wording is a governance statement,
  not a result.
- After P1-A0 ships, the only legitimate next moves are real V0 participants, or a new architect
  prompt for a further pre-G1 slice — and that prompt requires V0 Batch A `>= 4` eligible sessions plus
  an interim review first. Neither exists at the moment P1-A0 closes, so `.ai/ACTIVE_TASK.md` should
  then read `NONE` again until one does.
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
