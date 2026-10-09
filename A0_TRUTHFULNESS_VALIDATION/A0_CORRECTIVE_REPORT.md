---
title: "A0 Truthfulness Corrective Report"
doc_id: "FS-A0-CORRECTIVE-001"
product: "FirmwareSight"
version: "0.6.0"
status: "REPORT"
owner: "Engineering"
last_updated: "2026-10-09"
---

# A0_CORRECTIVE_REPORT

Unit `A0_PRODUCT_TRUTHFULNESS_CORRECTIVE`, nature `BOUNDED PRODUCT CORRECTIVE / TESTED / NO SCOPE EXPANSION`.
Authority: 《A0 Truthfulness Corrective｜产品真实性与误导性表述修复｜Coding Agent 执行授权 v1.0》, dated
2026-10-09, delivered as a file and archived verbatim at
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_A0_Truthfulness_Corrective_v1.0.txt` — 18,845 bytes, 252 lines, 0 `CR`,
LF-only, SHA-256 `660dae54c5d1ed3f892c4f927c353414b4b057ce78813d5ce32516f8c42f42eb`, measured identically on the
source file in the owner's Downloads folder and on the archived copy, so the archived bytes are the delivered bytes
and not a retranscription.

Start state: HEAD `19dcf889624fab172095e4de9e6b33ad53141f3c` = `origin/main`, worktree clean.

Final state: `A0_PRODUCT_TRUTHFULNESS_CORRECTIVE = COMPLETE`. All four subitems shipped; none was blocked.

---

## 1. What the round was for

The owner's premise for this unit is that FirmwareSight still promised things it does not do. Four such promises
were named, and §二 of the authorization opened exactly these four and nothing else. §七 listed eleven exclusions,
and the whole round stayed inside them: no new schema, table, field, persistence model, object-level diff,
attribution UI, verify command, bundle-manifest change, BIN/HEX parser, timeline, trend, adapter, theme, dependency,
toolchain change, or touch to the V1 cohort baseline, protocol, milestones, participants or metrics.

## 2. Preflight: does correcting the lie move an identity?

§三 attaches a STOP clause: if turning a false declaration into a true one unexpectedly moves a Gate verdict, a
Snapshot ID, a Release ID or committed evidence semantics, stop and report instead of continuing. This was answered
before any edit, by reading every composition function rather than by trusting that they were unaffected:

| Identity | Composition read | Capabilities in it? |
| --- | --- | --- |
| `SnapshotId` | `crates/firmwaresight-core/src/domain/build_snapshot.rs:34-41` — `compose(artifact_sha256, map_sha256)` | no |
| Gate `run_id` | `crates/firmwaresight-project/src/fingerprint.rs:34-39` — `"gate-" + sha256(canonical_input())`, canonical text at `domain/gate.rs:676-729` | no |
| Release id | `crates/firmwaresight-core/src/domain/release.rs:334-385` — `canonical_release_text()` | no |
| Evidence semantics | `crates/firmwaresight-core/src/domain/capability.rs:93` `best_supportable_evidence_class()` has **no production caller**; the only other reference is the `#[cfg(test)]` assertion at `capability.rs:171` | n/a |

The fourth row mattered most: `object_attribution != Available` is one of that function's three degradation tests, so
a naive reading said the correction could demote an evidence class somewhere. It cannot, because nothing in the
shipped product calls it.

Two independent corroboration events later confirmed the read (§4): `gate-results.json` came back byte-identical
from the golden regeneration, and the release manifest's `release.id`, `build.snapshot_id` and
`extensions.gate_run_id` are the same strings before and after. **The STOP clause was not triggered.**

## 3. A0-01 — Object Attribution capability declaration (GAP-01) — `FIXED`

**Before.** `crates/firmwaresight-artifact/src/pipeline.rs` `build_capabilities()` decided the MAP provision and the
object-attribution availability from one and the same condition, `map_evidence.is_some()`. Supplying a GNU ld MAP
therefore made the product declare `objectAttribution: available` — that object- or module-level results can be read
back out of the snapshot. They cannot. `map::parse` collects `MapObjectContribution` rows
(`crates/firmwaresight-artifact/src/map.rs`), nothing carries them into the snapshot, the persisted
sections/symbols rows have no object column (`crates/firmwaresight-storage/migrations/0001_initial.sql`), and
`domain::diff::ObjectAttribution` already told Compare the truth for the same build. The declaration contradicted the
diff layer in the same process.

