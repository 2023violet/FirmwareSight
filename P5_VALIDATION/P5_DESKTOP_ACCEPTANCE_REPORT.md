---
title: "P5 Desktop Acceptance Report"
doc_id: "FS-P5-DESKTOP-ACCEPTANCE"
product: "FirmwareSight"
version: "0.6.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — the §64 journey, walked end to end on the installed F1 build

**What this is.** Commit F2 ran the whole section 64 journey once, contiguously, on the machine: install →
onboarding → Analyze ×2 → Compare → Gate → Bundle plus relocation verification → History → source-independent
reopen → Diagnostics → close/reopen → repair → uninstall → reinstall → final uninstall → owner restore. It
used the **exact Windows artifact CI built at F1**, not a local rebuild, because a locally built installer
would prove something about this host's toolchain rather than about what the repository ships.

**What this is not.** Not `P5 PASS`, not `Productization COMPLETE`, not a beta, RC or GA claim. F2 does not
close P5 and cannot; that sentence belongs to Commit F3 (`§39`, `§42`). `P5 = IN_PROGRESS` at the end of this
file, deliberately.

**Verdict: `F2_JOURNEY = PASS`, with two named coverage boundaries** — installed migration is one path
(v4 → v5) and the uninstaller's data-deletion option was never exercised. Both are stated where they could
mislead, in §7 and in `P5_MIGRATION_RECOVERY_REPORT.md` §4.

## 1. The candidate, and why it is the candidate

| | |
| --- | --- |
| Artifact | `FirmwareSight-0.6.0-windows-x86_64-nsis.exe` |
| Bytes | 3,885,631 |
| SHA-256 | `a1152ef31a2c28b0fa522d88b7fe08eee543af1b86a161a4ab60acf04b3c076b` |
| Source | GitHub Actions artifact id `11337963032`, run `37293381181`, head `0bca373ed46082f98a9f474908d09f836174ff1b` |
| Rebuilt locally? | **No.** Downloaded, hashed, and installed as received |
| Installed binary | `firmwaresight-desktop.exe`, 15,362,048 bytes, `93be8c8e0abc711f1a98f497cec6837f1710ef9674e3cbcbb3ac7c10c0020c50`, ProductVersion/FileVersion 0.6.0 |

The package's own `SHA256SUMS.txt` was verified before install, and the install-side digest was recomputed
afterwards: the bytes CI attested are the bytes that ran. `01_artifact/ARTIFACT.txt`, `05_install/INSTALL.txt`.

## 2. How it was driven, and what that rules out

Every action was a real `SendInput` mouse click or keystroke against the installed window — the same
electrical event a hand produces. Three consequences matter for reading this report:

- **No database write stands in for a UI action.** The only SQL this round ran was read-only
  (`file:…?mode=ro`), used to *check* what the UI had already done, never to do it.
- **No WebView2 CDP, no remote debugging, no synthetic UIA `Invoke`.** The UIA tree exposes WebView content
  as generic regions, so a screenshot is the only channel that can show what a user would see — and every
  page claim below is backed by one.
- **The native dialogs are the real ones.** The file picker titled "Choose the firmware artifact to analyze",
  the MAP picker, the folder picker and the "Export the diagnostics file" save dialog were operated by
  typing into the File name field and pressing the actual Open/Save button.

That discipline is what turned two harness defects into findings rather than false passes, and §6 records
them. A click that lands on the wrong window now aborts instead of reporting success.

## 3. The journey, step by step

