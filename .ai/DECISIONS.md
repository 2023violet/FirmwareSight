---
title: "Decisions Index"
doc_id: "FS-AI-003"
product: "FirmwareSight"
version: "0.6.0"
status: "CURRENT"
owner: "Engineering"
last_updated: "2026-10-15"
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
| [0026](../09_ADR/ADR-0026-open-source-mvp-first-delivery.md) | Open-source MVP-first delivery; G1 on P0 basis (license itself still Proprietary) |
| [0027](../09_ADR/ADR-0027-project-policy-and-provenance-adapter.md) | Project policy and provenance adapter |
| [0028](../09_ADR/ADR-0028-release-identity-bytes-as-evidence.md) | Release identity: the bytes are the evidence |
| [0029](../09_ADR/ADR-0029-repository-baseline-checksums-use-git-index-blobs.md) | `SHA256SUMS` uses Git index blobs |
| [0030](../09_ADR/ADR-0030-release-attached-byte-evidence.md) | Release-attached byte evidence (BIN/HEX) |
| [0031](../09_ADR/ADR-0031-developer-preview-theme.md) | Developer Preview theme: System / Light / Dark |
| [0032](../09_ADR/ADR-0032-desktop-ui-language.md) | Desktop UI language |
| [0033](../09_ADR/ADR-0033-object-library-attribution.md) | Object and library attribution from the GNU ld MAP (BASELINE 2026-10-15) |

## Standing Owner rules

1. **Push:** push only on explicit Owner authorization, as one ordinary fast-forward of the exact reviewed SHA. Never force-push or rewrite history.
2. **Native QA deferral (NEXT-47):** engineering integration may proceed while native desktop acceptance is NOT_VERIFIED, but only with a registered risk. Deferred items are never reported as PASS.
3. **Compatibility claims:** no Keil or IAR support claim without real vendor samples (NEXT-46).
4. **Evidence in the repository (2026-10-14):** commit a short summary, the AC matrix and a hashed ZIP. Do not commit expanded raw logs. Strip local paths before publishing.
5. **Worktree hygiene (2026-10-14):** delete review worktrees once their verdict is signed and evidence is packed. Archive any non-Cargo content first. Never write scratch files into `target/`. No global `CARGO_TARGET_DIR`, because `scripts/verify_package_artifacts.py` reads `target/release/bundle`.
6. **Protected checkout:** the Owner's original `main` checkout and its nine staged files stay untouched without an explicit decision.
7. **Post-M2 route (2026-10-15):** delivery batches after M2 are named D2C, D3–D6, never M3–M6, because `V1_VALIDATION/V1_METRICS.md` owns M1–M6. The sequence and entry conditions are in [06_DELIVERY/12](../06_DELIVERY/12_POST_M2_ROUTE.md). Formal V1 is to be re-frozen to an M2-era build (option b), in its own authorized round. M2 review S2.1 (backfill of pre-0007 builds) is open.
