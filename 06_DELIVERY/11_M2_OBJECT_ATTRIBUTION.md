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

## 8. Independent review of 3eae003 and what changed

The review verdict was **FAIL**, with one S1 item. It was reproduced with the CLI. The review ran `cargo test --workspace` (1223 passed) and `vitest` (428 passed).

| Finding | Disposition |
|---|---|
| **S1** A host path survived as the object key when the build's objects shared no common directory (a toolchain `crti.o` beside the project; case-different drive letters; one drive with `Program Files`) | **Fixed.** `map.rs` `normalize_contributions`: absolute and `..` paths are cut at the shallowest directory that itself holds an object of the link, so only directories below an object directory survive. Five regression tests cover the reviewer's three reproductions, an empty common root, and a same-name collision. |
| S2.1 A build stored before 0007 never gains rows, because re-analysis reuses the stored snapshot | **Owner decision, not changed.** The options are a backfill on re-import (an additive INSERT; rows stay immutable) or keeping the current behaviour. The page now says which absence it is (next row), so it no longer tells such a user to do something that cannot help. |
| S2.2 Two objects with one file name merged silently | **Fixed** by S1's collision rule (`app/main.o`, `boot/main.o`). |
| S2.3 No A4 test (store migrated from 0006 reads Unavailable) | **Fixed.** `a_build_stored_before_0007_reads_unavailable_after_the_migration`. |
| S2.4 One generic reason instead of D6's distinct ones | **Fixed for what storage can tell.** `snapshot_has_map` separates "no MAP" from "a MAP but no rows", with a different next step for each. "Stored before 0007" and "a MAP without input sections" stay one reason, because nothing stored tells them apart. |
| S3 Remainder dropped a known side when the other was Unknown | **Fixed** in the CLI and the UI, which now show each side on its own. |
| S3 Misplaced doc comment; stale "ten" assertion message | **Fixed.** |
| S3 Diff reason overstated what the CLI text lists | **Fixed**: counts and largest changes. Goldens regenerated (reason text and dependent digests only). |
| S3 O(rows × objects) aggregation, recomputed per page | **Not changed.** Fixture-scale builds have tens of objects. Recorded for when a large real MAP is measured. |
| S3 Attributed bytes above the total make the remainder Unknown with no signal | **Not changed.** Unknown is the honest value. A diagnostic is a follow-up. |
| S3 Per-object comparability is always `Exact` | **No change, by reasoning.** A row's charge is either a MAP-stated size or Unknown, and Unknown already yields no delta. A partial budget total does not make one row's MAP size a floor. If the Owner reads D8 more strictly, this is one line in `diff.rs` `usage_change`. |

### 8.1 Re-review of aa00d2b

The verdict was **FAIL** again. The three original reproductions, UNC paths and `..` paths passed. The re-review ran `cargo test --workspace` (1229 passed), `vitest` (430 passed) and 13 constructed MAPs through `fwsight analyze --objects`.

| Finding | Disposition |
|---|---|
| **S1** The collision fallback kept a full host path when two same-named objects shared no prefix (`C:/Users/alice/.../main.o` and `D:/sdk/obj/main.o`) | **Fixed.** A collision never borrows directories: the first object in MAP order keeps the key and later ones become `key#2`, `key#3`. A final guard reduces any key that still reads as a path (drive, leading `/`, `\`, `..`) to its file name. |
| S2 Same-named objects differing only above the project (`/home/alice/projA`, `/home/bob/projB`) kept the user directory | **Fixed** by the same rule: `main.o`, `main.o#2`. |
| S2 An object at a drive root or `/` became an anchor, so other keys kept `Users/alice/...` | **Fixed.** A directory anchors only at depth two or more. |
| S2 A relative and an out-of-tree object with one name merged silently | **Fixed.** Collisions are checked across all rows; distinct objects are told apart by the text the linker printed. |
| §8's S2.2 row overstated the earlier fix | Superseded by this table. In-project same-name objects under absolute paths now read `main.o` / `main.o#2` rather than `app/main.o` / `boot/main.o`: readability is traded for a guarantee. Relative-path builds keep their directories. |

Six tests cover these cases (end of `map.rs` `attribution_tests`).

### 8.2 Second re-review of 86bcbc4

The verdict was **PASS_WITH_FINDINGS**, with no S1. All 19 constructed MAPs were clean, and `cargo test --workspace` passed 1234 tests. The findings were fixed in the next commit:

| Finding | Disposition |
|---|---|
| S2-1 A drive letter or UNC `server/share` counted as a directory level, so an object directly in `C:/Users` or a share root anchored the others | **Fixed.** `root_len` counts the drive, the UNC `server/share` and the `\\?\` prefix as root, not depth. One test covers drive, UNC and device paths. |
| S2-2 `#N` follows link order, so Compare could pair two unrelated `main.o#2` files | **Fixed with the reviewer's option 3**, the existing section rule: a name that repeats in either build (`main.o`, `main.o#2`) is left unpaired as Removed and Added with `ambiguous`, never Changed, and never ranked as growth. The desktop row says why, and the CLI text counts such rows. Options 1 (borrow a trailing directory below a qualifying anchor) and 2 (number by sorted path) remain open for the Owner if paired same-name objects are wanted. |
| S3 A generated suffix could take a real file's name (`main.o#2`) | **Fixed.** Generated suffixes skip every key another row already has. Covered by a test. |
| S3 §8.1 did not state the Compare effect | Stated above. |

### 8.3 Final check of 18a0f63

*Outcome: pushed to `main` as `04d800c` on Owner approval (Actions #117, 10/10); `ADR-0033` BASELINE. Next steps: `06_DELIVERY/12`.*

The verdict was **PASS_WITH_FINDINGS**, with no S1 or S2. The ambiguity rule matches D8, checked end to end by `fwsight diff` with swapped `main.o` / `main.o#2`, which gave 0 changed, 2 added, 2 removed and "4 row(s) left unpaired". `cargo test --workspace` passed 1237 tests. `vitest` passed 430 in three of four runs; the first run had one unidentified failure while `cargo test` loaded the CPU, which is the known UI race class (`L23`) and not attributed to this commit.

| Finding | Disposition |
|---|---|
| S3-1 Ambiguous rows appeared in "largest added objects" | **Fixed.** `largest_added_objects` skips `ambiguous` rows. Tested. |
| S3-2 Two same-named archives in different directories merge in the Libraries view (members stay distinct via `#N`) | **Not changed; rare.** Recorded for the Owner. |
| S3-3 The desktop added/removed counts include ambiguous rows without saying so (the CLI says so) | **Not changed; small.** Each such row carries its note in the table. |

