---
title: "P4 Release Bundle Desktop Smoke Report"
doc_id: "FS-P4-SMOKE-DESKTOP"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P4_RELEASE_BUNDLE"
owner: "Engineering"
last_updated: "2026-10-01"
---

# P4 Release Bundle — shipped Windows desktop smoke (prompt §59)

Every line below is something this report's author saw in a capture of the running window, computed
from a file the running app wrote, or read out of the database the running app holds. Nothing is
inferred from the test suite; where a step could not be reached through the window, it says so.

## Run under test

| | |
| --- | --- |
| Binary | `target/release/firmwaresight-desktop.exe`, 14,995,968 bytes, sha256 `54a87e341917ada3955939496301163f74eb35691b605bea09aa690d3f6244c9` |
| Build | one chained `cargo build --release -p fwsight` + `cargo build --release -p firmwaresight-desktop --features custom-protocol`; both printed `Finished release profile [optimized]` (54.87 s and 3 m 49 s) and the `sha256sum` stage that followed printed the two digests above, so the chain reached its end |
| CLI sibling | `target/release/fwsight.exe`, 4,257,792 bytes, sha256 `80feaff561cd4528f432bc047ecb4ed4427d1044bb636828941485ddebcc4a10` |
| Source equivalence | the last commit touching any shipped source path is `6b7e23e` (04:05:06); the binaries were written at 05:22:06 and 05:25:56; `git diff --stat 6b7e23e HEAD -- '*/src/*'` is empty, so the walked binary was built from shipped source identical to the final tested tree |
| Window | 1056 × 799 at (52,52); the capture tool reports 1015 × 768 window-relative pixels (×1.0404); main window id 5243742, pid 14380 |
| Database | clean validation DB; the pre-existing one was moved aside and handed back afterwards — see "Database handling" |
| Release subject | `%TEMP%\p4-smoke\project`, a throwaway Git repository. This repository is never a release subject (§47, §59) |
| Runs | One continuous session of the same window walked all 50 steps; a context compaction fell inside it (between steps 40 and 41), and the window stayed open across it, which is why steps 4–20's on-screen facts were captured again while walking 41+ |

## Inputs

Hashes are from `sha256sum` on the staged files, not from the screen.

| Side | File | Bytes | SHA-256 |
| --- | --- | --- | --- |
| base | `artifacts/base/firmware.elf` | 16,820 | `3f615b6247179d930f59b76dcdbaea1a5eca8c835ab6d410feb20a062f33f21f` |
| base | `artifacts/base/firmware.map` | 4,511 | `832690060a8d25f8acacd85a62ce76bc83dfb6768e435bcc04c51f9664ed363e` |
| target | `artifacts/target/firmware.elf` | 17,464 | `4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374` |
| target | `artifacts/target/firmware.map` | 4,664 | `a38575cd51ec2730442a0f3b8506c99fc49eec68d4b28ecdc9dda9434709ace6` |

Workspace HEAD `c593a305964e5358e2e142d8593532ddaaf2c488`, tagged `v1.2.3`; policy sha256
`c28e9968cdb8144f2aef87403421d5de78cc4c04f81a7c6771a8b1e42cf61865` (flash 512, RAM 256, growth
review 100, required `["elf","map"]`, `source = "git_tag"`, expected version `1.2.3`, notes at
`docs/RELEASE_NOTES.md`) — the same policy shape as the committed fixture
`fixtures/project/p4-release/firmwaresight.toml`, with a growth threshold that yields exactly one
REVIEW so the acceptance path has something to accept.

## How the window was driven

Input and capture came from the computer-use connector: `get_window_state` screenshots for the
WebView2 window (its UIA tree exposes only region nodes, so clicks are coordinate-based against a
fresh capture), UIA `Invoke` for native dialog controls where the tree is real, and `type_text` +
`Return` for the file and folder dialogs. Every click was followed by a capture to confirm it
landed, and several did not land the first time: the first attempt at "Choose destination folder"
missed because the page had scrolled, the folder picker once needed a click on 选择文件夹 *and* a
`Return` to close, one close attempt hit the maximize button instead of 关闭 (the window went
maximized and was restored), and one UIA element-index click resolved to the window centre and was a
no-op. The visible result, not the intent, is what is reported; step 50's close completed with
`Alt+F4`, which delivers the same `WM_CLOSE` the title-bar button sends.

