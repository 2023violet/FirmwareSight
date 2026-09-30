---
title: "P3 Release Gate Execution Report"
doc_id: "FS-P3-EXEC"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P3_RELEASE_GATE"
owner: "Engineering"
last_updated: "2026-09-30"
---

# FirmwareSight P3 Release Gate — execution report (prompt §72 sections 1–13)

## 1. What P3 was

`firmwaresight.toml` → project and Git evidence → `GatePolicy` → Core Gate evaluation → five factual
states → immutable `GateRun` → immutable review acceptance → `fwsight gate` → the desktop Release
Gate page. Nothing else: no bundle, no pricing, no cloud, no accounts, no telemetry, no AI (§72).

## 2. Stage and provenance

| | |
| --- | --- |
| Stage | `P3_RELEASE_GATE` |
| Started from | `32b23aa` (`origin/main` at takeover, P2 closed) |
| Commits on top | `6e997f1` authorize the adapter · `f1516b8` project policy, Git provenance, Core Gate · `272a8ce` persist GateRun and acceptance · `4e23212` portable contracts and the `gate` command · `633737b` desktop Release Gate · `00e7a9c` what the smoke and the gate found · `c4bb91a` record the validation and close the stage · `219178a` integrity artifacts |
| Pushed | Closed unpushed, then pushed: `origin/main` is `219178af195569ec6b13728d84d992ef78df8c04`, so §4.1 carries what CI measured on this stage's tree rather than the closure text predicting it |
| Governance | ADR-0027 `09_ADR/ADR-0027-project-policy-and-provenance-adapter.md` |

## 3. What was built

**`crates/firmwaresight-project`** (new crate, ADR-0027): `config` (TOML load/validate/save,
`ERR-CONFIG-7001..7008`), `git` (read-only provenance through the system `git` binary with a timeout,
`ERR-GIT-8001`, no shell strings, no network, no mutation), `version` (tag → version through the
policy pattern), `fingerprint` (canonical policy hash and run id), `evidence` (the fact carriers the
Gate reads, and the one place the `evidence:` and `file:` locator schemes are written).

**Core** (`domain/gate.rs`): `GatePolicy`, the ten MVP rules in one canonical order, `FindingState`
and `EffectiveSeverity` kept separate (ADR-0023), `aggregate()` precedence, `GateContext` and its
canonical input. Core gained no dependency on Tauri, SQLite, Tokio or filesystem access; it still
carries `#![forbid(unsafe_code)]`.

**Storage**: migration `0003_gate_history.sql` (additive: `gate_runs`, `gate_findings`,
`gate_finding_evidence`, `accepted_reviews`, all with CHECK constraints and no absolute paths),
`persist_gate_run` with §33's dedupe-or-invariant rule, `accept_review` (insert-only), and the read
half — `load_gate_facts`, `build_id_for_snapshot`, `evidence_id_for_field` — that lets a run be
judged from stored rows after its files are gone.

**Portable contracts**: `schemas/gate-results.schema.json`, `schemas/accepted-reviews.schema.json`,
`schemas/project-config.schema.json` (v1), with 13 contract tests in
`crates/firmwaresight-report/tests/gate_schema_contract.rs`.

**CLI**: `fwsight gate` with human and `--json` output and exit codes 0 / 4 / 5 / 6.

**Desktop**: five commands (`open_project_config`, `save_project_policy`, `run_release_gate`,
`accept_review`, `get_gate_run`), `release.rs`, and the `Release` page. Capability stays
`core:default`: the dialogs open in Rust, so the WebView can name a budget and cannot name a file.

## 4. Verification (§66, each command run for itself)

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | `Finished` — no warnings |
| `cargo test --workspace` | **556 passed / 0 failed / 0 ignored across 28 executable suites** (+6 doc-test suites carrying no tests) |
| `corepack pnpm install --frozen-lockfile` (in `apps/desktop/ui`) | `Lockfile is up to date` |
| `corepack pnpm typecheck` | `tsc --noEmit`, clean |
| `corepack pnpm lint` | `eslint .`, clean |
| `corepack pnpm test` | **135 passed (6 files)** — after the race in defect I below was fixed |
| `corepack pnpm build` | `✓ built`, 33.10 kB css / 306.08 kB js |
| `python scripts/check.py --only drift` | 6/6 |
| `python scripts/check.py --only deny` | 1/1 — `cargo-deny` executed for real over licenses, bans, sources, advisories |
| `python scripts/check.py --only core-smoke` | 3/3 |
| `python scripts/check.py` | **15/15** |
| `git diff --check` | exit 0 |

The first full gate of this closure round came back **13/14**, and the failing step was
`frontend/test`. That failure is what turned an earlier unreproduced flake into a named, diagnosed
defect (I, below); the `15/15` above is the gate re-run after the fix, not the same run restated.

Test counts by area, each re-read from a single `cargo test --workspace` invocation: Core
`domain::gate` 32 · project crate lib 68 + its config-schema contract 8 · portable Gate schema
contract 13 · storage `gate_history.rs` 22 · CLI `gate_cli.rs` 18 · desktop `release_gate.rs` 29 ·
frontend `release.test.tsx` 36.

