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

Status: **REMOTE CI RUN #3 AND RUN #4: SUCCESS — 7 OF 7 JOBS GREEN — P0 PROMOTED TO `PASS` AT v0.6.0**

The workflow has executed four times, and every conclusion below is read from GitHub with
`gh run view`, not from this repository's own reports.

- Run #1 (`36360310447`, head `f9b8ccb`): `failure`, 2 of 6 green. Kept as historical evidence.
- Run #2 (`36378384225`, head `ebda52d`): `failure`, 6 of 7 green. Run #1's four causes closed
  remotely; one job left red because a second CI job compiles the desktop crate without the Linux
  prerequisites the first one was given.
- Run #3 (`36399805005`, head `1cd6309`): **`success`, 7 of 7 green**, including the job Run #2
  could not pass.
- Run #4 (`36402637251`, head `5e58f77`): **`success`, 7 of 7 green** — the same workflow on the
  architect-reviewed HEAD, which differs from `1cd6309` by documentation only. It is the second green
  run, and it is the one that ran on the tree the architect read before signing.

P0's engineering evidence was complete on real runners at Run #3, and the promotion act that the line
above deferred to "the architect's next signed prompt" has now happened: Run #4 repeated the result on
the reviewed HEAD and the architect signed *P0 Final Promotion / v0.6.0 Baseline Closure v1.0*, which is
what wrote `P0 = PASS`, `baseline_version: 0.6.0` and the regenerated `SHA256SUMS`. This report claims
none of that on its own authority; it records what the runners measured. The first round's reasoning is in
`P0_CI_REMEDIATION_REPORT.md` and the second round's in `P0_CI_RUN_2_CLOSURE_REPORT.md`.

## Remote CI Run #1 — retained as failed evidence

| Field | Value |
| --- | --- |
| Repository | `2023violet/FirmwareSight` |
| Run id | `36360310447` |
| Workflow | `P0 verification gate` (`.github/workflows/p0-check.yml`) |
| Event / ref | `push` to `main` |
| Head SHA | `f9b8ccba7d9326ac397675602beb3a7e9c3bcd02` |
| Conclusion | `failure`, 5m 13s |
| Log | <https://github.com/2023violet/FirmwareSight/actions/runs/36360310447> |

| Job | Result | The error as it appeared |
| --- | --- | --- |
| Desktop UI (windows-latest) | pass | — |
| Desktop UI (ubuntu-latest) | pass | — |
| Rust (windows-latest) | **fail** | `committed_fixtures_match_their_recorded_hashes ... FAILED`, panic at `crates/firmwaresight-artifact/tests/p0_acceptance.rs:67`, `left: 7b872afd8bcc9205…` `right: f4bf022bb8562942…`, `test result: FAILED. 18 passed; 1 failed` |
| Rust (ubuntu-latest) | **fail** | `error: failed to run custom build command for glib-sys v0.18.1`; `pkg-config exited with status code 1`; `Package glib-2.0 was not found in the pkg-config search path.`; `FAILED: clippy (exit 101)` |
| Generated output drift | **fail** | `desktop icons are missing or stale:` `128x128.png`, `128x128@2x.png`, `icon.ico`; `FAIL(1) drift/desktop icons`; `1/2 steps passed` |
| Dependency policy | **fail** | `error[unexpected-keys]: found 1 unexpected keys, expected: [… "highlight", …]` at `deny.toml:24:1`; five `error[custom]: invalid character(s)` at `deny.toml:66:17 … 70:15`; `ERROR failed to deserialize config`; exit 1 |

The `cargo-deny` install step itself succeeded (`cargo install cargo-deny@0.20.2 --locked`), so the
`deny` failure was configuration, not a missing tool. A third invalid key,
`[advisories] severity-threshold`, was fatal too and only surfaced once the earlier two were fixed
locally — the run had stopped at the first deserialization error list.

Nothing in the two tables above is inferred: the messages are quoted from the job logs.

## Remote CI Run #2 — retained as failed evidence (6 of 7 green)

