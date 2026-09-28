---
title: "P0 Implementation Log"
doc_id: "FS-P0-003"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Implementation Log

Each entry follows the shape `05_ENGINEERING` expects of a decision record: the problem, the
authority that already covers it, what was chosen, what was rejected, and the test that proves
it. Refactors that changed no boundary are not here.

---

## 1. Fixtures come from a cross compiler, not the host

- **Problem.** The P0 prompt assumes a host-compiled ELF. On this Windows host, the available
  native toolchain emits PE/COFF, so "just compile a fixture" produces nothing the parser can read.
- **Existing authority.** `04_TECH/16` requires a real artifact rather than a hand-written byte
  array; nothing requires the host compiler specifically.
- **Chosen implementation.** Build fixtures with `arm-none-eabi-gcc 14.3.1` (Arm GNU Toolchain
  14.3.Rel1), commit the resulting binaries, and record the exact toolchain, command line and
  linker script alongside them.
- **Alternatives considered.** Hand-crafting ELF bytes (rejected: it tests the parser against
  someone's guess at a linker); requiring a Linux CI image to build them (rejected: it makes the
  test suite depend on a compiler the product never ships with).
- **Scope impact.** None; tests read committed files.
- **Dependency impact.** None at build time. The ARM toolchain is a fixture-authoring dependency
  only, invoked by `scripts/gen_p0_fixtures.py` by hand.
- **Test evidence.** `crates/firmwaresight-artifact/tests/p0_acceptance.rs` verifies each
  fixture's SHA-256 against `fixtures/manifest.json` before trusting any assertion.
- **ADR required?** No - no boundary moved.

---

## 2. `SnapshotId` derivation, a gap in the baseline

- **Problem.** `04_TECH/02_DOMAIN_MODEL.md` requires a stable, content-derived snapshot id but
  never defines how it is formed.
- **Chosen implementation.** `snap-{artifact sha256}-{normalization version}[-{map sha256}]`, in
  `firmwaresight-core/src/domain/build_snapshot.rs`.
- **Alternatives considered.** A UUID per run (rejected: not stable); hashing the normalized
  snapshot (rejected for P0: it would add a second hash dependency to a dependency-free core
  crate for no gain over the already-cryptographic artifact digests).
- **Test evidence.** `snapshot_id_is_stable_for_the_same_input_bytes`,
  `snapshot_id_changes_when_any_input_changes` and
  `the_same_bytes_produce_the_same_snapshot_identically_twice`; the storage dedupe test
  `the_same_snapshot_is_never_stored_twice` depends on that stability.
- **ADR required?** No, but it is recorded here because the baseline was silent.

---

## 3. Section role classification stays in Core, not in `object`

- **Problem.** `object 0.40` reports `.debug_*` sections as `SectionKind::Other`, so a
  `match` on the upstream kind would have counted DWARF as device image.
- **Existing authority.** ADR-0021 excludes toolchain metadata from both budgets.
- **Chosen implementation.** `classify_by_name()` in the artifact crate maps names to the Core
  `SectionRole` enum; kind only informs flags and role where the name is inconclusive.
- **Test evidence.** `debug_sections_are_excluded_from_both_budgets` in `p0_acceptance.rs`;
  `excluded_metadata_bytes` is a visible field in the goldens.
- **ADR required?** No.

---

## 4. The RAM discriminator is write permission, not execute

- **Problem.** A region attribute string of `xrw` was being classified as "mixed", which made
  executable RAM regions (the normal STM32 case) unattributable.
- **Chosen implementation.** `RegionAttributes::parse` decides volatility by the presence of
  `write`: `rwx` → volatile RAM, `xr` → nonvolatile.
- **Alternatives considered.** Treating any multi-flag region as mixed (rejected: it silently
  dropped most real linker scripts).
- **Test evidence.** `region_attributes_separate_rom_from_ram` and `initialized_data_is_charged_to_both_budgets_from_load_evidence` in `domain/memory.rs`.

