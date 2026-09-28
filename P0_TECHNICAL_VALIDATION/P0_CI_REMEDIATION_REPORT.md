---
title: "P0 CI Remediation Report"
doc_id: "FS-P0-020"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 CI Remediation Report

Scope: the architect prompt *FirmwareSight P0 — CI Closure / Cross-Platform Reproducibility
Remediation v1.0*. Target state: `LOCAL REMEDIATION COMPLETE / READY FOR REMOTE CI RERUN`. Not P0
`PASS`, not `v0.6.0`, not P1.

Result: **all five failures reproduced with a command before any of them was changed; four fixed at
the cause; one fifth found during the desktop smoke and fixed too; the local gate is 14/14 with zero
`SKIPPED` mandatory steps.** Run `36378384225` on the pushed HEAD then confirmed all five remotely -
six of seven jobs green - and left one failure this report did not anticipate, because it attached the
Linux prerequisites to the job that had failed rather than to the requirement. See the postscript at
the end of this file and `P0_CI_RUN_2_CLOSURE_REPORT.md` for the round that closes it.

## Remote CI Run #1, the thing being remediated

| | |
| --- | --- |
| Repository | `2023violet/FirmwareSight` |
| Run | `36360310447`, event `push`, head `f9b8ccba7d9326ac397675602beb3a7e9c3bcd02` |
| Conclusion | `failure` in 5m 13s |
| Green | `Desktop UI (windows-latest)`, `Desktop UI (ubuntu-latest)` |
| Red | `Rust (windows-latest)`, `Rust (ubuntu-latest)`, `Generated output drift`, `Dependency policy` |
| Absent | a macOS job, which `05_ENGINEERING/06_CI_CD_BASELINE.md` requires on a push to `main` |

Full log extracts are retained in `P0_CI_REPORT.md` as failed historical evidence.

## Failure A — fixture bytes depended on the checkout, not on the fixture

**Reproduced.** A fresh clone of the pre-fix tree with `core.autocrlf=true` (the Windows condition)
produced `p0-dual-region.ld` = `7b872afd…` where the manifest records `f4bf022b…`; the same clone
with `core.autocrlf=false` (the Linux condition) produced `firmware.map` = `c4182c5b…` where the
manifest records `b2be5f19…`. One of ten hashes disagreed in each case, and the disagreeing file was
different on each platform.

**Cause.** `.gitattributes` classified `*.elf` and `*.bin` as binary and listed ten text extensions,
but not `.ld`, `.map`, `.c` or `.h` — while its own header comment claimed the MAP files were
covered. Git only abstains from line-ending conversion for files it has classified, so an
unclassified text file is rewritten by whoever's `core.autocrlf` is set. The manifest then recorded
one platform's working copy rather than the committed bytes for `firmware.map`: for nine of ten
entries `disk == blob == recorded`; for the MAP, `recorded == CRLF working copy ≠ LF blob`.

**Fix.** `*.map -text`, `*.ld text eol=lf`, and `text eol=lf` for every other tracked source and
configuration type (`.c`, `.h`, `.js`, `.html`, `.yaml`, `.yml`, `.txt`, `.csv`, `.npmrc`, `.lock`,
`.gitignore`, `.gitattributes`, `SHA256SUMS`) alongside the extensions already there. One manifest
value corrected to the committed blob's own SHA-256. The assertion did not move: no newline
normalization was added to the parse or test path, no hash expectation was loosened, and the test
still compares raw bytes.

**Verified.** `git check-attr` reports `text: set, eol: lf` for the `.ld` and `text: unset` for the
`.map`; both fresh clones now agree with the manifest 10/10; `cargo test -p firmwaresight-artifact`
passes 19 + 4. The committed `.map` bytes never changed — `git diff` for that path is empty — only
its record, and the local working copy was returned to the blob it came from.

## Failure B — the Ubuntu runner had none of the Tauri Linux prerequisites

**Reproduced from the log.** `glib-sys v0.18.1` build script → `pkg-config exited with status code
1`, `Package glib-2.0 was not found in the pkg-config search path`, `PKG_CONFIG_PATH` containing only
the Python tool cache. Not a source defect: the identical crate graph compiles on Windows and the
Ubuntu UI job was green.

