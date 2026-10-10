# C1-U4 — Desktop + CLI + Installed Acceptance: the report the Architect reviews

Executed 2026-10-10 as stage 3 of the four-file continuous-delivery packet. Checkpoint word, and the only one this
round may use:

`C1_U4 = LOCAL_USER_WORKFLOW_GATE_PASS / ARCHITECT_REVIEW_PENDING`

Product status word this round is allowed to propose, not to grant:

`C1_U4 = READY_FOR_ARCHITECT_PRODUCT_ACCEPTANCE` — pending independent review.

Not `C1 COMPLETE`. Not `PASS_COMPLETE`, not `USER_AVAILABLE`, not `SUPPORTED_WITH_LIMITS`, not beta, not RC, not GA,
not a commercial release. Nothing was pushed: `origin/main` is still `2859bb2` and the local heads on top of it are a
fast-forward away whenever the owner chooses. Pushing is the owner's call, by normal fast-forward, never force.

---

## 1. What this unit was for

`C1-U1` gave an attachment an identity, `C1-U2`/`C1-U2R1` let attached bytes answer a Gate rule and enter a verified
bundle, and `C1-U3` put that into the portable contract. At `3cfbc03` none of it was reachable: no UI section, no
`--attach`, no IPC command, and `BIN_HEX_RELEASE_ATTACH` read `DESIGN_APPROVED / NOT_USER_AVAILABLE`. U4 is the unit
that puts the control in front of a person and then proves it in an **installed** build rather than a dev server,
because `04_TECH/28` §13 says so in those words.

Six sub-units ran, in order, each with its own commit or its own line in one:

| id | what landed | commit |
| --- | --- | --- |
| U4a | repeatable `--attach <FILE>` on `gate` and `release prepare`, exit codes unchanged | `d95acea` |
| U4b | four desktop IPC commands, session-local attachment handles, the set threaded into run/prepare/export | `d95acea` |
| U4c | the "Release attachments" section, §9's copy verbatim, UI tests | `d95acea` |
| U4d | the conditional report limits line, the storage invariant sentence, docs | `d95acea` |
| U4f-1 | the readiness table now says **which set** carried each required kind | `073c83c` |
| U4f-2 | the bundle preview gives an attachment the role `release-attachment`, not `artifact` | `073c83c` |
| U4e | the installed walk this document reports | this round, docs-only |

U4f is the honest part of the story: **the installed walk found two defects that no unit test, no UI test and no
library test had found**, and both are in §13's frozen copy's territory. They are §4 below.

## 2. Authority and entry condition

| item | value |
| --- | --- |
| prompt | 《FirmwareSight — C1-U4 Desktop + CLI + Installed Acceptance, Architect task prompt v1.0》, stage 3 of 3 |
| archived | `10_AUDIT/SOURCE_PROMPTS/FirmwareSight_C1_U4_Desktop_CLI_Installed_Acceptance_v1.0.txt`, registered with its digest proved against the received zip in the stage-0 entry |
| design | `04_TECH/28` §9 lines 385–408 (frozen per-surface copy), §13, §14; `ADR-0030` D-3 |
| plan | `C1_VALIDATION/C1_U4_USER_WORKFLOW_PLAN.md`, written before the first product edit |
| entry condition | stage 1 and stage 2 local objective checkpoints green at `920e764` then `3cfbc03`, 17/17 at each, clean tree, `main` ahead of `origin/main` with no divergence |

The packet's own line — *"do not interpret a coding agent's SELF-REVIEW as final architect PASS"* — was applied to
this round's predecessors rather than to itself: `C1_U2R1` and `C1_U3` still read `ARCHITECT_REVIEW_PENDING` in
`BASELINE.yaml`, and this document does not upgrade either one.

## 3. The environment decision, made before anything could touch the machine

Windows Sandbox is unavailable on this host (no `WindowsSandbox.exe`; the optional-feature query needs elevation,
which this round does not take unilaterally — `AGENTS.md` §9). P5's fallback therefore ran again, and U4 §23's
conditions with it:

* the owner's live store was **hashed, parked and restored**, and never opened, upgraded, copied or mutated;
* every install ran in a **disposable test profile** with its own store;
* the test store was **kept as evidence** rather than deleted;
* no V1 cohort was reused or replaced. `11573661113` at `41bb6a36` still has 0 eligible external sessions.

Pre-park digests, from `STATE-pre.json`, and the post-restore digests, measured again after the uninstall:

| file | bytes | sha256 before | after restore |
| --- | --- | --- | --- |
| `firmwaresight-p0.sqlite` | 155,648 | `d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468` | identical |
| `firmwaresight-p0.sqlite-shm` | 32,768 | `fd4c9fda9cd3f9ae7c962b0ddf37232294d55580e1aa165aa06129b8549389eb` | identical |
| `firmwaresight-p0.sqlite-wal` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | identical |

Nothing in that directory was deleted at any point. The twelve other store files already there (the P0/P2/P4/G2 smoke
sets and their sidecars) and the two `postg2-e2e-*` directories were left exactly where they were.

## 4. What the installed walk found, and what was changed because of it

### 4.1 Defect 1 — a readiness table that contradicted the verdict above it

The first installed build (`7669096320…`, packaged from `d95acea`) ran the Gate to **PASS** with `bin` and `hex`
attached, and its own "Required artifacts" table read **`bin | Required | Missing`** and **`hex | Required |
Missing`** under a verdict stating `artifacts.required PASS` with `attachment:bin:3080a363…` in its evidence list.
That transcription is from the accessibility tree captured during the first walk, before the reinstall; the binary is
now uninstalled, so the reading cannot be re-taken here, and what is claimed is only the three cells named.

The table had decided the question for itself: it looked at the analyzed snapshot rows only, so an attachment could
satisfy a rule and the table beside it still said the kind was absent. `04_TECH/28` §9 forbids exactly that — the
bundle preview must show an attachment "exactly as for analyzed rows", and a surface must not restate a verdict
wrongly.

Fix, in `073c83c`, with the decision kept in one place rather than duplicated:

* Core gains `requirement_evidence(word) -> Option<(ArtifactKind, RequirementSource)>` and the
  `RequirementSource::{Analyzed, Attachment}` pair (`crates/firmwaresight-core/src/domain/gate.rs`). The
  `artifacts.required` rule now *reads that predicate* instead of owning the mapping, so the words and the table
  cannot drift again; a Core test pins each word to the one set that can satisfy it.
* `GateArtifactRowDto` gains `satisfied_by: String` — `analyzed`, `attachment`, or `none` — computed in
  `apps/desktop/src-tauri/src/release.rs` from the same predicate.
* `Release.tsx` renames the column from "Build" to "Satisfied by" and renders `Analyzed build` /
  `Release attachment` / `Missing`. The UI maps a word Core chose; it decides nothing.

### 4.2 Defect 2 — a bundle preview that labelled shipped attachments as analyzed artifacts

Same build, same walk: the preview listed `artifacts/image.bin` and `artifacts/boot.hex` with role `artifact`, the
role the analyzed ELF and MAP carry. A reader of that preview could not tell the two evidence classes apart, which is
the distinction the whole of C1 exists to keep.

Fix, also `073c83c`: `FileRole` gains `ReleaseAttachment` (`as_str() → "release-attachment"`) in
`crates/firmwaresight-project/src/bundle.rs`, and the shipped-row loop selects the role from the row's own
`SourceClass`. `FileRole` reaches the preview DTO and no portable document, so no schema, no golden and no digest
moved.

### 4.3 Both fixes, read back off the installed binary

Re-packaged from `073c83c` (`FirmwareSight_0.6.0_x64-setup-U4F.exe`, 3,962,441 bytes, sha256
`3d9a3512af82e04a4d2e0de5de94491130d2c7b3e247dd421bbf9aa64f25be25`, built 13:04:29 with 0 dirty paths), reinstalled,
and re-walked:

```
Kind  Policy    Satisfied by        SHA-256                Size
elf   Required  Analyzed build      4b4087e1...5ddd8374    17,464 bytes
map   Required  Analyzed build      a38575cd...4709ace6     4,664 bytes
bin   Required  Release attachment  3080a363...0cd666be     4,120 bytes
hex   Required  Release attachment  1d806a87...376973fc       716 bytes
```

and in the preview:

```
artifacts/boot.hex       release-attachment    716 bytes   1d806a87...376973fc
artifacts/firmware.elf   artifact            17,464 bytes   4b4087e1...5ddd8374
artifacts/firmware.map   artifact             4,664 bytes   a38575cd...4709ace6
artifacts/image.bin      release-attachment   4,120 bytes   3080a363...0cd666be
```

An attachment-free guard test asserts every `artifacts/` row still reads `artifact`, so the new role cannot leak
into a bundle that ships none.

## 5. The installed journey, in the order a person walks it

Real NSIS 3.11 per-user install driven through its own wizard UI, launched from the installed path
(`%LOCALAPPDATA%\FirmwareSight\firmwaresight-desktop.exe`, not a `target/` path), against a disposable firmware
project at `C:\Users\16429\AppData\Local\Temp\fs-u4e\project` (its own git repo, HEAD
`d6258d4c3757115a9961ef4f1445f0a3622cb1ce`, exact tag `v1.2.3`, clean).

| verb | what the installed build did |
| --- | --- |
| **Analyze** | native dialog titled "Choose the firmware artifact to analyze"; `project/build/firmware.elf` chosen; chips `ELF supported` and `MAP provided` green, `Git` neutral-unknown; FLASH `Exact 376 bytes`, RUNTIME RAM `Exact 76 bytes`, LOAD EVIDENCE "Admissible for a hard limit", EVIDENCE RECORDED `11 observed · 0 derived · 0 declared · 0 unknown`; footer prints the full snapshot id and `firmwaresight.analyze/p0-internal-1 · normalization p0-normalize-1 · fwsight 0.6.0` |
| **Compare** | the page was opened on the installed build and used to bring in the baseline artifact and its MAP (window title `FirmwareSight - Compare`, 19:22–19:23Z, the pre-fix instance); the baseline stored as `snap-3f615b62…64ed363e`, 256 B / 8 B. The delta figures the Compare page itself printed are **not transcribed in this document** — the +120 B growth is recorded here from the Gate's growth table on the fixed build, which reads "The deltas are the same ones Compare shows: Core computed them once and this page moves them." |
| **Release** | "Release attachments" section, `Attach` opening "Choose a BIN or Intel HEX file to ship with this release"; rows carry leaf, declared kind, byte size, **full** SHA-256 and the evidence class "Bytes observed · kind declared · provenance unknown"; `firmwaresight.toml` loaded through its own dialog; `Run Gate` → `PASS`, "Clear — 8 rule(s) pass, nothing is blocked and nothing waits on a review", 0 Block / 0 Review / 0 Unknown / 8 Pass / 2 N/A |
| **Bundle** | `Prepare bundle` → preview naming release, version, both snapshots, the Gate run, 0 accepted reviews and the proposed folder; `Choose destination folder` → "Choose the folder to create the release bundle in"; `Export bundle` → written, self-verified, and recorded |

§9's frozen sentences were read off the installed surface, not off a fixture: the Release caption
"Required `bin`/`hex` are satisfied by release files you attach; FirmwareSight records their bytes and SHA-256 and
does not analyze them."; the limitation line "Bytes verified; build provenance not verified. BIN/HEX contents are
not analyzed."; the Attach explanation naming `ADR-0030` and refusing to read a file as a container. The verbs on
that section are Attach and Ship; the word "Analyze" never appears as an action there.

The report HTML the installed app wrote carries §9's limits line, and only because something was attached:

> 2 release attachment(s) ship as byte evidence: their bytes and their SHA-256 digests are verified by this bundle,
> their structure and their build provenance are not verified, and nothing in this report analyzes them.