---

## 5. The evidence ladder must limit what a verdict may claim

- **Problem.** A name heuristic and a linker-declared region table produce the same integer, so
  nothing in the number distinguishes them.
- **Existing authority.** ADR-0021 and `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md`.
- **Chosen implementation.** `MemoryEvidenceBasis` carries five rungs with `rank()`,
  `classification()` and `may_support_hard_block()`. `MemoryFootprint::admissible_for_hard_block`
  is the only place the ceiling is read.
- **Test evidence.** `flags_only_mapping_cannot_support_a_hard_block`,
  `name_heuristic_alone_is_derived_and_offers_no_number`, plus the CLI/Desktop parity tests that assert `admissibleForHardBlock` is `false` for the MAP-less fixture and `true` for
  the MAP fixture built from the same source.
- **ADR required?** No - ADR-0021 already decided this; P0 implements it.

---

## 6. Nonvolatile backing resolves through the load address

- **Problem.** With a MAP present, `.data` was reported as unknown in the image budget because
  the runtime region was preferred for both budgets.
- **Chosen implementation.** `nonvolatile_from_regions(...)` prefers the region containing the
  load address; the runtime budget uses the region containing the virtual address.
- **Test evidence.** `initialized_data_is_charged_to_both_budgets_from_load_evidence` (core) and
  `data_is_dual_accounted_from_explicit_load_address_evidence` (acceptance, 4 bytes in each at
  `MapRegionAndElfLoad`), and the dual-region golden.

---

## 7. GNU ld's omitted load-address clause

- **Problem.** GNU ld prints `load address 0x...` only when it differs from the virtual address.
  Treating the omission as unknown would have made most sections unattributable.
- **Chosen implementation.** Absent clause ⇒ `lma == vma`, documented at
  `crates/firmwaresight-artifact/src/map.rs` in `apply_map_evidence`.
- **Test evidence.** `.text` and `.rodata` resolve to the ROM region in the dual-region golden.

---

## 8. A foreign MAP is refused, not re-parsed

- **Problem.** ArmClang and IAR maps look superficially similar; a permissive reader would emit
  plausible nonsense.
- **Chosen implementation.** `map::detect()` recognizes the GNU ld banner and returns
  `MapUnsupported` for the others, with no fallback parser and no degradation into a guess.
- **Test evidence.** `foreign_toolchain_maps_are_rejected_without_falling_back`.

---

## 9. ts-rs lives only at the Desktop DTO module

- **Problem.** ADR-0019 forbids Core domain types knowing about TypeScript; the prompt asks for
  generated bindings.
- **Chosen implementation.** `ts-rs` is a dependency of `apps/desktop/src-tauri` only. The
  desktop defines `AnalysisSummaryDto` by explicit mapping from the report crate's
  `AnalyzeResultDto`. An earlier draft put an optional `ipc-ts` feature on the report crate; that
  was removed, because a feature that exists only to serve one consumer belongs with that
  consumer.
- **Test evidence.** `cargo test -p firmwaresight-desktop` regenerates 10 `.ts` files;
  `scripts/check.py --only drift` fails if the committed files move.
- **ADR required?** No - it follows ADR-0019.

---

## 10. Optional IPC fields serialize as `null`

- **Problem.** `skip_serializing_if = "Option::is_none"` matches the CLI's clean JSON, but
  ts-rs emits `foo: string | null` either way, so the generated type would promise a key the
  payload omitted.
- **Chosen implementation.** The IPC DTOs keep the key with `null`. The CLI report DTO keeps
  omitting it, because its golden must stay free of null noise.
- **Test evidence.** `error_envelope_carries_code_operation_and_remediation` (CLI, asserts no
  `null`); `the_ipc_payload_is_bounded_and_carries_no_path` (desktop).

---

## 11. The desktop analyzes a fixture key, never a path

- **Problem.** A `analyze(path)` command is `read arbitrary file` with extra steps
  (AGENTS.md 7).
