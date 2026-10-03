---
title: "P5 Release Identity and Line Endings — ADR Draft"
doc_id: "FS-P5-RELID-ADR-DRAFT"
product: "FirmwareSight"
version: "0.1"
status: "DRAFT_FOR_ARCHITECT_REVIEW"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-03"
related: "[L22] G2_VALIDATION/G2_KNOWN_LIMITATIONS.md:44, P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md §G, prompt §32"
---

# Draft: does a release's identity include its line endings? (prompt §32, limitation L22)

**Status: DRAFT. Nothing here is decided, and no behaviour described below is changed by P5.** §32 says
this round may document and test L22 but may not silently normalize release identity, and that a semantic
change requires this document followed by a STOP for Architect review. That is the whole of its mandate, so
this file states the current semantics precisely, what the alternatives would cost, and what the next
authoritative decision is. **Until an Architect approves one of options A or B, option C — today's
behaviour — remains the behaviour of the product.** `AGENTS.md` §2 puts a change to evidence
classification and to identity semantics behind an ADR, which is what this draft is for.

Every claim below was read off the code at head `e070508` with the `path:line` cited, or run. Where a
number is quoted, the command that produced it is named.

## 1. Current semantics: the bytes on disk are the evidence

A release notes file enters the product through exactly one function.

- `firmwaresight-project/src/evidence.rs:291` — `observe_release_notes` calls `fingerprint::file_sha256`,
  which (`fingerprint.rs:42-56`) streams the file's raw bytes through SHA-256 with no decoding, no text
  mode and no newline handling anywhere on the path. The result is recorded as
  `GateFileStatus::Present { sha256: Some(hex) }` (`core/domain/gate.rs:374-381`).
- That fact enters the Gate's canonical input as one labelled line,
  `notes=<relative-path>#present:<digest>` (`gate.rs:712-718` composed by `canonical_file_fact`,
  `gate.rs:1530-1541`).
- The Gate run id is `gate-<sha256 of that canonical input>` (`fingerprint.rs:32-39`), and the canonical
  input's own contract is that "the same facts always produce the same id, and any fact that changes the
  verdict produces a different one" (`fingerprint.rs:3-7`).
- The release id is `release-<sha256 of `ReleaseModel::canonical_release_text`>` (`bundle.rs:899-901`,
  `release.rs:405-407`), and that text contains **both** the notes digest (`notes=<hex>`,
  `release.rs:363-370`) **and** the Gate run id (`gate_run=<id>`, `release.rs:343`).

So the digest is not a checksum of "what the notes say". It is a checksum of the file the release was
built from — the same evidence class as the artifact's own SHA-256. That is deliberate: `AGENTS.md` §8
makes Observed facts report what was measured, and the bytes of a file the release owner handed over are
what was measured. An absent file is a different value again (`notes=…#missing`), so absence is never
collapsed into an empty digest.

Nothing in this chain claims the file is *canonical text*. Two documents can differ in line endings and be
the same prose; the product treats them as two documents, which is the honest reading of "the release
stands on these bytes" and the source of L22's discomfort.

## 2. Cross-platform line-ending impact

The mechanism is Git's, not the product's:

- `core.autocrlf=true` (the common Windows developer setting) converts LF in the object store to CRLF in
  the working tree at checkout. A notes file committed with LF is therefore **a different set of bytes** on
  a Windows checkout than on a Linux or macOS one.
- The conversion applies to whatever Git classifies as text. With no `.gitattributes` rule, that
  classification is the machine's choice, so two checkouts of one commit can disagree about the digest of
  one document.
- This repository has already closed the hole for its own evidence: `.gitattributes` states the
  guarantee twice over — `binary` for byte-identity-critical artifacts, and `text eol=lf` for every text
  extension tracked, with the header naming CI run `36360310447` as the reason (`p0-dual-region.ld` was
  smudged to CRLF and a hash test failed on the checkout, not on the fixture).
- **That protection does not extend to the customer's project.** The notes file lives in the firmware
  repository being released, under *its* `.gitattributes` and *its* checkout configuration. FirmwareSight
  has no authority there and takes none — writing into a project folder is out of bounds for this product
  (prompt §36, `AGENTS.md` §7), so "just add a `.gitattributes`" is advice for the user's own repository,
  never something the tool does on their behalf.

What a user observes when it bites: the same commit checked out with different line endings produces a
different `gate-…` run id and a different `release-…` id, while every finding in the run says the same
thing. With `release.require_clean_git = true` (the default, `gate.rs:261`) the working tree is also dirty
after such a smudge, and `git.clean` **BLOCKs** (`gate.rs:733-761`; the branch is pinned by the unit test at
`gate.rs:1822-1825`, and "a dirty workspace is a new run" by `gate.rs:2377-2382`). **The fail-closed
protection is therefore policy-dependent**: a project that sets `require_clean_git = false` makes
`git.clean` `N/A` rather than BLOCK, and the id then moves with nothing on screen saying why. That is the
real residual risk in L22, and it is worth stating more plainly than the current known-limitations row
does.

### What P5 added here

One test and no behaviour change:
`crates/firmwaresight-project/tests/bundle_builder.rs`,
`a_notes_file_that_differs_only_in_line_endings_is_a_different_release`. It writes the same notes words in
LF and in CRLF into two release subjects and asserts five things: each recorded digest equals the digest of
exactly the bytes on disk; the two digests differ; both the Gate run id and the release id differ; the
**verdict does not** (`overall_effective_severity` and the five-state `counts` are equal — identity and
correctness are different questions, and conflating them is what would let a normalization through as a
presentation fix); and the published bundle ships those same bytes unchanged, verifying against its own
manifest.

