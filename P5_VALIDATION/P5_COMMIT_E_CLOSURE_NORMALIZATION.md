---
title: "P5 Commit E closure normalization"
doc_id: "FS-P5-COMMIT-E-NORMALIZATION"
product: "FirmwareSight"
version: "1.0"
status: "RESOLVED_BY_ARCHITECT"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-04"
---

# P5 Commit E closure normalization — evidence vocabulary and the L15 decision

Documentation-only round under *FirmwareSight — P5 Commit E Closure Normalization (Evidence Vocabulary + L15
Architect Decision), Execution Prompt v1.0 — Architect Reviewed*. It changes no product source, no fixture byte,
no schema, no migration, no dependency and no workflow, and it decides nothing about L26.

## 1. Start authority

Read before the first write, not after:

| check | measured |
| --- | --- |
| `git fetch --prune origin` | performed |
| `git rev-parse HEAD` | `6981625a6491aeb82fd6346d17f219dcb413548b` |
| `git rev-parse origin/main` | `6981625a6491aeb82fd6346d17f219dcb413548b` — equal, so no stop-for-moved-remote |
| `git status --short`, `git diff`, `git diff --cached` | all empty |
| `git worktree list` | one worktree, `main` |
| Commit E engineering candidate / its run | `59d85c3` — Run `37228929762`, attempt 1, completed / success, 10 of 10 |
| Commit E read-back head / its run | `6981625` — Run `37230689636`, attempt 1, completed / success, 10 of 10 |
| P5 / product / baseline | `IN_PROGRESS` / MVP CANDIDATE / `0.6.0` |

