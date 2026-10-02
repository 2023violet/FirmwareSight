---
title: "G2 End-to-End Smoke Report"
doc_id: "FS-G2-SMOKE"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "G2_ENGINEERING_CLOSURE_AUDIT"
owner: "Engineering"
last_updated: "2026-10-01"
---

# G2 — whole-MVP smoke on the shipping build (prompt §19–§26)

Every row below is something read out of the running window, computed from a file a FirmwareSight
process wrote, or read out of a store that process held. Nothing is inferred from the test suite;
where a control could not be driven the way a hand would drive it, the row says how it was driven.
Host paths are written in environment-variable form so no account name enters the repository.

## Run under test

| | |
| --- | --- |
| Audited tree | `e35cfe70aa0e8f273a75ac14b9dc62377483af36` — `055b54e` plus the G2-F1 test-only fix; tracked changes 0 when the binaries were built |
| Desktop binary | `cargo build --release -p firmwaresight-desktop --features custom-protocol`, exit 0, 182 s ("Finished `release` profile [optimized] target(s) in 3m 02s"), after `corepack pnpm build` exit 0. `target/release/firmwaresight-desktop.exe`, 14,995,968 B, sha256 `84a26a0528f838fa8328ac251fa4ac4c4f4602c1dde48a7594279d79da6bc018` |
| CLI binary | `cargo build --release -p fwsight`, exit 0 (up to date). `target/release/fwsight.exe`, 4,257,792 B, sha256 `80feaff561cd4528f432bc047ecb4ed4427d1044bb636828941485ddebcc4a10` — the digest P4 recorded, because no CLI source changed since |
| Page | WebView2 `Edg/154.0.4258.48`, page URL `http://tauri.localhost/` — the embedded assets of a `custom-protocol` build, not a dev server; viewport 1040 × 760, device pixel ratio 1 |
| Database | the user's live `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite{,-shm,-wal}` renamed to `*.pre-g2-smoke` before launch (sha256 recorded); the app created a fresh store; see "Database handling" |
| Release subject | `%LOCALAPPDATA%\Temp\fs_g2\subject\project`, a throwaway Git repository. This repository is never a release subject |

### How the window was driven

This round had no screen-control connector, so the shipped window was driven through two standard
local interfaces, both test-harness only — no product source, configuration or capability changed:

- **WebView2 DevTools Protocol.** The process was started with the documented WebView2 environment
  variable `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333` (loopback only).
  Clicks are `Input.dispatchMouseEvent` press/release at the element's centre and keys are
  `Input.dispatchKeyEvent`, so they travel the real input pipeline rather than a synthetic
  `element.click()`. What the window showed was read with `document.body.innerText` and
  `Page.captureScreenshot`.
- **Win32 messages for native dialogs.** The file and folder pickers are owned by the Rust process, so
  they were found by exact title among visible `#32770` windows belonging to `firmwaresight-desktop`,
  the name field (control id 1148, or 1152 in the folder picker) received `WM_SETTEXT`, and the
  default button received `BM_CLICK`.

Harness incidents, none of them a product event: UI Automation `ValuePattern.SetValue` timed out
(`HRESULT 0x80131505`) on the first dialog and was replaced by Win32 messages; the first `WM_SETTEXT`
went through a wrong P/Invoke overload and set nothing, so the empty Open closed the picker with no
selection and the page correctly still read "Nothing has been analyzed in this session yet."; one
substring match clicked `Analyze page` instead of `Analyze` and was replaced by exact-name matching; one
destination click landed while Prepare was still busy and was retried (see scenario C).

## Inputs

| Side | File | Bytes | SHA-256 |
| --- | --- | --- | --- |
| base | `source/base/firmware.elf` | 16,820 | `3f615b6247179d930f59b76dcdbaea1a5eca8c835ab6d410feb20a062f33f21f` |
| base | `source/base/firmware.map` | 4,511 | `832690060a8d25f8acacd85a62ce76bc83dfb6768e435bcc04c51f9664ed363e` |
| target | `source/target/firmware.elf` | 17,464 | `4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374` |
| target | `source/target/firmware.map` | 4,664 | `a38575cd51ec2730442a0f3b8506c99fc49eec68d4b28ecdc9dda9434709ace6` |

