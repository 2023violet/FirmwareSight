---
title: "P5 Install and Recovery Report"
doc_id: "FS-P5-INSTALL-RECOVERY"
product: "FirmwareSight"
version: "0.6.0"
status: "IN_PROGRESS"
owner: "Engineering"
last_updated: "2026-10-04"
---

# P5 — real Windows install, uninstall and data behaviour

**What this document proves and what it does not.** Sections 38 A–L and section 39 were executed on this
machine with the real P5 installer, and every line below is a transcript of what the operating system and
the application did. Section 38 item **C — first-run onboarding — is not built yet**, and with it the whole
section 64 journey (`→ Gate → Bundle → History → Diagnostics`) stays open: those surfaces arrive in Commits
C and D. `status: IN_PROGRESS` is therefore the honest header for this file, and nothing here says
`P5 PASS`, `BETA`, `RC` or `GA`.

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
| C first-run onboarding | **not built.** The cold window shows the rail and "Nothing has been analyzed in this session yet. Choose an artifact and run Analyze." and nothing else | open — Commit C, prompt §13 |
| D Analyze a real supported fixture | `fixtures/elf/p0-basic/firmware.elf` through the native dialog titled "Choose the firmware artifact to analyze": File `firmware.elf`, SHA-256 `1c0ae94e…`, Size `7,288 bytes`, Format `Arm 32-bit little`, Kind `elf`, Parser `object-elf/0.40`, Entry `0x00008001`, Build ID "- no .note.gnu.build-id section is present"; Memory: Nonvolatile `Exact 160 bytes`, Runtime RAM `Partial 2 … 72 bytes` | screenshot transcript plus the store rows it wrote |
| E close / reopen | the close button ended the process (0 instances afterwards); relaunch from the same shortcut worked; the reopened window again said nothing had been analyzed | see §3 — this is where the real finding is |
| F manual reinstall / repair equivalent | re-running the same installer produced a maintenance page: "FirmwareSight 0.6.0 is already installed. Select the operation you want to perform", with **Add/Reinstall components** preselected. With the app still running the installer interrupted: **"FirmwareSight is running! Click OK to kill it"**; after OK, `uninstall.exe` was rewritten (11:59:42) while `firmwaresight-desktop.exe` kept its packaged timestamp (11:41:42) and digest `c68ebe28…`. The store was untouched by the repair: 1 build, 19 sections before and after | directory listing, digests, store query |
| G uninstall | the uninstaller's only page said "Uninstall FirmwareSight / Remove FirmwareSight from your computer. / Uninstalling from: `%LOCALAPPDATA%\FirmwareSight\`" — **no question about user data was asked, and none was deleted** | UI transcript plus §4 |
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

**Uninstall preserves data, silently and completely.** The uninstaller never mentions user data, never asks,
and removes nothing outside its install directory. That is the safe default and it is also a documentation
obligation: prompt §39 says app data may be deleted only "explicitly documented/confirmed", so the
retention behaviour has to be written where a user will read it (`P5_KNOWN_LIMITATIONS.md` and the install
guide, Commit F), not just measured here.

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

1. Commit C's onboarding makes §38 C executable and gives §66's questions something to answer from.
2. Commits C and D add History and Diagnostics, which turns §64's journey from a list of verbs into a run.
3. The full §64 journey then runs once, end to end, on the installed build, against a disposable firmware
   project, and this file's `status` changes from `IN_PROGRESS` on that evidence — with the same
   park-and-restore discipline used here.

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
