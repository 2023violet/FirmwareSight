---
title: "Decision Summary"
doc_id: "FS-AI-003"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-28"
---

# Decisions — v0.5.1

All v0.5.0 architecture/product/design decisions remain in force.

## V0 execution decisions

- The v1.1 execution prompt is authorized and preserved by SHA-256.
- V0 prototype is physically isolated under `V0_VALIDATION/`.
- No Cargo/Rust/Tauri/SQLite production implementation is created.
- MAP availability follows explicit STATE A→B timing.
- Parse failure is STATE C and preserves Last Good Artifact.
- Export controls are prototype-disabled; no E1 exporter is implemented.
- Review acceptance preserves original REVIEW and records disposition metadata.
- No mathematical V0 PASS threshold is invented.
- Formal V0 completion requires minimum N=8 eligible external sessions.

## Current Gate status

`V0 INCOMPLETE — insufficient external sample`

This is an evidence status, not a product-quality verdict.

It remains true after the P0 authorization below. V0 was re-sequenced as a non-blocking
research track; it was not completed, and no V0 participant evidence was created.

# P0 execution decisions — added 2026-09-27

Authorization:

- The P0 Technical Vertical Slice prompt v1.1 (Architect reviewed) is authorized and preserved
  by SHA-256 `60a59196708a53592be4d828a0c1275cf74b3bfad0bdc6455f863ab38d32929b`.
- Running P0 while V0 is unvalidated is not a new exception: `ADR-0020` already authorizes
  V0 and P0 as parallel tracks, with P1 gated on both.
- Only the execution state changed. No frozen product, architecture, evidence, gate or design
  baseline was altered, so no new architecture ADR was required to start.

Sequencing:

- V0 is now `DEFERRED / NOT YET EVIDENCE-VALIDATED`, a non-blocking research track.
- `0 / 8` external sessions stays on record; the Batch A hold on `v0.5.2` still stands.
- Formal G1 is not claimed while V0 is unvalidated, even if P0 passes.
- P1 requires a separate explicit authorization prompt.

Scope:

- First production code is created by this decision: Rust workspace, exactly four Phase-0
  library crates, `fwsight` CLI, minimal Tauri 2 shell, minimal React summary, fixtures,
  goldens, typed IPC, minimal SQLite, CI.
- `V0_VALIDATION/` is preserved read-only as research evidence; P0 code must not import the
  V0 static prototype into the production frontend.

Evidence-integrity rules accepted for this execution:

- Memory accounting must be proven from real ELF load/runtime evidence on a controlled fixture;
  a `.data` section-name heuristic may only be recorded as Derived/low-confidence and may never
  become a hard BLOCK basis.
- The GNU ld MAP adapter must produce at least one deterministic verified real fact; a detect
  plus capability claim alone does not count.
- Measured numbers only. Anything not measured is written as `NOT MEASURED` with a reason.
- LOCAL PASS, CI PASS and NOT RUN stay distinct; no unrun CI may be reported as green.
- Final status vocabulary is restricted to PASS / CONDITIONAL_PASS / FAIL / BLOCKED.

Observed environment facts that shaped the plan:

- The Rust toolchain matches the frozen pin exactly (1.98.1), so no baseline substitution was
  needed.
- The Tier 1 host is Windows, where the host C compiler emits PE/COFF rather than ELF, so real
  ELF fixtures come from `arm-none-eabi-gcc` instead of a host build.
- Installed pnpm is 11.21.0, below the baseline pnpm 12 line; corepack can resolve 12.7.0, so the
  baseline major line is kept rather than lowering it.

---

# P0 delivery record — added 2026-09-28

Corrections to the start-of-track observations:

- Corepack resolves **pnpm 12.7.0**, and `apps/desktop/ui/package.json` pins exactly that. The
  `packageManager` field briefly read `pnpm@12.6.0` — one patch below the frozen
  `examples/package.baseline.json` and `04_TECH/10_TOOLCHAIN_BASELINE.md`, both of which name
  12.7.0. Caught by cross-checking the delivered pin against the frozen assets before delivery, not
  by a failing test: 12.6.0 built and ran everything fine, which is exactly why a baseline drift of
  this kind needs a deliberate check. `pnpm install` at 12.7.0 initially refused
  `--frozen-lockfile` because the lockfile recorded the writing version; the only change was that
  metadata and the toolchain's own `@pnpm/exe` entries — no application dependency moved.

Decisions taken while executing, none of which moved a frozen baseline:

- `arm-none-eabi-gcc` supplies the ELF fixtures because the Windows host compiler emits PE/COFF. The
  binaries are committed with provenance, and tests are hash-first, so no test needs the toolchain.
- `firmwaresight-desktop` declares Tauri's template `custom-protocol` feature. Without it the
  release binary embeds no frontend, and the gate's `clippy --all-features` was the only step that
  made that visible. Recorded in `P0_IMPLEMENTATION_LOG.md` 18.
- TypeScript is pinned to 6.0.3 because typescript-eslint requires `<6.1.0`; 7.0.2 was available and
  deliberately not taken.
- One golden file was deleted (`golden/reports/p0-basic-summary.json`) and one first-party unused
  dependency set was removed. Both are in the implementation log.

Final status and what it does not mean:

- P0 was delivered as `EXECUTED — CONDITIONAL_PASS (LOCAL)` on the strength of a locally green gate.
  That label is retired: see the remediation record below, which replaces it with measured remote
  results.
- Baseline stays `0.5.1`. `v0.6.0` is reserved for an unconditional `PASS`, and `g1_claimed` stays
  `false`; P0 passing alone would not open G1 while V0 is unvalidated (ADR-0020).
- `sha256sum -c SHA256SUMS` fails on exactly nine entries, all governance or execution records that
  P0 was authorized to change: `.ai/ACTIVE_TASK.md`, `.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`,
  `.ai/HANDOFF.md`, `10_AUDIT/SOURCE_PROMPTS/README.md`, `BASELINE.yaml`, `README.md`,
  `CHANGELOG.md`, `INDEX.md`. `SHA256SUMS` itself is left as the v0.5.1 manifest: regenerating it
  would overwrite the frozen baseline's own integrity record, which is not P0's to do. The drift is
  therefore listed by name here rather than erased.
- P1 is not authorized and was not started.

---

# P0 CI closure remediation — added 2026-09-28 (superseded on the same date by the Run #2 section below)

Authorization:

- The architect prompt *FirmwareSight P0 — CI Closure / Cross-Platform Reproducibility
  Remediation v1.0 (Architect Reviewed)* is active. It was supplied inline rather than as a file,
  so unlike the V0 and P0 prompts there is no stored source file whose SHA-256 can be recorded; the
  repository's copy of its terms is this section plus `P0_CI_REPORT.md`.
- It authorizes local remediation, one `cargo-deny@0.20.2` installation and one real desktop window
  launch. It does not authorize a push, and the coding side does not write `REMOTE CI PASS`.

Status decision, taken from measured results rather than from the previous round's wording:

- P0 became `FAIL — REMOTE CI RUN #1` with remediation `LOCAL FIX COMPLETE`, a state this section
  records and the next section supersedes. `CONDITIONAL_PASS (LOCAL)` was not kept: the conditions
  that made it conditional were tested, and two of them failed. "CI has never run" and "cargo deny has
  never run" stopped being true statements about this repository at that point.
- The distinction this preserves is between what was proven (Core facts, memory accounting, the
  typed IPC boundary, storage migrations, determinism — 104 Rust and 19 UI tests) and what the
  first cross-platform run exposed (checkout text policy, runner provisioning, encoder-byte
  assertions, an unparseable supply-chain config, a missing macOS job).

What was deliberately not done:

- No expected fixture hash was edited to absorb a line-ending conversion, no hash assertion was
  weakened, and no newline normalization was added before hashing; the fix belongs in the Git text
  policy that decides what a checkout contains.
- The Ubuntu `clippy` failure is being closed by installing the Tauri Linux prerequisites, not by
  excluding `firmwaresight-desktop` — the desktop shell's cross-platform compile boundary is part
  of what the baseline validates.
- The icon drift check is being changed to compare decoded pixels rather than compressed bytes,
  after a local experiment showed the bytes move with encoder settings while the pixel content does
  not. A Linux-regenerated icon set was not simply committed in its place.
- `deny.toml` is being repaired against the keys `cargo-deny 0.20.2` actually accepts, and the
  license findings it then produces are handled with real SPDX, crate-specific exceptions or a fully
  specified `licenses.clarify` — not with a broader allow list.
- Baseline stays `0.5.1`, `v0.6.0` is not generated, G1 is not claimed, P1 is not started, and no
  `ADR-0020` term was changed.

Unchanged facts that this round must not quietly drop: peak RSS stays `NOT MEASURED` with its
recorded reason, and it is recorded as a measurement gap rather than a promotion blocker, because
the original P0 prompt accepted an unmeasured number when the reason was stated. `SHA256SUMS` stays
the frozen v0.5.1 package integrity record; the working source tree is not that package.

