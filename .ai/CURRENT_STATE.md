---
title: "Current Project State"
doc_id: "FS-AI-002"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-30"
---

# Current State

- Date: 2026-10-01
- Baseline: **v0.6.0 — P0 Technical Foundation Baseline**
- Active task: **`G2_ENGINEERING_CLOSURE_AUDIT`** — local verdict **G2 LOCAL PASS / READY FOR REMOTE
  CLOSURE**, opened 2026-10-01 by *FirmwareSight — G2 Product MVP Engineering Closure Audit, Execution
  Prompt v1.0 — Architect Reviewed* (file, SHA-256 `3c6ab83e…2bac9`, archived) and its inline *Storage Path
  Semantics Clarification Addendum v1.0*. Audited tree `e35cfe7` (`055b54e` + the G2-F1 test-only fix,
  Run `36906482900` attempt 1, 7 of 7); 769 Rust / 155 UI / `check.py` 15/15 / clean worktree 17/17; the
  whole chain on both surfaces with byte-identical parity; evidence in `G2_VALIDATION/`. The pointer
  returns to `NONE` and G2 becomes PASS only in the one successor commit the prompt allows.
- Previous task, kept as history: `P4_RELEASE_BUNDLE` reached `PASS / COMPLETE` on 2026-10-01 and the
  pointer returned to empty. P4 was opened on 2026-09-30 by *FirmwareSight — P4 Release Bundle MVP
  Implementation, Execution Prompt v1.0 — Architect Reviewed*, delivered as a file (the first stage prompt
  since P0 to arrive that way) and archived with its measured SHA-256
  `1baaec9204a1d2aa5aa53bd735b34d376ee56db7557a04d6abb79265c30840c5` under `10_AUDIT/SOURCE_PROMPTS/`.
  Its acceptance list was the frozen **US-004 Export Bundle** criteria at
  `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:42-49`, plus **PRD P0-6** at `01_PRODUCT/01_PRD_MVP.md:85-95`
  and `01_PRODUCT/01_PRD_MVP.md:126`'s independent-readability requirement. Evidence: `P4_VALIDATION/`.
  The only G2 statement the stage was allowed to write is the one it wrote:
  `READY_FOR_ENGINEERING_GATE_REVIEW` — never `PASS`.
  `P3_RELEASE_GATE`, `P2_COMPARE` and `P1_ANALYZE_DETAILS` closed earlier, on 2026-09-30, 2026-09-29 and
  2026-09-29, in `P3_VALIDATION/`, `P2_VALIDATION/` and `P1_VALIDATION/`.
  **The P4 start fact is `323afad`, one commit ahead of the `ba5e59e` the prompt's §0/§2 names.** §2's
  "if remote is newer, inspect and reconcile before writing" was run: `git diff --name-only ba5e59e
  323afad` moves eleven documentation, governance and integrity files plus two newly tracked
  baseline-artifact scripts, and no product source, fixture, schema or migration, and Run `36810689645`
  on `323afad` is `completed` / `success` / **7 of 7 jobs** (`gh run view 36810689645
  --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs`). Start counts re-measured
  before the first write: one `cargo test --workspace` = **556 passed / 0 failed / 0 ignored**,
  `corepack pnpm test` = **135 passed in 6 files**.
  **`G2_ENGINEERING_CLOSURE_AUDIT` is the next authorizable act now that P4 has closed, and this file
  does not authorize it**; P4's own prompt stopped at P4.
  Carried forward from P3, still true: `origin/main` sat at
  `219178af195569ec6b13728d84d992ef78df8c04`, where Run `36774472141` concluded `failure` on one job of
  seven — `Dependency policy`, reporting `error[yanked]` for `yoke-derive 0.8.3`, a transitive proc-macro
  that crates.io yanked the same afternoon, after the local `deny` step had passed against an older
  index. Rust on both platforms, Desktop UI on both platforms, the macOS core smoke and the drift check
  passed remotely, and the lockfile moved to `0.8.4` in a successor commit. The prior head was `32b23aa`
  on Run #19 `36665007523`, success, 7 of 7 jobs, following Run #18 `36648718199` on the P2 implementation
  tree `4a77ea1` and the mid-round failure Run #17 on `c7fc2a3` — the fixture-pair story in
  `P2_COMPARE_EXECUTION_REPORT.md` §5.1. Every P3 gate number in this baseline is still a locally measured
  number; the remote result is recorded separately rather than folded into them.