- **Chosen implementation.** `get_analysis_summary(fixture: FixtureKey)` where `FixtureKey` is a
  closed Rust enum, and `list_fixtures()` supplies the labels. Paths are resolved inside the
  shell.
- **Alternatives considered.** A dialog-plugin file picker (rejected for P0: it widens the
  capability surface and needs its own ADR); a sandboxed project root with path joining
  (rejected: traversal risk for no P0 benefit).
- **Test evidence.** `asks the shell to analyze a fixture key, never a path` (UI) and
  `the_ipc_payload_is_bounded_and_carries_no_path` (Rust).
- **ADR required?** No - it narrows the surface rather than moving a boundary.

---

## 12. TypeScript is pinned to 6.0.3, not the newest 7.0.2

- **Problem.** AGENTS.md 10 requires frontend lint. `typescript-eslint@8.70.1` declares its
  supported range as `>=4.8.4 <6.1.0`, which excludes TypeScript 7.
- **Chosen implementation.** Pin `typescript@6.0.3` exactly, the newest release inside the
  linter's supported range.
- **Alternatives considered.** TypeScript 7 with an unsupported parser (rejected: an
  unsupported parser is not a lint gate); dropping lint to keep 7 (rejected: AGENTS.md 10
  requires lint).
- **Dependency impact.** None beyond the pinned version.
- **Test evidence.** `pnpm typecheck`, `pnpm lint`, `pnpm test` all pass in the gate.
- **ADR required?** No, but a decision note is warranted if the baseline ever freezes a
  TypeScript major.

---

## 13. Tauri 2 stable, and no installer claim

- **Problem.** crates.io's newest `tauri` is `3.0.0-alpha.3`. AGENTS.md 2 freezes Tauri 2 and
  forbids a silent framework change.
- **Chosen implementation.** `tauri = 2.12.0`, `tauri-build = 2.7.0`, `@tauri-apps/api = 2.12.0`.
  `bundle.active = false`, because P0 validates the shell, not a signed installer.
- **Test evidence.** The workspace compiles and its tests run against that pin;
  `cargo tree -i tauri` shows one major.
- **ADR required?** Adopting Tauri 3 would be. Staying on the frozen major is not.

---

## 14. Error wording moved into the artifact crate

- **Problem.** The CLI owned a `user_message()` table. The desktop needed the same sentences,
  and a second copy is how one code starts meaning two things.
- **Chosen implementation.** `ArtifactError::user_message()` sits next to `stable_code()` and
  `remediation()`; both surfaces call it.
- **Test evidence.** `every_user_facing_error_maps_to_exit_three` and the desktop envelope tests.

---

## 15. Stage timings exist, but never in the payload

- **Problem.** `P0_PERFORMANCE_REPORT.md` must separate hash, read, parse and normalize cost, and
  wall-clock alone cannot do that.
- **Chosen implementation.** `Analysis { parse_ms, normalize_ms }` alongside the existing
  `GuardedInput { hash_ms, read_ms }`, printed only in the CLI's stderr diagnostics.
- **Test evidence.** The golden tests compare deterministic JSON; none of the timings appear in
  it. `scripts/measure_workloads.py` parses the diagnostics line.

---

## 16. `hex` and `sha2` avoided where the standard library already suffices

- **Problem.** The prompt's candidate dependency list includes `hex`; a lowercase-hex encoder is
  roughly ten lines.
- **Chosen implementation.** `lowercase_hex` written by hand in `intake.rs`. `Sha256` is
  validated by parsing 64 lowercase hex characters, so the encoder and the validator cannot
  disagree.
- **Dependency impact.** One fewer crate in a crate that touches every artifact path.
- **ADR required?** No.

---

## 17. Golden refresh and one deletion

- **Problem.** Two commits after generation, the `golden/core/*-memory.json` files still carried
  `fixture` and `sha256` keys the projection no longer emits, and
  `golden/reports/p0-basic-summary.json` held dual-region values under a p0-basic name with no
  test reading it.
