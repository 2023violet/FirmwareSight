---
title: "P0 Dependency Report"
doc_id: "FS-P0-013"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Dependency Report

Requirement (AGENTS.md 4): every dependency must name the need it serves, say why the standard
library or an existing dependency was insufficient, and state its license, maintenance and build
impact. Nothing enters because it is common in the ecosystem.

Status: **LOCAL PASS on all four checks, including cargo-deny itself.** The policy file is no
longer "written but not executed": `cargo deny 0.20.2` runs here and exits 0. What is still missing
is a CI run that says the same thing - Run #1's `deny` job failed on a schema error, and the HEAD
that fixes it has not been pushed.

## Scale

`Cargo.lock` resolves **466 packages**, of which 6 are workspace members: **460 third-party Rust
packages**. The Tauri 2 shell accounts for the overwhelming majority; the four library crates
together add a small direct set. The UI adds 15 packages under pnpm, Python tooling adds one.

## Rust: direct dependencies, one line of justification each

| Crate | Version | Used by | Why it is here | License | Trusted core? |
| --- | --- | --- | --- | --- | --- |
| `object` | 0.40.0 | artifact | ELF section/symbol/header reading; writing and maintaining a container parser is a larger risk than depending on one | MIT OR Apache-2.0 | no - confined to `elf.rs`, no `object` type crosses a boundary |
| `sha2` | 0.11.0 | artifact | SHA-256 over a streaming reader; the standard library has no cryptographic hash | MIT OR Apache-2.0 | no |
| `thiserror` | 2.0.21 | artifact, report, storage | typed error enums per `05_ENGINEERING/03`; hand-writing `Display`/`Error`/`From` for nine variants per crate is boilerplate with a panic surface | MIT OR Apache-2.0 | no |
| `serde` / `serde_json` | 1.0.229 / 1.0.151 | report, desktop | deterministic JSON with declaration-order keys | MIT OR Apache-2.0 | no |
| `rusqlite` (bundled) | 0.40.2 | storage | the frozen SQLite binding; `bundled` keeps the build self-contained | MIT OR Apache-2.0 | no |
| `tracing` | 0.1.44 | storage, cli | structured diagnostics with an operation id | MIT | no |
| `tracing-subscriber` | 0.3.23 | cli | env-filtered stderr logging; only the CLI configures a subscriber | MIT | no |
| `clap` (derive) | 4.6.7 | cli | argument parsing and the frozen usage/`--version` surface | MIT OR Apache-2.0 | no |
| `tauri` | 2.12.0 | desktop | the frozen desktop framework; 2.x stable, not the 3.0 alpha | Apache-2.0 OR MIT | no |
| `tauri-build` | 2.7.0 | desktop (build) | required by the Tauri 2 build pipeline | Apache-2.0 OR MIT | no |
| `ts-rs` | 12.0.1 | desktop | generate the TypeScript IPC contract (ADR-0019); declared in exactly one crate | MIT OR Apache-2.0 | no |

`firmwaresight-core` still has an empty `[dependencies]` table. That is the strongest structural
statement P0 makes about trust: the domain layer has no code in it that anyone else wrote.

Build impact: a release `fwsight.exe` is 1,689,088 bytes; the SQLite `bundled` feature requires a
C compiler on the build host, which both CI images provide.

## Refused, with the reason

| Candidate | Why not |
| --- | --- |
| `hex` | a ten-line lowercase encoder; `Sha256::parse` validates the same alphabet, so encoder and validator cannot drift |
| `anyhow` | typed errors are the frozen error model. It does appear in the graph, but only inside Tauri - no first-party crate declares it |
| `tokio` (direct) | Core stays synchronous. `tokio 1.53.1` is Tauri's own runtime, reached only through `tauri::async_runtime`; `grep "tokio::" crates apps` returns nothing |
| `memmap2` | the guard's contract is "one bounded buffer or a typed refusal", and an mmap fallback would make the limit a soft suggestion |
| `sqlx` | rusqlite is frozen (AGENTS.md 6) |
| `gix` | Git facts in P0 come from declared provenance or stay Unknown; a repository library is a larger attack surface than the feature warrants |
| `reqwest` / `rustls` / `hyper` | no network in a local-first product; **zero occurrences in the graph** |
| `rayon` | P0 is synchronous; parallelism needs an ADR |
| `wgpu`, `axum`, `tonic` | explicitly forbidden by AGENTS.md 2 and 4 |
| `cargo-fuzz` | needs a nightly toolchain and a separate crate; regression tests carry the no-panic claim instead |
| a UI component library, a chart library, a state library | DESIGN.md 5 is a ruleset, not a dependency, and P0 shows one summary screen |