| § | Step | Result | The measured fact, not the claim |
| --- | --- | --- | --- |
| 11 | F1 already installed | PASS | install dir, digest, Start Menu entry and `HKCU` registration all present and read back |
| 12 | First-run onboarding | PASS | cold window rendered the **Getting started** panel with its seven facts and "Hide this"; Help repeats the same seven and Diagnostics is discoverable from it |
| 13 | Analyze baseline | PASS | base ELF `3f615b62…f33f21f`, 16,820 B, Arm 32-bit little, parser `object-elf/0.40`, entry `0x08000049`, Nonvolatile `Exact 256 bytes`, RAM `Exact 8 bytes`, 19 sections / 47 symbols / 10 evidence, and Unknown reasons shown **in words** (".bss — the section has no file range; it occupies no bytes on disk") |
| 14 | Analyze target | PASS | target `4b4087e1…ddd8374`, 17,464 B, nv 376 / RAM 76 exact. Store went 1 → 2 builds and the baseline row stayed exactly one |
| 15 | Compare | PASS | Old/Base and New/Target, deltas **+120** nonvolatile / **+68** RAM, pair comparability `exact`, both filters working (section "Added" → 1 of 1 `.ota`; symbol "Added" → only added rows), change chips icon+label and never a raw enum |
| 15b | L15 in the installed binary | PASS | the target was analysed **without** its MAP to force basis `ElfAddressAndFlags`, and the Evidence Inspector printed **"Source: ELF address + flags evidence"** — the caption proven on the shipping artifact rather than inherited from F1's unit test |
| 16 | Release Gate | PASS | policy through the native picker, project `brake-node`, digest `b87aa057…e5e98`; run `gate-5fa…e11eebb9`, **PASS**, 10 findings in canonical order, nothing bypassed, `N/A → PASS` shown as a disposition rather than a silence |
| 17 | Release Bundle | PASS | `brake-node-1.2.3-4be1e7606a26`, nine top-level entries plus `artifacts/{firmware.elf,firmware.map}`; `release-4be1e7606a26…19fa85`; the bundled ELF rehashes to the recorded digest and the bundled notes to `d547160a…` |
| 17b | Relocated bundle | PASS | copied elsewhere and verified there: `sha256sum -c SHA256SUMS` 8/8 OK, `verify_bundle_portability.py` **59/59**, and **64/64** with `--schemas`. No host path and no wall-clock value in anything written |
| 18 | History | PASS | 3 builds / 1 Gate run / 1 release, each bounded and paginated, Details expandable on all three, **no edit or delete control anywhere on the page**, and no absolute path shown anywhere — the row states that where the artifact lives is never displayed |
| 18b | Source-independent reopen | PASS | app closed, the firmware folder **renamed** so no source file exists at its recorded location, app reopened from the same Start Menu shortcut: same three builds, one Gate run, one release, and a read-only replay confirming identical row payloads |
| 19 | Diagnostics | PASS | 1,122-byte export, 37 keys against a 41-key ceiling, 11 forbidden-content classes all absent, counts agreeing with History |
| 20 | Close / reopen | PASS | normal window close ended the process; relaunch from the same `.lnk` ran the installed binary at `%LOCALAPPDATA%\FirmwareSight\…`, not cargo or Vite; schema still 5, migrations 1..5 once each, every count identical |
| 21 | Repair over the install | PASS | maintenance flow "Add/Reinstall components", running-app prompt handled, payload re-extracted **byte-identical**, and a semantic before/after with `DIFF_KEYS = NONE` |
| 22 | Uninstall | PASS | install dir **ABSENT**, both shortcuts and the Start Menu folder gone, `HKCU` registration gone, `PATH` unchanged, firmware project 44/44 files identical, store retained |
| 23 | Reinstall | PASS | same artifact, footprint restored, History still 3/1/1 |
| 24 | Retained data behaves as documented | PASS | version 0.6.0, store opens at v5 healthy, **no migration re-run**, Analyze still works, diagnostics counts coherent, no duplicate rows |
| 25 | Final cleanup | PASS | machine back to the uninstalled state it started in; disposable inputs preserved outside the repository and removed from `%TEMP%` |
| 26 | Owner restore | PASS | `ORIGINAL_DB_RESTORED = YES`, `ORIGINAL_DB_SHA_MATCH = YES`, `OWNER_STORE_OPENED_BY_F1 = NO`, zero sibling stores lost — **before any commit**, as §26 requires |