## 6. The three refusals the installed build produced, and what each one proves

Two of these three were not staged by anyone: the `6114` pair is what a restart does to a session, and the round
followed the app's own instruction to get past it. The third, `6102`, **was** provoked on purpose — the packet's
journey asks for a preview followed by a change followed by an export — but the code it produced was not the one this
round expected, and §6 says which one actually fired and why.

**6114, twice — a bundle is composed from a session, not from a store.** After the reinstall, `Prepare bundle`
answered:

```
What happened  That build was not analyzed in this session, so it cannot be packaged.
Code           ERR-BUNDLE-6114
Why we know    a bundle is composed from the analysis of build snap-4b4087e and re-reads
               its source files, both of which this shell holds for the current run only
What to do     Analyze that artifact again, run the Gate over it, then prepare the bundle.
```

After re-analyzing the current build, the same refusal came back naming **`snap-3f615b6`** — the baseline. So the
boundary is per-side: a bundle that carries a comparison needs both sides of it held by the session. The Gate had
still run to PASS from stored rows alone, which is the correct asymmetry: judging stored facts survives a restart,
re-reading source bytes does not. Both builds were then analyzed in-session and the plan computed.

**6102, once — the plan is bound to the context that produced it.** With the destination set and the preview on
screen, `image.bin` was modified on disk (4,120 → 4,126 bytes; `3080a363…` → `54e4b510…`). `Export bundle` refused:

```
What happened  the Gate run selected is `gate-a6cb8e84…` but the current project context
               recomputes to a different run id; the workspace, configuration or Release
               Notes moved after that run was recorded
Code           ERR-BUNDLE-6102
What to do     run the Gate again and review the new result
```

The mechanism was checked rather than assumed: `build/image.bin` is tracked in the disposable project's git repo, so
mutating it made `git status` report `M build/image.bin`, which changes the `git.clean` input, which changes the
canonical Gate input, which changes the run id. The recorded run therefore no longer described the project, and the
export refused on identity rather than on a digest comparison. `out-u4f` was **empty before and empty after** —
nothing partial was written. Restoring the bytes (`sha256sum` back to `3080a363…`, `git status` empty) and
re-preparing let the export complete.

**6115, from the CLI — an attachment that cannot be read is an input failure, not a crash.**
`--attach build/nosuch.bin` printed ``error: `nosuch.bin` is not a readable regular file … (code
ERR-BUNDLE-6115, operation op-3ee8-…)`` and exited **3**, the frozen input-import code.

## 7. CLI evidence, every exit code beside its command

Full transcript: `CLI_TRANSCRIPT.txt` in the evidence root (394 lines). Binary `fwsight 0.6.0`; exit
`0=ok 2=usage 3=input import 4=gate review 5=gate block 6=export`.

| invocation | verdict | exit |
| --- | --- | --- |
| `gate … --artifact … --map … --baseline … --baseline-map …` (no `--attach`) | BLOCK — "Required artifact kind(s) missing from the snapshot: bin, hex." | **5** |
| same + `--attach build/image.bin --attach build/boot.hex` | PASS — "…elf and map from the analyzed build; bin and hex from release attachments (bytes verified, provenance not verified)." | **0** |
| same + `--attach build/image.bin` only | BLOCK — "Required artifact kind(s) are in neither the analyzed build nor the release attachments: bin, hex." | **5** |
| `gate … --attach build/nosuch.bin` | ERR-BUNDLE-6115 | **3** |
| `gate … --attach build/payload.img` | BLOCK 2 — declared by name as `unknown`, so it satisfies nothing: `attachment:unknown:3080a363…` | **5** |
| `release prepare … --attach build/image.bin --attach build/boot.hex --out …` | "Release bundle written", 12 files / 157,134 bytes, 4 artifacts, integrity verified | **0** |
| `python scripts/verify_bundle_portability.py <cli bundle>` | 63 of 63 checks passed | **0** |