Copies of the committed P2 pair `fixtures/elf/p2-diff/{base,target}`. Workspace: project `g2-node`, one
commit `b4446801d145a1963eb9838b8b42bce6583dda9a` (author and committer pinned, date
`2026-10-01T08:00:00+00:00`), tag `v2.0.0`, `git status` clean, its own top level. Policy
`firmwaresight.toml` (file sha256 `921365b1…`, canonical policy hash
`8f392ca7b1024900ca05e5048c63e794dfb964d0876add244e586f4062f9e8c1`): flash 512, RAM 256, flash growth
review 200, required `["elf","map"]`, `source = "git_tag"` with the P4 pattern, expected version
`2.0.0`, clean Git and Release Notes `docs/RELEASE_NOTES.md` (73 B, sha256 `05d78e86…85018bb2`)
required. Against the pair (target 376 B / 76 B, growth +120 B) every configured rule answers PASS with a
figure, and no Review is needed — which is what lets the CLI, which cannot accept a Review, be compared
with the desktop directly (§19).

## CLI whole-MVP chain (§20)

Non-interactive (`< /dev/null`), the shipped `fwsight.exe`, run twice into two different destination
parents (`out-a/` and `deeper/out-b/nested/`):

| Step | Command | Exit | Captured |
| --- | --- | --- | --- |
| Analyze base | `analyze base/firmware.elf --map base/firmware.map --json` | 0 | `snap-3f615b62…f33f21f-p0-normalize-1-83269006…4ed363e`; stdout sha256 `be523747…` |
| Analyze target | `analyze target/firmware.elf --map target/firmware.map --json` | 0 | `snap-4b4087e1…5ddd8374-p0-normalize-1-a38575cd…709ace6`; stdout `88b6ff6a…` |
| Diff | `diff base target --old-map … --new-map … --json --html diff.html` | 0 | diff.json `1930cdb6a098c67456297ec321910315d21aeb3ce79ef7264fcc4037e148242a`; diff.html `aa4b08df…` |
| Gate | `gate --project … --artifact … --map … --baseline … --baseline-map … --json` | 0 | `gate-e692eed242f1eee250f8b0968a7c08e8f4bee7645ae646faff9ca5e355bc91ed`, `overall_effective_severity` PASS, 10 findings (8 PASS, 2 N/A → effective PASS); json `9ed4d45b…` |
| Bundle | `release prepare … --out <dest> --json` | 0 | `release-2a7849c0f359141f8d385d4b8fe807e3970ad435c90c67b4ab7363d4752af9e7`, folder `g2-node-2.0.0-2a7849c0f359`, manifest `85e53c18d8846109be149fe659a9cbacab722e4652b9d1dcf8d38d1d4cf43abe` (stdout byte-equal to the file written) |

Bundle file set, both runs: `accepted-reviews.json` `a2cd337e`, `analysis.json` `1d80b90c`,
`artifacts/firmware.elf` `4b4087e1`, `artifacts/firmware.map` `a38575cd`, `diff.json` `1930cdb6` (= the
CLI diff document), `gate-results.json` `9ed4d45b` (= the CLI gate document), `release-manifest.json`
`85e53c18`, `release-notes.md` `05d78e86`, `release-report.html` `d81df476`, `SHA256SUMS` `4d6d3406`.

- **Run A vs run B:** every captured fact and every bundle file is byte-identical, so the destination
  folder moves no identity.
- **No database side effect:** the app's store directory listing (sizes and mtimes) and the sha256 of
  the live `sqlite`/`shm`/`wal` were identical before and after both chains; no `*.sqlite*` appeared
  under the subject or the outputs. Gate and release both print `this run is not stored`.
- **No interactive input, no desktop process.** `git status` of the subject stayed clean after both chains.
- **Independent reader** on the CLI bundle: `scripts/verify_bundle_portability.py --schemas schemas`
  **64 of 64**.

## Desktop whole-MVP smoke (§21)

