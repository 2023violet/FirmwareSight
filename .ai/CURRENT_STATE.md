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

Status: `FAIL — REMOTE CI RUN #1` · Remediation: `LOCAL FIX COMPLETE` · Remote rerun: `REQUIRED`

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

### What the remediation changed, and what it did not

All five causes were reproduced with a command before anything was edited, and each fix is a local
`PASS` on the same command that failed in CI:

| Cause | Fix | Local proof |
| --- | --- | --- |
| Checkout changed fixture bytes | `.gitattributes` text policy (`*.ld text eol=lf`, `*.map -text`, binaries unchanged) | fresh `git clone` → recorded hashes match; `cargo test` green |
| Ubuntu runner lacked Tauri prerequisites | apt step installing the nine documented packages plus `libdbus-1-dev`, `if: matrix.os == 'ubuntu-latest'` | coverage unchanged — `clippy --all-features` still compiles the desktop crate |
| Icon check compared encoder bytes | decode and compare: PNG dimensions + RGBA, ICO required size set + per-frame pixels, missing frame fails | `gen_desktop_icons.py --check` passes on this host |
| `deny.toml` unparseable by 0.20.2 | rewritten against the keys the tool accepts; allow list rebuilt from `cargo deny list` | `cargo deny check licenses bans sources advisories` exits 0 |
| No macOS job | `macos-core` job running `check.py --only core-smoke` on `main` pushes | 3/3 steps green locally |

The authorized desktop window launch then found a **sixth problem that no test had reached**: the
second artifact failed with `UNIQUE constraint failed: evidence.id`, because `evidence` had a
whole-table primary key while `04_TECH/15` §4 declares `Build 1─N Evidence`. Migration `0002`
rebuilds the table on `(build_id, id)`; schema version is now 2, Rust tests are 104 (storage 11), and
the upgrade was run against the real database the earlier launch had left behind. Full record:
`P0_TECHNICAL_VALIDATION/P0_DESKTOP_SMOKE_REPORT.md`.

Two advisories (`RUSTSEC-2024-0429`, `RUSTSEC-2024-0370`) are **not resolvable inside the frozen
Tauri 2.12.0 dependency architecture** — `cargo update -p glib --precise 0.20.0` fails against
`gtk = "^0.18"`. They are ignored with recorded evidence and reported to the architect as an
architecture conflict needing an ADR, not closed by a CI change.

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
P0: FAIL — REMOTE CI RUN #1 (2 of 6 jobs green); remediation LOCAL FIX COMPLETE, rerun REQUIRED
Formal G1: NOT CLAIMED (requires V0_PASS and P0_PASS)
P1: NOT AUTHORIZED
```

Full reasoning: `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md`; per-item evidence:
`P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`; run log and the four fixes:
`P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md` and
`P0_TECHNICAL_VALIDATION/P0_CI_REMEDIATION_REPORT.md`; the shipped-window result:
`P0_TECHNICAL_VALIDATION/P0_DESKTOP_SMOKE_REPORT.md`.

## Version rule

`v0.6.0` is not created unless P0 reaches `PASS` with real engineering evidence. P0's current
status is `FAIL`, so `baseline_version` stays `0.5.1`. A failed run must not be relabeled into a
PASS baseline, and a locally green gate must not be relabeled into a CI PASS.

## Next work

The remediation's scope is complete locally: the five CI causes are fixed, the gate runs with zero
skipped mandatory steps, and the desktop window has been launched and driven for real. One action
remains and it is not the coding agent's:

**Push the reported remediation HEAD to `origin/main` and read the new
`P0 verification gate` run.** That run is what turns `LOCAL FIX COMPLETE` into a remote result, and
it is verified by the architect — the coding side does not write `REMOTE CI PASS`.

What the push will test that this machine cannot: a Linux checkout of the `.gitattributes` policy, an
Ubuntu runner installing the Tauri prerequisites from apt, the pixel-based icon check under a
different Pillow build, the pinned `cargo-deny 0.20.2` against the rewritten policy, and the macOS
core smoke for the first time. If any of them fails, the status stays `FAIL` and `v0.6.0` stays
withheld.

`v0.6.0` is generated only after a real CI `PASS`; G1 requires V0 as well; P1 requires its own
authorization prompt, which has not been issued.

V0 resumes only when a real eligible participant/session source exists.
