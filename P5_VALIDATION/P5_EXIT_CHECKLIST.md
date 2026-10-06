---
title: "P5 Exit Checklist"
doc_id: "FS-P5-EXIT-CHECKLIST"
product: "FirmwareSight"
version: "0.6.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — Commit F2 exit criteria (prompt §41)

One row per criterion §41 names, with the measurement behind it. A row reads **PASS** only where a command
or a screen produced the value; nothing here is inferred from a sibling row.

The last three rows are the ones that cannot be self-certifying, and they say so rather than borrowing
confidence from the other thirty-one.

| §41 criterion | State | Measured by |
| --- | --- | --- |
| F1 exact CI Windows artifact used | **PASS** | artifact id `11337963032` of run `37293381181` at head `0bca373`, downloaded and installed **as received**; no local rebuild, no re-packaging, no other installer run |
| internal package checksums verified | **PASS** | the artifact set's own `SHA256SUMS.txt` verified before install, and the installed binary re-hashed afterwards to `93be8c8e…` at 15,362,048 bytes |
| actual installer hash recorded | **PASS** | `a1152ef31a2c28b0fa522d88b7fe08eee543af1b86a161a4ab60acf04b3c076b`, 3,885,631 bytes — in `01_artifact/ARTIFACT.txt`, `03_migration/MIGRATION_PROOF.txt` and `P5_DESKTOP_ACCEPTANCE_REPORT.md` §1 |
| owner backup / park barrier passed | **PASS** | `OWNER_STORE_PARKED = YES`, `OWNER_BACKUP_HASH_MATCH = YES`, `PROCESSES_CLOSED = YES` (`02_owner_backup/BARRIER.txt`), and applied a **second** time at 18:04 with the same result |
| installed synthetic migration PASS | **PASS** | v4 → v5 under the installed binary, **34/34 checks**, snapshot named `.pre-migration-v4-to-v5`, both `integrity_check ok`, 0005 present once, no duplicate on a second reopen. `P5_MIGRATION_RECOVERY_REPORT.md` §3 |
| §64 contiguous journey PASS | **PASS** | one pass, install → … → final uninstall → owner restore, all real input. `P5_DESKTOP_ACCEPTANCE_REPORT.md` §3 |
| onboarding checked | **PASS** | cold installed window rendered the Getting-started panel with its seven facts and "Hide this"; Help repeats the same seven |
| Analyze baseline PASS | **PASS** | base `3f615b62…`, 16,820 B, Arm 32-bit little, 19 sections / 47 symbols / 10 evidence, Unknown reasons shown in words |
| Analyze target PASS | **PASS** | target `4b4087e1…`, 17,464 B, nv 376 / RAM 76 exact; builds 1 → 2 with the baseline row still exactly one |
| Compare PASS | **PASS** | +120 nonvolatile / +68 RAM, comparability `exact`, both filters working, chips icon+label, and L15's caption proven by deliberately analysing the target **without** its MAP |
| Gate PASS | **PASS** | `gate-5fa…e11eebb9`, overall **PASS**, 10 findings in canonical order, nothing bypassed, `N/A → PASS` shown as a disposition |
| Bundle PASS | **PASS** | `brake-node-1.2.3-4be1e7606a26`, nine entries plus two artifacts; bundled ELF and notes rehash to the recorded digests; `release_records.manifest_sha256` equals the manifest file's digest |
| relocated Bundle verifier PASS | **PASS** | copied elsewhere and verified there: `sha256sum -c SHA256SUMS` 8/8 OK, `verify_bundle_portability.py` **59/59**, **64/64** with `--schemas` |
| History installed PASS | **PASS** | 3 builds / 1 Gate run / 1 release, bounded and paginated, Details on all three, no edit or delete control anywhere, no absolute path anywhere |
| History source-independent PASS | **PASS** | firmware folder renamed so no source file exists at its recorded location, app reopened from the Start Menu, same rows rendered and confirmed by a read-only replay |
| Diagnostics export / privacy PASS | **PASS** | 1,122-byte export, 37 keys inside a 41-key allowlist, **11 absence classes clean**, counts equal to a read-only query of the same store |
| repair / reinstall PASS | **PASS** | maintenance flow over a running app, prompted before terminating, payload re-extracted **byte-identical**, `DIFF_KEYS = NONE` across eight compared keys, no duplicate rows |
| uninstall removal PASS | **PASS** | install dir **ABSENT**, desktop shortcut, Start Menu `.lnk` and folder gone, `HKCU` registration gone, `PATH` sha unchanged |
| uninstall retention PASS | **PASS** | store retained with an identical semantic snapshot; disposable firmware project identical at 44/44 hashed files; **the destructive option was never exercised** and is recorded as a boundary, not a pass |
| reinstall retained-data PASS | **PASS** | version 0.6.0, existing v5 store opens healthy, **no migration re-run**, History 3/1/1, Analyze functional, diagnostics counts coherent, no duplicate rows |
| final product state uninstalled | **PASS** | third uninstall returned the machine to its pre-F2 state: install dir, shortcuts, Start Menu folder and registration all absent, no process |
| owner restored | **PASS** | `ORIGINAL_DB_RESTORED = YES`, both passes; parked copy verified against §6 **before** moving and again after |
| owner hashes exact | **PASS** | `d6e41034…` 155,648 B / `e3b0c442…` 0 B / `fd4c9fda…` 32,768 B, matching §6 byte for byte, twice |
| closure docs present | **PASS** | all eight §28 documents exist and carry measurements; `P5_CI_AUTHORITY.md` and `P5_INSTALL_RECOVERY_REPORT.md` updated |
| known limitations complete | **PASS** | every L1–L26 row accounted for, no closed row dropped, L15 split into its presentation and wire halves, and F2's three new rows (F2-1/2/3) added. **This row's assertion was one row short and F3 corrected it on 2026-10-06: L12 had no row in `P5_KNOWN_LIMITATIONS.md`,** although the fact it covers was carried honestly in `P5_SUPPORTABILITY_REPORT.md` §2 and `P5_SECURITY_SUPPORTABILITY_REVIEW.md` §2. §7 records the correction and the re-counted list |
| user docs audited | **PASS** | §29's 19 topics mapped to canonical surfaces in `P5_EXECUTION_REPORT.md` §3; **Troubleshooting recorded as absent rather than stubbed**, per §29's own prohibition |
| security review present | **PASS** | `P5_SECURITY_SUPPORTABILITY_REVIEW.md`, which says "passes with documented accepted risks" and never says "security clean" |
| release readiness correct | **PASS** | `P5_RELEASE_READINESS.md` carries §34's ten states verbatim |
| license pending owner | **PASS** | `OPEN_SOURCE_LICENSE_DECISION = PENDING_OWNER_CONFIRMATION`; no `LICENSE` file added; L13 unchanged |
| no tag / release / sign / notarize / updater | **PASS** | no tag, no GitHub Release, no signature, no notarization submission, no updater plugin; `createUpdaterArtifacts: false` untouched |
| F2 local gate green | **PASS** | `cargo fmt --check` 0, `clippy -D warnings` 0, **868 Rust / 0 failed across 47 targets**, UI **219 in 8 files**, full gate **17 of 17**, drift **8/8**, deny 1/1, core-smoke 3/3, package 4/4 with no gate skip, `verify_baseline_artifacts.py` **RESULT PASS** |
| detached proof green | **see §3** | executed at the F2 candidate commit, which does not exist until this file is committed — so it cannot be self-certified from inside the commit it proves |
| archive proof green | **see §3** | same asymmetry |
| F2 remote CI 10/10 | **see §3** | read after the push, by exact F2 SHA, all ten jobs individually |

