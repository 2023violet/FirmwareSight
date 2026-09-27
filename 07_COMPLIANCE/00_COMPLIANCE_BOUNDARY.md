---
title: "Compliance Boundary"
doc_id: "FS-COMP-001"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Compliance"
last_updated: "2026-09-26"
---

# Compliance Boundary

## What FirmwareSight may do

- collect engineering evidence；
- build artifact inventory；
- record provenance；
- generate SBOM formats；
- highlight missing evidence；
- retain release records；
- support internal processes。

## What FirmwareSight must not claim

- “Your product is CRA compliant”
- “Certified”
- “Audit passed”
- “No vulnerabilities”
- “SBOM is complete” unless coverage methodology supports that exact statement

## Product wording

Prefer:
- `Evidence available`
- `Evidence missing`
- `Review required`
- `Component detected`
- `Version unverified`

Avoid:
- compliant/non-compliant verdicts
- legal advice
- guaranteed security

## Human responsibility

最终：
- release approval；
- legal interpretation；
- conformity assessment；
- vulnerability reporting；
由用户组织和合格专业人员负责。


## UI readiness wording

`Can we ship now?` is allowed only as a shorthand for:
> “Does this build satisfy the configured FirmwareSight release policy with the currently available evidence?”

It must never be presented as:
- legal conformity approval
- cybersecurity certification
- safety certification
- regulatory authorization
- absence of vulnerabilities