| Field | Value |
| --- | --- |
| Run | `36378384225`, event `push` |
| Head | `ebda52d63f22ff9f25786c1d80e53f5193cff183` — the first remediation HEAD, pushed by the owner |
| Conclusion | `failure` |
| Jobs | 7 — **6 pass, 1 fail** |
| Read with | `gh run view 36378384225 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,event,jobs` |

| Job | Conclusion | Step-level evidence from the job's own log |
| --- | --- | --- |
| Rust (windows-latest) | pass | `5/5 steps passed`; `test result: ok.` lines summing to **104** |
| Rust (ubuntu-latest) | pass | `5/5 steps passed`; **104**; `firmwaresight-desktop` compiles, which Run #1 never reached |
| Desktop UI (windows-latest) | pass | `5/5 steps passed` |
| Desktop UI (ubuntu-latest) | pass | `5/5 steps passed`; `Test Files 3 passed (3)`, `Tests 19 passed (19)` |
| macOS Core Smoke | pass | `3/3 steps passed`; **87** tests. First execution of this job in the repository's history |
| Dependency policy | pass | `1/1 steps passed`; `advisories ok, bans ok, licenses ok, sources ok` |
| Generated output drift | **fail** | `PASS drift/design tokens`, `PASS drift/desktop icons`, then `error: failed to run custom build command for gobject-sys v0.18.0` |

The `5/5` against the local `3/3` on the two Rust jobs is the `frontend assets` pair firing: a clean
runner has no `apps/desktop/ui/dist/`, so the gate builds the UI before `clippy --all-features`
compiles the desktop crate. That is the shipped configuration being exercised rather than the warm
local one.

Run #1's four causes are therefore closed **remotely**, not just locally - including
`desktop icons are current (pixel-identical to this build).` printed on a Linux runner, which is the
evidence the encoder-byte assertion could never produce.

### The one failure, exactly

Job `108788787601`, step `Verification gate (drift)`, third gate step:

```text
=== [drift] ipc bindings
$ cargo test -p firmwaresight-desktop
error: failed to run custom build command for `gobject-sys v0.18.0`
> PKG_CONFIG_PATH=/opt/hostedtoolcache/Python/3.13.15/x64/lib/pkgconfig \
  PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1 pkg-config --libs --cflags gobject-2.0 'gobject-2.0 >= 2.70'
  Package gobject-2.0 was not found in the pkg-config search path.
The system library `gobject-2.0` required by crate `gobject-sys` was not found.
error: failed to run custom build command for `glib-sys v0.18.1`   (same cause)
```

`scripts/check.py --only drift` regenerates the ts-rs bindings by running `cargo test -p
firmwaresight-desktop` - the only way to prove the committed `.ts` files still match the Rust DTOs -
and that command links the GTK stack. The `rust` job was given `Install Linux prerequisites for the
Tauri shell` in the first remediation round; the `drift` job was not, because the round was reasoning
from which jobs had failed rather than from which jobs compile the shell. A provisioning omission in
one job, in the same class as Run #1's, and fixed by giving the second job the same step.

`P0_CI_RUN_2_CLOSURE_REPORT.md` carries the round: the classification, the copy-not-abstract decision,
what Run #2 confirmed, the advisory disposition from the architect, and the local verification that
did run here.

## Remote CI Run #3 — the workflow is green

| Field | Value |
| --- | --- |
| Run | `36399805005`, event `push` |
| Head | `1cd6309a09313a0a900a83cb12e054e1a3d7c5e3` - the round-2 HEAD, pushed by the coding side on the owner's explicit instruction (`git push origin main`, no force) |
| Conclusion | **`success`** |
| Jobs | 7 of 7 success |
| Read with | `gh run view 36399805005 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs` |