## Product/architecture baseline

The v0.5.0 product, architecture and design decisions remain inherited unchanged. v0.6.0 adds the
validated P0 technical foundation; it does not redefine the product, does not add a fifth product
verb, and does not change MVP scope. Nothing in the frozen baseline was renegotiated to build the
slice, and nothing was renegotiated to promote it.

Four governance changes are in force now, all decided by the architect rather than by the coding side:

- **ADR-0025** supersedes exactly one clause of ADR-0020 (`P1 Product MVP implementation 只有两者都 PASS
  后开始`). After `P0 PASS` the architect may authorize a bounded, reversible Pre-G1 Analyze slice.
  Its Batch A precondition on the *next* slice was superseded the same day by ADR-0026 below; the slice
  it authorized, P1-A0, is complete and its record stands.
- **ADR-0026** (2026-09-29) makes this an **open-source MVP-first delivery**: MVP proceeds
  P1 → P2 → P3 → P4 → G2 on engineering grounds, `G1 = P0 PASS` for this delivery, V0 becomes
  `NON_BLOCKING_USER_FEEDBACK_TRACK`, and pricing / willingness-to-pay / pilot signals leave the gates.
  It supersedes sequencing only: no V0 artifact, P0 evidence, P1-A0 evidence or technical safety boundary
  is relaxed, and it authorizes no cloud, account, telemetry, AI, updater or licensing work.
- **Design tokens `0.2.0 → 0.2.1`**, adding one numeric semantic (`border.width.hairline = 1`) to close
  the documented P0 gap. No color, spacing, radius, typography, layout, motion, shadow or status value
  changed, and the focus ring keeps its own 2px token.
- **ADR-0027** (2026-09-29) is P3 prompt §6 written down as an architecture record rather than
  re-derived in code: a fifth first-party library crate, `firmwaresight-project`, owns
  `firmwaresight.toml`, project-local release evidence, the read-only system Git adapter and the
  deterministic Gate input fingerprints, because CLI and Desktop need one implementation of those
  semantics and Core must stay filesystem-free, Git-process-free and dependency-free. The boundary,
  the prohibited dependencies and the short admission list were specified by the architect in the
  prompt; the coding side's role was to record them, measure the locked versions and licenses, and not
  install anything the requirement does not need. It authorizes no Gate rule semantic outside Core, no
  artifact parsing inside the adapter, and no network.

## P0 — Technical Vertical Slice

Status: **`PASS`** · promoted to v0.6.0 by the architect-signed
*P0 Final Promotion / v0.6.0 Baseline Closure v1.0* · Task at promotion time: **NONE** (recorded as
`active_task: NONE` in `BASELINE.yaml`; the task live on 2026-09-29 was `P1_ANALYZE_DETAILS`, then after it
closed the field read `NONE` for a few hours until the P2 prompt arrived the same day — it read
`P2_COMPARE`, returned to `NONE` when P2 closed, and reads `P3_RELEASE_GATE` now. Each move changes the
pointer and not this frozen P0 verdict)

The chain P0 claimed is proven by executed commands, on this machine and on GitHub's runners:

```text
Real ELF + GNU ld MAP → guarded intake → parse → normalize → evidence
→ memory accounting → BuildSnapshot → deterministic CLI JSON → SQLite → typed IPC → Desktop summary
```