## The 50 steps

| # | Step | Result | What was actually seen |
| --- | --- | --- | --- |
| 1 | launch shipping binary | PASS | window `FirmwareSight - Analyze`; rail lists exactly Analyze / Compare / Release; empty-session text on the clean DB |
| 2 | Analyze base + MAP | PASS | native dialog `Choose the firmware artifact to analyze`; typed path accepted with Return; report shows `snap-3f615b62…64ed363e` |
| 3 | Analyze target + MAP | PASS | report shows `snap-4b4087e1…4709ace6`, 17,464 bytes; `nonvolatile 376 bytes (exact)` |
| 4 | Release page | PASS | heading `6 · Release Bundle` under the Release rail item; no fifth navigation verb |
| 5 | load project config | PASS | OS dialog; path typed into the file-name field, accepted with Return |
| 6 | select target | PASS | target build row `snap-4b4087e1407c…a9434709ace6` |
| 7 | select baseline | PASS | baseline row `snap-3f615b624717…1f9664ed363e` |
| 8 | Run Gate | PASS | stored run `gate-9eab29ee3ec4be1b2eb91db80b0618b69a1065648f7dda3a5b77116803b5cd73`; disposition REVIEW with one finding |
| 9 | produce REVIEW scenario | PASS | growth row `FLASH 256 bytes → 376 bytes, +120 bytes, threshold 100 bytes, REVIEW effective REVIEW · delta exact` |
| 10 | record valid acceptance | PASS | acceptance by `rosa`, reason "the growth is the new bootloader, checked against the MAP", `accepted_at 2026-10-01T12:57:21Z`, original state REVIEW |
| 11 | confirm disposition PASS | PASS | `This run's disposition is PASS, so the release can be packaged into a folder you choose.` |
| 12 | Prepare Bundle | PASS | `Preparing…` then the preview block; the button re-enables afterwards |
| 13 | verify preview release id | PASS | `release-de87846cad6b1412ba302847d2f4cae5d9b874b215608d1eec4446d1173fbec9` |
| 14 | verify project release version | PASS | `1.2.3` — the tag the policy's pattern extracts, not an export-time value |
| 15 | verify current snapshot | PASS | `snap-4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374-p0-normalize-1-a38575cd51ec2730442a0f3b8506c99fc49eec68d4b28ecdc9dda9434709ace6` |
| 16 | verify baseline | PASS | `snap-3f615b6247179d930f59b76dcdbaea1a5eca8c835ab6d410feb20a062f33f21f-p0-normalize-1-832690060a8d25f8acacd85a62ce76bc83dfb6768e435bcc04c51f9664ed363e` |
| 17 | verify Gate run id | PASS | `gate-9eab29ee3ec4be1b2eb91db80b0618b69a1065648f7dda3a5b77116803b5cd73`, the id step 8 stored |
| 18 | verify accepted review count | PASS | `Accepted reviews 1` |
| 19 | verify expected file list | PASS | ten rows in bundle-path order, `accepted-reviews.json` 415 B … `SHA256SUMS` 675 B, with the digests the plan computed before anything was written |
| 20 | verify no host path | PASS | preview and notes blocks name `docs/RELEASE_NOTES.md` project-relatively; no drive letter, no `%TEMP%`, no `p4-smoke` anywhere in the page |
| 21 | Choose destination | PASS | folder dialog titled `Choose the folder to create the release bundle in`; `%TEMP%\p4-smoke\release-out` typed and accepted; status `Destination chosen for brake-node-1.2.3-de87846cad6b. … Nothing of that name is there yet.` |
| 22 | export bundle | PASS | `Writing…` then the result block |
| 23 | verify success result | PASS | `Bundle created`: release id as step 13, folder `brake-node-1.2.3-de87846cad6b`, `Manifest SHA-256 1397ae9cbdd2ce397b5d043ded718c86ca29dea969bd69b90583eca7aaf369e8`, `Files 10 (2 artifacts)` |
| 24 | inspect bundle directory independently | PASS | `find` lists exactly the ten files; the app's DB directory then held only `firmwaresight-p0.sqlite` |
| 25 | artifact ELF hash matches snapshot | PASS | on-disk `artifacts/firmware.elf` = `4b4087e1…5ddd8374`, the digest the preview and the manifest carry |
| 26 | MAP hash matches snapshot | PASS | on-disk `artifacts/firmware.map` = `a38575cd…4709ace6` |
| 27 | analysis.json validates | PASS | against `schemas/analysis.schema.json` by the independent reader |
| 28 | diff.json validates | PASS | against `schemas/diff.schema.json` |
| 29 | gate-results.json validates | PASS | against `schemas/gate-results.schema.json` |
| 30 | accepted-reviews.json validates | PASS | against `schemas/accepted-reviews.schema.json`; holds the one acceptance of step 10 verbatim |
| 31 | release-manifest.json validates | PASS | against `schemas/release-manifest.schema.json`; `sha256sum release-manifest.json` = `1397ae9c…af369e8`, the digest the screen printed at step 23 |
| 32 | SHA256SUMS verifies | PASS | `files: 9 sums ok: True` — every listed payload recomputed to its recorded digest |
| 33 | report opens without app resources | PASS | opened at its `file://` URL in a browser: `scripts 0, links 0, imgs 0, iframes 0, subresources []`, one inline style block. Not a literal Explorer double-click; the offline-rendering claim is what was tested |
| 34 | report shows FirmwareSight version | PASS | `hasToolVersion: true` read out of the live document |
| 35 | report shows project release version | PASS | `hasReleaseVersion: true`, alongside `hasReleaseId: true` |
| 36 | report shows Gate finding states | PASS | `states: ["PASS","REVIEW","N/A","UNKNOWN","BLOCK"]` present as icon+label pairs |
| 37 | report shows accepted Review audit | PASS | `hasAcceptance: true`; heading `6. Accepted reviews` |
| 38 | report contains no host path | PASS | `hostPath: false` in the live document, and the independent reader's sweep over the file |
| 39 | move bundle to another temp root | PASS | copied to `%TEMP%\p4-smoke\relocated\` |
| 40 | verify again with project/source/DB unavailable | PASS | `project` and `artifacts` renamed away first; the relocated copy passed 59 of 59 independent checks; inputs then restored byte-for-byte (`4b4087e1…`, `3f615b62…`, policy `f9c437d0…`) |
| 41 | choose same destination again | PASS | folder dialog re-opened, same path typed and accepted |
| 42 | confirmation required | PASS | `ERR-BUNDLE-6106`: "`brake-node-1.2.3-de87846cad6b` already exists in the chosen folder, and exporting would replace the bundle that is there"; "choose another folder, or confirm the replacement explicitly"; diagnostics `op-382c-18da6b030352e810`; group `Replace the existing bundle?` with two buttons |
| 43 | cancel replace → prior bundle byte-identical | PASS | `Keep it` → status `Kept the bundle that was there. Nothing was written.`; tree fingerprint before and after identical: 10 files, tree sha256 `d35c7527394e6cfbe3b5fcb769d945d926b9aff2d59bd8a95115ffbd2f5fa839`; destination holds only the bundle folder |
| 44 | explicitly confirm replace | PASS | Export again → 6106 again with a fresh diagnostics id `op-382c-18da6b2e81c5ee60` → pressed `Replace the existing bundle named brake-node-1.2.3-de87846cad6b` |
| 45 | safe replacement succeeds | PASS | `Bundle created` again with the same manifest digest; note: "This export replaced the bundle that was there, under your confirmation. The written folder was verified against its own SHA256SUMS and manifest before it was put in place. The release record was not written; the bundle itself is complete."; the bundle files' creation time moved to 06:35:25 while the database WAL's last write stayed at the first export's 06:08 — the UI's "record not written" claim checked against the store |
| 46 | modify source ELF after plan | PASS | last byte of `artifacts/target/firmware.elf` flipped: `4b4087e1…5ddd8374` → `06d0f3e5…dec6e3`; a byte-exact backup kept beside it |
| 47 | export stale plan | PASS | pressed `Export bundle` with the plan still held |
| 48 | export rejected; no partial replacement | PASS | `ERR-BUNDLE-6103`: "the source of `firmware.elf` is not the bytes the build recorded: its SHA-256 is not the digest the snapshot holds"; "re-analyze the intended artifact, or restore the exact bytes. A snapshot's hash is never updated to match a file that moved"; diagnostics `op-382c-18da6b7bf3811294`; on disk the tree fingerprint is unchanged (`d35c7527…`, 10 files) and no staging or aside directory exists anywhere under the smoke root |
| 49 | restore exact source | PASS | backup copied back and removed; digest is `4b4087e1…5ddd8374` again |
| 50 | clean close | PASS | `WM_CLOSE` delivered; process 14380 gone; the database reopens with exactly one release record and `integrity` intact — see "Database handling" |

## Two findings this smoke produced

**The independent reader's clock rule was too coarse (checker defect, fixed).** The first portability
run over the exported bundle reported 62 of 64 with two failures, both the blanket "no date-shaped
value" check: `accepted-reviews.json` records when a named person accepted a review, and the report
prints that same date. Those are recorded human acts, not generation times. The rule now names the one
clock a bundle may carry — `acceptances[].accepted_at` — and checks the report against the review
record rather than against nothing, so it still cannot invent a date. Re-run: 64 of 64 with schema
validation, 59 of 59 without. Fixed in `e799f2f`; the bundle itself was never wrong.

**A stale plan is consumed, so restoring the bytes alone does not re-enable Export (observation, by
design).** After step 48, pressing Export again with the bytes restored did nothing and the error
panel kept its diagnostics id: `ERR-BUNDLE-6103` is in the UI's stale-plan set, which drops the preview
and the destination (`Release.tsx`, `STALE_PLAN_CODES`), so the handler returns before calling the
engine. The remedy the error names — "re-analyze the intended artifact, or restore the exact bytes" —
is exactly what a fresh Prepare does, and a fresh Prepare over the restored bytes produced the same
release id and re-enabled the flow. Recorded because a reader of step 48 could otherwise expect the
same plan to export again.

## Not observed, not run

- A literal Explorer double-click on `release-report.html` (step 33 used a browser at the `file://`
  URL, which is the offline-rendering claim).
