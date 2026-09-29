---
title: "P2 Compare Desktop Smoke Report"
doc_id: "FS-P2-SMOKE-DESKTOP"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P2_COMPARE"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P2 Compare — shipped Windows desktop smoke (prompt §54)

Smoke of the shipping binary against a clean validation database, driven through real Win32 input
and read back from real window captures. Every observation below is something this report's author
saw in a screenshot of the running app or computed from a file the running app wrote; nothing is
inferred from the tests.

## Run under test

| | |
| --- | --- |
| Binary | `target/release/firmwaresight-desktop.exe` (workspace-root target dir) |
| Build | `cargo build --release --features custom-protocol` over `ui/dist` from `vite build` |
| Run 1 | pid 19236, window 2426754, binary of 12:43 — pre-fix, three defects found |
| Run 2 | pid 12648, window 3868612, sha256 `f6f9db40c2e597a9bf827f17fd86b812f50a180dc3d296cb19f2d63a308288ca` — post-fix re-verification |
| Window | 1015 × 768 client, default size, no scaling applied |
| Database | clean validation DB; the pre-existing DB was moved aside and restored afterwards (see "Database handling") |

`cargo build --release` **without** `--features custom-protocol` produces a binary that loads
`http://localhost:5173` and shows a WebView2 "cannot reach this page" error. That is the
`custom-protocol` feature gate working as designed, and it is recorded here because the smoke must
be run against a genuinely shipping build, not a dev build.

## Inputs

Hashes are from `sha256sum` on the staged files, not from the screen.

| Side | File | Bytes | SHA-256 |
| --- | --- | --- | --- |
| base | `firmware.elf` | 16,820 | `3f615b6247179d930f59b76dcdbaea1a5eca8c835ab6d410feb20a062f33f21f` |
| base | `firmware.map` | 4,511 | `832690060a8d25f8acacd85a62ce76bc83dfb6768e435bcc04c51f9664ed363e` |
| target | `firmware.elf` | 17,464 | `4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374` |
| target | `firmware.map` | 4,664 | `a38575cd51ec2730442a0f3b8506c99fc49eec68d4b28ecdc9dda9434709ace6` |

Both MAP hashes reappear unchanged inside the snapshot ids the app stored, so the MAP the app
recorded is the MAP that was attached.

## The 30 steps