Two things in that table are worth the Architect's eye. First, the same bytes reached two different locators
depending only on the **name**: `image.bin` → `attachment:bin:3080a363…`, `payload.img` →
`attachment:unknown:3080a363…`, and the second satisfies no requirement. That is ADR-0030 D-3's "declared, never
derived" rule operating as designed at a surface a person drives. Second, the no-attachment run still prints the
pre-C1 sentence "missing from the snapshot" — accurate when no attachment set exists — while any run that does carry
attachments switches to the two-set wording. §9's requirement was that the rule stop claiming a snapshot origin for
bytes that did not come from one; it never does so.

## 8. Identity agreement across the two edges

The strongest single result of the round: **the CLI and the installed desktop computed the same identities for the
same project**, independently, from different entry points.

| fact | CLI (`fwsight`, 12:09 local) | installed desktop (13:11–13:39 local) |
| --- | --- | --- |
| Gate run id | `gate-a6cb8e84d226921d242bafebc7ca45534ce82a27cb7b3d9e5946249168d47239` | identical |
| policy sha256 | `a6a38dbb5df043c4d06e99b1296da8ce4aee04b03aaad985adc645f5d6f9ea77` | identical |
| build snapshot | `snap-4b4087e1…-p0-normalize-1-a38575cd…` | identical |
| baseline snapshot | `snap-3f615b62…-p0-normalize-1-832690060a…64ed363e` | identical |
| release id | `release-acdd6e7193df941a80deae5b17c9f4ac252066b519c5466fa6704d8806646299` | identical |
| proposed folder | `u4e-brake-node-1.2.3-acdd6e7193df` | identical |
| Gate run id re-derived after the reinstall | — | `gate-a6cb8e84…d47239` again, unchanged |

The desktop's own identity panel also printed `Stored 2026-10-10T20:11:44Z`, `Workspace HEAD d6258d4c...622cb1ce`,
`exact tag v1.2.3 on workspace HEAD`, `clean (workspace, not artifact)`, and the sentence that keeps the two apart:
"Git facts describe the workspace FirmwareSight read. They are not proof of how this artifact was built, and nothing
here says it was built from a commit."

## 9. The bundle the installed app wrote, checked by a stranger

Written to `…/fs-u4e/out-u4f/u4e-brake-node-1.2.3-acdd6e7193df`: 12 files, 4 of them artifacts.

```
$ sha256sum -c SHA256SUMS
accepted-reviews.json: OK      analysis.json: OK        artifacts/boot.hex: OK
artifacts/firmware.elf: OK     artifacts/firmware.map: OK   artifacts/image.bin: OK
diff.json: OK                  gate-results.json: OK    release-notes.md: OK
release-report.html: OK                                  (10 of 10, SHA256SUMS itself excluded)

$ python scripts/verify_bundle_portability.py <bundle>
63 of 63 checks passed
```

Shipped bytes equal source bytes: `artifacts/image.bin` and `artifacts/boot.hex` hash to
`3080a363…0cd666be` and `1d806a87…376973fc`, identical to `project/build/`. `release-notes.md` carries
`3be19091…db76f70d`, the same digest the Gate rule cited as `file:docs/RELEASE_NOTES.md#sha256=…`. The manifest
records `git_dirty: false`, which is the restore proving itself.

**This closes `C1-U3` §12.1.** That report's first outstanding item was that "`sha256sum -c` has never been executed
against a bundle's `SHA256SUMS` by anything in this repository." It has now been executed, by Git Bash coreutils
against a bundle the installed desktop wrote, with 10 of 10 OK. That is an external SHA-256 implementation agreeing
with Rust's `sha2`.

The desktop's result panel said the same thing in its own words: "The written folder was verified against its own
SHA256SUMS and manifest before it was put in place. The release is recorded in this database." — and the
Bundle & History "Release records" tab then listed exactly one row: `release-…06646299`, version 1.2.3, Gate run
`gate-a6c…68d47239`, manifest `17ae9c6d…a0696299`, stored `2026-10-10T20:39:37Z`. The Gate runs tab holds its own
rows and the caption that says a stored run is immutable and a re-run adds a row rather than replacing one.

