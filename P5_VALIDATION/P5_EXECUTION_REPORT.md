---
title: "P5 Execution Report"
doc_id: "FS-P5-EXECUTION"
product: "FirmwareSight"
version: "0.6.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — execution report

P5 ran in six heads. This is the round's own record of what each did, what was measured, and what was
deliberately not done. It is written by Commit F2 and therefore reports F2 completely and F3 not at all.

*(the sentence above was true when F2 wrote it on 2026-10-05 and stopped being true the moment F3 existed. §7
is F3's continuation: the complete head lineage, the red and interrupted runs kept as they happened, and the
closure. Sections 1–6 are left exactly as F2 and F2R wrote them.)*

| Head | What it delivered | Gate |
| --- | --- | --- |
| `d83175a` → `a674774` | the audit, migration `0005`, version unification, packaging and the three CI package jobs | 10/10 from `1055242` onward |
| `9ab089f`, `4719e1f` | §38 A–L and §39 measured on this host with the owner's store parked, hashed and restored | 10/10 each |
| Commit B–C–D (`e863d0c` … `956e250`) | onboarding, Help/About with the window title fixed, History over three bounded read APIs, storage-owned `integrity_check`, the pre-migration snapshot, the 41-key Diagnostics allowlist, `ADR-0028` | 10/10, one red head (`3400981`) repaired test-only in `bccea88` |
| Commit E (`859648e`, `59d85c3`, `6981625`, `80b47c4`) | the compatibility fixture cohort, the supportability sweep over §48's fourteen rows, the L15 answer as Option E, and the closure normalization | 10/10; `80b47c4` needed two attempts and both stay in the record |
| **Commit F1** `0bca373` | `ADR-0029` decided and implemented — the repository baseline now digests canonical Git stage-0 index blobs, verified inside the authoritative drift step — plus L15's display-only caption | 10/10, attempt 1, run `37293381181` |
| **Commit F2** (this head) | the §64 installed journey, the installed v4 → v5 migration proof, the §28 closure pack, the §29 user-doc audit, §30's policy wording and §31's coverage boundary | see `P5_EXIT_CHECKLIST.md` |

## 1. What Commit F2 actually did

Ran the whole §64 journey once, contiguously, on the exact CI-built F1 Windows artifact — installed as
downloaded, never rebuilt locally — and wrote the closure evidence from what the machine did. `P5_DESKTOP_
ACCEPTANCE_REPORT.md` is that record.

Four things worth stating plainly because they are the round's real content:

- **Everything was driven with real input.** `SendInput` mouse and keyboard against the installed window,
  native file dialogs operated by typing a path and pressing the actual Open button. No DB write stood in for
  a UI action; SQL ran read-only, to check what the screen had already shown. No WebView2 debugging protocol,
  no synthetic UIA `Invoke`.
