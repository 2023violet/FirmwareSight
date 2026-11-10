---
title: "ADR-0042: The CLI may record an analysis in the local store"
doc_id: "FS-ADR-0042"
product: "FirmwareSight"
version: "0.8.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-11-09"
---

# ADR-0042: The CLI may record an analysis in the local store

## Status

**BASELINE** (2026-11-09). Batch D8, with ADR-0041 (`.ai/DECISIONS.md` F2). The plan named this record ADR-0041; see ADR-0041 for the renumbering.

## Context

- `fwsight` never writes to the store: `apps/cli` does not depend on `firmwaresight-storage`.
- A person who analyses builds from a terminal, a script or a build hook therefore has no history, and the desktop's Trends (ADR-0041) cannot see those builds.

## Decision

### D1. Opt-in, analyze only

- `fwsight analyze FILE [--map MAP] --record` stores the analysis after it succeeds. Without `--record` the CLI stays read-only, as before.
- `--store PATH` names another store file. It needs `--record`.
- Only `analyze` records in this batch. `gate` and `diff` do not write.

### D2. The same store, the same project

- With no `--store`, the store is the desktop's: `<data dir>/com.firmwaresight.desktop/firmwaresight-p0.sqlite`, where `<data dir>` is:
  - Windows: `%APPDATA%`;
  - macOS: `~/Library/Application Support`;
  - Linux and others: `$XDG_DATA_HOME`, else `~/.local/share`.
- This is the folder Tauri's `app_data_dir()` resolves for that identifier. It is resolved with `std::env` only; no new dependency.
- The build is filed under the desktop's local project (`local-desktop`, "Local analyses"), so it appears in the desktop's History and Trends.
- The snapshot is the same one the desktop would make (same `pipeline::analyze`), so the same files give the same snapshot id and the build is stored once.

### D3. Concurrency

- The store already runs in WAL mode with a 5-second busy timeout.
- A new storage call, `Database::record_snapshot`, checks for the snapshot and imports it inside one `IMMEDIATE` transaction. If the build is already stored it answers "already recorded" instead of failing on the duplicate key. The desktop uses the same call, so the desktop and a CLI importing the same build at the same moment store it once.

### D4. What is stored, and the output contract

- Exactly what the desktop stores for the same analysis (ADR-0041, migration 0007, `AGENTS.md` §7). That includes the artifact path, which the desktop also stores and which every reader reduces to a file name.
- Opening an older store migrates it, with the pre-migration backup the desktop already takes. A store written by a newer version is refused, as on the desktop.
- stdout does not change (`--json` stays one document). One line goes to stderr: `diagnostics: recorded build <id> in the local store`, or `… already recorded …`.
- A failed record is exit **6**, the existing "could not write an output" code, with the store's own code and message.
- The record happens after the file exports and before stdout, like the exports. So a failed record under `--json` leaves one document, the error, never the analysis followed by an error.

### D5. Dependency note (AGENTS.md §4)

- New dependency: `firmwaresight-storage` in `apps/cli`, an internal crate already in the workspace. It brings `rusqlite` (bundled SQLite) into the CLI binary, about 1 MB.
- Need: the user-visible record feature above. The standard library cannot write SQLite, and writing JSON files would break the storage rule (AGENTS.md §6).
- It does not enter the trusted Core: `firmwaresight-core` still has no SQLite.

## Consequences

- A terminal or script workflow builds the same history the desktop shows.
- The CLI binary grows by the bundled SQLite.
- Still local only: nothing is uploaded or synced; there is no shared or remote store.
