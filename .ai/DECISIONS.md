---
title: "Decision Summary"
doc_id: "FS-AI-003"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-10-04"
---

# Decisions — v0.6.0

All v0.5.0 architecture/product/design decisions remain in force. Sections below are dated records:
the earlier ones keep their v0.5.1 provenance and are not restated, and the promotion section at the
end is what the current baseline stands on.

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

---

# P0 Final Promotion Decision — 2026-09-28

Authorization:

- *FirmwareSight P0 — Final Promotion / v0.6.0 Baseline Closure v1.0 — Architect Signed.* Supplied
  inline to the execution environment, so there is no stored source file whose SHA-256 this
  repository can record; the copy of its terms kept here is this section plus
  `P0_TECHNICAL_VALIDATION/P0_FINAL_PROMOTION_REPORT.md`, and the prompt audit index says so rather
  than implying a byte-exact archive exists.
- It authorizes one act: promote P0 to `PASS` and freeze the v0.6.0 baseline record. Everything
  outside that - P1, G1, releases, signing, tags - remains unauthorized by it.

```text
Decision            P0 = PASS
Baseline            v0.6.0 — FirmwareSight_Project_Baseline_v0.6.0, the P0 Technical Foundation Baseline
Basis               Remote CI Run #3 (36399805005) on 1cd6309 — success, 7 of 7 jobs
                    Remote CI Run #4 (36402637251) on 5e58f77 — success, 7 of 7 jobs
                    A desktop window opened and driven on the shipping configuration
                    Local gate 14/14 with 0 skipped mandatory steps, plus 3/3 under --only core-smoke
Architecture change none
ADR required        no
V0                  unchanged: DEFERRED / NOT YET EVIDENCE-VALIDATED, 0 / 8 eligible external sessions
G1                  NOT CLAIMED
P1                  NOT AUTHORIZED
```

What the two HEADs mean, recorded separately on purpose: `1cd6309` is the **engineering-validated**
tree - the last commit that contains the source the gate measured. `5e58f77` is the
**architect-reviewed** tree, differing from it by governance, audit and evidence documentation only,
and it is the HEAD Run #4 executed. The promotion commit is a third, distinct HEAD, and it changes
documentation and integrity artifacts only; had it needed a production source change this round would
have stopped with `PROMOTION BLOCKED BY NEW ENGINEERING DEFECT` instead of signing the `PASS`.

What `v0.6.0` does not assert: V0 validation, a G1 pass, a complete product MVP, Compare / Gate /
Bundle workflows, an installer, signing, notarization or an updater, commercial stability, zero
vulnerabilities, or CRA compliance. Product scope is unchanged from v0.5 - the version records a
validated technical foundation, not a new product verb, and `PRODUCT_BASELINE.md` carries a dated note
saying exactly that.

What the promotion deliberately did not clean up: Run #1's four red jobs and Run #2's one red job stay
published as failed history; peak RSS stays `NOT MEASURED`; fuzzing stays `NOT RUN`; the two RustSec
advisories stay recorded as **accepted explicit P0 transitive risk** with their five revisit triggers;
`SHA256SUMS` and `DIRECTORY_TREE.txt` are regenerated as the v0.6.0 baseline record, which is the one
integrity act the previous rounds declined to take on their own authority.

---

# Pre-G1 sequencing revision and P1-A0 — added 2026-09-28

Authorization:

- *FirmwareSight — Pre-G1 Sequencing Revision + P1-A0 Real Artifact Intake v1.0 (Architect Reviewed)*,
  plus the *P1-A0 Design Contract Closure Addendum v1.0*. Both were supplied inline, so this repository
  records that fact instead of claiming a hash for bytes it never received
  (`10_AUDIT/SOURCE_PROMPTS/README.md`).
- The architect's decision is recorded as **ADR-0025**, and ADR-0025 supersedes exactly one clause of
  ADR-0020 — `P1 Product MVP implementation 只有两者都 PASS 后开始` — leaving the parallel-validation
  decision, `G1 = V0 PASS + P0 PASS` and every other ADR-0020 consequence intact.

```text
Decision            After P0 PASS, the architect may authorize one bounded, reversible, low-coupling
                    Pre-G1 Analyze slice without waiting for V0 PASS
Authorized slice    P1-A0 — real artifact intake + Analyze summary
Stop                P1-A0 only. P1-A1, P2, P3 and P4 remain unauthorized.
P1                  NOT PASS, NOT CLOSED
G1                  NOT CLAIMED (V0 still 0 / 8 eligible external sessions)
Baseline            stays v0.6.0 — a development slice does not create a version
Next gate           V0 Batch A >= 4 eligible sessions + interim architect review + a new prompt
```

The reasoning recorded in the ADR, in short: waiting entirely on V0 binds engineering speed to
recruitment speed, while opening P1-P4 immediately accumulates sunk cost in workflow depth nobody has
validated with a real user. Bounded parallelism takes the middle, and the bound is written into
`06_DELIVERY/06_STAGE_GATES.md` rather than left as a reading of prose.

Decisions taken inside the authorized scope:

- **The dialog plugin is decided here, not assumed.** `00_GOVERNANCE/03_DECISION_POLICY.md` lists
  插件系统 as ADR-requiring, so adopting `tauri-plugin-dialog` is written into ADR-0025's Decision
  section rather than slipped in as a dependency detail. `04_TECH/11_DEPENDENCY_BASELINE.md` already
  permits a dialog plugin for explicit user file selection, and the consequence is fixed: dialogs open
  from Rust-side use-case commands, the WebView gains no filesystem, shell or network surface, and no
  generic path-taking command exists.
- **No ADR for the token bump.** The addendum's test was whether repository authority explicitly
  requires one. Policy line 24 covers plugin systems, not design tokens; the no-ADR list covers UI
  changes that do not alter semantics; and the checklist rule for a token gap is a version bump. So
  `assets/design-tokens.json` moves `0.2.0 -> 0.2.1` with exactly one added semantic
  (`border.width.hairline = 1`) and no new ADR. Focus keeps its dedicated 2px token.
- **`active_task` now names the slice.** `BASELINE.yaml` carries a `pre_g1_execution` block so the
  authorization, its stop condition and its next gate are readable rather than inferred from this file.
  `p0_execution` keeps its frozen verdict — `status: PASS`, the promotion records and the remediation
  history are untouched; what was added there is the remote run this round measured, so the file states
  one current remote fact instead of two competing ones.
- **V0 wording moves, V0 numbers do not.** The track is described as
  `ACTIVE_EXTERNAL_VALIDATION_WAITING_FOR_REAL_PARTICIPANTS`; `external_participants_completed` stays
  `0`, Batch A stays `0 / 4-5`, and nothing in `V0_VALIDATION/**` is authored or edited by the coding
  side to make that wording true.

Historical finding now closed: `P0_DESIGN_CHECKLIST.md` and `P0_KNOWN_LIMITATIONS.md` recorded eight
authored structural `1px` borders with no token behind them. That gap was real when it was found and it
stays written down that way; design-tokens v0.2.1 closes it, and the P1-A0 checklist cites the
resolution rather than the P0 documents being rewritten to pretend the gap never existed.

---

# P1-A0 implementation — added 2026-09-29

The authorized slice landed as two commits: `governance: authorize bounded pre-G1 P1-A0` (this section's
predecessor) and `P1-A0: add real artifact intake and Analyze summary`. Evidence:
`P1_A0_VALIDATION/`. Decisions taken while implementing, all inside the authorized scope:

- **The summary DTO names its source instead of a fixture.** `AnalysisSummaryDto.fixture` became
  `source` (`"fixture"` | `"artifact"`) because a user-chosen file has no fixture key to carry. This is
  a field rename on an internal, `p0-internal` contract, regenerated through ts-rs rather than
  hand-edited; the portable schema and its strictness rules were not touched.
- **A selection is a session fact, never a persisted one.** `SelectionId` is `sel-<pid>-<n>`, held in a
  `Mutex<SelectionStore>`, absent from the snapshot identity and from the database, and issued without
  a randomness dependency. The alternative — storing selections — would have needed a migration, and
  §12 authorizes none.
- **User-chosen analyses belong to `local-desktop`, not to the P0 demo identity.** The fixture path
  keeps `p0-desktop` so the committed P0 evidence stays reproducible; the new path writes
  `local-desktop` / `Local analyses`. Proven in the shipping binary against an empty database.
- **Two findings were reported rather than fixed, because fixing them needs authority this slice does
  not have.** *(Both were closed the same day by the correctness-closure round recorded in the next
  section; the wording below is what was decided then and is kept as history.)* (1) Snapshot identity
  comes from the artifact bytes only, so re-analyzing the same bytes with a MAP dedupes onto the
  existing build and the strengthened evidence never reaches storage.
  Closing it changes snapshot semantics or the schema. (2) `evidence_summary.from_map` is a locator
  kind inherited from the frozen P0 goldens, not a report that a MAP file was supplied; it is now
  characterized by a test instead of being redefined.
- **The last-good rule is a rendering rule, not a state merge.** A failed analysis shows its own error
  and leaves the previous successful report visible with a sentence naming the file it came from. A
  failure never becomes a snapshot, and the surviving report is never relabelled as the new candidate.
- **One CSS defect of the round's own making was fixed with tokens only.** `text.disabled` and
  `accent.disabled` are the same value, so disabled buttons lost their labels; both rules now use
  `text.secondary` on `bg.subtle` (2.32:1 → 5.22:1). No token was added, so §21's STOP condition was
  never reached.

# P1-A0 evidence identity and persistence closure — added 2026-09-29

Authorization: *FirmwareSight — P1-A0 Evidence Identity & Persistence Correctness Closure, Execution
Prompt v1.0 — Architect Reviewed*. Supplied inline, so `10_AUDIT/SOURCE_PROMPTS/README.md`'s
convention applies: the fact of the prompt is recorded, no SHA-256 is invented for bytes this
repository never received. It closes the two findings above inside P1-A0; it authorizes no new
capability, and it is not P1-A1.

- **No new ADR, and the decision policy is the reason.** `00_GOVERNANCE/03_DECISION_POLICY.md` lists
  `Bug 修复` under 不需要 ADR, and prompt §5 establishes that the identity rule already exists rather
  than being introduced here: `SnapshotId::compose` already takes an optional MAP hash, `seal()`
  already looks for an artifact of kind `Map`, `ArtifactKind::Map` and `ParserId::gnu_ld_map()` already
  exist, and `04_TECH/15_STORAGE_DATABASE_BASELINE.md` already says `Build 1─N Artifact`. The defect
  was that `pipeline::analyze` never sealed the MAP, so the existing rule was bypassed. ADR-0025's
  revisit trigger ("a slice that touches storage semantics needs its own ADR") was checked against the
  implementation before proceeding and does not bite: no storage semantics changed, no schema changed,
  and no new domain type or field appeared. Prompt §20's STOP condition - a truly new semantic being
  required - was never reached, so no migration `0003` exists and `SCHEMA_VERSION` is still 2.
- **A supplied MAP is now the build's second artifact.** `artifacts[0]` stays the ELF, `artifacts[1]`
  is the MAP with its own SHA-256, byte size and `map-adapter/gnu_ld` parser id, and with architecture,
  bitness, endianness, entry point and build-id left `Unknown` - a linker MAP cannot carry those, and
  copying the ELF's values would attribute facts to a file that does not hold them. `NORMALIZATION_VERSION`
  is deliberately **not** bumped: the id formula for an ELF-only run is byte-for-byte what it was, so
  old ELF-only ids keep their meaning.
- **Evidence provenance is now derived from the accounting basis, not written beside it.** One function
  returns the rule, source type, locator and evidence class together for each `MemoryEvidenceBasis`, so
  the four cannot disagree. `MapRegionAndElfLoad` keeps `map` + `map:load-address`;
  `ElfAddressAndFlags` becomes `elf.program-header` + `+ elf:sh_flags`; `RegionConfigAndElfLoad` becomes
  `memory-region-config`; `SectionNameHeuristic` is `Derived` with `Low` confidence rather than the
  hardcoded `Observed` it used to claim; `Insufficient` claims no source beyond the rule. The dual
  charge on an ELF-only run comes from section role and sizes, so `sh_flags` is the re-checkable locator;
  the alternative `+ elf:load-address` would have been a second false claim, because the ELF-only
  fixture has no load address at all (`unknown_load_address_stays_unknown_rather_than_defaulting_to_zero`).
- **`EvidenceSummaryDto.from_map` is removed, not replaced.** It was desktop-only IPC, rendered nowhere,
  and redundant against `capabilities.map`, `memory.layoutSource` and `memory.weakestEvidenceBasis`. The
  test that characterized the old behavior was deleted with the field it characterized, and the generated
  TypeScript was regenerated by ts-rs rather than hand-edited.
- **`BuildSummary` names the primary artifact row instead of accepting any row.** The join was
  `artifacts ON build_id` with `query_row`, which is a single-row bet on SQLite's visit order once a
  build has two artifacts. It now selects the build's index-0 row, and the row id is built by one
  helper shared with the writer so reader and writer cannot drift. A storage test reproduces the hazard
  by moving the `#0` row to the highest rowid in a scratch database, which is what distinguishes a
  passing-by-accident test from a passing one.
- **Goldens were corrected in the committed style, not reformatted.** `scripts/update_goldens.py
  --confirm` emits serde declaration order while the committed CLI goldens are key-sorted, so running it
  rewrites ~350 lines per file. That is unrelated drift, which prompt §14 forbids, so the two affected
  files were restored to HEAD and given only the four reviewed leaf changes (4 added / 4 removed lines
  per file), then verified against the real binary by `cargo test -p fwsight --test golden` and
  `drift/goldens unchanged`. The script-versus-committed-goldens mismatch is reported as an open
  inconsistency, not silently repaired here.
- **What this round is not.** No P1-A1, no Sections/Symbols table, no Evidence Inspector, no Compare,
  Gate, Bundle or History surface, no new adapters, no new navigation, no portable schema change, and
  no claim that V0 has any participant evidence. `baseline_version` stays 0.6.0 and G1 stays NOT
  CLAIMED.

# V0 Batch A external validation activated — added 2026-09-29