- **The migration proof is 34 measured checks, not a narrative.** A repository-valid synthetic v4 store
  (built by applying the repository's own migrations `0001`–`0004` with `PRAGMA foreign_keys = ON`) was
  genuinely History-readable *before* the upgrade, so "rows preserved" could be compared row-by-row rather
  than asserted.
- **Retention is proved semantically.** Every before/after in this round compares read-only replays of the
  product's own candidate, Gate and release SQL plus table counts and `PRAGMA integrity_check`, because a
  WAL database legitimately churns bytes: across the repair the main file and WAL predated the operation and
  only `-shm` moved.
- **Two coverage boundaries were written down instead of being smoothed over.** Installed migration is one
  path, and the uninstaller's data-deletion option was never exercised. Both are in
  `P5_KNOWN_LIMITATIONS.md` §6 and explained where they could mislead.

## 2. What Commit F2 did not do, and why

| Not done | Because |
| --- | --- |
| Any change under `crates/`, `apps/`, `scripts/`, `fixtures/`, `schemas/`, `migrations/`, `.github/`, `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml` | §3 and §35. Any one of them invalidates the F1 installer as the candidate under test. Three product findings were recorded and **not patched** |
| Setting `P5 PASS_COMPLETE`, `Productization ENGINEERING_COMPLETE`, or `active_task NONE` | §39 and §42: those belong to F3 |
| Starting F3 | §42: "Do not start F3 … Return to Architect" |
| Signing, notarizing, publishing, tagging, creating a GitHub Release, enabling an updater | §34, and `AGENTS.md` §2 keeps the updater behind a signing ADR |
| Choosing an open-source licence or adding a `LICENSE` file | `AGENTS.md` §9 owner decision; no P5 prompt authorised an agent to pick one |
| Exercising the uninstaller's "Delete the application data" | that folder also holds unrelated historical stores from earlier phases, including ~1 GB of retired WAL files. Recorded as `NOT_TESTED_BY_DESIGN` |
| Rerunning a red CI attempt to get a green | §40 keeps attempt history truthful; a content failure is fixed forward, not re-rolled |

## 3. User-doc audit (prompt §29)

Each required topic mapped to the surface that **already** documents it, before any new file was considered.
Status is about the surface, not about the wish.

| Topic | Canonical surface | Status | Gap |
| --- | --- | --- | --- |
| Getting Started | `apps/desktop/ui/src/GettingStarted.tsx` (seven facts), shown on Analyze and repeated in `Help.tsx` | substantive | in-app only; no standalone markdown, and none is warranted |
| Installation | `P5_VALIDATION/P5_INSTALL_RECOVERY_REPORT.md` §2 and §8; install mode in `04_TECH/17` "Windows install mode" | substantive | engineering transcript, not user prose — §9 now carries the user-facing wording |
| Analyze | `apps/desktop/ui/src/Analyze.tsx`; `03_DESIGN/06_UI_REFERENCE_SCREENS.md` | substantive | design-facing; no end-user how-to |
| Compare | `apps/desktop/ui/src/Compare.tsx`; `03_DESIGN/02_UX_FLOWS.md` flow B | substantive | same |
| Release Gate | `apps/desktop/ui/src/Release.tsx`; `04_TECH/27_GATE_STATE_SEMANTICS.md` | substantive | — |
| Release Bundle | `apps/desktop/ui/src/Release.tsx`; `04_TECH/17` §5 | substantive | — |
| History | `apps/desktop/ui/src/History.tsx`; `P5_ONBOARDING_HISTORY_REPORT.md`; now `P5_HISTORY_DIAGNOSTICS_REPORT.md` §1 for the installed layer | substantive | — |
| CLI | `04_TECH/07_CLI_SPEC.md` | substantive | spec, not tutorial |
| `firmwaresight.toml` | `04_TECH/08_CONFIG_SPEC.md` | substantive | — |
| Status semantics | `04_TECH/27_GATE_STATE_SEMANTICS.md` + `ADR-0023` + the five state words on the Analyze panel | substantive | — |
| Support matrix | `04_TECH/20_PLATFORM_SUPPORT.md`, plus `P5_COMPATIBILITY_MATRIX.md` for the evidence classes | substantive | the Help screen points at it and says the installed package does not carry it |
| Known limitations | **`P5_VALIDATION/P5_KNOWN_LIMITATIONS.md` — created by F2** | substantive | this path was already named by `Help.tsx:50` with `shipped: false`, and until F2 **the file did not exist**. A stranger following the shipped UI's own pointer reached nothing. Fixed by writing the file, with no product code touched |
| Troubleshooting | **none** | **absent** | no canonical surface anywhere; `06_DELIVERY/00_ROADMAP.md` mentions it as future work. F2 deliberately did **not** create a stub to close a checklist row — §29 forbids exactly that. The recoverable facts that a troubleshooting page would carry are in `P5_MIGRATION_RECOVERY_REPORT.md` §6 and `P5_HISTORY_DIAGNOSTICS_REPORT.md` §3, and the gap is named here so the next stage inherits it honestly |
| Diagnostics | `04_TECH/21_OBSERVABILITY_DIAGNOSTICS.md` + the Diagnostics block in `Help.tsx`; installed layer in `P5_HISTORY_DIAGNOSTICS_REPORT.md` §2 | substantive | — |
| Privacy / local-first | `04_TECH/05_SECURITY_PRIVACY.md` + `ADR-0004` + the "Local first" section on Help | substantive | — |
| Upgrade / reinstall / uninstall | `P5_INSTALL_RECOVERY_REPORT.md` §8–§9; `04_TECH/17` "Windows install mode" | **substantive as of F2** | was thin: the behaviour was measured but no user-facing statement existed. §9 writes it and `04_TECH/17` points at it rather than duplicating it |
| Bundle verification | `scripts/verify_bundle_portability.py` + `P5_PACKAGING_REPORT.md` §5b; F2's relocated run in `10_bundle/relocated_verify_*.txt` | substantive | engineering-facing |
| Package verification | `scripts/verify_package_artifacts.py` + `P5_PACKAGING_REPORT.md` §4/§5c | substantive | engineering-facing |
| Signing / notarization readiness | `04_TECH/17` (Authenticode, notarization, "not claimed"); `P5_PACKAGING_REPORT.md` on `signtool verify` and `Get-AuthenticodeSignature` | substantive | states readiness, not completion — which is correct |

**New files F2 created: eight, none of them a stub** — exactly §28's list. `P5_KNOWN_LIMITATIONS.md` (a path
the shipped UI already named), `P5_MIGRATION_RECOVERY_REPORT.md`, `P5_HISTORY_DIAGNOSTICS_REPORT.md`,
`P5_DESKTOP_ACCEPTANCE_REPORT.md`, `P5_SECURITY_SUPPORTABILITY_REVIEW.md`, this file,
`P5_EXIT_CHECKLIST.md` and `P5_RELEASE_READINESS.md`.
Each carries measurements that existed nowhere before; none restates another document's content as its own
body — cross-references are used instead.

