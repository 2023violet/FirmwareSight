---
title: "ADR-0039: Composition tree and growth waterfall"
doc_id: "FS-ADR-0039"
product: "FirmwareSight"
version: "0.8.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-11-09"
---

# ADR-0039: Show where the bytes are (composition tree) and where growth came from (waterfall)

## Status

**BASELINE** (2026-11-09).
- This is batch D7, authorised by the Owner's written exception for the "too monotonous" round (`.ai/DECISIONS.md` F1, F5).
- It builds on ADR-0021 (memory accounting), ADR-0033 (object attribution) and ADR-0034 (the design language).

## Context

- Every fact the product knows is shown as a table. The Analyze page lists sections, symbols and objects separately, and nothing shows how they nest or what share of a budget each one takes.
- In Compare, the Summary ranks the largest movers but does not reconcile them: a reader cannot see that +84 bytes of Flash is imu.o +54, main.o +24 and 6 bytes of the linker's own.
- A held `BuildSnapshot` already carries everything a nested view needs: the footprint with per-section charges, the region usage, the object rows and the symbols. A diff already carries per-object `ByteChange`s and the total deltas.

## Decision

1. **The composition tree is Core's** (`firmwaresight-core::domain::composition`). The UI lays it out and draws it, and never computes a byte.
   - **One tree per budget side**: Nonvolatile (Flash, load image) or Runtime (RAM). Its levels are root → region → output section → object → symbol.
   - **Regions come from `MemoryFootprint::region_usage`.** Each section is placed in the region of the matching kind that lists it.
     - On the Nonvolatile side, a region of kind Nonvolatile; on the Runtime side, VolatileRam.
     - A section that no such region lists goes under one node, `(no region)`.
     - Without a region table, the root's children are the sections directly.
   - **A section's bytes are its side charge** (`MemoryContribution`). A section charged 0 on this side is not a node.
   - **An object's bytes come from `attribution::charge` of each of its MAP rows** in that output section, summed per object key. A row with Unknown charge makes the object Unknown.
   - **A symbol's bytes are its ELF size.** A symbol sits under the object row whose address range holds it, and is attached only when the row's charge on this side equals the row's size (an ordinary row). Under a compressed or Unknown row, symbols are not attached, because their share of the stored bytes is not known.
2. **The tree is conserved: nothing is silently dropped.**
   - At every level, when the parent and all children are known, the gap becomes an explicit child: **linker bytes** under a section (fill, stubs, rows with no object), **(no symbol)** under an object.
   - If the children add up to more than the parent (which a malformed input could cause), the parent keeps its own value, and the node says why.
   - Every node carries its evidence class:
     - Observed for section charges and object rows;
     - Derived for the computed gap nodes and for symbols matched by address;
     - Unknown with a reason.
3. **Unknown is never drawn as zero.**
   - An Unknown node has no area of its own. It is listed under its parent as a hollow, dashed entry with its reason.
   - The parent's area stays its known value.
4. **The growth waterfall is Core's** (`waterfall::growth_waterfall`, over a `DiffResult`). It is defined for a side and for objects or libraries:
   - **Steps:** base total → one step per object, ordered by |Δ| descending, at most `top_n`.
   - The rest fold into one step, **other N objects**. A final reconciling step is **linker and unattributed**: total Δ minus the sum of the object Δs, when every term is known.
   - The steps end at the target total.
   - Added and removed objects keep their change kind. An added object's step is its whole target size and a removed one's is minus its whole base size: absent is not zero as a value (the table still shows "—"), but it is as a share of a total.
   - A step whose Δ is Unknown breaks the arithmetic, so the reconciling step becomes Unknown with the reason. The two totals are still shown.
5. **No persisted shape changes.**
   - The analysis, diff, gate and release JSON schemas are unchanged, and so is storage.
   - The tree and the waterfall go only to IPC and to the visual and Markdown exports.
   - A build is only explorable while its snapshot is held (analysed in this session). A build that is not held says so and offers to analyse it again.
6. **Payload bounds.**
   - IPC returns at most two levels below a focus node, with at most 200 children per node, sorted by bytes. The rest fold into one **N more** node that names its total.
   - A symbol table of 100k entries therefore never crosses the bridge at once (`04_TECH/13`).

## Consequences

- **Analyze** gets a Composition view, the second tab after Memory (Memory stays the opening view):
  - a treemap with drill-down and breadcrumbs, keyboard navigation, and the Inspector for any node;
  - Flash/RAM switch.
- **Compare** gets a growth waterfall in its Summary, with an objects/libraries switch.
- **Exports (D7):**
  - the analysis Markdown from the desktop gains a **Composition** section: per budget, the five largest parts at each level (regions, sections, objects) with their share of the line above;
  - the comparison Markdown (desktop and `fwsight diff --markdown`) gains **Where the change came from**: the waterfall per budget, by object;
  - `fwsight analyze --markdown` is unchanged, because the CLI's analysis export does not take the composition yet.
- **Deferred:**
  - a treemap and a waterfall scene in the SVG/PNG/print-HTML exports. They need a second layout in the report crate and would rewrite every visual golden; they get their own change;
  - CLI flags for the composition, which come after D9 (CI integration) has landed its CLI changes, to keep the two batches apart.
- **Colour:**
  - the treemap uses the `chart.blue` ramp by depth;
  - the waterfall uses neutral bars with ▲/▼ glyphs and text, so growth is not judged by colour (ADR-0034);
  - status colours are not used.

## Alternatives rejected

- **Layout in Rust:** the squarified layout is presentation, depends on the viewport and is cheap. Doing it in TypeScript keeps Core free of geometry and the bridge free of rectangles.
- **Sunburst:** angles are harder to read than areas, and long names do not fit.
- **A chart library:** each one is a new dependency for two simple shapes, and none follows the design tokens.