**After.** The provision stays honest about the input it received and the availability states the result it does not
have: `Provision::Provided` with `Availability::Unavailable` on the MAP path, unchanged elsewhere. The field's own
doc comment in `crates/firmwaresight-core/src/domain/capability.rs:43-46` was rewritten from wording that invited the
misreading to wording that forbids it: `Available` now says plainly that it claims object or module results can be
read back out, so it requires a persisted attribution record, and that supplying a MAP or carrying debug sections is
a precondition and never the result.

**User-visible change.** Analyze's capability row "Object/module attribution" reads N/A instead of Pass once a MAP
is attached. The Release Bundle's §9 "Known capability limits" table gained the row it had always been written to
produce and never could: with the value now `unavailable`, the branch at
`crates/firmwaresight-report/src/release_render.rs:876-879` became reachable, and
`golden/reports/p4-release/release-report.html:455` now carries

> Totals are reported per section and per symbol only. No per-object-file or per-module total is claimed here, and
> none should be inferred from the section rows.

That sentence existed in the shipped renderer before this round and was dead code, because the flag that reaches it
was never set. Nothing was invented to fill the table; a real limit became visible in it.

**Regression.** `crates/firmwaresight-artifact/tests/p0_acceptance.rs:298-319`
`a_map_supplies_region_evidence_without_becoming_an_object_attribution_claim`. RED first, on unchanged product code:
`assertion left: Available, right: Unavailable` (`01_red/A0-01_p0_acceptance_RED.txt`). It also pins what the MAP
still genuinely does — sections and symbols available, `weakest_basis == MapRegionAndElfLoad`, footprint admissible
for a hard block — so the correction cannot be smuggled in as a withdrawal of the MAP's real contribution. The
MAP-less half of §九's item 8 was already covered and still holds at `p0_acceptance.rs:276-295`.

## 4. A0-02 — Unsupported-format remediation copy (GAP-06, copy half) — `FIXED`

**Before.** `crates/firmwaresight-artifact/src/error.rs:86-89` (`remediation()` `Self::UnsupportedFormat`) told a refused user: *"Provide an ELF linker output, or a
BIN/HEX image."* The entry point refuses BIN and Intel HEX — `intake.rs` detects them and answers
`UnsupportedFormat` — so the next step named a second rejection.

**After.** One sentence, from one place: *"Choose the ELF linker output this build produced; BIN and Intel HEX images
are not analyzed by this entry point."*

**Cross-surface.** `apps/desktop/src-tauri/src/ipc.rs` hands the artifact layer's own string to the desktop, and the
desktop test now pins the exact bytes instead of merely asserting that *some* remediation exists
(`apps/desktop/src-tauri/tests/real_artifact_intake.rs:599-605`). Two copies of that sentence would drift the way
the two verdict sentences once did; the pinned equality is what keeps one source.

**Regression.** `crates/firmwaresight-artifact/tests/p0_acceptance.rs:401-421`
`the_unsupported_format_next_step_offers_only_what_the_pipeline_accepts` feeds a real committed non-ELF input (the
dual-region MAP, which the pipeline genuinely refuses) and pins `ERR-FORMAT-0001`, the requirement that the next step
names the input that works, and the requirement that it states the limit. It deliberately does **not** assert
"must not contain BIN": the honest sentence does contain that word. The discriminating property is that the word now
sits in a refusal instead of an offer.

**Fail-before method, stated rather than implied:** the fix and the test were written in one step, so the RED run was
produced by restoring the pre-A0 string in place — one arm of `remediation()` swapped back to
`"Provide an ELF linker output, or a BIN/HEX image."` — running only that test, then putting the fixed file back. The
test fails at `p0_acceptance.rs:417` with *"the next step must state the limit instead of offering another rejected
format: Provide an ELF linker output, or a BIN/HEX image."*, and `error.rs` was re-read byte-identical after the cycle
(sha256 `ea9f3a461fd8d51e44c17f1c08c6624d2eee77920d2040860a63cc3f7f006ad7`). Evidence:
`01_red/A0-02_unsupported_format_RED.txt`, with the fixed file preserved beside it as `error.rs.fixed.bak`.

