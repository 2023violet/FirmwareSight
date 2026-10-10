---
title: "C1-U2 Gate and Bundle attached byte consistency implementation plan"
doc_id: "FS-C1-U2-001"
product: "FirmwareSight"
unit: "C1_U2_GATE_BUNDLE_ATTACHED_BYTE_CONSISTENCY"
status: "PLAN"
owner: "Engineering"
authority: "10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U2_Gate_Bundle_Six_Link_Consistency_v1.0.txt + ADR-0030 + 04_TECH/28 §§2-13 + AGENTS.md 2, 3, 6, 7, 8, 9, 10"
written_before_code: true
last_updated: "2026-10-09"
---

# C1-U2 plan — the six links, the paths each one forces, and what stays out of reach

Written at `1b33bbcb52b441efc4e63d7b6c91e829dc232d76` before any source file was edited, as §3 of the authorization
requires. Every line number below was opened in this round at that head. `04_TECH/28:20-22` cites `d41284e` and this
round's §3 forbids trusting inherited line numbers, so each citation is a fresh read, and where the frozen design's
*wording* and this round's *path permissions* disagree, §7 of this plan records the disagreement rather than designing
around it silently.

## 1. Authority, unit, and the archive this plan is written against

§0/§1 authorize one unit and nothing else: `CANONICAL_UNIT: C1_U2_GATE_BUNDLE_ATTACHED_BYTE_CONSISTENCY`, "This is an
explicit authorization of C1-U2 ONLY", with "No UI/CLI user access, no BIN/HEX analysis, no public-release assertion".
`C1-U3` and `C1-U4` stay outside it and §1 closes with "Do not start C1-U3 merely because U2 lands".

The archive is `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U2_Gate_Bundle_Six_Link_Consistency_v1.0.txt`. Measured:
**36,363 bytes, 632 `LF`, 0 `CR`**, sha256 `30fa19579f5d676f65256f51cabd8a2d3e41866d85b1e593f1e75088db15ae59`, staged
as blob `8779dc528b1363010d469212f321d528b5ea2491`, and `cmp` of that stage-0 blob against the working copy returned
identical (`git check-attr text eol --` reports `text: set`, `eol: lf`).

**This copy is a reconstruction, and that is the weakest provenance in the register.** The delivered path
`C:\Users\16429\Downloads\FirmwareSight_C1_U2_Gate_Bundle_Six_Link_Consistency_v1.0.txt` was already gone when the
archive was written: `ls -l` of that exact path returned `No such file or directory`, `ls` of the folder lists six
entries all belonging to another project, `find /c/Users/16429 -maxdepth 4 -iname '*Six_Link*' -o -iname '*C1_U2*'`
returned nothing, and a content grep of the CLI temp cache for `CANONICAL_UNIT: C1_U2` returned nothing. (A content
grep over Desktop and Documents was attempted and timed out; it is not evidence either way.) The bytes come from the
`Read` result this session recorded when it first read that path, so the digest above is the digest of the *archived
copy* and is **not** a delivered-transport digest — §1's "do not hand-edit checksums or" / "invent a delivered digest."
is honoured by recording no delivery-side hash. Two properties therefore cannot be proven: that the delivered file was
LF-only, and that it ended with exactly one newline (the reader numbers the empty trailing piece of a
newline-terminated file; that artifact line was removed, leaving `wc -l` = 632). Every §-quotation in this plan and in
the report was `grep -cF`-checked against the archived copy, so each is followable in what is stored here; if the
original is re-supplied and differs by a line ending, a quotation could in principle differ. That is the whole
residual risk of the reconstruction, stated rather than smoothed over.

## 2. Preflight, measured

| command | result |
| --- | --- |
| `git fetch --prune origin` | ran, no ref change reported |
| `git rev-parse --show-toplevel` | `D:/study/Software/FirmwareSight` |
| `git branch --show-current` | `main` |
| `git rev-parse HEAD` | `1b33bbcb52b441efc4e63d7b6c91e829dc232d76` |
| `git rev-parse origin/main` | `1b33bbcb52b441efc4e63d7b6c91e829dc232d76` |
| `git ls-remote origin refs/heads/main` | `1b33bbcb52b441efc4e63d7b6c91e829dc232d76` |
| `git status --short` | empty |
| `git diff` / `git diff --cached` | empty |
| `git worktree list` | one worktree, this one |

§0's marks re-read locally: migrations `0001`–`0006` with `crates/firmwaresight-storage/src/db.rs:13`
`pub const SCHEMA_VERSION: i64 = 6;`; `.ai/ACTIVE_TASK.md` `NONE.`; the frozen V1 cohort `11573661113`;
`BIN_HEX_ANALYSIS = UNSUPPORTED`. No STOP was owed at preflight. §1 forbids `reset --hard`, `clean -fd`, `stash`,
rebase or amend of published history and every force form; none is used in this round, and the push in §12 is an
ordinary fast-forward that requires the operator's confirmation first (§13: "An Architect-issued attachment is not an
override of user confirmation").

## 3. §3's twelve premises, each measured rather than inherited

