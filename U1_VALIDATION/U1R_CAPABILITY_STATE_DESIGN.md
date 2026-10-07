---
title: "U1R capability-state corrective: design and evidence note"
doc_id: "FS-U1R-001"
product: "FirmwareSight"
stage: "U1R (corrective inside the U1 UI productization track)"
related:
  - U1_VALIDATION/U1_VISUAL_ACCEPTANCE_REPORT.md # U1-V2-06 is recorded there
  - U1_VALIDATION/U1_VALIDATION_REPORT.md
---

# FS-U1R-001 — what changed on the Analyze page and why

Written before the installed recheck, so the code contract is on the record while it is still possible to
say what was intended rather than describe the diff afterwards. Canonical unit
`U1R_STALE_CAPABILITY_CORRECTIVE_AND_FINAL_RECHECK`, prompt archived at
`10_AUDIT/SOURCE_PROMPTS/FirmwareSight_U1R_Stale_Capability_Corrective_Final_Recheck_v1.0.txt`.

## 1. The defect (prompt §4, verified in the tree before editing)

`apps/desktop/ui/src/Analyze.tsx` resolves its visible summary with `const summary = lastGood;`.
`lastGood` only moves when the shell returns a summary, which is the correct product behavior: a failed
attempt must not erase the evidence the reader already had. The defect was one level up. The top strip
rendered `summary.capabilities.elf / .map / .git` whenever `summary !== null`, with no conditioning on
whether that summary belonged to the current attempt.

The reachable wrong read: analyze `a.elf`, then select `b.elf` whose MAP is absent, then let the analysis
of `b.elf` fail. The page then showed a green "ELF supported / MAP provided / Git unknown" strip taken from
`a.elf` above a row naming `b.elf` and an error panel saying `MAP: Not provided`. Two facts on one screen,
describing two different files, with the older one dressed as the current one. Recorded as U1-V2-06 in
`U1_VISUAL_ACCEPTANCE_REPORT.md`.

The prompt's own warning was honored: this is **not** fixed by deleting last-good retention. Retention stays
exactly as it was; what changed is which of the two facts the top of the page is allowed to speak for.

## 2. The contract (prompt §5 A–E)

| Case | Required read | Shipped |
| --- | --- | --- |
| A current successful analysis | capability chips from the current summary | three `StateBadge variant="chip"` from `currentSummary.capabilities.*` |
| B no analysis yet | neutral input-ready presentation | `No artifact selected yet` |
| C selection changed, not analyzed | not the last-good chips; neutral current-context | `<file> selected, not analyzed yet`, plus `Previous result retained below` |
| D current analysis failed | not the last-good chips; neutral/error-context | `Current analysis failed`, plus `Previous result retained below` |
| E retained report and Details | stay visible and stay explicitly labelled | unchanged `Last good analysis` region note, plus Details now named for the snapshot it belongs to |

The wording follows the visual language already on the page (`Chip` text is a plain fact sentence; the report
already says "Previous analysis of …"). No new domain state is introduced, nothing is mapped onto
PASS / REVIEW / BLOCK / UNKNOWN / N/A that was not already mapped, and the strip is presentation state, not a
Gate verdict.

## 3. Implementation shape (prompt §6)

One derived boolean-valued read, no duplicated domain logic, no mutation of the stored value:

```tsx
const currentSummary =
  summary !== null && error === null && !pendingSelection ? summary : null;
```

`currentSummary` is read by the header meta line and the capability strip. `summary` continues to be read by
the retained report and by `Details`, so nothing a reader can still see was rewritten. `pendingSelection` and
`error` are the shell's own state as already computed on this page; the derivation adds no IPC call, no
backend read, no storage, schema or wire DTO change, and clears no history.

`Details` gained one optional prop (`stale`, default `false`) used only to name its region:
`Snapshot details of the last good analysis` when the current attempt is not authoritative,
`Snapshot details` otherwise. `Analyze` passes `stale={error !== null || pendingSelection}` — the same
condition the report region already uses for its `Last good analysis` label, so one screen cannot call the
same snapshot current in one place and previous in another.