| # | Step | Result | What was actually seen |
| --- | --- | --- | --- |
| 1 | launch | PASS | page title `FirmwareSight`; "Nothing has been analyzed in this session yet." on the fresh store |
| 2 | only Analyze / Compare / Release | PASS | rail buttons exactly `Analyze page`, `Compare page`, `Release page` |
| 3 | Analyze base + MAP | PASS | picker "Choose the firmware artifact to analyze" → the page names only `firmware.elf`; "Choose the linker MAP for this artifact" → `MAP: firmware.map`; report visible 366 ms after the Analyze click |
| 4 | base artifact hash | PASS | SHA-256 `3f615b62…f33f21f`, 16,820 bytes; snapshot equal to the CLI's; NV 256 / RAM 8 exact, layout `map`, weakest basis `map-memory-configuration+elf-load`, admissible for a hard limit, dual-accounted `elf.section_header[4]`, 3,628 B excluded, 19 sections / 47 symbols / 10 evidence — every field equal to the CLI JSON |
| 5 | Analyze target + MAP | PASS | a new artifact choice resets MAP to `Not provided`; Add MAP → `firmware.map`; report in 352 ms |
| 6 | target artifact hash | PASS | SHA-256 `4b4087e1…5ddd8374`, 17,464 bytes; snapshot = CLI; NV 376 / RAM 76 exact, dual-accounted `[4]`, `[6]`, 4,294 B excluded, 19 / 52 / 11 = CLI |
| 7 | Sections | PASS | "Showing 1 to 19 of 19 sections"; `.bss` file offset `Unknown`, not 0 |
| 8 | Symbols | PASS | "Showing 1 to 52 of 52 symbols"; an empty-name entry renders "Unknown the symbol entry has an empty name" |
| 9 | Evidence | PASS | "Showing 1 to 11 of 11 evidence", all observed; Inspect `dual_accounted` → Locator `elf.section_header[4] + map:load-address`, Rule `map-memory-configuration+elf-load`, Evidence id `ev-dual-4` |
| 10 | bytes / KiB | PASS | KiB: `.text` 0.207 KiB, `.data` 0.00391 KiB, NV 0.367 KiB, RAM 0.0742 KiB; addresses, offsets and snapshot id unchanged; Bytes restores 212 / 4 / 376 / 76 |
| 11 | Compare base → target | PASS | both stored builds offered; Old = `snap-3f615b62…`, New = `snap-4b4087e1…` |
| 12 | old / new identity | PASS | full snapshot ids in both selectors; file name, short sha, 16,820 / 17,464 bytes |
| 13 | FLASH delta | PASS | 256 → 376, **+120 bytes**, exact = CLI `memory.nonvolatile` |
| 14 | RAM delta | PASS | 8 → 76, **+68 bytes**, exact = CLI `memory.runtimeRam`; pair comparability exact |
| 15 | section changes | PASS | 18 rows: Changed 16, Added 1, Removed 1 = CLI `sectionChanges` (18) |
| 16 | symbol changes | PASS | 76 rows: Added 38, Removed 33, Changed 5 = CLI (76); `SYMBOL-AMBIGUOUS` 67 unpaired = CLI warning |
| 17 | added / removed / changed separate | PASS | `.calib` Removed "a removal is not a change to zero"; `.ota` Added "an addition is not a change from zero"; unchanged 2 sections / 9 symbols = CLI |
| 18 | object attribution Unavailable | PASS | "Object attribution unavailable — the persisted symbol and section records carry no object-file or module attribution…" = CLI `objectChanges` `{available: false}` with the same reason |
| 19 | export Diff JSON | PASS | "Export the build diff as JSON" → 64,355 B, sha256 `1930cdb6…` — **byte-identical** to `fwsight diff --json` |
| 20 | export Diff HTML | PASS | "Export the build diff as HTML" → 41,953 B, sha256 `aa4b08df…` — **byte-identical** to `fwsight diff --html`; status "Wrote diff.html (HTML)." |
| 21 | Release page | PASS | "Release Gate", "1 · Project policy", "2 · Build and baseline"; with no policy the page says a run judges the stated default policy |
| 22 | load project config | PASS | "Choose the project's firmwaresight.toml" → `g2-node`, `schema_version 1 · policy 8f392ca7…e9e8c1` = CLI `policy_sha256`; text-and-attribute host-path sweep clean |

