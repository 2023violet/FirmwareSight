---
title: "P0 CLI Report and Determinism"
doc_id: "FS-P0-008"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 CLI Report and Determinism

Requirement: `fwsight analyze` renders Core facts and nothing else, emits deterministic JSON, and
returns only the frozen exit codes.

Status: **LOCAL PASS**

## Command surface

Only `analyze` is registered. `diff`, `gate` and `release` do not exist in the binary, so they
cannot be mistaken for working features; exit codes 4, 5 and 6 are unreachable because no code
path can return them.

```
$ fwsight --version
fwsight 0.1.0
```

## Exit codes, measured against the release binary

| Invocation | Exit | Meaning |
| --- | --- | --- |
| `analyze fixtures/elf/p0-basic/firmware.elf` | `0` | success |
| *(no arguments)* | `2` | usage |
| `analyze /nope/missing.elf` | `3` | parse/import failure |
| `1`, `4`, `5`, `6` | - | never produced; `1` is not part of the frozen table |

The usage path goes through `parse_from` rather than `Cli::parse`, because `Cli::parse` exits the
process itself and the code would then be clap's rather than the one the table promises.
`exit_code_one_is_never_produced` and
`every_user_facing_error_maps_to_exit_three` assert the mapping in tests;
`unregistered_future_commands_are_not_accepted` asserts that `diff`, `gate`, `release`, `watch`
and `doctor` are all rejected.

## Determinism, measured

Two runs of the dual-region fixture, one second apart:

```
$ fwsight analyze ... --json > /tmp/r1.json
$ fwsight analyze ... --json > /tmp/r2.json
$ cmp /tmp/r1.json /tmp/r2.json
BYTE-IDENTICAL across runs
$ sha256sum
96f0645d8d0b3c48718bb1fc1870a33caabd5b4010a4776317b6f8b5ae087f52   (both)
```

The payload carries no wall-clock timestamp, no per-run UUID and no absolute host path -
grep for a drive letter in the output returns nothing. Key order comes from Rust declaration
order, not from a `HashMap`, which is why the bytes match without needing a canonicalizer.

What is *not* deterministic on purpose: the `operation_id`, which appears only on stderr
diagnostics and in error envelopes, never in the JSON document. `--json` stdout carries exactly
one document, so a caller can pipe it straight into a parser.

## Error output shape

```
error: No artifact was found at that path. (code ERR-INPUT-0001, operation op-69c-18d9408560e053cc)
```

Four parts, as the error model requires: what happened, the stable code, what to do
(`--json` puts the remediation in the envelope), and the diagnostics id. The sentence comes from
`ArtifactError::user_message()` in the artifact crate, shared with the desktop rather than
copy-written per surface.

## Goldens

```
$ RUSTUP_TOOLCHAIN=stable cargo test -p fwsight --test golden
running 5 tests ... test result: ok. 5 passed; 0 failed
```

- `cli_analyze_json_matches_the_committed_golden` - full payload, dual-region
- `cli_analyze_without_map_matches_the_committed_golden` - full payload, MAP-less
- `memory_golden_records_evidence_and_not_only_numbers` - locator, classification and rule are
  asserted, not just arithmetic
- `without_region_evidence_the_same_source_is_reported_as_weaker_evidence`
- `memory_accounting_golden_passes_on_both_fixtures` - consumes `golden/core/`

Comparison is semantic (keys canonicalized), so a reordering that changes no meaning does not
fail, while any changed value does. Goldens are rewritten only by
`scripts/update_goldens.py --confirm`, which prints the semantic diff of every leaf it would
touch and refuses to write anything without the flag. A failing test never updates its own
expectation.

## What the CLI cannot claim

`p0-internal` is stamped in the payload itself (`schemaStability`), because this shape is not
`release-manifest v1` and P0 has not earned a compatibility promise. Exit code 3 covers parse
*and* import: there is no gate path, so nothing here says a firmware is releasable.
