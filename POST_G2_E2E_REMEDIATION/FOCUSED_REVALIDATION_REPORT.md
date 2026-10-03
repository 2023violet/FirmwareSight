---
title: "Post-G2 E2E Remediation — Focused Real-Desktop Re-Validation"
doc_id: "FS-POSTG2-REVAL"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "POST_G2_E2E_FINDINGS_REMEDIATION"
owner: "Engineering"
last_updated: "2026-10-02"
---

# Focused real-desktop re-validation

Not a re-run of the 283 cases. Five things were checked on the shipping binary with real mouse, real
keyboard and native dialogs: the two fixed findings, the third one included by gate, one happy-path
journey, and error recovery around the affected flows. No IPC call was substituted for a click, and no
browser was involved.

```
binary   target\release\firmwaresight-desktop.exe
sha256   ca2a4cdb69a603a1eb5df17832dca3a216924c4d0e8db20eb027298033c137a0
size     14,994,944 bytes
built    cargo build --release -p firmwaresight-desktop --features custom-protocol, exit 0
window   1056 x 799 (client 1040 x 760), 100 % DPI, one product process at a time
```

The E2E round's binary `84a26a05…` was overwritten by this build and appears nowhere in this phase.
Its screenshots appear as the "before" half of two comparisons, taken of the same files at the same
paths on this machine.

## Store isolation

The previously owner-authorized method, reused unchanged: with zero product processes running, hash the
owner's three store files, park them under `.postg2-e2e-remediation-backup`, keep an independent hash
record, let the app open a fresh store, run, close the app, restore, verify byte-exact. The real
database was not migrated and no earlier database was deleted.

```
unchanged before and after:
  firmwaresight-p0.sqlite       155,648  d6e41034d9a7fdcdf5e72b4621ca3c8445c79dbbf7fd9aab70f5bf6bca836468
  firmwaresight-p0.sqlite-wal           0  e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
  firmwaresight-p0.sqlite-shm    32,768  fd4c9fda9cd3f9ae7c962b0ddf37232294d55580e1aa165aa06129b8549389eb

this round's test store, 105,598,976 bytes (the 51 MB synthetic analysis): retired, not deleted.

ORIGINAL_DB_RESTORED = YES
ORIGINAL_DB_SHA_MATCH = YES
```

## Happy path, one journey (§19)

`journey.py 1` drove Analyze → Compare → Gate → Bundle → close through the native pickers, each step
verified against the screen, the store read only to corroborate:

```
launch                   pid=24896 visible_ms=631
base / target analyzed   px_changed=39686 / 83459
compare rendered         px_changed=44334
gate with 2 Down         PASS, row stored: sev=PASS gate=a17e9861
bundle on disk           brake-node-1.2.3-974f661d3a6d
bundle verified          tree=3f2b489eb871ed08 files=10
```

The bundle was then read by a tool that uses no FirmwareSight code — `scripts/verify_bundle_portability.py`
— and **59 of 59 checks passed**: every digest in `SHA256SUMS` and in the manifest recomputed and
matched, the five JSON documents satisfied their own contract urns, the report is self-contained, no
host path and no wall-clock value appears in anything written, and all nine "what can a stranger answer
from the bundle alone" questions were answerable. Gate rules, the PASS disposition and the 10-file
payload are unchanged from the round that closed G2.

## E2E-F001 (§17) — three fresh sessions

Two artifacts sharing the leaf name `firmware.elf`, chosen through the native picker. Per run: analyze A,
choose B and press nothing, analyze B, then choose an unparseable file and analyze. The marker is
measured as the pixel count of its own card background token `(241, 243, 245)` in the content column,
which is position-independent — the note moves down the page when an error panel is printed above it,
and that is one of the states that has to keep working.

| run | A, no note | B, note expected | C, analyzed | D, failed analyze | last-good report survives D | closed cleanly | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 0 | 38,462 | 0 | 164,582 | 4,249 ink | yes | PASS |
| 2 | 0 | 38,462 | 0 | 164,595 | 4,240 ink | yes | PASS |
| 3 | 0 | 38,462 | 0 | 164,575 | 4,248 ink | yes | PASS |

**3 / 3.** The pending screen reads:

> Previous analysis of firmware.elf. The selection now in this row is a different file with the same
> name, and it has not been analyzed yet.

above A's still-current SHA-256 and `16,820 bytes`, with `firmware.elf   MAP: Not provided` in the
selection row. After analyzing B the card is gone and the facts are B's. After the failed analysis the
error card (`ERR-FORMAT-0001`, `unsupported format: Binary at random-4096.bin`) sits above
"Previous analysis of firmware.elf. It is not an analysis of random-4096.bin: that selection has not
been analyzed yet." with the last-good report still below it — the failed-analysis semantics preserved
on the real screen.

