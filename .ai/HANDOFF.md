---
title: "Project Handoff"
doc_id: "FS-AI-004"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-09-28"
---

# Handoff — FirmwareSight v0.5.1 / P0 remote CI green (Run #3, 7 of 7), awaiting the architect's promotion sign-off

## Purpose

Close the last remote failure, then stop before promotion. That is done: Run #1 failed four jobs, the
first round fixed them, Run #2 left one (`drift` missing the Linux prerequisites its sibling job had),
the second round added that step, and **Run #3 (`36399805005`, head `1cd6309`) concluded `success`
with 7 of 7 jobs green.** The HEAD is pushed and local/remote are in sync.

The next action is the architect's, not a coding agent's: issue and sign the *P0 Final Promotion /
v0.6.0 Baseline Closure* prompt. That prompt - not this one - is where `P0 = PASS`,
`baseline_version: 0.6.0`, a regenerated `SHA256SUMS`/`DIRECTORY_TREE` and `active_task` are written.
Whoever resumes must read Run #3 from GitHub (`gh run view 36399805005 --repo
2023violet/FirmwareSight`) rather than trusting this file, and must not start P1 or promote V0: G1
still requires V0's `0 / 8` external sessions, which no CI run can supply.

## Read first

1. README.md
2. PRODUCT_BASELINE.md
3. BASELINE.yaml
4. AGENTS.md
5. DESIGN.md + assets/design-tokens.json (for the Desktop UI step)
6. `10_AUDIT/SOURCE_PROMPTS/README.md` — which prompt authorizes what
7. `P0_TECHNICAL_VALIDATION/P0_EXECUTION_PROVENANCE.md` — start HEADs and environment
8. `P0_TECHNICAL_VALIDATION/P0_IMPLEMENTATION_LOG.md` — decisions already taken
9. `P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md` — what is proven, and how
10. `P0_TECHNICAL_VALIDATION/P0_CI_RUN_2_CLOSURE_REPORT.md` — the latest remote run, read with `gh`
11. `P0_TECHNICAL_VALIDATION/P0_CI_REMEDIATION_REPORT.md` — Run #1's four failures, each with its reproducer and fix
12. `P0_TECHNICAL_VALIDATION/P0_DESKTOP_SMOKE_REPORT.md` — the shipped window, driven for real
13. `P0_TECHNICAL_VALIDATION/P0_DESIGN_CHECKLIST.md` — the AGENTS.md 11 review for the UI, with four open findings
14. `.ai/ACTIVE_TASK.md`

V0 status summary only (do not re-run V0 from this handoff):

- `V0_VALIDATION/README.md`
- `V0_VALIDATION/V0_EXECUTION_PROVENANCE.md`
- `V0_VALIDATION/deliverables/V0_GATE_RECOMMENDATION.md`

## Current state

P0 status: **`CONDITIONAL_PASS`** — Run #3 (`36399805005`, head `1cd6309`) is `success` with 7 of 7
jobs green, and the pushed HEAD is in sync with `origin/main`. The condition is the architect's
promotion sign-off, not an engineering gap. Baseline stays at `0.5.1` and the coding side writes no
`P0 PASS`, because the promotion act is issued in its own signed prompt.

What is proven and stays proven: 104 Rust tests and 19 UI tests, a single gate
(`python scripts/check.py`) that CI calls unchanged - 14 steps on a tree that already has the built
frontend, 16 when it has to build that too, plus 3 more under `--only core-smoke` - real ARM
ELF/MAP fixtures with recorded provenance, deterministic CLI JSON, memory accounting reproduced by
hand from `readelf`, a typed ts-rs IPC boundary, SQLite migrations with transactional import, a
512 MiB guard measured from both sides of the boundary, and Core/CLI/Desktop parity on the same bytes.

### Run #2, `36378384225`, on `ebda52d` — read with `gh run view`, not from a report

| Job | Result | Evidence in its own log |
| --- | --- | --- |
| Rust (windows-latest) | PASS | `5/5 steps passed`, `test result: ok.` summing to 104 |
| Rust (ubuntu-latest) | PASS | `5/5 steps passed`, 104, `firmwaresight-desktop` compiling |
| Desktop UI (windows-latest) | PASS | `5/5 steps passed` |
| Desktop UI (ubuntu-latest) | PASS | `5/5 steps passed`, `Tests 19 passed (19)` |
| macOS Core Smoke | PASS | `3/3 steps passed`, 87 tests, first execution |
| Dependency policy | PASS | `advisories ok, bans ok, licenses ok, sources ok` |
| Generated output drift | **FAIL** | tokens PASS, icons PASS (`pixel-identical to this build`), then `drift/ipc bindings` fails |

That closes Run #1's four causes remotely. The remaining cause is one job missing a step its sibling
has: `drift` runs `cargo test -p firmwaresight-desktop` to regenerate the ts-rs bindings, and the
`Install Linux prerequisites for the Tauri shell` block was added only to the `rust` job. The log
line is `Package gobject-2.0 was not found in the pkg-config search path` - the same missing `.pc`
class as Run #1's `glib-sys` failure, reached from a second job.

### Run #1, `36360310447`, on `f9b8ccb` — retained history

| Failed job | Cause | Now |
| --- | --- | --- |
| Rust (windows-latest) | `*.ld` absent from `.gitattributes`, so `core.autocrlf=true` rewrote the checkout to CRLF | closed remotely by Run #2 |
| Rust (ubuntu-latest) | `glib-sys`: `pkg-config cannot find glib-2.0` | closed remotely by Run #2 |
| Generated output drift | the icon check compared Pillow's compressed bytes | closed remotely by Run #2 |
| Dependency policy | `cargo-deny 0.20.2` could not parse `deny.toml` | closed remotely by Run #2 |
| (missing job) | no macOS core smoke | added; closed remotely by Run #2 |

The desktop window launch the first round authorized found a sixth defect on its own: analyzing a
second artifact raised `UNIQUE constraint failed: evidence.id`, because `evidence` had a whole-table
primary key on `id` while `04_TECH/15` §4 declares `Build 1─N Evidence`. Migration `0002` rebuilds the
table on `(build_id, id)`; schema version is 2; the two failing tests were written before the fix and
the upgrade was replayed against the real database the earlier launch left in `%APPDATA%`.

The two RustSec advisories are **no longer an open question handed up**: the architect accepted
`RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`,
unmaintained) as **explicit P0 transitive risk that does not block promotion**, with five revisit
triggers and no architecture ADR - because no architecture choice changed. They stay documented in
`deny.toml`'s reasons, `P0_DEPENDENCY_REPORT.md` and `P0_KNOWN_LIMITATIONS.md`. Replacing Tauri,
forking dependencies or moving the frozen desktop dependency family would need an ADR.

Peak RSS is `NOT MEASURED`, with the reason in `P0_PERFORMANCE_REPORT.md`. That is a measurement gap
the original P0 prompt accepted, not a promotion blocker; the promotion blocker is Run #3.

Two design-checklist findings stay open and are not CI failures: capability labels show Core's enum
words, a boundary decision about who owns user-facing wording; and eight `1px` borders have no
token, which needs a frozen-asset `design-tokens.json` bump P0 may not make. Two others the same
pass found - no live region, and a `select` with only hover and focus - were fixed and are covered by
a test. `P0_DESIGN_CHECKLIST.md` records all four with the commands that found them.

V0:

`DEFERRED / NOT YET EVIDENCE-VALIDATED` — formal external sessions `0 / 8 minimum`.
The prototype, protocol and internal functional dry run are complete; the missing input is
real participants. Deferral is not completion.

## Boundaries

- Four Phase-0 library crates only; a fifth requires architecture review plus an ADR.
- Core stays headless and synchronous: no Tauri, rusqlite, Tokio types or `object::*` leakage.
- No Compare/Gate/Bundle product workflow, no cloud/auth/telemetry/updater/wgpu/SQLx, no E1/E2/E3/GX.
- `v0.6.0` only on a real P0 `PASS`.
- This round ends at a local commit. Pushing belongs to the owner, and no `REMOTE CI PASS` may be
  written before an all-green run exists on the HEAD that was pushed.
- Fixing a CI failure never means editing the assertion: no expected fixture hash moves to absorb a
  checkout conversion, no dependency check becomes optional, no Ubuntu desktop coverage is dropped,
  and no drift step is deleted or moved to another OS to dodge a missing system library.
- Two jobs needing the same ten apt lines is accepted duplication for now. Abstracting it into a
  script is not automatically better: it adds a portability and testing surface. Revisit if a third
  job needs the same list.
- The advisory disposition is the architect's, not the coding agent's. Do not "fix" it by upgrading
  Tauri or gtk-rs, and do not re-report it as an open blocker.
- P1 requires a separate authorization prompt. Do not enter it.

## Continuation rules

- Re-run the read-only Git preflight before writing, and report start HEADs again; do not trust
  the HEAD recorded in this file.
- Never clean, reset, stash or restore to get a tidy tree. User work outranks tree cleanliness.
- Reports cite real commands and real output. Unmeasured stays `NOT MEASURED` with a reason.
- Do not weaken or delete a test to reach green.
- Do not edit V0 evidence, and do not turn the V0 gate recommendation into a PASS.
- Measure before describing a build. Two claims in this pack were false until re-measured: the
  fixture `fixture.toml` linker invocation, and the desktop binary "embedding the built UI"
  (it did not, without `custom-protocol`). Write what the command printed, not what it should print.
- Read remote CI with `gh run view`, not from this repository's own reports. A run's ID, head SHA and
  per-job conclusion are external facts, and a report that describes them is a claim about a moment
  that has already moved.
- Quote any `BASELINE.yaml` value containing ` #`. An unquoted `Run #2` silently truncates the scalar
  at the comment, and the file still parses - which is how `remote_state` shipped one round describing
  a run by half its name. Check with `python -c "import yaml; …"` after editing, not by eye.