Exactly four Phase-0 library crates, with `firmwaresight-core` declaring zero dependencies. 104 Rust
tests and 19 UI tests as promoted; the tree now runs 142 and 31, and `P1_A0_VALIDATION/` accounts for
every test after those. SQLite schema version 2, including the v1→v2 migration. Deterministic CLI JSON
reproduced byte-identical. Memory accounting reproduced by hand from `readelf` and the MAP region
table. A 512 MiB guard measured from both sides of the boundary. A typed ts-rs IPC boundary whose
generated TypeScript is drift-checked. A real Windows desktop window opened and driven in the
shipping configuration.

### Remote verification

| Run | HEAD | Conclusion | Jobs |
| --- | --- | --- | --- |
| #1 `36360310447` | `f9b8ccb` | `failure` | 2 of 6 green — Rust (win), Rust (ubuntu), Generated output drift, Dependency policy red |
| #2 `36378384225` | `ebda52d` | `failure` | 6 of 7 green — only `Generated output drift` red (Ubuntu provisioning in a second job) |
| #3 `36399805005` | `1cd6309` | **`success`** | 7 of 7 — engineering closure |
| #4 `36402637251` | `5e58f77` | **`success`** | 7 of 7 — pre-promotion revalidation of the governance-only successor |
| #5 `36416146281` | `738ae78` | **`success`** | 7 of 7 — revalidation of the v0.6.0 promotion commit itself |
| #6 `36419864513` | `7d2f38a` | **`success`** | 7 of 7 — revalidation of the baseline consistency-closure commit; the last remote fact of the P0 chain, and still `last_remote_ci` inside that block |

Read with `gh run view 36419864513 --repo 2023violet/FirmwareSight`, not from this file. Run #6's
matrix: `Rust (windows-latest)`, `Rust (ubuntu-latest)`, `Desktop UI (windows-latest)`,
`Desktop UI (ubuntu-latest)`, `Generated output drift`, `Dependency policy`, `macOS Core Smoke` —
all `success`. The consistency-closure commit is on `origin/main`, so the baseline record is green on
its own commit, not only on the commit the architect read.

The two failures are kept as history rather than edited. They are the reason the four successes that
followed mean anything: Run #1 exposed a checkout that could rewrite committed evidence bytes,
a runner with no
Tauri Linux prerequisites, a drift assertion that compared third-party encoder output instead of
pixels, and a supply-chain policy the pinned tool could not parse; Run #2 then exposed that a second
job compiled the desktop crate without the prerequisites its sibling had been given.

### What the two remediation rounds changed

| Cause | Fix | Confirmed |
| --- | --- | --- |
| Checkout changed fixture bytes | `.gitattributes` text policy (`*.ld text eol=lf`, `*.map -text`, binaries unchanged); one manifest value moved to the blob's own hash | Runs #2, #3, #4 |
| Ubuntu runner lacked Tauri prerequisites | the documented apt block, `if: matrix.os == 'ubuntu-latest'`, desktop crate still compiled | Runs #2, #3, #4 |
| Icon check compared encoder bytes | decode and compare pixels; missing ICO frame still fails | Runs #2, #3, #4 |
| `deny.toml` unparseable by 0.20.2 | rewritten to the keys the tool accepts; allow list from `cargo deny list`; bans evaluated over the four shipping targets | Runs #2, #3, #4 |
| No macOS job | `macos-core` running `check.py --only core-smoke` on `main` pushes | Runs #3, #4 |
| `drift` job lacked the same prerequisites | the proven apt block copied into that job — no abstraction for two jobs | Runs #3, #4 |

A sixth defect was found by the authorized desktop launch rather than by CI: `evidence` had a
whole-table primary key while `04_TECH/15` §4 declares `Build 1─N Evidence`, so a second artifact
could never be stored. Migration `0002` keys evidence by `(build_id, id)`; the two failing tests were
written before the fix; the upgrade was replayed against the real user-profile database.

## V0

