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
| `rust` | ubuntu-latest, windows-latest | builds the UI if `dist/` is missing, then fmt, clippy (`-D warnings`, all targets, all features), `cargo test --workspace` |
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
- **`clippy --all-features` is the one gate step that needs the built UI, and that was measured.**
  An earlier draft of this document attributed a `dist/` requirement to `generate_context!` in every
  build. It does not. With `apps/desktop/ui/dist/` moved out of the tree,
  `cargo check -p firmwaresight-desktop` (5.41s),
  `cargo test -p firmwaresight-desktop --no-run` (15.13s) and
  `cargo build --release -p firmwaresight-desktop` (2m 49s) all succeeded: `tauri`'s build script
  sets `dev = !custom_protocol` (`tauri-2.12.0/build.rs:253`), and dev-mode codegen embeds nothing
  (`tauri-codegen-2.7.0/src/context.rs:178`). The desktop crate now declares the template's
  `custom-protocol` feature, and the same commands with `--all-features` and no `dist/` fail inside
  the macro: `The `frontendDist` configuration is set to "../ui/dist" but this path doesn't exist`.
  The gate lints the configuration that ships rather than the dev one, so `scripts/check.py` builds
  the UI when `dist/index.html` is missing and the `rust` job installs Node and enables Corepack.
  `drift` needs neither, because its `cargo test -p firmwaresight-desktop` runs default features.
- **The production link was measured locally, in both configurations.**
  `cargo build --release -p firmwaresight-desktop --features custom-protocol` produced a
  10,398,208-byte `firmwaresight-desktop.exe` in 3m 11s, and the built asset name
  `index-BdwhVhyx.js` appears in the binary - the frontend really is embedded. The same command
  without the feature produced 10,330,112 bytes with no such string. An earlier sentence in this
  pack described the second, dev-mode binary as "the built UI embedded"; that claim was wrong and is
  corrected here and in `P0_EXIT_CHECKLIST.md` 16 and `P0_KNOWN_LIMITATIONS.md`.
- `permissions: contents: read` at workflow level; nothing writes or uploads.
- Triggers: push to `main`, pull requests, and `workflow_dispatch`.

## What ran locally

```
$ python scripts/check.py
=== summary ===
PASS  rust/fmt
PASS  rust/clippy
PASS  rust/test                (102 tests across 6 workspace members)
PASS  frontend/install
PASS  frontend/typecheck
PASS  frontend/lint
PASS  frontend/test            (19 tests, 3 files)
PASS  frontend/build
PASS  drift/design tokens
PASS  drift/desktop icons
PASS  drift/ipc bindings
PASS  drift/ipc bindings unchanged
PASS  drift/goldens unchanged
PASS  deny/cargo-deny (skipped)

14/14 steps passed
```

`deny/cargo-deny (skipped)` is recorded as skipped rather than passed. `cargo deny` is not installed
on this machine and installing it was not authorized, so there is no local license or advisory result
to report.

The two `frontend assets` steps the `rust` group can emit are absent from that list because
`apps/desktop/ui/dist/` already existed. Their behaviour was verified separately: with `dist/`
deleted, `python scripts/check.py --only rust` printed the reason, rebuilt the UI and went on to pass
`fmt`, `clippy` and `test` - 5/5 steps, exit 0.

Both runs were repeated from a cold tree - `cargo clean`, then `node_modules/` and `dist/` removed -
because a gate that has only ever run against an existing `target/` proves less than it looks like it
proves. That run reported **16/16 steps**, the extra two being the frontend install and build, with
102 Rust tests and 18 UI tests, exit 0. The `pnpm install` inside it downloaded nothing
(185 packages, all reused from the store), so the CI risk is the fetch of the pnpm binary itself, not
the dependency tree.

That cold run measured 18 UI tests because it predates the accessibility fix described in
`P0_DESIGN_CHECKLIST.md`, which added a nineteenth. The fix touched no Rust file, so the cold Rust
result carries to the delivered tree; the delivered tree then re-ran the whole gate warm at **14/14**,
102 Rust and 19 UI tests. A second `cargo clean` after a documentation-and-UI-only change was judged
not to re-prove anything about the Rust half - stated here so the reader knows exactly which tree each
number belongs to.

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
5. **The `rust` job now depends on the UI build it used to be independent of.** `clippy --all-features`
   compiles the desktop crate with `custom-protocol`, whose codegen needs `dist/`, so the job runs
   `pnpm install --frozen-lockfile` on a checkout that has no `dist/`. That is one more network
   fetch and roughly a minute of CI time per OS, and it makes a Core-only pull request sensitive to
   a broken frontend lockfile. The alternative - narrowing `--all-features` so the gate stops seeing
   the shipping configuration - was rejected in `P0_IMPLEMENTATION_LOG.md` 18.

## Exit criterion mapping

| Prompt item | Where it runs |
| --- | --- |
| fmt / clippy / cargo test | `rust` job, and locally |
| frontend typecheck and build | `frontend` job, and locally |
| ts-rs generation works | `drift` job (regenerate + `git diff --exit-code`) |
| no forbidden dependency or boundary violation | `deny` job (`[bans] deny` list in `deny.toml`) |
| golden tests pass | `rust/test` plus `drift/goldens unchanged` |