**The prompt's own bytes, and why there are two hashes.** Delivered on 2026-10-04 as 28,738 bytes with 1,372 CRLF
terminators and one unterminated final line: SHA-256
`9571df2cc08107ea134ed89acc4a984254803b40e451c57c6e9c3ca480075599`. Archived as
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitE_Closure_Normalization_v1.0.txt` at 27,366 bytes with 1,372 LF
and the same 1,373 content lines: SHA-256
`62040e3d02fffe0f7682909828e4a2d6a91d81b09a937c0bd0b4765e6c7af887`.

Nothing was reconstructed. `cp` then `cmp` proved the copy byte-identical at archive time; the repository's own
`.gitattributes` rule `*.txt text eol=lf` then normalized the terminator, and the result was checked line by
line against the delivered file — 1,373 lines each, content `identical: True`. `.gitattributes` was **not**
edited to preserve the CRLF: §1 of this prompt does not allow a line-ending or integrity-policy change, and
`AGENTS.md` 9 puts one in front of a human. Consequence stated rather than buried: the delivered CRLF hash does
not reproduce from a checkout of this directory. The other thirteen archived prompts arrived LF, so this is the
first time the two hashes differed here.

Because a tracked path was added, `DIRECTORY_TREE.txt` was regenerated first and `SHA256SUMS` **last**, after
every other file edit, under the current convention only.

## 2. Why normalization was required

Commit E's engineering evidence stands: 868 Rust tests, 217 UI tests, 14 fixture tests, five mutations, a clean
worktree proof, 10 of 10 twice. Two of its **documents**, though, had broken vocabularies their own prompt had
fixed — and a status column that means six things in five rows means nothing in any of them. Both were label
failures, never measurement failures, which is why this round edits tables and not tests.

Measured with a table parser rather than by eye, so "before" is a number too:

| table | cells checked | outside its required vocabulary, before | after |
| --- | --- | --- | --- |
| `P5_COMPATIBILITY_MATRIX.md`, every column headed `status` | 39 | **11** | 38 cells, **0** |
| `P5_COMPATIBILITY_FIXTURE_REPORT.md` §4, the A–H `disposition` column | 8 | **8** | 8 cells, **0** |

The matrix's cell count drops from 39 to 38 because one row left the status column entirely; nothing was
deleted.

## 3. Compatibility Matrix vocabulary corrections

Required set: `SUPPORTED / SUPPORTED_WITH_LIMITS / CI_BUILD_ONLY / NOT_TESTED / UNSUPPORTED`. No sixth value.
Every replacement below moved the row's own qualifier into the evidence column; no row's underlying claim was
upgraded, and three were made narrower because the row's own text already described a limit.

| row | before | after | why the fact is unchanged |
| --- | --- | --- | --- |
| Other Linux distributions | `BEST_EFFORT_NO_CLAIM` | `NOT_TESTED` | Tier 3 of `04_TECH/20` *is* a no-claim tier; the note now says the absence of a claim is the claim. |
| Windows ARM64, Linux ARM64 | `DEFERRED_NO_MVP` | `UNSUPPORTED` | Deferred / future roadmap / outside MVP moved to the evidence note, as §5 directs. |
| Windows x64 measured host | `SUPPORTED` | `SUPPORTED_WITH_LIMITS` | The row already said one host, one WebView2, 100 % DPI — that is L4's residue, so the limit belongs in the status word too. |
| Clang with an explicit Arm target | `SUPPORTED` for the tested target and layout cohort | `SUPPORTED_WITH_LIMITS` | Same sentence, now in two places that each mean one thing: the cohort is the limit. |
| host MinGW `gcc`/`g++` | `NOT_TESTED` for the product | `NOT_TESTED` | "for the product" relocated to the note. |
| DWARF semantic analysis | `UNSUPPORTED` (not implemented) | `UNSUPPORTED` | "not implemented" relocated to the note. |
| D DMA / custom writable region | `SUPPORTED` for the region, `NOT_CLAIMED` for cacheability | `SUPPORTED_WITH_LIMITS` | Region accounting supported, cacheability not claimed — both halves stay in the note; `NOT_CLAIMED` is not a status. |
| F ELF with debug info | `SUPPORTED` as presence, `UNSUPPORTED` as semantics | `SUPPORTED_WITH_LIMITS` | Presence accepted, semantics unread; the separate DWARF row keeps saying `UNSUPPORTED`. |
| Stripped with `objcopy -S` | `MEASURED, NOT INFERRED` | `SUPPORTED` | "measured on this host, not inferred from documentation" is evidence about the row, not a status. |
| Desktop app installed on Windows x64 | `SUPPORTED` with measured limits | `SUPPORTED_WITH_LIMITS` | Same reason as the platform row above. |
| Application package on macOS / Ubuntu | `BUILT_AND_VERIFIED_IN_CI`, not installed | `CI_BUILD_ONLY` | A package build has never been a runtime claim; this is the file's own distinction, now spelled with the one word that means it. |
| Signing, notarization, updater | `READY_NOT_EXECUTED`, `UPDATE_READY_MANUAL` in a status cell | **moved out of the status column** | These are P5 §11/§12 release-readiness states. They now sit in a new §7b under a column called `state`, because forcing them into the five would have made a readiness state look like a compatibility verdict. |

Twelve rows are listed above, eleven were vocabulary violations: the "Windows x64 measured host" row already held
a legal value (`SUPPORTED`), and §6's `SUPPORTED_WITH_LIMITS` is where its own text — one host, one WebView2,
100 % DPI, which is L4's residue — had always belonged. Re-labelled, not corrected.

Validation, run against the file rather than against memory:

```text
P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md: {'status'} cells checked=38 violations=0
```

The parser walks markdown tables, selects only columns whose header is exactly `status`, strips one pair of
backticks, and compares to the five-value set. Run against the pre-normalization file from `HEAD` with the same
code it reports `cells checked=39 violations=11` and lists them — which is how the "before" number above was
obtained. No blind substitution was performed over prose: §23's own wording, §8's refusals and the evidence
columns still use ordinary English, and ordinary English is not a status value.

## 4. A–H disposition corrections

Required set: `PROVED_BY_EXISTING_FIXTURE / PROVED_BY_NEW_FIXTURE / SUPPORTED_WITH_LIMITS / NOT_AVAILABLE`.

| case | before | after |
| --- | --- | --- |
| A classic FLASH + RAM | `covered, no new fixture` | `PROVED_BY_EXISTING_FIXTURE` |
| B executable-in-RAM | `**new**` | `PROVED_BY_NEW_FIXTURE` |
| C external SRAM | `**new**` | `PROVED_BY_NEW_FIXTURE` |
| D DMA / custom writable region | `**new**, cacheability explicitly not claimed` | `PROVED_BY_NEW_FIXTURE`, cacheability note kept in evidence |
| E long MAP preamble | `**new**, real discarded sections` | `PROVED_BY_NEW_FIXTURE` |
| F ELF with debug info | `already covered (…); recognized, not consumed` | `PROVED_BY_EXISTING_FIXTURE` |
| G stripped / no debug | `**new**` | `PROVED_BY_NEW_FIXTURE` |
| H multiple `PT_LOAD` | `**new and named**` | `PROVED_BY_EXISTING_FIXTURE` |

F and H are the two that a careless normalization would have gotten wrong, and both were decided by evidence
rather than by which commit touched the file last. Every pre-Commit-E fixture was already a `-g` build, so F's
first proof predates this round and `p5-clang-arm` **deepens** it (8 debug sections, recognized, excluded).
`p0-dual-region` already carried 3 `PT_LOAD` segments and the `p2-diff` pair 5 and 4, so H's fixture evidence was
existing too: what Commit E added was the direct per-fixture assertion, and a new assertion on an existing
fixture is not a new fixture.

Two of the four allowed words went unused, deliberately and with the reason recorded in the file: nothing in A–H
is `NOT_AVAILABLE` (all eight have committed evidence), and D's and F's limits are limits of the **claim**, which
live with their status cells in the matrix (`SUPPORTED_WITH_LIMITS`) rather than being stated twice in two
vocabularies. Validation:

```text
P5_VALIDATION/P5_COMPATIBILITY_FIXTURE_REPORT.md: {'disposition'} cells checked=8 violations=0
```

## 5. L15 — the Architect's decision (Option E)

`P5_COMMIT_E_SCHEMA_DECISION.md` was `STOPPED_FOR_ARCHITECT`. It is now `RESOLVED_BY_ARCHITECT`, and its §1–§10
are untouched: those sections are the measurement the decision was taken against, and rewriting them to agree
with the answer would destroy the only evidence that the stop was earned. §11 is new and records it.

**Option E — LEGACY WIRE IDENTIFIER PRESERVED, ACCURATE PRESENTATION / DOCUMENTATION**, an intentional fifth
shape beyond A/B/C/D:

| kept | refused |
| --- | --- |
| `SourceType::ElfProgramHeader` | a new `ElfSectionFlags` member |
| the `analysis:1` wire token `"elf.program-header"` | renaming the enum member |
| existing `evidence.source_type` rows holding `ElfProgramHeader` | renaming the wire value |
| `analysis:1` unchanged | rewriting persisted history |
| Release Bundle `analysis.json` contract unchanged | **migration 0006** |
| goldens unchanged | **`analysis:2`** |
| `SCHEMA_VERSION` unchanged (`crates/firmwaresight-storage/src/db.rs:14` reads `5`) | moving a golden byte |
| release identity unchanged, `ADR-0028` untouched | changing Bundle semantics |

The semantic definition that replaces the accuracy the name never had: `elf.program-header` is a **legacy
compatibility identifier** inside `analysis:1` and must not be read as a guarantee that the evidence came from an
ELF `PT_*` program header. For the current `MemoryEvidenceBasis::ElfAddressAndFlags`, the accurate human-facing
description is **ELF address + flags evidence**. Three rules follow: do not pretend the identifier is literally
accurate, do not redefine what published bundles say, do not rewrite persisted history.

Where the accurate wording already exists, measured rather than assumed:
`apps/desktop/ui/src/Compare.tsx:690` already maps `ElfAddressAndFlags` to the caption
`ELF address/flags evidence` — the prompt's phrase with a slash instead of a plus, which §10 says to leave alone
because it is already accurate.

Where it is not, recorded rather than fixed: the Evidence Inspector prints the stored token verbatim —
`crates/firmwaresight-storage/src/db.rs:507` writes `format!("{:?}", …)`, `query.rs:503` reads it back as a
`String`, `apps/desktop/src-tauri/src/details.rs:223` forwards it, `apps/desktop/ui/src/Details.tsx:667` renders
`row.sourceType`. A user expanding such a row reads `ElfProgramHeader`. That is **user-visible**, so §10 makes it
a Commit F item and this round touches no code: the fix there is a display-only caption table in the shape L20
already used, moving zero stored bytes.

Documentation of the legacy semantics itself lives in `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §7, added as a
dated section that states it changes no earlier clause — because that document's §4 item 3 is the precedence rule
whose label is loose. `04_TECH/02_DOMAIN_MODEL.md`, which lists `EvidenceItem.source_type`, was **not** edited:
it is `status: BASELINE`, `AGENTS.md` 2 puts an artifact evidence classification change behind an ADR, and §1 of
this round does not buy that permission.

