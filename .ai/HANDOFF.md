---
title: "Project Handoff"
doc_id: "FS-AI-004"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-29"
---

# Handoff — FirmwareSight v0.6.0 / P0 closed PASS / G1 PASS on the P0 basis / active task P1 Analyze details

## Purpose

P0 is finished, promoted and **frozen**: `P0 = PASS`, `baseline_version = 0.6.0`, and no further P0
closure or promotion prompt will be written. The one bounded pre-G1 slice, `P1-A0 Real Artifact Intake`,
is complete together with its correctness closure.

The direction changed on 2026-09-29 by `ADR-0026-open-source-mvp-first-delivery.md`: FirmwareSight builds
its local MVP first, `G1 = P0 PASS` for that delivery, and V0 is a **non-blocking** user-feedback track
rather than a precondition. The live task is therefore engineering again - **`P1_ANALYZE_DETAILS`**, the
rest of the Analyze verb: bounded Sections, Symbols and Evidence queries, top contributors, the Evidence
Inspector, and the `bytes / KiB` switch that US-001 requires. This file's job is to keep that boundary
precise: P1 ends at P1, and P2 Compare needs its own architect prompt.

Two wrong turns to refuse. If you were sent here to "continue P0", P0 is closed. If you were sent here to
continue past P1 into Compare, Gate, Bundle, an installer or anything cloud-shaped, no such work is
authorized - and note that ADR-0026 removed the *research* gate, not the requirement that each stage be
authorized, so "governance got easier" is not a licence to widen scope.

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
Correctness-closure commit     ace6fbe12fe90d2886d8128aeda3879c52ad539c — on origin/main; MAP companion identity, basis-aware provenance, deterministic primary-artifact query, from_map removal
Remote CI Run #10              36515470263  on ace6fbe  success, 7 of 7 jobs — the closure commit, recorded here after the owner pushed rather than inside its own commit (§23's anti-recursion rule)
Governance successor           3b59585e3843336b8237449e72f147ed35cf672d — on origin/main; documentation and integrity only, no product source
Remote CI Run #11              36516209283  on 3b59585  success, 7 of 7 jobs - the HEAD the V0 Batch A activation round started from, verified before any file was written
V0 Batch A activation          2026-09-29 by architect prompt; `active_task` moved from P1A0_REAL_ARTIFACT_INTAKE to V0_BATCH_A_EXTERNAL_VALIDATION. Research execution: no product code writable, and zero participant evidence existed so zero was written
Batch A activation commit      be09c65f451dc1b625c6bf597929a7ce93f5c509 — on origin/main; Run #12 36520562718 success, 7 of 7 jobs — the verified starting point of the P1 round
ADR-0026 governance reset      2026-09-29, same day: V0 becomes NON_BLOCKING_USER_FEEDBACK_TRACK, G1 is re-based on P0 PASS, `active_task` moves again to P1_ANALYZE_DETAILS. The Batch A activation above is real history and stands as a record; only its sequencing authority was superseded hours later
Withdrawn prompt               FirmwareSight_V0_Batch_A_Price_Anchor_Authorization_Participant_Acquisition_Pack_EXECUTION_PROMPT_v1.0 — WITHDRAWN_BY_ARCHITECT, never executed, no price anchors written, no recruitment pack produced, V0 sample still 0/8
```

Run #1 (`36360310447`, `f9b8ccb`) and Run #2 (`36378384225`, `ebda52d`) concluded `failure` and stay
published as history in `P0_CI_REPORT.md`. Read any run with
`gh run view <id> --repo 2023violet/FirmwareSight`, not from this repository's reports.

## Gate state

```text
G0      PASS
P0      PASS — frozen at v0.6.0
G1      PASS — basis: P0 PASS, per ADR-0026 (2026-09-29). Before that date G1 was `V0 PASS + P0 PASS`
        and NOT CLAIMED; P0's promotion pack and the P1-A0 pack record the older formula, and they are
        not rewritten
V0      NON_BLOCKING_USER_FEEDBACK_TRACK — 0 / 8 eligible sessions, an honest zero that gates no
        P-stage. The v0.1.0 prototype and protocol stay frozen so a later feedback round is comparable
Pre-G1  P1-A0 REAL ARTIFACT INTAKE — AUTHORIZED (ADR-0025), ON origin/main, GREEN ON RUN #9.
        ITS EVIDENCE-IDENTITY / PERSISTENCE CORRECTNESS CLOSURE IS IMPLEMENTED, TESTED, AND ALSO ON
        origin/main (`ace6fbe`), GREEN ON RUN #10
P1      IN_PROGRESS — P1_ANALYZE_DETAILS. PASS is earned by the acceptance list, not declared in advance
P2 / P3 / P4   NOT STARTED — P2 becomes the next authorizable stage when P1 is verifiably complete;
        ADR-0026 removed the research gate, not the requirement of its own architect prompt
Pricing / commercial research   DEFERRED_POST_MVP; the price-anchor prompt was withdrawn unexecuted
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
Evidence model  UNTESTED with real users. V0 is now a non-blocking track at 0 / 8, so whether people
  comprehension   distinguish Unknown from PASS, or read a Review correctly, is carried forward as risk
                  rather than measured. Recorded in ADR-0026's Consequences
