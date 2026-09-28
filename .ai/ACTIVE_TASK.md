---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.5.1"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-09-28"
---

# ACTIVE TASK

```text
P0_CI_CLOSURE_REMEDIATION
```

`FirmwareSight P0 — CI Closure / Cross-Platform Reproducibility Remediation v1.0 (Architect
Reviewed)`. Target: `LOCAL REMEDIATION COMPLETE / READY FOR REMOTE CI RERUN`. Not P1, not `v0.6.0`,
not a P0 `PASS` claim.

## Why this task exists

The P0 slice reported `CONDITIONAL_PASS (LOCAL)` because its three open conditions were outside the
authoring environment. The repository owner pushed `f9b8ccb`, remote Actions run `36360310447`
executed, and it concluded `failure` with two of six jobs green. Two of those conditions were then
tested and failed, so the honest status is `FAIL — REMOTE CI RUN #1`.

| Failure | Root cause, measured |
| --- | --- |
| Rust (windows-latest) | `.gitattributes` has no rule for `*.ld`, so `core.autocrlf=true` smudged the checkout from the recorded LF bytes to CRLF and the fixture hash test compared different bytes |
| Rust (ubuntu-latest) | No Tauri Linux system prerequisites on the runner; `glib-sys` fails at `pkg-config cannot find glib-2.0` before the tests run |
| Generated output drift | `scripts/gen_desktop_icons.py --check` asserts byte equality against Pillow's compressed output, which is encoder-state dependent across OSes |
| Dependency policy | `deny.toml` does not parse under `cargo-deny 0.20.2`: unsupported `[bans] highlight-warnings` key, and five `licenses.allow` entries are slash strings instead of SPDX |
| (no job) | `05_ENGINEERING/06_CI_CD_BASELINE.md` requires a macOS core smoke on `main` push; the workflow has none |

## Scope

Ten ordered phases: governance truth sync; fixture text policy; pixel-based icon drift check;
cargo-deny schema and real license run; Ubuntu prerequisites; macOS core smoke; full local
regression; one real desktop window launch; evidence refresh; small local commits, then stop.

Authorized within this task: installing `cargo-deny@0.20.2`, running the four `cargo deny` checks,
and starting the desktop app once in the current user session with `--features custom-protocol`.

## Explicitly out of scope

- Pushing. The owner pushes; the coding side reports the HEAD and waits.
- `v0.6.0`, `P0 PASS`, `G1 PASS`, `REMOTE CI PASS` written before a new run exists.
- P1 analyzer, Compare, full Gate, Release Bundle, new artifact formats, cloud, accounts, AI,
  telemetry, updater, SBOM/CVE, SQLx, wgpu, a fifth Phase-0 library crate, UI redesign.
- `SHA256SUMS` (frozen v0.5.1 package integrity record), `V0_VALIDATION/**`, `ADR-0020`, design
  tokens and the icon design, and the fixture hash assertion itself.
- Deleting a failing test, lowering an assertion, batch-regenerating goldens, or turning a `FAIL`
  into a `SKIPPED` by editing `scripts/check.py`.

## Gate status

```text
Formal G1: NOT CLAIMED  (g1_requires V0_PASS + P0_PASS; V0 still unvalidated)
P1:        NOT AUTHORIZED
P0:        FAIL — REMOTE CI RUN #1; remediation ACTIVE
```

## Version gate

`baseline_version` stays `0.5.1`. `v0.6.0` requires a real P0 `PASS`, which requires an all-green
Actions run on a HEAD that does not exist yet. A `FAIL` must not be relabeled into a PASS baseline.

## Durable facts preserved by this change

V0 remains `DEFERRED / NOT YET EVIDENCE-VALIDATED — NON-BLOCKING RESEARCH TRACK`. Formal eligible
external participants completed `0 / 8 minimum`; Batch A target `0 / 4–5`. The V0 blocker is the
absence of real human participants, not a technical failure. `V0_VALIDATION/` and every Batch A
recruitment artifact stay intact and unmodified.

Peak RSS stays `NOT MEASURED` with its reason in `P0_PERFORMANCE_REPORT.md`; it is recorded as a
measurement gap, not a promotion blocker.
