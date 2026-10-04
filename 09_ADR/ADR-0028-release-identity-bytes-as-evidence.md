---
title: "ADR-0028 Release Identity Bytes as Evidence"
doc_id: "ADR-0028"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-04"
---

# ADR-0028 — Release Identity Uses the Exact Bytes Observed on Disk

## Status

Accepted. The question was raised as limitation L22 (`G2_VALIDATION/G2_KNOWN_LIMITATIONS.md`), documented
with its alternatives in `P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md`, and decided by the Architect in
*FirmwareSight — P5 Commit D, Continuation Prompt v1.2 — Architect Reviewed* §3, which directs this ADR and
states the decision in its canonical form:

> RELEASE IDENTITY USES THE EXACT BYTES OBSERVED ON DISK.

This ADR supersedes no earlier ADR. It settles a question ADR-0027 left open — ADR-0027 authorized the
project adapter that *observes* release evidence, and this record decides what that observation means for
identity. It changes no behaviour: the code already behaved this way, and the decision is that it continues
to. `P5_VALIDATION/P5_RELEASE_IDENTITY_ADR_DRAFT.md` stays in the repository as decision-history evidence,
re-statused to point here.

## Context

A release notes file enters the product through one path and reaches the identity through one digest:

- `crates/firmwaresight-project/src/evidence.rs:270` `observe_release_notes` hashes the file with
  `fingerprint::file_sha256` (called at `evidence.rs:291`; `crates/firmwaresight-project/src/fingerprint.rs:42`
  streams raw bytes), so no decoding, no text mode and no newline handling exists anywhere on the path.
- That digest becomes one line of the Gate's canonical input (`notes=<relative-path>#present:<digest>`),
  composed by `canonical_file_fact` (`crates/firmwaresight-core/src/domain/gate.rs:1530`), and the Gate run id
  is the hash of that input.
- The release canonical text then carries **both** the notes digest
  (`crates/firmwaresight-core/src/domain/release.rs:365`) and the Gate run id
  (`release.rs:343`), and the release id is the hash of that text
  (`crates/firmwaresight-project/src/bundle.rs:901`).

So the digest is a checksum of the file the release was built from, in the same evidence class as the
artifact's own SHA-256 — not a checksum of "what the notes say". Two documents that differ only in line
endings are, to this product, two documents.

The discomfort L22 recorded is real and is Git's mechanism rather than the product's: `core.autocrlf=true`
converts LF in the object store to CRLF in the working tree at checkout, so one commit can present different
bytes on two platforms, and a different digest mints a different `gate-…` run id and a different `release-…`
id while every finding in the run says the same thing. With `require_clean_git = true` the smudged work tree
is also dirty and `git.clean` BLOCKs; with `require_clean_git = false` the same id moves and no rule on
screen says why.

Three answers to that problem were credible, which is why the draft asked rather than chose. The Architect
chose the third.

## Decision

**Release identity uses the exact bytes observed on disk.** Consequences, all of them accepted deliberately:

- LF and CRLF are distinct evidence. A release notes file that differs only in line endings is a different
  release.
- The Release Notes digest hashes exact bytes, as observed. `AGENTS.md` §8 is satisfied literally: the
  Observed fact reports what was measured.
- The Gate run id may differ for two checkouts of one commit, and the release id may differ with it.
- The bundle ships the bytes it hashed, and bundle verification stays byte-coherent: `verify_bundle`
  re-derives the release id from the bundle's own bytes and can still do so.
- No LF/CRLF normalization is introduced, at observation, in the digest, in the copy, or in the manifest.
- No Git-blob substitution is introduced. The identity of a release does not depend on the notes file being
  tracked, and a release built from an untracked or locally modified notes file keeps an identity that
  describes the file that shipped.
- No identity-schema change. There is no version field distinguishing "normalized" from "raw", because there
  is only one rule.

Two adjacent decisions belong to this ADR because §3 decided them in the same breath:

- **`release.require_clean_git = false` remains a valid, explicit project-policy option.** It is not forced
  to `true`, and it is not narrowed. The project that sets it has said something about its own release
  process, and the product reports the flag as the project set it.
- **FirmwareSight does not edit `.gitattributes`** — not its own repository's, and never a project's. Writing
  into a firmware project folder is out of bounds (`AGENTS.md` §7), and the byte guarantee a
  `.gitattributes` gives belongs to whoever owns that repository.