| # | premise | measured at `1b33bbc` |
| --- | --- | --- |
| 1 | `GateRunRequest` has no attachments field | `evidence.rs:495-502` — `target, growth, git, policy, release_notes` only |
| 2 | `build_context` forces an empty vector | `evidence.rs:530-532` `attachments: Vec::new(),` with the "Empty by construction this unit" comment |
| 3 | `BundleRequest` has no attachments field | `bundle.rs:244-265` |
| 4 | `observe()` recomputes the context from snapshot facts only | `bundle.rs:588-594` builds a `GateRunRequest` literal and calls `evidence::build_context` |
| 5 | `rule_required_artifacts` reads `context.artifacts` only | `gate.rs:1070-1082` — the `missing` filter tests `!self.artifacts.iter().any(…)` |
| 6 | `rule_artifact_hashes` reads `context.artifacts` only | `gate.rs:1115-1122` |
| 7 | `verify_sources` ships ELF and MAP only | `bundle.rs:942-952` returns `Internal` for any other kind |
| 8 | `InputChecks.sources` is snapshot rows | `bundle.rs:511` `(source path, digest, size)` "in snapshot order"; filled at `:882-891` from `shipped` |
| 9 | `changed_leaf` compares tuples by position | `bundle.rs:554-563` — `earlier.iter().zip(now)` |
| 10 | `bundle_names` disambiguates by content | `bundle.rs:1035-1054` via `disambiguated_name` (`release.rs:523-530`, `<kind>-<sha8>-<name>`) |
| 11 | one U1 test locks the boundary | `tests/release_attachments.rs:291-310` `an_attachment_still_satisfies_nothing_because_the_rules_do_not_read_it_yet` asserts `BLOCK` |
| 12 | U1 observation refuses `elf`/`map`, empty, and non-regular | `evidence.rs:414-450`, `AttachmentError` at `:339-355`, codes at `:364-370` |

All twelve hold; the source has not drifted from what §3 describes, so no re-plan is owed.

## 4. What "the bytes judged are the bytes shipped" means as a mechanism here

Read as one graph, the existing engine already chains verdict → identity → bytes → self-verification for snapshot rows.
U2's job is to make the attachment rows travel the *same* edges, and every edge it must travel already exists:

```
AttachmentSelection(path, declared_kind)
   └─ evidence::observe_attachment ──▶ ReleaseAttachment{file_name, kind, sha256, byte_size, kind_basis}
        ├─ as_gate_fact() ─▶ GateContext.attachments ─▶ canonical_input() "/2" ─▶ run_id ─▶ evaluate()
        │        (gate.rs:762-796 emits the block only when non-empty; fingerprint.rs:34-40 hashes it)
        └─ verify_attachments() ─▶ VerifiedRow ─┐
   verify_sources() ─▶ VerifiedRow ─────────────┴─▶ bundle_names() ─▶ ReleaseArtifact rows
            └─▶ ReleaseModel::validated() ─▶ canonical_release_text() `artifact=kind:sha:size:name`
                 └─▶ release_id ─▶ manifest files[] + extensions.attachments + SHA256SUMS
                      └─▶ write_tree ─▶ verify_bundle ─▶ model_from_documents ─▶ recompute release_id
```

Two of those edges are the ones a careless implementation quietly skips, and both are named in §6's links: the context
that produces the verdict must be the *same* context the plan ships from (L-1), and the row that ships must be a row
that context judged *and* the bytes that row was judged on (L-4).

## 5. Findings that shape the design (each one measured)

**F-1 — the two request structs cannot gain fields, so U2 adds siblings.** `04_TECH/28` §3 names
the judging path `GateRunRequest.attachments` and the planning path `BundleRequest.attachments`, and §5.D repeats
"Attachments must reach Gate via the one sanctioned `GateRunRequest` → `build_context()` path". A struct field would
break every literal of those two types. Measured literal sites: `apps/cli/src/main.rs:386` and `:492`,
`apps/desktop/src-tauri/src/release.rs:242` and `apps/desktop/src-tauri/src/bundle.rs:76` (all four in §11's forbidden
list), plus `crates/firmwaresight-project/tests/bundle_builder.rs:195` and `:230` and this crate's own `bundle.rs:588`.
Rust requires every field in a struct literal unless the caller uses functional update syntax, and none of them does.
§11 anticipates exactly this — "Prefer a backward-compatible library" constructor that preserves current callers — so
U2 keeps both request types byte-unchanged and adds the sibling entry points in §6 below. The *path* the design
freezes (one observation, one `build_context`, one `observe()`, one validator shared by preview and export) is
preserved; only the *naming* of the carrier differs, and §11's permission is what forces that. This deviation is
reported for the Architect, not argued away: `04_TECH/28` §3's table cannot be amended by this round, because §11
allows only a "status-only truthful addendum" to that file.

**F-2 — L-6 forces one additive portable field, and the only legal place for it is `extensions`.**
`verify_bundle` re-derives the release id from the bundle's own bytes, and `model_from_documents` (`bundle.rs:1944-1983`)
resolves every `artifacts/<leaf>` kind by looking it up in `analysis.json`, refusing otherwise:
"`{ARTIFACTS_DIR}/{leaf}` ships, and no document says what kind of artifact it is". An attachment must never appear in
`analysis.json` (`ADR-0030` D-1), so a shipped attachment would be unreadable by the verifier. Measured alternatives:
`files[]` items are `additionalProperties: false` with `required: ["path","sha256","size"]`, so a `kind` cannot ride
there without touching `schemas/**` (forbidden); the manifest's top level is `additionalProperties: false` (so no new
mandatory field — also §9's own prohibition); `extensions` is `additionalProperties: true` with no listed properties
and is not required. Hence `extensions.attachments`, exactly as `04_TECH/28` §7.5 and §12 specify it, and inside the allowance §4's I11
states — "U2 may only add the minimum *honest* serialization necessary for a" bundle containing attachments, with
explicit U3-deferred findings — and §9's "U2 may need small additive serialization to make a library-level bundle"
complete and verifiable. `gate-results.json`'s pair
(`extensions.canonical_input_label` + `extensions.attachments`) is **not** emitted by U2: `GateResultsDto::from_evaluation`
(`report/src/gate.rs:108-129`) takes no attachment input, §13 of the design gives that pair to `C1-U3`, and nothing in
L-1…L-6 needs it — the run id already binds the set.

