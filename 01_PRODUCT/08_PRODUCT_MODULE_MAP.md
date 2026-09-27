---
title: "Product Module Map"
doc_id: "FS-PRD-009"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product"
last_updated: "2026-09-27"
---

# Product Module Map

| Module | User question | Current MVP | Post-MVP extension boundary |
|---|---|---:|---|
| Intake | What exactly did I give FirmwareSight? | Yes | more fixture-proven toolchain inputs |
| Analyze | What is in this build? | Yes | component evidence, `.su`, richer evidence |
| Compare | What changed? | Yes | component drift, evidence-grade drift, optional ABI depth |
| Gate | Does configured policy allow this release? | Yes | new deterministic evidence rules only after validation |
| Bundle | What evidence ships with the release? | Yes | symbol archive / retention refinements |
| History | What did we know at each point in time? | Minimal/local | baseline pin, richer trends, watch-triggered entries |
| CLI | Can the same facts be scripted? | Yes | hardened workflow recipes/integration packaging |
| Report | Can evidence leave the app reproducibly? | HTML/JSON baseline | enhanced HTML, CSV/MD; XLSX/DOCX only if signal |
| Dependency evidence | What components/versions can we support with evidence? | evidence model/UI reference | fixture-gated component sources / completeness |
| Integration ports | Where does release evidence enter/leave workflows? | local file + CLI | recipes → conditional CI/Growth |

## Rule

No extension creates a fifth top-level mental-model verb.

Every new capability must attach to:
`Analyze / Compare / Gate / Release`
and identify the evidence it consumes/produces.