## 10. Verification, each figure beside the command that printed it

| what | measured | command |
| --- | --- | --- |
| local gate | **17/17, `GATE_EXIT=0`**, no SKIP, no FAIL | `python scripts/check.py` |
| Rust | **1,020 passed, 0 failed** across **48** `test result:` lines | `cargo test --workspace` inside `rust/test` |
| Desktop UI | **305 passed** in **9** files | `frontend/test` (vitest) |
| core-smoke | 3/3 (`fmt`, `clippy`, `test`) | `python scripts/check.py --only core-smoke` |
| package | 4/4, `PKG_EXIT=0` — frontend deps, CLI companion, desktop package, artifacts verified | `python scripts/check.py --only package` |
| dependency policy | "advisories ok, bans ok, licenses ok, sources ok" | `deny/cargo-deny` |
| drift | all 8 green, including `drift/goldens unchanged` and `drift/ipc bindings unchanged` | `check.py` |
| portable bundle, CLI | 63/63 | `verify_bundle_portability.py` |
| portable bundle, installed | 63/63 + `sha256sum -c` 10/10 | same, plus coreutils |
| baseline artifacts | `RESULT PASS` at **801 tracked / 799 sum entries / 929 tree lines** | `python scripts/verify_baseline_artifacts.py`, after the documented generate order |

The tracked set moved 797 → 800 → 801 across the unit: `d95acea` added this unit's plan document and two ts-rs
generated DTOs, and this round adds its own report. `SHA256SUMS` hashes canonical Git stage-0 index blobs, not working
bytes (`ADR-0029`), so the tree and manifest were regenerated in the order the generator's own docstring prescribes —
stage, tree, stage, sums, stage, verify — rather than by pointing the generator at a dirty index.

`drift/goldens unchanged` passing **is** the compatibility test §9's conditional line depends on: had the report's
limits line been written unconditionally, `apps/cli/tests/p4_golden.rs` and the drift step would both have failed and
this unit would have had to STOP rather than regenerate a golden. No golden was touched.

The Rust count moved 987 → 1,020 across the unit and the UI count 295 → 305. Both are stated as measured on this
Windows host; **no POSIX figure is claimed for this head**, because no POSIX runner exists here and the jobs that
would provide one cannot run until the push the packet withholds happens.

## 11. Compatibility matrix, moved only where a run was observed

| row | before U4 | after U4, and why |
| --- | --- | --- |
| `BIN_HEX_RELEASE_ATTACH` | `DESIGN_APPROVED / NOT_USER_AVAILABLE` | `DESIGN_APPROVED / USER_VERIFIED_ON_WINDOWS` — attach, Gate, preview and export were each performed through the installed Windows binary's own surfaces |
| `BIN_HEX_ANALYSIS` | `UNSUPPORTED` | **`UNSUPPORTED`, unchanged.** Nothing parsed a BIN or a HEX; the kinds are declarations from file names |
| macOS / Linux installed acceptance | `NOT_RUNTIME_VERIFIED` | **`NOT_RUNTIME_VERIFIED`, unchanged.** No macOS or Linux artifact was installed or run this round; the packet's "installed visual evidence" obligation is met only for Windows |
| P5 §38 row H ("app executable removed") | measured 2026-10-03 as "emptied but not deleted" | **this round measured the directory itself gone.** Reported here rather than rewritten into P5's dated text; the owner and Architect should decide whether P5 gains a dated correction as it already did for row G |

## 12. Things observed that are not defects but are not nothing

1. **A visible `git.exe` console window appears over the app during bundle work.** Observed twice on the installed
   build — once at `Prepare bundle`, once at `Export bundle` — each time stealing foreground and each time closing on
   its own within seconds. The Gate reads `git:status`/`git:head`, so a child `git` is expected; a console window
   appearing above the user's window is not. Worth a decision about `CREATE_NO_WINDOW` on the spawn path.