## 6. L15 final disposition

`P5_SUPPORTABILITY_REPORT.md` §1 and §2 now read:

```text
L15 = CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER
```

Not `CLOSED`. The identifier stays technically inaccurate inside a published contract forever; what changed is
that the inaccuracy is bounded, defined and documented instead of merely observed. A true rename remains
possible only inside a future `analysis:2`, which is a major-contract decision this round neither makes nor
previews. §48's own expected direction for L15 was `CLOSED only if narrow contract-safe wording fix`; the
measured tree showed no such fix exists, and the Architect's answer confirms that rather than overriding it.

## 7. Explicit no-code / no-schema impact

Every row §13 asks to verify, with the command or file that shows it:

| contract | state | how it was checked |
| --- | --- | --- |
| `analysis:1` | UNCHANGED | `schemas/analysis.schema.json:616` still lists `"elf.program-header"`; the file is not in the diff (`git diff --cached --name-only` shows zero paths under `schemas/`) |
| `diff:1`, `gate-results:1`, `accepted-reviews:1`, `release-manifest:1`, `project-config` | UNCHANGED | same — `ls schemas/` lists six files, none in the diff |
| SQLite schema / `SCHEMA_VERSION` | UNCHANGED at `5` | `crates/firmwaresight-storage/src/db.rs:14`; migrations remain `0001`–`0005`, and **no `0006` file exists** |
| goldens | UNCHANGED | `drift/goldens unchanged` step PASS (`git diff --exit-code -- golden`), and the four rows carrying `elf.program-header` in `golden/cli/p0-basic-analyze.json` still do |
| generated TS bindings | UNCHANGED | `drift/ipc bindings` and `drift/ipc bindings unchanged` both PASS |
| fixture bytes | UNCHANGED | zero paths under `fixtures/` in the diff; `drift/fixtures tracked` PASS over all 56 manifest paths |
| dependency sets | UNCHANGED | `Cargo.lock` and `apps/desktop/ui/pnpm-lock.yaml` absent from the diff; `deny/cargo-deny` PASS |
| capabilities / IPC surface | UNCHANGED | zero paths under `apps/` in the diff |
| release identity, `ADR-0028` | UNCHANGED | `09_ADR/ADR-0028-…md` absent from the diff, still `status: BASELINE`; L26 and the baseline-checksum semantics were not touched either |
| workflow | UNCHANGED | zero paths under `.github/` in the diff |