| Job | Conclusion | Per-step evidence |
| --- | --- | --- |
| Rust (windows-latest) | success | `Verification gate (fmt, clippy, test)` green; one step `skipped` - the apt block guarded by `if: matrix.os == 'ubuntu-latest'` - which is the guard working, not a gap |
| Rust (ubuntu-latest) | success | 11/11 steps, including `Install Linux prerequisites for the Tauri shell` |
| Desktop UI (windows-latest) | success | 10/10 steps |
| Desktop UI (ubuntu-latest) | success | 10/10 steps |
| Generated output drift | success | **`5/5 steps passed`** after `Install Linux prerequisites for the Tauri shell` ran (`Setting up libwebkit2gtk-4.1-dev`), with `desktop icons are current (pixel-identical to this build).` |
| Dependency policy | success | 6/6 steps; `advisories ok, bans ok, licenses ok, sources ok` |
| macOS Core Smoke | success | 7/7 steps |

The drift job is the whole of what round 2 changed, and it is the line that proves the round's
classification was right: the same apt block, the same package list the `rust` job had already proven
remotely, placed in the second job that compiles `firmwaresight-desktop`. No check was weakened to get
there - `drift` still runs all five gate steps, `cargo test -p firmwaresight-desktop` included, and
both `git diff --exit-code` assertions are intact.

One honest qualification stays in the record: the Windows Rust job's skipped step means that job never
exercises the apt block (correctly - it is Linux-only), so "no step failed" and "every step ran" are
different statements. The `skipped` one is reported as skipped rather than counted as a pass.

## Remote CI Run #4 — the reviewed HEAD, green again

| Field | Value |
| --- | --- |
| Run | `36402637251`, event `push` |
| Head | `5e58f778aad35f33188b96d0b8873401a31ccc3c` — the architect-reviewed HEAD |
| Conclusion | **`success`** |
| Jobs | 7 of 7 success |
| Read with | `gh run view 36402637251 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,jobs` |

| Job | Conclusion |
| --- | --- |
| Rust (windows-latest) | success |
| Rust (ubuntu-latest) | success |
| Desktop UI (windows-latest) | success |
| Desktop UI (ubuntu-latest) | success |
| Generated output drift | success |
| Dependency policy | success |
| macOS Core Smoke | success |

Run #4 adds no new engineering fact and claims none. Its value is procedural: it executed the tree the
architect actually reviewed, which is `1cd6309` plus fifteen documentation files, so the signature rests
on a run of the reviewed bytes rather than on a diff between two runs. `git diff --name-only
1cd6309..5e58f77` lists those fifteen and contains no `crates/`, `apps/`, `fixtures/`, `golden/`,
`migrations/`, lockfile, `deny.toml`, `scripts/check.py` or workflow path — the check is quoted in
`P0_FINAL_PROMOTION_REPORT.md` §3.

## One definition, two callers

`.github/workflows/p0-check.yml` contains no test commands. It installs the toolchain and calls:

```
python scripts/check.py --only rust
python scripts/check.py --only frontend
python scripts/check.py --only drift
python scripts/check.py --only deny
python scripts/check.py --only core-smoke
```

`scripts/check.py` prints each command line before running it, so a CI failure is reproducible by
pasting one line locally. If a step is added to CI, it is added to the script - which is also the
only way it becomes runnable on a laptop.

## Jobs

| Job | Runner | Runs |
| --- | --- | --- |
| `rust` | ubuntu-latest, windows-latest | on Ubuntu only: install Tauri's Linux prerequisites first. Then builds the UI if `dist/` is missing, fmt, clippy (`-D warnings`, all targets, all features), `cargo test --workspace` |
| `frontend` | ubuntu-latest, windows-latest | `pnpm install --frozen-lockfile`, typecheck, lint, test, build |
| `drift` | ubuntu-latest | install Tauri's Linux prerequisites first (added after Run #2 - this job compiles the desktop crate too), then tokens.css, desktop icons (compared by pixels), ts-rs bindings and goldens must be unchanged after regeneration |
| `deny` | ubuntu-latest | `cargo install cargo-deny@0.20.2 --locked`, then `cargo deny check licenses bans sources advisories` |
| `macos-core` | macos-latest | `--only core-smoke`: the four library crates and the CLI, selected by package name. Skipped on pull requests for cost; runs on a push to `main` and on `workflow_dispatch` |

