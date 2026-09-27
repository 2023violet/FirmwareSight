---
title: "P0 Security and Untrusted Input Report"
doc_id: "FS-P0-012"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Security and Untrusted Input Report

Requirement: artifacts are untrusted input, the WebView gets no general-purpose power, and the
product keeps its local-first boundary.

Status: **LOCAL PASS**

## 1. Untrusted input handling

Every file the tool reads arrives through one path with a fixed order:

```
stat -> size guard -> streaming SHA-256 -> magic detection -> single immutable read -> parse
```

| Property | Implementation | Evidence |
| --- | --- | --- |
| Allocation happens after the size check | `GuardedInput::load` compares `metadata.len()` before any buffer exists | `oversized_artifact_is_rejected_before_allocation`; measured 7.7 ms rejection with zero hash/read time |
| The default ceiling is 512 MiB | `DEFAULT_MAX_FULL_BUFFER_BYTES` | accepted at exactly 512 MiB, rejected at 512 MiB + 1 KiB |
| No mmap fallback | `intake.rs` documents and implements read-only streaming | `grep -rn "mmap\|memmap" crates apps` finds no use; `memmap2` absent from the tree |
| The bytes hashed are the bytes parsed | one immutable read after hashing, plus a re-stat that fails if the file changed mid-read | `InputUnreadable` / changed-while-reading path |
| Format detection without assumptions | `EI_CLASS` / `EI_DATA` read from the header; 32/64 and big/little all four accepted | `real_elf_fixture_is_detected_and_hashed` |
| No parser panic on hostile input | four malformed files, a truncated ELF, a zero-filled header, a foreign MAP | `malformed_inputs_return_typed_errors_and_never_panic`, `truncated_elf_is_reported_as_invalid_container_not_as_a_missing_file` |
| Unknown container is not guessed at | `UnsupportedFormat` for a non-ELF magic | `wrong-magic.bin` case |
| Absent facts stay absent | `Fact::Unknown { reason }`, never a defaulted zero | `unknown_load_address_stays_unknown_rather_than_defaulting_to_zero` |

Fuzzing was not run. `cargo-fuzz` needs a nightly toolchain and a separate crate; the tool policy
did not authorize that for P0, and the prompt asked for regression tests rather than a campaign.
This is listed in `P0_KNOWN_LIMITATIONS.md` rather than being claimed as coverage.

## 2. Tauri capability surface

`apps/desktop/src-tauri/capabilities/main.json` is the frozen baseline, verified field by field:

```
identical to baseline: True
permissions: ['core:default']   windows: ['main']
```

No shell, no filesystem, no dialog, no HTTP, no updater, no SQL. The webview cannot reach the
filesystem except through the two commands below.

### Commands, and why they are not arbitrary

| Command | Argument | Returns |
| --- | --- | --- |
| `list_fixtures` | none | the closed fixture set with labels |
| `get_analysis_summary` | `fixture: FixtureKey` - a two-variant Rust enum | a bounded `AnalysisSummaryDto` or an `ErrorEnvelopeDto` |

There is no `read_file`, `run_sql`, `execute_shell` or `get_any_path`. The WebView sends a key
from a closed set; the path is resolved inside the shell, so a hostile page cannot name a file
even in principle. Asserted by
`the_ipc_payload_is_bounded_and_carries_no_path` (no `path`/`filePath`/`directory` key in any
payload) and by the UI test `asks the shell for the selected fixture, never a path`.

### Content policy

`tauri.conf.json` sets a CSP that allows no remote origin:

```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
img-src 'self' data:; font-src 'self';
connect-src 'self' ipc: http://ipc.localhost;
object-src 'none'; base-uri 'self'; frame-ancestors 'none'
```

`style-src 'unsafe-inline'` is present because Vite injects CSS as inline styles in dev mode. It
is the one concession, and it is called out here rather than left to be discovered later.

`bundle.active = false`: P0 validates the shell, and produces no installer, no signing and no
updater - the areas AGENTS.md 7 reserves for a signing ADR.

## 3. Process and environment reach

```
$ grep -rn "process::Command" crates apps --include=*.rs
(no matches)
$ grep -rn "std::env" apps crates --include=*.rs
apps/cli/src/main.rs:71        std::env::args_os()
apps/desktop/src-tauri/src/service.rs:38   std::env::var_os("FIRMWARESIGHT_FIXTURE_DIR")
```

No subprocess is spawned anywhere - including for Git, which is why the Git capability reports
`unknown` instead of pretending an adapter ran. Two environment reads: process arguments, and one
fixture-root override intended for development and the parity harness. That override is operator
configuration, not attacker-controlled input: a process that could set the desktop's environment
already has filesystem access, so listing it honestly is more useful than implying a defence that
is not there.

## 4. Local-first boundary

- No HTTP client, no TLS stack, no telemetry, no update check: `reqwest`, `hyper`, `rustls`,
  `native-tls`, `awc`, `ureq` all have zero occurrences in the dependency graph.
- No server, no auth, no cloud sync, no background network activity.
- Artifact bytes are never copied into the database; only path metadata, hashes and derived facts
  are stored, and the database lives under the platform application-data directory.
- Deterministic payloads carry no absolute host path, so an exported analysis does not leak which
  machine produced it.

## 5. Unsafe code

```
#![forbid(unsafe_code)]   in core, artifact, report, storage and the desktop lib
$ grep -rn "unsafe" crates apps --include=*.rs | grep -v forbid
(no matches)
```

## 6. What this does not cover

No threat model beyond hostile-file and hostile-page; no supply-chain verification (no lockfile
signature checking or provenance attestation); no binary hardening review; no sandboxing beyond
the Tauri capability model; and no test of what happens when a malicious extension is installed
into the user's WebView2 runtime.
