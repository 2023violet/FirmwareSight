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
P0_CI_RUN_2_FINAL_DRIFT_CLOSURE
```

`FirmwareSight P0 — Remote CI Run #2 Final Drift Closure v1.0 (Architect Reviewed)`. Target:
`LOCAL FIX COMPLETE / READY FOR REMOTE CI RUN #3`. Not P1, not `v0.6.0`, not a P0 `PASS` claim, not an
architecture redesign.

## Why this task exists

The first remediation round fixed four CI failures and the missing macOS job. The owner pushed its
HEAD `ebda52d`, GitHub Actions run `36378384225` executed, and it concluded `failure` with **six of
seven jobs green** - so that round's work is confirmed remotely, and one job is left.

| Run #2 job | Result |
| --- | --- |
| `Rust (windows-latest)` | PASS - 104 tests |
| `Rust (ubuntu-latest)` | PASS - 104 tests, desktop crate compiling |
| `Desktop UI (windows-latest)` | PASS |
| `Desktop UI (ubuntu-latest)` | PASS - 19 UI tests |
| `macOS Core Smoke` | PASS - 87 tests, first execution |
| `Dependency policy` | PASS - advisories / bans / licenses / sources ok |
| `Generated output drift` | **FAIL** |

The failing step is `drift/ipc bindings`. `scripts/check.py --only drift` regenerates the ts-rs
bindings by running `cargo test -p firmwaresight-desktop`, which links the GTK stack, and the `drift`
job - unlike the `rust` job - had no step installing the Tauri Linux prerequisites. Its log says
`Package gobject-2.0 was not found in the pkg-config search path`, the same missing `.pc` class that
killed `Rust (ubuntu-latest)` in Run #1 at `glib-sys`.

**This is a CI job provisioning duplication defect.** Not a Core defect, not a ts-rs contract defect,
not generated drift, not a Tauri source defect.

## Scope

One workflow file, plus the governance and evidence record of Run #2. Minimal change, no abstraction:
the already-remotely-proven apt block is copied into the `drift` job rather than extracted into a new
script, because only two jobs need it.

## Explicitly out of scope

- Pushing. The owner pushes; the coding side reports the HEAD and waits.
- Writing `REMOTE CI PASS`, `P0 PASS`, `G1 PASS`, or generating `v0.6.0`.
- Any product source: `crates/**`, `apps/cli/**`, `apps/desktop/**`, `fixtures/**`, `golden/**`,
  `schemas/**`, `scripts/check.py`, `deny.toml`, `Cargo.lock`, `pnpm-lock.yaml`,
  `assets/design-tokens.json`, `V0_VALIDATION/**`, `SHA256SUMS`.
- A fifth Phase-0 library crate, moving IPC DTOs into Core, changing the Tauri version, the gtk-rs
  line or Cargo features, or a dependency-architecture migration.
- Weakening the drift check: deleting the desktop test command, dropping the IPC bindings step,
  marking it optional, moving the job to Windows, `continue-on-error`, `if: false`, or turning the
  failure into a warning.
- Removing the macOS job, the Linux CI or the cargo-deny job; changing workflow triggers or
  `permissions: contents: read`.
- Chasing the 23 duplicate-version warnings or upgrading dependencies to reduce noise.
- Re-running the desktop smoke for a third observation when no runtime source changed.

## Where this task stands

```text
Remote CI run #1:    FAILURE at f9b8ccb — retained as history
Remote CI run #2:    FAILURE at ebda52d — 6 of 7 jobs PASS; latest_remote_ci
Round 1 remediation: CONFIRMED REMOTELY (fixture bytes, Ubuntu Rust, icon semantics, deny, macOS)
Round 2 fix:         drift job installs the proven prerequisites; workflow-only, 21 added lines
Local gate:          PASS, 14/14 default steps, 0 SKIPPED mandatory steps
Desktop smoke:       PASS, carried forward — this round changes no runtime source
Remote CI:           RUN #3 REQUIRED AFTER USER PUSH
```

Two facts recorded honestly rather than smoothed over. The Linux provisioning step is **NOT LOCALLY
EXECUTED** - this host has no Ubuntu; the evidence chain is the same runner family, the same package
list already proven remotely, and the same compile requirement. And while writing this round's
records, four values in `BASELINE.yaml` were found to be silently truncated by YAML's inline-comment
rule (an unquoted `Run #2` ends the scalar at the `#`) - a documentation defect introduced by the
previous round's commit, fixed by quoting them in this one.

## Advisory disposition from the architect

`RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`,
unmaintained) are **accepted as explicit P0 transitive risk, not silent suppression, and do not block
P0 promotion** on current evidence. They stay recorded in `P0_DEPENDENCY_REPORT.md`,
`P0_KNOWN_LIMITATIONS.md` and `deny.toml`'s reasons, with five revisit triggers. No architecture ADR is
required, because no architecture choice changed; an ADR becomes necessary if Tauri is replaced,
dependencies are forked, or the frozen desktop dependency family changes.

## Gate status

```text
Formal G1: NOT CLAIMED  (g1_requires V0_PASS + P0_PASS; V0 still unvalidated)
P1:        NOT AUTHORIZED
P0:        FAIL — REMOTE CI RUN #2; remediation round 2 LOCAL FIX COMPLETE; RUN #3 REQUIRED
```

## Version gate

`baseline_version` stays `0.5.1`. `v0.6.0` requires a real P0 `PASS`, which requires an all-green
Actions run on a HEAD that does not exist yet. A `FAIL` must not be relabeled into a PASS baseline.

## Durable facts preserved by this change

V0 remains `DEFERRED / NOT YET EVIDENCE-VALIDATED — NON-BLOCKING RESEARCH TRACK`. Formal eligible
external participants completed `0 / 8 minimum`; Batch A target `0 / 4–5`. The V0 blocker is the
absence of real human participants, not a technical failure. `V0_VALIDATION/` and every Batch A
recruitment artifact stay intact and unmodified.

The first remediation round's own findings stay standing: the storage evidence key that needed
migration `0002` (schema version 2, Rust 102 -> 104) was found by the authorized window launch, and
`P0_DESKTOP_SMOKE_REPORT.md` remains its record, including the item it did not observe.

Peak RSS stays `NOT MEASURED` with its reason in `P0_PERFORMANCE_REPORT.md`; it is recorded as a
measurement gap, not a promotion blocker.

Run #1's four red jobs stay published as failed history in `P0_CI_REPORT.md`; Run #2 does not erase
them, and neither run's numbers are restated as better than they were.