Mutation proof, because an assertion that cannot fail is not evidence: `observe_release_notes` was
temporarily rewritten to hash the file's bytes with every `\r` removed — i.e. option A below, applied at
the one place that would make it invisible. The test failed at the "recorded digest equals the bytes on
disk" line (`left: afa3f566…`, `right: 6e03bb14…`), and the mutation was reverted; `git diff` on
`evidence.rs` afterwards is empty and the test passes again.

## 3. Option A — normalize before hashing

Hash `bytes.replace("\r\n", "\n")` (or a full CRLF/CR folding rule) at observation time.

- **Gains.** Two checkouts of one commit mint one run id and one release id. L22 disappears as a
  reproducibility surprise, and the `require_clean_git = false` hole in §2 closes with it.
- **Cost, and it is the load-bearing one: the digest stops describing the shipped bytes.** The bundle
  copies the notes file as it found it and its own `SHA256SUMS` and `release-manifest.json` record that
  copy's digest; if the identity digest were of normalized bytes while the artifact is unnormalized, the
  release id and the manifest would report different numbers for one file, and
  `verify_bundle` (`bundle.rs:1652`, which re-derives the release id from the bundle's own bytes) would
  either fail or have to normalize too. The only self-consistent variants are therefore: normalize the
  **copy** as well — which means FirmwareSight rewrites a copy of a document a human wrote, inside a
  release artifact, silently — or accept an id that cannot be recomputed from what shipped. Both are worse
  than the problem.
- **Blast radius.** Every existing release id whose notes contain a `\r` is re-issued. Ids are
  content-derived, and stored `gate_runs` / `release_records` rows are immutable by trigger, so the old rows
  stay while new runs mint different ids: History then shows two records for what a user thinks of as one
  release, which is the failure mode §17's read-only page exists to avoid misleading about.
- **Governance.** This changes what an Observed fact records, which `AGENTS.md` §2 reserves for an ADR and
  §8 forbids reversing by convenience ("不得把推测写成 Observed" cuts both ways: do not record a normalized
  value as though it were measured).

## 4. Option B — identify by Git blob, not by worktree bytes

Hash `git hash-object` of the tracked path (Git's own LF-normalized blob identity) instead of the file.

- **Gains.** Identity becomes checkout-independent by construction, is reproducible by any reviewer with the
  repository, and matches the mental model "the release stands on this commit".
- **Costs.** It requires the notes file to be tracked in a Git work tree. The observation path today needs
  no Git at all for this fact (`observe_release_notes` is a `std::fs` read), the product already handles the
  no-repository case as an *unknown* rather than an error (`gate.rs:762-770`), and the release notes of a
  build directory or an exported document set have no blob at all. An untracked or locally modified file
  makes the blob hash and the shipped bytes disagree, which is option A's inconsistency arriving by another
  route. It would also add a second identity source to a design where each fact has exactly one (`L18`'s
  adjudication that `artifacts.path` is the storage of record went the same way).
- **Governance.** Same as A: a change to evidence class, hence an ADR.

## 5. Option C — keep bytes-as-evidence, and make it legible (current behaviour)

No semantic change. What is asked for instead is that the surprise be visible where it is felt:

- the release id's own canonical text already carries the notes digest, so an investigator can always
  recover *why* two ids differ (`release.rs:363-370`);
- a project can pin its own bytes with a `.gitattributes` line for the notes path, and that is documentation
  for the user's repository, not a product behaviour;
- Commit F's `P5_KNOWN_LIMITATIONS.md` should state L22 in the terms of §2 above — including the
  `require_clean_git = false` caveat, which the G2 row does not currently carry — and Commit D's
  Diagnostics may name the notes digest it observed, since a digest of a user-owned document is already
  inside the Diagnostics allowlist question rather than outside it.

**This is the default if nobody approves anything, and it is the option P5 has implemented.**

## 6. Backward compatibility, for whichever option is chosen

Any change to what the notes digest measures re-issues ids for releases that were already made. There is no
migration that can fix this, because the ids live in immutable rows (`0003`/`0004` triggers abort an
UPDATE of `gate_runs`, `gate_findings` or `release_records`) and in bundles already handed to third parties.
The practical consequence: after a change, re-running the Gate on an old project produces a *new* run rather
than reproducing the stored one, and any external reference to an old `release-…` id remains correct about
the bytes it was minted from. Versioned identity semantics would need a schema or canonical-text version
field to be discoverable by a reader of an old bundle; the release manifest's `schema_version` and the
`schemas=` line in the canonical text (`release.rs:391-398`) are where that would have to be recorded, and
neither today distinguishes "normalized" from "raw" identity.

## 7. Questions for the Architect

1. Is a release's identity the bytes a human handed over, or the document those bytes encode? §3–§5 are
   three answers to that one question, and the choice should be stated in `AGENTS.md` §8's vocabulary.
2. If bytes: should the `require_clean_git = false` path be allowed at all, given that it is the only route
   by which the id moves silently? Narrowing a documented policy option is itself an `AGENTS.md` §2 act.
3. If Git blobs: what is a release built from an untracked notes file, and which of the four evidence
   classes does its digest belong to?
4. Does any externally-held bundle from before the change need to remain re-verifiable by a future
   version, and is `04_TECH/18`'s build-identity record the right place to state the answer?

## 8. STOP

Per §32, this sub-change stops here. P5 does not implement option A or B, does not normalize, does not
change `observe_release_notes`, `canonical_input`, `canonical_release_text`, either id derivation, or any
stored row. P5 continues with the unrelated workstreams the same prompt authorizes (diagnostics, recovery,
compatibility fixtures, documentation), and the next word on this file is the Architect's.
