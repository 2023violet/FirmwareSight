---
title: "Post-G2 E2E Findings Remediation Report"
doc_id: "FS-POSTG2-REMEDIATION"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "POST_G2_E2E_FINDINGS_REMEDIATION"
owner: "Engineering"
last_updated: "2026-10-02"
---

# Post-G2 E2E findings remediation

Round `POST_G2_E2E_FINDINGS_REMEDIATION`, execution prompt v1.0 (Architect reviewed). Not a re-run of
G2, not the 283-case E2E, and not P5: the only goal was to fix the narrow, evidence-backed product
defects the post-G2 real-desktop round observed, then prove each fix with a targeted regression and a
focused real-desktop check.

Starting state, verified before any write: `HEAD = origin/main = 5c52f3d135e16c263e5bf4d928582aeca87e8c26`,
tree clean, one worktree, that head green on Run `36949964372` attempt 1 at 7 of 7.

## Verdict

```
POST_G2_E2E_REMEDIATION = PASS

E2E-F001 = FIXED      E2E-S3
E2E-F002 = FIXED      E2E-S2
E2E-F003 = FIXED      E2E-S3   (included after the eight-condition gate, not by name)

S0 = 0    S1 = 0    S2 open = 0    S3 open = 0

G2 = PASS (unchanged)
Product MVP = ENGINEERING COMPLETE (unchanged)
Product state = MVP CANDIDATE
baseline_version = 0.6.0
active_task = NONE
```

## What each finding was, and what was done

| Finding | Cause, read from source | Fix | Commit |
| --- | --- | --- | --- |
| E2E-F001 | `Analyze.tsx` computed `stale` as `error !== null`, so the honesty note that already existed appeared only after a *failed* analysis. Choosing a second artifact left the first one's facts unlabelled — with two files named `firmware.elf` the only visible difference was the hash, which is the one thing that had not changed. | `App` owns `analyzedSelectionId` beside `lastGood` and the two move together (`Analyze` unmounts on a page switch, so it cannot hold session identity). `stale` becomes `error !== null \|\| pendingSelection`, where identity is the shell's selection handle and never the file name. | `e816dcb` |
| E2E-F002 | A destination that exists and is not a bundle this engine wrote was answered with the same "Replace the existing bundle?" card as one that is, above the engine's own 6106 sentence that the folder "already holds a FirmwareSight release bundle" — false here, and the engine refuses that folder outright with `ERR-BUNDLE-6107`. The card asked the release owner to authorize a decision that cannot be made. | `foreignOccupied = destination.exists && !destination.recognizableBundle` disables Export, offers no Replace path, and states the four true things. A refusal also clears the confirmation card. | `e816dcb` |
| E2E-F003 | `is_gnu_ld` searched the first 4096 characters while the foreign banners beside it searched the whole text, so a GNU ld MAP whose banner sat past the window was refused as "not from a supported linker" — a statement false about the file being refused. | The GNU test searches the whole already-buffered text. | `971015f` |

### E2E-F001 in its own terms

Two constraints the brief named are both kept. Identity is never the file name, because the case that
has to be caught is two artifacts sharing one leaf name. And the identity is not written to the
portable schema or to SQLite: it is one `useState` in the React tree, and `drift/ipc bindings
unchanged` plus `drift/goldens unchanged` pass.

