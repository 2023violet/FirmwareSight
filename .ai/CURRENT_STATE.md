---
title: "Current Project State"
doc_id: "FS-AI-002"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# Current State

Date: 2026-09-28  
Baseline: v0.5.1

## Product/architecture baseline

v0.5.0 product/architecture/design decisions remain inherited. v0.5.1 is the active baseline.
The P0 slice created the first production code; nothing in the baseline was renegotiated to do it.

## P0 — Technical Vertical Slice

Status: `CONDITIONAL_PASS` · Remote CI Run #3 (`36399805005`, head `1cd6309`): **`success`, 7 of 7 jobs green** · Outstanding condition: the architect's promotion sign-off

The source tree exists and its claims are proven by executed tests: 104 Rust tests, 19 UI tests,
one shared gate script, real ELF/MAP fixtures with recorded provenance, deterministic CLI JSON,
memory accounting reproduced by hand from `readelf`, a typed IPC boundary with generated TypeScript,
SQLite migrations and transactional import, and a 512 MiB guard measured from both sides of the
boundary. None of that is in question.

What is in question is that the same tree fails on the platforms that were not measured locally.
`origin/main` at `f9b8ccb` ran `.github/workflows/p0-check.yml` as run `36360310447`; it concluded
`failure` with two jobs green and four red:

| Job | Observed cause |
| --- | --- |
| Rust (windows-latest) | `committed_fixtures_match_their_recorded_hashes` — Git smudged `p0-dual-region.ld` from LF to CRLF at checkout, so the bytes no longer match the recorded hash |
| Rust (ubuntu-latest) | `glib-sys` build script: `pkg-config cannot find glib-2.0` — the runner has none of the Tauri Linux system prerequisites |
| Generated output drift | `scripts/gen_desktop_icons.py --check` compares raw encoder bytes; Pillow's zlib output differs between OSes while the pixels do not |
| Dependency policy | `deny.toml` is not parseable by `cargo-deny 0.20.2`: `[bans] highlight-warnings` is not a supported key and five `licenses.allow` entries are slash strings, not SPDX |

A fifth gap came from the frozen baseline rather than from a red job: `05_ENGINEERING/06_CI_CD_BASELINE.md`
requires a macOS core smoke on `main` push, and the workflow has no macOS job at all.

Per-run evidence, retained as failed history rather than rewritten: `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`.

### Remote Run #2, read from GitHub with `gh run view 36378384225`

The remediation HEAD `ebda52d` was pushed and ran. It concluded `failure` with **six of seven jobs
green**, which closes Run #1's four causes remotely:

| Job | Run #2 |
| --- | --- |
| Rust (windows-latest) | PASS - 5/5 steps, 104 tests |
| Rust (ubuntu-latest) | PASS - 5/5 steps, 104 tests, with `firmwaresight-desktop` compiling |
| Desktop UI (windows-latest / ubuntu-latest) | PASS - 19 UI tests |
| macOS Core Smoke | PASS - 3/3 steps, 87 tests, first execution of the job |
| Dependency policy | PASS - `advisories ok, bans ok, licenses ok, sources ok` |
| Generated output drift | **FAIL** - tokens and icons pass, then `drift/ipc bindings` dies in `gobject-sys`' build script |

The remaining failure is CI provisioning, not product source: `scripts/check.py --only drift`
regenerates the ts-rs bindings by running `cargo test -p firmwaresight-desktop`, which needs the same
GTK system libraries the `rust` job now installs - and the `drift` job was never given that step.
Run #1's `Rust (ubuntu-latest)` died the same way at `glib-sys`; this is the same missing
`.pc` files, reached from a second job. The fix is 21 added lines in
`.github/workflows/p0-check.yml`: the already-remotely-proven apt block, copied, with no package
added and no step, skip or permission changed. No Rust, TypeScript, fixture, golden or policy file
moved. `P0_TECHNICAL_VALIDATION/P0_CI_RUN_2_CLOSURE_REPORT.md` is the round's record.

### What the remediation changed, and what it did not

All five causes were reproduced with a command before anything was edited. Each fix was a local
`PASS` on the same command that failed in CI, and **Run #2 then confirmed all five remotely**:

| Cause | Fix | Proof |
| --- | --- | --- |
| Checkout changed fixture bytes | `.gitattributes` text policy (`*.ld text eol=lf`, `*.map -text`, binaries unchanged) | local fresh `git clone` → recorded hashes match; **remote `Rust (windows-latest)` PASS at 104 tests** |
| Ubuntu runner lacked Tauri prerequisites | apt step installing the nine documented packages plus `libdbus-1-dev`, `if: matrix.os == 'ubuntu-latest'` | coverage unchanged — `clippy --all-features` still compiles the desktop crate; **remote `Rust (ubuntu-latest)` PASS** |
| Icon check compared encoder bytes | decode and compare: PNG dimensions + RGBA, ICO required size set + per-frame pixels, missing frame fails | `gen_desktop_icons.py --check` green here; **remote `drift/desktop icons` PASS on Linux** with `pixel-identical to this build` |
| `deny.toml` unparseable by 0.20.2 | rewritten against the keys the tool accepts; allow list rebuilt from `cargo deny list` | exits 0 here; **remote `Dependency policy` PASS**, four categories ok |
| No macOS job | `macos-core` job running `check.py --only core-smoke` on `main` pushes | 3/3 locally; **remote `macOS Core Smoke` PASS at 87 tests** |