## 4. Two things the journey proved that no earlier round had

**Retention is proved semantically, not by a file digest.** §21 and §22 both compare read-only replays of
the product's own candidate, Gate and release SQL plus table counts, evidence rows and
`PRAGMA integrity_check`, because whole-file hashing is the wrong instrument for a WAL database. In the
repair case the main file and the WAL both predate the repair and only the `-shm` index moved; in the
reinstall case the store's three files came back with byte-identical digests after a full round trip that
included a live Analyze. A claim built on "the file hash changed" or "it didn't" would have been wrong
either way.

**The dedup rule was exercised, not assumed.** Re-analysing the same artifact without its MAP did not add a
fourth build, and the first reading of that was suspicion. Checked against the code rather than assumed to
be a silent failure: `Session::store` imports only when `build_id_for_snapshot` finds nothing
(`apps/desktop/src-tauri/src/lib.rs:486-494`), and the snapshot id is
`snap-{artifact_sha256}-{NORMALIZATION_VERSION}` with `-{map_sha256}` appended only when a MAP is attached
(`crates/firmwaresight-core/src/domain/build_snapshot.rs:34`). Reusing the row is
the designed behaviour and is pinned by `tests/real_artifact_intake.rs:705`. So one action satisfied both
remaining §24 items at once.

## 5. Owner-data barrier, held twice

The installed app resolves its store from Tauri's `app_data_dir()` with no environment override
(`apps/desktop/src-tauri/src/lib.rs:953`), so the only safe way to run it was to move the owner's store out
from under it first. That happened at 07:34, before the installer ran, and — because §24 needed a second
installed cycle — again at 18:04, each time released again afterwards.

Both times the same order was followed: confirm no process holds it, hash it, copy it byte-for-byte and
verify the copy, park the originals, confirm the live path is clear, and on the way back verify the parked
copy against the original digests *before* moving it, then again after. The owner's three files return to
`d6e41034…` / `e3b0c442…` / `fd4c9fda…` every time, and the sibling stores from earlier phases
(`.g2-smoke`, `.p2-smoke`, `.p4-smoke`, `.p0-smoke-history`, two `postg2-e2e-*` directories) were never
opened, moved or deleted — `sibling stores lost: 0` in both passes.

Reading the restored store is what produced the round's most useful surprise: **the owner's real store is at
schema v2.** That proves `OWNER_STORE_OPENED_BY_F1 = NO` more strongly than byte equality alone — had the
installed candidate ever opened it, migrations 0003–0005 would have run and it would be at v5. It also
bounds what §8's migration proof can claim, which is why `P5_MIGRATION_RECOVERY_REPORT.md` §4 says what it
says.

## 6. Findings, classified

Classified per §27 as PRODUCT / HARNESS / ENVIRONMENT / KNOWN_LIMITATION / OBSERVATION, with product
severity. **No S0 or S1 was found, so §27's stop does not apply.** Nothing was patched: a product fix after
F1 would mean the F1 artifact stops being the acceptance authority and a new package plus revalidation
would be required.

### PRODUCT

| Finding | Severity | Evidence | Why that grade |
| --- | --- | --- | --- |
| The Release page's Evidence basis column prints the raw Core enum `MapRegionAndElfLoad` while Analyze, Compare and History print human-readable text | **S3** (presentation) | `15`/`16` captures; `08_compare/N00–N12` | This is L20's shape on a surface L20 did not cover. No fact is wrong and nothing is blocked; it is the same class of inconsistency Commit E closed elsewhere. Not patched, because F2 may not touch `apps/**` |
| At a narrow window the Analyze sections table collapses its text column so rows become unreadable rather than reflowing | **S2** (layout) | `07_analyze/L00–L22` at the default 1056 × 799 and narrower | Degrades at a size a user will hit, but the data remains reachable by widening the window or using Details |
| History's "Details" button is clipped at the default window width | **S3** (layout) | `11_history/R00–R12` | Cosmetic; the control still works |

