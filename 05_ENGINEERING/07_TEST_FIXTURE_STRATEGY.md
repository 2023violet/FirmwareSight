---
title: "Artifact Fixture Strategy"
doc_id: "FS-ENG-008"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-09"
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

> **Dated 2026-10-09 (`ADR-0030`) — one line of this wishlist is now out of scope, not merely unbuilt.** The
> "Intel HEX valid + invalid checksum" fixture presumes a record-checksum validator, and C1 deliberately ships none:
> a release attachment's Intel HEX records, checksums and address span stay `Unknown / Not verified` and are never
> asserted. No such fixture is committed today, and none is required by C1's four units. If a later round wants to
> *verify* HEX content, that is structural analysis — a new ADR and a new unit, not a detail of this design
> (`04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md` §8, §11). The fixture set that C1 does need is byte-level: a BIN and
> a HEX image with known length and known SHA-256, plus a 0-byte file for the empty-attachment refusal.

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