Direct test counts, which §19 fixes at 868 / 217 and treats any movement as an alarm, came back exactly that:
see section 10.

## 8. L26 carry-forward

Unchanged and undecided, as §14 requires. `SHA256SUMS` is still generated from **working-copy** bytes under
`core.autocrlf=true` with `text eol=lf`, so it verifies on the host that wrote it and can disagree with a clean
checkout elsewhere. This round did exactly one thing with it that §21 allows: regenerated it under the current
convention, as the last write, because the tracked path set grew by the archived prompt
(`DIRECTORY_TREE.txt` first, then `SHA256SUMS`), and verified it with `scripts/verify_baseline_artifacts.py`
(`RESULT PASS`) and `sha256sum -c`.

What is **not** claimed: that those bytes are portable across checkout line-ending transformations. What was
**not** done, on purpose: no switch to Git blob bytes, no canonical checkout-byte definition, no baseline
verifier wired into CI, no `ADR-0028` amendment for repository baseline checksums. L26 stays
`CARRIED_FORWARD TO COMMIT F / ARCHITECT`.

One new wrinkle is recorded rather than smoothed: the archived prompt is the first evidence file in this
repository whose delivered bytes were CRLF and whose stored bytes are LF, so it joins the set of paths where the
working copy and a foreign checkout can disagree. It is a `.txt` under `text eol=lf`, which is the rule doing its
job, not a new class of defect — but a reader checking hashes across machines should expect
`9571df2c…075599` from the delivered file and `62040e3d…e6c7af887` from the archive.

