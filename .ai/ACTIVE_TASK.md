---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-09-29"
---

# ACTIVE TASK

```text
P2_COMPARE — the second MVP stage of the open-source MVP-first line ADR-0026 opened.

Authorization: "FirmwareSight — P2 Compare MVP Implementation, Execution Prompt v1.0 — Architect
Reviewed", supplied inline on 2026-09-29. It is registered in 10_AUDIT/SOURCE_PROMPTS/README.md without
a SHA-256, because no source file was delivered to this repository — recording a hash for bytes nobody
received would be a fabricated provenance record.

Status while this file is live: IN_PROGRESS. Nothing in this document claims P2 is complete.
```

## What the stage is

One product verb, `Compare`, taken from persisted snapshots to an inspectable, exportable diff:

```text
Analyze two builds → select Base / Target → Core diff → [Section | Symbol] changes → export JSON / HTML
```

`firmwaresight-core` owns every diff semantic. Storage retrieves normalized facts, the shell projects
them, and React paints them. No delta, match, ambiguity or comparability decision is computed in SQL,
in a Tauri command handler or in the frontend (`AGENTS.md` 3, prompt §7).

## Acceptance list

The frozen **US-002 Compare Builds** criteria in `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md:23-31`, all
five items, plus the PRD **P0-3 Build Diff** content list at `01_PRODUCT/01_PRD_MVP.md:44-53` including
its rule that a bare `+8 KB` is not an answer. P2 is not `PASS` while any applicable item is open.

| Source | What it binds |
| --- | --- |
| `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` US-002 | explicit old/new, signed delta on every numeric change, Added/Removed/Changed separate, ranking never hides a small important change, JSON and HTML export |
| `01_PRODUCT/01_PRD_MVP.md` P0-3 | nonvolatile/FLASH delta, RAM delta, section delta, symbol delta, object/module delta **only when evidence is sufficient**, added/removed/changed symbols, drill-down |
| `06_DELIVERY/08_MILESTONE_DELIVERABLE_MATRIX.md:21` | deliverables are build diff + contributors + evidence-aware change UI; exit evidence is deterministic diff tests |
| `DESIGN.md` §5 Diff Row, `03_DESIGN/04_COMPONENT_RULES.md` | every diff row shows old / new / delta, and absence is never `0` |

## Boundaries this round works inside

```text
Baseline            v0.6.0 — unchanged; no v0.7.0 without its own promotion prompt
Schema              SCHEMA_VERSION stays 2. No migration 0003, no diff_runs or diff_changes table:
                    a comparison is recomputable from stored snapshots
Dependency          none new. std / serde / rusqlite / tauri-plugin-dialog / React / ts-rs only
Design token        none new. diff.added / diff.removed / diff.changed exist at v0.2.1; a
                    delta-direction token is forbidden by 03_DESIGN/08:31-33, not missing
IPC               six use-case commands, bounded: list_compare_candidates, compare_snapshots,
                    query_section_changes, query_symbol_changes, export_compare_json,
                    export_compare_html. No run_sql / read_table / read_file / write_file / get_any_path
Payload             summary is bounded by construction; detail pages default 100, hard max 500, clamped
                    in Rust. The full diff never crosses IPC by default
Diff session        opaque cmp-<pid>-<counter>, session-local, not persisted, not portable, not part of
                    any deterministic diff identity
Evidence            Unknown is never zero; Added is never 0 → N; partial evidence is labelled partial;
                    missing MAP degrades the memory diff visibly instead of blocking it
Attribution         object/module reports Unavailable with its real reason. No empty Object tab
Time                builds.created_at is import time and is labelled Imported, never Build time
Privacy             no host path in IPC or UI; the save dialog opens Rust-side and the chosen path does
                    not come back
```

`AGENTS.md` 2 / 3 / 6 / 7 / 8 / 11 apply unchanged, and `ADR-0026` relaxed no technical boundary.

## Start facts, measured before the first write

```text
HEAD = origin/main      7a13660db873439f66eedee850561e8dae1cb3cf
worktree                clean (git status --short empty; git diff and git diff --cached empty)
Run #16                 36576568426 on 7a13660 — completed, success, 7 of 7 jobs
                        read with gh run view 36576568426 --repo 2023violet/FirmwareSight
Rust tests at start     184 passed / 0 failed / 0 ignored   (cargo test --workspace)
UI tests at start       58 passed                            (corepack pnpm test)
cargo-deny              0.20.2 installed, so deny/cargo-deny executes rather than taking the SKIPPED path
Fixture toolchain       arm-none-eabi-gcc 14.3.Rel1 on PATH, needed only to build the new P2 pair
```

## Stop condition

```text
STOP AFTER P2.

P3 Release Gate is the next authorizable stage and is NOT authorized by this file: it needs its own
architect prompt. No Gate rule, no Release Bundle, no History, no Project Settings, no installer,
signing, updater, SBOM, cloud, account, telemetry, AI, pricing or commercial work.
```

Three existing tests assert the absence of what this stage ships, and each is changed by name rather
than quietly deleted: `apps/cli/src/main.rs` `unregistered_future_commands_are_not_accepted` (drops
`diff`, keeps `gate` / `release` / `watch` / `doctor`), the same file's exit-code surface test (adds `6`
for export failure, keeps `4` / `5` unreachable), and `apps/desktop/ui/src/intake.test.tsx`'s
"no navigation, no Compare text" guard, which prompt §53 replaces with the Analyze + Compare
navigation assertion.