Decisions the remediation itself had to make:

- **Schema version 2, because version 1 contradicted the frozen data model.** The authorized desktop
  launch failed on the second artifact with `UNIQUE constraint failed: evidence.id`
  (`ERR-STORAGE-4006`). `evidence` carried a whole-table primary key on `id`, while
  `04_TECH/15` §4 states `Build 1─N Evidence`: the key was wrong, not the design. Migration `0002`
  rebuilds the table on `(build_id, id)`. `AGENTS.md` 2 forbids changing persistence schema
  *semantics* without an ADR; this change makes the stored relation equal to the semantics the
  baseline already froze, and it is recorded here for architect review rather than presented as a
  settled new decision. Tests were written first and failed first: two artifacts recording the same
  field identifier, and a version-1 database upgraded without losing its evidence.
- **Two advisories are an architecture conflict, not a config error.** `RUSTSEC-2024-0429`
  (`glib 0.18.5`, unsound) and `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`, unmaintained,
  host-only) reach the graph through the gtk-rs `0.18` line that Tauri `2.12.0` requires, and
  `cargo update -p glib --precise 0.20.0` fails against `gtk = "^0.18"`. Resolving them means moving
  gtk-rs, muda, tao and webkit2gtk, i.e. changing the frozen Tauri dependency architecture. They are
  ignored in `deny.toml` with their evidence, and the conflict is reported to the architect as a
  decision that needs an ADR — it was not resolved by upgrading a dependency, by raising a severity
  threshold, or by dropping the advisories check.
- **`ISC` and five slash-form SPDX strings left the allow list.** cargo-deny 0.20.2 parses
  `MIT/Apache-2.0` as a disjunction and no crate in the four shipping targets declares `ISC`, so the
  literal entries matched nothing; `licenses ok` after their removal is the check that says so.
- **Icon drift is measured by pixels, deliberately losing byte-level assertions.** PNG and ICO bytes
  depend on encoder state; `git status` will stay clean across a rebuild that changes only encoder
  settings. A missing ICO frame still fails.
- **The macOS job reuses the existing gate.** `macos-core` runs
  `python scripts/check.py --only core-smoke` — fmt, clippy and test over the five Core-side crates —
  rather than a fifth crate or a copy of the test list. It is locally usable and locally green
  (3/3), and it runs on `main` pushes but not on pull requests, which is what
  `05_ENGINEERING/06_CI_CD_BASELINE.md` requires.

---

# Remote CI Run #2 and its final drift closure — added 2026-09-28 (superseded the same day by the Run #3 section below)

Authorization:

- The architect prompt *FirmwareSight P0 — Remote CI Run #2 Final Drift Closure v1.0 (Architect
  Reviewed)*. Supplied inline, so there is no stored file to hash; the repository's copy of its terms
  is this section plus `P0_CI_RUN_2_CLOSURE_REPORT.md`.
- It authorizes a workflow-scoped fix and the governance record of Run #2. It does not authorize a
  push, does not permit `REMOTE CI PASS` from the coding side, and is explicitly not an architecture
  redesign, not P0 promotion and not P1.

Status decision, from the run rather than from the previous round's report:

- P0 became `FAIL — REMOTE CI RUN #2`. Run `36378384225` on head `ebda52d` concluded `failure` with
  **six of seven jobs green**. `last_remote_ci` points at it; `previous_remote_ci` keeps Run #1 as
  historical failure evidence instead of overwriting it.
- The first remediation round is **confirmed remotely**: fixture byte identity, Ubuntu Rust
  prerequisites, the pixel-based icon check (its Linux log line reads
  `desktop icons are current (pixel-identical to this build).`), the cargo-deny policy, the macOS core
  smoke and both frontend matrix jobs. 104 Rust tests ran green on both Windows and Ubuntu, and 19 UI
  tests on Ubuntu.
- The one red job is classified as a **CI job provisioning duplication defect**, not a product
  defect: `drift` regenerates ts-rs bindings with `cargo test -p firmwaresight-desktop` and so needs
  the same GTK system libraries the `rust` job installs, but only the `rust` job was given the step.
  `gobject-sys` failed at `pkg-config` exactly where Run #1's `glib-sys` did.

Decisions taken:

- **Copy the proven apt block rather than abstracting it.** Two jobs need ten lines. A shared
  `scripts/*.sh` would add a portability surface and a testing obligation, and the round's target is
  the smallest reversible change that closes a measured failure. The duplication is a known cost,
  stated here, not hidden; if a third job needs it the argument changes.