**Boundary kept.** §四 records that ADR-0006, the PRD, the MVP cohort documents and the current Compatibility Matrix
disagree about BIN/HEX scope, and forbids resolving that disagreement here. No BIN/HEX parsing was implemented,
ADR-0006 was not rewritten, and the conflict is **not** closed: it stays a D07/Owner decision. What was fixed is only
the sentence a user actually sees today.

## 5. A0-03 — Compare ranking-vs-total basis disclosure (GAP-09) — `FIXED`

**State found.** The page already told each list its own basis — `Compare.tsx:957-959` "Changed rows only, ranked by
the runtime-size delta", `:978-980` "Added rows, ranked by their runtime size in the target build" — and the tables
carry both `fileSize` and `memorySize`. What it never said was how the two lists relate to the headline
"Nonvolatile / load image" delta. A reader summing the visible rows gets a number that cannot land, and cannot tell
whether that is a basis difference or a truncation. So this is `FIXED`, not `NO_CHANGE_WITH_EVIDENCE`: the missing
fact was the relationship, not either basis.

**Change.** One sentence appended to the existing hint in the ranking area (`Compare.tsx:950-956`):

> Neither list sums to the "Nonvolatile / load image" delta above: that is the whole image's change, while these are
> only the top few rows ranked by runtime size, so a shortfall is a basis difference and a cut-off, not a missing row.

No new computation, no reconciliation figure, no chart, no card, no re-flow, and no second copy of an explanation
that already existed. `MAX_TOP_SECTIONS = 5` and `MAX_TOP_SYMBOLS = 10` (`apps/desktop/src-tauri/src/compare.rs:51-52`)
are the cut-off the sentence names, and they were read from the source rather than from a mockup.

**Regression.** `apps/desktop/ui/src/compare.test.tsx:1092-1108`, inside the existing
`describe('the ranking as a way to the full list')`, asserts inside the `Top growth and largest additions` region that
the disclosure exists and names the headline, both bases, and the "not a missing row" consequence. RED on unchanged
product code: `TestingLibraryElementError: Unable to find an element with the text: /load image[\s\S]*delta/i`
(`01_red/A0-03_compare_test_RED.txt`). GREEN with the sentence: `02_green/A0-03_compare_test_GREEN.txt`, and the whole
Compare file passes 58/58.

## 6. A0-04 — Overview real rule-ID regression coverage — `FIXED` (coverage; no product-code change)

**State found.** The canonical rule identities are `crates/firmwaresight-core/src/domain/gate.rs:172-181`'s
`as_str()` — `git.clean`, `release.commit_matches_expected`, `release.version_matches_policy`, `artifacts.required`,
`artifacts.hashes`, `memory.flash_budget`, `memory.ram_budget`, `diff.growth`, `release.notes`,
`evidence.unknown_review`. Every rule-id string `overview.test.tsx` had ever asserted is a mockup name
(`signed_image.required`, `flash_budget.soft`, `entry_point.valid`, `sbom.present`). That is legitimate for a page
whose only duty is to show `finding.ruleId` verbatim — and it is also why nothing in that file would have noticed if
the line stopped carrying the id at all.

**Change.** Three tests in a new describe, `apps/desktop/ui/src/overview.test.tsx:1135` (`A0 the open findings are
named by the rules the run evaluated`):
1. `memory.flash_budget` threaded from the DTO input to `within(ship).getByText('memory.flash_budget')`, with the
   row's own summary pinned in the same `<li>` and the aggregate sentence `Blocked — 1 rule(s) failed.` agreeing
   with the count of that run; a `release.notes` row in `N/A` stays out of the open list.
2. Five **canonical** open ids with the four-row cap: the four shown are named by their real ids and the fifth is
   counted and sent to the page that can open it (`1 more finding(s) on the Release Gate page.`) rather than dropped.
