---
title: "P5 Execution Report"
doc_id: "FS-P5-EXECUTION"
product: "FirmwareSight"
version: "0.6.0"
status: "IN_PROGRESS"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-05"
---

# P5 — execution report

P5 ran in six heads. This is the round's own record of what each did, what was measured, and what was
deliberately not done. It is written by Commit F2 and therefore reports F2 completely and F3 not at all.

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
