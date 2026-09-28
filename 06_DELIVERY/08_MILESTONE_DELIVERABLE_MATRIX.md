---
title: "Milestone and Deliverable Matrix"
doc_id: "FS-DEL-009"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Delivery"
last_updated: "2026-09-27"
---

# Milestone and Deliverable Matrix

| Stage | Purpose | Required deliverables | Exit evidence |
|---|---|---|---|
| G0 | Problem baseline | product/market/architecture baseline | PASS |
| V0 | workflow desirability | clickable 7-screen flow, moderated sessions, misunderstanding log, payment questions | V0 report |
| P0 | feasibility | real ELF slice, parse→normalize→snapshot→CLI→minimal Tauri, benchmarks | P0 technical report |
| G1 | allow Product MVP build | V0 PASS + P0 PASS | gate record |
| P1-A0 | bounded pre-G1 Analyze slice (ADR-0025; not a gate, not P1 completion) | real artifact intake through a native dialog, optional GNU ld MAP, reuse of the validated Analyze summary, opaque selection id, no full path in IPC/UI | P1_A0 execution report + exit checklist + real-window smoke + Rust/UI tests |
| P1 | Analyze | Intake/Analyze implementation + fixtures + evidence UI | acceptance/golden tests |
| P2 | Compare | build diff, contributors, evidence-aware change UI | deterministic diff tests |
| P3 | Gate | 5-state rules, unknown policy, review audit | rule matrix + contract tests |
| P4 | Bundle | manifest/gate/reviews/report/hash outputs | portable bundle verification |
| G2 | MVP candidate | complete credible workflow | QA + UX gate |
| V1 | own-artifact validation | real users/artifacts, actual n/N metrics, price/pilot signal | validation report |
| P5 | productization | packaging, onboarding, reliability, supportability | release readiness |
| B1 | private beta | pilot cohort | beta evidence |
| RC1 | release candidate | signed release candidate | RC checklist |
| GA1 | launch | GA package/docs/support | GA gate |
| E1/E2/E3/GX | post-MVP candidate horizons | only items separately authorized after V1 | candidate-specific evidence |

## Important

E1/E2/E3/GX are **not** extra mandatory stages in the current MVP lifecycle; they are namespaces for future candidate planning and may be skipped entirely.
