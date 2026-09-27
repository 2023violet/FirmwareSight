---
title: "P0 Execution Provenance"
doc_id: "FS-P0-000"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Audit / Engineering"
last_updated: "2026-09-27"
---

# P0 Execution Provenance

## Unique baseline

`FirmwareSight_Project_Baseline_v0.5.1`

## Source prompt

- Filename: `FirmwareSight_P0_Technical_Vertical_Slice_EXECUTION_PROMPT_v1.1_ARCHITECT_REVIEWED.txt`
- Preserved copy: `10_AUDIT/SOURCE_PROMPTS/` of this repository
- SHA-256: `60a59196708a53592be4d828a0c1275cf74b3bfad0bdc6455f863ab38d32929b`
- Byte size: `94802`
- Line count: `5064`
- Copy verification: SHA-256 of the archived copy re-computed after copying and equal to the
  value above, so the archived file is byte-identical to the delivered attachment
  (not a retranscription).

## Git state at execution start

Recorded after the mandatory read-only preflight required by PRE-0.1 and the real-time
remote sync required by PRE-0.2.

| Field | Value |
|---|---|
| START_BRANCH | `main` |
| START_LOCAL_HEAD | `e086e9880e00900e305a3cd88b4d899f73aa5711` |
| START_REMOTE_HEAD (`origin/main` after `git fetch --prune origin`) | `e086e9880e00900e305a3cd88b4d899f73aa5711` |
| START_REMOTE_HEAD recorded in the prompt (PRE-0) | `e086e9880e00900e305a3cd88b4d899f73aa5711` |
| ahead/behind `HEAD...origin/main` | `0 / 0` |
| Sync case | PRE-0.2 Case A (local == origin/main) |
| START_WORKTREE_STATUS | clean — `git status --short` empty; no staged, no untracked changes at preflight |

The reviewed snapshot in the prompt equals the live remote and local HEAD, therefore
`git diff --name-status e086e9880e00900e305a3cd88b4d899f73aa5711..origin/main` is empty and
no authority file changed after review. No STOP / ARCHITECTURE AUTHORITY CONFLICT applies.

## Working-history note

Earlier in the same working session the tree carried the uncommitted v0.5.1 execution work
(138 tracked modifications plus the untracked `V0_VALIDATION/` and `10_AUDIT/SOURCE_PROMPTS/`
directories). That work was committed as `e086e98` before this preflight, so it is preserved in
Git history. No `reset`, `clean`, `checkout -- .`, `restore` or `stash` was used at any point.

## Authorization

- Authorized by the user through the source prompt above.
- Consistent with `09_ADR/ADR-0020-validation-sequence.md` (Accepted), which authorizes
  V0 and P0 as parallel validation tracks, with P1 starting only after both PASS.
- This prompt supersedes the execution-state statement `P0 = NOT AUTHORIZED` recorded in
  `.ai/ACTIVE_TASK.md` during the V0 Batch A execution. It does not supersede any frozen
  product, architecture, evidence, gate or design baseline.

## Environment observed at execution start

Host fact used for planning, recorded so fixture provenance is auditable.

| Tool | Version | Note |
|---|---|---|
| rustc / cargo | 1.98.1 | matches the `1.98.1` baseline pin exactly |
| rustup host | `x86_64-pc-windows-msvc` | VS 2022 BuildTools present; `cargo build` links successfully |
| git | 2.55.0.windows.4 | |
| node | v24.19.0 | Node 24 LTS line |
| pnpm (installed) | 11.21.0 | below baseline pnpm 12 line; never used by the gate |
| pnpm (via corepack 0.35.0) | 12.6.0 | corrected at delivery: corepack resolves the version pinned in `apps/desktop/ui/package.json` (`packageManager: "pnpm@12.6.0"`), which is 12.6.0. The 12.7.0 first recorded here was what corepack would fetch with no pin, and no gate step ever used it |
| gcc (host, MinGW-W64) | 16.2.0 | produces PE/COFF on Windows, not ELF |
| GNU ld (host) | 2.47.20260726 | |
| arm-none-eabi-gcc | 14.3.1 (Arm GNU Toolchain 14.3.Rel1) | produces the real ELF fixtures |
| WebView2 / Edge runtime | 153.x / 154.x | present, so the desktop shell can run on this host |
| cargo-deny / cargo-fuzz / cargo-nextest | not installed | CI-side only; any local claim must be `NOT RUN` |

### Fixture consequence

`04_TECH/20_PLATFORM_SUPPORT.md` makes Windows the Tier 1 host, and the prompt's Fixture A
suggestion assumes a host-compiled ELF. On this host that assumption does not hold: MinGW gcc
emits PE/COFF, verified by `readelf` rejecting the produced binary's magic bytes. The real ELF
fixtures are therefore produced with `arm-none-eabi-gcc`, which is both a genuine GNU ELF output
and the closer analogue of embedded firmware. Fixture binaries are committed with full provenance
so parsing and golden tests do not require an ARM toolchain at test time; only regeneration does.

## Final validated HEAD

`PENDING — not yet determined. This field is filled only when the P0 final status is decided,
and must never be back-filled speculatively.`
