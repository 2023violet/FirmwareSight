---
title: "P5 Install and Recovery Report"
doc_id: "FS-P5-INSTALL-RECOVERY"
product: "FirmwareSight"
version: "0.6.0"
status: "IN_PROGRESS"
owner: "Engineering"
last_updated: "2026-10-05"
---

# P5 — real Windows install, uninstall and data behaviour

**What this document proves and what it does not.** Sections 38 A–L and section 39 were executed on this
machine with the real P5 installer, and every line below is a transcript of what the operating system and
the application did **on 2026-10-03**. Section 38 item **C — first-run onboarding — was not built when this
round ran**, and that is why the row reads as it does; Commits C and D have since built it together with Help,
History and Diagnostics, and the 2026-10-04 walk (§7 of this file) saw onboarding and Diagnostics operating
inside the installed binary. **The gap that sentence describes is now closed**: Commit F2 walked the whole
section 64 journey end to end on 2026-10-05, in one pass, on the exact CI-built F1 Windows artifact, against
a disposable firmware project and with the owner's store parked and restored twice. §8 records it and §9
writes the user-facing policy that §39 made this round's obligation. What still keeps this file at
`status: IN_PROGRESS` is neither an unbuilt surface nor an unrun journey — it is that P5 does not close in
F2, and the closure sentence belongs to Commit F3. Nothing here says `P5 PASS`, `BETA`, `RC` or `GA`.

Environment decision first, because section 38 asks for it: **Windows Sandbox is not available on this
host** — `C:\Windows\System32\WindowsSandbox.exe` does not exist, and the optional-feature query
(`Get-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM`) requires elevation, which
this round does not take unilaterally (`AGENTS.md` §9). Section 38's fallback is therefore what ran: the
current machine, with the owner's live store hashed, parked, and restored, and the restore verified.

## 1. The store was parked before anything could touch it

The installed application opens `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite`
(`apps/desktop/src-tauri/src/lib.rs:919-922` — `app_data_dir()`, then that filename), which is the owner's
live store. So it moved out of the way first, and the digests below are the proof it came back unchanged.

| File | Bytes | SHA-256 before | SHA-256 after restore |
| --- | --- | --- | --- |
| `firmwaresight-p0.sqlite` | 155,648 | `d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468` | identical |
| `firmwaresight-p0.sqlite-shm` | 32,768 | `fd4c9fda9cd3f9ae7c962b0ddf37232294d55580e1aa165aa06129b8549389eb` | identical |
| `firmwaresight-p0.sqlite-wal` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | identical |

Recorded with `sha256sum` over every file in the app-data directory before the installer ran, and again after
the restore; the full listing is `PRE_INSTALL_STATE.md` in the P5 evidence root, outside this repository.
Nothing in that directory was deleted at any point — the test-created store was moved aside into
`INSTALL_TEST_STORE_RETAINED/` instead, because it is evidence for §4 below.

## 2. Section 38 A–L, item by item

