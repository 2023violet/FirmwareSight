---
title: "ADR-0029 Repository Baseline Checksums Use Git Index Blobs"
doc_id: "ADR-0029"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-05"
---

# ADR-0029 — Repository Baseline Checksums Are the Canonical Git Stage-0 Index Blob Bytes

## Status

Accepted. The question was carried as limitation L26 (`P5_VALIDATION/P5_SUPPORTABILITY_REPORT.md`,
`P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md` §G) through five stages, was explicitly forbidden as a fix in
Commits D and E, and is decided here by *FirmwareSight — P5 Commit F Final Productization Closure, Execution
Prompt v1.0 — Architect Reviewed* §5, which states the rule in its canonical form:

> THE ROOT REPOSITORY `SHA256SUMS` REPRESENTS THE SHA-256 OF CANONICAL GIT STAGE-0 INDEX BLOB BYTES, NOT THE
> RAW BYTES CURRENTLY PRESENT IN THE WORKING DIRECTORY.

At a clean committed checkout, stage-0 index blob bytes equal the `HEAD` tree blob bytes. At pre-commit
generation time, stage-0 index blob bytes are the exact canonical bytes intended to be committed. That is the
repository **source-baseline** domain, and it is deliberately separate from ADR-0028's release-evidence
domain; see *Relationship to ADR-0028* below.

This ADR supersedes no earlier ADR and amends none. It changes repository tooling semantics, not product
behaviour: no firmware artifact byte, no MAP evidence byte, no Release Notes byte and no Release Bundle byte
is normalized, re-hashed or re-identified by this decision.

## Context

`SHA256SUMS` at the repository root is the baseline manifest: one `<sha256>  <path>` line per tracked file,
produced by `scripts/generate_baseline_artifacts.py sums` and re-checked by
`scripts/verify_baseline_artifacts.py`. Its path set came from the index (`git ls-files`), while its digests
came from the working directory (`open(path, "rb")`). Those are two different authorities about the same
file, and on a host where Git transforms content at checkout they disagree.

This host is such a host. `core.autocrlf` is `true` at both system scope (`D:/Git/etc/gitconfig`) and global
scope (`~/.gitconfig`), and `.gitattributes` declares `text eol=lf` for the document and source extensions.
Measured at the start of this round:

| quantity | measured |
| --- | --- |
| tracked stage-0 entries | 693 |
| unmerged (stage ≠ 0) entries | 0 |
| entries listed in `SHA256SUMS` | 690 |
| paths whose working-directory bytes differ from their stage-0 blob | **13** |
| those 13 lines' recorded digest equal to the working-copy digest | 13 of 13 |
| those 13 equal to the blob digest | 0 of 13 |

The same comparison done against **committed trees**, head by head, is the reason the old wording could not
be kept:

| head | manifest lines disagreeing with the committed blob |
| --- | --- |
| `3400981` | 19 |
| `90aa69d` | 19 |
| `57a904e` | 14 |
| `bfbc1aa` | 14 |
| `859648e` | **17** |
| `59d85c3` | 13 |
| `80b47c4` | 13 |

Five different values across seven consecutive heads, with no policy change in any of them. Every document
that quotes "17" quoted the value true when it was written; none of them was wrong at the time, and all of
them were wrong later. That is what an unstable identity produces.

Two further measured facts made the defect invisible rather than merely untidy:

- The verifier read the same working copy as the generator, so its `RESULT PASS` was agreement with the host
  that wrote the file, not evidence that the manifest describes the repository.
- **No CI job and no `scripts/check.py` step ran the verifier at all.** Nothing outside this machine ever
  disagreed.

## Decision

**Root `SHA256SUMS` entries are the SHA-256 of canonical Git stage-0 index blob content.**

Consequences, accepted deliberately:

- The generator hashes index blobs and never opens a working-directory file as a checksum authority. It reads
  the index through `git ls-files -s -z` (paths as raw bytes, so non-ASCII and space-bearing names survive)
  and fetches each blob's content through one batched `git cat-file --batch` call rather than a subprocess per
  file.
- The generator **refuses to write** a manifest while Git reports a semantic unstaged change to any tracked
  path. Forgetting `git add` before regenerating is the failure mode this closes; it now fails loudly, naming
  the paths.