**A count in this section was wrong and is corrected above rather than quietly replaced.** The first
version recorded the workspace total as **715 in 42 suites**. That figure was read off the *gate log*,
not off one command: `check.py` runs `cargo test --workspace` in the `rust` group and then runs
`cargo test -p firmwaresight-desktop` again as the `drift` group's regeneration check, so the desktop
crate's 159 tests (lib 69, `analyze_details` 9, `compare_ipc` 24, `desktop_parity` 7,
`real_artifact_intake` 21, `release_gate` 29, and two empty targets) appear twice in that log — 42
result rows, 28 of them unique suites. One `cargo test --workspace` is 556. The "Core gate 55" line
was the same class of error and is now the counted `gate::` list, 32. Nothing was added, removed or
weakened to reach the lower number: this is a recount of the same tests. `corepack pnpm test` was
never double-counted — 135 passed in 6 files. P3 opened at 345 Rust tests, so the stage added **211**,
not 370.

## 4.1 CI after the push (successor fact, measured 2026-09-30)

The closure commits were pushed after §4 was written, so `origin/main` moved to
`219178af195569ec6b13728d84d992ef78df8c04` and GitHub Actions covered a P3 tree for the first time.

| Fact | Measurement |
| --- | --- |
| Run | [`36774472141`](https://github.com/2023violet/FirmwareSight/actions/runs/36774472141), event `push` to `main`, started 2026-09-30T20:42:14Z |
| Result | `completed` / `failure` |
| Jobs | 6 success, 1 failure — `Dependency policy` |
| Green remotely | Rust (windows-latest and ubuntu-latest), Desktop UI (windows-latest and ubuntu-latest), macOS Core Smoke, Generated output drift |
| Red | `python scripts/check.py --only deny` → `advisories FAILED, bans ok, licenses ok, sources ok`, exit 1 |
| Named cause | `error[yanked]: detected yanked crate (try \`cargo update -p yoke-derive\`)` against `yoke-derive 0.8.3` in `Cargo.lock` |

The yank is an external event on the day of closure: crates.io reports `yoke-derive 0.8.3` as `yanked`
and `0.8.4` as created `2026-09-30T13:19:39Z`. FirmwareSight reaches that crate only transitively and
outside the trusted core — `yoke-derive` ← `yoke` ← `icu_*` ← `idna` ← `url` — so every platform was
already building the same version. §4's `deny` row had passed **locally** because this machine's
crates.io index snapshot predated the yank; the CI job installs `cargo-deny 0.20.2` and syncs a fresh
index. The failure is therefore a real fact about the tree and a stale fact about the local green, and
it is recorded as a limit of the local gate in §8.

Fix: `cargo update -p yoke-derive --precise 0.8.4`. The lockfile delta is that version and its
checksum, plus `cssparser-macros`'s already-satisfied `syn >=2, <4` edge re-resolving from `3.0.6` to
`2.0.119`. No `Cargo.toml` changed, no crate entered or left the graph, no license changed, and
nothing in Core or in the project crate's declared dependencies moved. Re-measured on the fixed tree:
`--only deny` 1/1, `cargo test --workspace` per-suite identical to the pre-fix run at 556, and
`python scripts/check.py` **15/15** — the same fifteen steps, not a redefinition of them. The shipping
configuration was rebuilt too: `cargo build --release -p firmwaresight-desktop --features
custom-protocol` → `Finished release profile in 4m 18s`, binary sha256
`02e7250b7b293ea7eaccd57c9826e1cd73dd0decc09bd33ea87850877b0dd9f7`. What is *not* re-claimed here: the
40-step window smoke and the 18-step CLI smoke were walked on the release binary built from the pre-bump
lockfile, and they are cited as evidence about that tree. That new binary was compiled, not opened — a
patch-level bump of a transitive proc-macro is no reason to re-walk a window, and it is no evidence that
one was walked either.

## 5. CLI Gate smoke (§62) — 18/18, re-run against the fixed binary

Against throwaway Git projects with the release `fwsight.exe` (sha256
`c3f7dca75c67febcdd632bf12f7f8714eae1c48b470aff3427e59b3d705159b6`):

```
PASS  human output exits 4 (review)  |  exit=4, stdout 3234 chars, stderr 5 diagnostic lines
PASS  human output names all ten rules  |  absent: []
PASS  human output groups by state  |  REVIEW PASS N/A
PASS  json exits 4 and stdout is one document  |  exit=4, one document of 10 findings
PASS  jsonschema validates gate-results v1  |  0 errors
PASS  no absolute path in the document  |  shapes found: []
PASS  every backslash left is policy text, not a path  |  refs carrying one: []
PASS  locators are the six Core schemes  |  artifact, diff, evidence, file, git, policy
PASS  the same input twice is byte-identical  |  sha256 89405df9c436a0af
PASS  exit 0 when every rule passes or does not apply  |  exit=0 aggregate=PASS
PASS  a different policy is a different run  |  gate-322a02a vs gate-a64631b
PASS  a dirty workspace exits 5 on git.clean  |  exit=5 git.clean=BLOCK
PASS  the workspace is clean again
PASS  missing notes exit 5 and name the project-relative path  |  exit=5 severity=BLOCK
PASS  the notes file is back
PASS  no repository is UNKNOWN at its policy disposition, never clean  |  exit=4 git.clean=UNKNOWN/REVIEW version=UNKNOWN/REVIEW
PASS  an UNKNOWN is not offered as acceptable  |  the portable document carries no acceptance field
PASS  gate leaves no database or report behind  |  42 files unchanged
```

`refs carrying one: []` is the post-fix form: before defect F was fixed, that same step reported the
version pattern as a locator carrying a separator.

## 6. Desktop smoke (§61)

40 steps walked on the shipping binary in Run 3, with defects F and H re-verified in the shipped
build and CLI/desktop run-id parity measured. Full table, inputs, tooling and database handling:
`P3_GATE_DESKTOP_SMOKE_REPORT.md`.

## 7. Findings closed by this stage

| | | What it was | How it is pinned |
| --- | --- | --- | --- |
| E | visual | A disabled primary button kept its accent border | `Release.module.css` / `Compare.module.css` `:disabled` rules; re-seen at smoke step 23 |
| F | blocking | The `[version] pattern` text was embedded in an evidence locator, so **any** project using the only MVP version source failed to persist a run with `ERR-STORAGE-4006` | locator is now `policy:version.pattern`; `release_gate.rs::a_version_pattern_that_carries_a_separator_still_persists_the_run`, the Core pattern test, and `config.rs` rejecting interior separators in configured paths |
| H | blocking | One build produced two run ids: the CLI read facts off disk, the desktop read them from SQLite, and the footprint evidence pointer plus a `reason` string were inside the fingerprint | `FOOTPRINT_EVIDENCE_FIELDS` + `Database::evidence_id_for_field` hydrate the pointer; `canonical_memory` no longer carries `reason`; `a_stored_build_fingerprints_exactly_like_a_fresh_analysis`; measured again across surfaces in the smoke |
| I | test-side race, found by this round's gate | `check.py` returned **13/14** with `frontend/test` failing on `compare.test.tsx > the change tables > keeps a size that was never recorded as Unknown with its reason` — `Unable to find an element with the text: g_threshold`. `Compare.tsx:1303` renders `Loading symbol changes…` inside the same region, so the awaited region lookup resolves while the table is empty and the synchronous `within(table).getByText(…)` loses the race with the second IPC call. The rows were never wrong: this is P2's test file, not product behaviour | The three row queries that follow a freshly loaded table are awaited instead (`.noinit`, `.oldboot`, `g_threshold`); the assertions are unchanged, so a row that genuinely never arrives still fails. 10 consecutive `compare.test.tsx` runs green after 1 failure in 5 before, then the full gate re-run |

## 8. Known limits carried forward

- Object/module attribution remains unavailable by evidence design (P0/P1 limit).
- P1 nullable-reason gaps remain: `sections.file_offset`, `symbols.address`.
- The Release page cannot open an older run by id; `get_gate_run` accepts one and is tested.
- The `Memory budgets` table is wider than the page and scrolls horizontally.
- Git provenance is read through the system `git` binary with a timeout; a missing or slow `git`
  yields Unknown facts, never a guessed clean tree.
- A local `deny/cargo-deny` green is only as current as this machine's crates.io index. §4.1 is the
  case in point: the step passed here and failed in CI one push later, on a yank published the same
  day. CI, not this host, is the authority on yanked and advisory state; `check.py` already says so
  when `cargo deny` is absent, and this limit is the mirror image of that.

## 9. Scope discipline

No third-party runtime dependency was added beyond what ADR-0027 admitted; `Cargo.lock` gained
first-party edges only. The one lockfile change after closure (§4.1) moved an already-present
transitive proc-macro off a yanked patch and admitted nothing. No P4 surface exists: no release
prepare, bundle, manifest, bundle hashes, History page (§64). No pricing, accounts, auth, cloud,
telemetry, AI judging, updater, signing, SBOM, CVE, OTA or flashing (§65). FirmwareSight's own
repository was never used as a Gate subject.

## 10. Open-source license gap

`OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`. No workspace license metadata was changed,
no license text was added, and nothing in this stage pretends first-party licensing is resolved. The
`deny/licenses` check passes because third-party licenses are individually acceptable; the product's
own license remains the owner's decision.

## 11. Governance state written by this stage

`P0 PASS · G1 PASS · P1 PASS/COMPLETE · P2 PASS/COMPLETE · P3 PASS/COMPLETE · P4
NEXT_AUTHORIZABLE_STAGE · G2 NOT REACHED · V0 NON_BLOCKING_USER_FEEDBACK_TRACK · pricing/commercial
DEFERRED_POST_MVP · active_task NONE · baseline_version 0.6.0`.