## 9. §64 carry-forward

Not executed, and not this round's to execute. No install, no Analyze, no Compare, no Gate, no Bundle, no
History, no Diagnostics walk, no close/reopen, no reinstall, no uninstall. **No package was installed and the
owner's live store was never opened** — §15 forbids a park/restore in a documentation-only round. Measured here
to prove the absence rather than assert it:

```text
%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite
  155,648 bytes  sha256 d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468
  last written 2026-09-30, equal to the value recorded in P5_DIAGNOSTICS_RECOVERY_REPORT.md:35
```

`§64 = OPEN FOR COMMIT F`, together with the three §16 documents that Commit F still owns
(`P5_MIGRATION_RECOVERY_REPORT.md`, `P5_HISTORY_DIAGNOSTICS_REPORT.md`, the final consolidated
`P5_KNOWN_LIMITATIONS.md`), the P5 final report and exit checklist, and the signing, notarization and
manual-upgrade readiness records. None was created here to tick a filename.

## 10. Local validation

All run on the final tree, with the documentation changes staged. The scratch validator of section 3/4 was re-run
last, after the tables were final.

| command | result |
| --- | --- |
| `git diff --check` / `git diff --cached --check` | clean, exit 0 |
| `cargo fmt --all -- --check` | exit 0, no output |
| `cargo test --workspace` | **868 passed / 0 failed across 47 targets** |
| `corepack pnpm test` (in `apps/desktop/ui`) | **217 passed / 0 failed in 8 test files** |
| `python scripts/check.py` | **16 of 16** — rust 3, frontend 5, drift 7, deny 1, every step PASS, no `SKIP` |
| `python scripts/check.py --only drift` | **7 of 7** |
| `python scripts/check.py --only deny` | **1 of 1** (`advisories ok, bans ok, licenses ok, sources ok`) |
| `python scripts/check.py --only package` | **4 of 4** (frontend deps, cli companion, desktop package, artifacts verified) |

No group was skipped and no repetition was multiplied into a count: 868 and 217 are the direct totals, unchanged
from Commit E because this round adds and removes no test, which is precisely the check §19 asks for.

## 11. Diff allowlist

The full changed-path set is documentation, governance, audit and integrity only:

```text
.ai/ACTIVE_TASK.md            .ai/CURRENT_STATE.md        .ai/DECISIONS.md
.ai/HANDOFF.md                .ai/README.md               04_TECH/23_MEMORY_ACCOUNTING_MODEL.md
10_AUDIT/SOURCE_PROMPTS/FirmwareSight_P5_CommitE_Closure_Normalization_v1.0.txt
10_AUDIT/SOURCE_PROMPTS/README.md                         INDEX.md
README.md                       P5_VALIDATION/P5_COMMIT_E_SCHEMA_DECISION.md
P5_VALIDATION/P5_COMPATIBILITY_FIXTURE_REPORT.md          P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md
P5_VALIDATION/P5_SUPPORTABILITY_REPORT.md                 P5_VALIDATION/P5_COMMIT_E_CLOSURE_NORMALIZATION.md
DIRECTORY_TREE.txt (regenerated first)                    SHA256SUMS (regenerated LAST)
```

