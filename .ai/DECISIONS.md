---
title: "Decision Summary"
doc_id: "FS-AI-003"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-29"
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