### HARNESS — mine, recorded because they changed what was believed

| Finding | Class | Note |
| --- | --- | --- |
| `clickscreen` raised whatever `WindowFromPoint` reported, so clicks aimed at installer buttons were delivered to a Chrome window covering the work area | **HARNESS**, root cause of the round's "clicks that did nothing" | Superseded by `clickctl`, which raises the intended window and **refuses** unless it owns the pixel. The two events earlier classified ENVIRONMENT (a minimized wizard) are reclassified here. `00_preflight/HARNESS_clickscreen_wrong_window.txt` |
| Sent message id `0x0010` to a wizard's child buttons intending `BM_GETCHECK`; `0x0010` is `WM_CLOSE`, which destroyed the finish page's controls and made it paint empty | **HARNESS** | The blank page is **not** product behaviour. That attempt's evidence is quarantined under `16_reinstall/aborted_attempt_harness_error/` and no verdict is taken from it. `00_preflight/HARNESS_wm_close_to_wizard_children.txt` |
| `f2_ui.py key` requires the *app* to be foreground, which is wrong for a native dialog in the same process: raising the app steals focus from the dialog's edit | **HARNESS** | Mangled the first save-dialog attempt; the dialog was cancelled rather than forced and re-opened clean. `00_preflight/HARNESS_key_action_steals_dialog_focus.txt` |
| One accidental title-bar close at 08:08, and one click plus possibly one 82-character typed path delivered to a bystander application | **HARNESS** | Store untouched, relaunched from the same `.lnk`; the bystander input had no effect on the repository or the disposable project. `00_preflight/HARNESS_foreground_denial.txt` |

### ENVIRONMENT / OBSERVATION

| Finding | Class | Note |
| --- | --- | --- |
| The file picker and the save dialog both restore the OS last-used folder, which on this machine is a private firmware project directory | **OBSERVATION**, ordinary Windows behaviour | Nothing private was selected or written; the disposable path was typed in each time. It is why §19 cancelled a first export. Recorded as an operator note in the supportability review rather than as a defect |
| Installer maintenance dialog wording "FirmwareSight is running! Click OK to kill it", rendered with 确定 / 取消 buttons on a Chinese-locale host while the wizard body stays English | **OBSERVATION** | Functional and non-blocking — it prompts rather than killing silently, which is the right shape. The mixed localisation and the blunt verb are wording, not behaviour |
| `NoModify` / `NoRepair` are set in the registry while the NSIS installer still offers a maintenance page | **OBSERVATION** | Windows "Apps & features" therefore shows neither button, so repair is reachable only by re-running the installer. It works; the two surfaces disagree about whether repair is offered |
| Two window-minimisation events during the round | **reclassified HARNESS** | The controlled reproduction that failed at the time is explained by the `clickscreen` defect above |

## 7. Coverage boundaries this round does not cross

- **Installed migration is one path: synthetic v4 → v5.** Fresh→v5 and v1/v2/v3→v5 are storage-integration
  proofs. The owner's store being at v2 makes the chained v2 → v5 installed path a named gap, not a
  hypothetical one. `P5_MIGRATION_RECOVERY_REPORT.md` §4.
- **The uninstaller's "Delete the application data" option was never exercised**, because that folder also
  holds unrelated historical stores from earlier phases. Default is do-not-delete and `tauri.conf.json`
  sets no `deleteAppDataOnUninstall`. Recorded as `NOT_TESTED_BY_DESIGN`, not as a pass.
- **The install directory after uninstall is not stable across runs** — one observed uninstall left it
  empty, three removed it. No rule established; the user-facing wording does not promise either.
- **WebView2 remains an untested negative.** This host carries it, so "runs without Rust/Cargo/Node/pnpm/
  Vite" is evidenced as *does not use them*, not as *would fail without them*.
- **`§64` was walked on one host, one DPI setting, one WebView2 version** — the export reports
  `webviewVersion 154.0.4258.53` and `tauriVersion 2.12.0` — which is L4's residue restated rather than
  resolved.

