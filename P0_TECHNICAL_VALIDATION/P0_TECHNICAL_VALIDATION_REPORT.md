---
title: "P0 Technical Validation Report"
doc_id: "FS-P0-017"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Technical Validation Report

## Status

**P0 TECHNICAL VALIDATION: CONDITIONAL_PASS**

Not PASS, and therefore **`FirmwareSight_Project_Baseline_v0.6.0` is not delivered**. The unique
baseline remains `v0.5.1`; `BASELINE.yaml` records the P0 track as executed with three open
conditions.

The technical claim - that one headless Core produces the same facts for a CLI and a desktop
through a typed IPC boundary, on real linker output, with the memory rule demonstrable and the
size guard holding - is proven by executed tests. What is missing is three authorizations, not
three engineering unknowns.

## Open conditions

| # | Condition | Why it is open | Closes when |
| --- | --- | --- | --- |
| C1 | The CI workflow has never executed | This task carries no push authorization; `p0-check.yml` is committed but has no run history | `p0-check.yml` runs green on `main` (both OS matrices) |
| C2 | `cargo deny` has never run | The tool is not installed locally and installing it was not authorized | the `deny` job reports licenses, bans, sources and advisories |
| C3 | The desktop window has never been opened | Starting a GUI in the user's session was not authorized; compile, link and jsdom rendering are proven | one manual launch of `firmwaresight-desktop` with the summary screen confirmed on screen |

None of the three can be closed by editing a document. Each needs an action this task was not
given leave to take.

## What was proven

- **Parity.** `desktop_and_cli_report_the_same_core_facts_for_the_same_bytes` drives the shell's
  own `Session::analyze_and_store` and compares every field on the prompt's list against the
  committed CLI golden. The desktop's projection is a copy of the CLI's `AnalyzeResultDto`, so
  neither surface recomputes anything.
- **Memory accounting under ADR-0021.** On real GNU ld output with `AT> ROM` placement:
  nonvolatile 160 bytes, runtime RAM 72 bytes, `.data` (4) and `.ota` (64) counted in both, and
  the totals reproduced by hand from `readelf` plus the MAP region table. Withholding the MAP
  keeps the same numbers but drops the claim to `elf-address-and-flags` and
  `admissibleForHardBlock: false` - the model reports weaker evidence instead of false confidence.
- **Determinism.** Two runs of the same fixture compare byte-identical; the payload contains no
  timestamp, no per-run identifier and no absolute host path.
- **The guard.** 512 MiB accepted in 2.79 s; 512 MiB + 1 KiB refused in 7.7 ms with zero hash or
  read time, because the size check happens on the `stat` result before a buffer exists.
- **No panic on hostile input.** Four malformed fixtures, a truncated ELF, a zero-filled header,
  a foreign MAP and a region-less MAP all return typed errors.
- **Storage.** Nine tests over migrations, foreign-key enforcement, transactional import that
  cannot leave a half-written build visible, content-addressed dedupe, and refusal to delete a
  schema it does not understand.
- **Boundaries.** `firmwaresight-core` still declares zero dependencies; `ts-rs` is declared in
  exactly one crate; the capability file is byte-identical to the frozen baseline; no network,
  subprocess or `unsafe` exists anywhere in first-party code.

## Test and gate totals

```
$ cargo test --workspace                 101 passed / 0 failed  (6 members)
$ pnpm test                               18 passed / 0 failed
$ cargo fmt --all -- --check              clean
$ cargo clippy --workspace --all-targets --all-features -- -D warnings   clean
$ python scripts/check.py                 14 steps, 13 pass + 1 skipped (cargo-deny)
```

## Architecture verdict

Nothing in the frozen baseline had to be renegotiated to build this. No ADR was required, no
technology was substituted, and no boundary moved: the four Phase-0 crates are the four that were
planned, and the places where the baseline was silent (`SnapshotId` derivation, the RAM
discriminator, `p0-internal` naming, TypeScript 6.0.3) were decided and recorded rather than
quietly invented.

Two findings worth naming because they are the kind that usually hide:

1. **Declared-but-unused dependencies.** The CLI declared two crates no source file referenced,
   and the artifact crate declared `tracing`. A grep for forbidden crates would not have caught
   this; the architecture check did.
2. **A provenance record that did not describe its own build.** `fixture.toml` wrote the linker
   invocation as `-p0-dual-region.ld` where the toolchain was actually called with
   `-T p0-dual-region.ld`. The artifacts were right and the record was wrong. Regenerating with
   `--force` showed both ELF files byte-identical, which is also the reproducibility proof.

## Gate recommendation

Do not open G1. Do not start P1.

P0's own claim is "the architecture can carry the product", and on the evidence above it can. The
conditions that keep this at `CONDITIONAL_PASS` are about who ran the check, not what the check
found - which is precisely the distinction the baseline says to preserve.

To convert this to PASS: authorize a push so C1 and C2 execute, then launch the desktop once for
C3. If any of the three fails, the status drops to `FAIL` or `BLOCKED` as appropriate, and the
v0.6.0 baseline stays withheld.

## Boundary kept with V0

The V0 prototype track remains deferred and unvalidated under ADR-0020. No V0 evidence was
modified, no V0 gate recommendation was softened, and nothing in this report implies the seven
prototype screens were re-examined.

## Document set

`README.md`, `P0_EXECUTION_PROVENANCE.md`, `P0_PLAN.md`, `P0_IMPLEMENTATION_LOG.md`,
`P0_ARCHITECTURE_CHECK.md`, `P0_FIXTURE_REGISTER.md`, `P0_PARSER_RESULTS.md`,
`P0_MEMORY_ACCOUNTING_REPORT.md`, `P0_CLI_PARITY_REPORT.md`, `P0_IPC_PARITY_REPORT.md`,
`P0_STORAGE_REPORT.md`, `P0_PERFORMANCE_REPORT.md`, `P0_SECURITY_INPUT_REPORT.md`,
`P0_DEPENDENCY_REPORT.md`, `P0_CI_REPORT.md`, `P0_KNOWN_LIMITATIONS.md`, `P0_EXIT_CHECKLIST.md`.