## 4. Retention and uninstall documentation (prompt §30)

Written into `P5_INSTALL_RECOVERY_REPORT.md` §9 in user words, with environment-variable paths and no
machine-specific ones, and summarised in `04_TECH/17`'s "Windows install mode" so the architecture document
does not go stale. It matches §8's measurements rather than intention, including the part no one would
volunteer: an empty install directory **may** remain after uninstall, because four observed runs produced
both outcomes and no rule has been worked out.

## 5. Method notes the next reader will want

- **Evidence root** `FirmwareSight-P5-F2-20261005T073141`, outside the repository, ~20 MB across 21 numbered
  directories. Screenshots of the native dialogs show a private project directory belonging to the owner;
  those files stay in the evidence root and are **not** referenced from the repository.
- **The F2 prompt itself had to be reconstructed.** The delivered file vanished from `Downloads` between the
  read at 14:16Z and the moment archiving was due. The archived copy was rebuilt from the session
  transcript's `Read` result, verified contiguous over lines 1–1,044 and matching the 19,040 bytes the
  directory listing recorded. `10_AUDIT/SOURCE_PROMPTS/README.md` says all of that instead of implying the
  bytes came from the original.
- **Three harness defects were found and owned**, and one of them had been silently producing "clicks that do
  nothing" for most of the round: the driver raised whatever window owned the target pixel, and a browser
  window covering the work area was swallowing installer clicks. Two wizard events earlier classified
  ENVIRONMENT are reclassified HARNESS. The detail is `P5_DESKTOP_ACCEPTANCE_REPORT.md` §6.

## 6. Addendum — what Commit F2R changed in the record (2026-10-06)

Sections 1–5 above are F2's and stay as F2 wrote them, including the finding that F2's prompt forbade it from
touching `apps/**` and the note about the reconstructed prompt file. What follows is the part of the stage
that happened after it.

**Why F2R exists.** F2 closed the §64 journey and, in the same report, raised three product findings it could
not fix. The Architect chose a narrow corrective between F2 and F3 rather than letting F3 inherit them, and
F2R's prompt scopes it to exactly those three findings, exactly one product path (`apps/desktop/ui/src/**`),
and exactly two heads.

**What landed.**