3. The Evidence card quotes the analyzed build's own class breakdown (`212`, `180 observed · 24 derived · 4 declared
   · 4 unknown`), and the region that states a verdict quotes no `evidence:ev-` locator it cannot resolve — the
   locator a real run carries (`footprint_evidence_id` at `crates/firmwaresight-project/src/evidence.rs:144`, wrapped
   by the `evidence:` scheme at `:190`) is input to that assertion, so it proves a boundary instead of restating one.

**Preserved.** §六 warns against blindly rewriting negative tests. The mockup-name tests were kept untouched:
`overview.test.tsx:361-381` still proves that a passing rule and an inapplicable rule are not open findings, and the
free display strings elsewhere remain valid tests of a presentation layer that must show whatever id Core sends. The
only fixture value changed in this file was `objectAttribution: 'partial'` → `'unavailable'`, which is not asserted
anywhere; it was aligned so that no fixture depicts a state A0-01 established is unreachable. All four UI capability
fixtures now read `unavailable`, matching production.

**Red→green.** A0-04 adds coverage, not behaviour, so its "fails before" is a mutation proof rather than a false
start: three mutations of `Overview.tsx` (hardcode the badge label to a mockup name; remove the four-row cap;
hardcode the Evidence card's breakdown) killed 2, 1 and 1 of the new tests respectively, each run against the mutated
file and each restored byte-for-byte
(`01_red/A0-04_mutation_summary.txt`, logs `A0-04_M1..M3.txt`, digest `142a847b…2e5991` re-read equal after the
cycle). No `reset`, `clean` or `checkout` was used, and the file was clean in `git status` before it began.

## 7. Goldens, contracts and what did not move

Five goldens were regenerated by the only sanctioned path, `python scripts/update_goldens.py`, whose dry run prints a
semantic diff and writes nothing (`01_red/A0-01_goldens_dryrun.txt`), then `--confirm`. No golden was edited by hand
and no failing test was allowed to rewrite its own expectation.

| Golden | Result |
| --- | --- |
| `golden/cli/p0-dual-region-analyze.json` | one leaf: `capabilities.objectAttribution: available → unavailable` |
| `golden/reports/p4-release/analysis.json` | the same one leaf; 26,961 → 26,963 bytes |
| `golden/reports/p4-release/release-report.html` | the value cell, the analysis row, and the newly reachable §9 limit row; 26,961 → 27,201 bytes |
| `golden/reports/p4-release/SHA256SUMS` | the two rows above, re-hashed by the shipped binary |
| `golden/reports/p4-release/release-manifest.json` | the same two file digests and the SHA256SUMS row it covers |

Unchanged, which is the point: `gate-results.json`, `diff.json`, `accepted-reviews.json`, `release-notes.md`,
`p0-basic-analyze.json`, `p0-dual-region-memory.json`, `p2-diff.json`, `p2-diff.html`.

In the manifest, byte-identical across the correction: `release.id
release-e400a8ac51a57d34e309d254536f8850044ef4dd53e26245045c24fdc86fdcc5`, `build.snapshot_id snap-4b4087e1…`,
`extensions.gate_run_id gate-a17e9861…`, `git_commit 585dafd3…`, `policy_sha256 b87aa057…`. The release identity of
the golden bundle did not move, and an unchanged `gate-results.json` is the empirical half of the §3 preflight.

**Public contracts.** Not touched. No portable schema (`urn:firmwaresight:schema:analysis:1` etc.) changed version;
no field was added, removed or renamed; the IPC bindings regenerated identical, which is exactly what the
`drift ipc bindings` step proves. The change is to a *value* the existing `objectAttribution` field reports, and to
one user-facing string. `04_TECH/26_PORTABLE_SCHEMA_POLICY.md:29-31` requires a schema major for a semantics change
and contract tests for an additive optional field; neither applies, because the field's documented meaning was already
"can object-level results be read back" — the product was reporting it wrongly, which is a defect, not a redesign.

## 8. Validation

Each subitem's red→green evidence is in §3–§6 above and archived outside the repository in
`C:/Users/16429/AppData/Local/Temp/FirmwareSight-A0-20261009/` (`00_authority/ 01_red/ 02_green/ 03_gate/ 04_ci/`).

- Rust: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  clean; the gate's `rust/test` step (`cargo test --workspace`) **870 passed / 0 failed** across 47 `test result:` lines
  — 41 test-binary lines and 6 doc-test lines, the same 47 the pre-A0 rounds cited.
  Up from 868: the two new `p0_acceptance` tests. An ad-hoc standalone re-run of the same tree on this host printed 49
  such lines, because that invocation emits `firmwaresight_report`'s and `firmwaresight_storage`'s doc-test targets
  twice; its total was the identical 870, and the archived gate log is the citation.
- Desktop parity and the real-intake cross-surface suite pass; `p0_acceptance` is 22/22.
- UI: `pnpm install --frozen-lockfile`, `typecheck`, `lint`, `test`, `build` all green. **295 tests in 9 files**, up
  from 291: one from A0-03, three from A0-04.
- `python scripts/check.py`: **17 of 17 steps passed, no `SKIP`, no `FAIL`** — `rust/fmt`, `rust/clippy`,
  `rust/test`, `frontend/install`, `frontend/typecheck`, `frontend/lint`, `frontend/test`, `frontend/build`,
  `drift/design tokens`, `drift/desktop icons`, `drift/baseline integrity`, `drift/ipc bindings`,
  `drift/ipc bindings unchanged`, `drift/fixtures tracked`, `drift/version identity`, `drift/goldens unchanged`,
  `deny/cargo-deny` (`advisories ok, bans ok, licenses ok, sources ok`). Full log:
  `03_gate/check_py_17_steps_FINAL.txt` — the run taken at the final staged state, after the last entry-document edit
  and the last ADR-0029 regeneration; the earlier run is kept beside it as `check_py_17_steps_prev.txt`, and the
  verifier output as `verify_baseline_final.txt`. That log deliberately carries no digest inside this report: the run
  hashes the index that contains the file quoting it, so any number written here would be stale the moment the run
  finished. Its totals are stable and are quoted here: 47 `test result:` lines summing to 870,
  `Test Files 9 passed (9) / Tests 295 passed (295)`, `p0_acceptance` 22.
  `drift/goldens unchanged` and `drift/ipc bindings unchanged` passing over the
  regenerated goldens is the proof that the committed goldens now equal what the binary emits and that no contract
  moved.
- `git diff --check`: no whitespace errors. Two line-ending drifts were caught by the audit and corrected in
  place: `Compare.tsx` had been rewritten with CRLF by the editor while `.gitattributes` pins `*.tsx text eol=lf`, and
  `.ai/README.md` entered the change set as a CRLF working copy because it had never been staged this round. Both
  working copies are LF again and each committed diff is the intended lines only. The same audit first found
  `BASELINE.yaml` staged with CRLF — a 2,749-line whole-file rewrite masquerading
  as a 77-line addition — and normalized it to LF, which is what `.gitattributes` `*.yaml text eol=lf` prescribes and
  what every other staged blob in this commit already was. All twenty-six staged blobs then measured 0 `CR`.
- `AGENTS.md` 11 binds a UI change to the design checklist, so the two surfaces A0 touched are answered in
  `A0_TRUTHFULNESS_VALIDATION/A0_DESIGN_CHECKLIST.md`: no Don't is hit (nothing visual was added — the Compare change
  is prose inside an existing `hint` element and the Analyze change is a value the same `CapabilityRow` already
  rendered), and the Do's it implements are "关键数字带上下文" (A0-03) and "能力 banner 如实反映证据状态，未夸大"
  (A0-01). Placement and line length at 1024/1440 are **not** covered here, because §五 forbade re-flowing the page and
  this round installed nothing.
- Changed-path audit: every path in the diff is A0-scoped (§9).
- §九 item 8's four user-visible cases: MAP provided (A0-01 test), MAP missing (`p0_acceptance.rs:276-295`),
  unsupported input (A0-02 test), real rule id (A0-04 tests), two size bases (A0-03 test).

**NOT_RUNTIME_VERIFIED.** This round installed nothing and ran no desktop binary. No runtime screenshot is claimed,
no V1 session exists, and no acceptance verdict is self-issued. The UI facts above come from component tests against
the shipped components, which is a different claim from a human having used the installed app.

## 9. Changed paths

**Twenty-six paths, every one inside the authorization's four subitems or its closeout duties.**

| Group | Paths |
| --- | --- |
| Product source (4) | `crates/firmwaresight-artifact/src/pipeline.rs`, `crates/firmwaresight-artifact/src/error.rs`, `crates/firmwaresight-core/src/domain/capability.rs`, `apps/desktop/ui/src/Compare.tsx` |
| Tests (5) | `crates/firmwaresight-artifact/tests/p0_acceptance.rs`, `apps/desktop/src-tauri/tests/real_artifact_intake.rs`, `apps/desktop/ui/src/compare.test.tsx`, `apps/desktop/ui/src/intake.test.tsx`, `apps/desktop/ui/src/overview.test.tsx` |
| Goldens (5) | §7's table |
| Governance (10) | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_A0_Truthfulness_Corrective_v1.0.txt` (new), `10_AUDIT/SOURCE_PROMPTS/README.md`, `A0_TRUTHFULNESS_VALIDATION/A0_CORRECTIVE_REPORT.md` (new), `A0_TRUTHFULNESS_VALIDATION/A0_DESIGN_CHECKLIST.md` (new), `.ai/ACTIVE_TASK.md`, `.ai/CURRENT_STATE.md`, `.ai/HANDOFF.md`, `.ai/README.md`, `BASELINE.yaml`, `INDEX.md` |
| Baseline artifacts (2) | `DIRECTORY_TREE.txt`, `SHA256SUMS` — regenerated by the script, never hand-edited |

