---
title: "P0 CI Run #2 Closure Report"
doc_id: "FS-P0-021"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 CI Run #2 Closure Report

Scope: the architect prompt *FirmwareSight P0 — Remote CI Run #2 Final Drift Closure v1.0*.
Target: `LOCAL FIX COMPLETE / READY FOR REMOTE CI RUN #3`. Not P0 promotion, not `v0.6.0`, not P1.

Run #2 closed four of the four failures from Run #1 and added the macOS job the baseline required. It
left one failure, and that failure is not in the product: it is a second CI job that needs the same
Linux build environment the first one already had.

## Run #2, as read from GitHub rather than from a report

| | |
| --- | --- |
| Repository | `2023violet/FirmwareSight` |
| Run | `36378384225`, event `push`, head `ebda52d63f22ff9f25786c1d80e53f5193cff183` |
| Conclusion | `failure` |
| Verification command | `gh run view 36378384225 --repo 2023violet/FirmwareSight --json databaseId,headSha,conclusion,event,jobs` |

| Job | Conclusion | What its own log shows |
| --- | --- | --- |
| `Rust (windows-latest)` | PASS | `5/5 steps passed`, `test result: ok.` summing to **104** |
| `Rust (ubuntu-latest)` | PASS | `5/5 steps passed`, **104** — with `firmwaresight-desktop` compiling, which Run #1 could not do |
| `Desktop UI (windows-latest)` | PASS | `5/5 steps passed` |
| `Desktop UI (ubuntu-latest)` | PASS | `5/5 steps passed`, `Tests 19 passed (19)` |
| `macOS Core Smoke` | PASS | `3/3 steps passed`, **87** tests — first execution of this job ever |
| `Dependency policy` | PASS | `1/1 steps passed`, `advisories ok, bans ok, licenses ok, sources ok` |
| `Generated output drift` | **FAIL** | `drift/design tokens` PASS, `drift/desktop icons` PASS, then `drift/ipc bindings` FAIL |

The `5/5` on the two `Rust` jobs against the local `3/3` is the `frontend assets` pair: a clean
runner has no `apps/desktop/ui/dist/`, so `check.py` builds the UI before `clippy --all-features`
compiles the desktop crate. The gate adapts to the tree it is given, and CI exercised the branch the
local warm tree cannot.

## What Run #2 confirmed remotely from the first remediation round

- **Fixture byte identity across the checkout.** `Rust (windows-latest)` passed `drift`-free: the
  `committed_fixtures_match_their_recorded_hashes` assertion that failed on a CRLF smudge in Run #1
  now passes with `.gitattributes` in the tree. The Linux side passes on the same manifest.
- **Ubuntu Tauri prerequisites.** The `rust` job's apt block let `clippy --workspace --all-targets
  --all-features` compile the desktop crate on `ubuntu-latest` - the coverage the round refused to
  obtain by excluding the crate.
- **Semantic icon check.** `drift/desktop icons` printed
  `desktop icons are current (pixel-identical to this build).` **on Linux**. The encoder-byte
  assertion that split the two platforms in Run #1 is replaced by a decoded-pixel comparison that both
  platforms pass without committing a Linux-regenerated asset set.
- **Dependency policy executes.** All four categories pass remotely with `cargo-deny 0.20.2` pinned by
  the workflow, including the ban list evaluated over the four shipping targets.
- **The macOS core smoke exists and is green** on its first run, satisfying the baseline requirement
  Run #1 could not meet even in principle.

## The one failure, at the step level

`scripts/check.py --only drift` regenerates the ts-rs bindings by running `cargo test -p
firmwaresight-desktop`, because that is the only way the committed `.ts` files can be proven to still
match the Rust DTOs. On `ubuntu-latest` that command links the GTK stack, and the drift job had no
provisioning step:

```text
error: failed to run custom build command for `gobject-sys v0.18.0`
pkg-config exited with status code 1
> PKG_CONFIG_PATH=/opt/hostedtoolcache/Python/3.13.15/x64/lib/pkgconfig \
  PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1 pkg-config --libs --cflags gobject-2.0 'gobject-2.0 >= 2.70'
  Package gobject-2.0 was not found in the pkg-config search path.
The system library `gobject-2.0` required by crate `gobject-sys` was not found.
HINT: you may need to install a package such as gobject-2.0, gobject-2.0-dev or gobject-2.0-devel.
```

Job `108788787601`, step `Verification gate (drift)`. The same block also fails `glib-sys v0.18.1`,
which is where Run #1's `Rust (ubuntu-latest)` died.

**Classification.** CI provisioning coverage, duplicated per job — not a Core defect, not a ts-rs
contract defect, not generated drift, not a Tauri source defect. Two jobs compile the desktop crate on
Linux; the first remediation round gave one of them the prerequisites and the omission was invisible
until the other one reached the same code path.

**Nothing about the drift check itself is wrong.** Its first two steps passed, and the third is the
one that requires a system library.

## Fix

`.github/workflows/p0-check.yml`: in the `drift` job, after `Install script dependencies` and before
`Verification gate (drift)`, the same apt block the `rust` job installed — parsed and compared after
the edit, the two `run:` blocks are byte-identical package lists:

```yaml
- name: Install Linux prerequisites for the Tauri shell
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      libwebkit2gtk-4.1-dev build-essential curl wget file \
      libxdo-dev libssl-dev libayatana-appindicator3-dev \
      librsvg2-dev libdbus-1-dev