## 8. What this file claims about P5

Nothing beyond "the journey passed on the F1 artifact". `F2 = COMPLETE`, `P5 = IN_PROGRESS`,
`F3 = READY_FOR_ARCHITECT_REVIEW`. No tag, no release, no signature, no notarization, no updater, and no
licence choice — §34's states are recorded in `P5_RELEASE_READINESS.md` and the F2 prompt's closing lines
send the round back to the Architect rather than forward to F3.

Evidence root: `FirmwareSight-P5-F2-20261005T073141`, outside this repository, with the running log at
`18_summary/JOURNEY_LOG.md`.

## 9. Addendum — the three product findings this report raised, and how F2R closed them (2026-10-06)

Nothing in §6, §7 or §8 above is rewritten. This report stays the record of what F2 measured on the **F1**
artifact, including that F2 was forbidden to patch anything it found (§35), and that the artifact under test
was F1's `11337963032`.

The Architect answered §6's three PRODUCT rows with one narrow corrective rather than letting F3 start
around them: *FirmwareSight — P5 Commit F2R — Installed UI Productization Corrective, Execution Prompt
v1.0*, executed as **F2R1** `fb5f628` (the product fix) and **F2R2** (the evidence head that writes this
paragraph). The full record is `P5_F2R_UI_CORRECTIVE_REPORT.md`; the mechanism, the measurements and the
rejected alternatives are in `P5_F2R_UI_CORRECTIVE_DESIGN.md`.

| §6 finding | F2 grade | Closed by | Re-measured on |
| --- | --- | --- | --- |
| Analyze sections table collapses its text column at a narrow window | **S2** | `.viewport` wrapper owns the scroll, the table keeps its intrinsic minimum, prose wraps at a `24ch` measure, the Unknown reason moves to its own line | F2R1's own CI-built Windows artifact (`11397938806`, installer `efbc45a3…d5fd4`), installed at 1024×720, 1056×799 and 1440×900 — **CLOSED** |
| History's "Details" clipped at the default width | **S3** | the row action moved to the **leading** cell of every row and header row, the placement this round's own Evidence table already used | same artifact, same three sizes, plus `SPACE` close/reopen with a visible focus ring — **CLOSED** |
| Release prints the raw Core enum `MapRegionAndElfLoad` | **S3** | one shared UI-only caption map, `apps/desktop/ui/src/evidenceBasis.ts`, read by `Release.tsx`, `Compare.tsx` and `Analyze.tsx`; stored value and wire token unchanged, caption never serialized | same artifact: "ELF address/flags evidence" without a MAP, **"MAP regions + ELF load evidence"** with one — **CLOSED** |

What this report still claims and the corrective does **not** change:

- **The journey this file documents was F2's, on F1's bytes.** F2R ran a *focused* revalidation (§30–§33),
  not a second §64 walk, and did not re-prove the migration, bundle, relocation or repair paths.
- **§7's boundaries all still stand.** One installed migration path; the owner's store still at schema v2;
  WebView2 still an untested negative; the uninstaller's "Delete the application data" option **still never
  exercised** — F2R read it as unchecked with `BM_GETCHECK` and deliberately left it alone, for the same
  reason this round did; and one host, one DPI setting, one WebView2 version.
- **New residue, stated here rather than buried:** at 1440×900 the corrected Sections table needs a small
  contained horizontal scroll (≈1.06× the pane) that the pre-fix squeezed layout did not have, because the
  pre-fix layout fitted the pane by being unreadable.
- F2R found no new S0 and no new S1. Its harness notes — one refused click that would otherwise have gone to
  a browser window, one mis-set step table, one lost foreground that was retried only after raising the app
  window, and a geometry driver that initially matched somebody else's window and was fixed before any
  installed action — are in `11_harness/DRIVER.txt` and `10_install/INSTALL_RECORD.txt`, classified as
  harness, and kept.