Seven jobs on a push to `main`, six on a pull request.

Notes on choices:

- **Three platforms, two depths.** Windows and Linux build and test everything, the Tauri shell
  included. macOS carries the required core smoke, because that is what the frozen event matrix asks
  for and because the shell would drag WebView prerequisites into a job whose purpose is Core. The
  toolchain is pinned to 1.98.1 by `rust-toolchain.toml`, so rustup installs it before the first cargo
  command and no CI step has to remember a version. An earlier draft of this file said "two OSes, not
  three" and "macOS is not claimed"; Run #1 plus `05_ENGINEERING/06_CI_CD_BASELINE.md` showed that was
  a gap rather than a decision.
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
  `drift` needs neither for `dist/`, because its `cargo test -p firmwaresight-desktop` runs default
  features - but it does need the GTK system libraries, which Run #2 is the first evidence of. The
  requirement is a property of compiling the shell on Linux, so it attaches to every job that does
  that, not to the job that happened to fail first.
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

### Before the remediation (tree `65cb2dc`, historical)

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

`deny/cargo-deny (skipped)` is what made that run weaker than it looks: the tool was not installed, so
the license and advisory result came from nowhere. The same tree, pushed as `f9b8ccb`, failed four CI
jobs. A locally green gate and a remotely red gate are not in contradiction - the local run measured
one platform, one checkout configuration and one encoder.

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

### After the remediation (the Run #2 and Run #3 tree)

```
$ python scripts/check.py
=== summary ===
PASS  rust/fmt
PASS  rust/clippy
PASS  rust/test                (104 tests across 6 workspace members)
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
PASS  deny/cargo-deny

14/14 steps passed
```

`deny/cargo-deny` is a real execution now: the tool is installed at `0.20.2` and printed
`advisories ok, bans ok, licenses ok, sources ok`. Zero steps reported `SKIPPED`, and the script was
not edited to produce that - the only change to the deny path is that the machine has the tool.

The two extra Rust tests are the storage pair written before the `evidence` key fix, so the count
moved 102 → 104 with a source change, not a documentation change. Separately:

```
$ python scripts/check.py --only core-smoke
PASS  core-smoke/fmt
PASS  core-smoke/clippy
PASS  core-smoke/test          (87 of the 104 Rust tests, no desktop crate in the selection)
3/3 steps passed
```

The 87 and 104 are not just local figures: Run #2's `Rust (windows-latest)` and
`Rust (ubuntu-latest)` logs each sum their `test result: ok.` lines to 104, and
`macOS Core Smoke` to 87. The round-2 change is one workflow step, so this gate result stands for
this tree as well - it was re-run after the workflow edit and reported the same 14/14 with exit 0.

## Known risks in the CI definition

1. **Windows clippy over the Tauri tree** is the least certain step. Locally it is clean with
   `-D warnings`, but a different MSVC point release could surface a lint nobody has seen. If it
   fails, the honest fix is in the code, not by relaxing `-D warnings`. Neither run hit it: Run #1's
   Windows Rust job failed on a fixture hash and Run #2's passed, so the lint surface has been clean
   on `windows-latest` twice.
2. **`cargo deny check advisories` depends on the live RustSec database**, so an advisory published
   against a Tauri transitive dependency can fail CI for reasons unrelated to any change. This risk
   materialized: `RUSTSEC-2024-0429` (glib, unsound) and `RUSTSEC-2024-0370` (proc-macro-error,
   unmaintained) both fired, and neither has an upgrade inside the frozen Tauri 2.12.0 tree. The
   previous sentence here claimed `severity-threshold = "medium"` bounded it - that key no longer
   exists in 0.20.2, which reports a vulnerability advisory as an error whatever its CVSS score is.
   What bounds it now is a two-entry `ignore` list carrying the evidence, plus
   `unused-ignored-advisory`'s default warning telling us the day either stops matching. A genuinely
   new advisory will still fail the job, and that is the intended behaviour.
