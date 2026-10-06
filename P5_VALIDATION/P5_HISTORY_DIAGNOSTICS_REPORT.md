---
title: "P5 History and Diagnostics Report"
doc_id: "FS-P5-HISTORY-DIAGNOSTICS"
product: "FirmwareSight"
version: "0.6.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — History and Diagnostics, seen from inside the installed build

Prompt §32 lists nine History properties and nine Diagnostics properties. Commit C built History and
Commit D built Diagnostics, both proven with tests. **This file is the F2 record of those same surfaces
being operated in the shipping `firmwaresight-desktop.exe`**, which is the layer no earlier round reached.

Everything below was produced by clicking. The database was read only to confirm what the screen had
already shown, always through `file:…?mode=ro`.

## 1. History — §32's nine properties

| Property | What the installed build did | Evidence |
| --- | --- | --- |
| **builds** | lists every stored build newest-first: 3 rows, `16:04:17Z` (no-MAP target), `15:50:24Z` (target), `15:36:06Z` (base), each with artifact leaf name, snapshot, truncated SHA-256, stored time, format and both footprints | `11_history/R00–R12` |
| **Gate** | one run, `gate-5fa…e11eebb9`, disposition **Passed**, findings broken out by state — Pass 8 / Review 0 / Block 0 / Unknown 0 / Not applicable 2 — stored `16:20:34Z` | same |
| **release** | one record, `release-…6819fa85`, version 1.2.3, naming the build, the Gate run that qualified it and the manifest digest `450860c5…d544df69`, stored `16:37:17Z` | same |
| **pagination** | every list is bounded and says so: "Showing 1 to 3 of 3 builds", "1 to 1 of 1 Gate runs", "1 to 1 of 1 release records", with Previous and Next **disabled** at the ends rather than wrapping or erroring | same |
| **detail** | Details expands for a build, a Gate run and a release, and shows ids, digests, sizes and timestamps only | same |
| **source-independent reopen** | the strongest of the nine, and the one a test cannot make: the app was closed, `%TEMP%\F2-firmware-project` was **renamed** so no source file exists at its recorded location, and the app was reopened from the same Start Menu shortcut. History rendered the same three builds, one Gate run and one release. A read-only replay of the store reports identical row payloads (`11_history/store_before_move.json` vs `13_reopen/store_after_move.json`) | `13_reopen/SOURCE_MOVED.txt`, `S00–S02` |
| **repair persistence** | after the §21 repair over the installed app, History still reads 3 / 1 / 1 and the semantic before/after diff is `DIFF_KEYS = NONE` across all eight compared keys | `14_repair/REPAIR.txt` |
| **uninstall / reinstall persistence** | across two full uninstall → reinstall cycles the store came back with the same three digests and the same History rows; the reinstalled app's own Diagnostics counts matched | `15_uninstall/STATE_*.txt`, `16_reinstall/W05–W06`, `20_second_cycle/X01–X02` |
| **path safety** | no artifact absolute path, no project root, no database path and no bundle destination appears anywhere on the page. The build detail states the rule in the product's own words: *"The file name is all FirmwareSight keeps visible here. Where the artifact lives is never shown, and this row does not need the file to still exist."* The release detail says the bundle folder is not part of the record and is never shown | `11_history/R00–R12` |

Two properties that are not in §32's list but matter to a reader:

- **There is no edit or delete control anywhere on the page.** The only row action is Details, and the Gate
  list states the reason: *"A stored run is immutable, so this list cannot be edited and a re-run adds a row
  rather than replacing one."* Storage enforces this with triggers on the Gate tables, so the sentence is a
  description of a constraint rather than a promise.
- **Keyboard reachability.** `Ctrl+Home` and `PageDown` move the page and the filter groups take a visible
  focus ring. This was checked by driving those keys, not by reading CSS.

## 2. Diagnostics — §32's nine properties

The export F2 produced is 1,122 bytes, `schema: "firmwaresight-diagnostics-1"`, and it was parsed outside
the product rather than trusted.

