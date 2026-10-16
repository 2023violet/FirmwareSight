---
title: "Human Native QA Runbook"
doc_id: "FS-EV-D2C-002"
product: "FirmwareSight"
version: "0.6.0"
status: "RUNBOOK_NOT_RUN"
owner: "QA"
last_updated: "2026-10-16"
---

# Human native-QA runbook (D2C C3 → C8)

**Nothing in this file has run.** It is the script for the human pass that closes the native risks named in `.ai/NOW.md` before any external Developer Preview, Pilot handout, Private Beta or RC (`NEXT47_VALIDATION/evidence/04_NEXT47_DEFERRED_NATIVE_QA.md`, forced deadline).

## Who, where, what

- **Operator:** a person, not an automated driver. Record their name outside Git and a role label inside it.
- **Machine:** a dedicated non-Owner Windows account. Before launch, prove that data, config, cache and temp all resolve inside that account. Owner data, real projects and credentials must be unreachable.
- **Build:** one installer from a named `main` commit. Record the commit, the binary SHA-256, and the installer SHA-256 with its CI origin. A rebuilt binary starts a new pass.
- **Inputs:** repository fixtures only: `fixtures/elf/p2-diff/{base,target}`, `fixtures/elf/m2-objects/{base,target}`, plus a synthetic writable folder and a synthetic non-writable folder (prove both).

## Cases

The detailed steps for AC24, AC26 and AC20 are in the deferred register (C1 steps 1–14, C2 and C3) and are not repeated here. This list adds what M1 left unobserved (`evidence/M1/README.md`, "Not shown") and the M2 screens.

| # | Case | Closes | Pass means |
|---|---|---|---|
| H01 | Visual export (PNG, SVG, HTML) from Analyze, Compare and Gate: cancel, deny overwrite at the OS prompt and at the app prompt, confirm overwrite, write into the **non-writable folder**, retry | AC24 gaps | Sentinel bytes unchanged on cancel, deny and failure; exactly one target replaced on confirm; the error names a diagnostics id; retry succeeds |
| H02 | Compare JSON/HTML: compare again **while the replace confirmation is open**, then press Replace (M1 review F1, fixed but never observed) | AC24 | Refused as stale; sentinel unchanged |
| H03 | English UI: open the replace confirmation | AC20 | Title, body and buttons in English, with Keep and Replace distinct |
| H04 | A loading state in each language (large ELF or slow disk) | AC20 | Loading text localized; no English leak |
| H05 | Tab / Shift-Tab across every page, recording the order and the focus ring at each stop; Esc on each dialog | AC26 | Order matches the visual order; the ring is always visible; focus returns to the opener |
| H06 | Narrator: read the five state words, a table header row, and an UNKNOWN badge with its note | AC26 | Each read as icon meaning + word; no state carried only by colour |
| H07 | 1440×900 and 1024×720, Light and Dark, each language, on Analyze, Compare, Release and Overview | AC26 | No sideways page scroll; table scrollbars reachable; no clipped state words |
| H08 | Drag an ELF and its MAP together from Explorer onto the window | M1 C25 (NOT_RUN) | One Analyze runs; both files named; no path shown |
| H09 | Analyze `m2-objects/target` with its MAP → Objects tab: sort by Flash, RAM and name; filter `libdrv`; page | M2 | Rows match `fwsight analyze --objects`; shares add up; "not attributed" line shown |
| H10 | The same ELF without a MAP → Objects tab | M2 | UNKNOWN badge, the no-MAP reason, the attach-MAP next step; no table of zeros |
| H11 | Compare `m2-objects` base → target → "By object and library": switch to Libraries, click a shortlist entry | M2 | `libdrv.a(spi.o)` Added with base "Not present"; the drill-down filters the table |
| H12 | Overview after H11: "Latest comparison" → Open Compare | M2 / M1 F2 | Shows the same pair and the Flash change as Compare |

## Evidence (Standing Owner rule 4)

- Commit a short summary and an AC matrix (expected, actual, verdict, UTC, operator role). Put screenshots and the screen recording in a hashed ZIP, with local user names and paths stripped. Raw logs stay outside Git.
- A case that failed and was re-run keeps its first result next to the re-run.
- Only observed cases change state. An unrun case stays `NOT_RUN`; a case shown only in part is `PASS_WITH_LIMITS`, with the missing part named.
- Stop and file a defect on any data loss, any wrong overwrite, any leak of a path or a name, or a keyboard trap. The fix gets a new SHA and the case is re-run on it.