- Keyboard-only traversal of the shipped window; a second Windows host; any DPI other than 100%.
- Re-opening an older release record through the window: the Release page shows the run this session
  computed, and `release_records` is read back through the store, not through a UI control.

## Database handling

`%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite` held the pre-smoke database. It was
moved aside to `…\firmwaresight-p0.sqlite.p3-pre-p4-smoke` before the smoke and handed back
afterwards; the live file is again that one — 8 tables, migrations 1–2, i.e. exactly the pre-P4 state.
The smoke's own database is kept, not deleted, as `…\firmwaresight-p0.sqlite.p4-smoke`: migrations
1–4 and the single release record
`release-de87846cad6b1412ba302847d2f4cae5d9b874b215608d1eec4446d1173fbec9`, version `1.2.3`,
manifest `1397ae9c…af369e8`, `created_at 2026-10-01T13:08:23Z` — the first export's moment, unchanged
by the two later ones.

Two handling notes, recorded rather than smoothed over. Mid-smoke an attempt to move the live database
failed with `Device or resource busy` while the app held it, so the "verify with the DB unavailable"
half of step 40 was completed after the clean close, which is when it could be honest. And a
read-only probe of the set-aside file orphaned its `-wal`/`-shm` sidecars (a 0-byte WAL and its shm);
both were probe artifacts of this session and were removed; nothing of the app's was touched.