2. **The Gate page's build and baseline selectors reset to defaults when the reader navigates away and back**, while
   the verdict panel keeps showing the last run. The panel is not lying — it names the run id, both snapshot ids and
   the stored timestamp, and `Prepare bundle` composes from *that run*, refusing (`6114`) rather than silently
   packaging the newly-selected build. But a person who does not read the identity block can see "Current build:
   snap-3f6…" beside a PASS whose Build row says `snap-4b4…`. A "this verdict is for the run above, not for the
   selection below" marker is a UI question, not this unit's to settle.
3. **A build-facts panel lagged one render behind a keyboard-only `<select>` change** on the pre-fix instance
   (showing the previously selected build's facts until the next repaint; with `Down` then `Return` it followed
   immediately). Not reproduced on the fixed build, so it is recorded as observed-once, not as a standing defect.
4. **One `Choose the project's firmwaresight.toml` dialog appeared unprompted** after an artifact pick on the fixed
   build, and did not reappear on the same action afterwards. It was answered with the same project file and nothing
   was harmed. This round does **not** claim the app auto-prompts for a config; it claims one dialog appeared once
   and this round could not reproduce it.

## 13. What did not move

`/1` Gate identity and every pre-C1 run id · `SnapshotId` · schema major 1 for every document · `SCHEMA_VERSION` 6
with no new migration · the P4 goldens, byte-for-byte · the design tokens and `assets/design-tokens.json` · Core's
zero third-party dependencies · no new `ERR-BUNDLE-61xx` code (6102, 6114 and 6115 all predate this round) · no
lockfile, capability, native-plugin or permission change · the V1 cohort `11573661113` at `41bb6a36`, still 0
eligible external sessions · the guarded 25-item tally · the licence · the publishing channel.

No BIN or HEX file was parsed, opened as a container, or described by anything in this round.

## 14. STOP conditions, and which ones came close

* U4 §17 (lockfiles, framework, migrations, native capabilities beyond existing use-case grants) — **not crossed**;
  nothing in this unit needed one.
* "If this cannot validate with existing `:1` schemas, STOP rather than bump a major" — never fired; no schema moved.
* U4 §27 ("do not say complete if only unit/UI tests ran") — **this is the condition the round took most seriously**,
  and the reason §5, §6, §9 and §11 exist in the form they do.
* The packet's "if a stage fails its contractual objective, do NOT enter the next stage" — stage 3 is the last stage,
  so there was no next stage to enter. The two defects in §4 were found *inside* stage 3 and closed inside it.

## 15. How to reproduce

```
python scripts/check.py                                   # 17/17
python scripts/check.py --only package                    # the installer used below
# install per-user, launch from %LOCALAPPDATA%\FirmwareSight, and walk:
#   Analyze  project/build/firmware.elf  + firmware.map
#   Compare  project/baseline/firmware.elf + its map
#   Release  Attach image.bin, Attach boot.hex, load firmwaresight.toml, Run Gate  -> PASS
#   Bundle   Prepare, Choose folder, mutate build/image.bin, Export -> ERR-BUNDLE-6102, nothing written
#            restore bytes, Prepare again, Choose folder, Export -> written
python scripts/verify_bundle_portability.py <the folder the app wrote>
```