- The distinction between "dirty" and "differently represented" is Git's, not the tool's. A working file whose
  clean-filtered form equals the index blob is a clean file. The guard asks `git diff`; it does not inspect
  raw EOL bytes and call them dirty.
- An unmerged index (any stage ≠ 0) is a hard refusal. A manifest written over a conflicted index would certify
  neither side.
- A new file is outside the baseline until it is staged, and inside it once it is. The manifest describes the
  commit being assembled, not the folder it is generated from.
- The verifier independently re-derives index blob content — through a different Git path (`:<path>` rather
  than a blob OID), with no import of and no shared hashing helper with the generator — and reports digest
  mismatch against the index, unmerged index entries, and whether the manifest and tree it is reading are
  byte-identical to their own index blobs.
- Baseline integrity becomes an authoritative CI claim: `scripts/check.py`'s `drift` group runs the verifier,
  so every push checks the manifest against the index on the Windows, Ubuntu and macOS runners. The
  authoritative job set stays ten jobs; the drift job gains one real step and no eleventh job is created.
- `sha256sum -c SHA256SUMS` against an arbitrary working copy is **no longer a valid verification command**, and
  on this host it now reports the 13 transformed files as failures. That is the decision working, not a
  regression. The canonical external proof is the archive form:
  `git archive <commit>` extracted outside the repository, then `sha256sum -c SHA256SUMS` over the extracted
  tree, which carries canonical blob content and is expected to be all-OK.
- Three checksum domains stay distinct and are never conflated: the repository baseline manifest (this ADR),
  the distribution package `SHA256SUMS.txt` (raw artifact bytes, a download check), and the Release Bundle's
  `SHA256SUMS` (raw release evidence, ADR-0028).
- The basename exclusion rule in both tools is retained, with its recorded consequence: the tracked file
  `golden/reports/p4-release/SHA256SUMS` gets no root-manifest entry, because it is a Release Bundle fixture in
  another domain and is self-excluded by that rule. It stays listed in the audit.

### Generation sequence this ADR makes mandatory

```text
edit → confirm no unrelated changes → stage intended changes
     → tree (if the tracked set changed) → stage DIRECTORY_TREE.txt
     → sums (from index blobs) → stage SHA256SUMS
     → verify → full gate → inspect staged diff → commit
```

Staging the tree before generating the manifest is not ceremony: the manifest hashes the tree's index blob, so
an unstaged tree would be certified at its old content.

## Relationship to ADR-0028

ADR-0028 says release identity uses **the exact bytes observed on disk**, and explicitly rejects identifying
release evidence "by Git blob". This ADR adopts index-blob identity. The two are not in tension because they
answer different questions about different objects:

| | ADR-0028 | ADR-0029 |
| --- | --- | --- |
| domain | a firmware release's evidence | this repository's source baseline |
| object under the digest | a file that shipped inside a Release Bundle | a tracked path in a commit being assembled |
| question the digest answers | "were these the bytes that released?" | "what content is about to be committed?" |
| why this identity rule | the bytes on disk are the evidence the product releases | the baseline must survive three OSes, two EOL policies, a clean worktree and an archive |
| normalization | none introduced; LF and CRLF are distinct releases | none introduced; the index's canonical form is what Git already committed to storing |

ADR-0028 is not amended, not superseded and not weakened by this record. Nothing in ADR-0029 normalizes a
firmware artifact, MAP evidence, Release Notes or a Release Bundle, and no code path in
`crates/firmwaresight-project/src/fingerprint.rs` or `evidence.rs` is touched by it. A future change to
release identity would still need a numbered successor to ADR-0028, per that ADR's own rule.

## Alternatives

**Option A — keep hashing working-directory bytes.** Rejected. It is the defect: a digest set that varies with
the writing host's checkout configuration, a verifier that agrees with itself, and a count that moved 19 → 14
→ 17 → 13 across seven heads without anyone deciding anything.

**Option B — hash `HEAD` tree blobs instead of the index.** Rejected. The manifest must be generated *before*
the commit that contains it, and `HEAD` cannot describe a staged new file at all — the very situation a
closeout is in. At a clean checkout the two are the same object, so B buys nothing that the index does not
already provide, while making the documented generation order impossible.