Status: **`NON_BLOCKING_USER_FEEDBACK_TRACK`** since ADR-0026 (2026-09-29). The sample state is
unchanged and is still an honest zero.

Formal eligible external sessions: `0 / 8 minimum`, `0 / 4–5 Batch A target`.

Completed: clickable prototype, fixture/state machine, protocol/session templates, internal
functional dry run, Batch A takeover, Batch A recruitment-ready package, Batch A activation.

No CI run, no green gate and no promotion signature can supply the missing input, which is real human
participants; the coding side authors none, and none were authored. What changed on 2026-09-29 is the
consequence of that zero: it no longer blocks P1, P2, P3, P4 or G1. The activation round earlier the same
day had already verified the recruitment pack and written nothing else; `ADR-0026` then moved the track
off the critical path, and the price-anchor prompt that would have followed it was
**`WITHDRAWN_BY_ARCHITECT`, never executed**.

The instrument stays frozen and reusable on purpose: `V0_VALIDATION/prototype/` at `v0.1.0` with
`protocol/TASK_SCRIPT.md`, `sessions/TEMPLATE.md` and both registers, so a later feedback round remains
comparable with the protocol that already exists. What is now unverified rather than merely deferred is
the thing V0 existed to check: whether a real firmware engineer distinguishes `Unknown` from `PASS`,
understands why a MAP is requested, or reads a `Review` correctly. That risk is carried forward and
recorded in `ADR-0026`'s Consequences, not argued away here.

## Gates

```text
G0: PASS
P0: PASS — promoted to the v0.6.0 Technical Foundation Baseline; frozen, no further P0 closure prompts
G1: PASS — basis is P0 PASS under ADR-0026 (2026-09-29). Before that date this file read `NOT CLAIMED` against `G1 = V0 PASS + P0 PASS`, and the historical records still say so
V0: NON_BLOCKING_USER_FEEDBACK_TRACK — 0 of 8 eligible external sessions, an honest zero that gates no P-stage and no G1
Pre-G1 (ADR-0025): P1-A0 REAL ARTIFACT INTAKE — COMPLETE, including its evidence-identity and persistence correctness closure
P1: PASS / COMPLETE — the Analyze verb as one product verb: intake, summary, top contributors, bounded Sections / Symbols / Evidence details, the Evidence Inspector, and the US-001 bytes/KiB presentation switch. Evidence: P1_VALIDATION/
P2: PASS / COMPLETE — Compare over persisted snapshots: Core-owned deterministic diff, bounded Compare IPC with a session-local registry, the second desktop page, `fwsight diff`, and portable Diff JSON v1 plus self-contained HTML. Evidence: P2_VALIDATION/. The round's own measurements are LOCAL PASS; the pushed head `4a77ea1` is green on Run #18 `36648718199` (7 of 7), which closes defect E — the fixture half `.gitignore` hid — on the remote too. Open on purpose: desktop smoke step 27 was not observed in the shipped window
P3: PASS / COMPLETE — Release Gate, authorized 2026-09-29 by *P3 Release Gate MVP Implementation, Execution
Prompt v1.1* and by ADR-0027, closed 2026-09-30. Evidence: P3_VALIDATION/. The round's own gate numbers are
LOCALLY measured; what the remote then said is recorded separately in that pack, and P3 is SEALED — no
further P3 documentation-only successor.
P4: PASS / COMPLETE — Release Bundle, opened 2026-09-30 by its own architect prompt (v1.0, delivered as
a file and registered with its SHA-256), closed 2026-10-01. Evidence: P4_VALIDATION/. The round's own
numbers are LOCALLY measured — one `cargo test --workspace` = 769 passed / 0 failed, UI 155 in 6 files,
`check.py` 15/15, the fifty-step shipped-window smoke, the independent reader at 64/64 and 59/59 — and
the remote result is recorded separately: the final implementation head `e799f2f` is green on run
`36872456446`, 7 of 7 jobs. P4 is the last core product-implementation stage of the MVP line.
G2: LOCAL PASS / READY FOR REMOTE CLOSURE — the whole-MVP engineering closure audit ran on 2026-10-01
under its own prompt and storage-path addendum; evidence G2_VALIDATION/. Not PASS until the one successor
commit records this closure's remote run 7 of 7
Pricing / willingness-to-pay / team-pilot signal: DEFERRED_POST_MVP; the price-anchor prompt was WITHDRAWN_BY_ARCHITECT and never executed
Active task: G2_ENGINEERING_CLOSURE_AUDIT — returns to NONE in the successor commit
Design tokens: v0.2.1
```