The authorized desktop window launch then found a **sixth problem that no test had reached**: the
second artifact failed with `UNIQUE constraint failed: evidence.id`, because `evidence` had a
whole-table primary key while `04_TECH/15` §4 declares `Build 1─N Evidence`. Migration `0002`
rebuilds the table on `(build_id, id)`; schema version is now 2, Rust tests are 104 (storage 11), and
the upgrade was run against the real database the earlier launch had left behind. Full record:
`P0_TECHNICAL_VALIDATION/P0_DESKTOP_SMOKE_REPORT.md`.

Two advisories (`RUSTSEC-2024-0429`, `RUSTSEC-2024-0370`) are **not resolvable inside the frozen
Tauri 2.12.0 dependency architecture** — `cargo update -p glib --precise 0.20.0` fails against
`gtk = "^0.18"`. They were reported to the architect as an architecture conflict; the architect's
disposition is now recorded: **accepted as explicit, documented P0 transitive risk that does not block
P0 promotion**, with five revisit triggers, and **no architecture ADR**, because no architecture choice
changed. See `.ai/DECISIONS.md` and `P0_DEPENDENCY_REPORT.md`.

## V0

Status:

`DEFERRED / NOT YET EVIDENCE-VALIDATED — NON-BLOCKING RESEARCH TRACK`

This is a re-sequencing of the research track, not a completion claim.

Completed:
- clickable prototype;
- fixture/state machine;
- protocol/session templates;
- internal functional dry run;
- Batch A takeover;
- Batch A recruitment-ready package.

Formal external sessions:
`0 / 4–5 Batch A target`
`0 / 8 V0 minimum`

The V0 blocker remains real human participants. Deferred does not mean passed, and P0 progress
does not close this gap. P0 changed no V0 artifact and no V0 recommendation.

## Gates

```text
G0: PASS
V0: DEFERRED / UNVALIDATED (0 of 8 eligible external sessions)
P0: CONDITIONAL_PASS — remote CI Run #3 success, 7 of 7 jobs green; condition = architect promotion sign-off
Formal G1: NOT CLAIMED (requires V0_PASS and P0_PASS; V0 still unvalidated)
P1: NOT AUTHORIZED
```

Full reasoning: `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md`; per-item evidence:
`P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`; Run #1 log and its four fixes:
`P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` and
`P0_TECHNICAL_VALIDATION/P0_CI_REMEDIATION_REPORT.md`; Run #2 and the drift provisioning closure:
`P0_TECHNICAL_VALIDATION/P0_CI_RUN_2_CLOSURE_REPORT.md`; the shipped-window result:
`P0_TECHNICAL_VALIDATION/P0_DESKTOP_SMOKE_REPORT.md`.

## Version rule

`v0.6.0` is not created unless P0 reaches `PASS` with real engineering evidence. P0's current
status is `FAIL`, so `baseline_version` stays `0.5.1`. A failed run must not be relabeled into a
PASS baseline, and a locally green gate must not be relabeled into a CI PASS.

## Next work

Both remediation rounds are now confirmed remotely. Run #3 on the pushed HEAD `1cd6309` concluded
`success` with all seven jobs green, `Generated output drift` included - the provisioning step whose
absence Run #2 measured executed (`Setting up libwebkit2gtk-4.1-dev`), and the drift gate passed 5/5.
Read it with `gh run view 36399805005 --repo 2023violet/FirmwareSight`, not from this file.

One action remains, and it is not a coding action:

**The architect issues and signs the *P0 Final Promotion / v0.6.0 Baseline Closure* prompt.** That is
where `P0 = PASS`, `baseline_version: 0.6.0`, a regenerated `SHA256SUMS` and `DIRECTORY_TREE`, and
`active_task` are written - not here. Until that prompt exists this track reads `CONDITIONAL_PASS`
with the condition named, and `v0.6.0` stays undelivered even though the engineering evidence is
complete.

What a green CI does not change: G1 still requires V0 (`0 / 8` eligible external sessions); P1 still
requires its own authorization prompt; peak RSS is still `NOT MEASURED`; and the two RustSec advisories
stay recorded under the architect's acceptance with their five revisit triggers.

`v0.6.0` is generated only after a real CI `PASS`; G1 requires V0 as well; P1 requires its own
authorization prompt, which has not been issued. If Run #3 is green, the next prompt is the
architect's P0 Final Promotion / v0.6.0 Baseline Closure, and it is not this round's to anticipate.

V0 resumes only when a real eligible participant/session source exists.