Authorization: *FirmwareSight — V0 Batch A External Validation Activation & Interim Review, Execution
Prompt v1.0 — Architect Reviewed*. Supplied inline, so the registry convention applies: the fact of the
prompt is recorded in `10_AUDIT/SOURCE_PROMPTS/README.md`, no SHA-256 is invented for bytes this
repository never received. It is a **research / evidence execution prompt**, not a coding prompt, and the
architect wrote into it that no product code may be produced under it.

- **The active pointer moved, no verdict did.** `active_task` goes from `P1A0_REAL_ARTIFACT_INTAKE` to
  `V0_BATCH_A_EXTERNAL_VALIDATION`, because ADR-0025 makes `V0 Batch A >= 4` eligible external sessions
  plus an interim architect review the precondition for authorizing any further pre-G1 slice. P0 stays
  `PASS` and frozen at `0.6.0`; P1-A0 and its correctness closure stay complete and are not re-opened;
  P1 is still `NOT PASS / NOT CLOSED`; P1-A1, P2, P3 and P4 remain unauthorized; `G1` stays NOT CLAIMED.
- **The instrument for Formal V0 stays the frozen clickable prototype**, `V0_VALIDATION/prototype/` at
  version `v0.1.0`, with the exact T1-T10 wording of `V0_VALIDATION/protocol/TASK_SCRIPT.md`. The P1-A0
  production desktop is **not** substituted for it, for a comparability reason rather than a preference:
  formal V0 tests comprehension of Analyze, Compare, Gate, Bundle and History, while P1-A0 implements
  only real intake plus the Analyze summary. Showing the desktop later is permitted only as a
  `NON_FORMAL_APPENDIX` that enters no numerator, no denominator, no Formal N and no V0 claim. Default:
  not shown.
- **No participant evidence existed, so none was written.** Zero `PA-00X.md` files, five screening rows
  still `NOT_RECRUITED` with `eligible = PENDING` and `real_external_human` blank, five scheduling rows
  still `NOT_SCHEDULED`, `external_participants_completed` still `0`. Synthetic participants, AI personas
  and fabricated quotes, outcomes, timings, willingness-to-pay or counts are prohibited by the prompt and
  were not used. This round's legitimate outcome is the activation itself, ending at
  `BATCH A ACTIVE — WAITING FOR REAL PARTICIPANTS`.
- **Batch A is not V0 PASS.** Four to five eligible sessions is a research milestone; formal V0 needs a
  minimum `N = 8`, and no completion ratio is reported below it. The interim review's disposition is a
  research finding returned to the architect, not an engineering authorization.
- **Stale `v0.5.2` consequence wording corrected in place, not erased.**
  `V0_VALIDATION/batch_a/V0.5.2_RELEASE_BLOCK.md` and `BATCH_A_STATUS.md` now carry dated notes saying
  `v0.5.2` was the pre-P0 planned Batch A evidence-patch version, is superseded by the live `v0.6.0`
  baseline, is not generated by completing Batch A, and that any post-Batch-A version promotion is an
  architect decision. Neither file was renamed and no `V0` document was mass-bumped: their `0.5.1`
  metadata is historical and stays. The project version is never decremented to fit a research document.
- **One real provenance gap was found while reading authority, and fixed.** `.ai/ACTIVE_TASK.md` asserted
  that `10_AUDIT/SOURCE_PROMPTS/README.md` records the inline P1-A0 prompts "without a SHA-256 rather
  than inventing one". It did not: the register stopped at P0 Final Promotion, and no P1-A0 prompt was
  ever entered. Three sections were added — the intake prompt with its design-contract addendum, the
  correctness closure, and this activation — each stating that no source file exists to hash. That is a
  sixth file beyond prompt §5's five-document list, taken because the alternative was to leave a false
  pointer about where the authorization for completed work lives.
- **Open owner-side input, deliberately not filled in.** Real participants are the only thing that can
  move `0 / 8`. Separately, `recruitment_ready/BATCH_A_RESEARCH_PRICE_ANCHORS.md` records that **no
  concrete research price anchor has been decided** and that the Product Lead may define two or three
  exploratory ones; this round invented none, so §19's commercial recording has anchors to capture only
  after that decision exists.

> Superseded the same day, on sequencing only: the entry below
> ("Open-source MVP-first delivery and P1 Analyze") and `ADR-0026` ended V0's role as a gate, so
> "the precondition for anything further" and "nothing else is authorized from this file" no longer
> describe the project. The rest of this entry stands as history: the activation did happen, the pack was
> verified, and no participant evidence was written because none existed.

# Open-source MVP-first delivery and P1 Analyze — added 2026-09-29

Authorization: *FirmwareSight — Open-Source MVP-First Governance Reset + P1 Analyze Completion, Execution
Prompt v1.0 — Architect Reviewed*, with the *P1 Analyze Acceptance Closure Addendum v1.0* attached to the
same round. Both were supplied inline, so `10_AUDIT/SOURCE_PROMPTS/README.md` records them without a
SHA-256 rather than inventing one. The architect decided the direction; the coding side proposed none of it.

- **`ADR-0026-open-source-mvp-first-delivery.md` is the record**, and it is the only place the reasoning
  lives. Summary of what it settles: MVP proceeds P1 → P2 → P3 → P4 → G2 on engineering grounds;
  **`G1 = P0 PASS`** for this delivery, which is a re-definition of the gate's basis and not merely a
  re-satisfaction of the old one; V0 becomes `NON_BLOCKING_USER_FEEDBACK_TRACK`; pricing,
  willingness-to-pay, buyer path and pilot signals leave every MVP gate.
- **What ADR-0026 did not touch is the part that matters for future rounds.** It supersedes sequencing
  only. The evidence classes, the `UNKNOWN 不等于 PASS` rule, the Rust-side dialog with no generic
  filesystem/shell/network capability, the no-host-path rule in IPC and UI, determinism, the portable
  bundle rules and `AGENTS.md` 2 / 7 / 11 all stand unchanged, and the ADR says so in its own Status
  section. "Governance got easier" is not an available reading.
- **The superseded sentences stay where they are.** `ADR-0020:31` and ADR-0025's Decision 4 and 6 keep
  their text with dated notes appended, because `00_GOVERNANCE/03_DECISION_POLICY.md` forbids editing an
  Accepted ADR's conclusion and because P1-A0 was implemented under exactly those terms - rewriting them
  would delete the authorization context of evidence already on disk.
- **G1's formula appeared in 20 places across 16 files.** Current-authority documents were updated; the P0
  promotion pack, the P1-A0 pack, `CHANGELOG.md` and the prompt register were left as written and are read
  as history. Three delivery documents outside the prompt's own file list (`00_ROADMAP.md`,
  `01_MVP_EXIT_CRITERIA.md`, `05_MVP_TO_PRODUCT_DEVELOPMENT_LIFECYCLE.md`) carried the old formula in
  identical mapping blocks and were corrected too, because leaving them would have made `BASELINE.yaml`
  contradict its own delivery docs.
- **The withdrawn prompt is recorded as withdrawn, not as absent.**
  `FirmwareSight_V0_Batch_A_Price_Anchor_Authorization_Participant_Acquisition_Pack_EXECUTION_PROMPT_v1.0`
  never had a registry entry and was never executed. Rather than leave a silence that could later be read
  as an omission, the register now carries a `WITHDRAWN_BY_ARCHITECT` entry stating no price anchors were
  written, no acquisition pack was produced, and the V0 sample stayed `0 / 8`. The same-day Batch A
  activation entry stays as written: it did happen, and its evidence state is accurate.
- **`V0`'s zero is unchanged and remains honest.** Demoting V0 from gate to track removes its power to
  block, not the requirement that any future session record describe a real person. The `v0.1.0`
  prototype, `protocol/TASK_SCRIPT.md`, `sessions/TEMPLATE.md` and both registers stay frozen in place so a
  later feedback round is still comparable. No V0 file was deleted.
- **P1's acceptance list is borrowed, not invented.** The addendum corrected a scope reading in the
  takeover report: `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` **US-001** already requires sortable and
  filterable symbols, artifact hash, an unsupported-file reason, no crash without debug info, and a
  **bytes / KiB numeric unit switch**. Those are therefore P1 exit criteria, and P1 may not be marked
  `PASS` while any of them is open. The units switch is presentation-only: values stay integer bytes in
  Core, SQLite and IPC, `1 KiB = 1024 bytes`, addresses, offsets, hashes, counts, ordinals and snapshot ids
  never convert, `Unknown` never renders as zero, and the choice lives in UI state with no settings page
  and no persistence.
- **Debug info gets no new module.** `Analyze` already renders a `Debug info` row from
  `capabilities.debugInfo`; the requirement is that it stays visible, renders from the Rust DTO, and
  survives absent debug info without crashing. If that field ever proves unable to express the states Core
  actually distinguishes, the mismatch is reported before any new semantic is invented.
- **Boundaries carried into the implementation, stated up front so a reviewer can check them:** no schema
  migration (`SCHEMA_VERSION` stays 2; `0003` would need a measured reason and a stop-and-report first), no
  new dependency, no new design token, default page 100 with a hard server-side maximum of 500, three
  use-case query commands and no generic table or SQL access, details bound to the last-good
  `snapshotId`, and a symbol's storage ordinal treated as a row position rather than identity.

# P2 Compare — opened 2026-09-29

Authorization: *FirmwareSight — P2 Compare MVP Implementation, Execution Prompt v1.0 — Architect
Reviewed*. Supplied inline, so `10_AUDIT/SOURCE_PROMPTS/README.md` records the fact of the prompt without
a SHA-256 rather than inventing one for bytes this repository never received. No ADR is required and none
was written: `ADR-0026` already made P1 → P2 → P3 → P4 a matter of engineering sequencing, and this stage
moves no technology baseline, persistence semantics, capability surface or evidence class.

Decisions taken at activation, before any product source changed:

- **The pointer moved; nothing was concluded.** `active_task` reads `P2_COMPARE`, `validation.p2_status`
  reads `IN_PROGRESS`, and `validation.next_authorizable_tracks` now names `P3_gate_after_P2_compare_closes`.
  A stage that has started is not a stage that has passed, and the `p2_execution` block says so in its own
  fields. Baseline stays `0.6.0`; no `v0.7.0`.
- **The three successor runs of the P1 round are recorded here rather than left owed.** Runs #14
  `36558893544` on `1e5880a`, #15 `36559834259` on `653e313` and #16 `36576568426` on `7a13660` are each
  `success`, 7 of 7 jobs, read with `gh api repos/2023violet/FirmwareSight/actions/runs/<id>` and
  `gh run view`. They were pushed separately, so unlike the four-commit P1 push each has its own run — and
  none of the three ids appeared in any tracked file until now. Run #16 on `7a13660` is this round's
  verified start fact.
- **One stale pointer found at takeover is corrected, not rewritten.** `README.md`'s read-order list
  still described `.ai/ACTIVE_TASK.md` as `P1A0_REAL_ARTIFACT_INTAKE` while `BASELINE.yaml` read `NONE`.
  It now reads `P2_COMPARE`, which is the current truth; the dated governance records keep theirs.
- **The start counts are measured, not inherited.** `cargo test --workspace` on `7a13660` before the first
  write: **184 passed / 0 failed / 0 ignored**. `corepack pnpm test`: **58 passed**. `cargo-deny 0.20.2`
  is installed, so the `deny` step executes rather than taking its SKIPPED-as-pass branch.
  `arm-none-eabi-gcc 14.3.Rel1` is on PATH, which is needed only to build the new P2 fixture pair.
- **Three tests that assert the absence of what P2 ships will change by name.** `apps/cli/src/main.rs`
  `unregistered_future_commands_are_not_accepted` drops `diff` and keeps `gate` / `release` / `watch` /
  `doctor`; the exit-code surface test in the same file adds `6` while keeping `4` and `5` unreachable;
  `apps/desktop/ui/src/intake.test.tsx`'s "no navigation, no Compare text" guard becomes the Analyze +
  Compare navigation assertion prompt §53 asks for. Each is a scope change the prompt orders, and each
  stays a test — none is deleted to reach green.
- **Two absences in the existing code are named instead of quietly filled.** No JSON-Schema validator
  dependency is admissible (prompt §57), and the three existing `schemas/*.schema.json` are referenced by
  no code today, so the diff contract test implements the schema subset actually used — `type`,
  `required`, `properties`, `additionalProperties`, `const`, `enum`, `items` — over `serde_json`.
  Likewise no HTML generator or escaping utility exists anywhere in `crates/` or `apps/`; the self-contained
  HTML report is new std-only code in `firmwaresight-report`, not a template library.
- **`Diff.identity_changes` is not silently dropped.** `04_TECH/02_DOMAIN_MODEL.md:64-71` lists identity
  changes as part of `Diff`, while prompt §31's DTO does not, and git/build-identity facts are not
  persisted per build today. P2 compares artifact identity (name, SHA-256, byte size) inside base/target
  and reports deeper identity/provenance comparison as unavailable rather than inventing storage for it.

Scope guard: this stage stops at Compare. P3 Gate, P4 Bundle, History, Project Settings, installer,
signing, updater, cloud, accounts, telemetry, AI, pricing and commercial work remain outside it, and no
prompt for any of them exists.

# P2 Compare — closed 2026-09-29

Verdict: **`PASS / COMPLETE`**, decided item by item against the frozen US-002 acceptance list in
`P2_VALIDATION/P2_COMPARE_EXIT_CHECKLIST.md`, not by a summary. `active_task` returns to `NONE`,
`validation.p2_status` returns to `PASS_COMPLETE`, `p2_execution` now carries measured results instead of
promises, and baseline stays **`0.6.0`** — this closure creates no `v0.7.0` and claims no G2.

Decisions taken at closure, each with the thing that forced it:

- **The layout label is Core's vocabulary, not a Rust enum name.** One portable document was rendering
  `MapMemoryConfiguration` from the CLI and `map` from the desktop for the same pair. The desktop was
  correct because it reads the persisted column back, so `LayoutSource::as_label()` became the single
  definition and the diff, the storage write and the report DTO all route through it - which also removed
  two private copies of the same mapping that agreed only by care. Two P2 goldens moved by four lines.
  They moved because the code was wrong, and the golden is not the authority that decides what correct is.