Diff stat on the 14 product/test/golden paths: **+200 / −18**. Forbidden paths touched: none — no `Cargo.toml`,
`Cargo.lock`, `deny.toml`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `assets/**`, `schemas/**`,
`migrations/**`, `fixtures/**`, `scripts/**`, `.github/**`, `09_ADR/**` or `.gitattributes`.

**Remote CI is not written into this file.** §九 forbids an extra commit whose only purpose is to record a CI number,
and a commit cannot know the run its own push produces; the run id, its head SHA and the job-by-job status are read
back from GitHub after the push and delivered in the round's report to the owner. `04_ci/` in the evidence root holds
those read-back outputs.

## 10. Historical evidence left alone

Two dated documents quote the old remediation string —
`P1_A0_VALIDATION/P1_A0_DESKTOP_SMOKE_REPORT.md:59` and
`U1_VALIDATION/U1P_R2_PENDING_SELECTION_CORRECTIVE_REPORT.md:262`. Both recorded what the product said on the day they
were written. §九 forbids rewriting immutable evidence, so both stand unmodified; the register entry for this prompt is
where the correction is dated.

Read and deliberately **not** changed: `README.md` (its per-round count lines are dated records, and its one
object-attribution sentence at `:1068` already says Compare reports it unavailable with its real reason — the A0-01
correction brings the rest of the product into line with it, not the other way round); `.ai/DECISIONS.md` (A0 opened no
ADR-level decision and moved no technical baseline, so it has nothing to record there); `04_TECH/03_FORMAT_SUPPORT.md`,
`05_ENGINEERING/03_ERROR_MODEL.md` and
`03_DESIGN/06_UI_REFERENCE_SCREENS.md` (the first two are internal specs whose example blocks describe shapes rather
than promises — see §11 items 5 and 6 — and the third is a frozen design reference §七 put out of scope); every
`U1_VALIDATION/`, `V1_VALIDATION/` and `P0`–`P5`/`G2` validation document, and `BASELINE.yaml`'s existing stage blocks.

