---
title: "M2 Object and Library Attribution"
doc_id: "FS-DEL-011"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE"
owner: "Engineering"
last_updated: "2026-10-15"
---

# M2 — Object and Library Attribution (plus M1 leftovers)

**Authorization:** the Owner asked on 2026-10-15 for a review of the proposed route, a complete plan, and then implementation following it. Base `6804ae7`. Branch `feature/m2-object-attribution`. Push only after an independent review and the Owner's approval.

## 1. Review of the proposed route

| Proposal | Verdict | Reason |
|---|---|---|
| Do not redo M1 or redesign the UI | **Agree** | M1 is integrated on `main` (`6804ae7`, CI #116 10/10). |
| Next: `.o` / `.a` attribution | **Agree, with a correction** | It answers the most common question ("which file or library grew?") and needs no vendor samples. The correction: this is **not** a surface-only task. The GNU ld MAP parser's object rows are not usable on real builds today (section 2), attribution reaches neither storage nor Compare, and Core states `object_attribution = Unavailable`. That makes it a Core + storage + capability change, which needs an ADR (AGENTS.md §2). |
| M1 open items | **Fold the cheap ones in** | The Overview Compare delta (F2) and the review residuals N1/N2 are small and belong to the same surfaces. Closing the native QA gaps (AC24/26/20) needs a human operator and a separate Windows profile; that is planned in section 6, not done by automation. |
| Keil/IAR | **Not now** | It still needs real vendor sample pairs (NEXT-46 rule). |

## 2. Measured gaps in the current code (`crates/firmwaresight-artifact/src/map.rs`)

1. **Wrapped input sections are lost.** With `-ffunction-sections`, GNU ld prints a long input-section name on its own line and `0xADDR 0xSIZE object` on the next. Only one-line rows are parsed.
2. **Absolute object paths are dropped.** Rows whose object contains `:` or starts with `/` are discarded (a fixture-privacy rule applied at parse time), so a normal Windows or Linux build loses nearly every row.
3. **Discarded input sections are counted.** Parsing does not start at `Linker script and memory map`, so the `Discarded input sections` block (gc-sections) would be attributed.
4. **No `COMMON`, `*fill*` or linker-stub handling**, and no output-section context.
5. **Nothing is persisted.** `pipeline.rs` hard-codes `object_attribution = Unavailable`, and `domain::diff::ObjectAttribution` always says unavailable.

## 3. Design (decided in ADR-0033)

- **Source:** GNU ld MAP rows after the memory-map banner only. Rows can be one-line or wrapped. Handling of special rows:
  - `COMMON` rows are attributed to their object.
  - `*fill*` padding and `linker stubs` are kept as named unattributed bytes.
  - Zero-size rows are ignored.
- **Identity, never a host path:**
  - An archive member reads `libname.a(member.o)`.
  - Object paths are made relative to the longest common directory of all objects in the MAP.
  - An absolute path outside that root (for example, a toolchain library) is reduced to its file name.
  - The display key is deterministic.
- **Accounting:** each contribution is charged like its output section in the existing ADR-0021 accounting. Flash counts if that section's nonvolatile charge is known and non-zero; RAM likewise. An unknown charge gives an Unknown contribution, never zero. Debug sections count toward neither.
- **Storage:** an additive migration `0007_object_contributions.sql`, one row per contribution, keyed by build. No existing table changes.
- **Capability:**
  - `object_attribution = Available` when a GNU ld MAP gave at least one attributed row.
  - `Unavailable` otherwise, with the reason.
  - Portable `analysis:1` and `diff:1` documents keep their schema; only the capability value changes. The affected goldens are regenerated deliberately.
- **Compare:**
  - Gives a per-object and per-library delta (old/new/delta for Flash and RAM, Added/Removed/Changed).
  - Available only when both builds stored attribution; otherwise Unavailable, with a reason that names the side.

## 4. Tasks and acceptance

| # | Task | Acceptance |
|---|---|---|
| A1 | Parser hardening | Unit tests: wrapped rows, `COMMON`, `*fill*`, stubs, discarded block skipped, archive members, Windows and POSIX absolute paths normalized, no absolute path in any record. Existing MAP tests unchanged. |
| A2 | Real fixture | `fixtures/elf/m2-objects/` built by arm-none-eabi-gcc 14.3.1 from committed sources with `-ffunction-sections`: three objects plus one static library. Generator script committed, manifest entry added, MAP contains no host path. |
| A3 | Core model | `ObjectAttribution` aggregates per object and per library; totals reconcile with the section footprint, and the remainder is reported. Pure, deterministic unit tests. |
| A4 | Storage | Migration 0007. Write on import, read back identical. A store migrated from 0006 opens with old builds reading "Unavailable". |
| A5 | Capability and goldens | Capability set from evidence; goldens regenerated with `update_goldens.py` and the diff reviewed. |
| A6 | Compare | Per-object and per-library delta with rules matching section changes; unit and IPC tests. |
| A7 | Desktop | Analyze tab "Objects & libraries" (bounded, sortable page: object, library, Flash, RAM, share). Compare "By object/library" (top growth plus table). Typed IPC; tokens only; Unknown never zero. |
| A8 | CLI | `fwsight analyze --objects` prints the per-object table in text mode. JSON output unchanged. |
| A9 | M1 leftovers | Overview shows the latest Compare delta (F2). N1: `Session::compare_export_guard` used by `export_now` and tested. N2: no "at least <0.1%". |
| A10 | Integration | `check.py` all steps, 0 SKIP; design checklist; docs (format support, CURRENT_STATE, NOW); one independent review; push on Owner approval. |

## 5. Not in M2

Keil/IAR MAP adapters, portable schema v2 (attribution inside `analysis`/`diff` JSON), DWARF-based attribution, treemaps.

## 6. Native QA plan for the open M1 items (needs the Owner)

The automated pass cannot supply three things: a person, a separate Windows profile and a real drag. Proposed session (about 45 minutes), on a fresh local Windows account, with the M2 build:
1. Drag an ELF and its MAP from Explorer.
2. Visual export (PNG/SVG/HTML): Save, OS-level No, app-level Keep, Replace, then a write failure into a read-only folder, then a retry.
3. English UI: the replace confirmation.
4. Tab / Shift-Tab across each page, recording the focus ring.
5. Narrator reading the five state words.
6. Windows at 1440×900 and 1024×720, light and dark.

The register's evidence format applies.

## 7. Status (2026-10-15)

| # | State | Commit |
|---|---|---|
| A1–A2 | Done | `1bd0f3c` |
| A3–A5 | Done; migration 0007 extended in A6 with the stored charge (unpushed, so still additive) | `15dcb7f` |
| A6 | Done | `f95b208` |
| A7 | Done | `50b8857` |
| A8 | Done | `7546086` |
| A9 | Done (F2, N1, N2) | `742edcd` |
| A10 | Gate and docs in this commit; independent review and push pending the Owner | — |

Found while building, for the Owner (not changed here):

1. `schemas/diff.schema.json`'s description of `objectChanges` still says "today available is false". D7 keeps the
   schema bytes unchanged, so the description is now stale; correcting a description is a schema edit and is left
   to an `ADR-0022` decision.
2. The `fwsight diff` text prints section and symbol names without terminal escaping (pre-existing; the new object
   lines are escaped). The analyze renderer escapes region names since `5f6a630`.
3. M2's desktop surfaces have unit, IPC and component tests but no native QA pass yet; section 6 applies to them too.
4. The first full `check.py` run failed `drift/fixtures tracked`: `**/target/` in `.gitignore` had hidden the fixture's `target/` side from 1bd0f3c. The local tests passed on the files on disk. Fixed in A10 with the same narrow exception P2 uses; the second run is the gate of record.