- **A public schema is now a compatibility promise.** `schemas/diff.schema.json`
  (`urn:firmwaresight:schema:diff:1`) became a shipped output format under prompt §59, which authorized it
  without an ADR because versioned machine-readable output is already policy and Diff is already a defined
  capability. That cuts both ways and is written down: no silent reinterpretation of any field, and a
  breaking semantic change needs a future major version. Whoever authors P3's Gate output and P4's Bundle
  inherits this constraint, not just their own schema.
- **Object / module attribution stays `Unavailable`.** PRD P0-3 makes it conditional on sufficient
  evidence, and nothing in an ELF's symbol values attributes bytes back to a source object. The stage
  reports the gap with its real reason and ships no Object tab, rather than inventing a derivation to fill
  a column.
- **A gate blind spot is closed by adding a step, not by relabeling a result.** `.gitignore`'s
  `**/target/` matched `fixtures/elf/p2-diff/target/`, so half of the P2 fixture pair never entered the
  repository while `fixtures/manifest.json` recorded all twelve paths. Every local run passed on the bytes
  the generator had left on disk; the pushed commit's CI run did not. The fix is three parts - commit the
  six files, un-ignore that one path with its reason in the comment, and give the drift group a
  `fixtures tracked` step that compares the manifest against `git ls-files`. It is the only step in the
  gate that reads the index. Changing the gate is deliberately the smaller act here; leaving a red remote
  and writing "green" would not be.
- **The red remote is reported, not smoothed over.** `origin/main` is at `c7fc2a3`, whose Run #17
  `36596452341` is `completed/failure` (3 of 7 jobs). The fix at `cfee1e5` is verified from a clean
  `git archive` checkout, but nothing after `c7fc2a3` was pushed, so P2 closes as `LOCAL PASS` with a red
  branch behind it. Pushing is the owner's act and no run number is claimed for a tree that has never run.
- **One smoke step is `NOT VERIFIED` and stays that way.** Desktop step 27, the same-pair lock in the
  shipped window, needs a native `<select>` popup that this environment cannot drive or capture. The
  behaviour has tests; it does not have an observation, so the report writes PARTIAL and the completion
  report lists it as an open item instead of folding it into "30/30".
- **No arithmetic was performed on a fixture to make a claim true.** The four window defects and the label
  defect were fixed in product and test code; the `--confirm` golden regeneration moved exactly the four
  lines named above and the P0 goldens were untouched. `cargo test --workspace` went 184 → 345 and UI 58 →
  99, all green, and no existing assertion was weakened to get there - the three tests that asserted P2's
  own absence were rewritten by name, as the activation entry recorded.

# P2 Compare — remote verification, same day, after the closure entry above

The closure entry says P2 closes with a red remote and no run covering its final tree. That was true when
it was written and is not rewritten here; this entry records what the owner's push measured afterwards.

- **The run is recorded by a successor document, never by the commit that triggered it.** That rule held
  through P0 and both P1 rounds and it holds here: `4a77ea1` carries the integrity artifacts and says
  nothing about the run its own push would produce. This document is the successor that reports it.
- **Mid-round Run #17 `36596452341` on `c7fc2a3` stands as `failure`** - 3 of 7 jobs red
  (`Rust (windows-latest)`, `Rust (ubuntu-latest)`, `macOS Core Smoke`) on
  `committed_fixtures_match_their_recorded_hashes`. It is history, not a defect to be re-labeled: it is
  precisely the measurement that made defect E visible, since no local run could see a fixture git had
  never carried.
- **Head Run #18 `36648718199` on `4a77ea1` concluded `success`, 7 of 7 jobs** (read with
  `gh run view 36648718199 --repo 2023violet/FirmwareSight`): `Generated output drift`, `Dependency policy`,
  `Rust (windows-latest)`, `Rust (ubuntu-latest)`, `Desktop UI (windows-latest)`,
  `Desktop UI (ubuntu-latest)`, `macOS Core Smoke`. The three that were red are green, so the fixture pair
  now reproduces on a machine that never ran the generator - which is the claim the P2 pack could only make
  locally before.
- **The wording of the verdict does not change.** P2's gate numbers were measured before any push, so the
  round still closes as `LOCAL PASS`, and #18 is the successor verification of the tree carrying them. A
  green run after the fact does not convert a `LOCAL PASS` into `CI PASS` for the measurements that
  preceded it, and the completion report is unchanged for that reason.
- **This document's own push starts another run.** Its number belongs to a later record, not to this one,
  and nothing here claims a result for a tree that has not yet run.

# P3 Release Gate — opened 2026-09-29

Authorization: *FirmwareSight — P3 Release Gate MVP Implementation, Execution Prompt v1.1 — Architect
Reviewed*. Supplied inline, so `10_AUDIT/SOURCE_PROMPTS/README.md` records the fact of the prompt without
a SHA-256 rather than inventing one for bytes this repository never received. Unlike P1 and P2, this stage
requires an ADR and has one: `ADR-0027`, which is prompt §6 transcribed into
`09_ADR/ADR-0027-project-policy-and-provenance-adapter.md` with the dependency table measured from the
lock file. The architect decided the crate boundary; the coding side measured versions, licenses and MSRV
and installed nothing the requirement did not need.

Decisions taken at activation, before any product source changed:

- **The pointer moved; nothing was concluded.** `active_task` reads `P3_RELEASE_GATE`,
  `validation.p3_status` reads `IN_PROGRESS`, and `next_authorizable_tracks` now names
  `P4_bundle_after_P3_gate_closes`. Baseline stays `0.6.0`; no `v0.7.0`.
- **Run #19 on the closure HEAD is the start fact, Run #18 stays the implementation proof.** Run #19
  `36665007523` on `32b23aa` and Run #18 `36648718199` on `4a77ea1` are each `completed` / `success`,
  7 of 7 jobs, read with `gh run view <id> --repo 2023violet/FirmwareSight`. #18 proves the tree the
  architect sealed as P2 FINAL PASS; #19 proves the latest successor HEAD this round starts from. Run #17
  remains in history as a real failure.
- **The §0 delta check found nothing to reconcile.** `origin/main` was exactly `32b23aa` at fetch time,
  the worktree was clean, and no owner-authored or third-party commit had landed after the P2 closure
  record, so P3 starts on the sealed HEAD rather than on a newer one.
- **P3 is the first stage since P0 to move three structural boundaries at once**, and each is named here
  so a reviewer can find it without reading a diff: a fifth first-party crate (`firmwaresight-project`),
  a third migration (`0003_gate_history.sql`, `SCHEMA_VERSION` 2 → 3, additive), and the first direct
  dependencies outside the frozen set (`toml`, `regex`). A fourth is semantic rather than structural:
  `4` REVIEW and `5` BLOCK become reachable CLI exit codes, where every previous stage ran with them
  forbidden by an assertion.
- **Where the new logic lives is the ADR's whole point.** Core owns Gate rule evaluation, five-state
  semantics, aggregate precedence and rule identity. The new crate owns config loading, canonical policy
  hashing, project-relative evidence reads, the read-only `git` process and the deterministic run
  fingerprint. Neither CLI nor Desktop gets its own copy, and none of it goes into Core, Artifact or
  Storage.
- **Dependency admission was measured, not assumed.** `toml 1.1.6+spec-1.1.0` and `regex 1.13.1` are
  already in `Cargo.lock` (read with `cargo tree -i toml@1.1.6` — `tauri-utils 2.10.0`,
  `cargo_toml 1.0.1`, `embed-resource 3.0.11`), so the crate reuses resolved versions instead of
  widening the graph. `sha2 0.11.0`, `thiserror 2.0.21` and `serde 1.0.229` are reused at their locked
  versions. `tracing 0.1.44` is authorized but is not installed unless a real diagnostic needs it.
  `gix`, `anyhow` and `rayon` stay in `deny.toml`'s ban list.
- **`accepted-reviews` v1 gets an optional field, not a breaking one.** ADR-0023 requires
  `original_state` and the shipped schema does not carry it; prompt §35 settles it by adding an OPTIONAL
  `original_state` enum `["REVIEW"]` while keeping the required list and `additionalProperties: false`
  intact, so a v1 document written before P3 still validates. `gate-results` stays v1 with no state
  vocabulary change; `policy_sha256`, `baseline_snapshot_id` and `project_config_schema_version` go
  through `extensions`.
- **`run_id` is content, not a clock.** `gate-<SHA-256>` over snapshot ids, canonical policy hash,
  normalized Git facts and Release Notes path/presence/digest. Absolute project root, `imported_at`,
  wall clock, pid and UI state are excluded by construction, and the exclusion is tested rather than
  promised.
- **The license gap is reported, not fixed.** The root `Cargo.toml`'s `[workspace.package]` table reads
  `license = "Proprietary"` and there
  is no root `LICENSE` file; `AGENTS.md` 9 puts a license change in front of a human and the prompt
  declines to choose one. Recorded as `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`, visible
  in the completion report, and not a P3 blocker.
- **`scripts/check.py`'s `CORE_PACKAGES` must gain `firmwaresight-project`.** An omitted workspace member
  is silently unexamined by the core-smoke group rather than loudly failed, which is the same class of
  blind spot that let half of P2's fixture pair stay out of the repository.
- **The Gate smoke subject is a temporary Git project built from the P2 fixture pair.** Prompt §61
  forbids using the FirmwareSight source repository as a Gate subject: this repo is dirty or tagged by
  the owner's acts, not by the release being checked, so its Git facts would be about the tool rather
  than about the firmware.
- **Five existing assertions will change by name, not by deletion**: `apps/cli/src/main.rs`
  `unregistered_future_commands_are_still_not_accepted` (drops `gate`, keeps `release` / `watch` /
  `doctor`) and `exit_code_one_is_never_produced` (gains `4` and `5`, keeps `1` impossible),
  `apps/desktop/ui/src/intake.test.tsx`'s third-nav-entry and no-Gate-text guards (prompt §43/§60
  replace them with the Analyze + Compare + Release assertion), and the four storage tests pinning
  `SCHEMA_VERSION` 2.

Scope guard: this stage stops at the Gate. P4 Bundle, `release prepare`, bundle chooser, manifest
generation, bundle checksums, History page, Project Wizard, installer, signing, updater, SBOM, cloud,
accounts, telemetry, AI, pricing and commercial work remain outside it, and no prompt for any of them
exists.

# P3 Release Gate — closed 2026-09-30

Verdict: **`PASS / COMPLETE`**, decided item by item against the frozen US-003 acceptance list and the
PRD P0-5 ten checks in `P3_VALIDATION/P3_GATE_EXIT_CHECKLIST.md` — forty boxes, each with the command,
test or smoke step that settles it. `active_task` returns to `NONE`, `validation.p3_status` returns to
`PASS_COMPLETE`, and baseline stays **`0.6.0`**: this closure creates no `v0.7.0` and claims no G2.

Decisions taken at closure, each with the thing that forced it:

- **A locator is a pointer, never a quotation.** The first shipped-binary run at Gate died with
  `ERR-STORAGE-4006 … instr(evidence_ref, char(92)) = 0`, because Core had put the project's
  `[version] pattern` text — `\d` included — inside an evidence locator. Any project using the only
  version source the MVP defines could not persist a Gate run at all. The locator is now
  `policy:version.pattern`; the pattern itself stays where it belongs, quoted in the finding's summary
  and inside the canonical input that identifies the run. `configured paths reject an interior
  separator too`, so a Windows path cannot enter a `file:` locator either.
- **One build, one fingerprint, whichever surface read it.** The CLI and the desktop produced two
  different run ids for the same policy, builds, HEAD and workspace. The cause was that a desktop run
  hydrates its footprint from SQLite, which stored the byte totals but not the `evidence:` row they
  came from, and Core's canonical memory block also carried a `reason` string the database never
  recorded. Both were inside the hash. The pointer is now read out of `evidence` beside the totals and
  `reason` left the canonical form, and `a_stored_build_fingerprints_exactly_like_a_fresh_analysis`
  holds the two assembly paths against each other. Measured again across surfaces in the smoke:
  `gate-a64631b2…0ff152cc` from both.
- **A Gate run's identity is content, so re-running it is a dedupe.** §33's rule was implemented as
  `GateRunWrite::AlreadyStored` rather than an upsert, and the smoke observed it: a second Run Gate on
  identical inputs left SQLite at one run, ten findings and the original `created_at`. A same id
  carrying a different verdict is an `Invariant` refusal, not an update — immutability is the property
  that makes an accepted review worth reading a week later.
- **Removing a tracked file is an uncommitted change, and the Gate says so twice.** Setting the release
  notes aside produced two BLOCKs in one run (`git.clean` and `release.notes`), not one. That is the
  honest result and the report states it as such rather than presenting a clean single-finding case
  that the workspace could not actually produce.
- **The Git subject stays outside this repository.** Every Gate run in the smoke and in the tests was
  judged against a throwaway repository under `%TEMP%`, because §61 forbids using the FirmwareSight
  source tree as a Gate subject: this repo is dirty or tagged by the owner's acts, not by the release
  being checked.
- **`custom-protocol` is not optional in a smoke.** The first attempt at step 1 launched a
  `cargo build --release` binary that loaded the dev URL and showed a WebView2 connection error. The
  P2 pack already recorded this; it recurred and is recorded again, because a smoke of a dev build
  proves nothing about shipping.
- **What the window could not show is written as not shown.** Smoke step 30's prior-run re-read is
  `PARTIAL`: `get_gate_run` accepts any run id and is tested, but the Release page has no run-id input,
  so no older record was opened *on screen*. Surfacing history is P4's page and §64 forbids building it
  here. Likewise the acceptance refusals (empty actor, empty reason, non-review, second opinion) are
  command-layer behaviours covered by tests, not behaviours driven through the form.
- **A gate that comes back 13/14 is reported as 13/14, and the flake it names is a bug.** The first full
  gate of the closure round failed `frontend/test` on
  `compare.test.tsx > the change tables > keeps a size that was never recorded as Unknown with its
  reason`. The same test had failed once in twelve earlier runs with its name lost to a log filter, which
  is how a real defect gets talked of as a flake. `Compare.tsx:1303` renders `Loading symbol changes…`
  inside the very region the test waits for, so the awaited region resolves against an empty table and the
  synchronous `getByText` that follows loses the race with the second IPC call. Three such queries are now
  awaited, with every assertion unchanged — a row that genuinely never arrives still fails, just with a
  legible message. Ten consecutive runs of the file pass where one in five had failed, and the `15/15`
  recorded above is the re-run after the fix, not the earlier run restated. Nothing was weakened to reach
  green, and no `--filter` skip was used.