3. **`jsdom` on Windows CI** has been reliable locally on Node 24 but is the likeliest place for an
   environment-specific failure. Neither run hit it: both UI jobs were green in Run #1 and again in
   Run #2, with `Tests 19 passed (19)` recorded on the Ubuntu job.
4. **Runtime.** A three-platform matrix building the Tauri dependency tree without caching is minutes
   per job, and the Ubuntu job now spends time in `apt-get` first. Accepted deliberately for P0; the
   macOS job is the cheap one, because it excludes the shell.
5. **The `rust` job now depends on the UI build it used to be independent of.** `clippy --all-features`
   compiles the desktop crate with `custom-protocol`, whose codegen needs `dist/`, so the job runs
   `pnpm install --frozen-lockfile` on a checkout that has no `dist/`. That is one more network
   fetch and roughly a minute of CI time per OS, and it makes a Core-only pull request sensitive to
   a broken frontend lockfile. The alternative - narrowing `--all-features` so the gate stops seeing
   the shipping configuration - was rejected in `P0_IMPLEMENTATION_LOG.md` 18.
6. **The Ubuntu prerequisite list is upstream documentation, not something this machine can test.**
   Every package named in the install step appears in Tauri's own Linux prerequisites, and each one
   was checked against a crate in this graph, but the resolution can only be confirmed by the runner:
   a package rename in the image, or a `.pc` file that arrives through a dependency this repo does not
   control, would bring Failure B back. `libdbus-1-dev` is listed explicitly for exactly that reason.
   Run #2 reduced this risk rather than eliminating it: the same list installed and resolved on
   `ubuntu-latest` in the `rust` job, and both Rust jobs went green. What remains unconfirmed is the
   copy in the `drift` job, which no runner has executed yet.
7. **Provisioning is per job, so it can be half-applied.** This is not a hypothetical risk - it is
   what Run #2 measured. Two jobs compile `firmwaresight-desktop` on Linux (`rust` through
   `clippy --all-features`, `drift` through `cargo test` for ts-rs), and the first round attached the
   prerequisites to the job that had failed rather than to the requirement. The structural fix is to
   read the graph and ask which jobs build the shell, not which jobs reported red; the copy that
   round 2 added is now measured - Run #3's drift job installed the same block and passed all five
   steps. A third job that compiles the desktop crate on Linux would make the
   duplication worth extracting into a script - and that argument should be made by a third instance,
   not in anticipation of one.

## Exit criterion mapping

| Prompt item | Where it runs |
| --- | --- |
| fmt / clippy / cargo test | `rust` job on two OSes, and locally |
| fixture byte identity is not platform-dependent | `rust/test` on Windows and Linux (`committed_fixtures_match_their_recorded_hashes`), plus `.gitattributes` |
| frontend typecheck and build | `frontend` job on two OSes, and locally |
| ts-rs generation works | `drift` job (regenerate + `git diff --exit-code`) |
| generated assets match this build | `drift` job, comparing decoded pixels rather than encoder bytes |
| no forbidden dependency or boundary violation | `deny` job (`[bans] deny` list in `deny.toml`), over the four shipping targets |
| no unresolved advisory policy | `deny` job's advisories check, with two reasoned ignores recorded |
| core works on the third platform | `macos-core` job (`--only core-smoke`) |
| golden tests pass | `rust/test` plus `drift/goldens unchanged` |
| the shipping window really opens | not a CI job: `P0_DESKTOP_SMOKE_REPORT.md`, manual, with the defect it found |

## Final disposition — P0 PASS, v0.6.0 (2026-09-28)

All four runs are in this file, and the two failures are still at the top of it in the order they
happened. The workflow is green, twice, on the two HEADs that mattered: `1cd6309` (engineering) and
`5e58f77` (architect-reviewed). The architect signed the promotion on that basis, which is the act this
report spent two rounds declining to perform for itself. `P0_FINAL_PROMOTION_REPORT.md` holds the
decision; this file holds what the runners measured.
