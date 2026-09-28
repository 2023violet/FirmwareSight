---
title: "Baseline Changelog"
doc_id: "FS-ROOT-CHANGELOG"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-28"
---

# Changelog

## Unreleased — P0 remote CI Run #3 green; awaiting promotion sign-off (2026-09-28)

The round-2 HEAD `1cd6309` was pushed to `origin/main` (`git push origin main`, no force) and GitHub
Actions run `36399805005` concluded **`success` — 7 of 7 jobs green**. This entry records the measured
fact and deliberately does not promote anything.

| Run #3 job | Conclusion | Evidence |
| --- | --- | --- |
| Rust (windows-latest) | success | gate green; the Linux apt step is `skipped` there by its `if: matrix.os == 'ubuntu-latest'` guard |
| Rust (ubuntu-latest) | success | 11/11 steps, prerequisites installed |
| Desktop UI (windows-latest) | success | 10/10 steps |
| Desktop UI (ubuntu-latest) | success | 10/10 steps |
| Generated output drift | success | `Install Linux prerequisites for the Tauri shell` ran (`Setting up libwebkit2gtk-4.1-dev`), then `5/5 steps passed`, with `desktop icons are current (pixel-identical to this build).` |
| Dependency policy | success | `advisories ok, bans ok, licenses ok, sources ok` |
| macOS Core Smoke | success | 7/7 steps |

- P0 moves to `CONDITIONAL_PASS`. The condition is named explicitly: the architect's promotion
  sign-off. No document in this repository writes `P0 = PASS` from the coding side.
- `baseline_version` stays `0.5.1`; `FirmwareSight_Project_Baseline_v0.6.0` is **not** generated.
  `SHA256SUMS` still differs on the same nine governance/execution documents and is not regenerated —
  regenerating it would overwrite the frozen v0.5.1 integrity record, which is not this round's to do.
- Runs #1 and #2 stay in `P0_CI_REPORT.md` as failed history. A third green run does not edit the two
  red ones into something they were not; `last_remote_ci` now points at Run #3 and the two earlier
  runs move to `previous_remote_ci` / `first_remote_ci`.
- What a green CI does not buy: G1 still requires V0 (`0 / 8` eligible external sessions — CI cannot
  recruit participants), P1 still requires its own authorization prompt, peak RSS is still
  `NOT MEASURED`, and the fuzz campaign is still `NOT RUN`.
- The next prompt is the architect's *P0 Final Promotion / v0.6.0 Baseline Closure*, and it is not
  anticipated here.

## Unreleased — P0 remote CI Run #2 and its final drift closure (2026-09-28)

Authorization: *FirmwareSight P0 — Remote CI Run #2 Final Drift Closure v1.0*. Target
`LOCAL FIX COMPLETE / READY FOR REMOTE CI RUN #3`. Baseline stays `0.5.1`.

The first remediation round's HEAD `ebda52d` was pushed and ran as `36378384225`. It concluded
`failure` with **six of seven jobs green**, which closes Run #1's four causes remotely:

- `Rust (windows-latest)` and `Rust (ubuntu-latest)` both passed 5/5 steps at **104 tests**, the
  Ubuntu one compiling `firmwaresight-desktop` — the fixture text policy and the Linux prerequisites
  are therefore proven on GitHub's runners, not only on this machine.
- `Desktop UI (ubuntu-latest)` passed with `Tests 19 passed (19)`.
- `macOS Core Smoke` passed 3/3 at **87 tests**, its first execution ever.
- `Dependency policy` passed: `advisories ok, bans ok, licenses ok, sources ok`.
- `drift/desktop icons` passed on Linux printing
  `desktop icons are current (pixel-identical to this build).` — the evidence an encoder-byte
  assertion could never produce.

One job stayed red, and the cause is this repository's own omission rather than the product's: the
`drift` gate regenerates ts-rs bindings with `cargo test -p firmwaresight-desktop`, which links the
GTK stack, and only the `rust` job had been given the prerequisites. Its log reads
`Package gobject-2.0 was not found in the pkg-config search path`. Classified as a **CI job
provisioning duplication defect**.