## E2E-F002 (§18) — three iterations of both destination cases

Enabled/disabled is measured, not read: the same label carries 57 dark pixels plain and 174 enabled.

Case 2, a folder this engine wrote — **3 / 3**, identical in all three iterations:

```
export_ink 174 (enabled)   first press -> ERR-BUNDLE-6106 + the Replace card
Keep it (located by its label ink) -> tree_after_cancel == tree_before == 3f2b489eb871ed08
second press -> Replace -> 10 files, bundle valid
```

Case 3, a foreign directory holding `DO_NOT_DELETE.txt` — **3 / 3**, identical in all three:

```
export_ink 57            Export disabled
replace_offered false    no Replace action anywhere on the screen
press_opened_dialog false, press_offered_replace false
canary sha 9197d76cae958857 unchanged; entries_after == entries_before == ['DO_NOT_DELETE.txt']
guidance_ink 557, guidance_band_changed 3,187
recovery: choose a clean parent -> export_ink 174, bundle written, 10 files
```

The sentence on screen:

> brake-node-1.2.3-974f661d3a6d is already there, and it is not a bundle this engine wrote, so nothing
> will be replaced and Export is disabled. Choose another destination folder to export.

No foreign directory was deleted at any point, so the round's E2E-S0 condition never triggered.

## E2E-F003 (§16 C) — the two MAPs that were refused, and the one that already worked

The MAP is attached the way the Tier B journeys attached it: one Tab from the row the artifact picker
closed, then Enter, then the native `Choose the linker MAP for this artifact` dialog. A fixed coordinate
cannot work — the control's x position is set by the width of the file name and the `MAP:` text in front
of it. The discriminator is what the defect wrote: a refused MAP paints the red failure card, so red
ink (the Tier C detector, reused unchanged) is the assertion, and each frame is diffed against the old
binary's screenshot of the same file at the same path.

| case | MAP, banner at char | old binary | runs | red ink | differs from old binary | verdict |
| --- | --- | --- | --- | --- | --- | --- |
| real | `themis-tec-at32.map`, 1,162,057 | refused `ERR-MAP-3001` | 3 | 0 | yes, all 3 | PASS |
| m50 | `big_m50.map`, 491,743 | refused `ERR-MAP-3001` | 1 | 0 | yes | PASS |
| m10 | `big_m10.map`, 609 | accepted | 1 | 0 | **no — byte-identical** | PASS |

**5 / 5.** The m10 row is the safety check that matters: the case that already worked now renders a
frame indistinguishable from the old binary's, so widening the GNU search changed nothing for a MAP
that never needed it.

The owner's real project now analyzes on the shipping binary with its MAP attached:
`themis-tec-at32.elf`, 2,408,804 bytes, `MAP: themis-tec-at32.map`, Memory Nonvolatile / load image
**Exact 170,756 bytes**, Runtime RAM **Exact 32,172 bytes**. The same file produced
`ERR-MAP-3001 / no GNU ld memory-map banner was found` in the round that recorded the finding.

## Error recovery around the affected flows (§16 E)

Covered by the two runs above rather than a third script: a failed analysis after a successful one keeps
the last good report and says so (F001 run D, 3 of 3), and a refused destination leaves a working path —
choose another folder, Export re-enables, the bundle writes (F002 case 3 recovery, 3 of 3).

## Harness corrections made during this phase

Four detector bugs were found and fixed while the product was already behaving correctly. They are
listed because a harness that invents a FAIL is as dangerous as one that hides one.

1. `rv_f002.row()` filtered accent blobs with `r[6] > 200`; index 6 is a 0..1 fill ratio, so every
   button was discarded and the bundle row was reported missing.
2. `rv_f002.row()` anchored on "the leftmost accent below y=400", assuming where the page was scrolled.
   Once the error card and preview render, the row moves to the top and the anchor landed on the
   confirmation card's own Replace button — reporting a truthful screen as having no card. Prepare is
   now identified by its shape, independent of scroll.
3. `rv_f001` derived its measurement band from the A/B difference box; inserting the note pushes every
   report row down, so the box covered the whole page and "empty before" could never be true. Replaced
   by the note-card colour detector, which was calibrated on the frames and then held identical values
   across runs.
4. `rv_f003` called the native picker without clicking the button that opens it, and both destination
   choices in `rv_f002` wrote the same frame filename, so the foreign case's evidence was overwritten by
   the recovery case. Both fixed before the runs recorded above.

`HARNESS-INCIDENT-16` (external evidence root) records a `WM_CLOSE` that went to a third-party window
because the harness selected windows by class name. The application was unaffected — its process kept
running — and the selector now filters on the owning image name.
