---
title: "P5 Exit Checklist"
doc_id: "FS-P5-EXIT-CHECKLIST"
product: "FirmwareSight"
version: "0.6.0"
status: "IN_PROGRESS"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-05"
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
| known limitations complete | **PASS** | every L1–L26 row accounted for, no closed row dropped, L15 split into its presentation and wire halves, and F2's three new rows (F2-1/2/3) added |
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