Known engineering `scripts/update_goldens.py` no longer reproduces the committed goldens' key order;
  items           `ElfProgramHeader` names a source the ELF parser never reads; the frozen release-manifest
                  schema describes one artifact per build; a release build without
                  `--features custom-protocol` shows a WebView network error. Each with its reason in
                  `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` §9.7
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
5. `10_AUDIT/SOURCE_PROMPTS/README.md` — which prompt authorizes what, and which was withdrawn
6. `09_ADR/ADR-0026-open-source-mvp-first-delivery.md` — why G1 now rests on P0 and what it did NOT relax
7. `09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md` — the P1-A0 slice and its safety terms
8. `04_TECH/14_IPC_DATA_CONTRACTS.md` and `04_TECH/15_STORAGE_DATABASE_BASELINE.md` — the bounded-query
   and schema rules the current task is written against
9. `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md` — the promotion record
10. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md` — what is proven, and how
11. `P0_TECHNICAL_VALIDATION/P0_IMPLEMENTATION_LOG.md` — decisions already taken
12. `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` — the P0-chain runs, including the two failures
13. `P0_TECHNICAL_VALIDATION/P0_KNOWN_LIMITATIONS.md`
14. `P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md`, `P1_A0_EXIT_CHECKLIST.md` and
    `P1_A0_CORRECTNESS_SMOKE_REPORT.md` — the completed slice, and how it was proven
15. `P1_VALIDATION/` — this round's execution report, exit checklist, design checklist and smoke report
16. `DESIGN.md` + `assets/design-tokens.json` (any UI work)
17. The V0 track, for a later non-blocking feedback round: `V0_VALIDATION/README.md`, `V0_PLAN.md`,
    `V0_VALIDATION/protocol/**`, `V0_VALIDATION/sessions/TEMPLATE.md`,
    `V0_VALIDATION/batch_a/**` including `recruitment_ready/**`, and
    `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`

## Boundaries still in force

- The completed slice stays closed: `P1-A0` was real artifact intake plus the Analyze summary, and its
  follow-up was a **correctness closure inside that same slice** - the MAP that strengthened evidence in
  memory but not in the persisted identity, and the evidence items that claimed MAP provenance they did
  not earn. Both are fixed and neither is listed as open. A green P1-A0 was not P1 progress, and the
  current P1 task does not retroactively make it so.
- Intake security shape is fixed by ADR-0025 and `AGENTS.md` 7, and ADR-0026 did not relax it: dialogs
  open Rust-side, the selection is held behind an opaque session-local id, no generic
  filesystem/shell/network capability, no `read_file(path)` or `get_any_path` command, no full path in
  normal IPC or UI. The same discipline governs the new query commands: three use-case commands, no
  `run_sql` / `read_table` / `query_any` / `get_database`, no raw SQL and no `rusqlite` type across IPC.
- No schema change and no migration: real artifacts get an explicit local/default project identity,
  schema version 2 keeps working, raw artifact bytes stay out of SQLite, `NORMALIZATION_VERSION` stays
  `p0-normalize-1`. The P1 details are read out of the tables that already exist; a measured need for an
  index stops the round and comes back as a proposal rather than arriving as migration `0003`.
- Payload discipline is now part of the contract: default page size 100, hard maximum 500, enforced in
  Rust, with `rows` / total / next offset returned. The whole symbol table never crosses IPC.
- Details follow the last-good snapshot. A failed later attempt keeps the summary and the details pointed
  at the snapshot actually shown, and a detail-query error belongs to the details area only.
- Presentation-only unit switch: bytes and KiB at 1024 bytes per KiB, applied to sizes and to nothing
  else - addresses, offsets, hashes, counts, ordinals and snapshot ids never convert, and the choice
  lives in UI state, never in SQLite, project policy or a settings page.
- Design: existing tokens only, `border.width.hairline` included. No new visual semantic, no shadow on
  tables or panels, no gradient, no glass, no purple, no colour-only state, no dark theme. If a value
  genuinely has no token, stop and report rather than editing a frozen asset.
- `ADR-0026` relaxed **sequencing**, nothing else. It removed the V0 precondition and re-based G1 on P0;
  it did not authorize cloud, accounts, telemetry, AI, updater, licensing work, or a stage beyond the one
  a prompt names. Each stage still needs its own architect prompt.
- Four Phase-0 library crates; a fifth needs architecture review plus an ADR.
- AGENTS.md §2: no silent baseline change — async-first Core, framework swaps, SQLx, HTTP client,
  wgpu, cloud/auth/telemetry, evidence classes, schema semantics, updater/signing model all need an
  ADR first.
- No Compare / Gate / Bundle product workflow **yet**: they are the authorized shape of P2-P4, and this
  round must not start them. Still out of scope entirely - installer, NSIS/MSI, AppImage/deb,
  notarization, Authenticode, GitHub Release, tag, updater metadata: v0.6.0 is a baseline promotion, not
  a stable release, and the repository has no tag convention and none was invented.
- `V0_VALIDATION/**` is research evidence and stays where it is: not deleted by the reset, not edited to
  imply a session that did not happen, and not a gate any more. Its `0 / 8` is an honest zero.

## Continuation rules

- If the V0 feedback track is ever resumed, it resumes under its own rules, not these: one real session at
  a time, processing exactly the evidence supplied; `V0_VALIDATION/batch_a/recruitment_ready/BATCH_A_EVIDENCE_INTEGRITY_CHECKLIST.md`
  must pass in full before any `PA-00X.md` counts, and a session failing a mandatory item is recorded
  `DISCOVERY_ONLY` or `EXCLUDED_FROM_FORMAL_N` with its reason rather than counted silently. No synthetic
  participant, no fabricated quote, outcome, timing or count - that rule does not expire with the gate.
- Engineering rounds resume by finishing the authorized task and stopping; they do not continue into the
  next stage because it is now sequenced more loosely. `P1` ending does not authorize `P2`.

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
