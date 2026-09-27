---
title: "Post-MVP Signal Register"
doc_id: "FS-RSCH-011"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-27"
---

# Post-MVP Signal Register

| Signal | Evidence strength | v0.5 treatment |
|---|---|---|
| basic in-app memory/change visualization | strong category evidence | presentation of MVP facts; no new MVP feature family |
| enhanced HTML report | medium-strong fit | E1 candidate |
| CSV / Markdown export | moderate / low-cost | E1 candidate |
| XLSX | no direct signal | HOLD / interview |
| DOCX | no direct signal | HOLD / enterprise-only if proven |
| native PDF renderer | weak value/cost mismatch | REJECT; HTML print view |
| component version drift | medium + official ESP-IDF lock evidence | E1 candidate |
| `.su` stack evidence | strong category evidence | E2 candidate, ADR |
| SBOM evidence completeness | medium + CRA/SBOM workflow pressure | E2 candidate |
| evidence retention semantics | medium, largely existing foundation | E2 incremental |
| release symbol archive | medium-strong category evidence | E2 candidate |
| ABI Gate | medium concept evidence, high complexity | E3/GX conditional |
| CI/PR comments/history | strong market pattern | GX candidate after V1/team signal |
| watch mode | product hypothesis | E3, only if repeat frequency proven |
| generic changelog generator | weak differentiation | default no |
| OTA metadata/platform | outside scope | reject/defer |
| full call graph | expensive adjacent problem | separate future ADR only |
| AI release judgment | conflicts with trusted-core ADR | reject |