## Node and pnpm

`apps/desktop/ui/package.json` pins `packageManager: "pnpm@12.7.0"` — the version the frozen
`examples/package.baseline.json` names — and exact dependency versions; `.npmrc` sets
`save-exact=true` and `engine-strict=true`. Installs run through Corepack so the version that wrote
`pnpm-lock.yaml` is the version that resolves it - the global pnpm on this machine is 11.21.0 and was
never used. Mid-track the pin read 12.6.0, one patch below the frozen asset; nothing failed because
of it, and it was found by comparing the delivered pin against `04_TECH/10_TOOLCHAIN_BASELINE.md`
before delivery.

Runtime: `react 19.3.0`, `react-dom 19.3.0`, `@tauri-apps/api 2.12.0`.
Toolchain: `typescript 6.0.3`, `vite 8.3.1`, `@vitejs/plugin-react 6.1.1`, `eslint 10.11.0`,
`typescript-eslint 8.70.1`, `@eslint/js 10.0.1`, `vitest 5.0.2`, `jsdom 30.1.1`,
`@testing-library/react 16.3.3`, `@testing-library/dom 10.4.2`, `@types/react` /
`@types/react-dom` 19.3.0.

**TypeScript 6.0.3 rather than 7.0.2** is a real dependency decision, not an oversight:
`typescript-eslint@8.70.1` declares `typescript: ">=4.8.4 <6.1.0"`, and AGENTS.md 10 requires
frontend lint. Taking 7.x would have meant an unsupported parser or no lint gate. Recorded in
`P0_IMPLEMENTATION_LOG.md` 12.

## Python

Only `Pillow==12.3.0` (`scripts/requirements.txt`), used by `gen_desktop_icons.py` to write the
`.ico` tauri-build requires on Windows. Every other repository script is standard library only.
`gen_p0_fixtures.py` additionally needs `arm-none-eabi-gcc`, by hand only - no CI job runs it.

## Licenses

The allow list in `deny.toml` is now read out of the tool instead of out of the archives.
`cargo deny list -t 1.0 --layout license` walks the graph for the four shipping targets named in
`[graph] targets` - 293 packages resolve for `x86_64-pc-windows-msvc` alone, against the 466 lines in
`Cargo.lock` - and lists a crate **once per alternative its declared expression offers**, so
`sha2@0.11.0` (`MIT OR Apache-2.0`) appears under both `MIT` and `Apache-2.0`. The counts below are
therefore crate-license pairs and sum to more than the number of packages. That is the correct shape
for the question being asked, which is "is every license a user could be handed by this tree on a
platform we ship allowed?"

| License | Entries |
| --- | --- |
| `MIT` | 323 |
| `Apache-2.0` | 224 |
| `Unicode-3.0` | 19 |
| `Zlib` | 11 |
| `Unlicense` | 7 |
| `MPL-2.0` | 5 |
| `BSD-3-Clause` | 4 |
| `0BSD`, `CC0-1.0`, `MIT-0`, `Apache-2.0 WITH LLVM-exception` | 1 each |
| `Unlicensed` | 6 - the workspace's own crates, exempted by `private = { ignore = true }` because cargo-deny evaluates dependencies, not members |

Reading this from the tool rather than from `LICENSE` files corrected three things the earlier draft
of this section asserted:

- **The slash-form entries were never required.** 12 crates reachable on the Windows target still
  write a legacy string instead of an SPDX expression (`bitflags@1.3.2` declares `MIT/Apache-2.0`).
  The previous allow list carried `MIT/Apache-2.0` and four other slash forms literally on the
  assumption that cargo-deny would treat them as opaque single tokens. It parses `/` as a
  disjunction, so the literal names matched nothing that the parsed forms had not already covered:
  deleting all five leaves `licenses ok`. A stale allow entry is not free - with
  `unused-allowed-license = "warn"` it is a warning that outlives the reason it was added, and it
  widens the policy for crates that no longer need the widening.
- **`ISC` is not in this graph.** It came from the archive count. Removing it also leaves
  `licenses ok`, which is the only check that distinguishes "absent" from "present and permitted".
- **`r-efi`'s LGPL option is outside the graph cargo-deny evaluates.** `r-efi` appears twice in
  `Cargo.lock` (5.3.0, 6.0.0) declaring `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; `cargo deny list`
  for the four shipping targets returns zero matches for it, because it is a UEFI-only crate no
  desktop build reaches. The disjunction was always satisfied by its MIT option, so nothing here was
  ever copyleft-bound; under the target filter the question is not asked at all. **No copyleft
  license is permitted on its own anywhere in this policy.**

## Verification status

```
$ python scripts/check.py --only deny
=== [deny] cargo-deny
$ cargo deny check licenses bans sources advisories      (cargo-deny 0.20.2)
warning[duplicate]: found 3 duplicate entries for crate 'base64'
   ...
advisories ok, bans ok, licenses ok, sources ok

=== summary ===
PASS       deny/cargo-deny

1/1 steps passed
```

`cargo deny 0.20.2` is installed on this machine and the check exits 0, so the license, ban, source
and advisory results are **LOCAL PASS** rather than `NOT RUN`. The contents of that pass, stated
precisely:

- **Zero vulnerability advisories.** The informational class carries two entries, both in the frozen
  Tauri 2.12.0 dependency architecture and both recorded in `deny.toml` with the reason rather than
  silenced by widening scope: `RUSTSEC-2024-0429` (`glib 0.18.5`, unsound) and `RUSTSEC-2024-0370`
  (`proc-macro-error 1.0.4`, unmaintained, host-only build dependency). Neither ignore entry has gone
  stale - the check prints no `unused-ignored-advisory` warning, which is the mechanism that would
  notice.
- **The glib upgrade is not available inside the frozen architecture.** `cargo update -p glib
  --precise 0.20.0` fails: `gtk v0.18.2`, required by Tauri 2.12.0's Linux stack, depends on
  `glib = "^0.18"`. Moving glib means moving gtk-rs, muda, tao and webkit2gtk, which is an ADR, not a
  CI fix. Reported to the architect as an architecture conflict rather than resolved by moving a
  dependency; see `P0_CI_REMEDIATION_REPORT.md`.
- **23 `warning[duplicate]` results** from `multiple-versions = "warn"`, all in the `syn`, `toml`,
  `thiserror`, `windows-sys`, `base64`, `sha2` and `png` families, produced by the Tauri and SQLite
  trees rather than by a choice of ours. They are warnings because the alternatives are dropping a
  frozen dependency or `skip`-ing the evidence away; `highlight = "all"` makes each one print the
  lowest version and the shortest path that reaches it.
- **The ten `[bans] deny` entries were exercised on the graph, not just declared.** `reqwest`,
  `hyper`, `rustls`, `memmap2`, `sqlx`, `gix`, `rayon`, `wgpu`, `axum` and `tonic` do not appear on
  any shipping target. `anyhow` does - as a non-optional dependency of `tauri`, `tauri-build` and
  `tauri-utils` - and the ban stays live for first-party code through `wrappers`, so a crate of ours
  that declared it would still fail. This is the boundary `AGENTS.md` 3 draws in our own error model,
  checked rather than asserted.

What this does **not** claim: CI has not seen this file. Run #1's dependency job failed on the
schema errors (an unsupported `severity-threshold`, five slash strings cargo-deny rejected) and was
fixed here; the remediation HEAD is unpushed, so the CI result for the dependency boundary is
`FAIL` at Run #1 and `NOT RUN` since. Both `P0_CI_REPORT.md` and `P0_EXIT_CHECKLIST.md` carry the
same split.
