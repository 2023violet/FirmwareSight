---
title: "ADR-0041: Per-region usage in the store, and History trends"
doc_id: "FS-ADR-0041"
product: "FirmwareSight"
version: "0.8.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-11-09"
---

# ADR-0041: Per-region usage in the store, and History trends

## Status

**BASELINE** (2026-11-09).
- This is batch D8, authorised by the Owner's written exception for the "too monotonous" round (`.ai/DECISIONS.md` F2).
- The plan named this record ADR-0040. D9 took that number (ADR-0040), so D8's two records are ADR-0041 (this one) and ADR-0042 (the CLI writes to the local store).
- It builds on ADR-0021 (two budgets), ADR-0033 (object contributions, migration 0007), ADR-0034 D7 (the pinned baseline) and DR1 (no silent backfill).

## Context

- The store already keeps, for each build, its two budget totals (`memory_footprints`) and its MAP object rows (`object_contributions`, since migration 0007).
- It does **not** keep per-region usage. The analysis computes it (`MemoryFootprint::region_usage`: capacity, used, state, risk per region), shows it, and then forgets it. So History cannot say "RAM headroom shrank from 40 % to 6 % over the last ten builds".
- History is three tables of rows. There is nothing that shows the store as a sequence over time.

## Decision

### D1. Migration 0009: two additive tables

- `region_usage_sets(build_id PK, layout_source, region_count)`: one row for **every** build imported by a version that has this migration, even when the build had no region table (`region_count = 0`).
- `region_usage(build_id, ordinal, name, kind, origin, capacity_bytes, used_bytes, state, risk, evidence_source)`: one row per region, in the analysis's own order.
- Why two tables: they keep apart three answers that one table would merge:

  | What the store has | What it means | Shown as |
  |---|---|---|
  | no `region_usage_sets` row | stored before this version | **Unknown (stored before this version)** |
  | a set row with `region_count = 0` | the build had no region table (no MAP, no layout) | **Not available (no region table)** |
  | region rows | recorded usage | the numbers |

- `capacity_bytes` and `used_bytes` are NULL when Unknown, never 0.
- No host path is stored: a region name is the linker's region name.
- Rows are immutable and follow their build (`ON DELETE CASCADE`), like every other stored fact.
- **No backfill (DR1).** A build stored before 0009 is not re-analyzed or re-derived, even though its artifact may still exist; it says Unknown.

### D2. The trend read

- `Database::history_trend(project_ids, limit)` returns the newest `limit` COMPLETE builds (default 100, ceiling 500), oldest first, each with:
  - its stored budget totals, with their stored state;
  - its region usage set, or "not recorded";
  - the newest Gate disposition stored for it, and the release versions recorded for it;
  - whether it is the project's pinned baseline;
  - the per-object totals summed from `object_contributions` (whether the build has object rows at all is part of the answer).
- Order is the import order: `builds.created_at`, then SQLite's insertion order, so two builds stored in one second keep their order.

### D3. Core decides the series

`firmwaresight_core::domain::trend` is pure and synchronous:
- **Budget series:** each point's value with the delta from the previous point whose value is known. An Unknown point has no value and no delta; it is never interpolated or drawn as 0.
- **Region series:** the union of region names in first-seen order. Each point is Recorded (capacity, used, remaining), Absent (the build recorded regions but not this one), No region table, or Not recorded. The latest headroom is read from the newest Recorded point.
- **Object movers:** the objects whose bytes moved most (largest max − min) across the builds that have object rows. An object missing from such a build is 0 there, because the MAP lists every object the linker placed. A build without object rows contributes no value (a gap), never 0.

The UI lays this out and draws it. It computes no delta, headroom or ranking (AGENTS.md §3).

### D4. History · Trends

- A new first tab, **Trends**:
  - Flash and RAM line charts over the builds, with a gap at each Unknown point and markers for the pinned baseline, release records and the stored Gate verdict (icon + label in the point's details);
  - one small chart per region with its capacity line and the latest headroom in words;
  - the five biggest object movers as sparklines, with first → last values.
- A point is a button. Activating it opens the Inspector with that build's stored facts. Selecting two points and **Compare** opens Compare on that pair, the same action the Builds tab offers.
- Charts are hand-written SVG with token colours only (no chart library, F5).
- **The headroom line is the region capacity, not a `firmwaresight.toml` budget.** History is the store of the desktop's local project, which has no policy file. The plan's "budget line from `[memory]`" would draw a number History does not own. Policy budgets stay on the Release Gate page.

### D5. Overview after a restart

- When this session has analyzed nothing yet, Overview shows the **most recently stored build** from the same trend read: its file name, Flash, RAM, the stored Gate verdict and when it was stored, labelled as history, with a button to History.
- It is a stored fact, not the "current build". It never feeds the ship question, which still needs an analysis in this session.

## Consequences

- One additive migration; `SCHEMA_VERSION` becomes 9. Older builds keep reading.
- Trends make the store useful to someone who did not do the analyses.
- Region capacity, not policy, sets the headroom drawn in History.
- The CLI can feed the same store (ADR-0042), so a CI or a terminal workflow builds the same history.

## Not decided here

- Per-region or per-library budgets and thresholds (F4, next round).
- Pruning or archiving old builds.