- **P3's code had no CI run at closure; it has one now, and it was 6 of 7.** At closure `origin/main` was
  still `32b23aa` (Run #19 green) and the P3 commits were local, so every P3 number in this pack is
  labelled as locally measured, and §67 forbade writing a future CI result into the commit that would
  trigger it. Pushing is the owner's act, and the commits were then pushed. Run `36774472141` on
  `219178af` concluded `failure`: Rust on both platforms, Desktop UI on both platforms, the macOS core
  smoke and the generated-output drift check all passed, and `Dependency policy` alone failed on
  `error[yanked]` for `yoke-derive 0.8.3`, a transitive proc-macro reached only through
  `yoke` ← `icu_*` ← `idna` ← `url`, which crates.io yanked at 2026-09-30T13:19:39Z — after this machine's
  `deny` step had gone green against an older index. `cargo update -p yoke-derive --precise 0.8.4` is the
  fix: one version, one checksum, and `cssparser-macros`'s already-satisfied `syn >=2, <4` edge
  re-resolving from `3.0.6` to `2.0.119`. No manifest changed, nothing entered or left the graph, no
  license moved, and the full gate is 15/15 on the fixed tree.
- **A closure count in this pack was a double count, and it is corrected rather than defended.** The Rust
  total was recorded as **715 in 42 suites**. One `cargo test --workspace` is **556 in 28 executable
  suites**: the 715 summed a gate log in which `check.py` runs the workspace once for tests and then runs
  `cargo test -p firmwaresight-desktop` again as the drift group's binding-regeneration check, so the
  desktop crate's 159 tests were counted twice. The per-suite counts are identical before and after this
  round's dependency fix, so no test appeared, vanished or was weakened — only the arithmetic changed.
  `Core gate 55` was the same class of error; the counted `domain::gate` list is 32. P3 therefore took Rust
  from 345 to 556, and the UI figure was never double-counted at 135.
- **CI went green on the dependency fix, then red again on a UI test that only Windows CI could lose.**
  Run `36779321479` on `2d1bcea` concluded `success` with **7 of 7 jobs**, Dependency policy included,
  which is the yank fix verified on a fresh index. Run `36779715108` on the next head, `893a635`,
  concluded `failure` with 6 of 7: `Desktop UI (windows-latest)` → `frontend/test`, `AssertionError:
  expected 2 to be +0` at `compare.test.tsx:731`, while `Desktop UI (ubuntu-latest)` passed the same file
  on the same commit and eight local runs had passed it. The KiB test sampled `querySectionChanges` call
  counts right after `runCompare()`, but `runCompare()` awaits only the `Build comparison` region — the
  two change tables fetch their own pages afterwards (`Compare.tsx:225`, `Compare.tsx:1001`) — so on a
  slower runner the page's own loading was charged to the radio click. Same class as defect I, in the
  same file, and this time the local gate could not see it.
- **That fix keeps the assertion and adds a guard, and it was proved by mutation rather than by green.**
  The sample now happens after both tables have resolved, the way the P1 test has done it since
  `details.test.tsx:352`, and `expect(changeCalls).toBeGreaterThan(0)` proves the sample is really after
  the fetch instead of vacuously early. The two delta assertions were not touched. To check the test
  still bites, a real `querySectionChanges` call was added to the unit toggle: the file failed at that
  line with `expected 4 to be 3`. `Compare.tsx` was then restored and verified byte-identical to HEAD
  with `git diff --exit-code`, and eight consecutive runs of the file pass. Run `36783457030` on the
  `02e8a81` head then concluded `completed / success` with **7 of 7 jobs**, `Desktop UI
  (windows-latest)` included, and its documentation-only successor `f66a93d` came back the same on Run
  `36784382005` — which is where this pack stops naming runs, since recording a doc-only successor's own
  result needs another successor. One green Windows run is not
  proof that a timing race is dead; eight local runs had already passed the file that CI lost. What makes
  the fix credible is that the sample is now causally after the fetch, guarded so it cannot pass by
  being vacuously early.
- **The license gap did not close with the stage.** `Cargo.toml` still reads `license = "Proprietary"`,
  there is still no root `LICENSE`, no workspace license metadata was touched and no license text was
  added. `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` stands, `AGENTS.md` 9 keeps a license
  change in front of a human, and the gap stays visible in the completion report.

Scope guard held: no `release prepare`, no bundle, no manifest generation, no bundle checksums, no
History page, no pricing, cloud, accounts, telemetry, updater, signing, SBOM, CVE, OTA, flashing or AI
judge. `release`, `watch` and `doctor` remain unregistered CLI commands, and `intake.test.tsx` asserts
that Bundle, History, Settings, SBOM, Pricing and Cloud appear nowhere. *(That last sentence described the
tree as of P3's closure; the P4 round below changes those two assertions by name, because §48 puts a Bundle
section under Release while §58 keeps the navigation rail at three pages.)*

# P4 Release Bundle — opened 2026-09-30

Authorization: *FirmwareSight — P4 Release Bundle MVP Implementation, Execution Prompt v1.0 — Architect
Reviewed*. Unlike P1, P2 and P3, this prompt reached the execution environment **as a file**, so it is
archived in `10_AUDIT/SOURCE_PROMPTS/` and registered with its recomputed SHA-256
`1baaec9204a1d2aa5aa53bd735b34d376ee56db7557a04d6abb79265c30840c5`; the file was already LF-only, so no text
conversion moved a byte between the delivered copy and the archived one. No ADR is required and none was
written: P4 moves no technology baseline and adds no crate, and `release_records` is a table
`04_TECH/15_STORAGE_DATABASE_BASELINE.md` §3 has listed since v0.5 — `0004` makes it real, additively, the
way `0003` made Gate history real.

Decisions taken at activation, before any product source changed:

- **The pointer moved; nothing was concluded.** `active_task` reads `P4_RELEASE_BUNDLE`,
  `validation.p4_status` reads `IN_PROGRESS`, `g2_status` reads `NOT_REACHED`, and
  `next_authorizable_tracks` now names the G2 engineering closure audit rather than P4 itself. Baseline
  stays `0.6.0`; no `v0.7.0`. P4's §5 allows exactly one G2 statement —
  `READY_FOR_ENGINEERING_GATE_REVIEW` — and forbids `PASS`, because a whole-MVP closure audit is not
  something an implementation round can grade.
- **The start fact is one commit ahead of the prompt's anchor, and that was inspected before writing.**
  §0/§2 name `ba5e59e` (Run #25 `36785425648`, success, 7 of 7). `origin/main` was at `323afad` when the
  round began: `git diff --name-only ba5e59e 323afad` is eleven documentation, governance and integrity
  files plus the two newly tracked baseline-artifact scripts, with no product source, fixture, schema,
  migration or configuration file, and Run `36810689645` on `323afad` is success at 7 of 7. Same tree for
  engineering purposes, so the newer green HEAD is recorded as the start — the identical judgement P3 made
  about Run #19 on its own successor HEAD.
- **Start counts were measured, not inherited.** One `cargo test --workspace`: **556 passed / 0 failed /
  0 ignored**. `corepack pnpm test`: **135 passed in 6 files**. Matching §0's numbers is the check that the
  retired `715` double-count was not brought back.
- **A bundle is a release verdict output, so it re-runs the Gate rather than trusting a stored one.** §9
  settles the staleness problem: a stored `PASS` plus a dirty workspace, changed config or changed Release
  Notes must not produce a bundle. Preparation re-observes the project through the P3 adapter, recomputes
  the deterministic run id, and refuses with `GATE_CONTEXT_CHANGED` unless that id equals the selected
  stored run. Gate semantics are reused, never re-derived (§9's own instruction), and disposition must be
  `PASS` after the acceptance-aware aggregate — with `UNKNOWN` never acceptable as a Review, so an evidence
  gap cannot be packaged as a release.
- **Two crates share the work, and neither is new.** `firmwaresight-report` owns the portable *content*
  (Analysis v1 projection, manifest DTO, report HTML, SHA256SUMS text, the file-bytes IR) and stays
  filesystem-free like the rest of that crate; `firmwaresight-project`, which already reads config,
  Release Notes and Git and already owns `sha2`, owns *staging, publish, swap and verification*. This is
  the placement that keeps §65's "no sixth crate" and §35's "do not have separate CLI bundle semantics"
  both true: one engine, called by the CLI and the desktop. Core keeps release *semantics* only — id
  fingerprint inputs, ordering, and what belongs in the model — and reads, writes and hashes nothing.
- **The manifest self-reference is documented, not faked.** §20: `SHA256SUMS` carries every payload file
  except itself and the manifest; the manifest carries every bundle file except itself, `SHA256SUMS`
  included. No empty or zeroed self-hash appears anywhere, and the rule is written into
  `extensions.integrity_model` and into the report so a reader can verify without asking FirmwareSight.
- **`release-manifest:1` is not reinterpreted.** Its `files[]` items are closed
  (`additionalProperties: false`), so the §49 preview's file *role* stays in the internal `BundlePreviewDto`
  and never enters the manifest; `build.artifact_sha256` names the primary shipped artifact and the MAP is
  covered by `files[]`. Git fields stay labelled workspace provenance, never artifact build provenance (§19,
  §43) — the honest sentence is "Workspace HEAD observed by the Release Gate", not "firmware built from
  commit X".
- **`analysis:1` is a new public contract, and the internal DTO is not it.** `AnalyzeResultDto` is
  `p0-internal`; copying it into a bundle and calling it durable would have made an IPC shape a
  compatibility promise by accident. A dedicated portable projection is authored instead, contract-tested
  through the crate's own dependency-free `schema_check` subset — which reports an unimplemented keyword as
  a failure, so the new schema may use only the keywords that subset actually evaluates. No JSON-Schema
  dependency is admitted (§15, §64).
- **The portable document may be large; the IPC payload may not.** A full symbol table in `analysis.json`
  is legitimate file output, and the 500-row IPC ceiling is a boundary rule about crossing into React, not
  about writing a file. The document is rendered Rust-side and never sent whole through IPC (§14).
- **Filesystem safety is a feature requirement, not an implementation detail.** US-004's fourth item and
  §30-§32 together mean: stage into a sibling, verify staged bytes, write `SHA256SUMS`, then the manifest
  last; publish by rename; and if a directory is already there, return `CONFIRM_REPLACE_REQUIRED`, ask the
  human, and replace only a destination recognizable as a FirmwareSight bundle — backing it up first and
  rolling back if the swap fails. An arbitrary user directory is never recursively deleted, `overwrite=true`
  notwithstanding.
- **No signing claim.** §62 forbids *trusted*, *authentic*, *signed* and *tamper-proof*. A SHA-256 proves
  byte consistency against an included manifest and authenticates nobody; the vocabulary is integrity
  verification, hash verification, bundle consistency.
- **The license gap stays open on purpose.** §76 names it: `license = "Proprietary"` in the root
  `Cargo.toml`'s `[workspace.package]` table, no root `LICENSE`, `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`
  unchanged, and no license text added silently. `AGENTS.md` 9 puts that decision in front of a human, and it
  does not block technical MVP completion.
- **The release subject is never this repository.** Every bundle test and smoke uses a throwaway project
  and a throwaway Git repository, for the reason §47 and P3's §61 both give: this workspace is dirty or
  tagged by the owner's acts, not by the firmware being released, so its Git facts would describe the tool.
- **Five known assertion changes, taken by name.** Three storage tests pin `SCHEMA_VERSION` as the literal
  `3` (`gate_history.rs:232`, `compare_candidates.rs:613`, `map_companion_persistence.rs:298`) and become
  `4`; `apps/cli/src/main.rs`'s unregistered-command test drops `release` while keeping `watch` and
  `doctor`; and the two whole-`document.body` "no Bundle text" guards (`intake.test.tsx:509-511`,
  `release.test.tsx:446-458`) narrow to the navigation rail, because §48 requires a Bundle section under
  Release while §58 requires the rail to stay three pages. None is deleted to reach green.

Scope guard: this stage stops at the Bundle. History page, Project Wizard, installer, code signing,
notarization, updater, SBOM, CVE, OTA, flashing, HIL, cloud, accounts, telemetry, AI, pricing and commercial
work remain outside it, as do `v0.7.0`, P5, V1, B1, RC1 and GA1, and the G2 closure audit itself.

# P4 Release Bundle — closed 2026-10-01

Verdict: **`PASS / COMPLETE`**, decided item by item against the frozen US-004 acceptance list and the
sixty-one boxes of §72 in `P4_VALIDATION/P4_BUNDLE_EXIT_CHECKLIST.md`. `active_task` returns to
`NONE`, and baseline stays **`0.6.0`**: this closure creates no `v0.7.0` and claims no G2 — the only
G2 sentence written anywhere is `READY_FOR_ENGINEERING_GATE_REVIEW`.

Decisions taken at closure, each with the thing that forced it:

- **The bundle's only clock is a recorded human act.** The independent reader first failed its own
  bundle on two checks, because a blanket "no date-shaped value" rule cannot distinguish a generation
  time from `accepted_at`, the moment a named person accepted a review, which the report is required
  to print. The rule now names the one permitted clock and checks the report *against the review
  record*, so it still catches an invented date. Fixed in `e799f2f`; the bundle was never wrong, the
  checker was.
- **A stale plan is consumed, not re-authorizable.** After `ERR-BUNDLE-6103` the UI drops the preview
  and the destination, so a second Export with restored bytes is a no-op by design rather than a
  silent retry; the error's own remedy — re-analyze or restore, then Prepare — is the only path back.
  Recorded in the smoke report as an observation, because step 48 otherwise reads as if the same plan
  should export again.
- **The release record lands after the bytes, and a same-id export writes no second record.**
  `the_release_record_lands_only_after_the_bytes_do` pins the order; the smoke observed the dedupe on
  the shipped binary — the replacement export said "The release record was not written", and the
  database's WAL still carried the first export's write time while the bundle files' creation times
  had moved. One release, one record, whichever export wrote the folder.
- **A golden bundle needs a pinned Git subject, and the pin is asserted.** The release id hashes Git
  facts, so reproducibility requires author, committer, both dates and message to be fixed; the test
  asserts the expected HEAD so a Git object-format change fails as a broken subject instead of as a
  mysteriously different bundle.
- **Shipping-binary equivalence is measured, not assumed.** §69 asks that the smoke walk the binary of
  the final tested tree; rather than rebuild and re-walk, the closure records that the last commit
  touching any shipped source path predates the build and that
  `git diff --stat <build-commit> HEAD -- '*/src/*'` is empty. Had it not been, the affected behaviour
  would have been re-walked and the report would name what was and was not.
- **The successor's own CI run stays outside the successor.** §73 allows exactly one
  documentation/governance closure commit after the implementation tree is green and requires that
  commit to reach 7 of 7; writing its run id into itself is impossible and chasing it into a third
  commit is forbidden. The run is read with `gh run list` and reported once, in the completion report.
- **The license gap stays open on purpose.** §76 again: `license = "Proprietary"` in the root
  `Cargo.toml`, no root `LICENSE`, `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION`
  unchanged. `AGENTS.md` 9 keeps that decision in front of a human, and it did not block technical MVP
  completion.

Scope guard: the closure added no product code. Four validation documents, three governance files and
two regenerated integrity artifacts are the whole commit set after `e799f2f`; the MVP line stops here
and returns to the architect for the G2 engineering closure audit.

# Two pre-G2 closures — 2026-10-01

- **P4 Governance Baseline Consistency Closure** (inline prompt). `BASELINE.yaml` still carried P4's
  running-state live fields after every other surface recorded P4 closed; `fe420d1` aligned them and
  touched nothing else. Its Run `36892307828` lost attempt 1 on a Compare test race this commit did not
  touch, and passed 7 of 7 on attempt 2 of the same SHA. A green re-run was not taken as closing that race.
- **Pre-G2 Compare UI Test Reliability Closure** (file prompt `dee5b8e4…`, archived in the G2 round).
  `055b54e` made `compare.test.tsx` wait for the pager text it asserts. The prompt required the *first*
  attempt on the repair head to be green, because the defect was non-determinism: Run `36899128645`
  attempt 1 was 7 of 7, and two more Windows UI executions agreed.

# G2 Product MVP Engineering Closure Audit — 2026-10-01

Authorization: the G2 prompt v1.0 (file, `3c6ab83e…2bac9`, archived) and its Storage Path Semantics
Clarification Addendum v1.0 (inline). Audit first; narrow repair only inside existing MVP contracts.

- **An audit that only re-reads verdicts proves nothing new, so the gate was run before anything was
  written — and it failed.** `release.test.tsx` "puts focus on the decision it just asked for" read focus
  synchronously after `findByRole`; the product focuses in a passive effect. G2-F1 was reproduced with a
  deferred-focus mutation, fixed with `waitFor`, guarded by a no-focus mutation, committed alone as
  `e35cfe7` before any evidence commit (prompt §41), and green on its first CI attempt. It is the fourth
  instance of one test shape in this suite (after P3's I and J and `055b54e`), recorded as L23.
- **The audit object moved once and only by authorization.** Everything observed in the window was
  observed on product source identical to `055b54e`; the addendum names `e35cfe7` as the audit head.
- **A checklist sentence that contradicted a frozen baseline was escalated, not reinterpreted.** The prompt's
  §13/§34 said "no host path persisted"; the smoke store's `artifacts.path` holds every source path, as
  `04_TECH/15` §7 and P1-A0 always designed. Removing it would change schema semantics, which G2 may not.
  The round stopped short of a verdict and asked; the architect's addendum adjudicated G2-F2 as expected
  local-only persistence and replaced the sentence with a boundary, which was then proved in 17 checks
  with a positive control (`G2_EVIDENCE_MATRIX.md` §7).
- **The window was driven without a screen-control connector**, through the WebView2 DevTools Protocol
  (real input events, a test-harness environment variable, no product change) and Win32 messages for the
  native pickers. Every harness misstep is written down; none is a product event.
- **Parity was measured as bytes where bytes were promised.** Diff JSON, Diff HTML and the whole bundle are
  byte-identical across CLI and desktop; the Gate's ten findings agree field by field, the page grouping
  them by state as a presentation choice.
- **A restore must restore bytes.** Scenario A's first restore used `git checkout`, and the account's
  `core.autocrlf=true` wrote CRLF back. It was caught by hashing, re-done from the LF source, and became
  limitation L22: line endings can move a release id, and the Gate fails closed when they do.
- **What stays unmeasured is written as unmeasured.** The 500 MB working set in the window and peak RSS
  are NOT_MEASURED; per the prompt's §27 they do not block, and they are not claimed.
- **One P4 sentence was found inaccurate and is reported, not edited.** `P4_BUNDLE_EXECUTION_REPORT.md` §1
  says the shells own staging; the source puts it in `firmwaresight_project::bundle`. Closed packs are
  records about the tree they measured.

Local verdict: **G2 LOCAL PASS / READY FOR REMOTE CLOSURE.** `active_task` reads
`G2_ENGINEERING_CLOSURE_AUDIT` until the one successor commit records this closure's remote run; that
commit sets G2 PASS, Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE, and the pointer back to
`NONE`. Baseline stays `0.6.0`; no tag, release, installer or `v0.7.0`.

# G2 Product MVP Engineering Closure Audit — closed 2026-10-01

Verdict: **`G2 = PASS` — Product MVP ENGINEERING COMPLETE, state MVP CANDIDATE**, on the §34 checklist as
the addendum corrected it (`G2_VALIDATION/G2_EXIT_CHECKLIST.md`). `active_task` returns to `NONE`; baseline
stays `0.6.0`.

- **Remote closure as §42–§44 define it.** The audit / product tree `e35cfe7` on Run `36906482900` and the
  evidence head `f75cbc5` on Run `36948719972` are each `completed / success`, **7 of 7 on attempt 1**:
  Rust (windows-latest), Rust (ubuntu-latest), Desktop UI (windows-latest), Desktop UI (ubuntu-latest),
  Generated output drift, Dependency policy, macOS Core Smoke. No rerun was needed, and the Compare race
  closed before G2 did not recur.
- **This successor touches only live narrative surfaces and the G2 pack's remote sections**, so every
  surface a newcomer reads first agrees on one state — the defect class the P4 consistency closure
  existed to remove. No product source, schema, migration, dependency, fixture or golden moved.
- **No next stage is implied.** V1 own-artifact / real-user validation and P5 productization remain
  potential tracks, each needing its own architect decision; no authority file makes V1 a precondition
  of P5 or the reverse, and none is written here.
- **What PASS does not mean** is written beside it everywhere it appears: not productization, beta, RC or
  GA; not production-ready, signed or installable; not real-user or commercially validated; not
  security-clean; and not a licensed open-source release while
  `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` stands.
- **This commit's own run is external evidence.** Prompt §43 ends the CI→docs loop here: it is read with
  `gh run list` and reported, never written into another commit.

# Post-G2 E2E findings remediation — closed 2026-10-02

Verdict: **`POST_G2_E2E_FINDINGS_REMEDIATION = PASS`.** Three real-desktop findings fixed
(`E2E-F001` S3, `E2E-F002` S2, `E2E-F003` S3), no S0 or S1, and no governance status moved: **G2 stays
PASS, the product stays MVP CANDIDATE, baseline stays 0.6.0, `active_task` stays `NONE`.** Evidence in
`POST_G2_E2E_REMEDIATION/`, fix head `971015f` green on Run #43 `37100371601` at 7 of 7 on attempt 1.

- **E2E-F002 was fixed in the UI, and the bundle engine was left alone.** The false sentence — that an
  occupied folder "already holds a FirmwareSight release bundle" — is authored in
  `firmwaresight-project::bundle`'s `ERR-BUNDLE-6106` message, so the tempting fix is to edit it.
  Rejected: rewriting an engine message changes bundle semantics, which this round's brief reserves for
  Architect review, and hiding it in the UI would withhold an engine-authored fact the release owner is
  entitled to read. Neither was needed, because the desktop command already derives
  `recognizableBundle` from the same `is_recognizable_bundle` the write path consults before it replaces
  anything. The page therefore knows the truth before it could ask, and the guard stays exactly as
  strict: the engine still refuses a foreign folder, and the race where a folder stops being a bundle
  between the choice and the replace still ends in the truthful `ERR-BUNDLE-6107` state.
- **E2E-F003 widened one predicate and accepted a named trade-off.** GNU ld detection now searches the
  whole already-buffered MAP text instead of its first 4096 characters, because the banner sits behind
  preambles whose length the project does not control. Consequence stated rather than buried: a foreign
  MAP carrying the literal `Memory Configuration` beyond 4 KB would now reach the GNU parser, where it
  still fails closed for want of memory regions — a different error message, never a fabricated layout.
  No new linker family is claimed, and no Keil/IAR adapter was written.
- **Session identity stays out of storage.** The marker that says which selection a surviving analysis
  describes is one React state value keyed on the shell's selection handle. It is not written to the
  portable schema and not a SQLite column, and file-name equality is not used as identity — the case
  that has to be caught is two artifacts sharing one leaf name.
- **Performance observations were not converted into work.** Near-500 MiB first-use latency and peak RSS
  carry forward with the wording `PARTIAL / environment-sensitive` and `MEASURED FOR TESTED WORKLOAD`.
  This round changed no parser algorithm, no memory model, no guard threshold, and claimed no
  optimization.
- **A round may fix what it can prove and stop.** The 283-case acceptance named more than three
  observations; only the ones whose evidence showed a product defect inside existing MVP contracts were
  taken, each with a regression that went red before the fix and a mutation proof after it. Everything
  else is listed as carried forward instead of being quietly widened into scope.

# P5 Productization — opened 2026-10-03

Authorization: *FirmwareSight — P5 Productization, Execution Prompt v1.0*, delivered as a file
(SHA-256 `722125f5aa68e324ba1dea4826f66d8392acab9ad9a015c4919ed5ade471e0ae`, 73,722 bytes, 3,442 lines),
archived in `10_AUDIT/SOURCE_PROMPTS/` and registered there against a hash recomputed from the stored
bytes. `active_task: P5_PRODUCTIZATION`, stage `P5`, state `IN_PROGRESS`. The product it starts on is
unchanged: G2 `PASS`, **MVP CANDIDATE**, baseline `0.6.0`.

- **The audit came first, and it changed the plan rather than decorating it.** §4 forbids product code
  before `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md`, and the audit's §A–§H answers are what the round will
  be built on: no package has ever been produced (`bundle.active: false` at `tauri.conf.json:29`, one CI
  workflow with five job keys and no packaging step, five icons and no `.icns`); History needs read APIs,
  not tables; `PRAGMA integrity_check` and any database backup appear nowhere in the repository; and the
  committed fixture set — 25 files, six sets — is entirely `arm-none-eabi-gcc 14.3.1` plus GNU ld, so the
  "Clang ELF" half of the supported cohort has no compiler-produced evidence at all.
- **§7's premise was checked rather than copied.** The prompt says the Tauri config holds `0.1.0`; it does,
  and so do `Cargo.toml:14` and `apps/desktop/ui/package.json:4`, with the CLI following through
  `env!("CARGO_PKG_VERSION")` (`fwsight 0.1.0`, measured from the binary). So this is not one straggler to
  bump: `0.6.0` had never existed outside a document, and unifying means a deliberate choice.
- **The owner chose that identity: artifacts unify on `0.6.0`,** with the workspace version as the single
  source so the CLI, the fingerprint and a Snapshot cannot disagree. `baseline_version` in `BASELINE.yaml`
  stays `0.6.0` — this aligns the artifacts to the promoted baseline instead of promoting a new one, and
  §5's "do not automatically bump product version" is a prohibition on drifting, not on this decision.
  No tag, no GitHub Release, no published installer.
- **Migration `0005` was earned by a write-path defect, not by a UI wish.** §18 forbids creating a
  migration merely to make History easier, and History genuinely needs none: the rows already exist. What
  is broken is that `optional_fact_u64` (`db.rs:567-569`) returns the value and discards the reason, so
  `sections.file_offset` and `symbols.address` — the only 2 of 7 nullable numeric columns with no
  `*_unknown` twin — lose an `Unknown` reason at the moment of writing, and no read API can recover what
  was never stored. The owner's order is `P5_MIGRATION_DECISION.md` first, then the additive migration
  (those two columns plus an index on `builds.created_at`), then the §21 matrix over real v1–v4 stores.
- **The package matrix is bounded by the machines that exist.** Windows gets a real per-user install,
  uninstall and reinstall on this host, with the owner's live store parked, re-hashed and restored at both
  ends of the session; macOS and Ubuntu get CI-built, archived packages recorded as `CI_BUILD_ONLY`. The
  word `SUPPORTED` is not available to those two rows, because §27 says a successful compile is not a
  runtime claim, and `04_TECH/20:18-20` puts them at Tier 2. 125/150 % DPI, a second hardware host and any
  macOS/Linux runtime stay carried forward.
- **The store keeps its inherited name.** `firmwaresight-p0.sqlite` under `app_data_dir()` is where the
  existing history already lives; a rename would be cosmetic and would put real user data in front of a
  data-move code path. The chosen fix is disclosure instead of migration: Diagnostics and the install
  documentation state the path, so the file is findable and backupable.
- **Packaging chose the targets the runners can prove, not the list that reads best.** Windows builds
  NSIS only — MSI would pull the WiX toolset for an "enterprise evaluation" no evidence asks for. macOS
  builds the `.app` and the `.dmg` that wraps it, because the embedded-frontend check needs the `.app`'s
  bytes. Ubuntu builds `.deb` and not AppImage, because AppImage needs `linuxdeploy` at build time and
  FUSE at run time on a platform this round may only call `CI_BUILD_ONLY`. `AGENTS.md` §2 is intact: no
  framework changed, no target forced, and §8's instruction to record the canonical target per platform
  is answered in `04_TECH/17`.
- **`installMode: currentUser` is written down rather than inherited.** It is Tauri's default
  (`tauri-utils` 2.10.0 `config.rs:825-844`), and it is the mode that cannot prompt for administrator
  rights and keeps its metadata under `HKCU` — which is what an install on the owner's own machine should
  do. Setting it explicitly makes any later change to it a reviewable diff instead of a silent drift.
- **The Tauri CLI is a CI tool, not a dependency.** `cargo install tauri-cli@2.12.1 --locked`, the same
  shape this workflow already uses for `cargo-deny@0.20.2`: it never enters `Cargo.lock`, so the audited
  product graph is unchanged. The npm alternative (`@tauri-apps/cli` as a devDependency) was rejected
  because it would have widened the frontend's locked dependency set for a tool the frontend does not use
  at runtime — an `AGENTS.md` §4 question with an answer of "the standard tooling here already has a
  pattern".
- **A check that would have passed the broken build was caught before it shipped.** Section 8's "must not
  depend on `localhost:5173`" invites grepping the binary for the dev URL. Measured on this tree: the
  release binary *with* `custom-protocol` and the debug binary *without* it both contain
  `localhost:5173`, because it is embedded config either way — so the tempting check fails every good
  build and passes every broken one. The discriminating signal is the built asset table
  (`assets/index-*.js`, `assets/index-*.css`), present in the shipping binary and absent in the dev-mode
  one, and `scripts/verify_package_artifacts.py` asserts that, against the binary that was actually
  packaged.
- **Version identity became a gate step.** §67 makes a version split a closure blocker, so
  `drift/version identity` now compares `[workspace.package] version`, `tauri.conf.json`,
  `apps/desktop/ui/package.json` and `BASELINE.yaml product.baseline_version` on every commit and
  requires every crate to inherit `version.workspace = true`. Both directions are proven to bind by
  mutation. The packaging that once asked a human to read four files is now a step that can fail.
- **CI went from seven authoritative jobs to ten, with the reason on record.** §41 asks for package jobs;
  the frozen matrix in `05_ENGINEERING/06` put packaging on a nightly or a release tag, and this
  repository has never had a scheduled workflow, so that row has never run and cannot produce this
  stage's evidence. The three jobs therefore run on a push to `main` and still not on a pull request, and
  the change is written into `05_ENGINEERING/06` rather than left implicit. The apt prerequisite list that
  used to exist in two jobs now exists once in `.github/actions/linux-tauri-prereqs`, which is the half of
  L17 this commit could close.
- **One gate run went red for a reason that was not the commit it ran on.** Run `37128593254` on `9e3b1de`
  failed `Desktop UI (windows-latest)` on a pre-existing race in `compare.test.tsx` — the region mounts
  before its first page lands, the same shape `055b54e` closed at the pager and said had been left
  elsewhere. It reproduced here without CI (1 of 30 fresh runs), and under 25 ms of injected mock latency
  (3 of 3). The repair is one awaited query, test-only, proven load-bearing by a never-resolving query
  that keeps it red. The failed run is recorded in `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` rather than
  re-run until a green attempt appeared.
- **A step that did not run is not a pass, and in CI it is a failure.** Run `37133706214` — the first
  10-job run — went red on all three package jobs for a reason that no result line admitted: `cargo
  install tauri-cli` leaves a binary called `cargo-tauri`, the group probed the bare name `tauri`, printed
  `SKIPPED: the Tauri CLI is not installed on this machine`, summed it as `4/4 steps passed`, exited 0, and
  was caught only by the job's own `if-no-files-found: error` upload. The rule now in `scripts/check.py` is
  the general one: a skip is recorded as `SKIP`, kept out of the passed total, and makes the run exit
  non-zero whenever `CI` is set, because a CI job installs every tool its gate needs and therefore has no
  license to skip one. cargo-deny's local skip moved onto the same machinery, and the Tauri CLI is
  discovered by probing the forms in the order the install methods produce them — `cargo tauri` first,
  because that is what a `cargo install` leaves, then the bare `tauri` the npm package installs.
- **The package step runs from `apps/desktop`, and that is where a hand-run build belongs too.** The CLI
  resolves its frontend directory from the process cwd and falls back to the shell directory's parent when
  it finds no `package.json` nearby, so `cargo tauri build` from inside `src-tauri` ran the config's
  `pnpm build` in a directory that is not a pnpm package and failed. `apps/desktop` is the folder that holds
  both `ui/` and `src-tauri/`, and from there the CLI's own lookup finds the frontend. Chosen over the
  config's `{script, cwd}` hook form because that `cwd` is still resolved against wherever a person
  happened to stand. Building it for real also moved two claims from assumption to evidence: the NSIS
  package does carry `target/release/firmwaresight-desktop.exe`, and this repository has now produced an
  installer (§5b of `P5_PACKAGING_REPORT.md`) — produced, not yet installed anywhere at the time this bullet
  was written. It has since been installed on this host, uninstalled and reinstalled with the owner's store
  parked and restored, and `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md` records what the uninstaller does to
  user data: nothing, and without asking.
- **A checksum index lists files, not directories, and the payload gets its own digest.** The macOS job's
  own artifact set was the evidence: its `SHA256SUMS.txt` gave `FirmwareSight-…-app.app` a single line
  carrying an aggregate tree digest, and `sha256sum -c` — the tool §41 names — answered
  `Is a directory / FAILED open or read` on a correct build. A directory bundle now contributes one line per
  file inside it, path relative to the index, and keeps the aggregate digest in `artifact-metadata.json` as
  the artifact's identity. Each entry also records `payload_sha256`, and diffing two consecutive builds of
  this host showed why that matters: the payloads are the same length with **20 bytes different**, all of
  them linker identity — the PE `TimeDateStamp` (repeated in three debug-directory entries) and the 16-byte
  RSDS CodeView GUID — and the installers differ in size too. **A package digest is therefore not an
  equality key across builds here or across hosts**, so no release, cache or attestation in this project may
  be justified by "the digest would have matched"; the reproducibility record stays the input fields
  `04_TECH/18` lists, and `04_TECH/18` now says so in its own words. The next run proved the index rule on
  the runner rather than at home (`37143046338`, head `53578e9`, **10 of 10**: five darwin lines, all `OK`,
  exit 0, and one flipped byte in a copied `Info.plist` making it exit 1 and name the file), and its
  comparison of the two runs added a nuance worth keeping: the macOS `.app` tree digest repeated exactly
  between them while the `.dmg` of identical length around it did not — one pair of runs is not
  reproducibility, and a changed container is not a changed program.
- **Two questions were refused, correctly.** §32 sends L22 — line endings moving a release's content-derived
  identity, which today fails closed — to the Architect as `P5_RELEASE_IDENTITY_ADR_DRAFT.md` plus a STOP,
  because normalizing the digest input would change identity semantics, an `AGENTS.md` §2 move. And §54 is
  absolute: this round must not choose MIT / Apache-2.0 / GPL / AGPL / MPL, so
  `OPEN_SOURCE_LICENSE_DECISION = PENDING_OWNER_CONFIRMATION` is written into every P5 document rather than
  resolved by convenience.
- **A dark theme stays a conflict, not a task.** The lifecycle document asks for one while ADR-0018 and
  `AGENTS.md` §11 freeze MVP light-only; implementing it would silently move a frozen design baseline, so
  P5 carries it forward and says why instead of quietly choosing a side.
- **What `IN_PROGRESS` forbids.** No `P5 PASS`, `BETA`, `RC` or `GA`; no signing or notarization executed
  (`READY_NOT_EXECUTED`) and no updater (`UPDATE_READY_MANUAL`); no new network capability, telemetry,
  analytics SDK, generic shell or filesystem permission; Diagnostics allowlist-only with positive-control
  leak tests; and the security sentence stays the permitted one — *dependency policy passes with documented
  accepted risks*, never "security clean".

# P5 Commit C — onboarding, Help and local History — 2026-10-03

Prompt §§13–18. One new product surface (History), one new auxiliary surface (Help), no new verb. The
decisions below are the ones a later reader cannot recover from `git log`, each with the alternative that
was rejected and what settled it.

- **History was built on reads, and §18's order was obeyed rather than assumed.** The prompt says try the
  existing persisted facts first and forbids a migration that merely makes History easier, so the work
  started by enumerating what the schema already stores: everything a build, Gate-run or release row needs
  is there. What did not exist was a *read path*, and that is what `crates/firmwaresight-storage/src/history.rs`
  adds — three bounded queries over the existing tables, with the build table delegating to
  `list_compare_candidates` instead of re-writing its SELECT. Migrations stay at five. The audit's own
  forecast needed correcting in passing: section C expected "one index if the ordering must be cheap", and
  `0005_unknown_reasons.sql:33` had already added `idx_builds_created`; measuring 100 builds / 100 runs / 50
  releases at 467.9µs / 541.5µs / 248.8µs on the release profile said no further index has a reason.
- **A page may not write its own title bar.** The frozen window title (L21) had two obvious fixes and both
  were rejected: `getCurrentWindow().setTitle()` from the WebView needs
  `core:window:allow-set-title`, which is a capability change `AGENTS.md` §9 reserves for a human *and* a
  channel from a file name into a window property; and `tauri.conf.json` can only hold one static string for
  a window that now has five pages. So `App.tsx` reports which page moved and Rust composes
  `FirmwareSight - <Page>` from the closed five-variant `MainWindowPage` enum — the same reason `FixtureKey`
  is an enum, it is the boundary. `the_title_fix_took_no_new_capability` reads `capabilities/main.json` and
  asserts the permission list is still exactly `["core:default"]`, so the claim survives someone editing the
  capability file later.
- **A filter box is not allowed to become a directory oracle.** Matching the stored `artifacts.path` was the
  natural thing for a search box to do, and §16 forbids the consequence: a field that answers "which folders
  on this machine hold firmware" is a question History may not ask, and it would be asked in the clear by
  anything that can type into the box. Filters therefore search identity columns only — build id, snapshot
  id, SHA-256, architecture, run/release id, version, policy and manifest digests — each a closed list in
  `history.rs`, with the text bound as a value through the existing `like_pattern` escape. The proof is the
  negative one: `a_filter_searches_identity_columns_and_never_a_directory` seeds real builds into a real
  temporary directory and asserts that filtering by that directory's own name returns zero rows.
- **The front end holds no copy of any fact Help displays.** D1 made the workspace version the single source
  of truth, so an About page with `0.6.0` written into JSX would be the first drift candidate in the tree.
  `get_app_identity` answers from `package_info()` and `config()`, and until it answers all seven rows read
  `not reported` — chosen over `-` because a dash reads as "this application has no version", which is a
  different and untrue claim. The store is named by **file** (`firmwaresight-p0.sqlite`) and not by folder:
  the path is a Diagnostics question with its own allowlist and positive-control tests, and Diagnostics is
  Commit D.
- **Guidance is one component with two readers.** `GettingStarted.tsx` exports the seven answers once;
  Analyze wraps them in a dismissible panel and Help renders the list. A second copy would have been the
  predictable failure — §13 and §14 ask for the same content in the same words — so `help.test.tsx` reads
  the `term` nodes off both surfaces and compares them to one array rather than trusting the shared import.
  Dismissal state lives in `App.tsx`, above the page switch, because hiding is the reader's act and a trip
  to History is not a reason to contradict them. No `<dialog>`, no first-run sequence, nothing on Analyze
  waits for it: the panel sits *below* the controls that answer it.
- **A pointer a stranger cannot follow is not printed as though it worked.** `bundle.resources` is `None`, so
  the installed package ships no markdown: Support Matrix, Known Limitations and Diagnostics each carry the
  label `not carried inside the installed package` on the line that names their repository path, and Help
  has zero `<a>` elements because this build has no URL worth giving and no network to open one with. The
  alternative — quietly bundling three documents under §14 — was rejected as a packaging change with no
  owner decision behind it, and the alternative of naming the paths plainly was rejected as a pointer that
  fails. The test counts them: four rows, three flagged.
- **Four workflow pages, and Help kept out of that sequence.** §15's canonical rail is Analyze / Compare /
  Release / History and "do not add more top-level verbs", so Help is listed under its own
  `aria-label="Help and about"` navigation rather than appended to the rail, where its position would read
  as a fifth stage of a workflow it does not participate in. Three rail-guard tests were widened together
  (`intake`, `compare`, `release`) and `History` left the banned-word lists while `Settings`, `Pricing`,
  `Cloud`, `SBOM` and a Bundle navigation verb stay excluded — `Bundle` is still a section of Release, and
  `release.test.tsx` still asserts the rail holds no `/bundle/i` button.
- **`History` is a word this round earned, and only this round's evidence says so.** The UI tests that had to
  change did so because the claim they pinned became false, not because a test was in the way: a rail that
  lists a real page must not be asserted to list three. Each such edit is listed in
  `P5_VALIDATION/P5_ONBOARDING_HISTORY_REPORT.md` §5 with the mutation that proves the new assertion bites,
  and seven of them are recorded there — filter offset, request guard, disposition fallback, full digest on
  open, filter payload, a write control on a read-only page, and the Help version default.

# P5 §32 — release identity and line endings: documented, not changed — 2026-10-03

L22 was the one P5 question this round may not answer itself. §32 allows documenting and testing it and
forbids silently normalizing release identity, and four governance sentences already claimed an ADR draft
had been sent to the Architect — a claim about a file that did not exist. It exists now
(`P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md`), and no behaviour changed to produce it.

- **What the current semantics actually are, stated once with their citations.** The release-notes digest is
  the digest of the file's bytes: `observe_release_notes` → `fingerprint::file_sha256`, which streams raw
  bytes with no decoding anywhere on the path. That digest is a labelled line in the Gate's canonical input,
  the Gate run id is the digest of that input, and the release id's canonical text contains both the notes
  digest *and* the Gate run id. So a checkout that rewrites LF to CRLF has not changed what the notes say —
  it has changed which release this is.
- **Tested, because §32 asks for a test and an argument is not one.**
  `a_notes_file_that_differs_only_in_line_endings_is_a_different_release` in
  `crates/firmwaresight-project/tests/bundle_builder.rs` writes the same words in LF and CRLF into two
  release subjects and pins five things: each recorded digest equals the digest of exactly the bytes on
  disk; the digests differ; the run id and the release id both differ; **the verdict does not** (same
  `overall_effective_severity`, same five-state counts); and the published bundle ships those same bytes and
  verifies against its own manifest. The fourth assertion is the one that keeps a normalization from
  arriving as a presentation fix — identity and correctness are different questions, and a test that only
  checked the ids would not notice them being conflated.
- **Proven to bite by implementing the forbidden thing temporarily.** `observe_release_notes` was rewritten
  to hash the bytes with every `\r` removed — option A applied at exactly the place that would make it
  invisible — and the test failed on "the recorded digest equals the bytes on disk". The mutation was
  reverted; `git diff` on `evidence.rs` is empty and the test passes. Reverted with `Edit`, not with
  `git checkout`, because that file is tracked and a restore would have been the destructive kind of tidy.
- **Why neither option was taken.** Normalizing before hashing makes the identity digest describe bytes that
  are not the bytes in the bundle, so the release id and the bundle's own `SHA256SUMS` disagree for one
  file; the only self-consistent variants are for FirmwareSight to rewrite a copy of a human's document
  inside a release artifact, or to ship an id nobody can recompute from what shipped. Hashing the Git blob
  instead fixes the checkout problem by construction but assumes the notes file is tracked in a repository,
  which the product explicitly does not assume today (a missing Git fact is recorded as *unknown*, not as an
  error), and it gives a second identity source to a design where each fact has one. Both change what an
  Observed fact records, which `AGENTS.md` §2 reserves for an ADR — hence the draft, hence the STOP.
- **One fact this round found that the G2 row did not carry.** The fail-closed behaviour a reader would
  expect here is policy-dependent: with the default `require_clean_git = true`, a smudged checkout is a dirty
  workspace and `git.clean` BLOCKs. With it set to `false`, the rule is `N/A` and the release id moves with
  nothing on screen saying why. That is the residual risk worth the Architect's attention, and it is written
  into both the draft and the audit's L22 row rather than left implicit.
- **What was deliberately not done.** No `.gitattributes` advice turned into product behaviour: writing into
  a customer's project folder is out of bounds (`AGENTS.md` 7), so the repository's own rules stay ours and
  the user's remain documentation. And `P5_KNOWN_LIMITATIONS.md` — where L22's outcome vocabulary belongs —
  stays Commit F's deliverable; this commit records the disposition, not the closure.



# P5 L23 — the race class's sixth instance, closed test-only — 2026-10-03

Counting from the frozen G2 row, which names P3's defects I and J, `055b54e`'s pager and G2-F1 as instances one
through four, `20b03e3` closed the fifth (the `columnheader` read Run `37128593254` lost) and this is the sixth.
`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` §0a had asked for a sweep "instead of letting the next red run name
the next instance". The next instance named itself eight
commits later, on a tree whose UI source was byte-identical to a head that had passed 10 of 10 twice: the
**local** full gate stopped at `frontend/test`, with `rust`, `drift` and `deny` green and the same file red —
`compare.test.tsx` › "lists growth and additions apart" reporting `Unable to find an accessible element with
the role "button" and name "Show sections changes for .noinit"`, while `.text` sat on the same screen.

- **Root cause, stated as the component's own contract.** The ranking region renders from `summary` alone; the
  added rows are a *second* read issued in the effect at `Compare.tsx:217-256`, which commits separately, and
  the component has an explicit branch for the interval between the two (`Reading the added rows…`,
  `Compare.tsx:876-877`). The test awaited only the region and then read second-wave content synchronously, so
  the outcome was decided by which React commit landed first. Three full-suite runs on an idle host passed; the
  gate, running under its own load, did not. That asymmetry is the reason the fix is a wait and not a retry.
- **Repaired without touching behaviour.** One `findByRole` replacing one `getByRole`, no production source, no
  timeout constant, no assertion removed or widened. `git diff --stat` for the repair is a single test file.
- **A contract test, because the flake was really an untested branch.** "says the added rows are still being read
  before it names one of them" holds the added-rows read open with a deferred promise and asserts the status
  line, the *absence* of any added-row button, and that growth is unaffected — then resolves and asserts both
  flip. Nothing in the suite covered the pending branch before this.
- **Three mutations, one per assertion that matters.** (1) the old synchronous read with the read held open
  reproduces the gate's error verbatim, which proves the awaited form is load-bearing rather than polite; (2)
  the status branch rewritten to the false claim `No section was added.` reddens the new test, which proves the
  test would catch a normalization of Unknown into absence; (3) an `.noinit` row injected into the growth list
  reddens the absence assertion, which proves the test catches a fabricated row. All three were reverted with
  `Edit`; `git diff --stat` shows `Compare.tsx` untouched.
- **20 fresh-process repetitions of the changed suite, 0 failures** — section 35's count for a suite this commit
  changed. The full gate then ran **16 of 16** on the exact tree: **813 Rust / 201 UI in 8 files**.
- **The sweep, in bounded form, with its limit written next to it.** Chained IPC waves — a read issued from state
  only another read produces — were enumerated across the six page components that hold effects, and one feeds
  content a test sampled early: the wave closed here. Details' contributor read and Release's derived default build
  sit in the same shape but their tests await the region each paints. Release keeps further `run`-keyed waves
  (`Release.tsx:1055,1062,1083`) that were not walked assertion by assertion, `within(region).getBy*` reads inside
  already-awaited regions were not audited at all, and the empirical half is 3 idle-host full runs plus 20 runs of
  the changed file, not §35's 20 repetitions of every suite. The five remaining top-level synchronous reads are
  same-commit or post-await (`compare.test.tsx:530-531`, `release.test.tsx:1152,1157,1323`). §0a carries the full
  list with its limits; L23 stays `SHOULD_CLOSE_P5`, and the complete sweep is a named task, not a claim.
- **What this changes about the closure rule.** `check.py` records only the steps it runs and stops a group at
  its first failure, so a red `frontend/test` reports **15** executed steps (3 + 4 + 7 + 1) where a clean pass
  is **16** (3 + 5 + 7 + 1). A fraction like `14/15` says where a run stopped; it is not a different gate, and it
  must never be read as one.

# P5 Commit D — diagnostics, integrity, backup and the startup boundary — 2026-10-04

Prompt §§4-27 of *P5 Commit D Continuation v1.2* (sha256 `600d70a3…274033`, 31,588 bytes, 1,472 lines,
archived and registered). The decisions below are the ones a later reader cannot recover from `git log`, each
with the alternative that was rejected and what settled it.

- **L22 is decided, and the deciding document is the ADR, not the draft.** `ADR-0028` is Accepted:
  *release identity uses the exact bytes observed on disk*. Rejected: normalize on read (it would silently
  re-hash a person's release notes and make an old release id unanswerable), and hash the Git blob instead
  of the file (it makes the working tree's bytes irrelevant while still letting `require_clean_git = false`
  move the id). The consequence is accepted rather than smoothed: two files with the same words and
  different line endings are two releases, and `require_clean_git = false` can move a release identity with
  nothing on screen saying why. `.gitattributes` is recommended **in the user's repository**; FirmwareSight
  does not normalize and does not edit its own. The draft that asked is re-statused `EXECUTION_RECORD` and
  its analysis stays, because "what was considered and rejected" is the part that stops being re-derivable.
- **The backup is SQLite's online backup API, and that was measured before it was chosen.** A plain file
  copy is wrong under WAL — the committed state is the file *plus* the frames the log still holds — and
  `VACUUM INTO` was tried and rejected: it binds its filename as SQL `TEXT`, so a non-UTF-8 store path
  either fails or names a different file, and pointed at an occupied path it answers with an engine message
  containing an **absolute path** the shell would then have to redact. What made the API affordable is that
  `rusqlite`'s `backup` feature has an empty dependency list, so enabling it adds no package: storage subtree
  22 either way, `Cargo.lock` unmodified. The rule, not the reading: the manifest was toggled and
  `cargo tree` re-run in both states.
- **A snapshot is owed to a store that already has a schema and has an upgrade pending — and to no other.**
  A fresh database is created at the current version, so it gets no backup and Diagnostics correctly reports
  `backupFiles: []`. Retention is bounded rather than growing beside the user's data. A snapshot that cannot
  be written stops the migration *before* it starts; the alternative (upgrade, then try) is how an
  unrecoverable half-state is produced.
- **Diagnostics lives on the page that already exists.** A fifth rail verb was the obvious shape and is
  wrong: `AGENTS.md` and the P5 prompt forbid a new product verb, and a support payload is a reading
  surface, which is what Help already is. The payload is assembled in Rust from an explicit allowlist —
  **41 keys**, the set `ALLOWED_KEYS: [&str; 41]` names at
  `apps/desktop/src-tauri/tests/diagnostics.rs:47`, which counts the nested object keys as well as their
  leaves — and the UI renders what the shell answered or `not reported`, never a default. This row's earlier
  "31 leaf fields, counted off the exported file" was the last survivor of the mistake the bottom of this
  document already records: the count now reads off the constant that defines it.
- **The store is named, not located.** `firmwaresight-p0.sqlite` appears on screen and in the file; its
  directory appears in neither. Four sentences elsewhere in this repository had promised that Diagnostics
  "will tell you where it sits"; they were stale, and they were corrected rather than re-explained. The
  privacy claim is a positive control, not an absence: the exported file contains no `/` and no `\`
  character at all, and a test that plants a path-shaped value and demands it come back is what makes that
  a proof instead of a hope.
- **Store health is prose with no status colour, and that is a reversal this commit made.** The first draft
  rendered `healthy` in `--fs-color-status-pass` and `unhealthy` in `--fs-color-status-block` — the verdict
  palette on a page with no verdict. `DESIGN.md` 5 names five states and health is not one of them;
  `DESIGN.md` 9 says colour only ever aids a carrier. Either a badge with its icon, or a fact in the page's
  own grey, and the second is what a reading page can actually say. The word is always present, which is
  what the test asserts, and the mutation that mistranslates the word still reddens it.
- **The startup refusal leaves from inside the hook, because the framework will not carry it out.**
  `tauri::Builder::run` was read as `build(context)?.run(…)`, and from that a `setup` error was expected to
  arrive as `Error::Setup`. The packaged binary answered **101** with a framework `panic!` naming a
  build-machine path: the hook runs inside the event loop's `Ready` arm and a returned `Err` is panicked by
  Tauri itself (`tauri-2.12.0/src/app.rs:1443-1445`). So `StartupFailure::abort()` prints and exits with
  `EXIT_CODE` (1) from where the refusal happens, and `run()` keeps `.expect(…)` for faults this round has
  no words for. A refusal and a crash are different events and a launcher can now tell them apart.
- **A file that is not a database is not a rolled-back migration.** `ensure_bookkeeping()` reports its own
  failure as `Migration { version: 0 }`, and the sentence written for a half-run step claimed an upgrade had
  begun and been undone. Version 0 now answers in its own words and carries the engine's phrase, filtered,
  because "file is not a database" and "database is locked" ask a person for different repairs.
- **`payload_sha256` is a locator, not the digest of what a user launches.** The bundler overwrites 3 bytes
  of the binary it packs — the token `__TAURI_BUNDLE_TYPE_VAR_UNK` becomes `…_VAR_NSS`, same length
  (`tauri-bundler-2.10.1/src/bundle.rs:41-95`) — so that the running application can name its own install
  channel, which is why the installed build reports `installChannel: "nsis"` and a `target/release` run does
  not. The verifier's own wording claimed "the exact file this installer bundles"; the measurement corrected
  the sentence rather than the digest.
- **The authority for this round is v1.2, and the record says why.** v1.2's §1 named
  `FirmwareSight_P5_CommitD_Continue_v1.1.txt` (`f38ee5f2…ab045`, 27,834 bytes, 1,014 lines) as canonical and
  required a STOP if those bytes could not be found. They cannot be found on this machine, in this
  repository or in the temp tree. The measurement and the stop were reported, nothing was reconstructed from
  memory, no byte-exact archive was claimed for v1.1, and the owner chose to archive **v1.2** as this round's
  authority. The v1.0 prompt that an earlier round had archived as *the* authority is kept, re-labelled
  `SUPERSEDED, HISTORICAL`.
- **A test that passes on one machine's clock is not a test.** The snapshot-replacement assertion in
  `integrity_and_backup.rs` compared two whole files and required them to differ, and the only difference its
  two fixtures could offer was the second-granular `applied_at` default on `schema_migrations`. So the product
  head came back **8 of 10**: `macOS Core Smoke` and `Rust (ubuntu-latest)` red on that one line, the same test
  green on `Rust (windows-latest)` and on this host. Rejected: sleeping until a second boundary (a slower test
  that still races), retrying until the bytes differ (a test that passes by waiting for luck), and dropping the
  assertion (which snapshot survives is worth proving). The repair marks each store with a named `sections` row
  and reads that row back out of the standing snapshot, so the claim is about data and the byte comparison is no
  longer the only evidence for it. The failed head stays listed in `P5_CI_AUTHORITY.md`; the row was not made to
  wait for a green one.
- **A number written into six sentences without reading the constant that defined it.** Commit D's
  documentation called Diagnostics "a closed 31-field allowlist"; `ALLOWED_KEYS: [&str; 41]`
  (`apps/desktop/src-tauri/tests/diagnostics.rs:47`) names 41 keys, the eight `Diagnostics*` structs declare
  41 public fields, and `no_field_of_the_payload_can_hold_a_path` fails if that set moves. The count was
  corrected in all six places, and the design doc §7 now records that 31 was never a stale value drifting —
  it was asserted while the authority for it sat in the same commit, unread. A count that is worth printing is
  worth reading from the file that defines it, which is the same rule that makes the round's test totals come
  from a `cargo test` run rather than from prose.
- **What this commit did not do.** No Commit E fixture expansion, no parser performance work, no signing, no
  notarization, no updater, no telemetry, no network, no cloud, no accounts, no licence choice, no fifth
  top-level page, no P5 closure. `capabilities/main.json` is still `["core:default"]`; the two new commands
  take no arguments at all; `unsafe_code` remains forbidden in both crates that grew code. The permitted
  security sentence stands as written: *dependency policy passes with documented accepted risks*.

# P5 Commit E — compatibility fixtures, support matrix and supportability — 2026-10-04

Authorization: *FirmwareSight — P5 Commit E — Compatibility Fixtures, Support Matrix & Supportability
Closure, Execution Prompt v1.0*, delivered as a file (SHA-256
`030ca28233b964958d6aea8e59a312c66ceb4d117273cba9e667950b4ba21148`, 53,915 bytes, 2,459 LF lines, §0–§64 — 65
numbered sections),
archived in `10_AUDIT/SOURCE_PROMPTS/` and registered there as the current active authority, which replaces
Commit D's entry as `CURRENT ACTIVE AUTHORITY`. Its discipline is the round's boundary: REAL FIXTURES / NO
HAND-EDITED EVIDENCE / CLAIM ONLY WHAT WAS PROVEN / NO FORMAT SCOPE EXPANSION / NO P5 CLOSURE / NO L26 FIX /
NO FULL §64 JOURNEY.

- **Start authority was waited for, not assumed.** Commit D's documentation closeout (`57a904e`) had shipped
  two sentences that its own existence made false, and its correction `bfbc1aa` was still running. The owner
  chose to start from `bfbc1aa` **after that run concluded** rather than beside it; measured afterwards: Run
  `37214675036`, `completed` / `success`, **10 jobs**, every one of the ten names read. So the round began on a
  head the remote had already agreed about, and the rebase audit in `P5_COMMIT_E_DESIGN.md` §1 was written
  against it rather than against the prompt's assumed tree.
- **A fixture's expected answer must be derivable without the model under test.** This is the round's central
  design decision. Each `fixture.toml` carries `expected_image_bytes` and `expected_live_ram_bytes` computed by
  `scripts/gen_p5_compat_fixtures.py` from `readelf` section headers plus the MAP's own `Memory Configuration`
  region attributes, and `p5_compat_fixtures.rs` parses those strings out of the record and holds the analyzer
  to them. The rule was first validated against evidence the round did not create: the same arithmetic
  reproduces `p0-dual-region` (image 160 / live RAM 72), `p2-diff/base` (256 / 8) and `p2-diff/target`
  (376 / 76) — `P5_COMPATIBILITY_FIXTURE_REPORT.md` §2 cites the pre-existing test lines that already carry
  those numbers. A test that re-prints the
  model's own totals would have kept this round's defect hidden.
- **That independence found a product defect, and the owner chose the narrow fix.** clang emits
  `.ARM.exidx.text.main` with `sh_type` `0x70000001` (`SHT_ARM_EXIDX`) and `SHF_ALLOC` set, 8 bytes at
  `0x08000030`; `map_section_kind()` mapped an unrecognised kind to `SectionRole::Unknown` and the model then
  inferred "not allocated" from "no recognised role", so those 8 image bytes entered neither budget while the
  total still reported `Exact` with nothing in `unattributed`. Three responses were on the table — a
  performance-shaped rework of the accounting model, an allowlist of ARM-specific section types, and reading
  `SHF_ALLOC` from the section header. The owner chose the third: it is what
  `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §3 already requires (a custom section settles by segment and region
  evidence, never by its name) and §4 item 3 already lists as an evidence source, it changes one derivation,
  and it is falsifiable by a fixture rather than by an argument. Mutation A reverts the rule (`left: 50,
  right: 58`), mutation E reads `SHF_WRITE` instead and reddens 8 tests across the workspace.
- **Ten of eleven were already right, and that is the lesson, not the fix.** The cohort-wide invariant
  (`the_reported_image_footprint_equals_the_bytes_the_file_claims_are_loaded`) reads every manifest-listed ELF's
  section headers and compares the sum of allocated non-`NOBITS` payload to the model's charged total. Only
  `p5-clang-arm` has an allocated section whose `sh_type` sits outside `SHT_PROGBITS`, so no per-fixture test
  written before this round could have seen the gap: the defect needed a second compiler, not a second assert.
- **L15 is a contract decision and this round stopped on it.** The prompt's §42 asks whether
  `ElfProgramHeader` should be renamed. Measured: the label is already serialized in three namespaces with two
  spellings — `elf.program-header` in `schemas/analysis.schema.json` and the report DTO, `ElfProgramHeader` in
  the stored `evidence.source_type` (`db.rs` formats the Debug name) and in the bundle's `analysis.json` — and
  four golden rows carry it. Renaming is a breaking change to a persisted, published shape, so
  `P5_COMMIT_E_SCHEMA_DECISION.md` records the three namespaces, costs four options, asks four questions and
  ends `STOPPED_FOR_ARCHITECT`. No byte moved.
- **A test's settle primitive was changed instead of its assertion, so the change needed its own proof.**
  `history.test.tsx`'s stale-reply test waited a macrotask; the bounded sweep found a second timer the test did
  not need. Replacing the sleep with `await act(async () => { await Promise.resolve(); })` keeps the test green
  but proves nothing on its own, so mutation **F** breaks the production guard (`History.tsx:94` →
  `if (false)`) and the edited test must still redden. It did; `History.tsx` was restored and verified by
  digest `29022971f963082d618ae9178771eaee5b0279dccdd10e8f229b3fa266e5ecea` with `git diff` empty. Deleting the
  claim was the rejected option.
- **The three questions the owner answered, answered before the work rather than after it.** Start from
  `bfbc1aa` once its run concluded; fold the `.ai/DECISIONS.md` "31 leaf fields" → **41 keys** correction into
  this commit rather than shipping a separate documentation-only head; take the Clang-exposed undercount now,
  narrowly, with a regression test and a mutation proof — not as a accounting-model project.
- **One push, one run, and the row that says so instead of inventing a second one.** §56 split the round into
  E1 (`859648e`) and E2 (`59d85c3`); §58 then required a fast-forward push and a 10-of-10 read. One
  `git push origin main` carried both commits, GitHub starts one workflow run per push and binds it to the tip
  head, so `gh run list --json headSha` shows `37228929762` on `59d85c3` and **no run whose head is `859648e`**.
  The temptation is to write a run number on the E1 row anyway; the record instead states the absence and gives
  E1's bytes a second, independent verification: a clean detached worktree at the candidate SHA where 56 manifest
  paths exist, are tracked and match their recorded hashes with no generator run, the 14 fixture tests pass with
  `arm-none-eabi-gcc`, `clang`, `arm-none-eabi-readelf` and `ld.lld` all absent from `PATH`, and the full gate
  runs **18 of 18** (the same 16 steps plus the two asset steps a cold checkout needs). Read back: attempt 1,
  `completed` / `success`, 10 jobs, every step inspected; the run's single `skipped` step is the conditional
  Ubuntu apt block on the Windows runner.
- **What this commit did not do.** No Commit F, no P5 closure, no `P5 PASS`, no tag, Release or published
  installer, no §64 installed journey, no L26 fix and no baseline-artifact regeneration beyond the current
  convention, no general source-control linter, no DWARF consumption, no new format, adapter, crate, capability,
  IPC command, schema, migration, design token or dependency, no licence choice, and no performance claim.
  Locally on the final tree: **868 Rust / 217 UI in 8 files**, `check.py` **16 of 16**, `core-smoke` 3/3,
  drift 7/7, deny 1/1, package **4 of 4 with no `SKIP`**, campaign **20 runs / 0 failing**. The permitted
  security sentence is unchanged: *dependency policy passes with documented accepted risks*.

# P5 Commit E closure normalization — evidence vocabulary and the L15 decision — 2026-10-04

Authorization: *FirmwareSight — P5 Commit E Closure Normalization (Evidence Vocabulary + L15 Architect
Decision), Execution Prompt v1.0 — Architect Reviewed*, delivered as a file and archived as
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitE_Closure_Normalization_v1.0.txt`. This is not a new product
stage: §1 permits documentation, governance, audit and integrity paths only, and §19 fixes the direct counts at
**Rust 868 / UI 217** so that a test-count movement in this round is itself the alarm, not a result.

- **A status column is a contract, and Commit E's matrix broke it while its facts stayed right.** The engineering
  prompt required five values (`SUPPORTED / SUPPORTED_WITH_LIMITS / CI_BUILD_ONLY / NOT_TESTED / UNSUPPORTED`)
  and the file shipped with `BEST_EFFORT_NO_CLAIM`, `DEFERRED_NO_MVP`, `NOT_CLAIMED`, `MEASURED, NOT INFERRED`,
  `BUILT_AND_VERIFIED_IN_CI` and half-sentences like "`SUPPORTED` for the region" in status cells. Each was
  re-labeled to the canonical word and its meaning moved verbatim into the evidence column; a scratch validator
  that parses the markdown tables and reads only columns whose header is `status` now reports **38 cells
  checked, 0 violations**. Two decisions inside that cleanup are worth naming, because both were tempting to do
  wrong: **"measured, not inferred" is evidence about a row, not a sixth status**, so the `objcopy -S` row is
  `SUPPORTED` with the measurement in its note; and **`READY_NOT_EXECUTED` / `UPDATE_READY_MANUAL` are release
  readiness states fixed by P5 §11/§12, not compatibility claims**, so they moved to a `state` column in a new
  §7b rather than being forced into a vocabulary that would have had to mean something they do not mean.
- **A new assertion on an existing fixture is not a new fixture.** The A–H table said F was "already covered"
  and H was "new and named", which is two different truths wearing one column. Under
  `PROVED_BY_EXISTING_FIXTURE / PROVED_BY_NEW_FIXTURE / SUPPORTED_WITH_LIMITS / NOT_AVAILABLE` the Architect
  mapped **A, F and H to existing** and **B, C, D, E and G to new**: every pre-Commit-E fixture was already a
  `-g` build, and `p0-dual-region` plus the `p2-diff` pair already carried 3, 5 and 4 `PT_LOAD` segments before
  this round counted them. H's evidence note now says plainly that what was new was the direct assertion, and
  `SUPPORTED_WITH_LIMITS` / `NOT_AVAILABLE` are left unused with the reason written next to them rather than
  invented a use for.
- **L15 is decided: Option E — legacy wire identifier preserved, accurate presentation / documentation.**
  `SourceType::ElfProgramHeader` and the `analysis:1` token `"elf.program-header"` stay exactly as they are,
  together with the stored `ElfProgramHeader` rows, the report DTO, the Bundle `analysis.json` contract, the
  goldens, `SCHEMA_VERSION`, release identity and `ADR-0028`. Refused: a new `ElfSectionFlags` member, an enum
  rename, a wire rename, rewritten history, **migration 0006**, `analysis:2`, and any golden byte — which is the
  price options B and C were written down to show. What replaces the accuracy the identifier never had is a
  definition: `elf.program-header` is a **legacy compatibility identifier** and carries no guarantee that the
  evidence came from a `PT_*` header, while the accurate human meaning of `MemoryEvidenceBasis::ElfAddressAndFlags`
  is **ELF address + flags evidence**. L15 becomes `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER` and §11 forbids
  marking it `CLOSED`, because preserving a name documents a risk instead of removing one. The rule this
  decision follows is the one that already decided ADR-0028: a published identity is bytes, and bytes already
  written are not edited to make a label look better.
- **Documenting an inaccurate name is allowed; changing what it points at is not.** The clarification lands in
  `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §7 — a dated section that says in terms that it changes no earlier
  clause, no evidence class and no precedence rule — because that document's §4 item 3 is the rule whose label
  is loose, and a reader of the model is where the confusion starts. `04_TECH/02_DOMAIN_MODEL.md`, which carries
  `EvidenceItem.source_type` and is `status: BASELINE`, was **not** edited: `AGENTS.md` 2 puts an artifact
  evidence classification change behind an ADR, and §1 of this prompt does not buy that permission.
- **One residue is visible to users and was deliberately left alone.** The Evidence Inspector renders the stored
  token verbatim (`Details.tsx:667` ← `details.rs:223` ← `query.rs:503` ← `db.rs:507`), so expanding such a row
  shows `ElfProgramHeader`, not the accurate phrase. §10 of the prompt says: if the UI still exposes the legacy
  wire token, do **not** fix code in a documentation-only round — record it for Commit F if user-visible. It is,
  so it is recorded, in the schema decision's §11, the supportability report's L15 row, and here.
- **The delivered prompt was CRLF and the archive is LF, and both hashes are in the record.** Delivered bytes:
  28,738 with 1,372 CRLF terminators, SHA-256 `9571df2cc08107ea134ed89acc4a984254803b40e451c57c6e9c3ca480075599`.
  Stored bytes: 27,366 with 1,372 LF, SHA-256 `62040e3d02fffe0f7682909828e4a2d6a91d81b09a937c0bd0b4765e6c7af887`.
  The difference is the terminator that `.gitattributes`' `*.txt text eol=lf` requires, which this round did not
  touch (`AGENTS.md` 9 makes a line-ending or integrity-policy change a human decision, and the other thirteen
  archived prompts all arrived LF). Content identity was proven, not assumed: `cp` + `cmp` at archive time, then
  a line-by-line split of delivered against stored — 1,373 lines each, `identical: True`. §3's rule against
  inventing a hash is why the record keeps the delivered hash too: the delivered CRLF hash does not reproduce
  from a checkout of this directory, and saying so is the honest version.
- **What this round may not do, stated as the boundary it held.** No product source, no fixture byte, no schema,
  no migration, no dependency, no capability, no workflow change; **L26 stays undecided** (§14: not blob bytes,
  not canonical checkout bytes, not the verifier wired into CI, not an `ADR-0028` edit), the §64 installed
  journey stays unrun (§15), no owner-store park/restore and no package installation, and Commit F's
  consolidation documents stay unwritten (§16) rather than created to check a filename off a list. The root
  `SHA256SUMS` was regenerated for ordinary bookkeeping under the current convention only, with the caveat
  retained and **no cross-checkout reproducibility claim**.
- **The only verdicts this round may write.** Before its own CI: Commit E engineering `PASS`, evidence closure
  `NORMALIZATION_PENDING_REMOTE_CI`. After that run reads 10 of 10: `COMMIT_E = FINAL PASS / COMPLETE`,
  `P5 = IN_PROGRESS`, product `MVP CANDIDATE`, baseline `0.6.0`, `Commit F = NOT_AUTHORIZED`, L15
  `CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER`, L26 `CARRIED_FORWARD TO COMMIT F / ARCHITECT`, §64
  `OPEN FOR COMMIT F`, licence `PENDING OWNER CONFIRMATION`. §27 closes the loop the way §32 and §59 did before
  it: this head is itself the final normalization successor, its run is external evidence, and no further commit
  is written merely to record it.