## 1. What F2 did **not** claim

- `P5 PASS`, `P5 PASS_COMPLETE`, `Productization COMPLETE`, `Productization ENGINEERING_COMPLETE`,
  `active_task NONE` — §39 reserves all of these for F3.
- `BETA`, `RC`, `GA`, `Production Ready`, `Security Clean`.
- Installed coverage beyond one migration path, and any claim that the uninstaller's data-deletion option
  works — it was never run.

## 2. The provenance barrier, checked rather than asserted

§35 lists the paths F2 may not change. Verified against the staged diff, not against intention: no path in
the diff is under `crates/`, `apps/`, `scripts/`, `fixtures/`, `schemas/`, `migrations/` or `.github/`, and
none of `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml` appears.

The stronger form of this check is §37's count rule, and it passed for the reason §37 exists: **868 Rust and
219 UI in 8 files are exactly F1's numbers.** A docs-only round that moved either count would be a product
change wearing a documentation diff, and the round would have had to stop.

One incidental file the round produced and removed: the first `pnpm install --frozen-lockfile` attempt was
run from the wrong directory, and pnpm created a default root `package.json` (name `FirmwareSight`, version
`1.0.0`, `test` script `echo "Error: no test specified" && exit 1`, licence ISC). It was never tracked, it
is not F1's file, and it was deleted once identified — `package.json` does not appear in the commit, which is
what §35 requires. The UI project's real manifest is `apps/desktop/ui/package.json`.