| # | Step | Result | What was actually seen |
| --- | --- | --- | --- |
| 1 | launch | PASS | Window titled `FirmwareSight - Analyze`, rail lists Analyze and Compare |
| 2 | Analyze base ELF + MAP | PASS | SHA-256, 16,820 bytes, `Arm 32-bit little`, `object-elf/0.40`, Entry `0x080000049`, Nonvolatile `Exact 256 bytes`, Runtime RAM `Exact 8 bytes`, `MAP: firmware.map` |
| 3 | Analyze target ELF + MAP | PASS | SHA-256 `4b4087e1…`, 17,464 bytes, Nonvolatile `Exact 376 bytes`, Runtime RAM `Exact 76 bytes`, Sections 19, Symbols 52, Evidence `11 recorded — 11 observed, 0 derived, 0 declared, 0 unknown`, MAP `provided`, Git `unknown` |
| 4 | open Compare | PASS | Heading `Compare`, subhead states it reads stored facts and never re-reads the files |
| 5 | both snapshots appear | PASS | Both pickers list both builds; `Showing 1 to 18 of 18 sections` and `1 to 76 of 76 symbols` follow from them |
| 6 | Old/Base = base | PASS | `firmware.elf · 3f615b624717`, 16,820 bytes, Nonvolatile exact 256, imported `2026-09-29T19:54:54Z` |
| 7 | New/Target = target | PASS | `firmware.elf · 4b4087e1407c · last analyzed` — the build Analyze had just proved is preferred, and says so |
| 8 | run Compare | PASS | Report renders; no error panel |
| 9 | nonvolatile old/new/delta | PASS | `Old 256 bytes` `New 376 bytes` `Delta +120 bytes`; JSON export carries `base 256, target 376, delta 120, comparability exact` |
| 10 | runtime RAM old/new/delta | PASS | `Old 8 bytes` `New 76 bytes` `Delta +68 bytes` |
| 11 | signs independent | PASS | `.debug_abbrev +48`, `.text +88`, `.shstrtab -2` in one column; `Delta file 0` and `Delta RAM +221` differ on the same row |
| 12 | Section Added/Removed/Changed | PASS | `Added 1` `.ota`, `Removed 1` `.calib`, `Changed 16`, `2 unchanged` |
| 13 | Symbol Added/Removed/Changed | PASS | `Added 38`, `Removed 33`, `Changed 5`, `9 unchanged`, `67 unpaired` |
| 14 | no fabricated zero | PASS | `.calib` New file reads `Not present` with `Delta Unknown` and the reason `present only in the base build: a removal…`; `.ota` Old file reads `Not present`. Largest additions lists `unnamed symbol 9 … Unknown`, never `0` |
| 15 | click top growth contributor | PASS | Clicking `.debug_loclists` filters Section Changes to `Showing 1 to 1 of 1 sections` and writes the name into the filter field |
| 16 | full list still available | PASS | Clearing the filter restores `Showing 1 to 18 of 18 sections` with all 18 rows |
| 17 | Bytes → KiB | PASS | Radio group `Size units`, KiB selects and every size re-renders |
| 18 | /1024 only | PASS | 256→`0.250 KiB`, 376→`0.367`, +120→`+0.117`, 8→`0.00781`, 76→`0.0742`, +68→`+0.0664`; `.ota` 64→`0.0625 KiB`; ranking `+2 bytes` ↔ `+0.00195 KiB` |
| 19 | addresses unchanged | PASS | The eight `$d` rows read `0x080000…`/`0x200000…`/`0x080100…` identically in Bytes and in KiB for the same pair direction |
| 20 | evidence quality / MAP status | PASS | `Base evidence` and `Target evidence` each show `Linker MAP used`, `layout map`, `weakest basis MapRegionAndElfLoad`, `a footprint row was recorded`; `Pair comparability exact` with the reason that the weaker side caps the pair |
| 21 | object attribution honest | PASS | `Object attribution — unavailable` with the full reason; no object or module table is invented. (Run 1 rendered the word broken across three lines — defect A) |
| 22 | Export JSON, native dialog | PASS | OS dialog titled `Export the build diff as JSON`, default name `firmwaresight-diff-3f615b62-4b4087e1.json`, filter `FirmwareSight json export (*.json)` |
| 23 | JSON validates, no host path | PASS | 64,355 bytes; `jsonschema` 4.26.0 against `urn:firmwaresight:schema:diff:1` → **0 errors**; regex for drive/`/home`/UNC paths → **0 matches**; `<script` → 0; no timestamp field |
| 24 | Export HTML, native dialog | PASS | Dialog `Export the build diff as HTML`, default name `…-4b4087e1.html`, filter `FirmwareSight html export (*.html)`; wrote 41,953 bytes |
| 25 | HTML opens independently | PASS | Opened as a standalone browser window titled `FirmwareSight build diff` — the file's own `<title>`, so it parsed and rendered from disk. Structurally: 1 inline `<style>`, 0 external `src`/`href`, 0 `@import`, 0 `url(`, 0 `<script>`, 0 host paths, no unclosed tags |
| 26 | cancel export → no error | PASS | `Export cancelled. No file was written.` — plain note, no alert role, no red |
| 27 | induce invalid compare attempt | PARTIAL | The same-pair lock could not be reached: see "Not verified". The equivalent standing-report case *was* reached through Swap and produced defect C |
| 28 | last-good remains | PASS | After Swap the Memory block still showed the earlier comparison (Old 256 → New 376) while the pickers named the opposite pair — the diff is never silently replaced |
| 29 | no path leak | PASS | Every on-screen export note names the file only (`Wrote firmwaresight-diff-3f615b62-4b4087e1.html (HTML).`); the footer carries content-derived snapshot ids and truncated hashes; the OS `Replace existing file?` dialog text also names the file only |
| 30 | clean close | PASS | Close button → window gone, `tasklist` reports no `firmwaresight-desktop.exe` for either run |