**F-3 — the disclosure is composed in `crates/firmwaresight-report/src/release.rs`, a path §11 neither lists nor
forbids.** `ReleaseManifestDto` and `ManifestExtensionsDto` live there (`:55-62`, `:107-122`) and `from_parts` builds
`extensions` from the model alone (`:192-244`). Writing the disclosure from `bundle.rs` instead would mean re-serializing
a `serde_json::Value` after `render_json`, which reorders keys and would move the bytes of every bundle including the
frozen P4 golden — the worst of the options. So the report crate gains one additive DTO, one omitted-when-empty field,
and one parameter on `from_parts`; its only caller is `bundle.rs:869-874`. This is stated as an out-of-expected-list
touch, with its reason, in §10's path audit and in the report.

**F-4 — `unknown` attachments ship, and the reader must not be taught a new word.** `ArtifactKind::from_word`
(`identity.rs:141-149`) deliberately has no `unknown` arm: "a bundle never ships a file whose kind it could not
establish". C1 changes that premise — §2.2 allows `unknown` as an attachment kind and §5 rule 5 puts its word into
release identity as `artifact=unknown:…`. Rather than widen `from_word` (a Core vocabulary change that would also change
how `analysis.json` rows are read, and is not needed), the bundle reader resolves attachment kinds the way storage
already does: `crates/firmwaresight-storage/src/gate.rs:770-784` matches the three attachable kinds through
`ArtifactKind::word()` precisely because `from_word` refuses `unknown`. Same technique, same reason, no vocabulary move.
`unknown` still satisfies no requirement (`ADR-0030` D-3, §8 M14) and is never counted in `unknown.count` (D-7, I5).

**F-5 — `build.artifact_sha256` cannot be hijacked by an attachment.** `ReleaseModel::primary_artifact()`
(`release.rs:414-419`) prefers the `Elf` row and falls back to the first, and `validate_entries`' `artifacts/` cross-check
(`report/src/release.rs:341-362`) already requires the shipped set to equal `model.artifacts`. Because `elf` and `map`
can only come from the snapshot and a bundle always ships the analyzed ELF, adding `bin`/`hex` rows leaves
`artifact_sha256` on the ELF — §7.5's requirement. A test names it, because canonical sort order puts `bin` before `elf`
and a `first()`-based reader would have silently repurposed the field.

**F-6 — the pairing in `observe()` is digest-only, which M2 would expose.** `bundle.rs:788-792` recovers each staged
row with `shipped.iter().find(|row| row.sha256 == artifact.sha256)`. Two distinct files with identical bytes (M2) both
match the first row, so the second destination would copy the first file rather than its own. The bytes shipped would
still verify, but the shipped file would not be the one that was selected, which is precisely the L-4 property. Fixed
by pairing `(ReleaseArtifact, source)` before the canonical sort instead of re-finding after it.