- **Chosen implementation.** Regenerate the memory goldens from the release binary's own
  `analyze --json` output (`scripts/update_goldens.py --confirm`), and delete the unconsumed
  report golden, leaving `golden/reports/README.md` to state what must exist before a file can
  live there.
- **Alternatives considered.** Keeping the mis-named file (rejected: a golden nobody reads is a
  false assurance).
- **Test evidence.** `memory_accounting_golden_passes_on_both_fixtures` now consumes
  `golden/core/`, and `scripts/check.py --only drift` fails on any further divergence.
- **Note.** This is the only place P0 removed a file; it is recoverable from git history.

---

## 18. The gate lints the configuration that ships

- **Problem.** The desktop crate had no `custom-protocol` feature, so `cargo build --release`
  produced a dev-mode binary. `tauri`'s build script sets `dev = !custom_protocol`
  (`tauri-2.12.0/build.rs:253`), and dev-mode codegen embeds no frontend
  (`tauri-codegen-2.7.0/src/context.rs:178`). Measured: the release binary was 10,330,112 bytes and
  did not contain the built asset name `index-BdwhVhyx.js`, although this pack had described it as
  "the built UI embedded".
- **Existing authority.** The upstream Tauri 2 template declares exactly this feature, and
  `tauri build` passes it. Nothing new was invented; the template line was simply missing here.
- **Chosen implementation.** `[features] custom-protocol = ["tauri/custom-protocol"]` in
  `apps/desktop/src-tauri/Cargo.toml`. The gate's `clippy --all-targets --all-features` now compiles
  the shipping configuration, which is also the configuration that requires `apps/desktop/ui/dist/`
  to exist - with `dist/` removed it fails inside the macro with `The `frontendDist` configuration is
  set to "../ui/dist" but this path doesn't exist`. `scripts/check.py` therefore builds the UI when
  `dist/index.html` is missing, and the CI `rust` job gained a Node step for that reason alone.
- **Alternatives considered.** (a) Leave the feature out and keep the gate dist-free - rejected: the
  gate would then never compile the configuration that ships, and the prompt's "minimal Tauri summary
  builds" would describe a binary that cannot display without a dev server. (b) Drop `--all-features`
  from clippy to avoid the dependency - rejected: that weakens a check to dodge a build-order fact.
  (c) Commit `dist/` - rejected: it is generated output, and `pnpm build` already reproduces it
  byte-for-byte (`diff -r` on two builds was empty).
- **Scope impact.** None outward-facing; P0 still does not bundle or launch a window.
- **Dependency impact.** Zero new crates: `custom-protocol` is a feature of the `tauri` dependency
  already in the tree.
- **Test evidence.** `cargo build --release -p firmwaresight-desktop --features custom-protocol` ->
  10,398,208 bytes, contains `index-BdwhVhyx.js`; the same command without the feature ->
  10,330,112 bytes, no such string. `python scripts/check.py --only rust` after `rm -rf
  apps/desktop/ui/dist` -> `PASS rust/frontend assets (install)`, `PASS rust/frontend assets (build)`,
  `PASS rust/fmt`, `PASS rust/clippy`, `PASS rust/test` (102 tests). When `dist/` exists the guard
  records no steps, so the documented 14-step gate is unchanged on a warm tree.
- **ADR required?** No - no boundary, dependency, schema or security surface changed. Recorded here
  because it corrected a claim already written into this pack.

---

## 19. The design checklist was run at delivery, and it found four things

- **Problem.** `AGENTS.md` 11 requires every UI change to pass
  `templates/DESIGN_CHECKLIST_TEMPLATE.md`, but no filled copy existed for the P0 screen. Writing one
  after the fact is either theatre or a surprise; this time it was a surprise.
- **Chosen implementation.** Filled it against the tree with the deciding command next to each ⚙
  box: `P0_DESIGN_CHECKLIST.md`. 10 of 11 Don'ts and 24 of 25 Do's are ticked; two findings were
  repaired in source and two are recorded open with the reason beside each.
