---
title: "AI Entry Point"
doc_id: "FS-AI-001"
product: "FirmwareSight"
version: "0.6.0"
status: "CURRENT"
owner: "Engineering"
last_updated: "2026-10-14"
---

# AI Entry Point

## Required reading, in order

1. [../AGENTS.md](../AGENTS.md): hard rules.
2. [NOW.md](NOW.md): live status, gates and open risks.
3. [ACTIVE_TASK.md](ACTIVE_TASK.md): the only work you may do. If it says `NONE`, do not start product work.
4. [DECISIONS.md](DECISIONS.md): the ADR index and standing Owner rules.
5. [CURRENT_STATE.md](CURRENT_STATE.md): product, formats, platforms, baseline.
6. The ADRs and specs the task names.

For UI work, also read [../DESIGN.md](../DESIGN.md) and [../assets/design-tokens.json](../assets/design-tokens.json).

## On demand only

- [archive/2026-10-14/](archive/2026-10-14/INDEX.md): the full dated history of these files up to 2026-10-14.
- [EVIDENCE_LEDGER.md](EVIDENCE_LEDGER.md): an evidence summary by round.
- `*_VALIDATION/`, `NEXT47_VALIDATION/`, `10_AUDIT/`: per-stage evidence and source prompts.
- [../BASELINE.yaml](../BASELINE.yaml): the baseline ledger.

## Keeping these files short

- `NOW.md`, `CURRENT_STATE.md`, `ACTIVE_TASK.md` and `DECISIONS.md` are overwritten, never appended. Size limits: `NOW.md` ≤ 4 KB, `CURRENT_STATE.md` ≤ 8 KB, `ACTIVE_TASK.md` ≤ 4 KB, `DECISIONS.md` ≤ 12 KB.
- When something stops being current, move it to a new dated folder under `archive/` rather than leaving it in a live file.
- Every statement keeps its evidence label from AGENTS.md §8 (Observed / Derived / Declared / Unknown) and links to its source.

## Not allowed

- Picking tasks from the roadmap on your own.
- Replacing the technology stack because something is "more modern".
- Adding cloud, AI, an account system, telemetry or network features.
- Widening format support claims without fixtures.
- Changing a baseline decision without an ADR.