The copy was corrected before the shipping build. The first draft named the candidate twice, which in
the same-name case read as a contradiction — "Previous analysis of firmware.elf. It is not an analysis
of firmware.elf." So when the names match it now says `The selection now in this row is a different
file with the same name, and it has not been analyzed yet.`, and when they differ it names the
candidate. Failed-analysis semantics are unchanged, and a successful analysis retires the marker.

### E2E-F002 without touching the engine

The first false sentence is authored in `crates/firmwaresight-project/src/bundle.rs`, in the 6106
message. Two alternatives were rejected: rewriting it changes bundle engine semantics, which the round's
brief reserves for Architect review; suppressing it in the UI would withhold an engine-authored fact the
user is entitled to read. Neither was needed, because the desktop command already derives
`recognizableBundle` from the same `is_recognizable_bundle` the write path consults before it replaces
anything — so the page knows the truth before it could ask.

The guard is not weakened, not bypassed and not re-decided: the engine still refuses, and the race where
a folder stops being a bundle between the choice and the replace still ends in the truthful 6107 state
with the card cleared. `ERR-BUNDLE-6107`'s engine tests are untouched.

### E2E-F003's accepted trade-off, stated

A foreign MAP carrying the literal `Memory Configuration` beyond 4 KB would now be handed to the GNU
parser. It still fails closed, because `parse` refuses a MAP that declares no memory regions, so the
worst case is a different error message and never a fabricated layout. Detection order is unchanged and
GNU still wins over a foreign banner, as it did before this commit.

Measured offsets that made the window indefensible: the owner's AT32 project at character 1,162,057 and
a 51 MB synthetic at 491,743, against the same linker accepted at 609. GNU prints one
`Archive member included to satisfy reference by file (symbol)` line per symbol pulled from a static
library, and a `Discarded input sections` block under `--gc-sections`, both before its banner. Neither
length is under the project's control.

## Validation

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean (one over-long line in the new test was reflowed by `cargo fmt --all` and nothing else) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0, no warning |
| `cargo test --workspace` | **770 passed, 0 failed** — baseline 769 plus the one new MAP test |
| `corepack pnpm install --frozen-lockfile` | lockfile up to date, no dependency change |
| `corepack pnpm typecheck` / `lint` | exit 0 |
| `corepack pnpm test` | **159 passed** in 6 files — baseline 155 plus four new regressions |
| `corepack pnpm build` | `dist/assets/index-ChIB_w0m.js` 315,294 B |
| `python scripts/check.py` | **15/15**, re-run on the committed state after the last copy change |
| `git diff --check` | clean |

Repetition, because this project has already produced async UI assertion races and a new regression
that passes once is not a regression: `intake.test.tsx` 30 fresh processes, twice (before and after the
copy change); `release.test.tsx` 30 fresh processes, twice concurrently; the F003 target test 30 times,
twice; `p0_acceptance.rs` whole file 10 times. Zero flakes, so the instruction not to call a flake
noise was never exercised.

Mutation proofs, on the final code. F001: reducing `stale` to `error !== null` makes both new tests
fail with `Unable to find role="region" and name "Last good analysis"` — the marker simply absent.
F002: the brief's prescribed mutation, restoring the unconditional `setConfirm(destination.bundleFolderName)`,
does **not** by itself turn the regression red, because Export is already disabled before that line can
run; re-enabling Export as well makes the Replace group appear and the test fails. The load-bearing
guard is therefore the disabled control plus the `recognizableBundle` condition, and the evidence says
so rather than crediting the line the mutation named. Both files were restored and verified byte-exact
with `cmp`.

## Remote CI

| Head | Run | Attempt | Result |
| --- | --- | --- | --- |
| `e816dcb` + `971015f` (pushed together, fast-forward from `5c52f3d`) | #43 `37100371601` | 1 | completed / success / **7 of 7** |

Jobs: Rust (windows-latest), Rust (ubuntu-latest), Desktop UI (windows-latest), Desktop UI
(ubuntu-latest), Generated output drift, Dependency policy, macOS Core Smoke. No rerun, no
provisioning fault, no red job.

## Scope discipline

Changed: three `.tsx` sources, two UI test files, one Rust function, one Rust test file. Not changed:
any CSS rule, design token, schema, migration, `Cargo.toml`, golden, fixture, IPC binding, generated
icon, README or governance status. `git show --stat` on both commits lists exactly those seven files.

Out of scope and untouched, per the brief: P5, History, installer, signing, notarization, updater,
SBOM, CVE, cloud, auth, telemetry, AI, parser performance, memory optimization, new formats, a Keil/IAR
adapter, new migrations, public schema v2, a sixth crate, new dependencies, design-system redesign,
125/150 % DPI, mouse-wheel infrastructure, the E2E harness refactor, the `update_goldens` issue, and
license selection, which remains **PENDING OWNER CONFIRMATION**.

Performance observations are carried forward with the wording the brief requires and were not
optimized:

* `First-use <60 s at that workload = PARTIAL / environment-sensitive` — the 519,179,252-byte ELF
  analyzed in ~4.73–4.89 s warm and ~68.7 s on a cold first read on this machine.
* `Peak RSS = MEASURED FOR TESTED WORKLOAD` — ~1,428 MB working set at that workload.

No claim of a leak, no claim that 500 MB passes universally, no claim of performance work done.

## Evidence location

The full external evidence root stays outside the repository, as the brief requires:
`%TEMP%\FirmwareSight-PostG2-E2E-Remediation-20261002` — `TAKEOVER_REPORT.md`, `LOCAL_GATES.md`,
`REPEATS.log`, `FOCUSED_REVALIDATION.md`, `mutation-backups/`, `13_findings/HARNESS-INCIDENT-16.md`,
`00_environment/store_state.json`, and every screenshot the focused re-validation measured. This
directory holds the summary; `FOCUSED_REVALIDATION_REPORT.md` holds the desktop results and
`EXIT_CHECKLIST.md` settles the round's exit conditions.
