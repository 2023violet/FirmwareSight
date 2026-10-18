---
title: "Decisions Index"
doc_id: "FS-AI-003"
product: "FirmwareSight"
version: "0.6.0"
status: "CURRENT"
owner: "Engineering"
last_updated: "2026-10-16"
---

# Decisions Index

Durable decisions live in ADRs. This page indexes them and lists the standing Owner rules that apply to every task. The dated decision log up to 2026-10-14 is [archive/2026-10-14/DECISIONS.md](archive/2026-10-14/DECISIONS.md). Read it only when a task needs the history behind a decision.

## ADRs (all BASELINE)

| ADR | Decision |
|---|---|
| [0001](../09_ADR/ADR-0001-product-name.md) | Product name |
| [0002](../09_ADR/ADR-0002-desktop-stack.md) | Desktop stack: Tauri 2 |
| [0003](../09_ADR/ADR-0003-rust-core.md) | Rust Core and shared CLI |
| [0004](../09_ADR/ADR-0004-local-first.md) | Local-first |
| [0005](../09_ADR/ADR-0005-no-ai-trusted-core.md) | No AI in the trusted core |
| [0006](../09_ADR/ADR-0006-mvp-format-scope.md) | MVP format scope |
| [0007](../09_ADR/ADR-0007-core-first-adapter-driven.md) | Core-first, adapter-driven architecture |
| [0008](../09_ADR/ADR-0008-async-boundary.md) | Async boundary |
| [0009](../09_ADR/ADR-0009-storage-rusqlite.md) | SQLite via rusqlite |
| [0010](../09_ADR/ADR-0010-artifact-parser.md) | Artifact parsing stack |
| [0011](../09_ADR/ADR-0011-frontend-baseline.md) | Frontend baseline: React |
| [0012](../09_ADR/ADR-0012-tracing.md) | Structured tracing |
| [0013](../09_ADR/ADR-0013-network-deferred.md) | Network client deferred |
| [0014](../09_ADR/ADR-0014-gpu-deferred.md) | GPU deferred; native island |
| [0015](../09_ADR/ADR-0015-release-update-lifecycle.md) | Release and update lifecycle |
| [0016](../09_ADR/ADR-0016-toolchain-pinning.md) | Toolchain pinning |
| [0017](../09_ADR/ADR-0017-git-cli-provenance.md) | Git CLI provenance adapter |
| [0018](../09_ADR/ADR-0018-ui-baseline-tokens-governance.md) | UI baseline and design-token governance |
| [0019](../09_ADR/ADR-0019-typed-ipc-bindings.md) | Typed IPC bindings |
| [0020](../09_ADR/ADR-0020-validation-sequence.md) | Validation sequence |
| [0021](../09_ADR/ADR-0021-memory-accounting.md) | Memory accounting |
| [0022](../09_ADR/ADR-0022-portable-schema-strictness.md) | Portable schema strictness |
| [0023](../09_ADR/ADR-0023-gate-five-state-semantics.md) | Gate five-state semantics |
| [0024](../09_ADR/ADR-0024-post-mvp-candidate-governance.md) | Post-MVP candidate governance |
| [0025](../09_ADR/ADR-0025-conditional-pre-g1-analyze-implementation.md) | Conditional pre-G1 Analyze implementation |
| [0026](../09_ADR/ADR-0026-open-source-mvp-first-delivery.md) | Open-source MVP-first delivery; G1 on P0 basis (license then still Proprietary; superseded by ADR-0035, Apache-2.0) |
| [0027](../09_ADR/ADR-0027-project-policy-and-provenance-adapter.md) | Project policy and provenance adapter |
| [0028](../09_ADR/ADR-0028-release-identity-bytes-as-evidence.md) | Release identity: the bytes are the evidence |
| [0029](../09_ADR/ADR-0029-repository-baseline-checksums-use-git-index-blobs.md) | `SHA256SUMS` uses Git index blobs |
| [0030](../09_ADR/ADR-0030-release-attached-byte-evidence.md) | Release-attached byte evidence (BIN/HEX) |
| [0031](../09_ADR/ADR-0031-developer-preview-theme.md) | Developer Preview theme: System / Light / Dark |
| [0032](../09_ADR/ADR-0032-desktop-ui-language.md) | Desktop UI language |
| [0033](../09_ADR/ADR-0033-object-library-attribution.md) | Object and library attribution from the GNU ld MAP (BASELINE 2026-10-15) |
| [0034](../09_ADR/ADR-0034-instrument-ledger-design-language.md) | Instrument Ledger: one design language for the app and every export; tokens 0.4.0 (BASELINE 2026-10-16); A1 tokens 0.4.1 layer scale and dialog width (2026-10-17) |
| [0035](../09_ADR/ADR-0035-apache-2-open-source-license.md) | FirmwareSight is open source under Apache-2.0 (BASELINE 2026-10-18) |
| [0036](../09_ADR/ADR-0036-armlink-map-adapter.md) | armlink (Keil MDK / Arm Compiler) MAP adapter (BASELINE 2026-10-18) |

## Owner decisions 2026-10-16: R-series redesign

The Owner redirected the next round from closeout to product building with a complete UI/UX redesign of the desktop app and every export (HTML, SVG, PNG, Markdown, CLI text). The Owner's B1 "Quiet Canvas" package is reference only. Plan: [03_DESIGN/09](../03_DESIGN/09_R_REDESIGN_SPEC.md); phases R0–R7 in ADR-0034 D7.

