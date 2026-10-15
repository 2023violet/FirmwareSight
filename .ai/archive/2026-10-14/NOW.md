---
title: "Current Project Status"
doc_id: "FS-AI-NOW"
product: "FirmwareSight"
version: "0.6.0"
status: "CURRENT_INDEX"
owner: "Engineering"
last_updated: "2026-10-14"
---

# FirmwareSight Now

## Current pointer — 2026-10-14 / NEXT-47

- **Observed integrated code SHA:** `76b5c2c006f1ca0850af86da6d1e4b957227a025`, the unchanged NEXT-46 candidate, integrated by one ordinary FF push under the explicit NEXT-47 authorization. [Actions #113](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036), attempt 1, has all 10 Jobs and 10 Check Runs successful on that exact code SHA; actual cargo-deny and baseline/drift logs were independently verified.
- **Independent scoped decision:** `CODE_INTEGRATION_GATE = PASS`; `NATIVE_USER_ACCEPTANCE_GATE = NOT_VERIFIED / BLOCKED_BY_ENV`. NEXT-44 AC24/AC26 and NEXT-45 AC20 remain MUST / NOT_VERIFIED with Owner-accepted deferred native QA. Close them before the earliest external Developer Preview distribution, Private Beta or RC. No native/installed acceptance, vendor formal compatibility or public product release is inferred.
- **Repository evidence upload:** the Owner subsequently requested uploading the delivery to main. [NEXT47_VALIDATION](../NEXT47_VALIDATION/README.md) contains a public derivative with host-path redaction and original/published digest mapping. Local originals, original historical judgments and the protected main's nine staged files remain unchanged. The upload is a later documentation/evidence commit; run #113 certifies the code SHA above, not this new documentation commit. Read Git/Actions for the upload's current SHA and CI state.
- **Compatibility:** Keil/IAR compiler cohorts remain UNVERIFIED; ArmLink/IAR MAP remain NOT_SUPPORTED / FIXTURE_BLOCKED. Clang class/endian fixtures establish basic ELF metadata only.

## Historical pointer — 2026-10-14 / NEXT-46 (preserved)

- **Observed remote baseline:** `4ac1e4f709396fac7a37a1d92b8095a070b3effa`, re-read from GitHub and `ls-remote`. [Actions #112](https://github.com/2023violet/FirmwareSight/actions/runs/38535378887) has all 10 Jobs successful on that SHA. Remote evidence proves engineering CI, not native Save-dialog or installed desktop acceptance.
- **Owner adjudication:** `NEXT45 = ENGINEERING_DELIVERED / REMOTE_CI_PROVEN / REVIEW_EXCEPTIONS_OPEN`. The earlier reviewer `ACCEPT_WITH_LIMITS` and prior conditional-push nonconformance remain historical audit facts. Correct nonduplicated Windows NEXT-45 workspace count: 1,154 passed / 0 failed / 2 opt-in ignored / 54 result lines; the old commit message is not amended.
- **Active authorized task:** NEXT-46 export output-boundary closure, SVG geometry regression, authoritative NEXT-44/45 acceptance read-back and evidence-based ELF/MAP interoperability foundation. Explicit Owner dispatch authorizes isolated local implementation and conditional single FF push only after fresh independent exact-SHA full mandatory acceptance. Candidate status and verdict are external delivery evidence; this navigation page grants no acceptance.
- **Code baseline additions since the older snapshot:** DOC-1 navigation integration, project readiness + privacy hardening, bounded intake/100K regression and PNG/SVG/print HTML report export. [Concise evidence ledger](EVIDENCE_LEDGER.md) separates code, CI, Owner verdict and unknown historical evidence.
- **Compatibility boundary:** [ELF interoperability matrix](../04_TECH/39_ELF_TOOLCHAIN_INTEROPERABILITY.md). Keil/IAR compiler cohorts remain UNVERIFIED; ArmLink/IAR MAP remain NOT_SUPPORTED / FIXTURE_BLOCKED. New Clang class/endian fixtures prove basic metadata only.
- **Protected state:** Original main remains at `e34f9e6` with the same nine staged files; it is intentionally older than remote. No reconciliation or push from that tree is authorized. Owner appdata, private firmware and V1 archives are outside this batch.
- **Unchanged product limits:** Native dialog/keyboard/theme runtime evidence remains NOT_VERIFIED unless independently measured. V1 external sessions remain unclaimed; product license stays Proprietary. No public release, signing, updater or native PDF claim.

## Historical navigation snapshot — 2026-10-13 (preserved)

As of 2026-10-13. This is the current status index. Older records retain the status known when written; this page does not rewrite history.

## Baseline

- **Last remote-verified baseline:** `origin/main` was `515a55a1e8c9a4d2b672dbf63b440cab296096a0`; [commit](https://github.com/2023violet/FirmwareSight/commit/515a55a1e8c9a4d2b672dbf63b440cab296096a0). Read the live local branch and HEAD from Git; this baseline does not identify later local work.
- **CI:** [GitHub Actions run #108](https://github.com/2023violet/FirmwareSight/actions/runs/38407028965) ran against that exact SHA; all 10 jobs succeeded. This establishes CI results, not installed desktop acceptance.
- **UNKNOWN:** Whether a newer GitHub status exists after run #108. Re-read `git rev-parse HEAD`, `git rev-parse origin/main`, and Actions before making a new claim.

## Accepted engineering slices

| Area | Current status | Evidence and boundary |
|---|---|---|
| B2/E03 Memory Region Core + CLI + Analyze Desktop | `ACCEPTED / REMOTE_CI_PROVEN / CLOSED` | **OWNER_REVIEW:** Owner NEXT-21/NEXT-22 decision. **CODE:** `crates/firmwaresight-core/src/domain/memory.rs`, `apps/desktop/src-tauri/src/service.rs`, `apps/desktop/src-tauri/src/ipc.rs`, `apps/desktop/ui/src/Analyze.tsx`. **TEST:** Core memory unit tests, `apps/desktop/src-tauri/tests/analyze_details.rs`, and `apps/desktop/ui/src/intake.test.tsx`. **CI:** run #107 above. Installed/visual desktop acceptance: `NOT_VERIFIED` (NEXT-25, `BLOCKED_BY_ENV`). |
| B2/E05 Compare percentage and explanation | `ACCEPTED / REMOTE_CI_PROVEN / CLOSED` | **OWNER_REVIEW:** Owner NEXT-24 decision. **CODE:** `apps/desktop/src-tauri/src/compare.rs`, `apps/desktop/ui/src/Compare.tsx`. **TEST:** percentage cases in `compare.rs` and `apps/desktop/ui/src/compare.test.tsx`. **CI:** run #107 above. Actual toolchain equivalence and growth without supported attribution remain `UNKNOWN` where evidence cannot prove them. |

## Security and product boundaries

- **D-N11-1 — OWNER_REVIEW:** `ACCEPTED / REMOTE_CI_PROVEN / CLOSED_WITH_DEFINED_THREAT_MODEL`. Scope: Release Notes reads confined to the selected project root using the approved capability-based access path. **CODE:** `crates/firmwaresight-project/src/` and `crates/firmwaresight-project/tests/release_notes_security.rs`. **TEST / CI:** run #107 on Ubuntu and macOS executed POSIX symlink cases, including rejection of an outside-project symlink. Windows Junction traversal rejection is covered by the accepted NEXT-27 regression. Windows file-symlink creation/test remains `NOT_VERIFIED` (host error 1314 during link creation). Hard-link unique provenance and concurrent in-place mutation of an already-open file are outside the guarantee. Unrelated release-attachment path APIs are out of scope.
- **Installed desktop QA — OWNER_REVIEW:** NEXT-25 is `NOT_VERIFIED / BLOCKED_BY_ENV`. Package CI success does not mean the installer was installed or the UI was interactively tested.
- **License — CODE / OWNER_REVIEW:** workspace `Cargo.toml` declares `license = "Proprietary"`; no root `LICENSE` has been chosen or added. Open-source publication and B3 Developer Preview are `NOT_AUTHORIZED`. Signing and notarization are separate decisions.
- **V1 cohort — HISTORICAL_RECORD:** artifact `11573661113` had expected GitHub expiry `2026-10-22`. Any privacy-preserving archive requires separate Owner authorization. No archival action is authorized here.

## Governance and next steps

- **CODE / HISTORICAL_RECORD:** `.ai/CURRENT_STATE.md`, `.ai/ACTIVE_TASK.md`, `README.md`, and `BASELINE.yaml` retain dated B2-U1R2 / D-N11-1-open language. This is historical governance drift, not a reversal of later Owner review. For current status, this dated page takes precedence; for historical status, retain the dated source record.
- **OWNER_REVIEW:** DOC-0 was accepted with scanner qualifications. NEXT-34 governs isolated local integration of DOC-1 Wave 1A on the security-fixed baseline. Read Git for its candidate state; NEXT-35 independent review remains required. This page makes no remote CI or acceptance claim for the DOC-1 candidate. No Wave 1B, Wave 2, product changes, license decisions, releases, or pushes are authorized.
- **UNKNOWN:** No product implementation task is active under this navigation change. Do not infer authorization for a feature from closed engineering slices.

## Evidence vocabulary

`CODE` means checked-in implementation or declaration; `TEST` means an executable regression; `CI` means a named remote job/run; `OWNER_REVIEW` means an explicit Owner decision; `HISTORICAL_RECORD` means a dated prior statement; `UNKNOWN` means available evidence does not establish the claim.