Extra, not in the step list: the app's own overwrite guard was exercised live. Saving over the
existing HTML raised `Replace existing file?` — `firmwaresight-diff-3f615b62-4b4087e1.html already
exists. Replacing it overwrites the file you have.` — and answering `Keep mine` produced `Kept the
file that was already there. Nothing was written.` with the file's sha256 and mtime unchanged
(`aa4b08df1ddbff0764e636bda255828fb90966efd0fa6895380a53c3aabab52e`, `13:20:19`).

## Reversal, re-checked in Run 2

Swapping the pair and pressing Compare again was used to confirm Core's reversal invariant in the
shipped binary rather than only in tests:

| | forward | swapped |
| --- | --- | --- |
| Nonvolatile | 256 → 376, `+120` | 376 → 256, `-120` |
| Runtime RAM | 8 → 76, `+68` | 76 → 8, `-68` |
| Sections | Added 1 / Removed 1 / Changed 16 | Added 1 / Removed 1 / Changed 16 |
| Symbols | Added 38 / Removed 33 / Changed 5 | Added 33 / Removed 38 / Changed 5 |
| `.debug_loclists` | 193 → 414, `+221` | 414 → 193, `-221` |
| Largest additions | `.ota` 64 bytes | `.calib` 32 bytes |

`SYMBOL-AMBIGUOUS: 67` is unchanged in both directions, as it must be.

**D — the CLI and the desktop disagreed about one field of the same portable document.**
Cross-checking the JSON this smoke exported against `fwsight diff --json` for the identical pair
showed every field equal except `base.memory.evidence.layoutSource` and
`target.memory.evidence.layoutSource`: the desktop said `map`, the CLI said `MapMemoryConfiguration`.
Cause: `diff.rs` built the label with `format!("{:?}", …)`, leaking a Rust enum name into a document
whose own schema describes the field as the *persisted* label. The desktop was right — it reads the
value back out of the database. Fix: `LayoutSource::as_label()` in Core, used by the diff, by the
database write and by the report DTO (which removes two duplicate copies of the mapping), plus
`apps/cli/tests/p2_golden.rs::the_layout_label_is_the_one_the_storage_layer_persists`, which pins the
document to Core's vocabulary instead of to a copied string.

This changed two committed goldens, through `python scripts/update_goldens.py --confirm`, which
reported exactly four lines moving and the two P0 goldens unchanged:

```
~ base.memory.evidence.layoutSource:  MapMemoryConfiguration -> map
~ target.memory.evidence.layoutSource: MapMemoryConfiguration -> map
-<dt>Layout</dt><dd>MapMemoryConfiguration &middot; MapRegionAndElfLoad</dd>
+<dt>Layout</dt><dd>map &middot; MapRegionAndElfLoad</dd>            (x2)
```

The goldens were regenerated because the code was wrong, not to make a gate pass: the desktop export
captured in this same smoke already read `map · MapRegionAndElfLoad`, so the golden was the outlier.
After the fix both renderers produce it.

## Defects found by this smoke, and what was done

All four were visible only by running the product; A, B and C were invisible to the jsdom suite and D
to the Rust suite, because each side was individually self-consistent.

**A — a state label was squeezed to four characters per line.** `Object attribution` rendered
`unav` / `ailabl` / `e`. Cause: `.value` sets `overflow-wrap: anywhere`, which is inherited and
reduces the state label's min-content width to one character, so the long note beside it took the
space. Fix: `StateBadge .label { flex: none }` — icon plus label is the state and must not be the
thing that gives way. Re-verified in Run 2: `— unavailable` on one line, note wrapping to its right.