Evidence root: `C:\Users\16429\AppData\Local\Temp\fs-u4e\` — `CLI_TRANSCRIPT.txt`, `INSTALLED_WALK_EVIDENCE.txt`,
`STATE-pre.json`, `TEST_STORE_HASHES.txt`, `ARTIFACT_IDENTITY*.txt`, `GATE-U4F*.log`, `PACKAGE-U4F.log`,
`LOCAL_GROUPS.log`, `parked-store/`, `INSTALL_TEST_STORE_RETAINED/`, `out-cli/`, `out-u4f/`, and 60 numbered PNG
screenshots (prefixes 01–52 with lettered variants) whose digests are listed in `INSTALLED_WALK_EVIDENCE.txt`. It is
outside the repository, as P5's is.

## 16. Outstanding, and what a reviewer should look at

1. **No CI-built artifact exists for any of the three packet stages.** `920e764`, `3cfbc03`, `d95acea` and `073c83c`
   are local. The install this round walked was built on this host from `073c83c` with a clean tree, which is real
   installed evidence but is **not** the same thing as P5 §8's CI-built artifact. The ten-job CI read-back that
   exists in the record belongs to `facf287`.
2. **`ERR-BUNDLE-6114` is a session requirement the UI cannot state in advance.** The refusal is correct and its
   "What to do" is actionable, but nothing on the Bundle section warns before the press that both sides of the
   comparison must have been analyzed since the app was launched. A reviewer may consider that a copy gap rather
   than a behaviour gap.
3. **The `6102` wording is hard to read at a glance.** It prints two long run ids and asks the person to notice the
   difference. It is truthful and it is the right refusal; whether it is the clearest one is a §9 copy question the
   Architect owns.
4. **`04_TECH/28` §8's kind-conflict sentence is still unimplemented** (carried from `C1-U3` §12.3): `kind_basis` is
   always `declared` and `detect_format` is never called on the attach path, so a file named `.hex` holding raw binary
   ships as a declared HEX with no disagreement shown anywhere. The installed walk re-confirms the behaviour
   (`payload.img` → `unknown`, `image.bin` → `bin`, same bytes) without resolving the design question.
5. **`M11` (POSIX symlinks) and `M12` (>512 MiB) remain `NOT_VERIFIED`**, and `ERR-INTERNAL-9004` for a cross-run
   acceptance remains the Architect's error-model question.
6. **P5 §38 row H** now has a dated measurement that disagrees with it (§11).
7. **The four observations in §12** are reported rather than fixed, because each is a change to a surface or a spawn
   path that this unit was not given authority over.

## 17. Machine state after this round

Uninstalled through the uninstaller's own UI, with "Delete the application data" left **unchecked** so the test store
survived as evidence. Measured: `%LOCALAPPDATA%\FirmwareSight` gone including the directory itself,
`Start Menu\Programs\FirmwareSight` gone, `HKCU\…\Uninstall\FirmwareSight` gone, `%APPDATA%\com.firmwaresight.desktop`
and `%LOCALAPPDATA%\com.firmwaresight.desktop\EBWebView` kept. The test store was moved to
`fs-u4e/INSTALL_TEST_STORE_RETAINED/` (sqlite `da63a52b…`) and the owner's store was put back and re-hashed to the
three digests in §3. The retained installers (`7669096320…` and `3d9a3512…`) make the install reversible without
this round leaving anything on the machine.

## 18. Stop

Stage 3's objective checkpoint is met and recorded: a person can attach a BIN or an Intel HEX from the installed
desktop and from the CLI, the Gate judges it with the evidence class named, the readiness table and the bundle
preview agree with that verdict, the export writes a bundle a stranger verifies 63/63, and a context that moved
after the preview is refused with nothing written.

What this round cannot give itself is what the packet reserves for others. **`C1-U4` is
`READY_FOR_ARCHITECT_PRODUCT_ACCEPTANCE`, pending independent review** — not accepted, not complete. `C1-U2R1` and
`C1-U3` remain `ARCHITECT_REVIEW_PENDING`. This round does not sign `C1 COMPLETE`, `USER_AVAILABLE`,
`SUPPORTED_WITH_LIMITS`, beta, RC, GA or a commercial release, and does not recruit for or alter V1.

Owner approval is requested, as a separate decision, for a **normal fast-forward push** of the local heads
(`99a1bfd`, `7a6af54`, `268440a`, `920e764`, `3cfbc03`, `d95acea`, `073c83c` and this round's docs commit) to
`origin/main`. **Never force, including `--force-with-lease`.** Nothing was pushed to produce this document, and one
Linux run on a CI-built artifact is what would close `M11` and give the POSIX Rust figure its measurement.