`ADR-0026-open-source-mvp-first-delivery.md` is what changed here. It re-bases G1 on `P0 PASS` for the
current open-source MVP and moves V0 off the critical path, superseding the sequencing conclusions of
ADR-0020 and ADR-0025 while leaving their decisions, evidence and safety boundaries in place. G1's
technical content in `06_DELIVERY/06_STAGE_GATES.md` is unchanged and still has to hold; what was removed
is the requirement that a moderated human panel exist before more of the product can be built. That is a
real cost, recorded in the ADR's own Consequences section: whether users distinguish `Unknown` from
`PASS`, or understand why a MAP is asked for, is now unverified rather than deferred.

Full reasoning: `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md`; the promotion act:
`P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`; per-item evidence:
`P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`; run history: `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`;
the two remediation rounds: `P0_CI_REMEDIATION_REPORT.md` and `P0_CI_RUN_2_CLOSURE_REPORT.md`; the
shipped-window result: `P0_DESKTOP_SMOKE_REPORT.md`.

## Known gaps that promotion does not remove

- **Peak RSS: `NOT MEASURED`**, with the reason and the measurements that do exist in
  `P0_PERFORMANCE_REPORT.md`.
- **Fuzz campaign: `NOT RUN`** — `cargo-fuzz` needs a nightly toolchain the tool policy did not
  authorize. The no-panic claim rests on regression tests over malformed fixtures.