## 3. The three rows that cannot be self-certifying

Rows 32–34 describe facts about **this commit**: a clean detached worktree at its SHA, `git archive` of it
piped through `sha256sum -c SHA256SUMS`, and the remote's ten jobs for its SHA. None of them can be true
before the commit exists, and writing a further commit purely to record them is forbidden — §27 of the Commit
E prompt and §40 here both keep that stop in force, and F1 met the same wall at the same place.

They are therefore executed, and reported to the Architect under §42's final report, rather than back-filled
into the commit they prove. What *was* established pre-commit is the part that is genuinely pre-commit
knowable: the baseline verifier passes on the staged tree, the full gate is 17 of 17, and both test counts
match F1 exactly.

## 4. Addendum — the Design / Accessibility entry, as of Commit F2R (2026-10-06)

The table above is F2's, against F2's prompt §41, and it is not rewritten: F2 found three product findings
and graded them S2 / S3 / S3 on an installed binary it was forbidden to patch. The Architect inserted one
narrow corrective before F3 — *P5 Commit F2R* — run as **F2R1** `fb5f628` (product) and **F2R2** (this
record). Everything below is measured on the artifact F2R1's own CI run built.

| Condition (F2R prompt §34) | State | Measured by |
| --- | --- | --- |
| frozen 1024 × 720 minimum works for the corrected surfaces | **PASS** | installed captures `S11`/`S18`/`S29`/`S30`/`S32` in `11_analyze_sizes/` |
| 1056 × 799 — the size F2 measured the defect at | **PASS** | `T01_sections_1056.png`, `H01_builds_1056.png`, `H02_row_open_1056.png` |
| design target 1440 × 900 | **PASS** | `S09`/`S10` (with the table's own scrollbar dragged to its end), `H04_builds_1440.png`, `R04`/`R18` |
| the row action keyboard-focusable and activatable | **PASS** | `H05_keyboard_activation_1056.png` plus `KB_A`→`KB_B`→`KB_C`: mouse-opened row, `SPACE` closed it, `SPACE` reopened it, focus ring visible in both keyboard frames |
| no text-only critical control clipped | **PASS** | `Details` is the **leading** cell of every row and header row in all three History tables; filter, sort and pager each operated at 1024 × 720 |
| no raw evidence enum on a corrected human-facing surface | **PASS** | "ELF address/flags evidence" and **"MAP regions + ELF load evidence"** where F2 captured `MapRegionAndElfLoad`; the Analyze weakest-basis line agrees |
| existing focus-visible / design-token rules remain | **PASS** | `assets/design-tokens.json` **0 bytes changed**; the only literal added is a `24ch` prose measure in the unit the shipped prose rules already use |
| full UI suite green | **PASS** | 225 tests in 8 files, and §22's 20 fresh-process repetitions 20 / 20 green |
| focused installed artifact green | **PASS** | run `37431977428` **10 of 10**, attempt 1, then the three revalidations above |

```
P5_DESIGN_ACCESSIBILITY = PASS_FOR_FROZEN_DESKTOP_SCOPE
WCAG_CERTIFICATION      = NOT_PERFORMED
MULTI_DPI_125_150       = NOT_TESTED
SECOND_WINDOWS_HOST     = NOT_TESTED
```

That is a **scope statement, not a certification**: one host (`DESKTOP-S0IK346`) at 96 dpi / 100 % scale, the
frozen desktop range, three corrected surfaces plus the §33 smoke list. No conformance level is claimed and
no other display, DPI setting or machine is covered.

One trade is recorded instead of hidden: at 1440 × 900 the corrected Sections table needs a **contained**
scroll of about 1.06× the pane to reach its last column, where the pre-fix layout fitted the pane by
rendering one character per line. The table area owns that scroll, which is the design authority's preferred
outcome; the zero-scroll alternative measured a 71 × 120 px prose column at 1024.
`P5_F2R_UI_CORRECTIVE_DESIGN.md` §7 carries both numbers.

## 5. Addendum — F2R's own exit criteria (its prompt §45)

| Criterion | State | Evidence |
| --- | --- | --- |
| F2 history preserved truthfully | **PASS** | F2's §6 grading, run #69's three attempts and its two zero-step cancellation batches all appear unchanged in `P5_CI_AUTHORITY.md` and in `P5_F2R_UI_CORRECTIVE_REPORT.md` §1 |
| three findings reproduced or anchored to F2 evidence | **PASS** | `01_layout_probe/` measurements plus `02_test_first_red/` red test; F2's own captures cited by id |
| F2R-01 fixed narrowly | **PASS** | `Details.module.css` + `Details.tsx`, inside `apps/desktop/ui/src/**` |
| F2R-02 fixed narrowly | **PASS** | `History.module.css` + `History.tsx`, leading-cell placement |
| F2R-03 fixed narrowly | **PASS** | `evidenceBasis.ts` shared map used by `Release.tsx`, `Compare.tsx`, `Analyze.tsx` |
| no new dependencies | **PASS** | no lockfile, manifest or workflow byte changed; §9's screenshot-library ban obeyed by adding none |
| no Rust / backend change | **PASS** | 0 paths under `crates/` or `apps/desktop/src-tauri/`; Rust direct count still 868 |
| no schema / migration change | **PASS** | SQLite schema 5, migrations 0001–0005 unchanged |
| no wire / public contract change | **PASS** | `analysis:1`, `diff:1`, `gate-results:1`, `accepted-reviews:1`, `release-manifest:1`, ADR-0028, ADR-0029, `elf.program-header` all unchanged; the caption is never serialized |
| no design-token mutation | **PASS** | `assets/design-tokens.json` 0 bytes changed, never staged |
| Release shared caption behavior tested | **PASS** | `release.test.tsx` +2, `intake.test.tsx` +1, and the two-spelling mutation reddening Release **and** Compare |
| Analyze structural responsive contract tested | **PASS** | `details.test.tsx` viewport/columns test |
| History action structural contract tested | **PASS** | `history.test.tsx` +2 (leading cell, wrapper scroll axis) |
| direct Rust count stable | **PASS** | 868 locally, 868 in the runner's own log |
| direct UI suite green | **PASS** | 225 in 8 files locally and on both CI runners |
| 20/20 UI reliability campaign green | **PASS** | `05_ui_reliability/CAMPAIGN.txt`, 7 min 25 s, no retry or sleep |
| full local gate green | **PASS** | 17 of 17 |
| drift green | **PASS** | 8 of 8 |
| package green | **PASS** | 4 of 4, no `SKIP` |
| baseline verifier green | **PASS** | tracked 707 / entries 705 / all mismatch counters 0 / RESULT PASS |
| F2R1 remote CI 10/10 | **PASS** | run `37431977428`, attempt 1, every job and step read individually |
| exact F2R1 Windows artifact installed | **PASS** | artifact id `11397938806`, installer `efbc45a3…d5fd4`, installed binary `2cf01a6d…670b` |
| owner store parked safely | **PASS** | `OWNER_STORE_PARKED = YES`, `OWNER_BACKUP_HASH_MATCH = YES` before install |
| Analyze 1024 / 1056 / 1440 installed PASS | **PASS** | §4 above |
| History 1024 / 1056 / 1440 installed PASS | **PASS** | §4 above |
| Release caption installed PASS | **PASS** | both evidence states |
| quick functional smoke PASS | **PASS** | §33 list, `SMOKE_AND_DISPOSITIONS.txt` |
| no new S0 / no new S1 / no open S2 | **PASS** | none appeared in the focused pass; F2R-01's S2 is closed |
| owner store restored, SHA exact | **PASS** | `ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`, `OWNER_STORE_OPENED_BY_F2R = NO` |
| F2R2 docs/evidence-only | **PASS, proved from the staged diff** | 0 paths under `apps/`, `crates/`, `scripts/`, `fixtures/`, `schemas/`, `migrations/` or `.github/`, and no `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml` or `assets/design-tokens.json` byte appears. Both counts are identical to F2R1 — **868 Rust in 47 suites and 225 UI in 8 files**, read out of the gate log's `[rust] test` and `[frontend] test` sections — and the build emitted the same content-hashed assets F2R1 did (`index-R0fRHss-.css`, `index-DCexUFJu.js`), which is the machine's way of saying no product byte moved |
| F2R2 local gate green | **PASS pre-commit; the two detached proofs are not self-certifiable** | full gate **17 of 17** (drift **8 of 8**, deny **1 of 1**), `--only core-smoke` **3 of 3**, `--only package` **4 of 4** with no `SKIP`, baseline verifier PASS. The clean detached worktree and the `git archive` extraction can only be addressed at a SHA that exists, so they run **after** the commit and are reported to the Architect — §3's asymmetry, unchanged |
| F2R2 remote CI 10/10 | **external** | read after the push with `gh run list --json headSha,runAttempt,conclusion`; §44 forbids a further head written only to record it |

## 6. What F2R did **not** claim

- No P5 closure sentence of any kind, and no `Productization ENGINEERING_COMPLETE`; F3 owns both, and F3 is
  explicitly not authorized in this round.
- `active_task` stays `P5_PRODUCTIZATION`; nothing was tagged, released, signed, notarized or updater-enabled.
- No installed coverage beyond what the focused pass ran: the §64 full journey is F2's, already walked on
  F1's artifact, and F2R deliberately revalidated only the three corrected surfaces plus the smoke list.
- The uninstaller's data-deletion option is still unexercised, and no claim about it changed.

*(§6 is F2R's own boundary, written on 2026-10-06 when F3 did not yet exist. The Architect then issued F3, and
§7 is that round's checklist. Nothing above is rewritten: F2R really did claim none of it.)*

## 7. Final P5 exit re-audit (F3 prompt §6 and §21), written 2026-10-06 at `a5ce7c4`

§6 of the F3 prompt requires every required engineering item to be `PASS`, `CARRIED_FORWARD_NONBLOCKING` or
`OWNER_DECISION_NONBLOCKING` before the closure sentence may be written, and forbids any required item reading
`BLOCKED`. §21 names the states; the middle column is what F3 wrote; the right column names the command or the
screen that produced it, because a row that borrows confidence from a sibling row is not a measurement.

| Area | State F3 writes | Produced by |
| --- | --- | --- |
| Packaging | **PASS** | F2R2 head green on all three package jobs; `--only package` 4 of 4 with no `SKIP`; nsis / app+dmg / deb canonical, distribution index one line per file and verified on the runner's own set |
| Onboarding | **PASS** | cold installed window rendered the Getting-started panel's seven facts and "Hide this"; Help repeats the same seven |
| History | **PASS** | installed 3 / 1 / 1 over three bounded read APIs, Details on every row, no edit or delete control, leaf names only; re-read after the source project moved; F2R-02 closed the clipped action on F2R1's artifact |
| Diagnostics | **PASS** | 1,122-byte export, 37 keys inside `ALLOWED_KEYS`' 41, 11 absence classes clean against the raw bytes, counts equal to a read-only query of the same store |
| Migration / recovery | **PASS** | installed v4 → v5 on 34 measured checks with a named pre-migration snapshot and `integrity_check ok` on both files; the machinery is integration-tested for v1–v4 → v5, v5 reopen and failure rollback |
| Compatibility | **PASS within the documented cohort** | five-value status column, 38 cells, 0 outside the vocabulary; 56 manifest entries; Windows runtime-evidenced, macOS / Ubuntu `CI_BUILD_ONLY` and never written as `SUPPORTED` |
| Design / Accessibility | **PASS_FOR_FROZEN_DESKTOP_SCOPE** | F2R prompt §34's nine conditions on the installed F2R1 artifact at 1024×720 / 1056×799 / 1440×900, plus §4's block. **Carried exactly, not upgraded**: `WCAG_CERTIFICATION = NOT_PERFORMED`, `MULTI_DPI_125_150 = NOT_TESTED`, `SECOND_WINDOWS_HOST = NOT_TESTED` |
| Documentation | **PASS for P5 canonical surfaces** | §29's 19 topics mapped to surfaces that already document them; Troubleshooting recorded as absent rather than stubbed; `P5_KNOWN_LIMITATIONS.md` now exists at the path `Help.tsx:50` names and now holds all 26 rows |
| Security / supportability | **PASS_WITH_DOCUMENTED_ACCEPTED_RISKS** | no TCP endpoint owned by the running app; packaged `http://tauri.localhost/` origin; capabilities grant no general shell or filesystem; `installMode: currentUser`, UAC never prompted, no HKLM key; `cargo deny` four checks ok with two advisories accepted in `deny.toml` and measured as unupgradable inside the frozen Tauri / gtk line. This is the bounded sentence, and the repository has never written the other one |
| Windows installed product acceptance | **PASS** | F2's §64 journey on the CI-built F1 artifact, F2R's focused revalidation on the CI-built F2R1 artifact, plus repair, uninstall, reinstall and a second uninstall |
| Version identity | **PASS** | every artifact version unified on `0.6.0` through `env!("CARGO_PKG_VERSION")`, `drift/version identity` in the gate, Help reading its identity from the running binary |
| Repository baseline integrity | **PASS** | `ADR-0029`: root `SHA256SUMS` digests canonical Git stage-0 index blobs, the verifier re-derives through a different Git path, and `drift/baseline integrity` runs it on all three CI platforms. At F2R2: tracked 708 / entries 706 / every mismatch counter 0 / `RESULT PASS` |
| Signing readiness | **PASS as `READY_NOT_EXECUTED`** | the readiness is the deliverable; executing needs a certificate, a key-management ADR and an owner decision, none of which F3 has |
| Notarization readiness | **PASS as `READY_NOT_EXECUTED`** | the macOS package builds and its index reads; no submission, no Apple credentials, none requested |
| Manual upgrade readiness | **PASS** | no updater by design; the manual path — newer installer over older, store migrating behind a snapshot — measured across the whole install cycle |
| License | **OWNER_DECISION_NONBLOCKING for technical P5** | `license = "Proprietary"`, no root `LICENSE`, `OPEN_SOURCE_LICENSE_DECISION = PENDING_OWNER_CONFIRMATION`; `AGENTS.md` §9 and P5 §54 put the choice with the owner. §14's consequence is in `P5_RELEASE_READINESS.md` |
| CI | **PASS through F2R2, before F3** | head `a5ce7c4` / run `37453402452` / attempt 1 / **10 of 10 success**, all ten job names read individually. F3's own run is `PENDING_EXTERNAL_EVIDENCE` inside this commit |
| Clean detached tree | **PASS** | executed at every head that can address one; F3's runs at the F3 SHA after the commit and is reported, not self-certified (§3's asymmetry) |
| Owner DB restore | **PASS** | F2R's `ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`, `OWNER_STORE_OPENED_BY_F2R = NO`; F3 §24 forbids touching the owner store and F3 touches nothing |
| External user validation | **NOT_EXECUTED — separate V1 track** | L11, V0 at `0 / 8`. P5 closes without it and does not start it |
| Public distribution | **NOT_AUTHORIZED** | §13's freeze, restated in `P5_RELEASE_READINESS.md` |

**No required engineering row is `BLOCKED`.** The only `BLOCKED` phrase the stage writes is the one §14 requires
of it, and it is about a distribution claim rather than an engineering item:
`PUBLIC OPEN-SOURCE REDISTRIBUTION CLAIM = BLOCKED BY OWNER LICENSE DECISION`.

**The one defect this re-audit found and fixed was in this repository's own record, not in the product:**
the §41 table's "known limitations complete" row above asserted a completeness this file's sibling list did not
have, and L12 had no row in `P5_KNOWN_LIMITATIONS.md`. F3 added the row and corrected the assertion rather than
deleting the claim. That is also why §6's performance and coverage language is unchanged: a closure round that
quietly re-labelled a carried limitation as closed would be making the stage look cleaner than the evidence
does, and F3 §11 forbids exactly that.

## 8. What F3 does **not** claim

`P5 = PASS_COMPLETE`, `Productization = ENGINEERING_COMPLETE` and `active_task = NONE` are F3's to write and are
written — in `BASELINE.yaml`, `.ai/CURRENT_STATE.md`, `.ai/ACTIVE_TASK.md`, `.ai/HANDOFF.md`, `README.md`,
`INDEX.md` and `P5_FINAL_CLOSURE_REPORT.md`. What F3 writes **none** of:

- BETA, RC, GA, Production Ready, market fit, real-user validated, `security clean`, or WCAG compliance.
- A signature, a notarization submission, a tag, a GitHub Release, a published installer or an updater.
- A licence decision, or any claim that FirmwareSight is a licensed open-source release.
- V1, B1, G3 or any other next track: no prompt authorizes them, and P5's closure is not one.
- A new checksum semantic, a renamed Core enum, a moved wire value, a migration `0006`, or `analysis:2`.
- A product byte. F3 changes none, which §5 tests by requiring **868 Rust / 225 UI in 8 files** to be unchanged
  and §25 proves from the staged diff.
- Its own remote CI run. It is external evidence read after the push; §34 forbids writing a further commit to
  record it.
