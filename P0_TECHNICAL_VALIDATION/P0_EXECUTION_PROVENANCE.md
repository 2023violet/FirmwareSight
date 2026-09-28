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
| Start HEAD to validated tree | `e086e98..65cb2dc`, 6 commits, all local |
| Commits after the validated tree | documentation only - run `git log --oneline 65cb2dc..HEAD` for the list, which is why no count is written here |
| Remote state | unchanged - `origin/main` stayed at `e086e98`; no push was authorized and none happened |
| Gate against that tree | `python scripts/check.py` -> 14/14 steps, 102 Rust tests, 19 UI tests, exit 0 |
| Cold rebuild against the Rust tree | `cargo clean` plus removing `node_modules/` and `dist/` -> 16/16 steps, exit 0, measured at `a014328`; no Rust file changed afterwards |

`65cb2dc` is named as the validated tree because the gate ran against exactly its contents with no
file edits in flight. One earlier gate run was discarded rather than reported: it started before a
test file was saved and finished after, so its output described a tree that never existed as a commit.

The commits after it change documentation only: they record this table and the `final_validated_head`
field in `BASELINE.yaml`, correct the `18 UI tests` figure that the accessibility fix made stale in
`.ai/CURRENT_STATE.md`, `.ai/DECISIONS.md`, `CHANGELOG.md` and `INDEX.md`, name the SHA256SUMS drift
entry by entry, and restate the source census to include the design checklist. Naming a SHA here would
otherwise require editing the commit it names, which cannot be done honestly, so the distinction
between the validated tree and the records that follow it is stated instead of hidden.

## Remediation round (2026-09-28)

The P0 prompt closed above. This round runs under a different authorization - *FirmwareSight P0 —
CI Closure / Cross-Platform Reproducibility Remediation v1.0*, supplied inline, so it has no stored
file and no SHA-256 to record here; its terms are in `.ai/DECISIONS.md` and
`P0_CI_REMEDIATION_REPORT.md`.

| Field | Value |
| --- | --- |
| Start HEAD, local | `f9b8ccb` (equal to `origin/main` at preflight, so no unauthorized delta) |
| Remote state that triggered it | Actions run `36360310447` on `f9b8ccb`: `failure`, 2 of 6 jobs green |
| Tooling added during the round | `cargo-deny 0.20.2` (`cargo install --locked cargo-deny --version 0.20.2`), the version CI pins. No other tool, and no dependency in either manifest |
| Frozen inputs that did not move | `Pillow==12.3.0`, Rust `1.98.1`, pnpm `12.7.0`, `assets/design-tokens.json`, the icon design, the fixture hash assertion |
| Desktop launches | two, both in the current user session, both `--release --features custom-protocol`; the second after the storage fix |
| Validated source tree | `ff9b34a` - the last commit that changes code or configuration. `python scripts/check.py` -> 14/14 with zero `SKIPPED`, and `--only core-smoke` -> 3/3. `cargo test --workspace` on the same tree reports 104 passed / 0 failed, and the UI group 19 |
| Commits after it | documentation only; run `git log --oneline ff9b34a..HEAD` rather than trusting a count written here, because the commit that writes a count is itself one of them |
| Push | not performed. The owner pushes; this round does not write `REMOTE CI PASS` |

`ff9b34a` is named as the validated source tree for the same reason `65cb2dc` was: the gate ran
against contents no different from its own. The documentation commits that follow it change no file
the gate reads - the drift group regenerates bindings, tokens and icons, and every one of those is
committed at or before `ff9b34a` - and the two reports they add cite the commands executed above, not
a rerun.

## Run #2 closure round (2026-09-28)

Authorization: *FirmwareSight P0 — Remote CI Run #2 Final Drift Closure v1.0*, supplied inline, so
there is no stored file to hash; its terms are in `.ai/DECISIONS.md` and
`P0_CI_RUN_2_CLOSURE_REPORT.md`.

| Field | Value |
| --- | --- |
| Start HEAD, local | `ebda52d63f22ff9f25786c1d80e53f5193cff183` |
| origin/main at preflight | the same SHA, after `git fetch --prune` - no delta to absorb |
| Working tree at preflight | `git status --short` empty; one worktree; no user modifications at risk |
| Run being remediated | `36378384225`, event `push`, head `ebda52d`, conclusion `failure`, 6 of 7 jobs green |
| How the run was read | `gh run view 36378384225 --repo 2023violet/FirmwareSight --json …,jobs` and `gh run view --job 108788787601 --log`, not from a report in this repository |
| Environment change | none. No tool installed, no dependency added, no manifest touched |
| Files changed | `.github/workflows/p0-check.yml` (21 added lines) plus governance and evidence documents |
| Validated source tree | `fe37847` - the commit carrying the workflow fix. `python scripts/check.py` -> 14/14, exit 0, zero `SKIPPED`; `--only core-smoke` -> 3/3; `cargo test --workspace` -> 104 passed / 0 failed; UI `Tests 19 passed (19)` |
| Not executed locally | the apt block itself - this host has no Ubuntu. The proven remotely-identical package list is the reason the change is expected to work, and Run #3 is the reason it would be known |
| Desktop launch | not repeated. No runtime source changed, so the previous round's evidence is carried forward |
| Push | not performed. The owner pushes; this round writes no `REMOTE CI PASS` |

`fe37847` is named as the validated source tree in the same sense as `65cb2dc` and `ff9b34a`: the gate
ran against contents equal to it. The documentation commits that follow change no file the gate reads.