**Option C — declare a normalization policy, or edit `.gitattributes` so the working copy stops transforming.**
Rejected. It changes every contributor's checkout semantics to satisfy one manifest, and FirmwareSight does
not edit `.gitattributes` to make a checksum agree — the same reasoning ADR-0028 records for a project's
attributes. `AGENTS.md` §9 also puts an integrity-policy change of that kind behind a human decision, and the
decision taken here is deliberately about the *artifact's* semantics instead.

**Option D — drop the root manifest, or exclude the transformed files from it.** Rejected. The manifest is the
baseline's identity artifact; shrinking its coverage to the files whose representation happens to be stable
would encode the defect into the file set and leave the tree claim (`DIRECTORY_TREE.txt`) unverifiable.

**Option E — record both digests (worktree and blob) per file.** Rejected as over-engineering with two answers
to one question. A second digest column would need a rule for which one gates, would double the manifest's
size, and would still leave the CI-absence problem untouched.

## Consequences

Positive:

- One stable answer. A clean checkout, a `core.autocrlf=true` host, a `core.autocrlf=false` clone, a detached
  worktree and a `git archive` extraction all verify against the same 690 digests.
- The claim the artifact makes is the claim the repository needs: "this commit's tracked content hashes to
  these values", checkable by anyone who can fetch the commit.
- A forgotten `git add` now fails the generator instead of silently certifying stale content — the mistake this
  project has already made while regenerating these two artifacts.
- CI can see all of it, because the verifier is now a drift-gate step on three platforms.
- The stale-count class of documentation error is bounded: a measured number in a manifest line is no longer a
  moving target, so future prose can cite the manifest instead of re-measuring a host.

Negative, and accepted:

- `sha256sum -c SHA256SUMS` on a working tree is no longer a correct instruction, and on this host it fails on
  13 files by design. Every document that told a reader to run it has to be re-scoped to the verifier or to
  the archive form, and a reader who does not read that sentence will see red output and think the baseline is
  corrupt.
- The tools now depend on Git plumbing (`ls-files -s -z`, `cat-file --batch`) rather than on the filesystem,
  so they cannot be pointed at a directory that is not a Git checkout. That is a real loss of flexibility for
  an artifact whose entire subject is a Git commit.
- Content staged but not yet committed is certified by the index, so a generator run and a later commit are
  only equivalent if nothing was staged in between; the gate order above is the discipline, not an option.
- Baseline digests for the 13 transformed files change value once, in this round's regeneration. Anyone
  diffing `SHA256SUMS` against an older release of the repository sees 13 changed lines whose file content did
  not change — measured at this head, where 27 lines moved in total: 14 because this round edited those files,
  13 because the rule now names the blob instead of the checkout. That is exactly what this ADR says those
  lines always should have meant.
- The `golden/reports/p4-release/SHA256SUMS` exclusion by basename stays, so "every tracked file is in the
  manifest" remains false by rule and must keep being stated as such.

Executable statement of this ADR:

- `scripts/verify_baseline_artifacts.py` is its contract, and
- `python scripts/check.py --only drift` runs it as the `drift/baseline integrity` step on every authoritative
  CI run. If either stops asserting index-blob agreement, this ADR is no longer implemented.

## Revisit trigger

Reopen this ADR if any of the following becomes true:

- the repository's `.gitattributes` policy changes in a way that alters what a canonical blob is, or the
  project adopts an `export-subst`/`export-ignore` rule that makes `git archive` stop carrying plain blob
  content (the archive proof would then need re-deriving rather than assuming);
- a baseline artifact is ever required to certify a directory that is not a Git checkout, or to certify
  working-directory bytes as such — that is ADR-0028's domain, not this one, and belongs in a separate record;
- the manifest grows a second digest column for another purpose, at which point "which digest gates" has to be
  decided rather than inherited;
- CI stops running the verifier, which would return the defect to its previous invisible state;
- a release-identity or bundle-verification change is proposed that touches repository baseline semantics,
  because the two domains are kept separate on purpose and a change that blurs them needs a new decision.