- **The drift check was not weakened.** It still runs all five steps, still invokes
  `cargo test -p firmwaresight-desktop`, and still asserts with `git diff --exit-code`. The
  alternatives the prompt forbids - dropping the IPC step, moving the job to Windows,
  `continue-on-error`, `if: false`, reclassifying the failure as a warning - were not used, and none
  of them would have produced the evidence the baseline asks for.
- **No product source changed.** The diff is 21 added lines in one workflow file. Fixtures, goldens,
  schemas, `deny.toml`, both lockfiles, `scripts/check.py` and the design tokens are untouched, so the
  desktop smoke evidence is carried forward instead of re-performed for noise.
- **`BASELINE.yaml` quoting fixed.** Four values containing `Run #2` / `Run #1` were being silently
  truncated when the file is parsed, because an unquoted ` #` starts a YAML comment; the earlier
  `remote_state` line had the same defect. They are quoted now and re-read with `yaml.safe_load` to
  confirm the parsed value equals the intended one.

Architect disposition on the two RustSec advisories, recorded as the architect's decision rather than
invented here:

- `RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and `RUSTSEC-2024-0370` (`proc-macro-error 1.0.4`,
  unmaintained) are **accepted as explicit P0 transitive risk, not silent suppression**, and **do not
  block P0 Technical Foundation promotion** on current evidence: both are recorded with reasons in
  `deny.toml`, Run #2 shows all four cargo-deny categories passing, no compatible upgrade exists
  inside the `gtk-rs 0.18` / Tauri 2.12 line, no first-party code calls the affected API, and this
  phase is a technical foundation rather than a GA security certification.
- Revisit triggers: a compatible fixed Tauri 2.x / gtk-rs line becomes available; the advisory's
  classification or severity materially changes; first-party code begins exercising the affected API;
  the P5 Productization dependency/security review; a Linux commercial release-candidate security
  review.
- **No architecture ADR is required by this decision, because no architecture choice changed.** An ADR
  becomes necessary if Tauri is replaced, dependencies are forked, or the frozen desktop dependency
  family changes. `unused-ignored-advisory` stays at its default `warn`, so an ignore entry that stops
  matching reports itself.

Unchanged: baseline stays `0.5.1`, `v0.6.0` is not generated, G1 is not claimed, V0 remains
`DEFERRED / NOT YET EVIDENCE-VALIDATED` at `0 / 8`, `ADR-0020` is untouched, P1 is not started, peak
RSS stays `NOT MEASURED` with its reason, and `SHA256SUMS` stays the frozen v0.5.1 package record.

---

# Remote CI Run #3 — governance consequence, recorded 2026-09-28

Measured, then decided:

- The round-2 HEAD `1cd6309` was pushed with the owner's explicit instruction (`git push origin main`,
  no force, no history rewrite). Run `36399805005` concluded `success` with **7 of 7 jobs green**,
  including the previously failing `Generated output drift`. Both remediation rounds are therefore
  confirmed by GitHub's runners rather than by this machine.
- P0 becomes `CONDITIONAL_PASS`, not `PASS`. This is not hedging: `PASS`, `baseline_version: 0.6.0`, a
  regenerated `SHA256SUMS` and `DIRECTORY_TREE`, and the archiving of P0 validation state are the
  promotion act the architect issues in its own signed prompt. A coding agent writing them after a
  green run would be doing what Run #1's `CONDITIONAL_PASS (LOCAL)` did in reverse - letting a
  convenient state substitute for the authority that decides it.
- The distinction this pack has kept since the beginning survives the good news: `LOCAL PASS`,
  `CI PASS`, `NOT RUN` and `NOT MEASURED` remain different claims. One Windows step reports `skipped`
  (the apt block guarded to Linux) and is recorded as skipped, not counted as a pass; peak RSS stays
  `NOT MEASURED`; fuzzing stays `NOT RUN`.
- Runs #1 and #2 are kept as failed history. `last_remote_ci` moves to Run #3 and the two earlier runs
  are re-listed as `previous_remote_ci` and `first_remote_ci`, so the sequence that produced green -
  four red jobs, then one, then none - stays legible instead of being replaced by its own conclusion.
- Unchanged by the green run: G1 (still requires V0, and V0 is `0 / 8` eligible external sessions),
  P1 (still requires its own authorization prompt), `ADR-0020`, V0 evidence, design tokens, and the
  two RustSec advisories under the architect's recorded acceptance with their five revisit triggers.
