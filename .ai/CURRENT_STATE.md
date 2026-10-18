---
title: "Current Project State"
doc_id: "FS-AI-002"
product: "FirmwareSight"
version: "0.6.0"
status: "CURRENT"
owner: "Engineering"
last_updated: "2026-10-15"
---

# Current State

These are only the facts that are true now. The file is overwritten, never appended. The full dated record up to 2026-10-14 is [archive/2026-10-14/CURRENT_STATE.md](archive/2026-10-14/CURRENT_STATE.md). Live SHAs and CI are in [NOW.md](NOW.md).

## Product

FirmwareSight is a local-first desktop workbench and CLI (`fwsight`) for embedded firmware releases, built around four verbs:

| Verb | What it does | Status |
|---|---|---|
| Analyze | ELF plus optional GNU ld MAP: sections, symbols, memory regions, top contributors, evidence | Shipped (P1, B2/E03) |
| Compare | Deterministic diff of persisted snapshots, with percentages and memory-delta explanations that carry a confidence level | Shipped (P2, B2/E05) |
| Release Gate | Five-state evaluation of configured project policy: PASS / REVIEW / BLOCK / UNKNOWN / N/A | Shipped (P3) |
| Bundle | Exports verified release evidence, including release-attached byte evidence (ADR-0030) | Shipped (P4, C1) |

The desktop app also has project readiness and release preflight, local History, Diagnostics, theme (System / Light / Dark), interface language (English / Chinese, including native dialog titles), and PNG, SVG and print-HTML visual report export for Analyze, Compare and Gate. M1 (on `main` since `6804ae7`) added drag-and-drop of an ELF and its MAP, per-region memory usage bars on Overview, and a numbered Release setup. M2 (on `main` since `04d800c`, `ADR-0033` BASELINE) added per-object and per-library Flash/RAM attribution from GNU ld MAP rows on Analyze, Compare and CLI text, and the latest comparison on Overview. The route after M2 is [06_DELIVERY/12](../06_DELIVERY/12_POST_M2_ROUTE.md).

Unknown or partial evidence stays explicit. It is never shown as zero or as proof.

## Formats

- ELF32 and ELF64, little and big endian: parsed. Verified memory-accounting cohorts are GCC + GNU ld and Clang + GNU ld.
- GNU ld `.map`: the only supported MAP grammar. Its input-section rows also give object and library attribution; portable JSON does not carry those rows (`ADR-0033` D7).
- Keil (armlink) and IAR (ILINK) output: UNVERIFIED as cohorts. armlink MAP: **supported** since ADR-0036, on one synthetic same-origin sample (Arm Compiler 6.24) cross-checked against armlink's totals and GNU readelf, and on two real Keil MDK projects (ADR-0036 A1/A2, `evidence/D4-REAL`). IAR MAP: NOT_SUPPORTED / FIXTURE_BLOCKED.
- `.bin` and Intel HEX: can be attached as release byte evidence. Content analysis is UNSUPPORTED.

Details: [04_TECH/03](../04_TECH/03_FORMAT_SUPPORT.md), [04_TECH/39](../04_TECH/39_ELF_TOOLCHAIN_INTEROPERABILITY.md).

## Platforms and packaging

- Windows x64: NSIS installer built in CI. A real install was proved during P5 on an earlier build. Native interactive acceptance of the current build is NOT_VERIFIED; see the open risks in [NOW.md](NOW.md).
- macOS and Ubuntu: CI_BUILD_ONLY.

Details: [04_TECH/20](../04_TECH/20_PLATFORM_SUPPORT.md).

## Technical baseline (changes need an ADR)

Rust Core (`firmwaresight-core`, headless and synchronous), Tauri 2, React, rusqlite (bundled), Rust toolchain pinned to `1.98.1`. No server, cloud, auth, telemetry, HTTP client or GPU. See [AGENTS.md](../AGENTS.md) §2–§7 and [04_TECH/09](../04_TECH/09_TECH_DECISION_MATRIX.md).

## Stage record

G0, P0, G1, P1, P2, P3, P4 PASS. G2 PASS: Product MVP engineering complete, state MVP CANDIDATE. P5 productization PASS_COMPLETE. U1 UI productization closed. Since then, C1, B1, B2 and NEXT-xx rounds have each been closed individually. V0 and V1 external validation have 0 eligible sessions and are non-blocking. Evidence for each stage lives in its `*_VALIDATION/` folder and in [10_AUDIT/](../10_AUDIT/).

## Business and legal boundaries

- License: **Apache-2.0** (ADR-0035, Owner decision 2026-10-18). `LICENSE`, `NOTICE`, `THIRD_PARTY_LICENSES.md`, `CONTRIBUTING.md`, `SECURITY.md` at the root.
- Not authorized: public release, Developer Preview distribution, signing or notarization, updater, network features.