**`.ai/README.md` is the one entry document this round added to its own change set, and the reason is the same one the
A0 brief exists for.** At `:99` it states "**The present counts are 868 Rust / 225 UI in 8 files**" — a live-state claim
in an entry point, which A0 makes false, and `AGENTS.md` 10 ("docs match behavior") does not allow an entry document to
keep asserting a number the tree no longer prints. Nothing was rewritten: the F2R2 sentence, the U1-closure sentence and
every other per-round figure stay exactly as written, and one dated segment was appended to the state block naming
**870 Rust across the same 47 result lines / 295 UI in 9 files** as the live pointer while identifying the older figures
as their own rounds' dated records. `.ai/CURRENT_STATE.md`, `.ai/HANDOFF.md` and `INDEX.md` carry the same A0 figures.
One citation was corrected the same way it should have been measured the first time: the round's own note had recorded
"49 `test result:` lines / 8 doc-test targets" from a standalone `cargo test --workspace` re-run, and the authoritative
gate step prints **47 lines (41 test-binary + 6 doc-test)** with the identical 870 tests — the standalone invocation
emits two crates' doc-test targets twice, which is a property of that invocation, not of the tree.

## 11. Known limitations

Created by this round: none beyond the disclosure itself. The Compare sentence states the relationship between the
ranking and the headline; it does not make the two reconcilable, and it must not be read as a promise that a sum of
visible rows will ever match.