| Item | What happened | How it was checked |
| --- | --- | --- |
| A fresh install | NSIS 3.11 wizard: Welcome → Choose Install Location → Choose Start Menu Folder → progress → "Installation Complete / Setup was completed successfully." | the wizard was driven through its own UI, not a silent flag |
| B launch from a normal OS surface | `cmd /c start "" "…\Start Menu\Programs\FirmwareSight\FirmwareSight.lnk"` started the installed binary; window title `FirmwareSight - Analyze`, process path `%LOCALAPPDATA%\FirmwareSight\firmwaresight-desktop.exe` | `list_apps` and `Get-Process` both report that path, not a `target/` path |
| C first-run onboarding | **not built at this run (2026-10-03).** The cold window shows the rail and "Nothing has been analyzed in this session yet. Choose an artifact and run Analyze." and nothing else | built since in Commit C and **seen operating in the installed binary on 2026-10-04** — see §7 |
| D Analyze a real supported fixture | `fixtures/elf/p0-basic/firmware.elf` through the native dialog titled "Choose the firmware artifact to analyze": File `firmware.elf`, SHA-256 `1c0ae94e…`, Size `7,288 bytes`, Format `Arm 32-bit little`, Kind `elf`, Parser `object-elf/0.40`, Entry `0x00008001`, Build ID "- no .note.gnu.build-id section is present"; Memory: Nonvolatile `Exact 160 bytes`, Runtime RAM `Partial 2 … 72 bytes` | screenshot transcript plus the store rows it wrote |
| E close / reopen | the close button ended the process (0 instances afterwards); relaunch from the same shortcut worked; the reopened window again said nothing had been analyzed | see §3 — this is where the real finding is |
| F manual reinstall / repair equivalent | re-running the same installer produced a maintenance page: "FirmwareSight 0.6.0 is already installed. Select the operation you want to perform", with **Add/Reinstall components** preselected. With the app still running the installer interrupted: **"FirmwareSight is running! Click OK to kill it"**; after OK, `uninstall.exe` was rewritten (11:59:42) while `firmwaresight-desktop.exe` kept its packaged timestamp (11:41:42) and digest `c68ebe28…`. The store was untouched by the repair: 1 build, 19 sections before and after | directory listing, digests, store query |
| G uninstall | the uninstaller's page says "Uninstall FirmwareSight / Remove FirmwareSight from your computer. / Uninstalling from: `%LOCALAPPDATA%\FirmwareSight\`" — **and, as measured on the F1 CI artifact on 2026-10-05, it also presents a "Delete the application data" checkbox, unchecked by default.** This row originally read "no question about user data was asked"; that was what the 2026-10-03 walk saw, and §8 corrects it against the shipping artifact | UI transcript plus §4, and `15_uninstall/V00_uninstall_confirm.png` in the F2 evidence root |
| H app executable removed | `%LOCALAPPDATA%\FirmwareSight` **emptied but not deleted** — `firmwaresight-desktop.exe` and `uninstall.exe` are gone and the directory itself is left behind at 0 entries; `Start Menu\Programs\FirmwareSight` gone; the `HKCU\…\Uninstall\FirmwareSight` entry gone | `ls -la` on both paths, registry enumeration |
| I documented user-data retention | measured, not asserted: the app-data directory and its store survived the uninstall intact | §4 |
| J reinstall | the same installer again, same wizard, "Installation Complete" again | UI |
| K retained data behaves as documented | the reinstalled app reopened the **same** store: digest `da63a52b…` identical before and after relaunch, `builds = 1`, and `schema_migrations` still five rows with `min(applied_at) = 2026-10-03T18:51:07Z` — the migrations did **not** re-run | `sqlite3` in read-only URI mode (`file:…?mode=ro`) |
| L final cleanup / restore | app closed, installed files removed, test store moved to the evidence root, owner's store restored and re-hashed to the three digests in §1; `PATH` entries mentioning FirmwareSight: **0**; leftover installer/uninstaller processes: 0; `git status` clean and `git diff -- fixtures` empty | the probes in `PRE_INSTALL_STATE.md` §5–§6 |

## 3. Three findings this round produced, none of them from the checklist

**The window title lies on an installed build.** After navigating to Compare, the page heading was
`Compare`, the rail highlighted `Compare`, and `Get-Process … MainWindowTitle` still returned
`FirmwareSight - Analyze`. The cause is not a bug in the navigation: `tauri.conf.json:15` sets a single
static string, and that string happens to name the first page — so the title is wrong for three of the four
pages a user can reach, and it will be wrong for a fifth when History lands. This is L21, reproduced on the
packaged artifact rather than in a source read, and it is the concrete form of prompt §14's "window title
should reflect FirmwareSight and current major page/state".

**A returning user can see nothing they did before.** Item E is the one that matters: the store kept
1 build, 1 artifact, 19 sections, 42 symbols and 10 evidence rows across a close and a reopen, and the UI
reported "Nothing has been analyzed in this session yet" and, on Compare, "Analyze another firmware build
before comparing." Both sentences are true of the session and false of the product's only durable state.
There is no surface in the shipped build where a persisted fact can be seen again — which is exactly what
prompt §15–§17 authorizes History to close, and it is now evidenced rather than argued.

**Uninstall preserves data, and — on the shipping artifact — says so.** This round's original wording was
"the uninstaller never mentions user data, never asks, and removes nothing outside its install directory".
The last clause holds. The first two do not: on the F1 CI artifact the uninstall page carries a
**"Delete the application data" checkbox, unchecked by default**, so the user is asked and the safe answer is
the one already selected. `tauri.conf.json` sets no `deleteAppDataOnUninstall`, which is why the box is
offered rather than acted upon. That is a better position than the one originally recorded, and it is still a
documentation obligation: prompt §39 permits app data to be deleted only when that is explicitly documented
and confirmed, and a checkbox most users will not read is not documentation. §8 writes the policy in user
words and names the one path that was deliberately never exercised.

## 4. Section 39 record

