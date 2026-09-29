---
title: "Current Project State"
doc_id: "FS-AI-002"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-29"
---

# Current State

- Date: 2026-09-29
- Baseline: **v0.6.0 — P0 Technical Foundation Baseline**
- Active task: **`NONE`** — `P1_ANALYZE_DETAILS` finished on 2026-09-29 and P1 Analyze is `PASS / COMPLETE`
  (Sections, Symbols, Evidence Inspector, top contributors, US-001 bytes/KiB switch), recorded in
  `P1_VALIDATION/`. `AGENTS.md` 1 forbids starting the next stage from the roadmap: **P2 Compare is the
  next authorizable stage and needs its own architect prompt.**

## Product/architecture baseline

The v0.5.0 product, architecture and design decisions remain inherited unchanged. v0.6.0 adds the
validated P0 technical foundation; it does not redefine the product, does not add a fifth product
verb, and does not change MVP scope. Nothing in the frozen baseline was renegotiated to build the
slice, and nothing was renegotiated to promote it.

Three governance changes are in force now, all decided by the architect rather than by the coding side:

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

## P0 — Technical Vertical Slice

Status: **`PASS`** · promoted to v0.6.0 by the architect-signed
*P0 Final Promotion / v0.6.0 Baseline Closure v1.0* · Task at promotion time: **NONE** (recorded as
`active_task: NONE` in `BASELINE.yaml`; the task live on 2026-09-29 was `P1_ANALYZE_DETAILS`, which is
now closed, so the field reads `NONE` again)

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
P2 / P3 / P4: NOT STARTED — P2 Compare is the next authorizable stage; each still needs its own architect prompt
Pricing / willingness-to-pay / team-pilot signal: DEFERRED_POST_MVP; the price-anchor prompt was WITHDRAWN_BY_ARCHITECT and never executed
Active task: NONE
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
file with a lowercase SHA-256, LF line endings, lexicographic path order and no self-entry. Verification
was run twice, by `sha256sum -c` and by an independent checker, because the manifest cannot certify
itself. `manifest.txt` is left exactly as the v0.5.1 delivery wrote it - the frozen package list, not
this tree - and `P0_FINAL_PROMOTION_REPORT.md` §10 records the commands and the counts.

## Next work

**`active_task: NONE`.** `P1_ANALYZE_DETAILS` completed on 2026-09-29 and P1 Analyze is `PASS / COMPLETE`:
bounded `Sections`, `Symbols` and `Evidence` queries over the snapshot SQLite already stores, three
use-case IPC commands, top contributors, the Evidence Inspector, and the `bytes / KiB` presentation
switch that `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` US-001 requires. Per-item evidence:
`P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md`; what was built and what was found:
`P1_ANALYZE_EXECUTION_REPORT.md`; the shipped-window result: `P1_ANALYZE_DETAILS_SMOKE_REPORT.md`;
design review: `P1_ANALYZE_DESIGN_CHECKLIST.md`. Earlier slices keep their own packs: `P0_TECHNICAL_VALIDATION/`
and `P1_A0_VALIDATION/`.

**`P2 Compare` is the next authorizable stage, and this file does not start it.** `AGENTS.md` 1 forbids
inventing work from the roadmap, so the work begins only when the architect issues a P2 prompt. The
boundaries that held for P1 hold for whatever follows and were not relaxed here: no schema migration
without its own decision (`SCHEMA_VERSION` is 2), no new dependency, no new design token - a genuinely
missing token stops a round rather than being written as a magic number - no whole-table payload across
IPC (default limit 100, hard max 500, enforced in Rust), details always bound to the last-good
`snapshotId`, `Unknown` never rendered as zero, addresses never unit-converted. `AGENTS.md` 2 / 7 / 11
keep applying in full; `ADR-0026` relaxed research sequencing, not a single technical boundary.

What belongs to the owner and the architect:

1. pushing remains the owner's act. It last happened through `e63afaf`, the head of the four-commit push
   that carried the P1 Analyze round, and its remote verification is
   **Run #13 `36556735551`, `success`, 7 of 7 jobs** (read with
   `gh run view 36556735551 --repo 2023violet/FirmwareSight`). The gate fires per push, so that one run
   covers the push rather than each commit: `872ad7e`, `f649afd` and `ae7759a` have no run of their own
   and none is claimed for them. `be09c65` and its **Run #12 `36520562718`** stay the record of the round
   before. No commit in either chain carries the run number its own push produced;
2. whether and when to resume the V0 feedback track with real participants - now a choice rather than a
   gate, with the `v0.1.0` instrument still frozen and ready;
3. a **separate architect prompt** for P2 Compare once P1 is verifiably complete; this round does not
   authorize it;
4. re-authorizing pricing, paid-tier or pilot work only after MVP, since `ADR-0026` defers it and the
   matching prompt was withdrawn before execution;
5. re-opening the two accepted advisories only on one of their recorded triggers.

Four items the closure round surfaced and left alone, each with its reason in
`P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` §9.7: the golden updater no longer reproduces the
committed goldens' key order; `ElfProgramHeader` names a source the ELF parser never reads; the frozen
release-manifest schema describes one artifact per build; and a release build without
`--features custom-protocol` shows a WebView network error instead of the product.

Do not start P1, Compare, Gate, Bundle, installer, signing, updater, SBOM, cloud, accounts, AI or
telemetry from this file.
