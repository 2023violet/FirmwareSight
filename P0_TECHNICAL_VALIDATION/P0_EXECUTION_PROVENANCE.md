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
| pnpm (via corepack 0.35.0) | 12.7.0 | the version the frozen `examples/package.baseline.json` names, and what `apps/desktop/ui/package.json` pins at delivery. It briefly read 12.6.0 mid-track; see `P0_DEPENDENCY_REPORT.md` |
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

The tree the delivered gate result belongs to:

| Field | Value |
| --- | --- |
| Final validated HEAD | `65cb2dc` |
| Commits in this track | `b618900`, `ff35bb2`, `2749cbf`, `725e7cd`, `a014328`, `65cb2dc` |
| Start HEAD to final | `e086e98..65cb2dc`, 6 commits, all local |
| Remote state | unchanged - `origin/main` stayed at `e086e98`; no push was authorized and none happened |
| Gate against that tree | `python scripts/check.py` -> 14/14 steps, 102 Rust tests, 19 UI tests, exit 0 |
| Cold rebuild against the Rust tree | `cargo clean` plus removing `node_modules/` and `dist/` -> 16/16 steps, exit 0, measured at `a014328`; no Rust file changed afterwards |

`65cb2dc` is named as the validated tree because the gate ran against exactly its contents with no
file edits in flight. One earlier gate run was discarded rather than reported: it started before a
test file was saved and finished after, so its output described a tree that never existed as a commit.

The commit that records this table is its child and changes documentation only - this table, the
matching `final_validated_head` field in `BASELINE.yaml`, and the `18 UI tests` count in
`.ai/CURRENT_STATE.md` / `.ai/DECISIONS.md` / `CHANGELOG.md` / `INDEX.md` that the accessibility fix
made stale. Naming a SHA here would otherwise require editing the commit it names, which cannot be
done honestly, so the distinction between the validated tree and the commit that notes it is stated
instead of hidden.
