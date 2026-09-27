---
title: "P0 CI Report"
doc_id: "FS-P0-014"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 CI Report

Requirement: a gate that runs the same checks a developer runs, so "CI is green" and "my machine
is green" cannot mean different things.

Status: **LOCAL PASS / CI NOT RUN**

The workflow is committed and syntactically validated. It has **never executed**, because this
task has no push authorization. Every statement below about CI is about what the file will do,
not about a result it produced.

## One definition, two callers

`.github/workflows/p0-check.yml` contains no test commands. It installs the toolchain and calls:

```
python scripts/check.py --only rust
python scripts/check.py --only frontend
python scripts/check.py --only drift
python scripts/check.py --only deny
```

`scripts/check.py` prints each command line before running it, so a CI failure is reproducible by
pasting one line locally. If a step is added to CI, it is added to the script - which is also the
only way it becomes runnable on a laptop.

## Jobs

| Job | Runner | Runs |
| --- | --- | --- |
| `rust` | ubuntu-latest, windows-latest | fmt, clippy (`-D warnings`, all targets, all features), `cargo test --workspace` |
| `frontend` | ubuntu-latest, windows-latest | `pnpm install --frozen-lockfile`, typecheck, lint, test, build |
| `drift` | ubuntu-latest | tokens.css, desktop icons, ts-rs bindings and goldens must be unchanged after regeneration |
| `deny` | ubuntu-latest | `cargo install cargo-deny@0.20.2 --locked`, then `cargo deny check licenses bans sources advisories` |

Notes on choices:

- **Two OSes, not three.** macOS is not claimed. The toolchain is pinned to 1.98.1 by
  `rust-toolchain.toml`, so rustup installs it before the first cargo command and no CI step has
  to remember a version.
- **No ARM toolchain in CI.** The fixtures are committed binaries with recorded provenance, so CI
  tests the parser rather than a compiler most contributors do not have.
- **Corepack, not a global pnpm.** `corepack enable` plus the `packageManager` field guarantees the
  lockfile is resolved by the version that wrote it.
- **No caching action, no third-party install action.** Caching and `taiki-e/install-action` were
  considered and dropped: `cargo install cargo-deny` costs a few minutes of CI time, and a smaller
  set of third-party code executed on every push is the better trade for a repository whose
  current job is verification.
- `permissions: contents: read` at workflow level; nothing writes or uploads.
- Triggers: push to `main`, pull requests, and `workflow_dispatch`.

## What ran locally

```
$ python scripts/check.py
=== summary ===
PASS  rust/fmt
PASS  rust/clippy
PASS  rust/test                (101 tests across 6 workspace members)
PASS  frontend/install
PASS  frontend/typecheck
PASS  frontend/lint
PASS  frontend/test            (18 tests)
PASS  frontend/build
PASS  drift/design tokens
PASS  drift/desktop icons
PASS  drift/ipc bindings
PASS  drift/ipc bindings unchanged
PASS  drift/goldens unchanged
PASS  deny/cargo-deny (skipped)
```

The `deny` line is recorded as skipped rather than passed. `cargo deny` is not installed on this
machine and installing it was not authorized, so there is no local license or advisory result to
report.

`drift/goldens unchanged` failed on its first run and passed after the refresh described in
`P0_IMPLEMENTATION_LOG.md` 17 was committed. That is the check behaving correctly: it caught a
working tree where a generated file had been rewritten but not yet committed.

## Known risks in the CI definition, stated before the first run

1. **Windows clippy over the Tauri tree** is the least certain step. Locally it is clean with
   `-D warnings`, but a different MSVC point release could surface a lint nobody has seen. If it
   fails, the honest fix is in the code, not by relaxing `-D warnings`.
2. **`cargo deny check advisories`** depends on the live RustSec database, so an advisory
   published against a Tauri transitive dependency can fail CI for reasons unrelated to a change.
   `severity-threshold = "medium"` bounds this; it does not eliminate it.
3. **`jsdom` on Windows CI** has been reliable locally on Node 24 but is the likeliest place for an
   environment-specific failure.
4. **Runtime.** A two-OS matrix building the Tauri dependency tree without caching is minutes per
   job. Accepted deliberately for P0.

## Exit criterion mapping

| Prompt item | Where it runs |
| --- | --- |
| fmt / clippy / cargo test | `rust` job, and locally |
| frontend typecheck and build | `frontend` job, and locally |
| ts-rs generation works | `drift` job (regenerate + `git diff --exit-code`) |
| no forbidden dependency or boundary violation | `deny` job (`[bans] deny` list in `deny.toml`) |
| golden tests pass | `rust/test` plus `drift/goldens unchanged` |