| 23 | select target | PASS | Current build defaults to the last analyzed `snap-4b4087e1…`; card SHA-256 `4b4087e1…5ddd8374`, 17,464 bytes |
| 24 | select baseline | PASS | native `<select>` focused and moved with ArrowDown → `snap-3f615b62…`; card `3f615b62…2f33f21f`, 16,820 bytes. The mouse-opened popup itself was not driven — the native-popup limit P2 step 27 recorded — so this row is the keyboard path, which is a real user path |
| 25 | Run Gate | PASS | `gate-e692eed2…91ed` = the CLI run id; Workspace HEAD `b4446801…583dda9a`, exact tag `v2.0.0`, "clean (workspace, not artifact)", "Git facts describe the workspace FirmwareSight read. They are not proof of how this artifact was built" |
| 26 | disposition PASS | PASS | "Disposition: PASS", "As computed: PASS"; Block 0 · Review 0 · Unknown 0 · Pass 8 · Not applicable 2 |
| 27 | all ten rule rows | PASS | 10 of 10 CLI findings on the page with the same rule id, summary and effective severity. The page groups rows by state (the eight PASS, then the two N/A); `gate-results.json` keeps canonical rule order. Presentation, not a different fact |
| 28 | UNKNOWN not fabricated as PASS | PASS | Unknown is 0 here because every fact was answered; the two N/A rows carry their policy reason. The Unknown path is shown in scenario U below and in the CLI's no-Git scenario B |
| 29 | Prepare Bundle | PASS | button enabled only on PASS; the preview appears |
| 30 | verify preview | PASS | `release-2a7849c0…af9e7` = CLI; version 2.0.0; build, baseline and run as above; accepted reviews 0; folder `g2-node-2.0.0-2a7849c0f359` = CLI; ten files whose digests equal the CLI bundle's; page host-path sweep clean |
| 31 | choose destination | PASS | "Choose the folder to create the release bundle in" → "Destination chosen for g2-node-2.0.0-2a7849c0f359. Nothing of that name is there yet."; Export enabled only now |
| 32 | export | PASS | "Bundle created" |
| 33 | verify success | PASS | release id as above, Manifest SHA-256 `85e53c18d8846109be149fe659a9cbacab722e4652b9d1dcf8d38d1d4cf43abe`, "Files 10 (2 artifacts)", "verified against its own SHA256SUMS and manifest before it was put in place. The release is recorded in this database." |
| 34 | inspect bundle independently | PASS | ten files; the destination holds only the bundle folder; `diff -r` against the CLI run-A bundle: **byte-identical** |
| 35 | schemas | PASS | `verify_bundle_portability.py --schemas schemas`: **64 of 64** (jsonschema 4.26.0) |
| 36 | SHA256SUMS | PASS | every listed digest recomputed; "SHA256SUMS lists neither itself nor the manifest"; "the manifest covers SHA256SUMS" |
| 37 | report offline | PASS | `file://` in a separate Chrome: 0 scripts, 0 links, 0 images, 0 iframes, one inline style block, no `src`/`href` but anchors, no resource entries, network log the document only; nine numbered sections; release id, version, tool version, run id and all five state labels present. Not a literal Explorer double-click |
| 38 | no host path in generated documents | PASS | the reader's "names no path on this machine" on all seven composed files; report `hostPath: false`; page sweep after export clean |
| 39 | relocate | PASS | copied to `fs_g2\relocated-root\elsewhere\`; tree fingerprint `b1095576f899d25f` unchanged |
| 40 | source / project / DB unavailable | PASS | subject (project and source artifacts), the validation store and the original bundle all moved away; no FirmwareSight process |
| 41 | relocated bundle verifies | PASS | reader **59 of 59** without schemas, **64 of 64** with; the nine questions answered from the bundle alone |
| 42 | restore what the smoke changed | PASS | subject and outputs back (HEAD `b444680`, clean, target ELF `4b4087e1…`); the smoke store kept as `firmwaresight-p0.sqlite.g2-smoke{,-wal,-shm}`; the user's store renamed back, sha256 of `sqlite`/`shm`/`wal` identical to before |
| 43 | clean close | PASS | `CloseMainWindow` (`WM_CLOSE`) → exited within 10 s; debug port closed. Done before 39–41 so the store could be made unavailable |

Additional observations on the same window:

| | Check | Result | Seen |
| --- | --- | --- | --- |
| O1 | same destination again | PASS | "A folder of that name is already there, and it reads as a FirmwareSight bundle." → Export → `ERR-BUNDLE-6106` "… exporting would replace the bundle that is there"; group "Replace the existing bundle?" |
| O2 | focus moves to that decision | PASS | `document.activeElement` = the Replace button the moment the question appears — the behaviour G2-F1's repaired test pins |
| O3 | Keep it, by keyboard | PASS | Tab → "Keep it", Enter → "Kept the bundle that was there. Nothing was written."; tree fingerprint identical before and after |
| O4 | explicit Replace, by keyboard | PASS | Enter on Replace → "This export replaced the bundle … The release record was not written; the bundle itself is complete."; manifest `85e53c18` again; fingerprint unchanged; still byte-identical to the CLI bundle |
| K | keyboard reachability | PASS | 22 real Tab presses on Release reach every primary control and the rail; 0 of 22 without a visible ring (2 px `rgb(37, 99, 235)`) |
| W | overflow at 1040 × 760 | PASS | page `scrollWidth` ≤ viewport on all three pages; wide detail tables scroll inside their own wrapper |
| S | states not colour-only | PASS | each count and finding carries an icon and a label; UNKNOWN is a dashed hollow circle in `status.unknown` (`#667085`, `fill: none`), PASS a filled `#18794E` circle, N/A a `#98A2B3` dash |

