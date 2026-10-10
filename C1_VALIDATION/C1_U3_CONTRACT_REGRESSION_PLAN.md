# C1-U3 — Portable Contract + Regression: the plan this round executes

Written 2026-10-10 before the first product edit, in the shape `04_TECH/28` §13 and the U3 prompt both
describe: a contract round whose job is to *prove* what the design claims, not to add a capability.

## 1. Authority, and what it bounds

| item | value |
| --- | --- |
| prompt | 《FirmwareSight — C1-U3 Portable Contract + Regression, Architect Task Prompt v1.0》, stage 2 of 3 |
| archived at | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U3_Portable_Contract_And_Regression_v1.0.txt` (5,949 B, 23 lines, 0 `CR`, sha256 `6e4053db37fd5ba9560e7941bfa9ba2773d912c13f5261a6d590364e16345f83`) |
| design authority | `04_TECH/28` §§5, 6, 7.1–7.7, 8, 12, 13; `ADR-0030`; `04_TECH/26` |
| entry gate | stage 1's objective gate: `920e764` local, tree clean, 17/17, 960 Rust / 48 lines / 0 failed |
| exit words | `C1_U3 = LOCAL_CONTRACT_GATE_PASS / ARCHITECT_REVIEW_PENDING` — and nothing beyond those words |
| forbidden by the packet | a schema major bump, a new mandatory root field, a rewritten golden, push without owner confirmation, treating this round's own self-review as an Architect PASS |

The packet's own scope line is the boundary this plan obeys: *"`firmwaresight-report` models and contract
tests; `firmwaresight-project` verifier/bundle tests needed for compatibility; associated `schemas/*` ONLY if
compatible additive optional extension validation warrants it (no major or top-level change); `04_TECH/26`,
narrow `04_TECH/28` addendum; `C1_VALIDATION` and minimal project pointers. No Desktop UI or CLI, no
migration, parser, dependency or toolchain change, no lockfile, no unrelated scripts or CI workflow."*

## 2. Preflight, measured 2026-10-10 before the first write

`git status --porcelain=v1` → empty · branch `main` · `git rev-parse HEAD` →
`920e7649f47b51a7a0ffe40b7fb123ac1dc8425d` · `git rev-parse origin/main` and `git ls-remote origin
refs/heads/main` → both `2859bb2a7335b8cdea5be2f817a5b848dce126a1` · `git rev-list --left-right --count
origin/main...HEAD` → `0  4` (nothing to pull; four local heads, of which three are the DOCS-ONLY closeouts
and one is stage 1) · `git worktree list` → this directory only · `git stash list` → empty.

Ancestry of the four local heads: `2859bb2` → `99a1bfd` → `7a6af54` → `268440a` → `920e764`. The three
DOCS-ONLY heads touch no `crates/**`, `apps/**`, `schemas/**`, `golden/**`, `migrations/**` or `.github/**`
path, which was re-checked by `git diff --name-only 2859bb2 268440a` before stage 1 was written and is true
of `268440a..920e764` only through stage 1's own scoped code (§11 of its report).

The packet's MANDATORY ENTRY says stage 1 must be "GREEN with a written report" and that "a coding agent's
SELF-REVIEW" is not a final Architect PASS. Both halves are obeyed: stage 1's gate is objectively green and
its report exists, and stage 2 starts **only** on the packet's standing authorization ("Under the separately
handed-off continuous program, only then begin stage 2") while `C1_U2R1` stays
`ARCHITECT_REVIEW_PENDING`, not accepted.

## 3. Premises re-read against the code, including two the design got wrong

Everything below was measured this round, not inherited.

1. **The `gate-results:1` extension pair does not exist.** `grep -rn "canonical_input_label"` over
   `crates/**`, `apps/**`, `schemas/**` and `golden/**` returns **zero** hits. `GateExtensionsDto`
   (`crates/firmwaresight-report/src/gate.rs:63-73`) carries exactly four fields — `policy_sha256`,
   `project_config_schema_version`, `baseline_snapshot_id`, `overall_effective_severity` — and the frozen
   P4 golden's `extensions` object has exactly those four keys. So §7.3's headline row (`T-C1-04`) and the
   document half of `T-C1-01`/`T-C1-05` are unimplemented, which is why this unit exists.
2. **The `/1`-vs-`/2` canonical grammar *is* implemented** — `GateContext::canonical_input()` at
   `crates/firmwaresight-core/src/domain/gate.rs:758-797` picks the label from `canonical_attachments()` and
   writes the `attachments[...]` block only when a row exists. (An earlier onboarding note in this session
   claimed U1 had not landed `/2`; that claim was wrong and is corrected here by the reading above.)
3. **`04_TECH/28` §7.3 cites a test that does not exist.** `T-C1-07` says "the existing
   `the_run_is_judged_from_stored_rows_after_the_files_are_gone` test extended to a no-attachment run". No
   such test exists anywhere in the repository; the nearest are
   `crates/firmwaresight-storage/tests/gate_history.rs:1422` (an *attachment* run re-judges from its rows)
   and `:1253` (a no-attachment run reads back an empty set but never re-judges). This round writes the
   missing test under the name the design used and records in §14's successor that the design's word
   "existing" was false when written — it does not quietly re-attribute the row.
4. **`schemas/*` needs no edit.** `gate-results.schema.json` and `release-manifest.schema.json` both declare
   `"extensions": {"type": "object", "additionalProperties": true}`, so the disclosure travels through the
   reserved slot exactly as §7.3 and §7.5 predict, and `additionalProperties: false` stays where it is: at
   the top level of both documents and inside `files[]` items. `schema_check` implements `minimum`
   (`crates/firmwaresight-report/src/schema_check.rs:257-264`), so `byte_size`/`size` bounds are assertable.
   **Consequence, stated plainly:** the presence-or-absence rule (`/2` ⇔ rows) **cannot** be expressed in
   the schema, because `extensions` is deliberately open. It is therefore a *reader* rule, enforced by
   `verify_bundle` and tested there, not a schema rule. A round that "solved" it by narrowing `extensions`
   would have widened a published contract's rejection set — a semantics change — which the packet forbids.
5. **The manifest half landed in U2 and needs no new field.** `ManifestAttachmentDto`
   (`release.rs:141-175`) emits `{path, kind, sha256, size, kind_basis, provenance}`; `validate_attachments`
   (`release.rs:468-495`) refuses a disclosure whose path is not shipped (`AttachmentNotShipped`) or whose
   digest/size disagree with the index entry (`AttachmentEntryMismatch`); the write path derives the
   disclosure from the same `paired` set it ships (`bundle.rs:1064-1078`), so "shipped but undisclosed" is
   unconstructible; and `disclosed_attachments` (`bundle.rs:2283-2352`) refuses on the read side a row whose
   `kind_basis` is not `declared` or whose `provenance` is not `unknown`.
6. **The `path`-vs-`file` divergence is real and the Architect has ruled it.** §7.5 and §12 sketch
   `{file, kind, kind_basis, sha256, size}`; U2 shipped `path` (plus `provenance`). The U3 prompt §4 rules:
   keep `path` as the canonical bundle-relative locator, keep `provenance: unknown` as mandatory, and write a
   **dated reconciliation naming the superseded sketch** in `04_TECH/26` and `04_TECH/28` §14 or its
   successor. That is a documentation deliverable of this round, not a code change.
7. **The verifier never compares the Gate document to the manifest.** `verify_bundle` reads
   `gate-results.json` for contract validity (`bundle.rs:1968-1971`) and re-derives the release id from the
   manifest's own `extensions.gate_run_id` (`model_from_documents`, `bundle.rs:2359+`), but no check ties the
   document's `run_id` to that field. A bundle can therefore carry a Gate document that describes a different
   run than the one its manifest gates, and still self-verify. The packet asks for "high-signal tamper"
   proof; this is the highest-signal one available, so this round closes it and tests it with a *consistent*
   rewrite (both indexes re-recorded by the existing `reindex` helper, `tests/bundle_builder.rs:615`), which
   is the only kind of tamper that proves a rule rather than proving that hashing works.

## 4. What this round delivers, and the test that pins each item

### 4.1 The `gate-results:1` extension pair (the contract deliverable)

- **Core**: `GateContext::canonical_input_label() -> &'static str`, and `canonical_input()` takes its label
  from that method, so one predicate decides both the grammar and the disclosure and neither can drift.
- **Report**: `GateAttachmentDto { kind, sha256, byte_size }` and two new fields **appended** to
  `GateExtensionsDto`, each `skip_serializing_if` (`Option::is_none` / `Vec::is_empty`), populated from
  `context.canonical_attachments()` and gated on the same emptiness test. Appending, not inserting, is what
  keeps an attachment-free document byte-for-byte the document P4 wrote — field order *is* emitted key order
  in this crate (`gate.rs:11`).
- **`from_evaluation` gains the context as a parameter** rather than growing a second constructor. The
  reason is the discriminability rule: a projection that could be built from an evaluation alone would let a
  caller write a `/2` run's verdict without its rows, which is the exact defect §7.3's "and vice versa"
  forbids. Two production call sites change: `crates/firmwaresight-project/src/bundle.rs:961` and
  `apps/cli/src/main.rs:396`. `release_render.rs` names its extension fields one by one
  (`release_render.rs:478-498`), so the bundle report's bytes cannot shift, and `GateResultsDto` is not a
  ts-rs type, so `drift/ipc bindings unchanged` must stay green.
- **Project (reader)**: a `check_gate_document` step inside `verify_bundle` refusing, deterministically and
  all through `ERR-BUNDLE-6109` (no new code is allocated), a Gate document whose pair is half-present, whose
  label is not exactly `firmwaresight-gate-input/2`, whose rows are not in `(kind word, digest)` order or
  repeat a `(kind, digest)` pair, whose rows do not equal the manifest disclosure projected to
  `(kind, sha256, size)` and deduplicated, or whose `run_id` is not the manifest's `gate_run_id`.
  A malformed digest, a non-attachable kind word, a zero byte length or an unknown-valued `byte_size` in a
  disclosed row are refused on the same path.

### 4.2 The T-C1 matrix closure

| row | what is missing today | what this round writes |
| --- | --- | --- |
| T-C1-01 | the document half | `tests/gate_schema_contract.rs`: a zero-attachment projection's `extensions` has exactly the four pre-C1 keys, in the same order, with neither new entry present |
| T-C1-04 | both directions, entirely | same file: a bound context writes `canonical_input_label` + `attachments` together, in §5 order, one row per `(kind, digest)`; and the reader-side halves in `tests/bundle_builder.rs` (label without rows, rows without label) |
| T-C1-05 | the recompute-from-document half | `tests/release_attachments.rs`: rebuild the canonical text from the document's disclosed rows plus the saved context and require the original text and run id back; a row whose digest moved must produce a different id, not a coerced `/1` |
| T-C1-06 | entirely | `tests/release_attachments.rs`: every pre-C1 document the repository actually carries — all five JSON contracts in `golden/reports/p4-release/` — validated against `schemas/` *by the name each document declares*, with the corpus enumerated, not hand-picked |
| T-C1-07 | the named test does not exist | `tests/gate_history.rs`: `the_run_is_judged_from_stored_rows_after_the_files_are_gone` for a no-attachment run, deleting the source files before the re-judge |
| T-C1-09 | (covered) | kept, and re-run as the mutation control for "a schema field was added by accident" |
| T-C1-10 | the golden half | `tests/release_attachments.rs`: each golden document hashes to the line its own `SHA256SUMS` carries, with the missing `artifacts/*` files named as a fact about the corpus rather than papered over |
| T-C1-11 | the exact-row-set half | `tests/bundle_builder.rs`: a bundle that ships an attached BIN and HEX has `analysis.json` artifact rows whose kinds are exactly `{elf, map}` |
| T-C1-12 | the document-side half | `tests/gate_schema_contract.rs` + the unchanged `apps/cli/tests/p4_golden.rs` byte comparison, which is the real before/after proof and is reported, not replaced |
| T-C1-14 | the golden read-back | `tests/release_attachments.rs`: the relocated golden bundle, with the two committed fixture artifacts copied in and the project deleted, verifies and re-derives the release id the golden manifest claims |
| T-C1-17 | entirely | `tests/bundle_builder.rs`: bytes that begin with the ELF magic declared `hex` satisfy a required `hex` as raw bytes, and neither the finding summary nor any document in the bundle claims Intel HEX |
| T-C1-18 | the findings-and-page half | `tests/bundle_builder.rs`: sweep every finding that cites an `attachment:` locator, and every line of `release-report.html` that names an attached leaf, for the structural vocabulary §11 forbids (span, record, checksum, section, symbol, entry point, architecture) |
| T-C1-19 | (covered) | kept; no test deleted to make room |

Rows 02, 03, 08, 13, 15, 16 are already covered and are **not** rewritten; §6 of this plan lists them with
the test that holds them so the report's matrix is a read-back rather than a claim.

### 4.3 Negative schema tests, written first

In `tests/release_schema_contract.rs`: `files[]` rejects a `kind` property (the strict-boundary proof the
packet §6 asks for by name), and the existing closed-top-level negatives stay. In
`tests/gate_schema_contract.rs`: a Gate document with a new *root* property is still refused, so this round's
addition is provably confined to `extensions`.

## 5. Public surface touched, and nothing else

`firmwaresight-core`: one new inherent method on `GateContext`. `firmwaresight-report`: one new DTO type,
two new fields on `GateExtensionsDto`, one added parameter on `GateResultsDto::from_evaluation`.
`firmwaresight-project`: one new private reader function called from `verify_bundle`, plus its call site.
`apps/cli`: that call site, no new argument, no new output. No schema file, no migration, no parser, no
dependency, no lockfile, no workflow, no design token, no `apps/desktop/**` path, no golden byte.

## 6. Rows already held, listed so this round does not re-claim them

T-C1-02 `release_attachments.rs:101`/`:116` + `bundle_builder.rs:2128` · T-C1-03 `release_attachments.rs:178`
+ `gate.rs:2810` · T-C1-08 `gate_history.rs:1514` + `bundle_builder.rs:3359` · T-C1-09
`release_schema_contract.rs:1387` · T-C1-13 `bundle_builder.rs:3279` · T-C1-15 `bundle_builder.rs:2261`
(and the case-fold refusals at `:2306`, `:2357`, `:2386`) · T-C1-16 `release_attachments.rs:250` · T-C1-19
`gate.rs:2437`.

## 7. Order of work, and how RED is produced honestly

1. Negative schema tests first (§4.3) — green immediately where they pin a boundary that already holds; the
   `files[]`-rejects-`kind` case is asserted against the current schema and mutation-proved.
2. The `gate-results` pair tests (writer side) — these go RED because the fields do not exist yet: the
   document has no `canonical_input_label` key at all. That is a real absence, not a fabricated failure.
3. Core label accessor + DTO fields + the two call sites → 2 turns GREEN.
4. Reader rule + its tamper tests — RED because `verify_bundle` today accepts a Gate document naming a
   different run than its manifest.
5. Corpus read-back tests (T-C1-06/10/14) — these are **preservation proofs**, green on arrival, and they are
   worthless unless shown to discriminate: each is killed by a named mutation in §9, and one (the golden
   verification) is killed by pointing the verifier at a document whose `schema_version` it does not know.
6. `04_TECH/26` landing section, `04_TECH/28` §14.2 reconciliation, `BASELINE.yaml` block, entry-document
   pointers, this plan and the report.
7. Full verification, then one scoped local commit. No push.

## 8. Platform and environment facts decided in advance

- No POSIX runner exists on this host (`docker` absent; `wsl` installed with no distribution), so a POSIX
  figure this round reports is a **projection**, named as one. The `#[cfg(unix)]` module `C1-U2R1` wrote is
  still uncompiled here and stays `NOT_VERIFIED`.
- This host cannot create symlinks (`ERROR_PRIVILEGE_NOT_HELD`, 1314); no U3 test needs one.
- The immutable goldens are read, never regenerated: `scripts/update_goldens.py` is not run, and
  `drift/goldens unchanged` is the gate step that would catch a drift.

## 9. Mutations this round must run before it claims a discriminating test

| mutation | expected killer |
| --- | --- |
| M-A emit `canonical_input_label`/`attachments` unconditionally (drop the two `skip_serializing_if`) | the zero-attachment document test, the golden-corpus validation, and `apps/cli/tests/p4_golden.rs`'s byte comparison |
| M-B drop the label-from-grammar predicate (hard-code `/1` in the extension) | the `/2` writer test and the reader's grammar test |
| M-C skip `canonicalized()` in the projection (emit context order, keep duplicates) | the §5 order test and the reader's order test |
| M-D drop the `run_id` ⇔ `gate_run_id` tie in `verify_bundle` | the consistent-tamper test |
| M-E accept a half-pair (rows without label) | the two reader pair tests |
| M-F `files[]` items opened to additional properties in `schemas/release-manifest.schema.json` | the `files[]`-rejects-`kind` negative |

Each is reverted with the file digest re-measured, and no marker string survives in `crates/**` or
`apps/**`.

## 10. STOP decisions, fixed before the first edit

- If the disclosure cannot validate against the unmodified `:1` schemas → **STOP**, report the minimal
  alternative, do not bump a major and do not loosen a closed boundary.
- If a golden document has to be rewritten to stay green → **STOP**; a golden that moved is a contract change
  wearing a test-suite edit.
- If a pre-C1 corpus document fails to validate under the current schemas → **STOP** and report it as the
  compatibility break it is; do not "fix" the corpus.
- If closing a §4.1 refusal requires a new `ERR-BUNDLE-61xx` code → STOP and reconsider: this round's refusals
  are index/disclosure failures, which `ERR-BUNDLE-6109` already means.
- If a stage-1 finding turns out to be unproven → do not start stage 3 on the strength of this round's plan.

## 11. What this round must NOT be reported as

Not `PASS_COMPLETE`. Not `USER_AVAILABLE`, not `SUPPORTED_WITH_LIMITS`, not "the feature is done": no UI
control, no `--attach`, no IPC command exists after this round either, so `BIN_HEX_RELEASE_ATTACH` stays
`DESIGN_APPROVED / NOT_USER_AVAILABLE` until `C1-U4` ships a surface and an installed run proves it. Not a
V1 action: the cohort stays `11573661113` at `41bb6a36` with 0 sessions. Not a push: `origin/main` stays
`2859bb2` and no force flag appears anywhere. Not an accepted review: the Architect's `REWORK REQUIRED` on
`C1-U2` and its pending verdicts on `C1-U2R1` stand until the Architect says otherwise.