| Field | Value |
| --- | --- |
| Install location | `%LOCALAPPDATA%\FirmwareSight` — per-user, no `Program Files`, no elevation prompt at any point |
| Package size | installer 3,811,421 bytes; installed executable 15,001,088 bytes; wizard reported "Space required: 14.3 MB" |
| Installer hash | `eac31920213072e4e6a7e1cd74c58bc7e4c396418ced46f3bce1d656323e6567` (this host's build of head `53578e9`; the CI set's installer is a different digest, as `P5_PACKAGING_REPORT.md` §5e explains) |
| Uninstaller behaviour | UI-driven only; deletes the files it installed, the Start Menu folder and the `HKCU` entry, and leaves the now-empty install directory behind; asks nothing about user data and deletes none |
| User-data behaviour | `%APPDATA%\com.firmwaresight.desktop` untouched by repair, uninstall and reinstall |
| Start Menu behaviour | one folder, one shortcut: `FirmwareSight/FirmwareSight.lnk`; the wizard offers a "Do not create shortcuts" checkbox; the finish page as captured showed only "Installation Complete", a progress summary and a "Show details" control, with no launch checkbox in view — so B was satisfied by the OS surface rather than by the wizard, which is the stronger form of the test either way |
| PATH | 0 entries mentioning FirmwareSight before, 0 after (user and machine) |
| Shell profile | none created (`Documents\PowerShell\Microsoft.PowerShell_profile.ps1` absent) |
| Network | the running app owned **no** TCP endpoint (`netstat -ano` filtered on its PID returned nothing); the WebView document origin is `http://tauri.localhost/`, i.e. the packaged custom protocol, not a dev server |
| Firmware project directories | untouched: `git status` clean and `git diff -- fixtures` empty after analyzing a fixture in place |
| UAC | never prompted — `installMode: currentUser` in `tauri.conf.json` behaves as recorded in `04_TECH/17` |

One thing this round did **not** settle: `start "" uninstall.exe /S` exited 0 and removed nothing, leaving a
stray `Un.exe` behind. That is recorded as an inconclusive probe of an invocation path this repository does
not document, not as a defect claim — the verified removal route is the uninstaller's own UI, and no
guidance in this project tells a user to run it silently.

## 5. Section 65, as far as it can go on this host

The installed binary ran with no dev server, no `cargo`, no `node` and no repository in its path: it loaded
from `%LOCALAPPDATA%`, served its frontend from `http://tauri.localhost/`, and owned no socket. What this
host cannot prove is the negative half — it has the whole toolchain installed, so "runs without Rust,
Cargo, Node, pnpm or Vite" is evidenced as *does not use them*, not as *would fail without them*. The
runtime assumption that is documented rather than tested: **WebView2**, which `tauri.conf.json` relies on
and which this machine already carries; a machine without it is the cleanest-environment gap section 65
points at, and it stays open.

## 6. What closes this file

1. ~~Commit C's onboarding makes §38 C executable and gives §66's questions something to answer from.~~
   **Done** — Commit C built it, Commit D's walk saw it installed, and Commit F2 walked it as step 12 of the
   full journey (§8).
2. ~~Commits C and D add History and Diagnostics, which turns §64's journey from a list of verbs into a run.~~
   **Done** — both surfaces were operated inside the installed binary on 2026-10-05, including a
   source-independent reopen and an independently parsed diagnostics export.
3. ~~The full §64 journey then runs once, end to end, on the installed build…~~ **Done on 2026-10-05**, on the
   exact F1 CI artifact rather than a local build, with the same park-and-restore discipline. §8 records it.

**What still keeps `status: IN_PROGRESS` is no longer an unrun journey.** It is that this file is a P5
validation record and P5 does not close in Commit F2: `P5 = IN_PROGRESS` and the closure sentence belongs to
F3. Two named boundaries also survive — the install-directory residue that varies between runs (§8), and the
data-deletion option that was deliberately never exercised (§8) — and neither is a reason to call the walk
incomplete, only a reason not to call it total.

Every number above came from a command in this session. The raw transcripts, the store listings, the
screenshots of the wizard pages and the retained test store live in the P5 evidence root outside this
repository, named `PRE_INSTALL_STATE.md` and `INSTALL_TEST_STORE_RETAINED/`.

## 7. What Commit D's installed walk settled, and what it did not (2026-10-04)

Item 1 and half of item 2 above have since happened, on a packaged build, and the record says so here rather
than leaving §2's rows to read as permanent.

- **§38 C is no longer "not built".** The Commit D walk installed
  `FirmwareSight-0.6.0-windows-x86_64-nsis.exe` and the cold window rendered the **Getting started** panel —
  the seven facts, "Hide this", and the sentence that hiding them hides nothing else. So the row that read
  "**not built.** … open — Commit C, prompt §13" is now: built, and **seen operating in an installed
  binary**. Its `status` consequence is limited to exactly that: it was seen, its controls were not
  stress-tested, and its dismissal persistence was not re-checked after a reopen.