## Cross-surface parity (§22)

| Fact | CLI | Desktop | Verdict |
| --- | --- | --- | --- |
| artifact SHA-256, base / target | `3f615b62…` / `4b4087e1…` | same | same |
| base / target SnapshotId | `snap-3f615b62…-83269006…` / `snap-4b4087e1…-a38575cd…` | same, full strings compared | same |
| memory facts | NV 256 / 376, RAM 8 / 76, all exact, layout map, same weakest basis, same dual-accounted and excluded bytes | same | same |
| Diff JSON | `1930cdb6…` | exported `1930cdb6…` | **byte-identical** |
| Diff HTML | `aa4b08df…` | exported `aa4b08df…` | **byte-identical** |
| Gate run id | `gate-e692eed2…91ed` | `gate-e692eed2…91ed` | same |
| Gate findings | 10, rule id / state / effective severity / summary | 10, all four fields equal | same facts; the page groups by state for reading |
| policy hash | `8f392ca7…` | `8f392ca7…` | same |
| release version / id | 2.0.0 / `release-2a7849c0…` | 2.0.0 / `release-2a7849c0…` | same |
| `release-manifest.json` | `85e53c18…` | `85e53c18…` | **byte-identical** |
| every bundle file | ten digests | ten digests | **`diff -r` identical** |

## Failure and recovery (§23)

| Scenario | Surface | Result | What happened |
| --- | --- | --- | --- |
| A — dirty Git after a PASS | CLI | PASS | Release Notes edited → `gate` exit 5, `gate-3e8e451f…`, `git.clean` BLOCK; `release prepare` exit 5 `ERR-BUNDLE-6101` "only a PASS disposition may be packaged"; 0 bundles |
| A — dirty Git after a stored PASS | Desktop | PASS | Prepare against the stored `gate-e692…` → `ERR-BUNDLE-6102` "the Gate run selected is `gate-e692…` but the current project context recomputes to `gate-3e8e451f…`"; no preview. Run Gate → BLOCK `gate-3e8e451f…` — the CLI's id; Prepare disabled. Restored → Run Gate → PASS `gate-e692…` with the original `Stored 2026-10-01T19:09:37Z`: same content, same id, no second record |
| B — no Git repository | CLI | PASS | config, notes and artifacts copied to a directory outside any work tree → `gate` exit 4, `git.clean` UNKNOWN → effective REVIEW, `release.version_matches_policy` UNKNOWN → effective REVIEW ("this directory is not a git repository"); `release prepare` exit 4 `ERR-BUNDLE-6101`; 0 bundles |
| U — Unknown on the desktop | Desktop | PASS | baseline cleared with a growth threshold configured → REVIEW, Unknown 1, `diff.growth` UNKNOWN effective REVIEW "no baseline snapshot was supplied", Next step "Select a baseline build…"; "No bundle is prepared: the disposition is REVIEW"; Prepare disabled |
| C — source mutated after the preview | Desktop | PASS | fresh Prepare and destination; last byte of the source target ELF flipped (`4b4087e1…` → `6c9919f7…`) → Export → `ERR-BUNDLE-6103` "… its SHA-256 is not the digest the snapshot holds" / "A snapshot's hash is never updated to match a file that moved"; destination empty; the earlier bundle's fingerprint unchanged; no staging, aside or temp entry anywhere; destination and Export disabled (the plan is consumed); source restored byte-exact |
| F — malformed input | Desktop | PASS | `truncated-elf.bin` → "Analysis failed / Could not parse artifact as ELF. / ERR-PARSE-2002" with *Why we know* and *What to do*; "Previous analysis of firmware.elf. It is not an analysis of truncated-elf.bin."; Compare still offers exactly the two builds |