## 4. Tests (prompt §7)

Added to `apps/desktop/ui/src/intake.test.tsx` under
`describe('U1R capability strip: current attempt, not retained evidence')`, all six built on the existing
`selection()` / `summary()` / `chooseArtifact()` / `analyze()` helpers so they drive the same IPC stubs the
P1-A0 suite already drives:

- **T1** — after a success, the strip states the current capability words (`ELF supported`, `MAP provided`,
  `Git unknown`) and the header carries the current figures.
- **T2** — selecting a second artifact without analyzing it replaces those words with
  `other.elf selected, not analyzed yet`; the test asserts the *absence* of every current-attempt capability
  word, not a pixel.
- **T3** — a failed attempt shows `Current analysis failed` and `Previous result retained below`, and no
  borrowed capability chip appears anywhere in the strip.
- **T4** — the retained report and `Details` stay visible and keep their Previous / last-good names.
- **T5** — the failure that actually happened is still reported verbatim (`ERR-PARSE-2002` and its own
  sentence); the corrective does not soften an error into a status pill.
- **T6** — a later success returns the current words again, so the strip is not a one-way latch.

Full suite after the change: **242 passed in 9 files** (236 before, six added, none deleted), typecheck,
lint and build each exit 0. Rust stays **868 passed / 0 failed** in 47 result lines — no Rust product source
was touched, so the prompt's STOP condition on a Rust change was never reached.

### Mutation proof

To show the tests bite rather than merely pass, the derivation was reverted to the shipped-bug form
(`const currentSummary = summary;`) with the tests left untouched, and `intake.test.tsx` re-run:

```text
❯ U1R capability strip: current attempt, not retained evidence (6)
  ✓ T1  × T2  × T3  ✓ T4  ✓ T5  × T6
Tests  3 failed | 26 passed (29)
```

T2, T3 and T6 go red under the mutation and green under the fix. T1, T4 and T5 staying green is the expected
shape: T1 and T4 assert behavior the bug also got right, and T5 asserts the error panel, which the mutation
does not touch. A test that cannot fail is not a guard, so those three reds are the evidence that the
contract is enforced. The mutation was then undone by digest, not by hand:

```text
$ sha256sum -c target/u1r_pre_mutation.txt
Analyze.tsx: OK
Details.tsx: OK
intake.test.tsx: OK
$ grep -c MUTATION-PROOF Analyze.tsx Details.tsx intake.test.tsx
0 / 0 / 0
```

The mutation code and the harness logs live under `target/` (gitignored) and are not part of the commit.

## 5. Scope record (prompt §10, §11)

Changed product paths — all three inside the UI layer and all on the Analyze page:

- `apps/desktop/ui/src/Analyze.tsx` (the derivation, the strip, the header meta, the `stale` handoff)
- `apps/desktop/ui/src/Details.tsx` (one optional `stale` prop used only for the region's accessible name)
- `apps/desktop/ui/src/intake.test.tsx` (T1–T6)

Not changed, deliberately: Overview / Compare / Release / History layout, design tokens, Core, storage,
schema, migrations, wire DTOs, release identity, Gate semantics, bundle semantics, V1 metric definitions.
`assets/design-tokens.json` is byte-identical: the blob digest at HEAD and the worktree digest both measure
`94336906997d23b1f36cbaf81711c6824082d88b18aa89c38dd12721564a14d4`, and
`git diff --quiet HEAD -- assets/design-tokens.json` exits 0. No new token value, no new component, no new
CSS — `Chip` and `StateBadge variant="chip"` were already the page's vocabulary.

The seven U1-V1 minor visual gaps stay `OPEN_FOR_ARCHITECT_VISUAL_JUDGEMENT`. U1R closes U1-V2-06 only.