- **Two RustSec advisories accepted, not resolved**: `RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and
  `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`, unmaintained) enter through the `gtk-rs 0.18` line
  Tauri 2.12.0 requires and have no compatible upgrade inside it. Disposition: accepted as explicit
  P0 transitive risk, not a promotion blocker, with five revisit triggers. The dependency policy is
  "PASS under a documented policy with two explicitly accepted transitive advisories" — not
  "zero vulnerabilities" and not "security clean".
- **Desktop smoke: one Windows host, one WebView2 version, 100% scaling.** No Windows 11, no other
  DPI, no Linux or macOS window, and the loading state stayed a test-only claim (recorded NOT
  OBSERVED).
- **Two linker layouts tested.** Executable-in-RAM, external SRAM, DMA pools, overlays and
  MPU-aligned sections are unproven; DWARF is recognized but not consumed.
- **Two stored facts carry no reason.** `sections.file_offset` and `symbols.address` are nullable with
  no companion `*_unknown` column, so a reason cannot survive the write. The read layer types them as a
  plain option and the UI says `Unknown` with no invented explanation. Closing this needs a migration,
  which P1 deliberately did not write; found and reported in
  `P1_VALIDATION/P1_ANALYZE_EXECUTION_REPORT.md` §6.
- **One dead `Apply filter` click is unresolved.** It happened once on an intermediate build and could
  not be reproduced on the shipped binary or in the automated reproducer. The leading hypothesis is the
  WebView2 autofill popup, and `autoComplete="off"` removes that surface, but no cause is established.
  Recorded as unresolved in `P1_ANALYZE_DETAILS_SMOKE_REPORT.md` §6 rather than written up as fixed.
- **Icon provisioning duplication is a real cost.** Two jobs copy the same apt block; a third job that
  compiles the shell on Linux is the trigger to extract it.
- **Every gate step except one reads the working tree, not the index.** That blind spot is how P2's
  fixture pair survived eight commits with only half of it in the repository: `.gitignore`'s
  `**/target/` matched `fixtures/elf/p2-diff/target/`, the manifest recorded all twelve paths, and the
  hash test happily digested the files the generator had left on disk. CI caught it because CI clones.
  `drift/fixtures tracked` now closes the case for anything a manifest lists; a source file ignored by
  accident is still unchecked, because no manifest names it. Recorded in
  `P2_VALIDATION/P2_COMPARE_EXECUTION_REPORT.md` §5.1.

## Version rule

v0.6.0 exists because P0 reached `PASS` with real engineering evidence measured on three platforms,
signed by the architect. The rule that produced it stays in force for the next promotion: a failed run
must not be relabeled into a PASS baseline, a locally green gate must not be relabeled into a CI PASS,
and a green CI does not equal an architectural promotion. `v0.7.0` and G1 claims will require their own
measured evidence and their own signed prompt.

## Integrity record

`DIRECTORY_TREE.txt` and `SHA256SUMS` are the **v0.6.0** baseline artifacts, regenerated by the promotion
round after every document change and gate run, in that order: the tree lists the tracked layout rooted
at `FirmwareSight_Project_Baseline_v0.6.0/`, and the manifest hashes each baseline-controlled tracked
file with a lowercase SHA-256, LF line endings, lexicographic path order and no self-entry. Both are
produced by `scripts/generate_baseline_artifacts.py` (`tree`, then `sums`) and re-checked by
`scripts/verify_baseline_artifacts.py`, which shares no code with the generator: it re-derives the
tracked set from `git ls-files` and hashes the files straight off disk, because the manifest cannot
certify itself. `sha256sum -c SHA256SUMS` is the third, external pass. This tooling used to live under
gitignored `target/`, which meant a fresh clone could not regenerate either artifact; it is tracked now.
`manifest.txt` is left exactly as the v0.5.1 delivery wrote it - the frozen package list, not
this tree - and `P0_FINAL_PROMOTION_REPORT.md` §10 records the commands and the counts. P4's closure
regenerated both artifacts after the tracked path set grew by this stage's fixtures, golden bundle,
tests, scripts and validation pack — `DIRECTORY_TREE.txt` first and `SHA256SUMS` last, as the order
requires — and the closure commit records the checker's zero-mismatch result.

**How to read a `file:line` citation in this repository.** A closed validation pack is evidence about the
tree it measured, so its line numbers are as-of-writing, not live: `App.module.css` and `App.tsx` have
both shrunk since P0, and several P0/P1/P2 checklist lines now point past the end of those files. Those
packs are not rewritten to match current code - that would turn a record into a claim. Within a *living*
document, cite the key or the symbol rather than the line number, because a line number goes stale the
moment an adjacent line is added: P3 inserting `crates/firmwaresight-project` into the workspace member
list pushed `license = "Proprietary"` from `Cargo.toml` line 17 to line 18 and silently invalidated every
citation written before it. The current stage's pack is the one place a line pointer is still checkable.

## Next work

**`active_task: NONE`.** P4 Release Bundle closed `PASS / COMPLETE` on 2026-10-01, so no stage is live
and nothing may be started from this file. `AGENTS.md` 1 means what it has meant in every round: the
pointer names ONE stage, and no agent lifts the next one off the roadmap. What becomes authorizable
now is the **G2 engineering closure audit**, which P4's own §5 explicitly forbade this round from
performing or claiming, and which only an architect prompt can open.

**`P4 Release Bundle` closed `PASS / COMPLETE` on 2026-10-01, under its own architect prompt.** One
product verb, `Bundle`, over results the earlier stages had already computed and stored: a release
plan — id, version, ten-file list with digests — assembled before any byte is written; a staged copy
of the composed documents and the current artifacts; verification of the staged bytes against their
own `SHA256SUMS` and manifest; publish by rename; replacement only under an explicit confirmation and
only of a destination this engine wrote, with rollback if the swap fails; and a `release_records` row
written after the bytes, never before. Per-item evidence:
`P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md`; what was built, every gate number, the CLI smoke and the
CI runs: `P4_BUNDLE_EXECUTION_REPORT.md`; the shipped-window result over all fifty §59 steps:
`P4_BUNDLE_DESKTOP_SMOKE_REPORT.md`; design review: `P4_BUNDLE_DESIGN_CHECKLIST.md`. The boundaries
held and were not relaxed: Core stayed headless and filesystem-free (`grep -rn "std::fs"
crates/firmwaresight-core/src/` counts zero), assembly lives in the project crate, no sixth crate and
no new third-party dependency entered, no host path crossed IPC or entered a composed document, the
only clock a bundle carries is the moment a named person accepted a review, and the stage stopped at
the Bundle — no History page, no installer, no signing, no updater.

**`P3 Release Gate` closed `PASS / COMPLETE` on 2026-09-30, under its own architect prompt.** One product
verb, `Gate`, over a stored build plus the workspace facts outside the artifact:
`firmwaresight.toml` → project and Git evidence → `GatePolicy` → Core Gate → five factual states →
immutable `GateRun` → immutable review acceptance → `fwsight gate` → the desktop `Release` page. Per-item
evidence: `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md`; what was built, every gate number and the three
defects this round found: `P3_GATE_EXECUTION_REPORT.md`; the shipped-window result over all forty §61
steps: `P3_GATE_DESKTOP_SMOKE_REPORT.md`; design review: `P3_GATE_DESIGN_CHECKLIST.md`. The boundaries
held and were not relaxed: Core stayed headless and dependency-free, the adapter owns the config and the
read-only Git process, no rule is re-evaluated in SQL or React, no host path crossed IPC, the database or
the portable output, and the stage stopped at the Gate.

`P1_ANALYZE_DETAILS` completed on 2026-09-29 and P1 Analyze is `PASS / COMPLETE`:
bounded `Sections`, `Symbols` and `Evidence` queries over the snapshot SQLite already stores, three
use-case IPC commands, top contributors, the Evidence Inspector, and the `bytes / KiB` presentation
switch that `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` US-001 requires. Per-item evidence:
`P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md`; what was built and what was found:
`P1_ANALYZE_EXECUTION_REPORT.md`; the shipped-window result: `P1_ANALYZE_DETAILS_SMOKE_REPORT.md`;
design review: `P1_ANALYZE_DESIGN_CHECKLIST.md`. Earlier slices keep their own packs: `P0_TECHNICAL_VALIDATION/`
and `P1_A0_VALIDATION/`.

**`P2 Compare` closed `PASS / COMPLETE` on 2026-09-29, under its own architect prompt.** The prompt
changed the pointer while the round ran, not the standards: `P2` was `IN_PROGRESS` until `US-002` was
green item by item, and it is now — per-item evidence in `P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md`,
what was built and every gate number in `P2_COMPARE_EXECUTION_REPORT.md`, the shipped-window result in
`P2_COMPARE_DESKTOP_SMOKE_REPORT.md`, design review in `P2_COMPARE_DESIGN_CHECKLIST.md`. The boundaries
that held for P1 held for P2 and were not relaxed: no schema migration (`SCHEMA_VERSION` still 2, no
0003), no new dependency (`Cargo.lock` gained no package), no new design token (still v0.2.1) - a
genuinely missing token stops a round rather than being written as a magic number - no whole-table
payload across IPC (default limit 100, hard max 500, enforced in Rust), details always bound to the
last-good handle, `Unknown` never rendered as zero, addresses never unit-converted. `AGENTS.md` 2 / 7 /
11 keep applying in full; `ADR-0026` relaxed research sequencing, not a single technical boundary.

**`P3 Release Gate` closed `PASS / COMPLETE` on 2026-09-30, under its own architect prompt.** The same
prompt line that authorized the work authorized a structural change, so it is stated here rather than left
to be discovered in a diff: P3 added the fifth first-party library crate (`crates/firmwaresight-project`,
ADR-0027), the third migration (`0003_gate_history.sql`, `SCHEMA_VERSION` 2 → 3, additive only), the
first direct dependencies outside the frozen five (`toml`, `regex`, `sha2`, `thiserror`, `serde` — all but
`regex` already present in the graph or admitted by ADR-0027), a third top-level desktop page (`Release`),
two CLI exit codes that were unreachable and are now not (`4` REVIEW / `5` BLOCK — the surface test
asserted `[0, 2, 3, 6]`, and P3 changed it by name rather than by deleting it), and three new portable
contracts (`gate-results`, `accepted-reviews`, `project-config` at v1). It added **no** design token, did
not touch the license metadata, did not create a Bundle table and did not start History. The verdict is
item by item in `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md`; what the round found on the way is in the other
three documents of that pack.

Three product defects were found by P3's own validation and fixed in product code, each with a regression
test: a version pattern quoted inside an evidence locator made a Gate run unpersistable for any project
using the only MVP version source (`ERR-STORAGE-4006`); one build produced two different run ids on the
two surfaces because the stored footprint carried no evidence pointer and Core hashed a `reason` the
database never recorded; and a disabled primary button kept its accent border.

What belongs to the owner and the architect:

1. pushing remains the owner's act. The P2 round pushed twice. The first, mid-round at `c7fc2a3`, produced
   **Run #17 `36596452341`, `failure`, 3 of 7 jobs red** — `Rust (windows-latest)`, `Rust (ubuntu-latest)`
   and `macOS Core Smoke`, all on `committed_fixtures_match_their_recorded_hashes`, for the reason in the
   gap above: `.gitignore`'s `**/target/` had kept the target half of the fixture pair out of the
   repository while every local run digested the bytes the generator had left on disk. That is defect E,
   fixed at `cfee1e5` and re-verified from a clean `git archive` checkout. The owner then pushed the head,
   and **Run #18 `36648718199` on `4a77ea1` concluded `success`, 7 of 7 jobs** — the same seven that
   carried P1's `e63afaf` on Run #13 `36556735551`. `origin/main` is now green again. No commit carries the
   run its own push produced: #18 is recorded by the successor document that reports it, and the run this
   document's own push starts belongs to a later one;
3. the **open-source license decision**, which is not P3's to make. The root `Cargo.toml`'s
   `[workspace.package]` table still reads
   `license = "Proprietary"` and the repository root still has no `LICENSE` file, while the project is
   being delivered as open source under ADR-0026; `AGENTS.md` 9 places a license change in front of a
   human, so the P3 prompt explicitly declines to choose one and the gap is recorded as
   `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` rather than quietly resolved;
4. a **separate architect prompt** for P4 Release Bundle, which P3's closure will make authorizable and
   this file does not authorize. P3's own prompt says STOP AFTER P3, and nothing in P4's scope — a
   bundle directory, `release prepare`, manifest generation, bundle checksums, a History page — exists
   or is implied by it;
5. whether and when to resume the V0 feedback track with real participants - now a choice rather than a
   gate, with the `v0.1.0` instrument still frozen and ready;
6. re-authorizing pricing, paid-tier or pilot work only after MVP, since `ADR-0026` defers it and the
   matching prompt was withdrawn before execution;
7. re-opening the two accepted advisories only on one of their recorded triggers.

Four items the closure round surfaced and left alone, each with its reason in
`P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` §9.7: the golden updater no longer reproduces the
committed goldens' key order; `ElfProgramHeader` names a source the ELF parser never reads; the frozen
release-manifest schema describes one artifact per build; and a release build without
`--features custom-protocol` shows a WebView network error instead of the product.

Do not start P1, Compare, Gate, Bundle, installer, signing, updater, SBOM, cloud, accounts, AI or
telemetry from this file.