| Head | Content | Proof it carries |
| --- | --- | --- |
| `fb5f628` (**F2R1**) | the three fixes plus six new UI tests, the design record, the archived F2R prompt, `DIRECTORY_TREE.txt` and `SHA256SUMS` | run `37431977428` (#70) **10 of 10 on attempt 1**; 868 Rust / 225 UI read out of the runner's own logs; its Windows artifact `11397938806` was downloaded, installed, and used for the focused revalidation |
| **F2R2** (this head) | evidence, dispositions and governance only — `P5_F2R_UI_CORRECTIVE_REPORT.md` plus the addenda in `P5_KNOWN_LIMITATIONS.md` §8, `P5_EXIT_CHECKLIST.md` §4–§6, `P5_DESKTOP_ACCEPTANCE_REPORT.md` §9, `P5_CI_AUTHORITY.md`'s F2 and F2R1 rows, and the `.ai` entry documents | §43's count rule: **868 Rust / 225 UI in 8 files, identical to F2R1.** A docs-only head that moved either number would be a product change wearing a documentation diff |

**The three findings, in one line each.** F2R-01 (S2): `display: block` on the `<table>` plus
`overflow-wrap: anywhere` on the reason column made that column's minimum one character, so it absorbed the
whole deficit between the pane and the table's real minimum — fixed by giving a `.viewport` wrapper the scroll
axis, letting the table keep its intrinsic minimum, and bounding the prose at a `24ch` measure. F2R-02 (S3):
History's action was the **last** cell of a table whose rigid minimum measured 939 px against a 736 px pane,
so it sat 195 px past the right edge — fixed by making it the leading cell, the placement the Evidence table
already used. F2R-03 (S3): Release printed `row.basis` straight — fixed by one shared caption map that Compare
already owned and three surfaces now read.

**Method notes the next reader will want, in the same spirit as §5.**

- Evidence root `FirmwareSight-P5-F2R-20261005T221112`, outside the repository, 21 numbered directories. The
  owner's store was parked with digests before install and restored byte-exact afterwards
  (`OWNER_STORE_OPENED_BY_F2R = NO`); only the disposable live store's files were removed, and their final
  bytes were preserved first.
- The desktop driver was hardened **before** any installed action: `require_app()` refuses with `NO_PROCESS`
  / `NO_WINDOW` rather than falling back to some other application's window, and every click still passes the
  pixel-ownership guard that refused one real click during this round. Only documented Win32 entry points; no
  `WM_*` message ids, no CDP, no injected `click()`, no direct IPC, no SQL writes.
- Two §21/§22 runs were **superseded rather than deleted**, because three comment-only lines moved while they
  were running. The kept files say they back no claim.
- vitest emits colour escapes; a plain grep for its summary text matches nothing in the campaign log. The
  counts were taken with the escapes stripped, and the record states that.
- One transcription typo in `06_local_validation/S21_S22_FINAL.txt` ("225 files" for "8 files") was corrected
  against the log it quotes, in place, with the correction dated and the log line reproduced — the file itself
  is the evidence, and the edit is visible inside it.

## 7. Commit F3 — the complete lineage, and the closure (2026-10-06)

F3 is governance, evidence and indexing only: **zero** paths under `apps/`, `crates/`, `scripts/`, `fixtures/`,
`schemas/`, `golden/`, `migrations/` or `.github/`, and no `Cargo.toml`, `Cargo.lock`, `package.json`,
`pnpm-lock.yaml`, `tauri.conf.json`, `deny.toml`, `rust-toolchain.toml` or `assets/design-tokens.json` byte.
Because no product byte moves, §5 requires both test counts to stand still, and they do: **868 Rust / 225 UI
in 8 files**. §24 forbids installing anything in F3 and nothing was installed; §24 forbids touching the
owner's app-data store and it was not opened.

### The whole stage, head by head

Every red and every interrupted run keeps its row. The count of heads is not six, and the two paragraphs above
that say otherwise are F2's and F2R's own records rather than the stage's.

| Layer | Head(s) | Run / attempt | Result |
| --- | --- | --- | --- |
| P5 opening baseline | `d83175a` (post-G2 remediation closure, = `origin/main`, tree clean) | `37101619245` a1 | 7 of 7, success |
| Commit A — audit first, then migration `0005` | `4cc8d93`, `812b472` | `37125456689` a1, `37127791999` a1 | 7 of 7 each |
| version unification | `9e3b1de` | `37128593254` a1 | **6 of 7 — failure**, a pre-existing `compare.test.tsx` race it never touched |
| the race's test-only repair | `20b03e3` | `37129900728` a1 | 7 of 7 |
| Commit B — packaging, three CI package jobs | `0c031cd` | `37133706214` a1 | **7 of 10 — failure**: three package jobs printed a `SKIP` and `4/4 steps passed`, then went red on their own upload step |
| Commit B, repaired | `1055242` | `37138881977` a1 | 10 of 10 — first run in the repository to attach a built package on all three platforms |
| the `.app` indexed file by file | `53578e9` | `37143046338` a1 | 10 of 10 |
| read-back, then the first real install | `a674774`, `9ab089f`, `4719e1f` | `37145302229`, `37147434366`, `37147577288` a1 each | 10 of 10 each |
| Commit C — onboarding, Help/About, History | `e863d0c` | `37154946484` a1 | 10 of 10 (812 Rust / 200 UI) |
| Commit C read-back | `e070508` | `37156287999` a1 | 10 of 10 |
| §32 L22 draft + L23's sixth instance | `111fe32`, `2cdfced` | `37158606478`, `37159810291` a1 each | 10 of 10 each |
| Commit D — integrity_check, backup, Diagnostics, `ADR-0028` | `3400981` | `37200245520` a1 | **8 of 10 — failure**: a clock race inside a test this commit wrote, red on `macOS Core Smoke` and `Rust (ubuntu-latest)` |
| its test-only repair and read-backs | `bccea88`, `90aa69d`, `956e250`, `57a904e`, `bfbc1aa` | `37202016141`, `37202301591`, `37204847474`, `37212546755`, and `bfbc1aa`'s own | 10 of 10 each, attempt 1 |
| Commit E1 — the compatibility cohort + the `SHF_ALLOC` fix | `859648e` | **no run of its own** | one push of two commits starts one run at the tip; its bytes are verified by the tip run and by a clean detached worktree |
| Commit E2 — §48's fourteen dispositions | `59d85c3` | `37228929762` a1 | 10 of 10 |
| Commit E read-back | `6981625` | `37230689636` a1 | 10 of 10 |
| Commit E closure normalization | `80b47c4` | `37262147348` **(#67) a1 then a2** | **attempt 1: 9 of 10 — failure**, `Generated output drift` killed by Ubuntu rustup provisioning before it compiled anything; **attempt 2: 10 of 10** after one `gh run rerun --failed`. Two attempts, stated as two attempts |
| Commit F1 — `ADR-0029`, the index-blob baseline, L15's caption | `0bca373` | `37293381181` a1 | 10 of 10 (gate 16 → **17** steps, drift 7 → 8) |
| Commit F2 — the §64 installed journey | `d1dc61c`, `55fad63` | `d1dc61c` **no run**; `55fad63` = `37367382516` **(#69) a1, a2, a3** | **attempt 1: 3 jobs success, 7 jobs `cancelled` with 0 executed steps**; **attempt 2: 6 success, 4 `cancelled` with 0 steps**; **attempt 3: 10 of 10**. Zero-step cancellations are allocation, not product, and all three stay |
| Commit F2R1 — the three installed-UI fixes | `fb5f628` | `37431977428` **(#70) a1** | **10 of 10**, one run, no cancellation to explain; 868 Rust / 225 UI read out of the runner's own logs; its artifact `11397938806` is the one installed |
| Commit F2R2 — the installed read-back and the record | `31f10c7`, `a5ce7c4` | `31f10c7` **no run**; `a5ce7c4` = `37453402452` **(#71) a1** | **10 of 10**. `31f10c7` is the head that first carried two figures it had not yet measured; `a5ce7c4` corrects them at the head that lands them, which is why it exists |
| **Commit F3 — this head** | the head that lands this row | **not knowable from inside the commit** | `F3 remote CI = PENDING_EXTERNAL_EVIDENCE`. The SHA exists once the commit is written; its run exists once it is pushed. §19 forbids fabricating it here and §34 forbids a fourth round written only to record it |

**Four heads went red or were interrupted, and each keeps its row for a different reason.** `9e3b1de` shows a
green CI can say nothing about a race the local gate is the only thing that loses. `0c031cd` shows a gate
summary that could lie, and it is why `check.py` distinguishes `SKIP` from `PASS` and fails on a skip in CI.
`3400981` shows an assertion that was a race with a clock and passed on the host that wrote it. `80b47c4` and
F2's #69 show two kinds of non-product failure — a provisioning fault and a zero-step allocation cancellation —
and the rule that covers both: read the log before touching the rerun button, and never rewrite a run as
first-attempt green.

### What F3 wrote, and what it refused to write

Wrote: `P5 = PASS_COMPLETE`, `Productization = ENGINEERING_COMPLETE`, `active_task = NONE`, product
**MVP CANDIDATE** at baseline `0.6.0`, narrative **FirmwareSight Productized MVP Candidate**, and the §6
re-audit that had to pass first (`P5_EXIT_CHECKLIST.md` §7; the row-level record is
`EXIT_REAUDIT_FINAL.txt` in the F3 evidence root outside this repository). It also corrected one assertion of
its own repository's making: `P5_KNOWN_LIMITATIONS.md` was missing its L12 row while the exit checklist
asserted the list was complete, so F3 added the row and corrected the assertion rather than deleting the claim.

Refused: any tag, GitHub Release, publication, signature, notarization, updater or licence choice; V1, B1, G3,
RC or GA; `security clean`, WCAG compliance, any claim that performance was optimized or that a memory target was met, all-DPI or all-monitor coverage; a new
checksum semantic, a renamed Core enum, a moved wire value, migration `0006` or `analysis:2`; and a further
commit whose only content is F3's own run number.

`P5 = PASS_COMPLETE` means the MVP was productized to P5's engineering scope. It does not mean market fit,
real-user validation, beta or RC quality, GA readiness, a signed or notarized release, an automatic update
path, all-platform runtime validation, WCAG certification, all-DPI validation, a selected licence or a public
open-source release. `P5_FINAL_CLOSURE_REPORT.md` §14 states that boundary, and the paragraph below keeps the
stage's performance truth unchanged: the valid ELF at **519,179,252 bytes** with about **2,020,073 symbols**
ran warm in **~4.73–4.89 s**, its cold first read took **~68.7 s**, the peak observed working set was
**~1,428 MB**, and it never showed "Not Responding" and never crashed. Near-500-MiB UI is `MEASURED`;
first-use under 60 s is **not proved at this workload** and stays environment-sensitive; RSS is measured for
the tested workload only.