Remaining, in the order the owner will meet them:
1. **The attribution is still missing, now honestly labelled.** A0-01 corrected the declaration only. GAP-02 — real
   object-level attribution, its persistence and its UI — stays in §七's exclusion list and needs Stage B's own
   authority. Supplying a MAP still buys region evidence, not object evidence.
2. **Overview still cannot open the run it quotes.** The new tests pin that boundary; they do not remove it. Findings
   carry `evidenceRefs` that Overview does not display, and §七 excludes the finding→Inspector links (GAP-03) that
   would change that.
3. **The finding badge states its rule id and its sentence, and its state by glyph alone**
   (`Overview.tsx:511-516`; `StateBadge` renders `StateIcon` + `label` + `note`). The aggregate state word is spelled
   out above it in the verdict chip. Whether a per-row state word is wanted is a design decision, not a truthfulness
   defect, and this round was forbidden from re-drawing the screen.
4. **The D07 owner decisions are untouched**: D07-1 through D07-6 remain the owner's, including the BIN/HEX scope
   conflict that A0-02 deliberately left standing, the BuildIdentity/ArtifactTimes/Git binding (GAP-04), the verify
   surface (GAP-05) and the timeline/Trigger/Source schema (GAP-07).
5. **The CLI prints no capability block of its own.** `04_TECH/03_FORMAT_SUPPORT.md:41-51` says every import must give
   a seven-line capability report, and `Capabilities::rows()` (`capability.rs:105-121`) is that shape — but `rows()`
   has no caller and `apps/cli/src` never prints it. The capability surfaces that exist today are the analysis
   document's `capabilities` object, the Analyze page and the bundle's §9. That is a doc-to-implementation gap rather
   than a false promise to a user, and closing it was not authorized here.
6. **The `04_TECH/03_FORMAT_SUPPORT.md` capability-report sample still lists `Object attribution: Partial`.** That
   block is an internal example of the report's *shape*, and `partial` remains a legal value of the enum for other
   fields, so it was left as written rather than edited into a claim that the field can never be partial. A future
   Stage B would make it reachable again on purpose.

## 12. Recommended next item (advice only, not executed)

The single highest-value follow-up is the one this round's own §四 left open by design: the **BIN/HEX scope
ruling** (D07, owner). The code now tells users the truth about what the entry point accepts, but the governance
documents still disagree with each other about whether BIN/HEX is "Basic" support, so the next misleading statement a
user could meet is not in the product — it is in the spec the product has not been reconciled to. A one-page owner
decision, followed by a docs-only round that aligns ADR-0006, the PRD, the cohort documents and the Compatibility
Matrix to whichever answer is chosen, would close the last promise-as-fact that this round was not permitted to
touch. Stage A (Evidence Loop) and Stage B (Object Attribution) are larger, costlier and each needs its own
authorization; neither is implied by this recommendation.

## 13. What this round is not

A0 is **not** product-value validation. It is not a new V1 cohort, not a re-freeze, and not a verdict on any research
question. The frozen cohort build stays artifact `11573661113` at product head `41bb6a36` with digest
`9a51e86a…c87d93`; if this round's push makes a new GitHub artifact, that artifact is a validation output of this
corrective and is **not** a new identity for the frozen research build, and nothing replaces `11573661113` with it.
U1's `PASS_COMPLETE / VISUAL_ACCEPTED_WITH_KNOWN_LIMITATIONS` and its guarded 25-item tally (23 PASS / 1 FAIL / 1
NOT_VERIFIED, 0 NOT_CAPTURED, 2 `MISMATCH_PROVED`) are not rewritten; V1's paused state, zero eligible external
sessions, M1–M6 `NOT_MEASURED`, the participant register and every metric operation stay as they were. B1, RC and GA
remain `NOT_AUTHORIZED`, and this round does not enter Stage A, Stage B, V1, B1, RC or GA.
