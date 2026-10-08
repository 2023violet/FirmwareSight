# U1P-R3 narrow corrective plan — written before any product code

Canonical unit: `U1P_R3_FINAL_NARROW_CORRECTIVE_AND_VISUAL_REVIEW`.
Start authority: HEAD = origin/main = remote main = `f01eec110c76591c741d283e4ee862566e211a2f`, clean tree, one
worktree, one branch (measured in §1 of the prompt's command set, not assumed).
Prompt: 32,849 bytes, sha256 `a185890edd043344a4332dfcc048fd63add1f8a5226dead4af373cc1ceab4de9`, stored blob
`384e8ca42db8d8238d263c18111d58e5d8da1ab1` — the same digest, because it arrived LF-only (745 LF, 0 CR).

This is a plan, not a report: every claim about current behaviour below was read out of the source at `f01eec1`
before it was written, with the line it came from.

## 1. Why a comparison disappears — the exact files and the React lifecycle

Three facts, each verified in the tree:

1. `Compare.tsx:104-116` keeps **all** of the comparison in component-local state: `candidates`, `baseId`,
   `targetId`, `summary`, `error`, `comparing`, `sectionFocus`, `symbolFocus`, `additions`, and the three export
   fields.
2. `App.tsx:183-191` renders `<Compare/>` only while `page === 'compare'`. The other five pages are in the same
   ternary chain, so switching pages **unmounts** Compare and React discards every one of those hooks.
3. `Compare.tsx:408` gates the whole report on `summary === null`, and the only `EmptyState` on the page
   (`Compare.tsx:323-331`) is reached when `!hasTwoBuilds`. So with two or more stored builds and no summary —
   the state every re-entry lands in — the page renders header, pickers, unit switch, and then **nothing**.
   That is the blank area in `REG_Compare_1440x900.png`, and it is a rendering gap, not a data gap: the shell
   still holds the diff.

`App.tsx:117-126` already proves the pattern this round needs: five facts live in the shell precisely because
they must outlive a page switch, and the file's own header comment says so ("what Analyze currently holds, so
navigating to Compare and back does not make the session forget the build it just analyzed"). The comparison is
the sixth such fact and was simply never lifted.

## 2. What `diffId` is, and what it therefore may not survive

`Compare.tsx:473-475` states it on screen: *"The diff id is this session's handle for the computed comparison:
not stored, not portable, and not part of what the diff proves."* Every downstream read —
`querySectionChanges` / `querySymbolChanges` (`Compare.tsx:229-247`), `exportCompareJson` / `exportCompareHtml`
(`Compare.tsx:271-272`) — takes that handle and nothing else.

So the handle is an in-process registry key. It survives a page switch because the process survives; it does not
survive a process exit, and there is no store to read it back from. That is the boundary §5 draws, and it is why
the retention below is **state lifted, not state persisted**: no SQLite, no IPC, no `localStorage`, no new
schema, no changed DTO. A new process starts with `App`'s initial state, which is `null`, so scenario C5 is
satisfied by construction rather than by a guard.

## 3. Selected pair versus computed pair — the truthfulness rule that already exists

`Compare.tsx:296-300` computes `stale` as "a summary exists, and either the last attempt failed or the selected
pair no longer matches the pair the summary was computed from", and `Compare.tsx:411-419` labels the report with
the two build identities it actually answers for. This round keeps that rule untouched and inherits its wording
for the retained case: after a return to the page, the pair and the result come back **together**, so `stale`
stays false when they still match and becomes true the moment the reader moves a selector — which is exactly
scenario C3, and it needs no new state machine.

## 4. The minimal retention architecture

One typed object, owned by the shell, holding only what must outlive a page:

```
type ComparisonSession = {
  readonly baseId: string | null;
  readonly targetId: string | null;
  readonly summary: CompareSummaryDto | null;
};
```

- `App.tsx` adds `const [comparison, setComparison] = useState<ComparisonSession>(
  { baseId: null, targetId: null, summary: null })` beside the five existing session facts, and passes
  `comparison` + `onComparisonChange={setComparison}` to `Compare`.
- `Compare` becomes **controlled** for those three values: it reads `comparison.baseId` / `.targetId` /
  `.summary` and writes through one callback that spreads the rest. No mirror state, so nothing can drift
  between a copy in the page and a copy in the shell.
- Everything genuinely transient stays local: `comparing`, `error`, the two focus cursors, the export trio, and
  `candidates` (which is re-read on mount, because a build analyzed while the page was away must appear).
- The default-pair effect (`Compare.tsx:149-167`) keeps its "choose, never compare" behaviour and now writes the
  chosen pair into the session; it still must not run when a pair is already named, which is what the
  `baseId !== null || targetId !== null` guard already does — that guard now also protects the restored session
  from being overwritten on re-entry.
- `compare()` writes the summary **through the shell**, not only into local state, so a comparison that finishes
  while the reader is on another page still lands instead of being dropped by an unmounted setter (scenario C6).

Rejected alternatives, with the reason: keeping the page mounted and hidden (it would keep polling and would
render an invisible region to assistive technology unless proven otherwise — §5 refuses that without a
lifecycle proof); a context/provider (a second mechanism for one fact that already has an owner); anything
persisted (forbidden, and false to the handle's nature).

## 5. The two-candidate no-result state

When `candidates` has loaded, `hasTwoBuilds` is true and there is no summary and no attempt in flight, the page
gets a purposeful region rather than a void: a heading **"Ready to compare"** and the sentence **"Choose Old /
Base and New / Target, then select Compare."** It names no numbers, claims no delta, asserts no verdict, and
sits where the blank space was, below the selectors, so the reader's eye reaches it after choosing. The
fewer-than-two path keeps its existing `Nothing to compare yet` flow unchanged.

## 6. Real data above the fold, without hiding the evidence

The first viewport at 1440×900 must carry the pair identity, a compact summary, and at least the first section
change rows. The measured cost today, from the CSS: `PageHeader` subhead is a three-line paragraph
(`Compare.tsx:306`), `.selectors` (`Compare.module.css:18-25`) spends a `--fs-space-4` gap plus the same in
`padding-bottom` above a hairline, `SizeUnitSwitch` takes its own full-width row (`Compare.tsx:406`), and
`MemoryComparison` opens with its two side-evidence rows before the table appears.

Three composition moves, each preserving every word's meaning somewhere on the page (a fourth was drafted
and is cancelled at the end of this section):

1. the page subhead drops to one sentence, and the sentence it loses — that a comparison re-reads nothing and
   writes nothing — moves into the report's own footer note, where the handle caveat already lives;
2. `.selectors` tightens to `--fs-space-3` gap and padding, using tokens that already exist;
3. `SizeUnitSwitch` joins the summary heading row instead of occupying a row of its own, since the unit governs
   the figures beside it.

No type size changes, no new tokens, no animation, no removal of any caveat, and no invented preview data. If
the installed capture still shows no section rows above the fold after this, the round iterates **before** the
captures, because §12 forbids product edits after them.

A fourth move was drafted and **cancelled before implementation**: relocating `MemoryComparison`'s base/target
evidence rows below the table. `compare.test.tsx` asserts seven `weakest basis …` strings *inside* the region
named `Memory comparison` (lines 745-791), so the move would have broken accepted tests for a layout reason, and
§9 forbids weakening or deleting a test to reach green. The evidence rows stay where they are.

## 7. The 1024 capability band, fixed locally

`Panel.module.css:82-99` gives every band `repeat(auto-fit, minmax(var(--fs-layout-evidence-inspector-min), 1fr))`
— 320 px per cell (`tokens.css:60`). At a 1024 px window the workspace column is roughly
`1024 − 224 (rail) − padding`, which fits **two** 320 px tracks, so the third cell wraps and the container's
grey background shows through the unfilled track: the "missing fourth capability" in
`R2_R01_Matched_1024x720.png`.

The fix is a local Overview wrapper class that restyles only the band it contains, switching that band from a
grid to a wrapping flex row with `flex: 1 1 var(--fs-layout-evidence-inspector-min)` on the cells. A grid leaves
the last row's remainder empty; a flex line **grows the items on the final line to fill it**, which is the whole
defect. Consequences by width, from the same token: at 1440 all three cells still sit on one balanced row; at
1024 and 1056 the layout is 2 + 1 with Git spanning the remainder; at any width there is no placeholder, no
fourth item, and no empty track. `components/Panel.module.css` is not touched, so Analyze, Compare, Release and
History keep their bands exactly as they are — including the metric rows §7 names as the risk.

## 8. Before and after, and what the screenshots must show

| Contract | Before (already in the R2 pack) | After (this round's capture) |
| --- | --- | --- |
| two candidates, no comparison | blank below the selectors | `P03_Compare_Ready_1440x900.png` shows "Ready to compare" + usable Compare |
| a real comparison | section table below the fold | `P04_Compare_Result_1440x900.png` shows identity, summary and section rows in the first viewport |
| away and back | result gone | `P05_Compare_AfterNavigation_1440x900.png` shows the same real result, same base/target |
| Overview at 1024 | grey void beside Git | `R01_Overview_Matched_1024x720.png` shows three cells, none blank |
| Overview at 1056 | same void risk | `R03_Overview_Matched_1056x799.png` |
| pending selection (must not regress) | `R2_S03_PendingSelection_1440x900.png` | `P09_Overview_PendingSelection_1440x900.png` |
| Gate mismatch (must not regress) | `R2_S02_OtherBuild_1440x900.png` | `P10_Overview_GateMismatch_1440x900.png` |
| parse failure (must not regress) | `P06_ParseFailure_1440x900.png` | `P08_ParseFailure_1440x900.png` |

## 9. What stays reference-only

The mockups' right-hand "Biggest growth contributors" panel exists in the product as the `Ranking` block, which
is a capped list beside the tables and is **not** moved into a new column; `FS-UI-07` stays
`REFERENCE_ONLY_NOT_IMPLEMENTED`; the references' icon rail, theme toggle, "Recent activity" entry and
`LARGEST SECTION` / budget-headroom figures stay unimplemented, because there is no real source in this build
that would fill them and §6 forbids invented numbers.

## 10. Tests that must fail before the code exists

`compare.test.tsx` renders the whole `<App/>` against a mocked bridge, which is what makes cross-navigation
testable at all. T1 (two candidates, no summary → the ready state), T3 (compare → navigate → return → the real
result is still on screen), T4 (its base/target and the handle the tables query are still the computed pair),
T7 (a fresh mount starts with no phantom comparison) and T8 (an obsolete response cannot overwrite a newer pair)
all fail against `f01eec1` by construction: today the report simply is not there after a return. T2, T9, T10,
T11 and T13 are regression locks on behaviour that must not move, and are expected to pass before and after.
T12 asserts the *layout contract* — that the capability band renders exactly three cells and that the wrapper
class is the flex variant — and says plainly in its name that jsdom measures no pixels; the blank-cell question
is answered by the installed screenshot, not by a test.

## 11. Contradiction pass over this plan

- §4 lifts state; §5 keeps `candidates` local. Not a conflict: the list is re-read and can legitimately change,
  while the comparison must not be recomputed on its own.
- §3's `stale` rule and §4's controlled pair could fight if the default-pair effect ran on re-entry and
  re-pointed the selectors away from the restored pair. It cannot: the effect returns early when a pair is
  already named, and after a restore both ids are named. This is exactly what T3 and T4 pin down.
- §6's fourth drafted move did collide: seven accepted assertions read the basis strings inside the
  `Memory comparison` region, so that move is cancelled (see §6) rather than the tests being loosened.
- §7's flex change must not be applied to the shared band, and the metric band on the same page is deliberately
  left alone until a capture shows whether it has the same void — if it does, it is reported, not silently
  widened in the same commit.
- Nothing here needs a Rust command, a DTO change, a schema, a migration or a dependency. If implementing it
  discovers otherwise, §9's stop applies and the round returns the blocker instead of the feature.

## 12. What changed while implementing this plan (written after the code, not before)

The plan above is the one written before any product file was touched, and it is kept verbatim. Three things
turned out differently, and each is recorded here rather than edited backwards into §1-§11:

1. **§6's "four composition moves" was already three**, because the fourth (relocating `MemoryComparison`'s
   side-evidence rows) was cancelled before implementation for the reason §6 gives. The heading counted the
   draft, the list counted the plan; the sentence now reads as three moves plus one cancellation, and only
   those three are in the commit.
2. **§6.3's unit switch keeps a row of its own while there is no result.** The plan said it joins the summary
   heading row, and it does - once a summary exists. With none, the figures on screen are the two selectors'
   recorded sizes, so the switch stays under them; hiding it until a comparison exists would take a control
   away from the reader who is still choosing. Exactly one switch is on screen in either state, asserted in
   `compare.test.tsx`'s accessibility block.
3. **The §8 copy audit found a fourth defect, in a sentence this plan never mentioned.** The Overview Git cell
   read "Judged by the `git.clean` rule, from the repository this build came from." Neither half is what the
   source says: `capabilities.git` is left at Core's default by the analysis pipeline
   (`crates/firmwaresight-artifact/src/pipeline.rs` builds every snapshot from `Capabilities::elf_only()`, and
   `with_git()` has no production caller), and the only repository the gate reads is the *opened project's*
   workspace (`apps/desktop/src-tauri/src/release.rs`), which is not "the repository this build came from".
   No safety-critical false Gate PASS exists - `rule_git_clean` passes only on a resolved `dirty == false`
   and dispositions an absent reading through `UNKNOWN` with `Review|Block` only - so §8's STOP does not
   fire, and the corrective action is the wording, which §10 permits as a copy regression on a file this
   round already had to open.

4. **§6's three moves became five, because the fold was measured instead of estimated.** The accepted
   `U1P_BASELINE/P03_Compare_1440x900.png` in the R2 pack is a 1440x900 client capture of the composition this
   round starts from, and reading its text bands off the pixels says where everything actually sits: the
   selector hairline at y=372, the unit-switch row at 405, `What moved` at 451, its two count rows at 484 and
   531, and `Memory` from 600 to about 838 - so the section table began near y=862 and showed nothing above
   the fold. Three moves (subhead, `.selectors`, the unit switch) recover about 81 px of the 900, which leaves
   one row and a half. Two more were made, both of the same kind and neither removing a word: the delta legend
   joins the `Memory` heading row instead of taking a row of its own, and the report's inter-section gap drops
   from `--fs-space-6` to `--fs-space-4`. That is roughly 60 px more, and it is why the section rows land
   inside the first viewport rather than just touching it. The region order the U1P round accepted
   (`compare.test.tsx`'s `names the pair first, then what moved, then what it cost, then the prose`) is
   unchanged - the fold was bought with composition, not by re-sequencing the answer.

5. **A capture-integrity defect in the inherited harness was found in that same image and is fixed for this
   round's captures.** The bottom 33 px of `P03_Compare_1440x900.png` are the Windows taskbar: the grab is
   1440x900 of *screen* area and the window's client rectangle did not lie wholly inside the work area, so a
   first-viewport claim in that file was partly a claim about the shell. Nothing in the older harness checked
   where the window sat. `target/u1p_r3_capture.py` now pins the window to the origin before sizing it, reads
   `SPI_GETWORKAREA`, records `client_bottom_on_screen` and `client_inside_work_area` for every capture, and
   refuses (non-zero, no manifest line marked OK) a grab whose client bottom falls below the work area. The
   historical image is left exactly as it is - it is evidence about the round that produced it, and rewriting
   it is not this round's to do.

`components/Panel.tsx` and `Panel.module.css` were edited first with an opt-in `fill` prop on `Band`, then
reverted to HEAD once §10's risk boundary was re-read: an opt-in prop is provably inert for other pages, but
"provably inert" is a weaker claim than "not touched", and the local `Overview.module.css` wrapper §7 already
specified is the stronger evidence. The reverted files are byte-identical to `f01eec1` (see the digests in
`target/u1p_r3_red.txt`).
