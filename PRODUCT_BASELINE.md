---
title: "FirmwareSight Product Baseline v0.5.0"
doc_id: "FS-PRODUCT-BASELINE"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Product / Architecture"
last_updated: "2026-09-28"
---

# FirmwareSight Product Baseline v0.5.0

> **Product scope: v0.5.0 — unchanged.** The *project* baseline is now **v0.6.0**, and that version
> records the P0 Technical Vertical Slice closing `PASS`, not a product decision. See §12.
> Everything below is the v0.5.0 product scope that v0.6.0 inherits verbatim.

## 1. Product position

FirmwareSight is a **local-first Embedded Firmware Release Workbench**.

Tagline:

> **Know exactly what ships.**

It is not a generic ELF viewer, IDE, debugger, flasher, cloud SBOM service, OTA fleet platform, reverse-engineering suite, or an AI release judge.

The product exists to turn scattered build/release facts into an inspectable release evidence chain:

```text
Analyze → Compare → Gate → Release
```

The durable product advantage is not “more charts” or “more ports”; it is **evidence depth, repeatability, explicit uncertainty and zero workflow migration**.

## 2. Target users

### Primary
1. Firmware Engineer — verify memory/build identity/change before shipping.
2. Firmware Lead — review reproducible diffs and policy evidence rather than verbal confidence.
3. QA / Compliance-oriented reviewer — consume structured, self-contained evidence rather than screenshots + spreadsheets.
4. Small hardware founder/team — gain disciplined release assurance without a dedicated DevOps/release engineering team.

### MVP supported cohort
The market persona is broader than the first compatibility promise.

MVP starts with fixture-proven:
- ELF 32/64;
- GNU ld MAP;
- BIN/Intel HEX basic metadata/hash;
- optional Git provenance.

Keil/ArmClang/IAR remain important discovery cohorts, but become `Supported` only after real fixture + adapter + regression coverage.

## 3. Core functional modules

### A. Project & Artifact Intake
- immutable artifact identity;
- SHA-256;
- format detection;
- optional MAP/Git;
- capability banner;
- typed diagnostics.

### B. Analyze
- architecture / endian / entry point;
- sections and symbols;
- nonvolatile/load image footprint;
- runtime RAM footprint;
- top contributors;
- evidence classifications.

### C. Compare
- build A / build B;
- section/symbol deltas;
- Added / Removed / Changed;
- biggest contributors;
- identity/provenance changes.

### D. Release Gate
Canonical finding states:

`PASS / REVIEW / BLOCK / UNKNOWN / N/A`

Unknown remains a fact state; policy may map it to effective `REVIEW` or `BLOCK` without erasing uncertainty.

### E. Release Bundle & History
- versioned portable manifest;
- gate results;
- accepted review audit;
- HTML/JSON evidence output;
- hashes;
- local history.

### F. CLI
`fwsight` is an MVP product surface:
- stable JSON;
- stable exit codes;
- same Rust Core as Desktop.

Hosted/team CI packaging is a later product layer, not proof that the CLI itself is “post-MVP”.

## 4. UI / information model

Instrument-grade clarity:
- evidence before verdict;
- tables first;
- ranked bars as judgment accelerators;
- charts never replace source rows;
- Unknown is first-class;
- state = icon + label, never color alone;
- one primary focus per screen;
- no marketing dashboard styling.

The accepted seven-screen reference set lives in `assets/ui-mockups/`.

## 5. Architecture

```text
React/Tauri Desktop        fwsight CLI
          \                 /
           Application/use cases
                   ↓
          Pure Rust domain Core
                   ↓ ports
     Artifact | Storage | Provenance | Report
```

Hard constraints:
- Core does not depend on Tauri, React, SQLite or Tokio types;
- UI does not reimplement parser/diff/gate logic;
- async is an execution boundary concern, not a domain property;
- SQLite is local storage/index, not the portable source of truth;
- portable contracts are versioned JSON/HTML/Bundle artifacts.

## 6. Current product scope

v0.5.0 does **not** expand the current MVP.

