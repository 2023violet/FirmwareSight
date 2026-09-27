---
title: "P0 Performance Report"
doc_id: "FS-P0-011"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Performance Report

Requirement: record what the pipeline costs on measured workloads, and show that the 512 MiB
full-buffer guard rejects an oversized input **before** allocating for it.

Status: **LOCAL PASS** for the guard and the timings; **NOT MEASURED** for peak RSS, with the
reason stated below rather than filled in with an estimate.

## How these numbers were produced

```
$ cargo build --release -p fwsight
$ python scripts/gen_p0_workload.py            # writes the gitignored workloads
$ python scripts/measure_workloads.py --runs 3
```

Stage timings are read from the CLI's own stderr diagnostics line, which reports
`hash / read / parse / normalize` separately; the total is measured around the process by the
harness. Each workload is run once untimed to warm the file cache, then three timed times, and
the best stage sample is paired with the median total. Nothing here is an estimate.

## Measured

| Workload | File size | hash ms | read ms | parse ms | normalize ms | total ms | exit | result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `firmware.elf` (real fixture, no MAP) | 7,288 | 0 | 0 | 0 | 0 | 8.1 | 0 | accepted |
| `firmware.elf` (real fixture + MAP) | 10,960 | 0 | 0 | 0 | 0 | 9.1 | 0 | accepted |
| `guard-100mib.elf` (synthetic) | 104,857,600 | 483 | 38 | 0 | 0 | 558.1 | 0 | accepted |
| `guard-256mib.elf` (synthetic) | 268,435,456 | 1,233 | 90 | 0 | 0 | 1,353.8 | 0 | accepted |
| `guard-512mib.elf` (synthetic) | 536,870,912 | 2,559 | 185 | 0 | 0 | 2,794.7 | 0 | accepted |
| `guard-over-512mib.elf` (synthetic) | 536,871,936 | - | - | - | - | 7.7 | 3 | **rejected** `ERR-GUARD-0001` |

Totals are the median of 3 runs; the sub-millisecond stage columns read as `0` because they are
reported in whole milliseconds.

## Environment

- Host: Windows 10 (10.0.19045), AMD64, 8 logical CPUs
- Build profile: release, `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`
- Binary: `target/release/fwsight.exe`, 1,689,088 bytes
- Rust: `rustc 1.98.1 (48a229cea 2026-09-01)`, `cargo 1.98.1 (797e8a9bc 2026-08-05)`
- Storage: local disk on `D:`; NVMe or HDD not distinguished by the harness
- Every workload file's SHA-256 is recorded in `P0_FIXTURE_REGISTER.md`

## The guard, and what the shape of the table proves

The rejected file is 1,024 bytes larger than the accepted one, and it costs 7.7 ms with **no**
hash or read time at all. That is the ordering working as specified:

```
stat -> size guard -> streaming SHA-256 -> magic detection -> single immutable read -> parse
```

`GuardedInput::load` compares `metadata.len()` against `max_full_buffer_bytes` before any buffer
exists, so an oversized input is refused without touching its bytes. `oversized_artifact_is_rejected_before_allocation`
asserts the typed `ArtifactTooLarge` result, and the boundary case at exactly 512 MiB is accepted,
which pins the comparison as `>` rather than `>=`.

Hashing dominates the cost at roughly 210 MB/s, and the read is roughly 7x faster than the hash -
consistent with a streaming SHA-256 over 64 KiB chunks against an OS-cached file. Parse and
normalize stay below a millisecond even at 512 MiB because the ELF structural read touches
headers and tables near the start of the buffer, not its volume. **That last sentence is an
observation about these synthetic files, not a claim about real firmware**, where table sizes
scale with symbol and section counts.

## Peak RSS: NOT MEASURED

Reason: reading a child process's peak working set on this host needs either a Win32 API
dependency inside the CLI (`GetProcessMemoryInfo`) or polling from the harness. The first would
add a non-sanctioned dependency and `unsafe` code into a crate that carries
`#![forbid(unsafe_code)]`; the second is unreliable for a process that lives for a few hundred
milliseconds. A number invented from the 512 MiB buffer size would be an estimate presented as a
measurement, which the baseline forbids.

What can be stated without measuring, and is labelled as derived rather than observed: the
pipeline holds at most one full buffer of `min(file_size, 512 MiB)` per artifact, plus a second
one if a MAP file is also within the limit, and it never memory-maps. The bound is a property of
the code path, not a measurement.

## What this benchmark does not cover

- No real firmware at these sizes. The workloads are synthetic by construction and marked as
  such in `P0_FIXTURE_REGISTER.md`.
- No concurrent analysis, no long-running session, no memory-growth-over-time curve.
- No comparison against another tool, so "fast" is not claimed - only "this is what it costs
  here".
- No CI timing. These figures are one machine, one build, and are not comparable to a shared
  runner.
- Fuzzing was not run. The scaffold requirement was satisfied by no-panic regression tests over
  the malformed corpus; `cargo-fuzz` needs a nightly toolchain and a separate crate, which the
  dependency and tool policy did not authorize for P0.
