---
title: "Artifact Fixture Strategy"
doc_id: "FS-ENG-008"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Artifact Fixture Strategy

## Why fixtures are product infrastructure

FirmwareSight makes claims about toolchain artifacts.
Compatibility cannot be tested with mocked structs alone.

## Fixture classes

### Self-built
Source kept in repo; CI rebuilds selected fixtures where deterministic enough.

### Frozen binary
Checked-in small ELF/MAP/HEX with:
- source；
- toolchain/version；
- SHA-256；
- expected facts。

### Public project
License permits redistribution or fixture is built in CI from source.

### Malformed corpus
Truncated/mutated edge cases; no confidential input.

## MVP matrix

At least:
- Cortex-M GCC minimal
- FreeRTOS-like project
- LVGL-like larger image/font sections
- stripped ELF
- DWARF ELF
- no-symbol ELF
- GNU ld MAP with archives/object contributions
- Intel HEX valid + invalid checksum
- malformed ELF headers

## Golden facts

Each fixture has:
```text
fixture.toml
expected-analysis.json
expected-capabilities.json
```

No human-only screenshot assertions for parser truth.

## Regression rule

A parser output golden change requires:
- explanation；
- review；
- fixture evidence。

Never mass-update goldens because tests failed.