| Property | Measured |
| --- | --- |
| **41-key allowlist** | `ALLOWED_KEYS: [&str; 41]` at `apps/desktop/src-tauri/tests/diagnostics.rs:47`. F2's instance carried **37** keys — under the ceiling, because a field is only present when the store can actually say something about it. No key outside the allowlist appeared at the top level or nested |
| **8 structs** | `DiagnosticsDto`, `DiagnosticsProductDto`, `DiagnosticsRuntimeDto`, `DiagnosticsStoreDto`, `DiagnosticsCountsDto`, `DiagnosticsGitDto`, `DiagnosticsSupportDto`, `DiagnosticsPolicyDto` (`apps/desktop/src-tauri/src/ipc.rs:1402–1535`) |
| **integrity** | `store.health: "healthy"`, `schemaVersion: 5` with `supportedSchemaVersion: 5`, `journalMode: "wal"`, `backupFiles: []`. A read-only `PRAGMA integrity_check` on the same store returned `ok`, so the product's health verdict and the raw check agree |
| **Git** | `git.available: true`, `version: "git version 2.55.0.windows.4"` — a version string, and no remote, because the allowlist has no key for one |
| **runtime** | `osFamily: windows`, `architecture: x86_64`, `platform: windows-x86_64`, `osVersion: "not_reported"`, `tauriVersion: 2.12.0`, `webviewVersion: 154.0.4258.53`. Note `osVersion` reporting `not_reported` rather than guessing: an absent fact is stated as absent |
| **counts** | `projects 1, builds 3, gateRuns 1, acceptedReviews 0, releaseRecords 1` — each equal to the read-only `SELECT COUNT(*)` on the same store at the same moment, so the export is coherent as well as private |
| **logical store** | `storeFileName: "firmwaresight-p0.sqlite"` — file name only, no parent directory. The historical `p0` stays in the name deliberately, and the Help screen says the store is named by file, not by folder |
| **privacy export** | `target/f2_diagnostics_privacy.py` asserted **11 absence classes** against the raw bytes: no firmware path, no project root, no database path, no bundle destination, no git remote, no username or home, no Release Notes body, no symbol name, no firmware byte, no MAP content, no environment dump. All 11 clean; `DIAGNOSTICS_PRIVACY = PASS` (`12_diagnostics/privacy_report.json`) |
| **no upload / network capability** | the file is written by the native save dialog and goes nowhere else; the running app owned no TCP endpoint; the WebView document origin is the packaged `http://tauri.localhost/` custom protocol, not a dev server; and the capability set contains no network plugin |

`policy: null` and `recentErrorCodes: []` in F2's export are worth reading correctly: they are not failures.
The policy block is populated when a release policy has been loaded into the running session, and the error
list is bounded at eight stable codes — this session had none to report.

## 3. Where the export went, and a trap a support engineer will hit

Both native dialogs on this machine restore the **OS last-used folder**, which happens to be a private
firmware project directory belonging to the owner. F2 therefore cancelled the first export rather than
writing into it, verified the directory was unchanged afterwards (no `firmwaresight-diagnostics*` entry, and
its files still carried their May dates), and re-exported to the evidence root by typing the target path.

This is ordinary Windows picker behaviour, not a product defect, and the product's privacy position is
unaffected — the payload carries no path either way. But it is a real operational trap: a user who has just
analysed a private firmware and then exports diagnostics will find the dialog offering to drop the file
into that same private tree, silently. It belongs in the supportability guidance
(`P5_SECURITY_SUPPORTABILITY_REVIEW.md` §5) rather than in the limitations register, because nothing about
the product's behaviour is wrong.

## 4. What this file does not claim

- It does not claim History or Diagnostics were stress-tested for volume. Every list here held 1–3 rows.
  The read APIs are bounded and the export's row counts are bounded, but a store with thousands of builds
  has never been walked in an installed window.
- It does not claim the privacy assertions generalise beyond this payload. The 11 absence classes were
  checked against **this** file, produced by **this** store, on **this** machine.
- It does not offer a restore. Diagnostics tells a support conversation what the store believes about
  itself; the recovery path is still the pre-migration snapshot on disk
  (`P5_MIGRATION_RECOVERY_REPORT.md` §6).