- **Findings.** (a) `DESIGN.md` 3 enumerates the value classes that must be tokenised and border width
  is not among them, while `DESIGN.md` 4 requires hairline borders - so `1px` × 8 is a gap in the
  design contract, not a component that ignored a token. (b) No live region: the "Analyzing…" line was
  a plain `<p>`, so a screen reader never heard the request start or finish. (c) The `select` styled
  only `:hover` and stayed usable mid-request, which let the fixture change under an in-flight
  analysis. (d) Capability badges display Core's serialized enum words ("supported", "available",
  "not-provided"), which is API vocabulary rather than product copy.
- **Chosen response.** (b) and (c) were fixed: the line is now
  `role="status" aria-live="polite"`, the `select` is disabled while loading and styled for it, and a
  test holds the IPC promise open to assert both controls' `disabled` and the status role. (a) is left
  open because the only correct fix is a `design-tokens.json` version bump, which changes a frozen
  design asset. (d) is left open as a boundary question.
- **Alternatives considered.** Ticking all boxes and moving on (rejected: two of the four are visible
  to a user who cannot see the screen). Filing (b) and (c) as P1 work without fixing them (rejected for
  the accessibility half: an unannounced asynchronous state is not a polish item, and it was a
  four-markup-line change with a test). Doing (d) by adding a copy layer in the view (rejected:
  `AGENTS.md` 11 and the parity tests forbid the UI from renaming facts, and the CLI prints the same
  strings, so a copy layer is precisely where the two surfaces would start to disagree). Deferring
  everything because the gate was already green (rejected: a green gate says nothing about markup it
  never inspects).
- **Scope impact.** Two small UI behaviour changes, both inside the already-authorized summary screen:
  one live region, one disabled control. No contract, DTO, schema or boundary changed; the Rust side is
  byte-for-byte untouched, which `git diff --name-only` over the delivery commits confirms.
- **Dependency impact.** None.
- **Test evidence.** Every claim in the checklist names the grep or test that decided it.
  `it('announces the in-flight state and locks both controls until it resolves')` covers (b) and (c);
  `grep -rnoE "#[0-9a-fA-F]{6}" src/ | grep -v tokens.css` and
  `grep -rn "box-shadow|text-shadow|filter:|drop-shadow" src/` both return nothing; `tokens.css` is
  generated from the frozen asset and drift-checked by the gate. After the change the whole gate ran
  again: 14/14 steps, 102 Rust tests, 19 UI tests.
- **ADR required?** No. The border-width token bump would need a design-asset change request, which is
  why it is left open rather than performed here.

---

## 20. The pnpm pin was one patch below the frozen baseline

- **Problem.** `apps/desktop/ui/package.json` read `packageManager: "pnpm@12.6.0"`. The frozen
  `examples/package.baseline.json` names `pnpm@12.7.0`, and `04_TECH/10_TOOLCHAIN_BASELINE.md` records
  12.7.0 as the research-date current of the pinned 12 line.
- **Existing authority.** The baseline states the exact version; a `packageManager` field is the
  repository's own pin, so it must match unless an ADR says otherwise.
- **Chosen implementation.** Pin 12.7.0 and re-verify. `corepack pnpm install --frozen-lockfile`
  refused first - `ERR_PNPM_FROZEN_LOCKFILE_WITH_OUTDATED_LOCKFILE` - because pnpm records the version
  that wrote the lockfile. A single `corepack pnpm install` updated that metadata, and the resulting
  lockfile diff is the `pnpm`/`@pnpm/exe` platform entries only: no application dependency moved.