Scenario A's first CLI restore used `git checkout`, and the account's global `core.autocrlf=true`
checked the file back out with CRLF (`i/lf w/crlf`): not the bytes it had. The file was rewritten from
its LF source (sha256 `05d78e86…` again) and re-staged so the index stat matched; the Gate then
reproduced `gate-e692…` byte for byte. A throwaway copy with CRLF notes produced a different run id and
`git.clean` BLOCK — recorded in `G2_KNOWN_LIMITATIONS.md` as an environment effect, not a defect.

## Supported input and fail-closed (§26, CLI)

| Input | Exit | Result |
| --- | --- | --- |
| `p0-basic` ELF; `p0-dual-region` ELF + MAP; P2 base + MAP; P2 target + MAP; P2 target ELF only | 0 ×5 | 42–52 ms each, no panic; without a MAP the build says `map: not-provided`, `layoutSource: none`, `admissibleForHardBlock: false` |
| `malformed/empty.bin`, `sparse-elf-header.bin`, `wrong-magic.bin` | 3 | `ERR-FORMAT-0001` "That file is not a format FirmwareSight can analyze yet." |
| `malformed/truncated-elf.bin` | 3 | `ERR-PARSE-2002` "Could not parse artifact as ELF." |
| plain text file; an Intel HEX file; 4 KiB of random bytes | 3 | `ERR-FORMAT-0001` — BIN and HEX are **not** analyzed, and nothing claims they are |
| ELF with a non-MAP file as `--map` | 3 | `ERR-MAP-3001` "That MAP file is not from a supported linker." |
| missing path | 3 | `ERR-INPUT-0001` |
| target ELF with `--max-bytes 1024` | 3 | `ERR-GUARD-0001` "Artifact is 17464 bytes, above the 1024 byte full-buffer limit." — the guard mechanism; the 512 MiB default is P0's measurement |

No input produced a panic line; every failure wrote a JSON error document (≈200 B) to stdout and an
explicit code to stderr.

## Database handling

The smoke store, read from a copy after the window closed: `integrity_check` ok; `schema_migrations`
0001–0004; builds 2; gate runs 3 (`e692…` PASS, `3e8e451f…` BLOCK, `938ea372…` REVIEW without a
baseline) — the restored-workspace re-run deduped onto `e692…`; `release_records` 1 (`release-2a7849c0…`,
gate `e692…`, version 2.0.0, manifest `85e53c18…`) — the Replace export wrote no second record;
`accepted_reviews` 0; no BLOB cell. A host-path sweep over all 2,507 cells found four hits, every one
in `artifacts.path`: the local source path of each analyzed file — finding **G2-F2**, adjudicated by
the architect's Storage Path Semantics Clarification Addendum as expected local-only persistence
(`04_TECH/15` §7), with the 17-point boundary proof in `G2_EVIDENCE_MATRIX.md`. No other table, and no
`gate_finding_evidence` or `release_records` cell, holds a path. The live store was handed back byte-for-byte; the smoke store stays
beside it as `*.g2-smoke`.

## Not observed, not run

- A literal Explorer double-click on `release-report.html` (step 37 used a browser at its `file://` URL).
- The baseline `<select>` opened by mouse (step 24 used the keyboard path).
- A second Windows host, any DPI other than 100%, a macOS or Linux window.
- A 500 MB working set in the window; peak RSS. See the PRD metric table in the closure report.