| # | Decision |
|---|---|
| RD1 | The redesign overrides the "large UI reworks" exclusion in [06_DELIVERY/12](../06_DELIVERY/12_POST_M2_ROUTE.md) §4 for the R series only. |
| RD2 | Targeted token bump to v0.4.0 under ADR-0034. Palette, spacing, radius, motion and the no-shadow rule do not move. |
| RD3 | Spec and prototype first (R0). R1–R7 start only after the Owner accepts the R0 prototype, each named in ACTIVE_TASK. |
| RD4 | D3 (History baseline pin, compare from History) folds into R5. This is the written Owner exception to ADR-0024's "V1/user signal" entry condition that route §2 asks for. |
| RD5 | Light `color.diff.removed` becomes `#C7372C` (4.56:1 on `#FBECEA`); ADR-0031 D6's contrast exemption is retired and the goldens that print it are regenerated on purpose in R1. |
| RD6 | A human-readable Markdown report (Analyze, Compare, Gate) is added in R6 as candidate E1-B by Owner exception. It is a presentation format, not an interchange schema; it is not in the release bundle. Whether it may list per-object rows is settled in R6 against ADR-0033 D7 / ADR-0022 before that code lands. |
| RD7 | 2026-10-16: the Owner accepted the R0 prototype's visual direction; ADR-0034 is BASELINE and R1 (foundation) is authorized ([06_DELIVERY/15](../06_DELIVERY/15_R1_FOUNDATION.md)). |
| RD8 | R7 (2026-10-17, under the Owner's standing goal to carry the plan through without interim confirmation): tokens 0.4.1 is additive (ADR-0034 A1: `layout.dialog`, `layer.*`); the scrim stays transparent. Human-only checks (window-minimum drag, NVDA/Narrator, 125%/150% DPI) stay NOT_RUN and are listed in [06_DELIVERY/21](../06_DELIVERY/21_R7_QUALITY.md) §3. |

D2C's open items (R4, human native QA, Pilot approval, V1 re-freeze authorization, ArmClang samples) stay open; the redesign does not close them.

(RD8 was numbered RD6 when it was written in R7, colliding with the Markdown decision above; renumbered in D2C-R, wording unchanged.)

## Owner decisions 2026-10-17: R-series acceptance and D2C-R

The Owner accepted R0–R7 as delivered ([06_DELIVERY/22](../06_DELIVERY/22_R_SERIES_FINAL_REPORT.md)) and authorized the cleanup of the merged R-series branches (done with `git branch -d` only). NVDA/Narrator listening and real system DPI stay **to be verified by a person**; they do not block the R-series close, and no external release gate is claimed on the strength of R7. The Owner then started **D2C-R** ([06_DELIVERY/23](../06_DELIVERY/23_D2C_R.md)).

| # | Decision |
|---|---|
| DR1 | **Route decision R4 (M2 review S2.1) is decided conservatively.** Builds stored before migration 0007 do not gain Object/Library rows when re-imported. No silent backfill: stored history stays as it was observed. If re-analysis of old builds is wanted later, it is designed as an explicit, traceable update with its own task. Storage behaviour does not change in D2C-R. |
| DR2 | The Pilot direction is approved **in principle**; D2C-R prepares admission only. Nothing is distributed and nobody is recruited until the entry conditions in `PILOT_VALIDATION/PILOT_PROTOCOL.md` §3 hold (final Owner approval, the Pilot minimum of `evidence/D2C-R/HUMAN_QA_PACK.md`). |
| DR3 | The route D2C → D4/D5 → D6 stands, updated for what R4–R7 already shipped (the pinned baseline, D3, and the Markdown report). |
| DR4 | Not authorized in D2C-R: public release, external distribution of test installers, system-setting changes, unreviewed product changes. Integration to `main` keeps Standing Owner rule 1. |

## Standing Owner rules

1. **Push:** push only on explicit Owner authorization, as one ordinary fast-forward of the exact reviewed SHA. Never force-push or rewrite history.
2. **Native QA deferral (NEXT-47):** engineering integration may proceed while native desktop acceptance is NOT_VERIFIED, but only with a registered risk. Deferred items are never reported as PASS.
3. **Compatibility claims:** no Keil or IAR support claim without real vendor samples (NEXT-46).
4. **Evidence in the repository (2026-10-14):** commit a short summary, the AC matrix and a hashed ZIP. Do not commit expanded raw logs. Strip local paths before publishing.
5. **Worktree hygiene (2026-10-14):** delete review worktrees once their verdict is signed and evidence is packed. Archive any non-Cargo content first. Never write scratch files into `target/`. No global `CARGO_TARGET_DIR`, because `scripts/verify_package_artifacts.py` reads `target/release/bundle`.
6. **Protected checkout:** the Owner's original `main` checkout and its nine staged files stay untouched without an explicit decision.
7. **Post-M2 route (2026-10-15):** delivery batches after M2 are named D2C, D3–D6, never M3–M6, because `V1_VALIDATION/V1_METRICS.md` owns M1–M6. The sequence and entry conditions are in [06_DELIVERY/12](../06_DELIVERY/12_POST_M2_ROUTE.md). Formal V1 is to be re-frozen to an M2-era build (option b), in its own authorized round. M2 review S2.1 (backfill of pre-0007 builds) is open.

## Owner decisions 2026-10-18: open source and release direction

| # | Decision |
|---|---|
| OS1 | **Licence: Apache-2.0** ([ADR-0035](../09_ADR/ADR-0035-apache-2-open-source-license.md)). Closes L13. |
| OS2 | Human testing before release is not required: automated native QA driven with real Windows input stands in for it, and public users test after release. Checks automation cannot do (screen-reader listening, system DPI) are disclosed as untested, not claimed. |
| OS3 | The route continues without lengthy deliberation: open-source readiness, automated QA, then features worth adding (first: the armlink MAP adapter, D4). Release binaries may ship unsigned with the disclosure in `SECURITY.md`. Making a release public remains one explicit confirmation per release. |