- Fix: the already-remotely-proven apt block, copied into the `drift` job. 21 added lines in
  `.github/workflows/p0-check.yml`, and nothing else. No product source, fixture, golden, schema,
  `deny.toml`, lockfile or `scripts/check.py` change; no `continue-on-error`, skip, trigger or
  permission change; the drift job still runs all five of its steps.
- Abstracting the ten apt lines into a script was considered and rejected: two jobs need it, and a
  shell portability plus testing surface is not a smaller change.
- The desktop smoke was not repeated. Round 2 changes no runtime source, so
  `P0_DESKTOP_SMOKE_REPORT.md` is carried forward.
- `Linux provisioning itself = NOT LOCALLY EXECUTED`: this host has no Ubuntu. The evidence chain is
  the same runner family, the same package list already proven remotely and the same compile
  requirement; Run #3 gives the verdict.
- **Architect disposition recorded:** `RUSTSEC-2024-0429` and `RUSTSEC-2024-0370` are accepted as
  explicit P0 transitive risk, not silent suppression, and do not block P0 promotion, with five
  revisit triggers. No architecture ADR is required because no architecture choice changed.
- Fixed while writing this round's record: four `BASELINE.yaml` values containing `Run #1` / `Run #2`
  were being silently truncated at the `#` by YAML's inline-comment rule. They are quoted now and the
  parsed values were re-read to confirm.
- New document: `P0_TECHNICAL_VALIDATION/P0_CI_RUN_2_CLOSURE_REPORT.md`; the pack is now 22 files.
- Status as this round closed: `FAIL — REMOTE CI RUN #2`, remediation round 2 `LOCAL FIX COMPLETE`,
  `RUN #3 REQUIRED`. The entry above records what Run #3 then measured.
  is not claimed, V0 remains `0 / 8`, P1 is not authorized, `v0.6.0` is not generated.

## Unreleased — P0 CI closure remediation (2026-09-28)

Authorization: the architect prompt *FirmwareSight P0 — CI Closure / Cross-Platform Reproducibility
Remediation v1.0*. Target `LOCAL REMEDIATION COMPLETE / READY FOR REMOTE CI RERUN`. No version is
claimed here either: the baseline stays `0.5.1`.

The P0 tree was pushed as `f9b8ccb` and GitHub Actions run `36360310447` measured it: **failure**, two
of six jobs green. That replaced the pack's own `CONDITIONAL_PASS (LOCAL)`, and it is kept as history
rather than rewritten.

- **Fixture byte identity no longer depends on the checkout.** `*.ld` had no `.gitattributes` rule, so
  `core.autocrlf=true` smudged a linker script from the recorded LF bytes to CRLF on Windows and the
  hash test compared different bytes. Fixed in the Git text policy (`*.ld text eol=lf`, and `*.map`
  given an explicit `-text`); no expected hash was moved to accommodate CRLF and no newline was
  normalized before hashing - the manifest's job is to verify the byte identity of committed evidence.
  Verified with a fresh clone.
- **Desktop icon drift is now judged by pixels.** Pillow's PNG and ICO encoders emit different bytes
  for identical RGBA data depending on encoder state, so the byte check tested the library. The check
  decodes instead: PNG dimensions and pixels, and for the `.ico` the required size set with per-frame
  pixels, where a missing frame fails. No icon design or token changed, and no Linux-regenerated set
  was committed in place of the recorded one.
- **`deny.toml` is executable under the version CI installs.** `severity-threshold` and
  `highlight-warnings` are not keys of cargo-deny 0.20.2 and five `licenses.allow` entries were slash
  strings; the file did not parse. Rewritten against the keys the tool accepts, with the allow list
  rebuilt from `cargo deny list`, `[graph] targets` limited to the four shipping platforms, and
  `anyhow` banned with its three Tauri wrappers named so first-party use still fails. `cargo deny
  check licenses bans sources advisories` now exits 0 here instead of being `SKIPPED`.
- **The Ubuntu job installs the Tauri Linux prerequisites** rather than excluding the desktop crate,
  so `clippy --all-features` keeps its cross-platform compile coverage.
- **A macOS core smoke job exists**, because `05_ENGINEERING/06_CI_CD_BASELINE.md` requires one on
  `main` push and the workflow had none. It runs `python scripts/check.py --only core-smoke` - the
  same script, not a fourth copy of the tests - and is locally green (3/3).