**B — the navigation rail scrolled away.** `.rail` was `position: sticky` *and* `align-self:
stretch`, so its box was already as tall as the page and could not stick; the brand and the two page
buttons left the screen with the text. Fix: the rail box keeps the full-height surface and hairline,
and a `.railSticky` group inside it carries `position: sticky; top: var(--fs-space-6)`. Re-verified
in Run 2: Analyze and Compare stay visible at 1,000 px of scroll.

**C — the standing report did not say it was standing.** After Swap the pickers named the reversed
pair while the Memory block still answered for the earlier one, with nothing on screen marking that
difference. The note that would have said it existed only on a failed attempt, and named builds by
file name alone — which for two builds of one artifact is not a name at all: it would have read
"not a comparison of firmware.elf and firmware.elf". Fix: a `buildLabel` helper (file name plus the
same 12-hex-digit prefix the pickers use) and `stale` extended to "the report is not the selected
pair". Re-verified in Run 2: *This is the comparison of firmware.elf · 3f615b624717 and
firmware.elf · 4b4087e1407c that you asked for. The pair selected above is a different one; press
Compare to move to it.*

Fixes: A in `StateBadge.module.css`; B in `App.tsx` + `App.module.css`; C in `Compare.tsx` +
`compare.test.tsx`; D in `firmwaresight-core` (`memory.rs`, `diff.rs`), `firmwaresight-storage`
(`db.rs`), `firmwaresight-report` (`dto.rs`, `diff_schema_contract.rs`), `apps/cli` (`p2_golden.rs`)
and the two P2 goldens.

Frontend gates after A–C: `tsc --noEmit` clean, `eslint .` clean, `vitest run` 99/99, `vite build`
clean. Rust gates after D: `cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets
--all-features -- -D warnings` clean, `cargo test --workspace` 345 passed / 0 failed.

The desktop binary was rebuilt from the final tree. It was not re-smoked, because D cannot reach the
desktop path at all: the desktop reads `layout_source` out of the database, which is why its export in
this very smoke already carried the correct `map`.

## Not verified

**Step 27, same-pair lock, in the shipped window.** Reaching it requires choosing the build the
other side already names, which means operating a native `<select>` popup. That popup is not
captured by the window-capture path available here (WGC returns a black frame, so capture falls back
to GDI `PrintWindow`, which excludes it), and synthetic `Down`, `Return` and type-ahead keys did not
change the value while the document itself kept receiving keys. The behaviour is covered by
`compare.test.tsx` ("refuses a pair of one build: the action locks and the screen says why") and by
the IPC tests, and the note's wording was corrected as part of defect C — but it was **not observed
in the running app**, and this report does not claim it was.

## Database handling

- Before the smoke the existing DB was copied to `%LOCALAPPDATA%\Temp\p2-smoke\db-backup` and moved
  aside in place as `firmwaresight-p0.sqlite.pre-p2-smoke`.
- The smoke ran against a fresh DB at `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite`.
- After the smoke that DB was renamed to `firmwaresight-p0.sqlite.p2-smoke` (with `-wal`/`-shm`) and
  kept for inspection; the original was restored.
- Restored file: 114,688 bytes, sha256
  `ec845c8a840453af077f611ddda21d4a4d65ab11e4c651d67673e514bc08479e` — identical to the pre-smoke
  record.
- `firmwaresight-p0.sqlite.p0-smoke-history` and its `-wal`/`-shm` were already present from an
  earlier session's smoke. They were left untouched; removing them is a deletion and needs a human
  decision.

## Artifacts the smoke produced

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `…\Temp\p2-smoke\export\firmwaresight-diff-3f615b62-4b4087e1.json` | 64,355 | `1930cdb6a098c67456297ec321910315d21aeb3ce79ef7264fcc4037e148242a` |
| `…\Temp\p2-smoke\export\firmwaresight-diff-3f615b62-4b4087e1.html` | 41,953 | `aa4b08df1ddbff0764e636bda255828fb90966efd0fa6895380a53c3aabab52e` |

Both live under the OS temp directory, outside the repository, and are not part of the frozen tree.