**Fix.** The `rust` job installs Tauri's documented Linux prerequisites before calling the same
`python scripts/check.py --only rust`, for `ubuntu-latest` only. `pkg-config` is deliberately not
installed: the log shows it ran and returned status 1, so it was present — the `.pc` files were what
was missing. `libdbus-1-dev` is named explicitly because `libdbus-sys`, reached through `tao`, asks
pkg-config for `dbus-1`; betting a CI cycle on another package pulling it transitively is not worth
it. No package was added that the documentation does not list.

**Coverage kept.** `clippy --workspace --all-targets --all-features` still compiles
`firmwaresight-desktop`. The alternative — excluding the desktop crate on Linux — would have deleted
the only evidence that the shell's compile boundary holds on a second platform, and was rejected.

**Verified where it can be.** Locally the rust group is 3/3 on Windows. The Ubuntu result can only be
produced by the rerun.

## Failure C — the icon drift check asserted encoder bytes

**Reproduced.** Not a Pillow version difference: the drift job installed the pinned `Pillow==12.3.0`
from `scripts/requirements.txt`. Re-encoding this build's own mark with `optimize=True` (equivalent to
`compress_level=9`), `optimize=False` and `compress_level=1` changes the PNG SHA-256 at every size
while the decoded RGBA content yields exactly one value per size. `--check` compared
`committed_bytes == generated_bytes`, so it asked each platform to reproduce one encoder's zlib and
filter decisions. The two files CI did not flag (`32x32.png`, `icon.png`) were not different in kind —
their encoder output merely happened to agree.