```

`if: matrix.os == 'ubuntu-latest'` is deliberately absent from this copy: `drift` pins
`runs-on: ubuntu-latest`, so the guard would be dead configuration.

**Why a copy and not a shared script.** Only two jobs need this list. A `scripts/*.sh` would add a
shell portability surface and a testing obligation to save ten YAML lines, and the round's target is
the smallest reversible change that closes a measured remote failure. If a third job ever needs it,
the duplication becomes a real argument.

**What was not touched.** No trigger, no `permissions` (still `contents: read`), no new third-party
action, no secret, no `continue-on-error`, no `if: false`. `drift` still runs all five of its steps,
including `cargo test -p firmwaresight-desktop` and both `git diff --exit-code` assertions - the check
was never weakened to route around the environment. No Rust, TypeScript, fixture, golden, schema,
`deny.toml`, `Cargo.lock`, `pnpm-lock.yaml` or `scripts/check.py` change: the diff is 21 added lines in
one workflow file.

## Local verification on this tree

Windows host; the apt block itself is **NOT LOCALLY EXECUTED** — this machine has no Ubuntu, and
pretending otherwise would be the exact substitution this pack keeps refusing to make. What was run:

```text
cargo fmt --all -- --check                                   exit 0
cargo clippy --workspace --all-targets --all-features -D warnings   exit 0, silent
cargo test --workspace                                       104 passed / 0 failed
corepack pnpm (install|typecheck|lint|test|build) in the UI package
                                                             exit 0; Tests 19 passed (19)
python scripts/check.py --only drift                          5/5, exit 0
python scripts/check.py --only deny                           1/1, exit 0 (cargo-deny 0.20.2)
python scripts/check.py --only core-smoke                     3/3, exit 0 (87 tests)
python scripts/check.py                                       14/14, exit 0, 0 SKIPPED
git diff --check                                              clean
```

The test count is unchanged at 104 Rust / 19 UI, which is what a workflow-only fix should produce.

One local execution detail worth recording: `corepack pnpm -C apps/desktop/ui test` fails on this host
with `ERR_PNPM_BAD_PM_VERSION` (it resolves 12.6.0 because corepack reads `packageManager` from the
working directory, and the repository has no root `package.json`). Invoked from inside the package -
which is what `check.py` and both CI frontend jobs do - `corepack pnpm --version` reports the pinned
**12.7.0** and the run is green. Nothing in the tree is wrong; the `-C` form is simply not the way this
project's pin is meant to be resolved, and `05_ENGINEERING/00_REPO_STRUCTURE.md`'s root `package.json`
question is already recorded in `P0_KNOWN_LIMITATIONS.md`.

## Desktop smoke: carried forward, not re-run

No file under `apps/desktop/**`, `crates/**` or the UI source changes in this round - the diff is one
workflow file - so `P0_DESKTOP_SMOKE_REPORT.md` stands as the evidence, including the two launches on
the shipping configuration and the defect those launches found. Re-opening the window to add a third
identical observation would be noise, and the prompt says so.

## Advisory disposition from the architect

`RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`,
unmaintained) are **accepted as explicit P0 transitive risk, not silenced**, and **do not block P0
promotion** on the current evidence: both are recorded with reasons in `deny.toml`, Run #2 shows all
four cargo-deny categories passing, neither has a compatible upgrade inside the `gtk-rs 0.18` / Tauri
2.12 line, and no first-party code calls the affected API. Revisit triggers, the five the architect
named, are recorded in `P0_DEPENDENCY_REPORT.md` and `P0_KNOWN_LIMITATIONS.md`. No architecture ADR is
required, because this round changes no architecture choice; an ADR becomes necessary if Tauri is
replaced, dependencies are forked, or the frozen desktop dependency family changes.

## State this round produced, and what Run #3 then measured

```text
At delivery:   P0 FAIL — REMOTE CI RUN #2 (6 of 7); remediation round 2 LOCAL FIX COMPLETE;
               Run #3 required after a push.
After the push 1cd6309 went to origin/main and Run 36399805005 concluded success, 7 of 7 jobs green,
               including Generated output drift: the job installed the same apt block
               (`Setting up libwebkit2gtk-4.1-dev`) and passed all five gate steps.
```

Run #3's log lines are quoted in `P0_CI_REPORT.md`, the file that keeps per-run history. The remote CI
result is now a measured fact with a command behind it. This round's author still writes no
`P0 = PASS`: that promotion act, `baseline_version: 0.6.0` and a regenerated `SHA256SUMS` belong to
the architect's next signed prompt. The distinction cuts both ways - a green run does not sign
itself, and a promotion claimed without one would be a fabrication.

## Historical record

Run `36360310447` (Run #1, head `f9b8ccb`, 2 of 6 green) stays in `P0_CI_REPORT.md` as executed
history. Run #2 is now `latest_remote_ci`; the two are not merged, relabeled or cleaned up, because
the pair - one run that failed four ways and one that failed one way - is the actual evidence that the
first remediation worked.
