---
title: "FirmwareSight"
doc_id: "FS-ROOT-README"
product: "FirmwareSight"
version: "0.6.0"
status: "CURRENT"
owner: "Project Lead"
last_updated: "2026-10-14"
---

# FirmwareSight

**Embedded firmware release workbench. Know exactly what ships.**

FirmwareSight is a local-first desktop app and CLI (`fwsight`) that turns a firmware build into release evidence you can check:

- **Analyze** an ELF, optionally with its linker MAP (GNU ld, or armlink from Keil MDK / Arm Compiler): sections, symbols, memory regions, objects and libraries, and top contributors.
- **Compare** two builds: a deterministic diff, with each memory delta explained and given a confidence level.
- **Release Gate**: evaluates your project policy into PASS / REVIEW / BLOCK / UNKNOWN / N/A.
- **Bundle**: exports a verified, portable release evidence package.

Nothing leaves your machine. Unknown or partial evidence stays explicit and is never shown as zero or as proof.

Status: version `0.6.0`, open source under the [Apache License 2.0](LICENSE) ([ADR-0035](09_ADR/ADR-0035-apache-2-open-source-license.md)). Release binaries are not code-signed yet; see [SECURITY.md](SECURITY.md). Live status is in [.ai/NOW.md](.ai/NOW.md).

## Supported inputs

| Input | Support |
|---|---|
| ELF32 / ELF64, little or big endian | Parsed. Memory accounting is verified for GCC + GNU ld and Clang + GNU ld |
| GNU ld `.map` | Supported |
| Keil MDK / Arm Compiler (armlink) `.axf` | Parsed; memory accounting matches armlink's own totals on the committed sample ([ADR-0036](09_ADR/ADR-0036-armlink-map-adapter.md)) |
| armlink `.map` (`--map`, ideally with `--load_addr_map_info`) | Supported: regions, MAP-backed accounting, object and library attribution. Verified on one synthetic Arm Compiler 6.24 sample; real-project cohort UNVERIFIED |
| IAR (ILINK) output and `.map` | Not supported yet |
| `.bin` / Intel HEX | Attachable as release byte evidence; content not analyzed |

Details: [04_TECH/03_FORMAT_SUPPORT.md](04_TECH/03_FORMAT_SUPPORT.md), [04_TECH/39_ELF_TOOLCHAIN_INTEROPERABILITY.md](04_TECH/39_ELF_TOOLCHAIN_INTEROPERABILITY.md).

## CLI

```bash
cargo run -q --bin fwsight -- analyze fixtures/elf/p0-dual-region/firmware.elf --map fixtures/elf/p0-dual-region/firmware.map --json
```

| Command | Purpose |
|---|---|
| `fwsight analyze <ELF> [--map <MAP>] [--json]` | Analyze one build |
| `fwsight diff <OLD> <NEW>` | Compare two builds |
| `fwsight gate --project <DIR> --artifact <FILE>` | Evaluate the release Gate |
| `fwsight release prepare --project <DIR> --out <DIR>` | Write a verified Release Bundle |
| `fwsight project doctor <DIR>` | Read-only readiness check |

Full spec, including exit codes and visual report export: [04_TECH/07_CLI_SPEC.md](04_TECH/07_CLI_SPEC.md).

## Build and verify

Prerequisites: Rust `1.98.1` (pinned in `rust-toolchain.toml`), Node with Corepack (pnpm), Python 3, and on Linux the Tauri system packages.

```bash
python scripts/check.py
```

This is the full local gate: Rust fmt, clippy and tests, frontend typecheck, lint, tests and build, generated-output drift checks, and cargo-deny. Run `python scripts/check.py --only <rust|frontend|drift|deny|core-smoke|package>` for a single group. CI runs the same groups on Windows, macOS and Ubuntu: [05_ENGINEERING/06_CI_CD_BASELINE.md](05_ENGINEERING/06_CI_CD_BASELINE.md).

## Repository map

| Path | Contents |
|---|---|
| `crates/` | Rust Core (headless), artifact, storage, project, report crates |
| `apps/cli/` | `fwsight` CLI |
| `apps/desktop/` | Tauri 2 shell (`src-tauri/`) and React UI (`ui/`) |
| `fixtures/`, `golden/`, `schemas/` | Test inputs, deterministic reference outputs, public data contracts |
| `scripts/` | Gate, generators and verifiers |
| `01_PRODUCT/` … `08_RESEARCH/` | Product, brand, design, technical and engineering specifications |
| `09_ADR/` | Architecture decisions |
| `*_VALIDATION/`, `10_AUDIT/` | Per-stage evidence and archived source prompts (history) |
| `.ai/` | Short current state for contributors and coding agents |

Document index: [INDEX.md](INDEX.md). Contributor rules: [AGENTS.md](AGENTS.md), [DESIGN.md](DESIGN.md).

## History

The previous version of this README was a 1,288-line dated project chronicle. It is preserved unchanged at [.ai/archive/2026-10-14/ROOT_README.md](.ai/archive/2026-10-14/ROOT_README.md). Changes by version: [CHANGELOG.md](CHANGELOG.md).

## License

FirmwareSight is licensed under the [Apache License, Version 2.0](LICENSE). See [NOTICE](NOTICE). Release binaries include third-party components under their own licences, listed in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

Contributions are welcome: see [CONTRIBUTING.md](CONTRIBUTING.md). Report security issues privately: see [SECURITY.md](SECURITY.md).