- **Diagnostics is now an installed surface with a file behind it.** §27's focused walk exported the payload
  through the native dialog, parsed it outside the product, and found no path-shaped character anywhere in
  it. That is `P5_DIAGNOSTICS_RECOVERY_REPORT.md` §2, rows 8-11, and it closes the Diagnostics half of item 2.
- **History was not visited.** The walk went install → Analyze → Help → close → reopen → Help → export →
  uninstall. The History page has still never been operated in an installed binary, so item 2's other half
  stays open and this file stays `IN_PROGRESS`.
- **One measurement here no longer generalizes.** §2 H recorded
  `%LOCALAPPDATA%\FirmwareSight` left behind **empty but present** after an uninstall. Commit D's
  install → uninstall cycle on the same host left **no such directory**. Both rows are true for their own
  run; the rule was not worked out, and the user-facing uninstall wording is Commit F's to write against a
  fresh measurement rather than against either one.
- **§64's full journey is still one run away**, and it is the same list as before: install → onboarding →
  Analyze → Compare → Gate → Bundle → History → Diagnostics → close → reopen → reinstall → uninstall →
  reinstall → documented data behaviour, against a disposable firmware project.
  **It is no longer one run away.** Commit F2 ran it on 2026-10-05; §8 below replaces this bullet.

## 8. Commit F2: the §64 journey, run end to end on the exact F1 CI artifact (2026-10-05)

The candidate was `FirmwareSight-0.6.0-windows-x86_64-nsis.exe`, 3,885,631 bytes, SHA-256
`a1152ef31a2c28b0fa522d88b7fe08eee543af1b86a161a4ab60acf04b3c076b`, taken from GitHub Actions artifact id
`11337963032` of run `37293381181` at head `0bca373` and installed **as downloaded** — no local rebuild,
because a locally built installer would have tested this host's toolchain rather than what the repository
ships. The installed binary is `firmwaresight-desktop.exe`, 15,362,048 bytes,
`93be8c8e0abc711f1a98f497cec6837f1710ef9674e3cbcbb3ac7c10c0020c50`, version 0.6.0.

Every action was a real `SendInput` mouse or keyboard event against the installed window. No database write
stands in for a UI action; SQL ran only read-only, to check what the screen had already done. No WebView2
debugging protocol and no synthetic UIA `Invoke` were used — the UIA tree exposes WebView content as generic
regions, so screenshots are the only channel that can show what a user would see. The full journey, its
per-step measurements and its findings classification are in
`P5_DESKTOP_ACCEPTANCE_REPORT.md`; this section records only what belongs to install and recovery.

### What install and recovery looked like this time

| Step | Measured |
| --- | --- |
| Repair over a running install | the wizard offered "Already Installed → Add/Reinstall components", showed the **unchanged** destination, and raised "FirmwareSight is running! Click OK to kill it". Confirming terminated pid 16552 and the install section continued to "Setup was completed successfully" |
| Binary refresh on repair | `firmwaresight-desktop.exe` re-extracted **byte-identical** (`93be8c8e…`, packaged mtime preserved); `uninstall.exe` rewritten (`d1b1fa11…`, mtime advanced); Start Menu and desktop shortcuts rewritten in place, one each |
| Logical data across repair | read-only semantic replay gives **`DIFF_KEYS = NONE`** across eight compared keys — schema 5, migrations 1..5 once each, `integrity_check ok`, 1 project / 3 builds / 5 artifacts / 57 sections / 151 symbols / 31 evidence / 3 footprints / 1 gate run / 10 findings / 1 release |
| Uninstall | install directory **ABSENT** (removed entirely, not left empty); desktop shortcut, Start Menu `.lnk`, Start Menu folder and the `HKCU` registration all gone; `PATH` sha unchanged; the disposable firmware project identical at 44/44 hashed files; store retained with an identical semantic snapshot |
| Reinstall | same artifact, footprint restored — identical binary digest, identical `uninstall.exe` digest, same shortcut sizes, one `HKCU` entry. The only differences from the pre-uninstall footprint are mtimes and the process id |
| Retained data behaves as documented | version 0.6.0 on Help/About; existing v5 store opens with health `healthy`; **no migration re-run** (`schema_migrations` still exactly five rows, export `backupFiles: []`); History still 3 builds / 1 Gate run / 1 release; Analyze still functional; diagnostics counts coherent with a read-only query; no duplicate rows |
| Final state | uninstalled again, machine back to what it was; disposable inputs preserved outside the repository and removed from `%TEMP%` |