**F-7 — `attachment:` is a seventh locator scheme, and three places enumerate the six.** `artifact_ref`
(`gate.rs:1576-1582`) is the scheme Core documents at `04_TECH/28` §5 rule 7; §5.B requires `attachment:<kind>:<sha256>`
beside it. Measured enumerations: `crates/firmwaresight-report/tests/gate_schema_contract.rs:39-46` (used at `:239-246`),
`crates/firmwaresight-storage/tests/gate_history.rs:214-227`, and `apps/desktop/src-tauri/tests/release_gate.rs` (forbidden).
The desktop one never sees an attachment locator — no desktop surface can attach — so it stays green and stays stale;
that is a U4 item, reported not patched. The report-crate array is the repository's only statement of "the locator
schemes Core cites as evidence", and after this unit that statement is false by omission, so it gains the seventh entry
(§9's "minimum honest" rule applies to documents too). The storage array gains nothing unless a U2 storage test needs it;
planned so that it does not.

**F-8 — `evidence.rs:358-362`'s comment names the family as `6101..6114`.** After §7's allocation it is `6101..6117`
plus E-4; the comment is in an authorized path and must match behaviour (`AGENTS.md` §10).

## 6. Public API, exactly as it will be written

`crates/firmwaresight-core/src/domain/gate.rs` — no new public type, no field move:
- `fn attachment_ref(fact: &GateAttachmentFact) -> String` (private, beside `artifact_ref`) →
  `attachment:<kind>:<sha256>` / `attachment:<kind>:-`.
- `rule_required_artifacts`, `rule_artifact_hashes`: two evidence classes, D-3 kind separation, the §5.C wording in §8
  below. **When `canonical_attachments()` is empty both rules emit today's exact summaries and refs**, so every
  pre-C1 finding byte-matches (I1, AC-01, AC-11).

`crates/firmwaresight-project/src/evidence.rs`:
- `pub struct AttachmentSelection { pub path: PathBuf, pub declared_kind: ArtifactKind }` — the caller's intent, before
  any byte is read; `path` is a host path and stays private to validation (I13).
- `pub fn build_context_with_attachments(request: &GateRunRequest, attachments: &[ReleaseAttachment]) -> GateContext` —
  the only function that assembles a context; rows enter through `ReleaseAttachment::as_gate_fact()` and nowhere else
  (no second canonicalization, no second digest).
- `pub fn build_context(request: &GateRunRequest) -> GateContext` — delegation with `&[]`; the `Vec::new()` cap and its
  comment are deleted.

`crates/firmwaresight-project/src/bundle.rs`:
- `pub fn prepare_with_attachments(request: &BundleRequest, attachments: &[AttachmentSelection]) -> Result<BundlePlan, BundleError>`;
  `prepare` delegates with `&[]`.
- `BundlePlan::publish_with_attachments(&self, fresh: &BundleRequest, attachments: &[AttachmentSelection], parent: &Path, overwrite: bool)`;
  `publish` delegates identically, so preview and export share one validator (AC-06, I14).
- `pub enum BundleError` gains `#[error(transparent)] Attachment(#[from] AttachmentError)` — E-1/E-2/E-3 keep their own
  code and remediation through the bundle surface (§7's "explicit typed mapping", never a string) — and
  `AttachmentSetChanged { name: String, change: &'static str }` at `ERR-BUNDLE-6118` (§9).
- `struct SourceRow { class: SourceClass, path: String, digest: String, size: u64 }` with
  `enum SourceClass { Snapshot, Attachment }` (both private to the crate; `InputChecks` is already private);
  `verify_attachments()` (§8's L-4); pairing fix (F-6); `model_from_documents` reads the disclosure (F-2).

`crates/firmwaresight-project/src/lib.rs` — three re-exports added to the existing `pub use` lists (the same two lines
U1 touched for `AttachmentError`/`observe_attachment`).

`crates/firmwaresight-report/src/release.rs` — `pub struct ManifestAttachmentDto { file, kind, sha256, size, kind_basis }`,
`ManifestExtensionsDto.attachments: Vec<ManifestAttachmentDto>` with
`#[serde(skip_serializing_if = "Vec::is_empty")]`, and `from_parts(…, attachments: &[ManifestAttachmentDto])`. An
attachment whose `KindBasis` is not `Declared` is refused with `BundleError::Internal` before any document is composed:
the schema stores the basis *word* and not the leading-byte sample (`gate.rs:427-446`, and storage refuses to read a
derived row back for the same reason), so disclosing a derived basis in a portable document would state an evidence
detail the document cannot show. `observe_attachment` only ever produces `Declared`, so this arm is the same class of
structural guard as `verify_sources`' existing one.

## 7. Old versus new, state by state

| state | before | after |
| --- | --- | --- |
| `GateRunRequest` / `BundleRequest` | no attachment carrier | unchanged, byte-for-byte, and every caller in `apps/` compiles untouched |
| `build_context` | forces `Vec::new()` | delegates to `build_context_with_attachments(_, &[])` |
| context with an empty set | `/1` text, run id `gate-a17e9861…` for the P4 fixture | identical text, identical id, identical findings |
| context with rows | impossible from any caller | `/2` text, rows sorted by `(kind word, digest)`, `(kind, digest)` bound once |
| `artifacts.required` | `bin`/`hex` unsatisfiable | `elf`/`map` from snapshot rows only; `bin`/`hex` from attachment rows only; `unknown` from neither |
| `artifacts.hashes` | snapshot rows counted | both classes counted, `artifact:` and `attachment:` locators both present, limit sentence attached |
| `evidence.unknown_review` | counts snapshot `Unknown` items | unchanged by an attachment (I5, D-7) — a test asserts the count and the state do not move |
| bundle without attachments | today's engine | same code path with an empty selection list; P4 golden bytes unchanged |
| bundle with attachments | impossible | `artifacts/<leaf>` for each row, `files[]` + `SHA256SUMS` entries, `extensions.attachments` disclosure, release id binds kind+digest+size+name |
| `verify_bundle` | refuses a leaf `analysis.json` does not name | also resolves a leaf named by `extensions.attachments`, and still refuses an unexplained one |
| preview → publish | pairwise `changed_leaf` | typed rows: same path + new digest/size → `SourceArtifactChanged`; add/remove/rename → `AttachmentSetChanged` |
| stored run | immutable, `/1`-bound | unchanged mechanism; a run that bound attachments recomputes only with the same set (L-3) |
| `schemas/*` | majors 1 | untouched, un-bumped; the disclosure sits in `extensions` which is `additionalProperties: true` |
| migrations / `SCHEMA_VERSION` | 6, `0001`–`0006` | unchanged (0006 already stores attachment rows; §10 forbids extra migrations) |

## 8. The six links, each with the mechanism and the tests that carry it

| link | mechanism this round adds or reuses | positive test | negative control |
| --- | --- | --- | --- |
| **L-1 verdict subject** | `observe()` builds the context from the same observed rows it plans to ship; the `Vec::new()` cap is gone | `a_release_that_requires_bin_and_hex_passes_only_when_both_are_attached_and_bound` (project) + `the_gate_passes_a_required_bin_from_a_bound_attachment` (core) | `a_required_bin_with_no_attachment_still_blocks` — the retained half of U1's boundary test |
| **L-2 identity** | `canonical_attachments()` → `/2` block → `run_id` (all U1 code, now reachable) | `mutating_one_attached_byte_moves_the_run_id_and_not_the_snapshot` + `a_renamed_attachment_keeps_the_run_id_and_moves_the_release_id` | `a_size_claim_difference_alone_is_not_a_gate_identity_input` (U1, kept) and the `/1` byte-lock in `fingerprint.rs` (U1, kept) |
| **L-3 stored-run recompute** | `observe()` recomputes and compares to `selected_run_id` before anything is read for shipping | `a_stored_run_that_no_longer_recomputes_is_refused_before_any_write` (attachment dropped / added / re-declared) | `a_missing_attachment_refuses_the_bundle_and_leaves_the_stored_run_untouched` + storage's `a_stored_run_that_bound_attachments_is_rejudged_without_its_files` (T-C1-07 extended) |
| **L-4 membership and bytes** | `verify_attachments`: stat, regular file, non-empty, streamed SHA-256, digest equality, size equality, membership in `context.attachments` by `(kind, digest)` | `every_shipped_attachment_is_a_row_the_judged_context_carries` | white-box `an_attachment_row_the_judged_context_does_not_carry_is_refused` (inline `bundle/tests.rs`) + `a_file_offered_for_its_name_and_not_its_bytes_is_refused` |
| **L-5 preview/export staleness** | typed `SourceRow` set in `InputChecks.sources`, `compare_with` classification, `changed_leaf` keyed by path instead of position | `a_changed_attachment_between_preview_and_publish_is_refused_by_name` | `an_attachment_added_between_preview_and_publish_names_itself_as_added` and `…_removed` / `…_renamed` (E-4, three cases, not one) |
| **L-6 self-verifying output** | `artifacts/<name>` collision-safe leaves, `files[]` + `SHA256SUMS`, `extensions.attachments`, `verify_bundle` re-derivation | `a_bundle_that_ships_attachments_verifies_and_re_derives_its_release_id` + `the_release_id_changes_when_only_an_attachment_name_changes` | `tampering_with_a_shipped_attachment_breaks_the_bundle_it_sits_in` + `a_relocated_bundle_with_attachments_verifies_with_its_sources_deleted` + P4's three golden tests unchanged |

Six proof rows, each with a named positive and a named negative, per §6's "A single broad happy-path bundle test is NOT
enough".

## 9. Error registry, scanned from live `code()` arms

Every `code()` implementation in the workspace, measured: `core/src/domain/diff.rs:611`,
`core/src/domain/release.rs:598`, `project/src/bundle.rs:168`, `project/src/error.rs:68`,
`project/src/evidence.rs:364`. Inventory of the `ERR-BUNDLE` family across all tracked files (`git grep -rhoE
"ERR-BUNDLE-[0-9]{4}" | sort | uniq -c`): **6101–6117 all occupied**, with `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md:474`
keeping its closed-stage `6101–6114` tally as U1 left it. `git grep -n "6118"` returns **nothing anywhere in the
repository** — no code, no document, no schema — so 6118 is vacant by measurement rather than by guess, and it is the
next number in the family. There is still no markdown registry, so the allocation is proved by a live test:
`the_new_codes_take_no_number_that_an_existing_refusal_already_uses` (U1's) is extended to call the real `code()` on a
constructed `BundleError::AttachmentSetChanged { … }` and on each bridged `AttachmentError`, asserting 6118 is not in
the 6101–6114 family, not equal to 6115/6116/6117, and that E-1..E-3 still answer 6115/6116/6117 through the bundle
surface. Nothing else is renumbered, and §7 keeps E-4 stable once released.

E-4's planned envelope: message `the release attachment set changed after the preview: `{name}` was {change}`, with
`change` one of `added` / `removed` / `renamed` chosen by the deterministic rule in §8's L-5 row, remediation "prepare the bundle
again against the files this release actually ships: a preview names the bytes it judged, and a set that moved is not
that preview" — it says re-preview and re-check, names only a sanitized leaf, and carries no absolute path (§7).

## 10. Path audit

**Expected by §11 and touched:** `crates/firmwaresight-core/src/domain/gate.rs`,
`crates/firmwaresight-project/src/evidence.rs`, `crates/firmwaresight-project/src/bundle.rs`,
`crates/firmwaresight-project/tests/**` (`release_attachments.rs`, `bundle_builder.rs`, inline `bundle/tests.rs`).

**Allowed by §11 as "relevant … specific release/verifier tests" / "a narrow storage read-back test":**
`crates/firmwaresight-report/tests/gate_schema_contract.rs` (F-7),
`crates/firmwaresight-storage/tests/gate_history.rs` (L-3 / T-C1-07 extended).

**Not listed by §11, not forbidden, and necessary — disclosed as a scope note:**
`crates/firmwaresight-project/src/lib.rs` (three re-export lines; U1's precedent),
`crates/firmwaresight-report/src/release.rs` (F-3, the only place the manifest document is composed).

**Forbidden by §11 and untouched:** `apps/desktop/ui/**`, `apps/desktop/src-tauri/**` — including
`tests/release_gate.rs`'s stale six-scheme list (F-7) and the four request literals of F-1, which is exactly why the
siblings exist — `apps/cli/**`, `migrations/**`, `schemas/**`, `golden/reports/p4-release/**`, unrelated `fixtures/**`,
`SnapshotId::compose`, the ELF parser, `build_snapshot`, analysis/diff identity, all lockfiles, `.github/**`,
`scripts/**`, `toolchains/**`, `assets/design-tokens.json`, updater/signing/licensing.

**No STOP is owed on a compile failure**: the CLI's `release_exit_code` (`apps/cli/src/main.rs:582-590`) and the
desktop's `envelope_from_bundle` (`apps/desktop/src-tauri/src/bundle.rs:352-361`) both match through a catch-all or
through `code()`/`remediation()` only, so new `BundleError` variants cannot break either binary. This was checked before
the API was chosen, not after.

## 11. TDD order, RED first

§10's five behaviours are written as failing tests against the U1 code *as it stands*, and their output is kept:
1. `artifacts.required` satisfied by kind mapping — RED: the core rule still reads snapshot rows only.
2. hashes + `attachment:` locators — RED: no attachment row is ever cited.
3. an attachment-bearing request reaching real `observe`/`prepare`/`publish` — RED: `build_context_with_attachments`
   and `prepare_with_attachments` do not exist, so this RED is a compile failure in the test crate, recorded as such
   rather than dressed up as an assertion failure.
4. membership refusal including a stale-preview set change — RED: `changed_leaf`'s positional zip reports the wrong
   file, which is the defect E-4 exists to close.
5. unmodified no-attachment behaviour — GREEN *before* the rest: `/1` bytes, `gate-a17e9861…`, the P4 goldens and the
   policy fingerprint `b87aa057…` must already hold at `1b33bbc`, and are re-asserted after.

Then GREEN. U1's boundary test is converted into the positive-binding regression and keeps its missing-attachment
negative; no test is deleted.

**Mutations** (§10, each to be an actual red result, restored byte-identically with digests recorded before and after):
satisfy `bin` from a snapshot row and `elf` from an attachment; delete `verify_attachments`' membership test; delete its
fresh-digest comparison; make the set-change classifier fall back to the pairwise rename path; and either emit the
`attachments[` block under `/1` for an empty set or drop `file_name` from the `artifact=` line.

**Test map for §8's M1–M16**: M1–M3 in `bundle_builder.rs`'s World (real files, one BIN, one HEX, duplicates by path
and by content); M4/M5 against `sanitize_leaf_name` + `bundle_names` and a case-folded manifest refusal; M6/M7/M10 in
the staleness section (E-4's three cases); M8/M9 as refusal-before-write, proving the destination folder stays empty and
the stored run row unchanged; M11 directory/missing/zero-byte (E-1/E-3 through the bridge), traversal and absolute-path
attempts refused by `is_safe_bundle_relative_path` and `verify_bundle`'s host-path scan, symlink behind `#[cfg(unix)]`
with the Windows non-run recorded; M12 a 1 MiB attachment hashed and shipped through `file_sha256`'s 64 KiB streaming
buffer, with the >512 MiB fixture **not** created and named as NOT_VERIFIED rather than assumed; M13 declared non-HEX
bytes satisfying `hex` with the limitation sentence and no validity claim; M14 `unknown` satisfying neither `bin` nor
`hex`; M15 the P4 golden eight files byte-identical and `apps/cli/tests/p4_golden.rs` untouched; M16 is satisfied by
construction — everything above is library-level with real regular files and real SQLite files, no mocked observation,
and no installed artifact is required.

## 12. Staging, push, and the STOP decisions I have already made

1. RED logs, then implementation, then the full §12 validation list; the frontend must stay **295 passed / nine
   files**, and any variation is a STOP rather than a re-baseline.
2. ADR-0029 closeout in the generator's documented order — edit → `git add` → `tree > DIRECTORY_TREE.txt` → add →
   `sums > SHA256SUMS` → add → `verify_baseline_artifacts.py` → full `scripts/check.py` at the final staged state. The
   redirections are part of the order, not decoration.
3. One narrow product commit (code + tests + this plan + the archived prompt + baseline artifacts), a full staged
   path audit against §11's forbidden classes, then **ask the operator before pushing** and push only as an ordinary
   fast-forward. At the exact product HEAD, read all ten jobs individually.
4. At most one docs-only successor, and no commit that exists only to record its own CI result.
5. STOP-and-report conditions already fixed in advance, so they are not negotiated later: any need to touch
   `apps/**`, `schemas/**`, `migrations/**` or the goldens; any attachment that can ship while unjudged or ship bytes
   other than the judged ones; any `/1` drift; any stored-run mutability under one id; any provenance claim about an
   attachment's origin; any Rust/UI count that cannot be reproduced by the command that prints it.

## 13. What U2 must not be reported as

`CODE IMPLEMENTED` and `TESTED LIBRARY API` may be true at the end of this round. `PRODUCT USER ENTRY AVAILABLE`,
`PORTABLE CONTRACT COMPLETE`, `INSTALLED VERIFICATION` and `V1 SESSIONS` are false and stay false: nothing in §11's
forbidden paths changes, no surface offers a file, and the V1 cohort stays `11573661113` at `41bb6a36`.

Explicitly NOT_VERIFIED / deferred to `C1-U3`, per §9: the `gate-results:1` pair
`extensions.canonical_input_label` + `extensions.attachments` (T-C1-04 and T-C1-05's document half, T-C1-06's
re-validation of the pre-C1 corpus against new documents); the pre-C1 corpus readability proof as a matrix rather than
as one golden test; `T-C1-09` only in its "validates within the currently implemented `:1` shape" half, which U2 can
test, and not its old-corpus-compatibility half; `T-C1-10`, `T-C1-14`, `T-C1-15` are U2's to prove for bundles that
carry attachments; the golden-regression closure and any schema-level statement about the new extension entries.
And one boundary U2 cannot cross by definition: the attachment set a *person* chooses, which is `C1-U4`'s.

## 14. Executed 2026-10-09 — every place the round differed from this plan

Written after the last green run, against the files as committed. The sections above are the plan as it stood before
any source was edited; they are not corrected in place, because a plan that is edited to match the result proves
nothing. This section is the diff between the two.

**Counts, measured.** `cargo test --workspace` **953 passed / 0 failed across 48 `test result:` lines** at the pre-U2
head `1b33bbc`'s 900. The +53 is per file, counted from the staged diff
(`git diff -U0 | awk '/^diff --git/{f=$0} /^\+[[:space:]]*#\[test\]/{c[f]++}'`):
34 `crates/firmwaresight-project/tests/bundle_builder.rs` + 8 `crates/firmwaresight-core/src/domain/gate.rs`
+ 6 `crates/firmwaresight-report/tests/release_schema_contract.rs` + 3
`crates/firmwaresight-project/src/bundle/tests.rs` + 1 `crates/firmwaresight-project/tests/release_attachments.rs`
+ 1 `crates/firmwaresight-storage/tests/gate_history.rs` = 53. The 48-line shape did not move: no new test binary was
created, because the two project integration targets already existed at U1.

**Path audit delta.** §10 named `release_schema_contract.rs` nowhere; it is added to the *allowed* bucket
("specific release/verifier tests"), because L-6's disclosure is composed in `firmwaresight-report` and its refusal
rules can only be proven at that layer. Every other touched path is in §10's first two buckets. No §11 forbidden path
was touched, and no `apps/**` compile failure was ever seen, so the §12 fallback never fired.

**Test names.** §8's six-link table predicted **seven** names that exist in no source file, and this paragraph is the
grep-checkable correction: each prediction is followed by the delivered name that carries the same claim.
`the_gate_passes_a_required_bin_from_a_bound_attachment` → `a_bound_bin_and_hex_attachment_pair_satisfies_their_own_requirements`
(core) and `a_bound_bin_attachment_satisfies_a_required_bin_and_an_unbound_one_still_blocks` (project);
`mutating_one_attached_byte_moves_the_run_id_and_not_the_snapshot` → `attaching_a_file_moves_the_run_id_and_leaves_the_snapshot_alone`
and `changing_only_the_attached_bytes_changes_the_run_id_and_not_the_snapshot`;
`a_missing_attachment_refuses_the_bundle_and_leaves_the_stored_run_untouched` →
`a_stored_run_that_no_longer_recomputes_is_refused_before_any_write`;
`a_stored_run_that_bound_attachments_is_rejudged_without_its_files` → `a_stored_run_that_bound_attachments_rejudges_from_its_own_rows`
(the U1 name was reworded, and it is a real test, not a prediction);
`an_attachment_row_the_judged_context_does_not_carry_is_refused` → `an_attachment_the_judged_context_does_not_carry_is_refused`;
`a_file_offered_for_its_name_and_not_its_bytes_is_refused` → `the_attachment_rows_a_context_binds_are_the_facts_the_observation_produced`;
`the_release_id_changes_when_only_an_attachment_name_changes` → `a_renamed_attachment_keeps_the_run_id_and_moves_the_release_id`.
The delivered names, link by link, are in `C1_U2_GATE_BUNDLE_EXECUTION_REPORT.md` §6, and all 57 test names that report
cites were checked with `git grep` against the tree: every one resolves to a `fn` in a source file. The substance of each
predicted row is covered, and no row is covered by fewer than one positive and one negative test. The eighth name this
paragraph could have counted, `an_attachment_still_satisfies_nothing_because_the_rules_do_not_read_it_yet`, is U1's
boundary test at §12's item 11 — real when the plan was written, converted by this round rather than invented, and the
conversion is quoted in the report's §2.

**Mutations.** §11 predicted five; seven ran. Two of the predicted five landed as predicted (the `elf`/`map` kind satisfied from an attachment
row, and the disabled rename identification). Three ran in a different, stronger form:
"delete `verify_attachments`' membership test" became `if false {` around it, which kills it through the same test;
"delete its fresh-digest comparison" was **not** run as a separate mutation — the membership mutation subsumes the
refusal path it would have exercised, and the fresh-digest comparison is covered by L-4's positive test and by
M9's byte-mutation refusal instead; "emit the `/1` block for an empty set" is U1's mutation `M4` and was not repeated
here, so this round's fifth mutation is instead the removal of `#[serde(skip_serializing_if = "Vec::is_empty")]` from
the disclosure field, which is the L-6 half of the same "write nothing when there is nothing" rule. A sixth mutation
(the set-change classifier returning `None` for every pair) was run last, and it is the one that found a real gap in
this round's own process, stated next. A seventh was added after §14's own writing: a
`#[serde(rename = "kind_word")]` on the disclosure's `kind`, run to prove that the T-C1-18 key-set lock described two
paragraphs below can fail rather than pass vacuously — it did, with `left: Null / right: String("bin")`. Seven in total,
all restored byte-identically.

**A process defect this round caused and fixed.** While adding M10's `added` case, a Python region rewrite of
`bundle_builder.rs` matched zero tests and silently dropped `an_attachment_added_between_preview_and_publish_names_itself_as_added`
— the suite went to 77 and stayed green, which is exactly how a deleted test hides. It was found only when the sixth
mutation's filter answered `77 filtered out`, and the test was re-added (78, then 78 green again). The lesson is the
one §10 already stated in the abstract — "Do not delete tests to hide changed requirements" — met by accident rather
than by intent. Nothing else was deleted; `every_refusal_a_caller_can_branch_on_has_a_code_and_a_remediation` was
extended rather than replaced, and U1's boundary test was converted rather than deleted.

**Restoration digests.** Before and after, byte-identical: `bundle.rs`
`192ae54180f59ff4de1967d3ca6a23a1e5e45f4a3f40f52861ecdf1d3f711873`, `gate.rs`
`0c62a9980a5083192ed0c4ed347ad583f1d74dee0a130abf50877d8da128c091`, `report/src/release.rs`
`d6634bc8da9fec70c12470eedbff0c0c3c25b0d18bba0198a119cfe99e036815`. For `release.rs` the honest limit is that the
"before" digest is the copy taken immediately before its mutation, not one recorded at the start of the round; the
file is otherwise identical to the state the rest of §12 validated.

**A wording imprecision in this plan's own §8 L-4 row, corrected here rather than there.** That row describes
`verify_attachments` as doing "stat, regular file, non-empty, streamed SHA-256, digest equality, size equality, membership
in `context.attachments`". Delivered code splits the duty: `evidence::observe_attachment` (one call per selection, via
`observe_attachments` at `bundle.rs:1229`) does the resolve → stat → regular-file → non-empty → streamed-digest →
read-back-length chain, and `verify_attachments` (`bundle.rs:1271`) does **not** re-hash — it refuses a row whose kind a
release may not attach, whose `kind_basis` is not `Declared`, whose observation left no digest, or which the judged
`context.attachments` does not carry as a `(kind, digest)` pair. The fresh-versus-preview byte comparison the row calls
"digest equality / size equality" is `InputChecks::compare_with`, which `publish_with_attachments` runs after re-calling
`observe()` (`bundle.rs:427-428`). §14's earlier draft of the governance documents repeated the plan's compressed wording,
and `.ai/ACTIVE_TASK.md`, `.ai/CURRENT_STATE.md` and `BASELINE.yaml` now carry the split version; the L-4 positive and
negative tests are unchanged, since both chains run inside the same call path either way.

**Four findings this round produced and did not silently fix**, because each is a decision rather than a defect:

1. **M5's remediation quality.** Two selections that fold to one host file are refused by the manifest's
   `DuplicatePath`, which the bundle layer maps onto `ERR-BUNDLE-6109 VERIFY_FAILED` with the message "two entries name
   `artifacts/bin-<sha8>-app.bin`, and a manifest cannot hash one file twice". That is true and it is not useful: the
   caller's mistake was a *selection*, not a manifest. A dedicated code, or a pre-index check in `bundle_names`, is a
   portable-contract decision that belongs with §9's U3 scope, so the behaviour is reported rather than patched here.
2. **Export precedence.** `publish_with_attachments` recomputes the verdict (`observe()`) *before* it compares the
   preview, so withdrawing an attachment the policy requires answers `ERR-BUNDLE-6101`
   (`ReleaseError::GateNotReady`), not E-4. That
   is the right order — a release that no longer passes cannot be packaged however its file set moved — and it is now
   stated in code and asserted by `a_withdrawn_attachment_the_policy_requires_is_refused_by_the_gate_first` rather than
   left as a reader's inference.
3. **M11's symlink half, NOT_VERIFIED.** `04_TECH/28` §4.4 states symlink refusal for what is *inside a bundle*
   (`verify_bundle` keeps refusing it, unchanged here); it says nothing about a symlink offered as a *source* path, and
   `observe_attachment` follows it, as any `std::fs` read would. Writing a `#[cfg(unix)]` test would have fixed one
   behaviour by choosing one, which §15 forbids ("do not invent a new scope, product API… to continue"). What ran
   where: `#[cfg(windows)] one_host_file_offered_under_two_cases_is_refused_rather_than_shipped_twice` ran on this host
   and cannot run on the CI ubuntu job; no `#[cfg(unix)]` test exists, so no Unix-only case ran anywhere; everything
   else in M1–M16 is platform-neutral and ran on both.
4. **M12's upper bound, NOT_VERIFIED.** A 1 MiB attachment is hashed and shipped through the same streaming read
   (`a_large_attachment_is_hashed_and_shipped_by_the_same_streaming_read`); the >512 MiB case was **not** created, so
   memory behaviour at that size is unmeasured rather than assumed safe.

**One test written after the behaviour it guards, marked as such.** `04_TECH/28` §7.5's `T-C1-09` says a manifest with
attachments must validate against the unmodified `release-manifest:1`. The disclosure passed that test only by
inspection until this paragraph's writing: the schema's `extensions` is `additionalProperties: true`, so the shape fits,
but no assertion said so. `a_manifest_that_discloses_attachments_still_validates_against_the_unmodified_v1_schema` was
therefore added **after** the implementation, which is the one case this round's own TDD order failed to cover, and it is
recorded rather than smoothed over. Its discriminating power is proved inside the test: the same document with the
disclosure's `kind` moved into a `files[]` item — the alternative §7.5 names and refuses, since those items are
`additionalProperties: false` — is rejected by the same validator, so the passing assertion is awake rather than empty.
That addition is the 53rd test, and it lives in the count above.

**§12 measured, in the order the prompt lists it.** `git diff --check` clean; `cargo fmt --all -- --check` clean;
`cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; `cargo test --workspace` 953/0 in 48
lines; `corepack pnpm install --frozen-lockfile` / `typecheck` / `lint` clean; `corepack pnpm test` **295 passed in 9
files** (the required figure, unmoved); `corepack pnpm build` clean; `python scripts/check.py` **17/17**;
`--only drift` **8/8**; `--only deny` **1/1**; `--only core-smoke` **3/3**; `--only package` **4/4** with no `SKIP`;
`python scripts/verify_baseline_artifacts.py` PASS.