- **Alternatives considered.** Leaving 12.6.0 because it builds and tests identically (rejected: that
  is how a silent baseline drift becomes permanent - nothing would ever fail loudly enough to force the
  question). Adding a root `package.json` and `pnpm-workspace.yaml` to match the repo-structure sketch
  (rejected for P0 and recorded as a named simplification: with one frontend package, `pnpm -r` adds
  nothing, and the structure doc's own rule - "Frontend is one workspace package at MVP" - is met).
- **Scope impact.** None outward-facing.
- **Dependency impact.** One toolchain patch version, from the frozen asset rather than from choice.
- **Test evidence.** `corepack pnpm --version` -> 12.7.0; frozen install exit 0; the whole
  `frontend` group re-run green afterwards.
- **ADR required?** No - it restores the frozen value rather than changing it.

---

# The CI closure round (entries 21-25)

Remote CI Run `36360310447` on `f9b8ccb` failed four jobs and the frozen baseline required a fifth
thing the workflow never had. Each cause is reproduced and fixed in
`P0_CI_REMEDIATION_REPORT.md`; only the decisions that could be made wrongly are recorded here.

## 21. Fixture byte identity was moved into the Git text policy, not into the test

- **Problem.** `committed_fixtures_match_their_recorded_hashes` failed on `windows-latest`: the
  checkout yielded CRLF where the manifest recorded the LF blob's hash.
- **Existing authority.** `fixtures/manifest.json`'s own purpose is to verify the byte identity of
  committed evidence; `AGENTS.md` 8 forbids weakening an evidence claim.
- **Chosen implementation.** `.gitattributes` classifies every tracked text type explicitly
  (`text eol=lf`), marks linker maps `-text` so Git never touches them, and leaves the binaries
  `binary`. One manifest value moved to the blob's own SHA-256, because the previous value had
  recorded a CRLF working copy.
- **Alternatives considered.** Normalizing newlines before hashing (rejected: the manifest would then
  verify "some bytes resembling the fixture"); deleting the `.ld` from the manifest (rejected: it is
  the input that produces the linker's layout claim); changing the expected hash to the CRLF value
  (rejected outright by the prompt - it would make the assertion platform-dependent).
- **Test evidence.** Two fresh clones, `core.autocrlf=true` and `false`, both match 10/10;
  `git check-attr` shows `text: set, eol: lf` for the `.ld` and `text: unset` for the `.map`.
- **ADR required?** No - it makes a hash assertion mean what it already claimed.

## 22. Icon drift compares decoded pixels instead of encoder bytes

- **Problem.** `drift/desktop icons` failed on Linux with `Pillow==12.3.0` installed from the pinned
  requirements, so this was not a version difference: the check compared bytes of a compressed stream.
- **Existing authority.** `05_ENGINEERING` requires drift checks to detect a real change; `DESIGN.md`
  and `assets/design-tokens.json` are frozen, so the icons themselves are not negotiable here.
- **Chosen implementation.** Decode both sides: PNG dimensions plus RGBA buffer; ICO the required size
  set plus per-frame RGBA, with a missing or unexpected frame failing.
- **Alternatives considered.** Commit the Linux-regenerated set (rejected as the prompt's forbidden
  default - it moves the artifact to satisfy the checker); a project-owned byte-deterministic encoder
  (rejected: a heavy dependency or a hand-written container writer for a guarantee nobody asked for,
  and cross-platform SHA equality cannot be proven from one machine).
- **Test evidence.** Three conditions against the committed set: untouched -> exit 0; identical pixels
  with all five files holding different bytes -> exit 0; one repainted pixel and one dropped ICO frame
  -> exit 1 naming both.
- **ADR required?** No. The lost guarantee - encoder-byte stability - is recorded in
  `P0_KNOWN_LIMITATIONS.md` as a deliberate trade.

## 23. `deny.toml` was rewritten to the schema the tool actually has

- **Problem.** cargo-deny 0.20.2 refused to parse the file: `severity-threshold` and
  `highlight-warnings` are not keys it accepts, and five `licenses.allow` values were slash strings in
  a field parsed as SPDX.
- **Existing authority.** `AGENTS.md` 2 and 4 freeze the dependency red lines; the prompt requires the
  tool's real schema over an impression of it.
- **Chosen implementation.** Keys confirmed from `cargo deny init` and from the vendored
  `cargo-deny-0.20.2` `Config` deserializers. `[graph] targets` names the four shipping platforms from
  `04_TECH/20`, so `tauri`'s Android/iOS-only `reqwest` edge is outside the evaluated graph and the ban
  stays absolute rather than being exempted. `anyhow` keeps its ban with
  `wrappers = ["tauri", "tauri-build", "tauri-utils"]`, so first-party use still fails.
- **Alternatives considered.** Dropping the advisories check or diluting it to informational-only
  (rejected); renaming crates out of `deny` (rejected); `skip`-ing the 23 duplicate-version warnings
  (rejected - they are warnings, and silencing them deletes the evidence that the Tauri tree is the
  reason they exist).
- **Test evidence.** `cargo deny check licenses bans sources advisories` -> exit 0, four `ok`. The
  allow list is the output of `cargo deny list -t 1.0 --layout license`, and removing `ISC` plus the
  five slash strings left it `ok`, which is what proves those entries asserted nothing.
- **ADR required?** For the two unresolvable advisories, yes - and it is not this round's to write.
  Reported as an architecture conflict.

## 24. The macOS core smoke calls the same gate script

- **Problem.** `05_ENGINEERING/06_CI_CD_BASELINE.md` requires a core smoke on `main` push; no macOS job
  existed, so six green jobs would still not have met the baseline.
- **Chosen implementation.** A `core-smoke` group in `scripts/check.py` selecting the four library
  crates and the CLI by package name, and a `macos-core` job that runs
  `python scripts/check.py --only core-smoke`.
- **Alternatives considered.** A fifth crate containing macOS-only tests (rejected by the prompt and by
  the Phase-0 crate budget); a hand-written command list in the workflow (rejected: a second place for
  the gate to disagree with itself); running the full workspace on macOS (rejected - the desktop crate
  needs a platform's WebView stack, and its compile boundary is proven on the two OSes that ship it).
- **Test evidence.** `--only core-smoke` -> 3/3, 87 of the 104 Rust tests; `--list` shows the group; it
  is not in the default set, so `cargo test --workspace` is not run twice.
- **ADR required?** No - it adds coverage the baseline already required.

## 25. Evidence is keyed by build, because the window found the key was wrong

- **Problem.** The second artifact in one session returned `ERR-STORAGE-4006` /
  `UNIQUE constraint failed: evidence.id`. 102 Rust tests passed with the defect present because each
  storage test uses a fresh database.
- **Existing authority.** `04_TECH/15` §4 states `Build 1─N Evidence`; `AGENTS.md` 6 requires every
  schema change to go through a migration.
- **Chosen implementation.** Migration `0002_evidence_keyed_by_build.sql` rebuilds the table on
  `PRIMARY KEY (build_id, id)`, copies the rows and recreates both indexes; `migrate()` applies an
  ordered list, each migration in its own transaction; `SCHEMA_VERSION` is 2. A SQLite primary key
  cannot be `ALTER`ed, so the copy-and-rename shape is not a style choice.
- **Alternatives considered.** Making evidence identifiers globally unique by namespacing them per
  build (rejected: it changes the meaning of a stored fact to dodge a constraint); deleting and
  recreating the user's database (rejected: it is user data); no migration at all because "there is no
  GA database to upgrade" (rejected - there was one on this machine, in `%APPDATA%`, with a build in
  it).
- **Test evidence.** `two_builds_may_record_the_same_evidence_identifier` and
  `a_version_one_database_is_upgraded_without_losing_its_evidence` failed first with the window's own
  message, then passed; storage is 9 -> 11 tests, workspace 104. The real user-profile database was
  upgraded in place to version 2 with both builds and 21 evidence rows present.
- **ADR required?** Not for the fix: it restores the relation the frozen data model already declares.
  Flagged in `.ai/DECISIONS.md` so the architect confirms that reading rather than inheriting it.