**Fix, route 1 (the prompt's preferred route).** `--check` now decodes both sides and compares
semantics: PNG dimensions and RGBA pixel bytes; ICO the required frame set plus per-frame RGBA pixels,
with a missing frame failing and an unexpected frame failing. The icon design, the branding and
`assets/design-tokens.json` were not touched, and no Linux-regenerated set was committed in place of
the existing one. Route 2 (a project-owned byte-deterministic encoder) was not taken: it needs a
heavy dependency or a hand-written container writer to be worth anything, and CI evidence of
cross-platform SHA equality cannot be produced from this machine.

**Verified by three conditions, run against the real committed set:**

```text
1. committed icons, untouched                        -> exit 0
2. identical pixels, 5/5 files hold different bytes  -> exit 0   (what Run #1 could not tolerate)
3. one repainted pixel and one dropped ICO frame     -> exit 1
   128x128.png: pixel content differs at 128x128
   icon.ico: missing frame 24x24; …
```

The check therefore still has teeth: it fails on a changed picture and on a missing frame, and it no
longer fails on a compressor's choice.

## Failure D — `deny.toml` was written for a different cargo-deny schema

**Reproduced.** `cargo deny 0.20.2` rejects `deny.toml:24` `[bans] highlight-warnings` with
`error[unexpected-keys]` naming `highlight` as the supported key, and rejects the five slash strings
in `licenses.allow` with `error[custom]: invalid character(s)` because that list is parsed as SPDX.
A third key the CI log never reached was also fatal: `[advisories] severity-threshold`, removed by
PR#611 and made an error by PR#681. The schema was confirmed against the installed tool, not from
memory: `cargo deny init` in a scratch directory, and the `Config` deserializers in the vendored
`cargo-deny-0.20.2` source.

**Fix and what the real run then said.**

| Finding | Action |
| --- | --- |
| `severity-threshold = "medium"` | removed; 0.20.2 no longer gates by CVSS, so `unmaintained`/`unsound` are declared as scopes and `yanked = "deny"` is written out to keep the old fatal-on-yanked intent |
| `highlight-warnings = true` | `highlight = "all"` |
| five slash entries in `licenses.allow` | deleted. `cargo deny check licenses` still reports ok afterwards, so those lines asserted nothing: the tool already resolved those crates from their license files |
| `ISC` allowed but encountered by no crate | removed, so the list stays a census rather than a wish |
| `reqwest` / `hyper` banned and present | they are a **non-optional Android/iOS** dependency of `tauri 2.12.0` (`[target.'cfg(any(target_os = "android", all(target_vendor = "apple", not(target_os = "macos"))))'.dependencies.reqwest]`), invisible on desktop builds. `[graph] targets` now names the four shipping targets from `04_TECH/20`, so the boundary is checked where it can be violated and the ban stays absolute rather than being exempted by name |
| `anyhow` banned and present | genuinely unavoidable on desktop: `tauri`, `tauri-build` and `tauri-utils` depend on it. It stays in `deny` with `wrappers = ["tauri", "tauri-build", "tauri-utils"]`, so a first-party dependency on it still fails the gate |
| five `error[wildcard]` on our own crates | `allow-wildcard-paths = true`: `{ workspace = true }` inheritance writes no version requirement in the child manifest, and path dependencies cannot be published as `*`. `wildcards = "deny"` remains for registry crates |
| 23 `warning[duplicate]` | left as warnings, as the policy says; nothing was added to `skip` or `skip-tree` to quiet them |

**Result.** `cargo deny check licenses bans sources advisories` → exit 0,
`advisories ok, bans ok, licenses ok, sources ok`, run locally with the tool installed. The license
census behind the allow list is reproducible with `cargo deny list -t 1.0 --layout license`.

### Architecture conflict to report, not to fix here

Two advisories remain, both in the informational class, and neither can be resolved without changing
a frozen dependency:

| Advisory | Crate | Path | The advisory's own solution | Fixable inside the frozen family? |
| --- | --- | --- | --- | --- |
| `RUSTSEC-2024-0429` (unsound) | `glib 0.18.5` | `firmwaresight-desktop → tauri 2.12.0 → tao 0.37.1 / muda 0.20.0 / webkit2gtk 2.0.2 → gtk 0.18.2 → glib 0.18.5` | upgrade to `>=0.20.0` | **No.** `cargo update -p glib --precise 0.20.0` fails: `gtk v0.18.2` requires `glib = "^0.18"`, and `tauri v2.12.0` requires `gtk = "^0.18"` |
| `RUSTSEC-2024-0370` (unmaintained) | `proc-macro-error 1.0.4` | `… → glib 0.18.5 → glib-macros 0.18.5 → proc-macro-error 1.0.4` | "No safe upgrade is available!" | **No.** The crate is unmaintained; the alternatives listed are rewrites of the consumer |

Both are Linux-only in this graph, neither is a vulnerability, and `glib::VariantStrIter` is not
reachable from P0's code. They are recorded in `deny.toml`'s `ignore` with that evidence attached, so
`unused-ignored-advisory` will report the day either stops matching. **No frozen dependency was
moved.** If the architect judges that an unsound advisory in the WebView stack must be fatal rather
than recorded, that is an ADR about the Tauri version baseline, which is outside this round's scope.

## The fifth problem: the required macOS core smoke did not exist

`05_ENGINEERING/06_CI_CD_BASELINE.md` states, for a push to `main`: Windows required, Linux required,
**macOS required core smoke**; for a pull request, macOS optional/cost-controlled. The workflow had no
macOS job, so even six green jobs would not have satisfied the frozen baseline.

**Fix.** A `core-smoke` group in `scripts/check.py` selecting the four library crates and the CLI by
package name — `firmwaresight-core`, `firmwaresight-artifact`, `firmwaresight-storage`,
`firmwaresight-report`, `fwsight` — running fmt, `clippy --all-targets -D warnings` and `test`. No
fifth crate, no reimplemented test, and no CI-only command: the workflow step is the same
`python scripts/check.py --only core-smoke` a developer runs. It is deliberately not part of the
default gate, because `cargo test --workspace` already covers those packages; `--list` shows it.

**Measured locally on Windows:** 3/3 steps, **87 of the 104 Rust tests**, leaving 17 in the desktop
crate's own coverage on the two platforms that build the shell. Coverage is core's 39 domain tests,
artifact's 4 unit + 19 `p0_acceptance.rs` (fixture identity, parser, memory accounting, evidence
ladder), storage's 11, report's 4, and the CLI's 5 unit + 5 golden tests.

## Sixth problem, found by the desktop smoke and fixed