- **The desktop window was launched for real**, and found a defect no test had reached: `evidence`
  had a whole-table primary key on `id`, so a second artifact raised `UNIQUE constraint failed` while
  `04_TECH/15` §4 declares `Build 1─N Evidence`. Migration `0002` rebuilds the table on
  `(build_id, id)`; schema version is 2 and the Rust total moved 102 -> 104. The failing tests were
  written first, and the upgrade was replayed against the real database the launch left behind.
- **Reported as an architecture conflict, not fixed:** `RUSTSEC-2024-0429` (`glib 0.18.5`) and
  `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`) enter through the gtk-rs `0.18` line Tauri `2.12.0`
  requires, and `cargo update -p glib --precise 0.20.0` fails against `gtk = "^0.18"`. Closing them
  would change the frozen desktop dependency architecture, which needs an ADR.
- New documents: `P0_TECHNICAL_VALIDATION/P0_CI_REMEDIATION_REPORT.md` and
  `P0_TECHNICAL_VALIDATION/P0_DESKTOP_SMOKE_REPORT.md`; the pack is now 21 files.
- Status as that round closed: the remediation HEAD was still unpushed, so nothing was green in CI
  yet, and `REMOTE CI PASS` was not written. The entry above this one records what the push measured. G1 is still not claimed, V0 still `0 / 8`, P1 still not authorized.

## Unreleased — P0 Technical Vertical Slice executed (2026-09-28)

No version is claimed here on purpose: `0.5.1` stays the only baseline, and `v0.6.0` is reserved for
an unconditional P0 `PASS`. This entry records work on the working tree.

### P0 source (first production code in this repository)

- Rust workspace, pinned toolchain 1.98.1, exactly four Phase-0 library crates:
  `firmwaresight-core` (declares zero dependencies), `-artifact`, `-report`, `-storage`;
- `fwsight` CLI: synchronous, deterministic JSON, exit codes 0 / 2 / 3 exercised, 4 / 5 / 6 not
  registered because no code path returns them;
- untrusted-input intake order `stat -> size guard -> streaming SHA-256 -> magic detect -> single
  read -> parse`, with the 512 MiB guard measured from both sides of the boundary;
- ELF parsing plus a memory-accounting model whose evidence ladder limits what a verdict may claim;
- GNU ld MAP adapter producing a real verified region fact, and refusing foreign-toolchain maps
  instead of approximating them;
- minimal SQLite persistence via `rusqlite + bundled`, one migration, transactional import,
  content-addressed dedupe;
- thin Tauri 2 desktop shell with a use-case-oriented command surface (`FixtureKey`, never a path),
  ts-rs-generated TypeScript bindings, and analysis off the calling thread;
- React 19.3 + TypeScript 6.0.3 summary screen, token-driven, five-state badges;
- real ARM ELF fixtures with recorded provenance and a regeneration script, four malformed inputs,
  goldens, and generated 100 / 256 / 512 MiB workloads (gitignored);
- one verification gate, `scripts/check.py`, that CI calls unchanged (14 steps, four job groups);
- `deny.toml` encoding the AGENTS.md dependency red lines as a ban list.

### P0 status

Status as recorded at delivery, superseded by the remediation entry above and retained as history:

- `EXECUTED — CONDITIONAL_PASS (LOCAL)`; 102 Rust tests and 19 UI tests passed locally.
- Conditions, all outside this environment: CI had never run, `cargo deny` had never run, peak RSS is
  `NOT MEASURED`, and the desktop window had never been opened. Two of the first two then failed when
  they were finally run, and the third was answered by a real launch.
- `P0_TECHNICAL_VALIDATION/` holds the evidence pack: at delivery the 17 documents the prompt names,
  plus `P0_EXECUTION_PROVENANCE.md` from the takeover and `P0_DESIGN_CHECKLIST.md` required by
  `AGENTS.md` 11 - 19 files, each citing executed commands.
- G1 is not claimed; V0 remains deferred with `0 / 8` external sessions; P1 is not authorized.

### Corrections made to this pack's own claims during P0