### Two things this pass settled, and one it did not

**The install directory after uninstall is not stable, and no rule has been worked out.** Four observed
uninstalls on this host: the 2026-10-03 walk left `%LOCALAPPDATA%\FirmwareSight` behind **empty**, Commit D's
2026-10-04 walk left **no directory**, and all three of F2's uninstalls left **no directory**. Both earlier
rows stay in the record as true for their own run. The honest wording is therefore "normally removed; an
empty directory may remain as installer residue" — which is what §9 says, rather than picking the majority
outcome and calling it a guarantee.

**The uninstaller does ask about user data.** §2 G originally recorded that no question was asked. On the F1
CI artifact the page carries **"Delete the application data", unchecked by default**, and
`tauri.conf.json` sets no `deleteAppDataOnUninstall` — so the box is offered, and the safe answer is already
selected. Whether the 2026-10-03 build lacked it or the walk missed it is not established; what is
documented from here is the shipping artifact's behaviour.

**What was deliberately not tested: checking that box.** `%APPDATA%\com.firmwaresight.desktop` holds,
beside the F2 disposable store, unrelated historical stores from earlier phases — `.g2-smoke`, `.p2-smoke`,
`.p4-smoke`, `.p0-smoke-history` and two `postg2-e2e-*` directories whose retired WAL files run to ~1 GB
each. A checkbox labelled "the application data" that acts on the whole folder would destroy evidence that
belongs to no part of this round, and no acceptance goal required learning what it does. Recorded as
`RETENTION_DESTRUCTIVE_OPTION = NOT_TESTED_BY_DESIGN` — a coverage boundary, not a pass.

### Owner-data discipline, applied twice

The installed app resolves its store from Tauri's `app_data_dir()` with no environment override
(`apps/desktop/src-tauri/src/lib.rs:953`), so running it at all requires the owner's store to be out of the
way. F2 parked, verified and restored it at 07:34, and — because §24 needed a second installed cycle that
the first pass had not fully covered — again at 18:04. Both passes followed the same order: confirm no
process holds it, hash it, copy it byte-for-byte and verify the copy, park the originals, confirm the live
path is clear; on the way back, verify the parked copy against the §6 digests **before** moving it, then
re-verify after. `d6e41034…` / `e3b0c442…` / `fd4c9fda…` every time, and `sibling stores lost: 0` in both
passes.

Reading the restored store produced the pass's most useful fact: **the owner's real store is at schema v2.**
That proves the installed candidate never opened it — had it done so, migrations 0003–0005 would have run —
and it also bounds what the installed migration proof can claim. See
`P5_MIGRATION_RECOVERY_REPORT.md` §4.

## 9. What a user is told (prompt §30)

This is the policy text, written to match what §8 measured rather than what seems likely. It uses
environment-variable paths and no machine-specific ones.

> **Uninstalling FirmwareSight.** The uninstaller removes the application files, the desktop shortcut, the
> Start Menu folder and the `HKCU` registration entry under which Windows finds it. It does **not** remove
> your local FirmwareSight data. The History store at
> `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite` stays where it is, along with any
> pre-migration snapshots the application kept beside it, so reinstalling later reopens the same History
> rather than an empty one. The uninstall page offers a **"Delete the application data"** checkbox; it is
> **unchecked**, and ticking it is the only way the uninstaller will remove that folder.
>
> The install directory at `%LOCALAPPDATA%\FirmwareSight` is normally removed with the program. In some runs
> an **empty** directory is left behind; that is installer residue, it holds no data, and deleting it is
> safe.
>
> **Reinstalling or repairing.** Running the installer again over an existing install offers a maintenance
> choice ("Add/Reinstall components"). It rewrites the program files and the shortcuts in place and leaves
> your data untouched. If FirmwareSight is running, the installer asks before closing it rather than
> terminating it silently. Windows' "Apps & features" does not offer Modify or Repair for this install, so
> the way to repair it is to run the installer again.
>
> **Upgrading.** There is no automatic update. Upgrading means running the newer installer over the older
> install; the store is migrated on first launch and a snapshot of it is written beside the store first if
> it was created by an older version.

Nothing in that paragraph overstates what was measured: the retention is proved semantically at the row level
across three uninstalls and two reinstalls, the residue sentence is exactly what four runs showed, the
running-app prompt was seen on both the repair and the uninstall path, and the absence of Modify/Repair in
the registry was read from `NoModify` / `NoRepair` rather than inferred. The one thing it does not describe
is what the checked box does, because that path was never exercised — which is why the text says only that
ticking it "is the only way the uninstaller will remove that folder" and not what, precisely, it then keeps.