Minimum product remains:
- import/analyze;
- compare;
- build identity;
- deterministic release gate;
- release bundle;
- Desktop minimum credible UI;
- shared foundation CLI.

Basic visualization necessary to read these facts is part of the presentation of existing MVP facts.

Advanced visualization/export/integration/evidence extensions discovered in the expert round live in the **Post-MVP Candidate Registry**, not the current MVP.

## 7. Current validation sequence

```text
G0 Problem Baseline              PASS

V0 Workflow Prototype Validation ┐
                                 ├─ both required → G1
P0 Technical Vertical Slice      ┘

G1
 ↓
P1 Analyzer
P2 Compare
P3 Release Gate
P4 Release Bundle
 ↓
G2 MVP Candidate
 ↓
V1 Own-artifact External Validation
 ↓
P5 Productization
B1 Private Beta
RC1
GA1
```

V0 and P0 require explicit authorization. `ACTIVE_TASK` remains NONE in this baseline.

## 8. Post-MVP horizon namespace

Exploration reports written against older baselines used labels such as “P1/P2/P4” for future ideas. In v0.5.0 those labels **must not be imported literally**, because P1–P4 are already canonical MVP implementation phases.

Post-MVP candidates use:

- **E1 — Low-cost workflow depth**
- **E2 — Evidence depth**
- **E3 — Conditional advanced experience**
- **GX — Growth / ecosystem integrations**

A candidate horizon is not a commitment or implementation authorization.

## 9. Commercial posture

The most important unvalidated assumption remains willingness to pay.

Validation should test payment intent around:
- release risk reduction;
- repeatability;
- team review;
- evidence retention;
- CI/policy workflow;
- compliance preparation.

Do not monetize “a pretty treemap” as the value thesis.

## 10. Compliance boundary

FirmwareSight may provide:
- observed/derived/declared/unknown evidence;
- component/provenance information;
- release records;
- future SBOM evidence/export.

It must not claim:
- legal CRA compliance;
- certification;
- no vulnerabilities;
- zero risk;
- complete SBOM coverage unless the exact scoped completeness definition is proven.

## 11. v0.5.0 thesis

The expert round materially strengthens one conclusion:

> The most promising expansion direction is **evidence completeness + workflow continuity**, not adding a fifth product verb.

The candidate roadmap therefore deepens Analyze / Compare / Gate / Release rather than turning FirmwareSight into an IDE, cloud portal, OTA service or generalized dashboard.

## 12. v0.6.0 — technical foundation, product scope unchanged

Added 2026-09-28 when P0 closed `PASS`. Read together with §6 and §11; it does not amend them.

`v0.6.0` is the first project baseline that contains production source. What it asserts is that the
**foundation** under this product scope is built and verified: the Core-first architecture of §5 (four
Phase-0 library crates, a headless synchronous Core, the CLI and Desktop surfaces of §3.F sharing it),
the intake facts of §3.A, the analysis facts of §3.B on real ARM ELF and GNU ld MAP fixtures, a local
SQLite index per §5's storage constraint, and a typed IPC boundary. The evidence for each is in
`P0_TECHNICAL_VALIDATION/`.

What it does **not** assert, and no wording here may upgrade it:

- §3.C Compare, §3.D the deterministic Release Gate verdict surface, and §3.E Release Bundle & History
  remain **unimplemented product scope**, ahead of this baseline in `06_DELIVERY/00_ROADMAP.md`;
- optional Git provenance is not part of what P0 validated;
- the MVP cohort promise in §2 stays fixture-proven only — Keil/ArmClang/IAR are still not `Supported`;
- the commercial posture of §9 is untouched: willingness to pay is still the unvalidated assumption, and
  V0's `0 / 8` external sessions are the reason;
- §10's compliance boundary is unchanged and gains force from this baseline rather than losing any: P0's
  two accepted RustSec advisories are recorded, not resolved, so "no vulnerabilities" remains a claim
  this project must never make.

No product verb was added, no artifact class was reclassified, and no gate semantic moved. If a later
reader needs to know whether v0.6.0 changed the product: it changed what is proven, not what is
promised.