What is required instead of a code change is that the caveat be legible wherever the flag is reported:
with `require_clean_git = false`, working-tree byte differences — line endings included — can change the
Release Notes digest, the Gate run id and the release id without `git.clean` blocking the release. The
recommended project practice is a `.gitattributes` rule pinning byte-sensitive release files, which is
advice for the user's repository, documented in the canonical configuration and release documents.

## Alternatives

**Option A — normalize before hashing** (draft §3). Rejected. The gain is reproducibility; the cost is that
the digest stops describing the shipped bytes. The bundle copies the notes file as it found it and its own
`SHA256SUMS` and `release-manifest.json` record that copy, so a normalized identity digest beside an
unnormalized artifact would either make `verify_bundle` fail or force it to normalize too — and the only
self-consistent variant left is FirmwareSight rewriting a copy of a document a human wrote, inside a release
artifact, silently. It would also re-issue every existing release id whose notes contain a `\r`, leaving
History with two records for what a user thinks of as one release. `AGENTS.md` §2 reserves a change to
evidence classification for an ADR, and §8 forbids recording a derived value as though it were measured.

**Option B — identify by Git blob** (draft §4). Rejected. It requires the notes file to be tracked, while the
observation path today needs no Git at all for this fact; an untracked or locally modified file makes the
blob hash and the shipped bytes disagree, which is option A's inconsistency by another route; and it would
add a second identity source to a design where each fact has exactly one.

**Option C — force `require_clean_git = true`** (not in the draft, raised by §3's caveat). Rejected. It
narrows a documented policy option, which is itself an `AGENTS.md` §2 act, and it hides the question instead
of answering it: the identity would still be byte-derived, and a project with a deliberately dirty release
process would simply be unable to say so.

## Consequences

Positive:

- One rule, stated in one place: an id can be recomputed from the bytes that shipped, by anyone, forever.
- Bundle self-verification needs no exception, no normalization flag and no schema version to say which era
  a bundle belongs to.
- Firmware bytes, MAP contents and release documents are never rewritten by this product, which keeps the
  local-first and read-the-project-as-it-is boundary intact.
- The residual surprise is documented where it is felt — the known-limitations row, the configuration
  specification, the README's release section, and Diagnostics' own `require_clean_git` note — rather than
  removed by changing what identity measures.

Negative, and accepted:

- A release id can move for a reason a reader cannot see in the prose: two checkouts of one commit, different
  line endings, two ids, identical verdicts.
- Cross-platform reproducibility of ids therefore depends on the *project's* own byte hygiene, which
  FirmwareSight can recommend (`.gitattributes`) and cannot enforce.
- With `require_clean_git = false`, nothing on the Gate blocks that movement. The flag is the project's
  decision, and the caveat text is the price of it.
- Documentation is now load-bearing: the caveat has to be stated wherever the flag appears, and a future
  field added to Diagnostics or to a release document has to keep stating it.

Canonical contract test:

- `crates/firmwaresight-project/tests/bundle_builder.rs:900`
  `a_notes_file_that_differs_only_in_line_endings_is_a_different_release` is this ADR's executable statement.
  It asserts that each recorded digest equals the digest of exactly the bytes on disk, that the two digests
  differ, that the Gate run id and the release id differ, that the **verdict does not**, and that the published
  bundle ships those same bytes unchanged and verifies against its own manifest. It may not be removed or
  weakened; a change to it is a change to this ADR.

## Revisit trigger

Reopen this ADR if any of the following becomes true:

- a release must be re-verifiable across a change in identity semantics, which would need a versioned
  identity field and is the one case where byte-coherence alone stops being enough;
- FirmwareSight itself begins *writing* release notes rather than observing the release owner's file, at
  which point the product owns the bytes and the question changes character entirely;
- evidence appears that the documented caveat is not reaching the users it is meant to warn — measured, not
  assumed — which would make the choice between documentation and a policy default a live question again;
- a portable-schema or bundle-format revision is being made anyway, where an identity-semantics marker could
  ride along without costing a compatibility break of its own.

Nothing above is a licence to normalize quietly. A change of rule requires this ADR to be superseded by a
numbered successor first.