`evidence.id` was a whole-table primary key while the analyzer reuses identifiers across builds, so a
second artifact could never be stored. The window showed it as `ERR-STORAGE-4006`; migration `0002`
keys evidence by `(build_id, id)`. Full evidence, the two tests written before the fix, and the
in-place upgrade of the real user-profile database: `P0_DESKTOP_SMOKE_REPORT.md`.

## Local verification matrix

| Item | Command | Result |
| --- | --- | --- |
| A | `git check-attr -a -- …/p0-dual-region.ld`, `…/firmware.map`; raw SHA of every manifest entry in two fresh clones | `text set / eol lf`, `text unset`; 10/10 match on both |
| B | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace` | clean; clean; **104 passed / 0 failed** |
| C | the five `corepack pnpm -C apps/desktop/ui` steps | install, typecheck, lint, test (**19 passed**), build — all pass |
| D | `python scripts/check.py --only drift` | 5/5: design tokens, desktop icons, ipc bindings, bindings unchanged, goldens unchanged |
| E | `cargo deny check licenses bans sources advisories`, then `python scripts/check.py --only deny` | exit 0, four `ok`; gate reports `PASS deny/cargo-deny`, not SKIPPED |
| F | `python scripts/check.py` | **14/14 steps, exit 0, zero `SKIPPED`** |
| G | desktop window smoke | PASS after the `0002` fix, with one item recorded as not observed |

`scripts/check.py` was not edited to turn any failure into a skip; the only script change in this
area is the new `core-smoke` group.

## What is still unproven

| Open | Closes when |
| --- | --- |
| Ubuntu clippy with the prerequisites installed | the owner pushes and the new run executes the `rust (ubuntu-latest)` job |
| macOS core smoke on `macos-latest` | the same run's `macOS Core Smoke` job |
| Icon drift across real encoders on Linux | the same run's `Generated output drift` job — the semantic check is expected to make it pass, and the three-condition experiment above is the local evidence for that expectation |
| cargo-deny on a clean runner with a freshly cloned advisory DB | the same run's `Dependency policy` job |
| Peak RSS | a measurement tool is authorized; `NOT MEASURED`, and not a promotion blocker |

Nothing here may be read as `REMOTE CI PASS`. The next run must be green on a head equal to the pushed
remediation tip; a green rerun of `f9b8ccb` would prove nothing about this tree.

## Postscript: Run #2 measured this round, and one line of it was wrong

Run `36378384225` executed on the pushed HEAD `ebda52d` and concluded `failure` with **six of seven
jobs green**. Read against the table above:

| Claim in this report | Run #2 |
| --- | --- |
| Ubuntu clippy needs the apt block | **CLOSED** - `Rust (ubuntu-latest)` passed, 5/5 steps, 104 tests, desktop crate compiling |
| macOS core smoke is unproven | **CLOSED** - `macOS Core Smoke` passed, 3/3 steps, 87 tests |
| Icon drift on Linux may fail | **CLOSED** - `drift/desktop icons` passed on the Linux runner printing `desktop icons are current (pixel-identical to this build).` |
| cargo-deny on a clean runner | **CLOSED** - `Dependency policy` passed, `advisories ok, bans ok, licenses ok, sources ok` |
| Fixture byte identity off Windows is unproven | **CLOSED** - `Rust (windows-latest)` and `(ubuntu-latest)` both ran the hash assertion green at 104 tests |
| Peak RSS | still `NOT MEASURED`, and still not a promotion blocker |

The one thing this round got wrong is stated in the report's own terms rather than smoothed over: it
attached the Linux prerequisites to **the job that had failed**, not to **the requirement**. Two jobs
compile `firmwaresight-desktop` on Linux - `rust` via `clippy --all-features`, `drift` via
`cargo test -p firmwaresight-desktop` for ts-rs binding regeneration - and the second one had no apt
step. Run #2's `Generated output drift` job passed its first two steps and then died in `gobject-sys`'
build script with `Package gobject-2.0 was not found in the pkg-config search path`.

That is the same missing system-library class as Run #1's `Rust (ubuntu-latest)`, reached from a
different job - and the reproduction of it is a CI provisioning omission, not a product defect, not a
ts-rs contract defect and not generated drift. The fix, the classification and the copy-versus-script
decision belong to `P0_CI_RUN_2_CLOSURE_REPORT.md`.