- `fixture.toml` recorded the linker invocation as `-p0-dual-region.ld`; the toolchain was called with
  `-T p0-dual-region.ld`. Regenerating proved both ELF files byte-identical, so the record was wrong
  and the artifacts were not.
- The release desktop binary was described as embedding the built UI. It did not: without Tauri's
  `custom-protocol` feature the dev-mode codegen embeds nothing. The feature was added and the
  production link re-measured (10,398,208 bytes against 10,330,112).
- The Rust test total was recorded as 101 before a ninth storage test landed; it is 102.
- `AGENTS.md` 11 requires every UI change to pass `templates/DESIGN_CHECKLIST_TEMPLATE.md`, and no
  filled copy existed. Running it found four things instead of confirming the work: `DESIGN.md` 3
  never tokenised border width while `DESIGN.md` 4 requires hairline borders, so the eight `1px`
  borders are a gap in the design contract that needs a frozen-asset version bump to close; the
  summary screen had **no live region**, so a screen reader was never told that an analysis started or
  finished; the fixture `select` styled only hover and focus and stayed usable mid-request; and
  capability badges display Core's enum words ("supported", "not-provided") rather than product copy.
  The two accessibility findings were fixed in source and are covered by a new test rather than filed
  for later. The other two stay open, recorded in `P0_DESIGN_CHECKLIST.md` and
  `P0_KNOWN_LIMITATIONS.md`: one needs a frozen design asset to change, which P0 may not do, and the
  other is a boundary decision about who owns user-facing wording.
- The UI's `packageManager` pin drifted one patch below the frozen baseline: it read
  `pnpm@12.6.0` where `examples/package.baseline.json` and `04_TECH/10_TOOLCHAIN_BASELINE.md` both
  name 12.7.0. Nothing failed — which is why it is recorded as a catch of the baseline cross-check
  rather than of the gate. It is now pinned at 12.7.0, and the only lockfile change was that
  metadata plus the toolchain's own `@pnpm/exe` entries; no application dependency moved.
- `apps/desktop/ui/package.json` is the only package.json in the tree. The repo-structure baseline
  sketches a root `package.json`, `pnpm-workspace.yaml` and `.node-version`; P0 ships one frontend
  package with the node floor expressed as `engines: ">=24.0.0 <25.0.0"`, which meets that sketch's
  stated intent ("Frontend is one workspace package at MVP") without the root plumbing. Recorded as
  a named simplification in `P0_KNOWN_LIMITATIONS.md` rather than left implicit.

### Unchanged

- `BASELINE.yaml` still reports `baseline_version: 0.5.1`; `SHA256SUMS` still describes the frozen
  v0.5.1 package and was deliberately not regenerated; `DIRECTORY_TREE.txt` still lists that package,
  not this working tree. Six manifest entries now differ — all governance or execution records P0 was
  authorized to change, listed in `.ai/DECISIONS.md`.

## 0.5.1 — 2026-09-27

### V0 execution
- executed the authorized V0 Prompt v1.1 against the unique v0.5.0 baseline;
- created the `V0_VALIDATION/` execution workspace;
- implemented the seven-screen clickable static prototype;
- encoded MAP STATE A→B and parse-failure recovery states;
- implemented demo review acceptance and conditional Bundle flow;
- added moderator, screening, task, interview and privacy protocols;
- added session template and analysis registers;
- ran internal Node/state-machine/scope/design dry-run checks.

### Evidence limitation
- external participant sessions completed: 0;
- minimum required: 8;
- V0 status: `INCOMPLETE — insufficient external sample`;
- no V0 PASS/G1 claim.

### Scope
- no Cargo/Rust/Tauri/SQLite production implementation;
- P0 remains unauthorized in this V0 execution;
- no E1/E2/E3/GX feature implementation.

## 0.5.0 — 2026-09-27
Expert-review verification + whole-product consolidation baseline; complete seven-screen UI source set; Post-MVP candidate governance.

## 0.4.0 — 2026-09-27
Audit resolution + UI baseline freeze.

## 0.3.0 — 2026-09-26
Lifecycle baseline.

## 0.2.0 — 2026-09-26
Technical baseline.

## 0.1.0 — 2026-09-26
Initial product baseline.
