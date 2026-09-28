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

Status: `FAIL — REMOTE CI RUN #1` · Remediation: `ACTIVE`

The source tree exists and its claims are proven by executed tests: 102 Rust tests, 19 UI tests,
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
P0: FAIL — REMOTE CI RUN #1 (2 of 6 jobs green); remediation ACTIVE
Formal G1: NOT CLAIMED (requires V0_PASS and P0_PASS)
P1: NOT AUTHORIZED
```

Full reasoning: `P0_TECHNICAL_VALIDATION/P0_TECHNICAL_VALIDATION_REPORT.md`; per-item evidence:
`P0_TECHNICAL_VALIDATION/P0_EXIT_CHECKLIST.md`; run log: `P0_TECHNICAL_VALIDATION/P0_CI_REPORT.md`.

## Version rule

`v0.6.0` is not created unless P0 reaches `PASS` with real engineering evidence. P0's current
status is `FAIL`, so `baseline_version` stays `0.5.1`. A failed run must not be relabeled into a
PASS baseline, and a locally green gate must not be relabeled into a CI PASS.

## Next work

`active_task` is the CI closure remediation authorized by the architect prompt, and its scope ends
where that prompt ends. The five things it must fix, all measured rather than assumed:

1. the Git text policy that lets a checkout change fixture bytes;
2. the Ubuntu runner's missing Tauri system prerequisites, fixed by installing them rather than by
   narrowing `clippy` coverage;
3. an icon drift check that compares pixels instead of third-party encoder output;
4. a `deny.toml` that `cargo-deny 0.20.2` can actually parse, then a real license/ban/advisory run;
5. the macOS core smoke the CI baseline requires and the workflow omits.

Then the desktop window launch, which is now an explicit part of this round:
`pnpm -C apps/desktop/ui build` followed by
`cargo run --release -p firmwaresight-desktop --features custom-protocol`. Without that feature the
binary is a dev-mode build that expects the Vite dev server, so the launch would prove nothing about
the shipped configuration.

Pushing the remediation is the owner's action. Only a new Actions run on the new HEAD can turn P0
into `PASS`, and only then may `v0.6.0` be considered.

V0 resumes only when a real eligible participant/session source exists.