Proved by `git diff --name-only` filtered against §1's forbidden list:
`crates/`, `apps/`, `scripts/`, `fixtures/`, `schemas/`, `golden/`, `.github/`, `Cargo.toml`, `Cargo.lock`,
`package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml`, `migrations/**`, any generated TS binding, any
ELF/MAP/binary fixture — **zero hits**. `04_TECH/23` is the only technical-baseline file touched, and only to add
the dated §7 note §5 of this document explains; no enumerated value in it changed.

## 12. Commit SHA

One commit, as §25 requires, with the message *P5: normalize Commit E evidence and record the L15 decision*. A
commit cannot carry its own hash without a second commit, which §27 forbids, so this section names the head
rather than inventing a value: it is the head `git rev-parse HEAD` reports on `main` after 2026-10-04 whose
subject is that message. `6981625` is its parent, and `59d85c3` / `859648e` are the Commit E pair before it.

## 13. Remote CI

**§13a, read afterwards (written by Commit F1 on 2026-10-05, the head after this one).** This head could not
record its own run, and §27 forbade a further commit whose only content would be that record — so the run
became readable only from its successor, which is this paragraph. Run `37262147348` (#67) at headSha
`80b47c4…`: **attempt 1 completed/failure, 9 of 10** — the single red `Generated output drift`, where rustup on
the Ubuntu runner logged "recovering from a partially installed toolchain" and then
`failed to install component: 'clippy-preview-x86_64-unknown-linux-gnu', detected conflict: 'bin/cargo-clippy'`,
rolling back about half a second in, before any crate was compiled. The raw log was read before the rerun, and
corroborated by the same SHA's green `Rust (ubuntu-latest)` and `Package Ubuntu`, which bootstrap the same
toolchain — the provisioning class `BASELINE.yaml:514` already records for P4's `157f749`, not a repository
content failure. One `gh run rerun --failed` followed: drift returned **7 of 7** and the run concluded
**10 of 10 success**. Attempt 2 is one re-executed job plus nine attempt-1 executions carried into the attempt-2
object, and is not written here as first-attempt green. §14's verdict line below therefore resolves to
**`COMMIT_E = FINAL PASS / COMPLETE`**, with the row-level authority in `P5_CI_AUTHORITY.md`.

```text
Remote CI = PENDING
```

Written before the run existed, because §24 forbids fabricating it. Read it back with
`gh run list --repo 2023violet/FirmwareSight --json headSha,status,conclusion` filtered to this head, then
`gh run view <id> --json jobs` and every one of the ten jobs inspected individually — Rust (windows-latest), Rust
(ubuntu-latest), Desktop UI (windows-latest), Desktop UI (ubuntu-latest), Generated output drift, Dependency
policy, macOS Core Smoke, Package Windows, Package Ubuntu, Package macOS — against the required **10 of 10**.
Per §27 the run is external final evidence: it is reported back to the Architect in chat and no further commit is
written merely to record it.

## 14. Verdict as it may be written right now

```text
Commit E engineering          = PASS                     (measured, 59d85c3, 10 of 10 on 37228929762)
Commit E evidence closure     = NORMALIZATION_PENDING_REMOTE_CI
P5                            = IN_PROGRESS
Product state                 = MVP CANDIDATE
baseline                      = 0.6.0
Commit F                      = NOT_AUTHORIZED
L15                           = CARRIED_FORWARD — LEGACY_WIRE_IDENTIFIER (decided: Option E)
L26                           = CARRIED_FORWARD TO COMMIT F / ARCHITECT
§64                           = OPEN FOR COMMIT F
Open-source licence           = PENDING OWNER CONFIRMATION
```

Once this head reads 10 of 10, §18 permits exactly one change here: `Commit E = FINAL PASS / COMPLETE`. Nothing
in this round licenses `P5 PASS`, `Productization COMPLETE`, `BETA`, `RC`, `GA` or `Production Ready`, and the
security sentence stays *dependency policy passes with documented accepted risks*.
