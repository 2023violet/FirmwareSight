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
- Active task: **`P1A0_REAL_ARTIFACT_INTAKE` — COMPLETE**, together with its evidence-identity and
  persistence correctness closure. Nothing further is authorized to code from this file.

## Product/architecture baseline

The v0.5.0 product, architecture and design decisions remain inherited unchanged. v0.6.0 adds the
validated P0 technical foundation; it does not redefine the product, does not add a fifth product
verb, and does not change MVP scope. Nothing in the frozen baseline was renegotiated to build the
slice, and nothing was renegotiated to promote it.

Two governance changes are in force now, both decided by the architect rather than by the coding side:

- **ADR-0025** supersedes exactly one clause of ADR-0020 (`P1 Product MVP implementation 只有两者都 PASS
  后开始`). After `P0 PASS` the architect may authorize a bounded, reversible Pre-G1 Analyze slice.
  `G1 = V0 PASS + P0 PASS` is unchanged, `P1-A0` is not a stage gate and does not close P1.
- **Design tokens `0.2.0 → 0.2.1`**, adding one numeric semantic (`border.width.hairline = 1`) to close
  the documented P0 gap. No color, spacing, radius, typography, layout, motion, shadow or status value
  changed, and the focus ring keeps its own 2px token.

## P0 — Technical Vertical Slice

Status: **`PASS`** · promoted to v0.6.0 by the architect-signed
*P0 Final Promotion / v0.6.0 Baseline Closure v1.0* · Task at promotion time: **NONE** (recorded as
`active_task: NONE` in `BASELINE.yaml`; the live task is now `P1A0_REAL_ARTIFACT_INTAKE`)

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

Status: **`ACTIVE EXTERNAL VALIDATION — WAITING FOR REAL PARTICIPANTS`** (resumed, not validated)

Formal eligible external sessions: `0 / 8 minimum`, `0 / 4–5 Batch A target`.

Completed: clickable prototype, fixture/state machine, protocol/session templates, internal
functional dry run, Batch A takeover, Batch A recruitment-ready package.

P0 passing does not move this number, and neither does P1-A0. The V0 blocker is the absence of real
human participants, which no CI run, no green gate and no promotion signature can supply. Resumed is
not passed: the count stays `0`, the wording change is a governance statement about the track, and the
coding side authors no transcript, session or metric. `V0_VALIDATION/**` is not modified by any P0 or
P1-A0 round. Under ADR-0025, **V0 Batch A `>= 4` eligible sessions plus an interim architect review is
the precondition for authorizing any pre-G1 slice after P1-A0**.

## Gates

```text
G0: PASS
V0: ACTIVE EXTERNAL VALIDATION / WAITING FOR REAL PARTICIPANTS (0 of 8; Batch A 0 of 4-5)
P0: PASS — promoted to the v0.6.0 Technical Foundation Baseline; frozen, no further P0 closure prompts
Formal G1: NOT CLAIMED (requires V0_PASS and P0_PASS; V0 is 0/8)
Pre-G1 (ADR-0025): P1-A0 REAL ARTIFACT INTAKE — COMPLETE, including its evidence-identity and persistence correctness closure
P1: NOT PASS / NOT CLOSED
P1-A1: NOT AUTHORIZED (needs V0 Batch A >= 4 sessions + interim review + a new prompt)
P2 / P3 / P4: NOT AUTHORIZED
Active task: P1A0_REAL_ARTIFACT_INTAKE
Design tokens: v0.2.1
```

`06_DELIVERY/06_STAGE_GATES.md` defines `G1 = V0 PASS + P0 PASS`. With V0 unvalidated, a P0 `PASS`
alone does not open G1, and this round claims nothing beyond P0. ADR-0020's own text is unchanged except
for a status note recording that ADR-0025 supersedes one clause of it: the sequencing question that used
to read "the architect must issue a decision if P1 is ever to start while V0 is deferred" has now been
answered — narrowly, for one slice, and documented as `P1-A0` rather than as P1.

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

**`active_task: P1A0_REAL_ARTIFACT_INTAKE`, and that task is now finished.** P0 is closed and frozen;
the one bounded pre-G1 slice ADR-0025 authorized was implemented, tested in a real window, and
recorded in `P1_A0_VALIDATION/`. Its stop condition was P1-A0 itself, so the pointer names work that is
complete rather than work to resume: **nothing further is authorized to code from this file**, and
`AGENTS.md` 1 still forbids inventing anything beyond it.

What belongs to the owner and the architect rather than to this round:

1. pushing was the owner's act, and it happened: the correctness-closure commit `ace6fbe` is on
   `origin/main` and **Run #10 `36515470263` measured it `success`, 7 of 7 jobs** (read with
   `gh run view`). Run #9 `36499759371` on `2960173` is the intake commit's own green run. The rule held
   where it applied — neither commit carries the run number its own push produced; that is recorded in
   the successor documentation instead;
2. the next engineering authorization, which requires V0 Batch A `>= 4` eligible external sessions, an
   interim architect review of that evidence, and a new prompt;
3. re-opening the two accepted advisories only on one of their recorded triggers.

Four items the closure round surfaced and left alone, each with its reason in
`P1_A0_VALIDATION/P1_A0_EXECUTION_REPORT.md` §9.7: the golden updater no longer reproduces the
committed goldens' key order; `ElfProgramHeader` names a source the ELF parser never reads; the frozen
release-manifest schema describes one artifact per build; and a release build without
`--features custom-protocol` shows a WebView network error instead of the product.

Do not start P1, Compare, Gate, Bundle, installer, signing, updater, SBOM, cloud, accounts, AI or
telemetry from this file.
